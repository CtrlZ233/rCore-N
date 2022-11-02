use alloc::boxed::Box;
use core::future::Future;
use core::mem;
use core::mem::transmute;
use core::ops::Add;
use core::pin::Pin;

pub static mut UNFI_SCHE_INTERFACE: usize = 0;

pub fn add_coroutine(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize) {
    unsafe {
        let unfi_sche_interface_addr: *const usize = UNFI_SCHE_INTERFACE as *const usize;
        let add_coroutine: fn(Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, usize) =
            transmute(*(unfi_sche_interface_addr.add(0)) as usize);
        add_coroutine(future, prio);
    }
}

pub fn poll_future() {
    unsafe {
        let unfi_sche_interface_addr: *const usize = UNFI_SCHE_INTERFACE as *const usize;
        let poll_future: fn() = transmute(*(unfi_sche_interface_addr.add(1)) as usize);
        poll_future();
    }
}

pub fn wake_coroutine(cid: usize) {
    unsafe {
        let unfi_sche_interface_addr: *const usize = UNFI_SCHE_INTERFACE as *const usize;
        let wake_coroutine: fn(usize) = transmute(*(unfi_sche_interface_addr.add(2)) as usize);
        wake_coroutine(cid);
    }
}

pub fn get_current_cid() -> usize {
    unsafe {
        let unfi_sche_interface_addr: *const usize = UNFI_SCHE_INTERFACE as *const usize;
        let get_current_cid: fn() -> usize = transmute(*(unfi_sche_interface_addr.add(3)) as usize);
        get_current_cid()
    }
}