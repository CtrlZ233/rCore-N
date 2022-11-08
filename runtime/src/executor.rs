
use core::task::Waker;

use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use crate::{
    coroutine::{Coroutine, CoroutineId},
};

pub struct Executor {
    pub tasks: BTreeMap<CoroutineId, Arc<Coroutine>>,
    pub ready_queue: Vec<VecDeque<CoroutineId>>,
    pub waker_cache: BTreeMap<CoroutineId, Arc<Waker>>,
    pub lock: Mutex<()>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            ready_queue: Vec::new(),
            waker_cache: BTreeMap::new(),
            lock: Mutex::new(()),
        }
    }
}