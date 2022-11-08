#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use user_lib::{getpid, sleep, add_coroutine};
use alloc::boxed::Box;

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    add_coroutine(Box::pin(test1()), 2);
    add_coroutine(Box::pin(test()), 1);

    sleep(100);
    0
}

async fn test() {
    println!("hello_world test async");
}

async fn test1() {
    println!("hello_world test async  33333");
}
