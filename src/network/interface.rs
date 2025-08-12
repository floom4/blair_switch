use std::sync::OnceLock;
use rawsock::{
  open_best_library,
  traits::{
    DynamicInterface,
    Library,
  }
};

use super::frame::Frame;

static LIB: OnceLock<Box<dyn Library>> = OnceLock::new();

fn lib() -> &'static dyn Library {
  LIB.get_or_init(|| open_best_library().expect("Could not open library")).as_ref()
}

pub struct Interface {
  name: String,
  socket: Box<dyn DynamicInterface<'static>>,
}

impl Interface {
  pub fn open(name: &str) -> Interface {
    let socket = lib().open_interface(name).expect("Could not open network interface");
    println!("Interface {} opened, data link: {}", name, socket.data_link());
    Interface{name: name.to_string(), socket: socket}
  }

  pub fn receive(&mut self) -> Frame {
    let data = self.socket.receive().expect("Failed to listen interface");
    let frame = Frame::build(&data.to_vec());
    println!("Received frame: intf: {}, {}", self.name, frame);
    frame
  }

  pub fn send(&self, frame: &Frame) -> Result<(), rawsock::Error> {
    let res = self.socket.send(&frame.to_bytes());
    println!("Packet sent to {}", self.name);
    res
  }
}
