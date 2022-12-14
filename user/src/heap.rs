use alloc::{alloc::handle_alloc_error, vec, collections::VecDeque};
use alloc::collections::BTreeMap;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use customizable_buddy::{BuddyAllocator, LinkedListBuddy, UsizeBuddy};
use unifi_exposure::Executor;
use spin::Mutex;
use buddy_system_allocator::LockedHeap;


pub type MutAllocator<const N: usize> = BuddyAllocator<N, UsizeBuddy, LinkedListBuddy>;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: LockedHeap = LockedHeap::empty();

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Executor = Executor::new();

// 托管空间 16 KiB
const MEMORY_SIZE: usize = 32 << 18;
#[no_mangle]
#[link_section = ".data.memory"]
static mut MEMORY: [u8; MEMORY_SIZE] = [0u8; MEMORY_SIZE];


/// 初始化全局分配器和内核堆分配器。
pub fn init() {
    // println!("heap {:#x}", unsafe{ &mut HEAP as *mut Mutex<MutAllocator<32>> as usize });
    // println!("heap {:#x}", core::mem::size_of::<Mutex<MutAllocator<32>>>());
    // println!("EXECUTOR ptr {:#x}", unsafe{ &mut EXECUTOR as *mut Executor as usize });
    // println!("memory {:#x}", unsafe{ &mut MEMORY as *mut u8 as usize });

    unsafe {
        HEAP.lock().init(
            MEMORY.as_ptr() as usize,
            MEMORY_SIZE,
        );
        // HEAP.lock().transfer(NonNull::new_unchecked(MEMORY.as_mut_ptr()), MEMORY.len());
    }
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
        HEAP.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP.dealloc(ptr, layout)
    }
}


