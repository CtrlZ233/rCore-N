use crate::loader::get_app_data_by_name;
use alloc::vec::Vec;
use lib_so::{get_symbol_addr, VDSO_SPAWN};
use crate::mm::{KERNEL_SPACE, MemorySet};
use lazy_static::*;
use alloc::sync::Arc;
use core::mem::transmute;
use alloc::boxed::Box;
use xmas_elf::ElfFile;


lazy_static! {
    pub static ref SHARED_SCHE: Arc<Vec<u8>> = Arc::new(get_app_data_by_name("sharedscheduler").unwrap().to_vec());
    pub static ref SHARED_SCHE_MEMORYSET: MemorySet = MemorySet::from_module(
        SHARED_SCHE.as_slice()
    );
    pub static ref SHARED_ELF: ElfFile<'static> = ElfFile::new(SHARED_SCHE.as_slice()).unwrap();
}

pub fn init(){
    debug!("lkm init");
    add_lkm_image();
    debug!("lkm init done");
}

fn add_lkm_image(){

    KERNEL_SPACE.lock().add_kernel_module(&SHARED_SCHE_MEMORYSET);

    KERNEL_SPACE.lock().activate();
    lib_so::init_spawn(get_symbol_addr(&SHARED_ELF, "spawn"));
    lib_so::init_poll_kernel_future(get_symbol_addr(&SHARED_ELF, "poll_kernel_future"));
    lib_so::init_re_back(get_symbol_addr(&SHARED_ELF, "re_back"));
    lib_so::init_current_cid(get_symbol_addr(&SHARED_ELF, "current_cid"));
    lib_so::init_max_prio_pid(get_symbol_addr(&SHARED_ELF, "max_prio_pid"));
    lib_so::init_update_prio(get_symbol_addr(&SHARED_ELF, "update_prio"));
}

















