use spin::Mutex;
use lazy_static::*;
use crate::mm::{FrameTracker, frame_alloc_more, frame_dealloc, PhysAddr, PhysPageNum, PageTable, kernel_token, VirtAddr, StepByOne};
use alloc::vec::Vec;
use virtio_drivers::{Hal, BufferDirection};
use core::{
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
lazy_static! {
    static ref QUEUE_FRAMES: Mutex<Vec<FrameTracker>> =
        unsafe { Mutex::new(Vec::new()) };
}

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (usize, NonNull<u8>) {
        let trakcers = frame_alloc_more(pages);
        let ppn_base = trakcers.as_ref().unwrap().last().unwrap().ppn;
        QUEUE_FRAMES
            .lock()
            .append(&mut trakcers.unwrap());
        let pa: PhysAddr = ppn_base.into();
        let paddr = pa.0;
        let vaddr = NonNull::new(paddr as _).unwrap();
        (paddr, vaddr)
    }

    // fn dma_dealloc(pa: usize, pages: usize) -> i32 {
    //     let pa = PhysAddr::from(pa);
    //     let mut ppn_base: PhysPageNum = pa.into();
    //     for _ in 0..pages {
    //         frame_dealloc(ppn_base);
    //         ppn_base.step();
    //     }
    //     0
    // }

    unsafe fn dma_dealloc(paddr: usize, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: usize, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> usize {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // Nothing to do, as the host already has access to all memory.
        virt_to_phys(vaddr)
    }

    unsafe fn unshare(_paddr: usize, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }

    
}

fn virt_to_phys(vaddr: usize) -> usize {
    vaddr
}
