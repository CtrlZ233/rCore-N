/// 协程支持的优先级数目
pub const PRIO_NUM: usize = 8;
/// 支持的最大进程数量
pub const MAX_PROC_NUM: usize = 0x1000;
/// 最大线程数量 CPU + 1个唤醒线程
pub const MAX_THREAD_NUM: usize = 5;
