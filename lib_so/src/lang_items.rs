use syscall::exit;
use spin::Mutex;
use core::ptr::NonNull;
/// _start() 函数，返回接口表的地址
#[no_mangle]
#[link_section = ".text.entry"]
extern "C" fn _start() -> usize {
    main()
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> usize {
    panic!("Cannot find main!");
}

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap();
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    } else {
        println!("Panicked: {}", err);
    }
    exit(-1);
}

use core::{
    alloc::{GlobalAlloc, Layout},
};
use crate::config::HEAP_BUFFER;
use buddy_system_allocator::Heap;
type LockedHeap = Mutex<Heap>;

/// 共享代码中默认的分配器，使用的是内核和用户程序各自的堆
/// 前提：堆的虚拟地址都保存在 HEAP_BUFFER 这个虚拟地址中
/// 分配和回收时，先读取 HEAP_BUFFER 虚拟地址中的内容
/// 再类型转换成正确的数据结构指针
/// 如果是把 heap 的指针当作参数传进需要使用的代码中，那么在分配的时候，需要显式的指出堆分配器
/// 通过这种方式，可以让默认的分配器使用不同的堆
#[global_allocator]
static GLOBAL: Global = Global;

struct Global;
unsafe impl GlobalAlloc for Global {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).lock().alloc(layout).ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heapptr = *(HEAP_BUFFER as *const usize);
        let heap = heapptr as *mut usize as *mut LockedHeap;
        (*heap).lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}