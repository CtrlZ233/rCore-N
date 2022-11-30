#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{getpid, sleep, add_coroutine};
use alloc::boxed::Box;
use syscall::gettid;
use user_lib::trap::hart_id;

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    for i in 0..4096 {
        if i & 1 == 0 {
            add_coroutine(Box::pin(test1()), 1);
        } else {
            add_coroutine(Box::pin(test()), 1);
        }
    }
    // sleep(100);
    0
}

async fn test() {
    println!("hello_world test async, hart_id: {}, tid: {}", hart_id(), gettid());
}

async fn test1() {
    println!("hello_world test async  33333, hart_id: {}, tid: {}", hart_id(), gettid());
}
