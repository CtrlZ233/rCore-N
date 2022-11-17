use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use alloc::alloc::handle_alloc_error;
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use spin::Mutex;
use crate::config::UNFI_SCHE_BUFFER;

pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;


#[global_allocator]
static GLOBAL: Global = Global;

struct Global;
unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // println!("alloc something");
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut Mutex<MutAllocator<32>>;
        if let Ok((ptr, _)) = (*heap).lock().allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heapptr = *(UNFI_SCHE_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut Mutex<MutAllocator<32>>;
        (*heap).lock().deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}
