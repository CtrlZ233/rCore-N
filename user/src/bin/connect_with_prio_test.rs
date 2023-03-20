#![no_std]
#![no_main]

extern crate alloc;
extern crate user_lib;
use core::future::Future;
use core::sync::atomic::{AtomicBool, AtomicU16};
use core::sync::atomic::Ordering::Relaxed;
use core::task::{Context, Poll};
use core::pin::Pin;
use core::u64::MAX;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use core::ops::{Add, Sub, Mul};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use user_lib::*;
use alloc::boxed::Box;
use lazy_static::*;
use syscall::set_timer;
// pub const REQUEST: &str = "test";
// pub const REQUEST: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";   // 测试数据
pub const BUFFER_SIZE: usize = 4096;        // 缓冲区大小
pub const DATA_C: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const DATA_S: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

static SEND_TIMER: usize = 100; 

static mut START_TIME: usize = 0;
static RUN_TIME_LIMIT: usize = 5_000;

static RES_LOCK: Mutex<usize> = Mutex::new(0);

const MATRIX_SIZE: usize = 1;
static MSG_COUNT: Mutex<usize> = Mutex::new(0);
// pub const DATA_C: &str = "a";
// pub const DATA_S: &str = "x";

const MAX_CONNECTION: usize = 16 * (8 - 1);
const SERVER_USE_PRIO: usize = 8 - 1;

static mut CONNECTIONS: Vec<[usize; 4]> = Vec::new();

const SERVER_POLL_THREDS: usize = 2 - 1;
const CLIENT_POLL_THREDS: usize = 2 - 1;

static mut RECEIVE_BUFFER: Vec<Mutex<usize>> = Vec::new();
static mut SERVER_BUFFER: Vec<Mutex<usize>> = Vec::new();

static mut SERVER_AWAIT: Vec<AtomicBool> = Vec::new();
static mut SENDER_AWAIT: Vec<AtomicBool> = Vec::new();

static mut TIMER_QUEUE: Vec<Mutex<VecDeque<usize>>> = Vec::new();
static mut REQ_DELAY: Vec<Vec<usize>> = Vec::new();

static COMPLETE_CONNECTIONS: Mutex<usize> = Mutex::new(0);
static TOTAL_DELAY: Mutex<Vec<Vec<usize>>> = Mutex::new(Vec::new());

static COMPLETE_SERVER: Mutex<usize> = Mutex::new(0);
static COMPLETE_RECV: Mutex<usize> = Mutex::new(0);
static COMPLETE_SENDER: Mutex<usize> = Mutex::new(0);


static PRINT_LOCK: Mutex<()> = Mutex::new(());

fn println(a: usize, b: usize) {
    let _lock = PRINT_LOCK.lock();
    println!("end: {}, start: {}", a, b);
}

#[no_mangle]
pub fn main() -> i32 {
    let father_pid = getpid();
    let start = get_time();
    unsafe {
        START_TIME = start as usize;
    }
    let end = get_time();
    println!("main tid: {}", gettid());
    // lib_so::spawn(move ||client_send(0, 0, pid as usize), 1, pid as usize + 1, lib_so::CoroutineKind::UserNorm);

    for i in 0..MAX_CONNECTION {
        let mut fds0 = [0usize; 2];
        let mut fds1 = [0usize; 2];
        pipe(&mut fds0);
        pipe(&mut fds1);
        unsafe {
            CONNECTIONS.push([fds0[0], fds0[1], fds1[0], fds1[1]]);
        }
    }

    for _ in 0..SERVER_USE_PRIO {
        TOTAL_DELAY.lock().push(Vec::new());
    }

    let pid = fork();
    if pid == 0 {
        // 客户端进程
        let cur_pid = getpid() as usize;
        for i in 0..MAX_CONNECTION {
            unsafe {
                TIMER_QUEUE.push(Mutex::new(VecDeque::new()));
                REQ_DELAY.push(Vec::new());
            }
        }

        let init_res = init_user_trap();
        println!(
            "[hello world] trap init result: {:#x}, pid: {}, cost time: {} ms",
            init_res, pid, end - start
        );

        for _ in 0..CLIENT_POLL_THREDS {
            add_virtual_core();
        }

        unsafe {
            for i in 0..MAX_CONNECTION {
                close(CONNECTIONS[i][0]);
                close(CONNECTIONS[i][3]);
                lib_so::spawn(|| client_send(CONNECTIONS[i][1], i, father_pid as usize),
                            0, cur_pid + 1, lib_so::CoroutineKind::UserNorm);
                lib_so::spawn(||client_recv(CONNECTIONS[i][2],i + MAX_CONNECTION),
                            0, cur_pid + 1, lib_so::CoroutineKind::UserNorm);
            }
        }

    } else {
        // 服务端进程
        let cur_pid = getpid() as usize;
        for i in 0..MAX_CONNECTION {
            unsafe {
                RECEIVE_BUFFER.push(Mutex::new(0));
                SERVER_BUFFER.push(Mutex::new(0));
                SERVER_AWAIT.push(AtomicBool::new(false));
                SENDER_AWAIT.push(AtomicBool::new(false));
            }
        }

        let init_res = init_user_trap();
        println!(
            "[hello world] trap init result: {:#x}, pid: {}, cost time: {} ms",
            init_res, pid, end - start
        );
        
        for _ in 0..SERVER_POLL_THREDS {
            add_virtual_core();
        }

        unsafe {
            for i in 0..MAX_CONNECTION {
                close(CONNECTIONS[i][1]);
                close(CONNECTIONS[i][2]);
                let send_cid = lib_so::spawn(|| msg_sender(CONNECTIONS[i][3], i + MAX_CONNECTION, pid as usize), 
                                (i % SERVER_USE_PRIO) + 1, cur_pid + 1, lib_so::CoroutineKind::UserNorm);
                let server_cid = lib_so::spawn(|| msg_server(i, send_cid),
                                (i % SERVER_USE_PRIO) + 1, cur_pid + 1, lib_so::CoroutineKind::UserNorm);

                lib_so::spawn(|| msg_receiver(CONNECTIONS[i][0], i, server_cid),
                                (i % SERVER_USE_PRIO) + 1, cur_pid + 1, lib_so::CoroutineKind::UserNorm);
            }
        }
        sleep(RUN_TIME_LIMIT + 1000);
        unsafe {
            let req = DATA_C;
            for i in 0..MAX_CONNECTION {
                syscall::write!(CONNECTIONS[i][3],  req.as_bytes(), i + MAX_CONNECTION, pid as usize);
            }
        }
        
        // let mut exit_code = 0;
        // waitpid(pid as usize, &mut exit_code);
    }

    0
}


