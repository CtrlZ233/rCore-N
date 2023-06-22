use alloc::{vec, collections::VecDeque};
use syscall::yield_;
use core::{
    alloc::{GlobalAlloc, Layout}, ptr::NonNull,
};
use lib_so::Executor;
use buddy_system_allocator::Heap;
use spin::Mutex;
#[no_mangle]
#[link_section = ".data.heap"]
pub static mut HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

#[no_mangle]
#[link_section = ".data.executor"]
pub static mut EXECUTOR: Executor = Executor::new(false);

// 托管空间 16 KiB
const MEMORY_SIZE: usize = 1 << 21;
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
        EXECUTOR.ready_queue = vec![VecDeque::new(); lib_so::PRIO_NUM];
    }
}


struct Global;

#[global_allocator]
static GLOBAL: Global = Global;

unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        while true {
            let op_heap = HEAP.try_lock();
            if op_heap.is_some() {
                return op_heap.unwrap().alloc(layout).ok()
                .map_or(0 as *mut u8, |allocation| allocation.as_ptr());
            }
            yield_();
        }
        return 0 as *mut u8;
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        while true {
            let op_heap = HEAP.try_lock();
            if op_heap.is_some() {
                op_heap.unwrap().dealloc(NonNull::new_unchecked(ptr), layout);
                return;
            }
            yield_();
        }
    }
}


