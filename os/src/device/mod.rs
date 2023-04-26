pub mod plic;
pub mod uart;
mod bus;
mod net;

pub use net::NetDevice;


pub fn init() {
    net::init();
}
