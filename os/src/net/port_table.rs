use crate::{fs::File, task::add_task};
use crate::task::TaskControlBlock;
use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;
use lazy_static::lazy_static;
use lose_net_stack::packets::tcp::TCPPacket;
use super::tcp::TCP;
pub struct Port {
    pub port: u16,
    pub receivable: bool,
    pub schedule: Option<Arc<TaskControlBlock>>,
}

lazy_static! {
    static ref LISTEN_TABLE: Mutex<Vec<Option<Port>>> =
        unsafe { Mutex::new(Vec::new()) };
}

pub fn listen(port: u16) -> Option<usize> {
    let mut listen_table = LISTEN_TABLE.lock();
    let mut index = usize::MAX;
    for i in 0..listen_table.len() {
        if listen_table[i].is_none() {
            index = i;
            break;
        }
    }

    let listen_port = Port {
        port,
        receivable: false,
        schedule: None,
    };

    if index == usize::MAX {
        listen_table.push(Some(listen_port));
        Some(listen_table.len() - 1)
    } else {
        listen_table[index] = Some(listen_port);
        Some(index)
    }
}

// can accept request
pub fn accept(listen_index: usize, task: Arc<TaskControlBlock>) {
    let mut listen_table = LISTEN_TABLE.lock();
    assert!(listen_index < listen_table.len());
    let listen_port = listen_table[listen_index].as_mut();
    assert!(listen_port.is_some());
    let listen_port = listen_port.unwrap();
    listen_port.receivable = true;
    listen_port.schedule = Some(task);
}

pub fn port_acceptable(listen_index: usize) -> bool {
    let mut listen_table = LISTEN_TABLE.lock();
    assert!(listen_index < listen_table.len());

    let listen_port = listen_table[listen_index].as_mut();
    listen_port.map_or(false, |x| x.receivable)
}

// check whether it can accept request
pub fn check_accept(port: u16, tcp_packet: &TCPPacket) -> Option<()> {
    let mut listen_table = LISTEN_TABLE.lock();
    let mut listen_ports: Vec<&mut Option<Port>> = listen_table
            .iter_mut()
            .filter(|x| match x {
                Some(t) => t.port == port && t.receivable == true,
                None => false,
            })
            .collect();
    if listen_ports.len() == 0 {
        debug!("no listen");
        None
    } else {
        let listen_port = listen_ports[0].as_mut().unwrap();
        let task = listen_port.schedule.clone().unwrap();
        
        if accept_connection(port, tcp_packet, task) {
            listen_port.receivable = false;
            add_task(listen_port.schedule.take().unwrap());
            Some(())
        } else {
            None
        }
        
        
    }
}

pub fn accept_connection(_port: u16, tcp_packet: &TCPPacket, task: Arc<TaskControlBlock>) -> bool {
    let process = task.process.upgrade().unwrap();
    let mut inner = process.acquire_inner_lock();
    let fd = inner.alloc_fd();
    debug!("[accept_connection]: local fd: {}, sport: {}, dport: {}", fd, tcp_packet.dest_port, tcp_packet.source_port);
    match TCP::new(
        tcp_packet.source_ip,
        tcp_packet.dest_port,
        tcp_packet.source_port,
        0,
        tcp_packet.seq + 1,
    ) {
        Some(tcp_socket) => {
            inner.fd_table[fd] = Some(Arc::new(tcp_socket));
            let cx = task.acquire_inner_lock().get_trap_cx();
            cx.x[10] = fd;
            true
        }
        _ => {
            debug!("invaild accept req");
            false
        }
    }

    

   
}


// store in the fd_table, delete the listen table when close the application.
pub struct PortFd(usize);

impl PortFd {
    pub fn new(port_index: usize) -> Self {
        PortFd(port_index)
    }
}

impl Drop for PortFd {
    fn drop(&mut self) {
        LISTEN_TABLE.lock()[self.0] = None
    }
}

impl File for PortFd {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, _buf: crate::mm::UserBuffer) -> Result<usize, isize> {
        Ok(0)
    }

    fn write(&self, _buf: crate::mm::UserBuffer) -> Result<usize, isize> {
        Ok(0)
    }

    fn awrite(&self, buf: crate::mm::UserBuffer, pid: usize, key: usize) -> core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = ()> + 'static + Send + Sync>> {
        todo!()
    }

    fn aread(&self, buf: crate::mm::UserBuffer, cid: usize, pid: usize, key: usize) -> core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = ()> + 'static + Send + Sync>> {
        todo!()
    }
}