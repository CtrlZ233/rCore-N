#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;
use user_lib::*;
use alloc::boxed::Box;
pub const PAIR_NUM: usize = 1;              //
pub const MAX_LEN: usize = 128;            //
pub const REQUEST: &str = "send request";   // 测试数据
pub const BUFFER_SIZE: usize = 4096;        // 缓冲区大小

#[no_mangle]
pub fn main() -> i32 {
    let init_res = init_user_trap();
    println!(
        "[hello world] trap init result: {:#x}, now using timer to sleep",
        init_res
    );
    let mut key: usize = 1;
    for i in 0..PAIR_NUM {
        // 先创建一个管道，客户端先写请求
        let mut fd1 = [0usize; 2];
        pipe(&mut fd1);
        let first_write = fd1[1];
        let mut readi = fd1[0];
        let first_key = key;
        for j in 0..MAX_LEN - 1 {
            let mut fd2 = [0usize; 2];
            pipe(&mut fd2);
            let writei = fd2[1];
            add_coroutine(Box::pin(server(readi, writei, key + 1)), 1);
            readi = fd2[0];
            key += 1;
        }
        add_coroutine(Box::pin(client(first_write, readi, first_key, key)), 0);
        key += 2;
    }
    0
}


// 服务端接收用户端的请求，从管道中读取内容
async fn server(fd1: usize, fd2: usize, key: usize) {
    println!("server read start, cid: {}", current_cid());
    let mut buffer = [0u8; BUFFER_SIZE];
    let ac_r = AsyncCall::new(ASYNC_SYSCALL_READ, fd1, buffer.as_ptr() as usize, buffer.len(), key - 1);
    ac_r.await;
    // read(fd1, &mut buffer);
    let resp = REQUEST;
    async_write(fd2, resp.as_bytes().as_ptr() as usize, resp.len(), key);
    println!("server read end");
}

// 客户端发送请求，向管道中写请求内容
async fn client(fd1: usize, fd2: usize, key1: usize, key2: usize) {
    println!("client write start");
    let req = REQUEST;
    async_write(fd1, req.as_bytes().as_ptr() as usize, req.len(), key1);
    
    let buffer = [0u8; BUFFER_SIZE];
    let ac_r = AsyncCall::new(ASYNC_SYSCALL_READ, fd2, buffer.as_ptr() as usize, buffer.len(), key2);
    ac_r.await;
    print!("------------------buffer: ");
    for c in buffer {
        if c != 0 {
            print!("{}", c as char);
        }
    }
    println!("");

    println!("client write end");
}

#[no_mangle]
pub fn wake_handler(cid: usize) {
    println!("wake tid: {}", cid);
    re_back(cid);
    // add_coroutine(Box::pin(async_wake_coroutine(cid)), 0);
}
