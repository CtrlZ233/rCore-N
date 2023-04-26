use crate::task::{current_process, current_task, current_trap_cx, block_current_and_run_next, suspend_current_and_run_next};
use alloc::sync::Arc;
use crate::net::{accept, listen, port_acceptable, PortFd};

// listen a port
pub fn sys_listen(port: u16) -> isize {
    match listen(port) {
        Some(port_index) => {
            let process = current_process().unwrap();
            let mut inner = process.acquire_inner_lock();
            let fd = inner.alloc_fd();
            let port_fd = PortFd::new(port_index);
            inner.fd_table[fd] = Some(Arc::new(port_fd));

            // NOTICE: this return the port index, not the fd
            port_index as isize
        }
        None => -1,
    }
}

// accept a tcp connection
pub fn sys_accept(port_index: usize) -> isize {
    debug!("accepting port {}", port_index);

    let task = current_task().unwrap();
    accept(port_index, task);
    // suspend_current_and_run_next();
    block_current_and_run_next();

    // net_interrupt_handler();
    // NOTICE: There does not have interrupt handler, just call it munually.
    loop {
        
        if !port_acceptable(port_index) {
            break;
        }
    }
    debug!("recived!!!!");
    let cx = current_trap_cx();
    cx.x[10] as isize
}