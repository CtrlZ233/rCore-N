#![no_std]
#![no_main]
#[macro_use]
extern crate alloc;
extern crate user_lib;
use core::ops::Add;

use embedded_hal::blocking::delay;
use user_lib::{*, matrix::{string_to_matrix, print_matrix, Matrix, matrix_multiply, matrix_to_string}};
use alloc::{string::{String, ToString}, vec::Vec, collections::{VecDeque, BTreeMap}};
use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::Mutex;
use lazy_static::*;
#[derive(PartialEq, Eq)]
enum ModelType {
    Coroutine = 1,
    Thread = 2,
}

const BUF_LEN: usize = 2048;
const MATRIX_SIZE: usize = 20;

const CLOSE_CONNECT_STR: &str = "close connection";

static MAX_POLL_THREADS: usize = 3;
static MODEL_TYPE: ModelType = ModelType::Coroutine;
static CONNECTION_NUM: usize = 128;

static mut REQ_MAP: Vec<VecDeque<String>> = Vec::new();
static mut REQ_MAP_MUTEX: Vec<usize> = Vec::new();
static mut RSP_MAP: Vec<VecDeque<String>> = Vec::new();
static mut RSP_MAP_MUTEX: Vec<usize> = Vec::new();

static mut USER_THREAD_ACTIVE: usize = 0;
// static mut TIMER_QUEUE: Vec<Mutex<VecDeque<usize>>> = Vec::new();
// static mut DELAY_QUEUE: Mutex<usize> = Mutex::new(0);
// fn get_req_queue(client_fd: usize) -> &'static Mutex<VecDeque<String>> {
//     unsafe {
//         &REQ_MAP[client_fd]
//     }
// }

// fn get_rsp_queue(client_fd: usize) -> &'static Mutex<VecDeque<String>> {
//     unsafe {
//         &RSP_MAP[client_fd]
//     }
// }

fn init_connection() {
    for _ in 0..(CONNECTION_NUM + 10) {
        unsafe {
            REQ_MAP.push(VecDeque::new());
            REQ_MAP_MUTEX.push(mutex_blocking_create() as usize);
            RSP_MAP.push(VecDeque::new());
            RSP_MAP_MUTEX.push(mutex_blocking_create() as usize);
            // TIMER_QUEUE.push(Mutex::new(VecDeque::new()));
        }
    }
}

fn handle_tcp_client(client_fd: usize) -> bool {
    // println!("start tcp_client");
    let str = "connect ok";
    let mut begin_buf = vec![0u8; BUF_LEN];
    read!(client_fd as usize, &mut begin_buf);
    syscall::write!(client_fd, str.as_bytes());
    loop {
        let mut buf = vec![0u8; BUF_LEN];
        let _len = read!(client_fd as usize, &mut buf);
        let recv_str: String = buf.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
        // get_req_queue(client_fd).lock().push_back(recv_str.clone());
        unsafe {
            // println!("[handle_tcp_client]get mutex: {}", client_fd);
            mutex_lock(REQ_MAP_MUTEX[client_fd]);
            // println!("[handle_tcp_client]get mutex succ {}", client_fd);
            let mut req_queue = &mut REQ_MAP[client_fd];
            req_queue.push_back(recv_str.clone());
            mutex_unlock(REQ_MAP_MUTEX[client_fd]);
        }
        if recv_str == CLOSE_CONNECT_STR {
            break;
        }
    }

    exit(2);
}

fn matrix_calc(client_fd: usize) {
    loop {
        unsafe {
            // println!("[matrix_calc]get mutex1: {}", REQ_MAP_MUTEX[client_fd]);
            mutex_lock(REQ_MAP_MUTEX[client_fd]);
            // println!("[matrix_calc]get mutex1 succ: {}", REQ_MAP_MUTEX[client_fd]);
            let mut req_queue = &mut REQ_MAP[client_fd];
            if let Some(req) = req_queue.pop_front() {
                mutex_unlock(REQ_MAP_MUTEX[client_fd]);
                let rsp;
                if req != CLOSE_CONNECT_STR {
                    // println!("test1");
                    let matrix = string_to_matrix::<MATRIX_SIZE>(&req);
                    // println!("test2");
                    let ans = matrix_multiply(matrix.clone(), matrix.clone());
                    // println!("test3");
                    rsp = matrix_to_string(ans);
                    // println!("test4");
                } else {
                    rsp = CLOSE_CONNECT_STR.to_string();
                }
                // get_rsp_queue(client_fd).lock().push_back(rsp);
                // println!("[matrix_calc]get mutex: {}", client_fd);
                mutex_lock(RSP_MAP_MUTEX[client_fd]);
                // println!("[matrix_calc]get mutex success: {}", client_fd);
                let mut rsp_queue = &mut RSP_MAP[client_fd];
                rsp_queue.push_back(rsp);
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                if req == CLOSE_CONNECT_STR {
                    break;
                }
            } else {
                mutex_unlock(REQ_MAP_MUTEX[client_fd]);
                yield_();
            }
        }
        
    }
    exit(2);
}

fn send_rsp(client_fd: usize) {
    loop {
        unsafe {
            // println!("[send_rsp]get mutex");
            mutex_lock(RSP_MAP_MUTEX[client_fd]);
            // println!("[send_rsp]get mutex sucess");
            let mut rsp_queue = &mut RSP_MAP[client_fd];
            if let Some(rsp) = rsp_queue.pop_front() {
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                if rsp == CLOSE_CONNECT_STR {
                    println!("[send_rsp] break");
                    // println!("close socket fd: {}", client_fd);
                    close(client_fd);
                    break;
                }
                syscall::write!(client_fd, rsp.as_bytes());
            } else {
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                yield_();
            }
        }
        
    }
    exit(2);
}


