#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;
use core::ops::Add;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicUsize;
use spin::Mutex;
use alloc::vec;
use user_lib::{exit, thread_create, waittid};
use user_lib::{mutex_blocking_create, mutex_lock, mutex_unlock};

const TOTOL_COUNT: usize = 40000;
const THREAD_NUM: usize = 8;
const ADD_COUNT: usize = TOTOL_COUNT / THREAD_NUM;
static mut COUNT: usize = 0;


pub fn thread_b() -> ! {
    for _ in 0..ADD_COUNT {
        // print!("b");
        unsafe {
            mutex_lock(0);
            COUNT += 1;
            mutex_unlock(0);
        }
    }
    exit(2)
}



#[no_mangle]
pub fn main() -> i32 {
    assert_eq!(mutex_blocking_create(), 0);
    println!("threads test========");
    let mut v = vec![];
    for i in 0..THREAD_NUM {
        println!("create tid: {}", i);
        let tid = thread_create(thread_b as usize, 0);
        println!("create tid: {} end", i);
        v.push(tid);
    }
    println!("create end");
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    unsafe {
        println!("main thread exited.: {}", COUNT);
    }
    
    println!("==========================v size: {:?}", v);
    0
}
