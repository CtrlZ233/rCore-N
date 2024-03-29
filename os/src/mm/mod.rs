mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use address::{StepByOne, VPNRange};
pub use frame_allocator::{frame_alloc, FrameTracker, frame_alloc_more, frame_dealloc};
pub use memory_set::remap_test;
pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE, kernel_token};
pub use page_table::{
    translate_writable_va, translated_byte_buffer, translated_refmut, translated_str,
    PageTableEntry, UserBuffer, UserBufferIterator, PageTable
};
use page_table::PTEFlags;
pub use heap_allocator::MutAllocator;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
}

pub fn init_kernel_space() {
    KERNEL_SPACE.lock().activate();
}
