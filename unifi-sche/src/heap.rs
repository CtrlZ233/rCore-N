use core::{
    alloc::{GlobalAlloc, Layout},
};
use crate::config::UNFI_SCHE_BUFFER;
use buddy_system_allocator::LockedHeap;

/// 共享代码中默认的分配器，使用的是内核和用户程序各自的堆
/// 前提：堆的虚拟地址都保存在 UNFI_SCHE_BUFFER 这个虚拟地址中
/// 分配和回收时，先读取 UNFI_SCHE_BUFFER 虚拟地址中的内容
/// 再类型转换成正确的数据结构指针
/// 如果是把 heap 的指针当作参数传进需要使用的代码中，那么在分配的时候，需要显式的指出堆分配器
/// 通过这种方式，可以让默认的分配器使用不同的堆
#[global_allocator]
static GLOBAL: Global = Global;

struct Global;
unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).dealloc(ptr, layout)
    }
}
