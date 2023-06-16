
use alloc::{vec, collections::VecDeque};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use lib_so::Executor;
use spin::Mutex;
use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::Heap;

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Executor = Executor::new(true);

#[no_mangle]
#[link_section = ".bss.memory"]
static mut MEMORY: [u8; KERNEL_HEAP_SIZE] = [0u8; KERNEL_HEAP_SIZE];


/// 初始化全局分配器和内核堆分配器。
pub fn init_heap() {

    unsafe {
        HEAP.lock().init(
            MEMORY.as_ptr() as usize,
            KERNEL_HEAP_SIZE,
        );
        // HEAP.lock().transfer(NonNull::new_unchecked(MEMORY.as_mut_ptr()), MEMORY.len());
        
    }
    // error!("heap {:#x}", unsafe{ &mut HEAP as *mut Mutex<MutAllocator<32>> as usize });
    // error!("heap {:#x}", core::mem::size_of::<Mutex<MutAllocator<32>>>());
    // error!("EXECUTOR ptr {:#x}", unsafe{ &mut EXECUTOR as *mut Executor as usize });
    // error!("memory {:#x}", unsafe{ &mut MEMORY as *mut u8 as usize });
    unsafe {
        EXECUTOR.ready_queue = vec![VecDeque::new(); lib_so::PRIO_NUM];
    }
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP.lock().alloc(layout).ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}


