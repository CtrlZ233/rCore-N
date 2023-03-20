#![no_std]
#![no_main]

extern crate alloc;

extern crate user_lib;
use user_lib::*;


#[no_mangle]
pub fn main() -> i32 {
    let init_res = init_user_trap();
    println!(
        "[hello world] trap init result: {:#x}, now using timer to sleep",
        init_res
    );
    let mut pipe_fd = [0usize; 2];
    pipe(&mut pipe_fd);
    let read_end = pipe_fd[0];
    let write_end = pipe_fd[1];
    // 先添加读的协程，再添加写的协程，两个协程的优先级相同
    lib_so::spawn(move || server_read(read_end, 333), 0, getpid() as usize + 1, lib_so::CoroutineKind::UserNorm);
    lib_so::spawn(move || client_write(write_end, 333), 1, getpid() as usize + 1, lib_so::CoroutineKind::UserNorm);
    0
}

pub const REQUEST: &str = "send request .......................";
pub const BUFFER_SIZE: usize = 80;

// 服务端接收用户端的请求，从管道中读取内容
async fn server_read(fd: usize, key: usize) {
    println!("server read start, cid: {}", current_cid());
    let mut buffer = [0u8; REQUEST.len()];
    read!(fd, &mut buffer, key, current_cid());
    print!("buffer: ");
    for c in buffer {
        if c != 0 {
            print!("{}", c as char);
        }
    }
    println!("");
    println!("server read end");
}

// 客户端发送请求，向管道中写请求内容
async fn client_write(fd: usize, key: usize) {
    println!("client write start");
    let req = REQUEST;
    syscall::write!(fd, req.as_bytes(), key, getpid() as usize);
    println!("client write end");
}

#[no_mangle]
pub fn wake_handler(cid: usize) {
    println!("wake tid: {}", cid);
    re_back(cid);
}