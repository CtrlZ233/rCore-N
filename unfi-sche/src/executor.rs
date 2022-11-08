
use runtime::Executor;

use crate::{config::CPU_NUM, hart_id};
const EMPTY_EXECUTOR: Option<&mut Executor> = None;
/// HEAP 指向的是用户进程的 HEAP
pub static mut EXECUTOR: [Option<&mut Executor>; CPU_NUM] = [EMPTY_EXECUTOR; CPU_NUM];

pub fn init(executor: &'static mut Executor) {
    // 将用户进程堆的指针传递给共享库的堆，从而使得可以在用户进程的堆中分配数据
    unsafe { EXECUTOR[hart_id()] = Some(executor) };
}