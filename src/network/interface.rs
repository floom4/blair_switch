use std::{fmt, io, mem};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicU64, Ordering};

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
  fd: OwnedFd,
  in_pkts: AtomicU64,
  out_pkts: AtomicU64,
  in_bytes: AtomicU64,
  out_bytes: AtomicU64,
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
    Ok(Interface{name: name.to_string(), fd: unsafe { OwnedFd::from_raw_fd(fd) },
      in_pkts: AtomicU64::new(0), out_pkts: AtomicU64::new(0),
      in_bytes: AtomicU64::new(0), out_bytes: AtomicU64::new(0)})
  }

  pub fn receive(&self) -> io::Result<Frame> {
    // TODO handle frame bigger than buffer
    let mut buf = vec![0; 4096];
    let n = unsafe { recv(self.fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len(), 0) };
    if n <= 0 {
      return Err(io::Error::last_os_error());
    }
    let frame = Frame::build(&buf, n as usize);
    self.in_pkts.fetch_add(1, Ordering::Relaxed);
    self.in_bytes.fetch_add(n as u64, Ordering::Relaxed);
    println!("Received frame: intf: {}(ptks {}, bytes {}), {}", self.name,
      self.in_pkts.load(Ordering::Relaxed), self.in_bytes.load(Ordering::Relaxed),
      frame);
    Ok(frame)
  }

  pub fn send(&self, frame: &Frame) -> io::Result<()> {
    let data = frame.to_bytes();
    let sent = unsafe { send(self.fd.as_raw_fd(), data.as_ptr() as *const _, data.len() as usize, 0) };
    if sent != data.len() as isize {
      return Err(io::Error::last_os_error());
    }
    self.out_pkts.fetch_add(1, Ordering::Relaxed);
    self.out_bytes.fetch_add(sent as u64, Ordering::Relaxed);
    println!("Data sent to {}(pkts {}, bytes {})", self.name,
      self.out_pkts.load(Ordering::Relaxed), self.out_bytes.load(Ordering::Relaxed));
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
    let c_name = CString::new(if_name).unwrap();
    let index = unsafe { if_nametoindex(c_name.as_ptr()) };
    if index == 0 {
      return Err(io::Error::last_os_error());
    }
    Ok(index)
}

impl fmt::Display for Interface {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}:\n----------\nFd: {}\nIn Pkts: {} Out Pkts: {}\nIn bytes: {}, Out bytes: {}",
      self.name, self.fd.as_raw_fd(),
      self.in_pkts.load(Ordering::Relaxed), self.out_pkts.load(Ordering::Relaxed),
      self.in_bytes.load(Ordering::Relaxed), self.out_bytes.load(Ordering::Relaxed))
  }
}
