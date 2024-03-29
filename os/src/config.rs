pub const USER_STACK_SIZE: usize = 0x4000;
pub const KERNEL_STACK_SIZE: usize = 0x4000;
pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

#[cfg(feature = "board_qemu")]
pub const MEMORY_END: usize = 0x84000000;

#[cfg(feature = "board_lrv")]
// pub const MEMORY_END: usize = 0x100A00000;
pub const MEMORY_END: usize = 0x101000000;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

pub const USER_TRAP_BUFFER: usize = TRAMPOLINE - PAGE_SIZE;
pub const HEAP_BUFFER: usize = USER_TRAP_BUFFER - PAGE_SIZE;
pub const TRAP_CONTEXT: usize = HEAP_BUFFER - PAGE_SIZE;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;

#[cfg(feature = "board_lrv")]
pub const CLOCK_FREQ: usize = 10_000_000;

pub const CPU_NUM: usize = 4;
pub const TRACE_SIZE: usize = 0x1000_0000; // 256M
