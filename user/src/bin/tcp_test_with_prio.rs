#![no_std]
#![no_main]
#[macro_use]
extern crate alloc;
extern crate user_lib;
use user_lib::{*, matrix::{string_to_matrix, print_matrix, Matrix, matrix_multiply, matrix_to_string}};
use alloc::{string::{String, ToString}, vec::Vec, collections::{VecDeque, BTreeMap}};
use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::Mutex;
use lazy_static::*;

const BUF_LEN: usize = 2048;
const MATRIX_SIZE: usize = 20;

const CLOSE_CONNECT_STR: &str = "close connection";

static MAX_POLL_THREADS: usize = 1 - 1;

const SERVER_USE_PRIO: usize = 8;
const CONNECTION_NUM: usize = SERVER_USE_PRIO * 16;


static mut REQ_MAP: Vec<Mutex<VecDeque<String>>> = Vec::new();
static mut RSP_MAP: Vec<Mutex<VecDeque<String>>> = Vec::new();

fn get_req_queue(client_fd: usize) -> &'static Mutex<VecDeque<String>> {
    unsafe {
        &REQ_MAP[client_fd]
    }
}

fn get_rsp_queue(client_fd: usize) -> &'static Mutex<VecDeque<String>> {
    unsafe {
        &RSP_MAP[client_fd]
    }
}

fn init_connection() {
    for _ in 0..(CONNECTION_NUM + 10) {
        unsafe {
            REQ_MAP.push(Mutex::new(VecDeque::new()));
            RSP_MAP.push(Mutex::new(VecDeque::new()));
        }
    }
}

async fn handle_tcp_client_async(client_fd: usize, matrix_calc_cid: usize) {
    // println!("start tcp_client");
    let str = "connect ok";
    let mut begin_buf = vec![0u8; BUF_LEN];
    read!(client_fd as usize, &mut begin_buf, 0, current_cid());
    syscall::write!(client_fd, str.as_bytes());
    loop {
        let mut buf = vec![0u8; BUF_LEN];
        read!(client_fd as usize, &mut buf, 0, current_cid());
        let recv_str: String = buf.iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
        
        let mut req_queue = get_req_queue(client_fd).lock();
        req_queue.push_back(recv_str.clone());
        re_back(matrix_calc_cid);

        if recv_str == CLOSE_CONNECT_STR {
            // println!("[handle_tcp_client_async] break");
            break;
        }
    }
}

async fn matrix_calc_async(client_fd: usize, send_rsp_cid: usize) {
    loop {
        let mut req_queue = get_req_queue(client_fd).lock();
        if let Some(req) = req_queue.pop_front() {
            let mut rsp;
            if req != CLOSE_CONNECT_STR {
                drop(req_queue);
                let matrix = string_to_matrix::<MATRIX_SIZE>(&req);
                let ans = matrix_multiply(matrix.clone(), matrix.clone());
                rsp = matrix_to_string(ans);
            } else {
                rsp = CLOSE_CONNECT_STR.to_string();
            }

            let mut rsp_queue = get_rsp_queue(client_fd).lock();
            rsp_queue.push_back(rsp);
            re_back(send_rsp_cid);

            if req == CLOSE_CONNECT_STR {
                // println!("[matrix_calc] break");
                break;
            }

        } else {
            drop(req_queue);
            let mut helper = Box::new(AwaitHelper::new());
            helper.as_mut().await;
        }
    }
}

async fn send_rsp_async(client_fd: usize) {
    loop {
        let mut rsp_queue = get_rsp_queue(client_fd).lock();
        if let Some(rsp) = rsp_queue.pop_front() {
            if rsp == CLOSE_CONNECT_STR {
                // println!("[send_rsp] break");
                // println!("close socket fd: {}", client_fd);
                close(client_fd);
                break;
            }
            drop(rsp_queue);
            syscall::write!(client_fd, rsp.as_bytes());
        } else {
            drop(rsp_queue);
            let mut helper = Box::new(AwaitHelper::new());
            helper.as_mut().await;
        }
    }
}

#[no_mangle]
pub fn main() -> i32 {

    println!("This is a very simple http server");
    let pid = getpid();
    let init_res = init_user_trap();
    for _ in 0..MAX_POLL_THREADS {
        add_virtual_core();
    }
    println!(
        "[hello tcp test] trap init result: {:#x}, pid: {}",
        init_res, pid
    );
    
    let tcp_fd = listen(80);
    if tcp_fd < 0 {
        println!("Failed to listen on port 80");
        return -1;
    }
    init_connection();
    for i in 0..CONNECTION_NUM {
        let client_fd = accept(tcp_fd as usize);
        let send_rsp_cid = spawn(move || send_rsp_async(client_fd as usize), i % SERVER_USE_PRIO);
        let matrix_calc_cid = spawn(move || matrix_calc_async(client_fd as usize, send_rsp_cid), i % SERVER_USE_PRIO);
        spawn(move || handle_tcp_client_async(client_fd as usize, matrix_calc_cid), i % SERVER_USE_PRIO);
    }

    // println!("finish tcp test");
    0
}


#[no_mangle]
pub fn wake_handler(cid: usize) {
    re_back(cid);
}