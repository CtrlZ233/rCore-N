#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{getpid, sleep, spawn};

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    for i in 0..4096 {
        if i & 1 == 0 {
            spawn(move || test1(), 1);
        } else {
            spawn(move || test(), 1);
        }
    }
    // sleep(100);
    0
}

async fn test() {
    // println!("hello_world test async, hart_id: {}, tid: {}", hart_id(), gettid());
    sleep(5);
}

async fn test1() {
    // println!("hello_world test async  33333, hart_id: {}, tid: {}", hart_id(), gettid());
    sleep(5);
}
