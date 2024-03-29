use super::File;
use crate::fs::ReadHelper;
use crate::mm::UserBuffer;
use crate::task::suspend_current_and_run_next;
use alloc::sync::{Arc, Weak};
use spin::Mutex;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

#[derive(Clone)]
pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

const RING_BUFFER_SIZE: usize = 4096;

#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    write_end: Option<Weak<Pipe>>,
    read_end: Option<Weak<Pipe>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::EMPTY,
            write_end: None,
            read_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }

    pub fn set_read_end(&mut self, read_end: &Arc<Pipe>) {
        self.read_end = Some(Arc::downgrade(read_end))
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::NORMAL;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::FULL;
        }
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::NORMAL;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::EMPTY;
        }
        c
    }
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::EMPTY {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + RING_BUFFER_SIZE - self.head
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::FULL {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }

    pub fn all_read_ends_closed(&self) -> bool {
        self.read_end.as_ref().unwrap().upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(Pipe::read_end_with_buffer(buffer.clone()));
    let write_end = Arc::new(Pipe::write_end_with_buffer(buffer.clone()));
    buffer.lock().set_write_end(&write_end);
    buffer.lock().set_read_end(&read_end);
    (read_end, write_end)
}

impl File for Pipe {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize> {
        assert!(self.readable);
        let mut buf_iter = buf.into_iter();
        let mut read_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                if ring_buffer.all_write_ends_closed() {
                    return Ok(read_size);
                }
                drop(ring_buffer);
                // debug!("[pipe sync read] suspend");
                suspend_current_and_run_next();
                continue;
            }
            // read at most loop_read bytes
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe {
                        *byte_ref = ring_buffer.read_byte();
                    }
                    read_size += 1;
                } else {
                    return Ok(read_size);
                }
            }

            if buf_iter.is_full() {
                return Ok(read_size);
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> Result<usize, isize> {
        assert!(self.writable);
        let mut buf_iter = buf.into_iter();
        let mut write_size = 0usize;
        loop {

            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                debug!("iter ++");
                if ring_buffer.all_read_ends_closed() {
                    debug!("pipe readFD closed");
                    return Ok(write_size);
                }
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref });
                    write_size += 1;
                } else {
                    return Ok(write_size);
                }
            }
        }
        debug!("pipe write end");
    }
    fn awrite(&self, buf: UserBuffer, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>> {
        Box::pin(awrite_work(self.clone(), buf, pid, key))
    }
    fn aread(&self, buf: UserBuffer, cid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>{
        // debug!("UserBuffer len: {}", buf.len());

        // log::warn!("pipe aread");
        Box::pin(aread_work(self.clone(), buf, cid, pid, key))
    }

    fn readable(&self) -> bool {
        self.readable
    }

    fn writable(&self) -> bool {
        self.writable
    }
}

async fn awrite_work(s: Pipe, buf: UserBuffer, pid: usize, key: usize) {
    assert!(s.writable);
    let mut buf_iter = buf.into_iter();
    let mut write_size = 0usize;
    let mut helper = Box::new(ReadHelper::new());
    loop {

        let mut ring_buffer = s.buffer.lock();
        let loop_write = ring_buffer.available_write();
        if loop_write == 0 {
            debug!("iter ++");
            if ring_buffer.all_read_ends_closed() {
                debug!("pipe readFD closed");
                break;
                // return Ok(write_size);
            }
            drop(ring_buffer);
            // suspend_current_and_run_next();
            helper.as_mut().await;
            continue;
        }
        // write at most loop_write bytes
        for _ in 0..loop_write {
            if let Some(byte_ref) = buf_iter.next() {
                ring_buffer.write_byte(unsafe { *byte_ref });
                write_size += 1;
            } else {
                break;
                // return Ok(write_size);
            }
        }
        if buf_iter.is_full() {
            debug!("write complete!");
            break;
        }
    }
    let async_key = crate::syscall::AsyncKey { pid, key};
    // 向文件中写完数据之后，需要唤醒内核当中的协程，将管道中的数据写到缓冲区中
    if let Some(kernel_cid) = crate::syscall::WRMAP.lock().remove(&async_key) {
        // info!("kernel_cid {}", kernel_cid);
        lib_so::re_back(kernel_cid, 0);
    }
    debug!("pipe write end");
}

async fn aread_work(s: Pipe, buf: UserBuffer, cid: usize, pid: usize, key: usize) {
    let mut buf_iter = buf.into_iter();
    // let mut read_size = 0usize;
    let mut helper = Box::new(ReadHelper::new());
    loop {
        let mut ring_buffer = s.buffer.lock();
        let loop_read = ring_buffer.available_read();
        if loop_read == 0 {
            debug!("read_size is 0");
            if ring_buffer.all_write_ends_closed() {
                break ;
                //return read_size;
            }
            drop(ring_buffer);
            crate::syscall::WRMAP.lock().insert(crate::syscall::AsyncKey{pid, key}, lib_so::current_cid(true));
            helper.as_mut().await;
            continue;
        }
        debug!("read_size is {}", loop_read);
        // read at most loop_read bytes
        for _ in 0..loop_read {
            if let Some(byte_ref) = buf_iter.next() {
                unsafe { *byte_ref = ring_buffer.read_byte(); }
            } else {
                break;
            }
        }
        if buf_iter.is_full() {
            debug!("read complete!");
            break;
        }
    }
    // 将读协程加入到回调队列中，使得用户态的协程执行器能够唤醒读协程
    debug!("read pid is {}", pid);
    debug!("key is {}", key);
    let _ = push_trap_record(pid, UserTrapRecord {
        cause: 1,
        message: cid,
    });
}

use core::task::{Context, Poll};
use crate::trap::{push_trap_record, UserTrapRecord};


