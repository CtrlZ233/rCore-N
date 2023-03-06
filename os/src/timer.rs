use crate::config::{CLOCK_FREQ, CPU_NUM};
use crate::sbi::set_timer;
use crate::task::hart_id;
use alloc::{collections::BTreeMap, sync::Arc, vec::Vec};
use lazy_static::*;
use riscv::register::time;
use spin::Mutex;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
pub const USEC_PER_SEC: usize = 1_000_000;

pub struct TaskID {
    pub pid: usize,
    pub coroutine_id: Option<usize>,
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[allow(dead_code)]
impl TimeVal {
    pub fn new() -> Self {
        TimeVal { sec: 0, usec: 0 }
    }
}

#[allow(unused_variables)]
pub fn get_time(mut ts: Vec<*mut usize>, tz: usize) -> isize {
    let t = time::read();
    unsafe {
        *ts[0] = t / CLOCK_FREQ;
        *ts[1] = (t % CLOCK_FREQ) * 1000000 / CLOCK_FREQ;
        trace!("t {} sec {} usec {}", t, *ts[0], *ts[1]);
    }

    0
}

pub fn sleep_for_kernel(time_ms: usize) {
    let start = get_time_ms();
    while get_time_ms() < start + time_ms {
        // sys_yield();
    }
}

#[allow(dead_code)]
pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

#[allow(dead_code)]
pub fn get_time_us() -> usize {
    time::read() * USEC_PER_SEC / CLOCK_FREQ
}

pub fn set_next_trigger() {
    // set_timer(time::read() + CLOCK_FREQ / TICKS_PER_SEC);
    set_virtual_timer(time::read() + CLOCK_FREQ / TICKS_PER_SEC, 0, usize::MAX);
}

lazy_static! {
    pub static ref TIMER_MAP: [Arc<Mutex<BTreeMap<usize, TaskID>>>; CPU_NUM] = Default::default();
}

pub fn set_virtual_timer(mut time: usize, pid: usize, cid: usize) {
    if time < time::read() {
        warn!("Time travel!");
        // return;
    }

    let coroutine_id = if cid == usize::MAX {
        None
    } else {
        Some(cid as usize)
    };

    let task_id = TaskID {
        pid: pid,
        coroutine_id
    };

    let mut timer_map = TIMER_MAP[hart_id()].lock();
    while timer_map.contains_key(&time) {
        time += 1;
    }
    timer_map.insert(time, task_id);
    if let Some((timer_min, _)) = timer_map.first_key_value() {
        if time == *timer_min {
            set_timer(time);
        }
    }
}
