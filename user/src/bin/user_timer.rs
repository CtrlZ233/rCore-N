#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use alloc::vec;
use riscv::register::uie;
use user_lib::{exit, get_time, getpid, init_user_trap, set_timer, sleep, thread_create, waittid};
static IS_TIMEOUT: AtomicBool = AtomicBool::new(false);

static TIME_PER_INTERRUPT: usize = 100_000;

struct Timer {
    time: usize,
}

impl Timer {
    pub fn new(time_us: usize) -> Self {
        Timer {
            time: time_us,
        }
    }

    pub fn run(&mut self) {
        let mut cur = get_time() * 1000;
        let target_time = cur + self.time as isize;
        while cur < target_time {
            set_timer(cur + TIME_PER_INTERRUPT as isize);
            while !IS_TIMEOUT.load(Relaxed) {}
            cur += TIME_PER_INTERRUPT as isize;
            IS_TIMEOUT.store(false, Relaxed);
        }
    }
}

pub fn thread_a() -> ! {
    sleep(10000);
    // println!("thread_a");
    exit(1)
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    sleep(1000);
    let init_res = init_user_trap();
    println!(
        "[hello world] trap init result: {:#x}, now using timer to sleep",
        init_res
    );
    unsafe {
        uie::set_usoft();
        uie::set_utimer();
    }
    let mut v = vec!();
    let max_len = 49;
    for _ in 0..max_len {
        let tid = thread_create(thread_a as usize, 0);
        v.push(tid);
    }
    let time_us_wait = 10_000_000;
    let start = get_time() * 1000;
    let mut timer = Timer::new(time_us_wait);
    timer.run();
    let end = get_time() * 1000;
    for tid in v.iter() {
        let exit_code = waittid(*tid as usize);
        println!("thread#{} exited with code {}", tid, exit_code);
    }
    println!("target wait time: {}, real_wait_time: {}", time_us_wait, end - start);
    println!("[hello world] timer finished, now exit");
    0
}

#[no_mangle]
pub fn timer_intr_handler(time_us: usize) {
    // println!(
    //     "[user trap default] user timer interrupt, time (us): {}",
    //     time_us
    // );
    IS_TIMEOUT.store(true, Relaxed);
}
