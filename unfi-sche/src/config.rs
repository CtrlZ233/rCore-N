pub const PAGE_SIZE: usize = 0x1000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

pub const USER_TRAP_BUFFER: usize = TRAMPOLINE - PAGE_SIZE;
pub const UNFI_SCHE_BUFFER: usize = USER_TRAP_BUFFER - PAGE_SIZE;

pub const ENTRY: usize = 0x1000;
// 最大的进程数量
pub const MAX_PROC_NUM: usize = 0x1000;


