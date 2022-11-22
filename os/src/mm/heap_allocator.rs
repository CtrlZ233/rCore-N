
use alloc::{vec, collections::VecDeque};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use unifi_exposure::Executor;
use spin::Mutex;
use crate::config::KERNEL_HEAP_SIZE;

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: Mutex<MutAllocator<32>> = Mutex::new(MutAllocator::new());

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Executor = Executor::new();

#[no_mangle]
#[link_section = ".bss.memory"]
static mut MEMORY: [u8; KERNEL_HEAP_SIZE] = [0u8; KERNEL_HEAP_SIZE];


/// 初始化全局分配器和内核堆分配器。
pub fn init_heap() {

    unsafe {
        HEAP.lock().init(
            core::mem::size_of::<usize>().trailing_zeros() as _,
            NonNull::new(MEMORY.as_mut_ptr()).unwrap(),
        );
        HEAP.lock().transfer(NonNull::new_unchecked(MEMORY.as_mut_ptr()), MEMORY.len());
        
    }
    // error!("heap {:#x}", unsafe{ &mut HEAP as *mut Mutex<MutAllocator<32>> as usize });
    // error!("heap {:#x}", core::mem::size_of::<Mutex<MutAllocator<32>>>());
    // error!("EXECUTOR ptr {:#x}", unsafe{ &mut EXECUTOR as *mut Executor as usize });
    // error!("memory {:#x}", unsafe{ &mut MEMORY as *mut u8 as usize });
    unsafe {
        EXECUTOR.ready_queue = vec![VecDeque::new(); unifi_exposure::PRIO_NUM];
    }
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Ok((ptr, _)) = HEAP.lock().allocate_layout::<u8>(layout) {
            ptr.as_ptr()
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.lock().deallocate_layout(NonNull::new(ptr).unwrap(), layout)
    }
}