async fn handle_tcp_client_async(client_fd: usize, matrix_calc_cid: usize) {
    // println!("start tcp_client");
    let str: &str = "connect ok";
    let current_cid = current_cid();
    let mut begin_buf = vec![0u8; BUF_LEN];
    read!(client_fd as usize, &mut begin_buf, 0, current_cid);
    syscall::write!(client_fd, str.as_bytes());
    loop {
        let mut buf = vec![0u8; BUF_LEN];
        read!(client_fd as usize, &mut buf, 0, current_cid);
        let recv_str: String = buf.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
        unsafe {
            mutex_lock(REQ_MAP_MUTEX[client_fd]);
            let mut req_queue = &mut REQ_MAP[client_fd];
            req_queue.push_back(recv_str.clone());
            mutex_unlock(REQ_MAP_MUTEX[client_fd]);
        }
        if get_pending_status(matrix_calc_cid) {
            re_back(matrix_calc_cid);
        }
        
        if recv_str == CLOSE_CONNECT_STR {
            break;
        }
    }
}

async fn matrix_calc_async(client_fd: usize, send_rsp_cid: usize) {
    loop {
        unsafe {
            mutex_lock(REQ_MAP_MUTEX[client_fd]);
            let mut req_queue = &mut REQ_MAP[client_fd];
            if let Some(req) = req_queue.pop_front() {
                mutex_unlock(REQ_MAP_MUTEX[client_fd]);
                let mut rsp;
                if req != CLOSE_CONNECT_STR {
                    let matrix = string_to_matrix::<MATRIX_SIZE>(&req);
                    let ans = matrix_multiply(matrix.clone(), matrix.clone());
                    rsp = matrix_to_string(ans);
                } else {
                    rsp = CLOSE_CONNECT_STR.to_string();
                }
                
                mutex_lock(RSP_MAP_MUTEX[client_fd]);
                let mut rsp_queue = &mut RSP_MAP[client_fd];
                rsp_queue.push_back(rsp);
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                
                if get_pending_status(send_rsp_cid) {
                    re_back(send_rsp_cid);
                }
                
                if req == CLOSE_CONNECT_STR {
                    break;
                }

            } else {
                mutex_unlock(REQ_MAP_MUTEX[client_fd]);
                let mut helper = Box::new(AwaitHelper::new());
                helper.as_mut().await;
            }
        }
    }
}

async fn send_rsp_async(client_fd: usize) {
    loop {
        unsafe {
            mutex_lock(RSP_MAP_MUTEX[client_fd]);
            let mut rsp_queue = &mut RSP_MAP[client_fd];
            if let Some(rsp) = rsp_queue.pop_front() {
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                if rsp == CLOSE_CONNECT_STR {
                    // println!("[send_rsp] break");
                    // println!("close socket fd: {}", client_fd);
                    close(client_fd);
                    break;
                }
                
                syscall::write!(client_fd, rsp.as_bytes());
            } else {
                mutex_unlock(RSP_MAP_MUTEX[client_fd]);
                let mut helper = Box::new(AwaitHelper::new());
                helper.as_mut().await;
            }
        }
        
    }
    // unsafe {
    //     println!("total delay: {}", DELAY_QUEUE.lock());
    // }
}

#[no_mangle]
pub fn main() -> i32 {

    println!("This is a very simple http server");
    let pid = getpid();
    if MODEL_TYPE == ModelType::Coroutine {
        let init_res = init_user_trap();
        for _ in 0..MAX_POLL_THREADS {
            add_virtual_core();
        }
        println!(
            "[hello tcp test] trap init result: {:#x}, pid: {}",
            init_res, pid
        );

    }
    
    let tcp_fd = listen(80);
    if tcp_fd < 0 {
        println!("Failed to listen on port 80");
        return -1;
    }
    init_connection();
    let mut wait_tid = vec![];
    for _ in 0..CONNECTION_NUM {
        let client_fd = accept(tcp_fd as usize);
        // println!("client connected: {}", client_fd);
        if MODEL_TYPE == ModelType::Thread {
            let tid1 = thread_create(handle_tcp_client as usize, client_fd as usize) as usize;
            let tid2 = thread_create(send_rsp as usize, client_fd as usize) as usize;
            let tid3 = thread_create(matrix_calc as usize, client_fd as usize) as usize;
            wait_tid.push(tid1);
            wait_tid.push(tid2);
            wait_tid.push(tid3);
        } else {
            let send_rsp_cid = spawn(move || send_rsp_async(client_fd as usize), 0);
            let matrix_calc_cid = spawn(move || matrix_calc_async(client_fd as usize, send_rsp_cid), 0);
            spawn(move || handle_tcp_client_async(client_fd as usize, matrix_calc_cid), 0);
        }
    }

    for tid in wait_tid.iter() {
        waittid(*tid);
    }

    println!("finish tcp test");
    0
}


#[no_mangle]
pub fn wake_handler(cid: usize) {
    unsafe {
        USER_THREAD_ACTIVE += 1;
    }
    // println!("reback: {}", cid);
    re_back(cid);
}