#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use core::sync::atomic::{AtomicIsize, Ordering};
use riscv::register::uie;
use user_lib::{exit, get_time, init_user_trap, send_msg, spawn, yield_, UserTrapContext, UserTrapQueue, fork, exec};
use syscall::set_timer;
static PID: AtomicIsize = AtomicIsize::new(0);

#[no_mangle]
pub fn main() -> i32 {
    println!("user trap demo");
    let pid = fork();
    if pid > 0 {
        PID.store(pid, Ordering::SeqCst);
        init_user_trap();
        let time_us = get_time() * 1000;
        for i in 1..=10 {
            set_timer!(time_us + i * 1000_000);
        }
        unsafe {
            uie::set_uext();
            uie::set_usoft();
            uie::set_utimer();
        }
        loop {
            yield_();
        }
    } else {
        if exec("uart_ext\0", &[0 as *const u8]) == -1 {
            println!("Error when executing!");
            return -4;
        }
    }
    0
}

use riscv::register::{ucause, uepc, uip, utval};
pub const PAGE_SIZE: usize = 0x1000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const USER_TRAP_BUFFER: usize = TRAMPOLINE - PAGE_SIZE;
pub const TRAP_CONTEXT: usize = USER_TRAP_BUFFER - PAGE_SIZE;
// pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;
// pub const USER_TRAP_BUFFER: usize = TRAP_CONTEXT - PAGE_SIZE;
#[no_mangle]
pub fn user_trap_handler(cx: &mut UserTrapContext) -> &mut UserTrapContext {
    let ucause = ucause::read();
    let utval = utval::read();
    match ucause.cause() {
        ucause::Trap::Interrupt(ucause::Interrupt::UserSoft) => {
            let trap_queue = unsafe { &mut *(USER_TRAP_BUFFER as *mut UserTrapQueue) };
            let trap_record_num = trap_queue.len();
            println!("[user trap demo] trap record num: {}", trap_record_num);
            while let Some(trap_record) = trap_queue.dequeue() {
                let cause = trap_record.cause;
                let msg = trap_record.message;
                println!("[user trap demo] cause: {}, message {}", cause, msg,);
                if ucause::Interrupt::from(cause) == ucause::Interrupt::UserTimer {
                    handle_timer_interrupt();
                }
            }
            unsafe {
                uip::clear_usoft();
            }
        }
        ucause::Trap::Interrupt(ucause::Interrupt::UserTimer) => {
            println!("[user trap demo] user timer interrupt at {} ms", get_time());
            handle_timer_interrupt();
            unsafe {
                uip::clear_utimer();
            }
        }
        _ => {
            println!(
                "Unsupported trap {:?}, utval = {:#x}, uepc = {:#x}!",
                ucause.cause(),
                utval,
                uepc::read()
            );
        }
    }
    cx
}

fn handle_timer_interrupt() {
    static TRAP_COUNT: AtomicIsize = AtomicIsize::new(0);
    let prev_trap_count = TRAP_COUNT.fetch_add(1, Ordering::SeqCst);
    if prev_trap_count == 9 {
        println!("[user trap demo] sending SIGTERM");
        send_msg(PID.load(Ordering::SeqCst) as usize, 15);
        exit(0);
    } else {
        let msg = 0xdeadbeef00 + prev_trap_count as usize + 1;
        println!("[user trap demo] sending msg: {:#x}", msg);
        send_msg(PID.load(Ordering::SeqCst) as usize, msg);
    }
}
