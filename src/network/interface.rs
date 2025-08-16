use std::sync::OnceLock;
use std::{io, mem};
use std::ffi::CString;
use libc::{
  close,
  bind,
  if_nametoindex,
  recv,
  send,
  socket,
  sockaddr,
  sockaddr_ll,
  AF_PACKET,
  ETH_P_ALL,
  SOCK_RAW,
};

use super::frame::Frame;

pub struct Interface {
  name: String,
  fd: i32,
  in_counter: u32,
  out_counter: u32,
}

impl Interface {
  pub fn open(name: &str) -> io::Result<Interface> {
    let if_index = get_if_index(name).unwrap();
    println!("IF index: {if_index}");
    let fd = unsafe { socket(AF_PACKET, SOCK_RAW, ETH_P_ALL.to_be()) };
    if fd < 0 {
      return Err(io::Error::last_os_error());
    }
    let mut addr: sockaddr_ll = unsafe { mem::zeroed() };
    addr.sll_family = AF_PACKET as u16;
    addr.sll_protocol = (ETH_P_ALL as u16).to_be();
    addr.sll_ifindex = if_index as i32;
    unsafe {
      if bind(fd, &addr as *const _ as *const sockaddr, mem::size_of::<sockaddr_ll>() as u32) < 0 {
          let e = io::Error::last_os_error();
          close(fd);
          return Err(e);
      }
    }
    Ok(Interface{name: name.to_string(), fd: fd, in_counter: 0, out_counter: 0})
  }

  pub fn receive(&self) -> io::Result<Frame> {
    let mut buf = vec![0; 4096];
    let n = unsafe { recv(self.fd, buf.as_mut_ptr() as *mut _, buf.len(), 0) };
    if n <= 0 {
      return Err(io::Error::last_os_error());
    }
    let frame = Frame::build(&buf, n as usize);
    println!("Received frame: intf: {}, {}", self.name, frame);
    Ok(frame)
  }

  pub fn send(&self, frame: &Frame) -> io::Result<()> {
    let data = frame.to_bytes();
    let sent = unsafe { send(self.fd, data.as_ptr() as *const _, data.len() as usize, 0) };
    if sent != data.len() as isize {
      return Err(io::Error::last_os_error());
    }
    println!("Packet sent to {}", self.name);
    Ok(())
  }
}

impl PartialEq for Interface {
  fn eq(&self, other: &Interface) -> bool {
    self.name == other.name
  }

  fn ne(&self, other: &Interface) -> bool {
    self.name != other.name
  }
}

fn get_if_index(if_name: &str) -> io::Result<u32> {
  unsafe {
    let c_name = CString::new(if_name).unwrap();
    let index = if_nametoindex(c_name.as_ptr());
    if index == 0 {
      return Err(io::Error::last_os_error());
    }
    Ok(index)
  }
}
