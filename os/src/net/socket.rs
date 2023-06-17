use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use lose_net_stack::{IPv4, packets::tcp::TCPPacket};
use spin::Mutex;

use crate::{task::{TaskControlBlock, add_task}, net::ASYNC_RDMP};

// TODO: specify the protocol, TCP or UDP
pub struct Socket {
    pub raddr: IPv4,                // remote address
    pub lport: u16,                 // local port
    pub rport: u16,                 // rempote port
    pub buffers: VecDeque<Vec<u8>>, // datas
    pub seq: u32,
    pub ack: u32,
    pub block_task: Option<Arc<TaskControlBlock>>,
}

const MAX_SOCKETS_NUM: usize = 512;

lazy_static! {
    static ref SOCKET_TABLE: Mutex<Vec<Option<Arc<Mutex<Socket>>>>> =
        unsafe { Mutex::new(Vec::with_capacity(MAX_SOCKETS_NUM)) };
}

pub fn get_mutex_socket(index: usize) -> Option<Arc<Mutex<Socket>>> {
    let socket_table = SOCKET_TABLE.lock();
    socket_table.get(index).map_or(None, |x| (*x).clone())
}

pub fn get_s_a_by_index(index: usize) -> Option<(u32, u32)> {
    let socket_table = SOCKET_TABLE.lock();

    assert!(index < socket_table.len());

    socket_table.get(index).map_or(None, |x| match x {
        Some(x) => {
            let socket = x.lock();
            return Some((socket.seq, socket.ack));
        }
        None => None
    })
}


pub fn get_socket(raddr: IPv4, lport: u16, rport: u16) -> Option<usize> {
    let socket_table = SOCKET_TABLE.lock();
    for i in 0..socket_table.len() {
        let sock = &socket_table[i];
        if sock.is_none() {
            continue;
        }

        let sock = sock.as_ref().unwrap().lock();
        if sock.raddr == raddr && sock.lport == lport && sock.rport == rport {
            return Some(i);
        }
    }
    None
}


pub fn add_socket(raddr: IPv4, lport: u16, rport: u16, seq: u32, ack: u32) -> Option<usize> {
    if get_socket(raddr, lport, rport).is_some() {
        return None;
    }

    let mut socket_table = SOCKET_TABLE.lock();
    let mut index = usize::MAX;
    for i in 0..socket_table.len() {
        if socket_table[i].is_none() {
            index = i;
            break;
        }
    }

    let socket = Socket {
        raddr,
        lport,
        rport,
        buffers: VecDeque::new(),
        seq: seq,
        ack: ack,
        block_task: None,
    };

    if index == usize::MAX {
        socket_table.push(Some(Arc::new(Mutex::new(socket))));
        Some(socket_table.len() - 1)
    } else {
        socket_table[index] = Some(Arc::new(Mutex::new(socket)));
        Some(index)
    }
}

pub fn remove_socket(index: usize) {
    let mut socket_table = SOCKET_TABLE.lock();

    assert!(socket_table.len() > index);
    socket_table[index] = None;
}

pub fn push_data(index: usize, packet: &TCPPacket) {
    let mut socket_table = SOCKET_TABLE.lock();
    if socket_table.len() <= index || socket_table[index].is_none() {
        return;
    }
    assert!(socket_table.len() > index);
    assert!(socket_table[index].is_some());
    let mut socket = socket_table[index].as_mut().unwrap().lock();
    socket.buffers.push_back(packet.data.to_vec());
    socket.ack = packet.seq + packet.data_len as u32;
    socket.seq = packet.ack;
    debug!("[push_data] index: {}, socket.ack:{}, socket.seq:{}", index, socket.ack, socket.seq);
    match socket.block_task.take() {
        Some(task) => {
            debug!("wake read task");
            add_task(task);
        }
        _ => {

        }
    }

    if let Some(cid) = ASYNC_RDMP.lock().remove(&index) {
        // debug!("wake read coroutine task");
        lib_so::re_back(cid, 0);
    }

}

// pub fn pop_data(index: usize) -> Option<Vec<u8>> {
//     let mut socket_table = SOCKET_TABLE.lock();

//     assert!(socket_table.len() > index);
//     assert!(socket_table[index].is_some());

//     socket_table[index].as_mut().unwrap().buffers.pop_front()
// }