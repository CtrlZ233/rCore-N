//! 实现内核的系统调用以及暴露给用户态的系统调用

#![no_std]

mod async_help;
mod user_interface;

extern crate syscall_macro;

use syscall_macro::{GenSysMacro, GenSysTrait};
pub use async_help::AsyncCall;
pub use user_interface::*;

#[repr(usize)]
#[derive(Debug, GenSysMacro, GenSysTrait)]
pub enum SyscallId{
    #[arguments(args = "fd")]
    Dup = 24,
    #[arguments(args = "path_ptr, flag_bits")]
    Open = 56,
    #[arguments(args = "fd")]
    Close = 57,
    #[arguments(args = "pipe_ptr")]
    Pipe = 59,
    #[arguments(args = "fd, buffer_ptr, buffer_len, key, cid")]
	Read = 63,
    #[arguments(args = "fd, buffer_ptr, buffer_len, key, cid")]
    Write = 64,
    #[arguments(args = "exit_code")]
    Exit = 93,
    Yield = 124,
    #[arguments(args = "time_ptr, tz")]
    GetTime = 169,
    GetPid = 172,
    Fork = 220,
    #[arguments(args = "path_ptr, args_ptr")]
    Exec = 221,
    #[arguments(args = "pid, exit_code_ptr")]
    WaitPid = 260,
    #[arguments(args = "path_ptr")]
    Spawn = 400,
    #[arguments(args = "buffer_ptr, buffer_len")]
    MailRead = 401,
    #[arguments(args = "pid, buffer_ptr, buffer_len")]
    MailWrite = 402,
    FlushTrace = 555,
    #[arguments(args = "tid")]
    InitUserTrap = 600,
    #[arguments(args = "pid, msg")]
    SendMsg = 601,
    #[arguments(args = "time_us, cid")]
    SetTimer = 602,
    #[arguments(args = "device_id")]
    ClaimExtInt = 603,
    #[arguments(args = "device_id, enable")]
    SetExtIntEnable = 604,
    #[arguments(args = "entry, arg")]
    ThreadCreate = 1000,
    GetTid = 1001,
    #[arguments(args = "tid")]
    WaitTid = 1002,
    Hang = 1003,
    #[arguments(args = "blocking")]
    MutexCreate = 1010,
    #[arguments(args = "id")]
    MutexLock = 1011,
    #[arguments(args = "id")]
    MutexUnlock = 1012,
    #[arguments(args = "arg")]
    CondvarCreate = 1030,
    #[arguments(args = "condvar_id")]
    CondvarSignal = 1031,
    #[arguments(args = "condvar_id, mutex_id")]
    CondvarWait = 1032,
    // #[arguments(args = "fd, buffer_ptr, buffer_len, key, pid")]
    // AsyncWrite = 2502,
    #[arguments(args = "port")]
    Listen = 1200,
    #[arguments(args = "fd")]
    Accept = 1201,
}