async fn msg_server(key: usize, cid: usize) {
    unsafe {
        while (get_time() as usize) < (START_TIME + RUN_TIME_LIMIT) {
            {
                let mut recv_count = RECEIVE_BUFFER[key].lock();
                if recv_count.eq(&0) {
                    drop(recv_count);
                    SERVER_AWAIT[key].store(true, Relaxed);
                    // 切换协程
                    let mut helper = Box::new(AwaitHelper::new());
                    helper.as_mut().await;
                    continue;
                }
                *recv_count = recv_count.sub(1);
            }
            
            matrix::matrix_mul_test(MATRIX_SIZE);
            {
                let mut server_count = SERVER_BUFFER[key].lock();
                *server_count = server_count.add(1);
                if SENDER_AWAIT[key].load(Relaxed) {
                    // 唤醒sender协程
                    
                    re_back(cid);
                    SENDER_AWAIT[key].store(false, Relaxed);
                }
            }
        }

        unsafe {
            if SENDER_AWAIT[key].load(Relaxed) {
                re_back(cid);
                SENDER_AWAIT[key].store(false, Relaxed);
            }
        }
    }

    {
        let mut count = COMPLETE_SERVER.lock();
        *count = count.add(1);
        if count.eq(&MAX_CONNECTION) {
            println!("[msg_server] exit");
        }
    }
    

}

async fn msg_receiver(server_fd: usize, key: usize, cid: usize) {
    let mut buffer = [0u8; DATA_C.len()];
    unsafe {
        while (get_time() as usize) < (START_TIME + RUN_TIME_LIMIT) {
            // println!("[msg_receiver] read start");
            read!(server_fd, &mut buffer, key, current_cid());
            // println!("[msg_receiver] read end");
            // println!("[msg_receiver] server recv1");
            {
                let mut recv_count = RECEIVE_BUFFER[key].lock();
                *recv_count = recv_count.add(1);
                if SERVER_AWAIT[key].load(Relaxed) {
                    // 唤醒server协程
                    re_back(cid);
                    SERVER_AWAIT[key].store(false, Relaxed);
                }
            }
        }
    }
    
    unsafe {
        if SERVER_AWAIT[key].load(Relaxed) {
            // 唤醒server协程
            re_back(cid);
            SERVER_AWAIT[key].store(false, Relaxed);
        }
    }

    {
        let mut count = COMPLETE_RECV.lock();
        *count = count.add(1);
        if count.eq(&MAX_CONNECTION) {
            println!("[msg_receiver] exit");
        }
    }
    
    close(server_fd);
}

