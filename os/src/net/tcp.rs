
use alloc::vec;
use lose_net_stack::packets::tcp::TCPPacket;
use lose_net_stack::IPv4;
use lose_net_stack::MacAddress;
use lose_net_stack::TcpFlags;
use super::socket::block_current;
use super::socket::{add_socket, pop_data, get_s_a_by_index, remove_socket};
use super::LOSE_NET_STACK;
use crate::task::block_current_and_run_next;
use crate::task::current_task;
use crate::task::suspend_current_and_run_next;
use crate::{device::NetDevice, fs::File};
pub struct TCP {
    pub target: IPv4,
    pub sport: u16,
    pub dport: u16,
    pub seq: u32,
    pub ack: u32,
    pub socket_index: usize,
}

impl TCP {
    pub fn new(target: IPv4, sport: u16, dport: u16, seq: u32, ack: u32) -> Option<Self> {
        match add_socket(target, sport, dport) {
            Some(index) => {
                Some(
                    Self {
                        target,
                        sport,
                        dport,
                        seq,
                        ack,
                        socket_index: index,
                    }
                )
            }
            _ => {
                None
            }
        }

        
    }
}


impl File for TCP {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, mut buf: crate::mm::UserBuffer) -> Result<usize, isize> {
        loop {
            if let Some(data) = pop_data(self.socket_index) {
                let data_len = data.len();
                let mut left = 0;
                for i in 0..buf.buffers.len() {
                    let buffer_i_len = buf.buffers[i].len().min(data_len - left);

                    buf.buffers[i][..buffer_i_len]
                        .copy_from_slice(&data[left..(left + buffer_i_len)]);

                    left += buffer_i_len;
                    if left == data_len {
                        break;
                    }
                }
                return Ok(left);
            } else {
                let current = current_task().unwrap();
                block_current(current, self.socket_index);
                block_current_and_run_next();
            }
        }
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> Result<usize, isize> {
        let lose_net_stack = LOSE_NET_STACK.0.lock();

        let mut data = vec![0u8; buf.len()];

        let mut left = 0;
        for i in 0..buf.buffers.len() {
            data[left..(left + buf.buffers[i].len())].copy_from_slice(buf.buffers[i]);
            left += buf.buffers[i].len();
        }

        let len = data.len();

        // get sock and sequence
        let (ack, seq) = get_s_a_by_index(self.socket_index).map_or((0, 0), |x| x);

        let tcp_packet = TCPPacket {
            source_ip: lose_net_stack.ip,
            source_mac: lose_net_stack.mac,
            source_port: self.sport,
            dest_ip: self.target,
            dest_mac: MacAddress::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            dest_port: self.dport,
            data_len: len,
            seq,
            ack,
            flags: TcpFlags::A,
            win: 65535,
            urg: 0,
            data: data.as_ref(),
        };
        NetDevice.transmit(&tcp_packet.build_data());
        Ok(len)
    }

    fn awrite(&self, buf: crate::mm::UserBuffer, pid: usize, key: usize) -> core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = ()> + 'static + Send + Sync>> {
        todo!()
    }

    fn aread(&self, buf: crate::mm::UserBuffer, cid: usize, pid: usize, key: usize) -> core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = ()> + 'static + Send + Sync>> {
        todo!()
    }
}

impl Drop for TCP {
    fn drop(&mut self) {
        remove_socket(self.socket_index)
    }
}
