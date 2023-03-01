#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;
use user_lib::*;

pub const MAX_PROCESS: usize = 3;

#[no_mangle]
pub fn main() -> i32 {
    println!("===== async pipe fork test ====");
    for _ in 0..MAX_PROCESS {
        let pid = fork();
        if pid == 0 {
            if exec("async_pipe\0", &[0 as *const u8]) == -1 {
                println!("Error when executing!");
                return -4;
            }
        }
    }
    let mut exit_code: i32 = 0;
    for _ in 0..MAX_PROCESS {
        assert!(wait(&mut exit_code) > 0);
        assert_eq!(exit_code, 0);
    }
    0
}