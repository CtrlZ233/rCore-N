#![no_std]
#![no_main]
#[macro_use]
extern crate alloc;
extern crate user_lib;
use user_lib::*;

use alloc::string::{String, ToString};

fn handle_tcp_client(client_fd: usize) -> bool {
    println!("start tcp_client");
    let str = "connect ok";
    let mut begin_buf = vec![0u8; 1024];
    read!(client_fd as usize, &mut begin_buf);
    syscall::write!(client_fd, str.as_bytes());
    let server_times = 20;
    for i in 0..server_times {
        let mut buf = vec![0u8; 1024];
        let len = read!(client_fd as usize, &mut buf);
        println!("server time: {}, receive {} bytes", i, len);
        for c in buf {
            if c != 0 {
                print!("{}", c as char);
            }
        }
        println!("");
        
        let responese = "response from server";
        // write a response
        syscall::write!(client_fd, responese.as_bytes());
    }
    
    close(client_fd);
    exit(2);
}

#[no_mangle]
pub fn main() -> i32 {
    println!("This is a very simple http server");
    let mut accept_num = 8;
    let tcp_fd = listen(80);
    if tcp_fd < 0 {
        println!("Failed to listen on port 80");
        return -1;
    }
    let mut wait_tid = vec![];
    while accept_num > 0 {
        let client_fd = accept(tcp_fd as usize);
        println!("client connected: {}", client_fd);
        let tid = thread_create(handle_tcp_client as usize, client_fd as usize) as usize;
        wait_tid.push(tid);
        accept_num -= 1;
    }
    
    for tid in wait_tid.iter() {
        waittid(*tid);
    }

    println!("finish tcp test");
    0
}
