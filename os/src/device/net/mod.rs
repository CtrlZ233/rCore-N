
use core::{any::Any, ptr::NonNull};
use alloc::sync::Arc;
// const VIRTIO8: usize = 0x10008000;
use lazy_static::*;
use virtio_drivers::{
    device::{blk::VirtIOBlk, gpu::VirtIOGpu, input::VirtIOInput, net::{VirtIONet, RxBuffer}, self},
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};
use crate::device::bus::virtio::VirtioHal;
use spin::Mutex;

pub static mut NET_DEVICE_ADDR: usize = 0x10008000;
const NET_QUEUE_SIZE: usize = 32;
const NET_BUFFER_LEN: usize = 2048;

static mut NET_DEVICE: usize = 0;

pub struct NetDevice;

impl NetDevice {
    pub fn transmit(&self, data: &[u8]) {
        let net = get_net_device();
        net.lock().send(device::net::TxBuffer::from(data)).expect("can't send data");
    }

    pub fn receive(&self) -> Option<RxBuffer> {
        let net = get_net_device();
        match net.lock().receive() {
            Ok(buf) => {
                Some(buf)
            }
            Err(virtio_drivers::Error::NotReady) => {
                debug!("net read not ready");
                None
            }
            Err(err) => {
                panic!("net failed to recv: {:?}", err)
            }
        }
    }

    pub fn recycle_rx_buffer(&self, buf: RxBuffer) {
        let net = get_net_device();
        net.lock().recycle_rx_buffer(buf);
    }
}

fn get_net_device() -> &'static mut Mutex<VirtIONet<VirtioHal, MmioTransport, NET_QUEUE_SIZE>> {
    unsafe {
        &mut *(NET_DEVICE as *mut Mutex<VirtIONet<VirtioHal, MmioTransport, NET_QUEUE_SIZE>>)
    }
}


pub fn init() {
    unsafe {
        let header = NonNull::new(NET_DEVICE_ADDR as *mut VirtIOHeader).unwrap();
        let transport = MmioTransport::new(header).unwrap();
        debug!("NET_DEVICE_ADDR: {:#x}", NET_DEVICE_ADDR);
        let virtio = VirtIONet::<VirtioHal, MmioTransport, NET_QUEUE_SIZE>
            ::new(transport, NET_BUFFER_LEN)
            .expect("can't create net device by virtio");
        let net = Arc::new(Mutex::new(virtio));
        NET_DEVICE = net.as_ref() as *const Mutex<VirtIONet<VirtioHal, MmioTransport, NET_QUEUE_SIZE>> as usize;
        core::mem::forget(net);
    }
    
}
