mod port_table;
mod tcp;
mod socket;

use spin::Mutex;
use alloc::{sync::Arc, vec, collections::BTreeMap};
use lose_net_stack::{results::Packet, LoseStack, MacAddress, TcpFlags, IPv4};
use socket::{get_socket, push_data, get_s_a_by_index};
use port_table::check_accept;
use crate::device::NetDevice;

pub use port_table::{accept, listen, port_acceptable, PortFd};
pub struct NetStack(Mutex<LoseStack>);



impl NetStack {
    pub fn new() -> Self {
        unsafe {
            NetStack(Mutex::new(LoseStack::new(
                IPv4::new(10, 0, 2, 15),
                MacAddress::new([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]),
            )))
        }
    }
}

lazy_static::lazy_static! {
    pub static ref LOSE_NET_STACK: Arc<NetStack> = Arc::new(NetStack::new());
    pub static ref ASYNC_RDMP: Arc<Mutex<BTreeMap<usize, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
}


pub fn net_interrupt_handler() {
    match NetDevice.receive() {
        Some(buf) => {
            let packet = LOSE_NET_STACK
                .0
                .lock()
                .analysis(buf.packet());
            match packet {
                Packet::ARP(arp_packet) => {
                    let lose_stack: spin::MutexGuard<LoseStack> = LOSE_NET_STACK.0.lock();
                    let reply_packet = arp_packet
                        .reply_packet(lose_stack.ip, lose_stack.mac)
                        .expect("can't build reply");
                    let reply_data = reply_packet.build_data();
                    NetDevice.transmit(&reply_data)
                }
        
                Packet::TCP(tcp_packet) => {
                    
                    let target = tcp_packet.source_ip;
                    let lport = tcp_packet.dest_port;
                    let rport = tcp_packet.source_port;
                    let flags = tcp_packet.flags;
                    debug!("[TCP] target: {}, lport: {}, rport: {}", target, lport, rport);
                    if flags.contains(TcpFlags::S) {
                        // if it has a port to accept, then response the request
                        if check_accept(lport, &tcp_packet).is_some() {
                            let mut reply_packet = tcp_packet.ack();
                            reply_packet.flags = TcpFlags::S | TcpFlags::A;
                            NetDevice.transmit(&reply_packet.build_data());
                        } else {
                            error!("check accept failed");
                        }
                        NetDevice.recycle_rx_buffer(buf);
                        return;
                    } else if tcp_packet.flags.contains(TcpFlags::F) {
                        // tcp disconnected
                        let reply_packet = tcp_packet.ack();
                        NetDevice.transmit(&reply_packet.build_data());
        
                        let mut end_packet: lose_net_stack::packets::tcp::TCPPacket = reply_packet.ack();
                        end_packet.flags |= TcpFlags::F;
                        NetDevice.transmit(&end_packet.build_data());
                    } else if tcp_packet.flags.contains(TcpFlags::A) && tcp_packet.data_len == 0 {
                        let reply_packet = tcp_packet.ack();
                        NetDevice.transmit(&reply_packet.build_data());
                        NetDevice.recycle_rx_buffer(buf);
                        return;
                    }
        
                    if let Some(socket_index) = get_socket(target, lport, rport) {
                        let packet_seq = tcp_packet.seq;
                        if let Some((seq, ack)) = get_s_a_by_index(socket_index) {
                            debug!("packet_seq: {}, ack: {}", packet_seq, ack);
                            if ack == packet_seq && tcp_packet.data_len > 0 {
                                debug!("push data: {}, {}", socket_index, tcp_packet.data_len);
                                push_data(socket_index, &tcp_packet);
                            }
                        }
                    }
                }
                _ => {}

            }
            NetDevice.recycle_rx_buffer(buf);
        }
        None => {
            debug!("do nothing");
        },
    }
}