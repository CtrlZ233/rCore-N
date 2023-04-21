
use core::any::Any;
use alloc::sync::Arc;
const VIRTIO8: usize = 0x10004000;
use lazy_static::*;
use virtio_drivers::{VirtIOHeader, VirtIONet};
use crate::device::bus::virtio::VirtioHal;
use spin::Mutex;

lazy_static! {
    pub static ref NET_DEVICE: Arc<dyn NetDevice> = Arc::new(VirtIONetWrapper::new());
}

pub trait NetDevice: Send + Sync + Any {
    fn transmit(&self, data: &[u8]);
    fn receive(&self, data: &mut [u8]) -> usize;
    fn handle_irq(&self);
}

pub struct VirtIONetWrapper(Mutex<VirtIONet<'static, VirtioHal>>);

impl NetDevice for VirtIONetWrapper {
    fn transmit(&self, data: &[u8]) {
        self.0
            .lock()
            .send(data)
            .expect("can't send data")
    }

    fn receive(&self, data: &mut [u8]) -> usize {
        self.0
            .lock()
            .recv(data)
            .expect("can't receive data")
    }

    fn handle_irq(&self) {
        
    }
}


impl VirtIONetWrapper {
    pub fn new() -> Self {
        unsafe {
            let virtio = VirtIONet::<VirtioHal>::new(&mut *(VIRTIO8 as *mut VirtIOHeader))
                .expect("can't create net device by virtio");
            VirtIONetWrapper(Mutex::new(virtio))
        }
    }
}