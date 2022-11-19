
use super::fs::{sys_write, sys_close};
use crate::{task::{current_process, current_user_token}, mm::{UserBuffer, translated_byte_buffer}};
use lazy_static::*;
use alloc::{
    collections::BTreeMap,
    sync::Arc
};
use runtime::CoroutineId;
use spin::Mutex;

// key -> r_id, write coroutine can use WRMAP to find the corresponding read coroutine id 
lazy_static! {
    pub static ref WRMAP: Arc<Mutex<BTreeMap<usize, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
}

// tid 表示当前用户进程执行的写协程， rtid 表示对应的读协程
// 向文件中写完之后，应该唤醒对应的 read 协程
pub fn async_sys_write(fd: usize, buf: *const u8, len: usize, key: usize) -> isize {
    sys_write(fd, buf, len);
    sys_close(fd);
    // 向文件中写完数据之后，需要唤醒内核当中的协程，将管道中的数据写到缓冲区中
    if let Some(kernel_cid) = WRMAP.lock().remove(&key) {
        error!("kernel_cid {}", kernel_cid);
        crate::lkm::wake_kernel_future(kernel_cid);
    }
    error!("async_sys_write done");
    0
}

pub fn async_sys_read(fd: usize, buf: *const u8, len: usize, key: usize, cid: usize) -> isize {
    error!("async_sys_read do nothing");
    let token = current_user_token();
    let process = current_process().unwrap();
    let pid = process.pid.0;
    // let task = current_task().unwrap();
    // let inner = task.acquire_inner_lock();
    let inner = process.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        let work = file.aread(UserBuffer::new(translated_byte_buffer(token, buf, len).unwrap()), cid, pid, key);
        crate::lkm::add_coroutine(work, 0);
        0
    } else {
        -1
    }    
}