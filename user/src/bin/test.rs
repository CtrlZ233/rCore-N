#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use alloc::vec;
use riscv::register::uie;
use user_lib::{add_coroutine, exit, get_time, getpid, init_user_trap, poll_future, set_timer, sleep, thread_create, waittid, sleep_await, wake_coroutine, async_wake_coroutine};
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);



#[no_mangle]
pub fn main() -> i32 {
    let init_res = init_user_trap();
    unsafe {
        uie::set_uext();
        uie::set_usoft();
        uie::set_utimer();
    }
    println!(
        "[hello world] trap init result: {:#x}, now using timer to sleep",
        init_res
    );
    add_coroutine(Box::pin(test()), 0);
    poll_future();
    0
}

async fn test() {
    println!("=======test======");
    sleep_await(1000).await;
    println!("=======test end=======");
}

#[no_mangle]
pub fn timer_intr_handler(time_us: usize) {
    IS_TIMEOUT.store(true, Relaxed);
}

#[no_mangle]
pub fn wake_handler(cid: usize) {
    println!("wake tid: {}", cid);
    wake_coroutine(cid);
    // add_coroutine(Box::pin(async_wake_coroutine(cid)), 0);
}
