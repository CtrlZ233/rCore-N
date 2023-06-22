use core::cmp::min;

use crate::fs::{make_pipe, File};
use crate::task::{current_process, current_task, current_user_token};
use crate::{
    mm::{translated_byte_buffer, translated_refmut, UserBuffer},
    // task::find_task,
};
use lazy_static::*;
use alloc::{
    collections::BTreeMap,
    sync::Arc
};
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct AsyncKey {
    pub pid: usize,
    pub key: usize,
}

// key -> r_id, write coroutine can use WRMAP to find the corresponding read coroutine id 
lazy_static! {
    pub static ref WRMAP: Arc<Mutex<BTreeMap<AsyncKey, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
}


pub fn sys_write(fd: usize, buf: *const u8, len: usize, key: usize, pid: usize) -> isize {
    if fd == 3 || fd == 4 || fd == 0 || fd == 1 {
        // debug!("sys_write {} {}", fd, len);
    }
    // debug!("buffer len: {}", len);
    let token = current_user_token();
    let process = current_process().unwrap();
    let inner = process.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        if key == usize::MAX {
            if let Ok(buffers) = translated_byte_buffer(token, buf, len) {
                match file.write(UserBuffer::new(buffers)) {
                    Ok(write_len) => write_len as isize,
                    Err(_) => -2,
                }
            } else {
                -3
            }
        } else {
            
            let work = file.awrite(UserBuffer::new(translated_byte_buffer(token, buf, len).unwrap()), pid, key);
            lib_so::spawn(move || work, 0, 0, lib_so::CoroutineKind::KernSyscall);
            0
        }
    } else {
        -4
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize, key: usize, cid: usize) -> isize {
    if fd == 3 || fd == 4 || fd == 0 || fd == 1 {
        // debug!("sys_read {} {}", fd, len);
    }
    let token = current_user_token();
    let process = current_process().unwrap();
    let pid = process.pid.0;
    let inner = process.acquire_inner_lock();
    // info!("test1: {}", fd);
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        if key == usize::MAX && cid == usize::MAX {
            if let Ok(buffers) = translated_byte_buffer(token, buf, len) {
                match file.read(UserBuffer::new(buffers)) {
                    Ok(read_len) => read_len as isize,
                    Err(_) => -2,
                }
            } else {
                -3
            }
        } else {
            // info!("test2: {}", fd);
            let work = file.aread(UserBuffer::new(translated_byte_buffer(token, buf, len).unwrap()), cid, pid, key);
            lib_so::spawn(move || work, 0, 0, lib_so::CoroutineKind::KernSyscall);
            // info!("test3: {}", fd);
            0
        }
    } else {
        -4
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_process().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_process().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_mailwrite(pid: usize, buf: *mut u8, len: usize) -> isize {
    // let token = current_user_token();
    // if let Some(receive_task) = find_task(pid) {
    //     debug!("find task");
    //     if receive_task.acquire_inner_lock().is_mailbox_full() {
    //         return -1;
    //     } else if len == 0 {
    //         return 0;
    //     }
    //
    //     if let Ok(buffers) = translated_byte_buffer(token, buf, min(len, 256)) {
    //         let socket = receive_task.create_socket();
    //         match socket.write(UserBuffer::new(buffers)) {
    //             Ok(write_len) => write_len as isize,
    //             Err(_) => -1,
    //         }
    //     } else {
    //         -1
    //     }
    // } else {
    //     debug!("not find task");
    //     -1
    // }
    -1
}

pub fn sys_mailread(buf: *mut u8, len: usize) -> isize {
    // let token = current_user_token();
    // let task = current_task().unwrap();
    // debug!(
    //     "mail box empty ? {}",
    //     task.acquire_inner_lock().is_mailbox_empty()
    // );
    // if task.acquire_inner_lock().is_mailbox_empty() {
    //     return -1;
    // } else if len == 0 {
    //     return 0;
    // }
    // let mail_box = task.acquire_inner_lock().mail_box.clone();
    // if let Ok(buffers) = translated_byte_buffer(token, buf, min(len, 256)) {
    //     match mail_box.read(UserBuffer::new(buffers)) {
    //         Ok(read_len) => {
    //             debug!("mail read {} len", read_len);
    //             read_len as isize
    //         }
    //         Err(_) => -1,
    //     }
    // } else {
    //     -1
    // }
    -1
}
