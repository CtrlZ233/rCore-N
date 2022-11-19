use super::File;
use crate::config::UNFI_SCHE_BUFFER;
use crate::mm::{UserBuffer, translate_writable_va, MutAllocator};
use crate::task::{suspend_current_and_run_next, pid2process};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::Mutex;
use alloc::boxed::Box;
use core::{future::Future, pin::Pin};
use runtime::{Executor, CoroutineId};

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

const RING_BUFFER_SIZE: usize = 32;

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
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::EMPTY,
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
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
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(Pipe::read_end_with_buffer(buffer.clone()));
    let write_end = Arc::new(Pipe::write_end_with_buffer(buffer.clone()));
    buffer.lock().set_write_end(&write_end);
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
    }
    fn aread(&self, buf: UserBuffer, tid: usize, pid: usize, key: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>{
        async fn aread_work(s: Pipe, _buf: UserBuffer, tid: usize, pid: usize, key: usize) {
        let mut buf_iter = _buf.into_iter();
        // let mut read_size = 0usize;
        let mut helper = Box::new(ReadHelper::new());
        loop {
            let mut ring_buffer = s.buffer.lock();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                log::warn!("read_size is 0");
                if ring_buffer.all_write_ends_closed() {
                    break ;
                    //return read_size;
                }
                drop(ring_buffer);
                unsafe { crate::syscall::WRMAP.lock().insert(key, crate::lkm::current_cid()); }
                helper.as_mut().await;
                continue;
            } 
            log::warn!("read_size is {}", loop_read);  
            // read at most loop_read bytes
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe { *byte_ref = ring_buffer.read_byte(); }
                    // read_size += 1;
                } else {
                    break;
                    //return read_size;
                }
            }
        }
        // 将读协程加入到回调队列中，使得用户态的协程执行器能够唤醒读协程
        warn!("read pid is {}", pid);
        warn!("key is {}", key);
        let _ = push_trap_record(pid, UserTrapRecord {
            cause: 1,
            message: tid,
        });
        // let process = pid2process(pid).unwrap();
        // let token = process.acquire_inner_lock().memory_set.token();
        // unsafe {
        //     let vaddr = *(translate_writable_va(token, UNFI_SCHE_BUFFER).unwrap() as *const usize);
        //     let vaddr = vaddr + core::mem::size_of::<Mutex<MutAllocator<32>>>();
        //     warn!("exe vaddr is {:#x}", vaddr);
        //     let exe = translate_writable_va(token, vaddr).unwrap() as *mut usize as *mut Executor;
        //     warn!("exe paddr is {:#x}", exe as *mut usize as usize);
        //     let callback_vaddr = &mut (*exe).callback_queue as *mut Vec<CoroutineId>;
        //     let va = (*callback_vaddr).as_ptr() as usize;
        //     warn!("callback ptr {:#x}", va);
        //     let va = translate_writable_va(token, va).unwrap();
        //     let len = (*callback_vaddr).len();
        //     let cap = (*callback_vaddr).capacity();
        //     warn!("callback ptr {:#x}", va);
        //     warn!("callback len {}", len);
        //     warn!("callback cap {}", cap);
        //     let mut callback_vec = Vec::<CoroutineId>::from_raw_parts(va as *mut usize as *mut CoroutineId, len, cap);
        //     callback_vec.push(CoroutineId::get_tid_by_usize(tid));
        // }
    }
    // log::warn!("pipe aread");
    Box::pin(aread_work(self.clone(), buf, tid, pid, key))
    }
}


use core::{task::{Context, Poll}};
use crate::trap::{push_trap_record, UserTrapRecord};


pub struct ReadHelper(usize);

impl ReadHelper {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Future for ReadHelper {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0 += 1;
        if (self.0 & 1) == 1 {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}