async fn msg_sender(server_fd: usize, key: usize, pid: usize) {
    let req = DATA_C;
    unsafe {
        while (get_time() as usize) < (START_TIME + RUN_TIME_LIMIT) {
            {
                let mut server_count = SERVER_BUFFER[key - MAX_CONNECTION].lock();
                if server_count.eq(&0) {
                    drop(server_count);
                    // 切换协程
                    SENDER_AWAIT[key - MAX_CONNECTION].store(true, Relaxed);
                    let mut helper = Box::new(AwaitHelper::new());
                    helper.as_mut().await;
                    continue;
                }
                *server_count = server_count.sub(1);
            }
            syscall::write!(server_fd,  req.as_bytes(), key, pid);
        }
    }
    {
        let mut count = COMPLETE_SENDER.lock();
        *count = count.add(1);
        if count.eq(&MAX_CONNECTION) {
            println!("[msg_sender] exit");
        }
    }
}


async fn client_send(client_fd: usize, key: usize, pid: usize) {
    let req = DATA_C;
    unsafe {
        while (get_time() as usize) < (START_TIME + RUN_TIME_LIMIT) {
            // let start = get_time();
            // println!("timer start");
            let mut helper = Box::new(TimerHelper::new());
            helper.as_mut().await;
            // println!("timer end");
            // let end = get_time();
            // println!("time interval: {}", end - start);
            // println!("{}, {}", get_time_us(), get_time());
            let start = get_time_us() as usize;
            unsafe {
                if start < START_TIME * 1000 {
                    println(start, START_TIME);
                }
            }
            TIMER_QUEUE[key].lock().push_back(get_time_us() as usize);
            syscall::write!(client_fd, req.as_bytes(), key, pid);
        }
    }
    // println!("client send close");
    close(client_fd);
}


async fn client_recv(client_fd: usize, key: usize) {
    let mut throughput = 0;
    let mut buffer = [0u8; DATA_C.len()];
    unsafe {
        while (get_time() as usize) < (START_TIME + RUN_TIME_LIMIT) {
            read!(client_fd, &mut buffer, key, current_cid());
            throughput += 1;
            let cur = get_time_us() as usize;
            if let Some(start) = TIMER_QUEUE[key - MAX_CONNECTION].lock().pop_back() {
                REQ_DELAY[key - MAX_CONNECTION].push(cur - start);
            } else {
                println!("REQ_DELAY size: {}", REQ_DELAY[key - MAX_CONNECTION].len());
                println!("error!!!");
            }
        }
    }
    close(client_fd);
    unsafe {
        statistic(key - MAX_CONNECTION, &mut REQ_DELAY[key - MAX_CONNECTION]);
        let mut complete_connections = COMPLETE_CONNECTIONS.lock();
        *complete_connections = complete_connections.add(1);
        if complete_connections.eq(&MAX_CONNECTION) {
            final_statistic();
        }
    }
    
}

fn statistic(key: usize, req_dealy: &mut Vec<usize>) {
    if req_dealy.len() < 3 {
        return;
    }
    // 去掉最大值和最小值
    req_dealy.sort();
    req_dealy.remove(0);
    req_dealy.pop();
    // println!("req_delay: {:?}", req_dealy);

    {
        let mut total_delay = TOTAL_DELAY.lock();
        total_delay[key % SERVER_USE_PRIO].append(req_dealy);
    }
}

fn final_statistic() {
    let total_delay = TOTAL_DELAY.lock();
    for i in 0..SERVER_USE_PRIO {
        let (avg, sigma) = calc_per_prio(&total_delay[i]);
        let _lock = RES_LOCK.lock();
        println!("prio: {}, avg: {}, sigma: {}, throughput: {}", i, avg, sigma, total_delay[i].len());
    }
}

fn calc_per_prio(req_dealy: &Vec<usize>) -> (isize, isize) {
    let mut avg = 0;
    let mut sigma = 0;
    let sum: usize = req_dealy.iter().sum();
    avg = sum as isize / (req_dealy.len() as isize);
    let mut sigma_sum = 0;
    for delay in req_dealy.iter() {
        let tmp = (*delay) as isize - avg;
        sigma_sum += tmp * tmp;
    }

    sigma = sigma_sum / (req_dealy.len() as isize);
    (avg, sigma)
}


struct TimerHelper {
    time: usize,
}

impl TimerHelper {
    fn new() -> Self {
        TimerHelper {
            time: get_time() as usize,
        }
    }
}

impl Future for TimerHelper {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let cur_time = get_time() as usize;
        if self.time + SEND_TIMER > cur_time {
            // println!("send start: {}", current_cid());
            set_timer!(((self.time + SEND_TIMER) * 1000) as isize, current_cid());
            return Poll::Pending;
        }
        
        return Poll::Ready(());
    }
}


struct AwaitHelper {
    flag: bool,
}

impl AwaitHelper {
    fn new() -> Self {
        AwaitHelper {
            flag: false,
        }
    }
}

impl Future for AwaitHelper {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.flag == false {
            self.flag = true;
            return Poll::Pending;
        }
        return Poll::Ready(());
    }
}


#[no_mangle]
pub fn wake_handler(cid: usize) {
    re_back(cid);
}