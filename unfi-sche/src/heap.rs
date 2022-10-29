use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use alloc::alloc::handle_alloc_error;
use crate::{config::CPU_NUM, hart_id};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;

const EMPTY_HEAP: Option<&mut MutAllocator<32>> = None;
/// HEAP 指向的是用户进程的 HEAP
static mut HEAP: [Option<&mut MutAllocator<32>>; CPU_NUM] = [EMPTY_HEAP; CPU_NUM];


pub fn init(heap: &'static mut MutAllocator<32>) {
    // 将用户进程堆的指针传递给共享库的堆，从而使得可以在用户进程的堆中分配数据
    unsafe { HEAP[hart_id()] = Some(heap) };
}

#[global_allocator]
static GLOBAL: Global = Global;

struct Global;
unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok((ptr, _)) = HEAP[hart_id()].as_mut().unwrap().allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP[hart_id()].as_mut().unwrap().deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}
