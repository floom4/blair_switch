use std::{fmt, io, mem};
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, BorrowedFd};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;

use libc::{
  close,
  c_void,
  bind,
  if_nametoindex,
  recv,
  send,
  setsockopt,
  socket,
  sockaddr,
  sockaddr_ll,
  timeval,
  AF_PACKET,
  ETH_P_ALL,
  SOCK_RAW,
  SOL_SOCKET,
  SO_RCVTIMEO,
};

use super::frame::Frame;

pub enum IntfCmd {
  Shutdown,
  NoShutdown,
}

#[derive(Clone)]
pub struct InterfaceRoData<'a> {
  //TODO move fd out of here and have one egr fd for each sender thread intf
  fd: Option<BorrowedFd<'a>>,
}

pub struct InterfaceView<'a> {
  pub name: String,
  tx: Sender<IntfCmd>,
  in_pkts: AtomicU64,
  out_pkts: AtomicU64,
  in_bytes: AtomicU64,
  out_bytes: AtomicU64,
  debug_mode: AtomicBool,
  intf_ro_data: ArcSwap<InterfaceRoData<'a>>,
}

pub struct Interface<'a> {
  pub name: String,
  if_index: u32,
  fd: Option<OwnedFd>,
  pub view: Arc<InterfaceView<'a>>,
}

impl Interface<'_> {
  pub fn init(name: &str, tx: Sender<IntfCmd>) -> Interface {
    let if_index = get_if_index(name).unwrap();
    let intf_view = InterfaceView{ name: name.to_string(), tx: tx,
      in_pkts: AtomicU64::new(0), out_pkts: AtomicU64::new(0),
      in_bytes: AtomicU64::new(0), out_bytes: AtomicU64::new(0),
      debug_mode: AtomicBool::new(false),
      intf_ro_data: ArcSwap::from_pointee(InterfaceRoData{ fd: None})};
    Interface{name: name.to_string(), if_index: if_index, fd: None, view: Arc::new(intf_view)}
  }

  pub fn open(&mut self) -> io::Result<()> {
    let fd = unsafe { socket(AF_PACKET, SOCK_RAW, ETH_P_ALL.to_be()) };
    if fd < 0 {
      return Err(io::Error::last_os_error());
    }
    let dur : Duration = Duration::from_millis(200);
    let timeout = timeval { tv_sec: dur.as_secs() as _, tv_usec: dur.subsec_micros() as _};
    let mut addr: sockaddr_ll = unsafe { mem::zeroed() };
    addr.sll_family = AF_PACKET as u16;
    addr.sll_protocol = (ETH_P_ALL as u16).to_be();
    addr.sll_ifindex = self.if_index as i32;
    unsafe {
      if bind(fd, &addr as *const _ as *const sockaddr, mem::size_of::<sockaddr_ll>() as u32) < 0 {
          let e = io::Error::last_os_error();
          close(fd);
          return Err(e);
      }
    }
    unsafe {
      if setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout as *const _ as *const c_void, size_of::<timeval>() as u32) < 0 {
        let e = io::Error::last_os_error();
        close(fd);
        return Err(e);
      }
    }
    self.fd = unsafe { Some(OwnedFd::from_raw_fd(fd)) };
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    intf_ro_data.fd = unsafe { Some(BorrowedFd::borrow_raw(fd)) };
    self.view.intf_ro_data.store(Arc::new(intf_ro_data));
    return Ok(())
  }

  pub fn close(&mut self) {
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    intf_ro_data.fd = None;
    self.view.intf_ro_data.store(Arc::new(intf_ro_data));
    self.fd = None;
  }

  pub fn receive(&self) -> io::Result<Option<Frame>> {
    // TODO handle frame bigger than buffer
    let mut buf = vec![0; 4096];
    if let Some(fd) = &self.fd {
      let n = unsafe { recv(fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len(), 0) };
      if n <= 0 {
        let err = io::Error::last_os_error();
        match err.kind() {
          io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted => {
            return Ok(None)
          },
          _ => {
            return Err(err)
          }
        }
      }
      let frame = Frame::build(&buf, n as usize);
      self.view.in_pkts.fetch_add(1, Ordering::Relaxed);
      self.view.in_bytes.fetch_add(n as u64, Ordering::Relaxed);
      if self.view.debug_mode.load(Ordering::Relaxed) {
        println!("Received frame: intf: {}, {}", self.name, frame);
      }
      Ok(Some(frame))
    } else {
      Err(io::Error::new(io::ErrorKind::BrokenPipe, "fd closed"))
    }
  }

  pub fn send(&self, frame: &Frame) -> io::Result<()> {
    if let Some(fd) = &self.fd {
      let data = frame.to_bytes();
      let sent = unsafe { send(fd.as_raw_fd(), data.as_ptr() as *const _, data.len() as usize, 0) };
      if sent != data.len() as isize {
        return Err(io::Error::last_os_error());
      }
      self.view.out_pkts.fetch_add(1, Ordering::Relaxed);
      self.view.out_bytes.fetch_add(sent as u64, Ordering::Relaxed);
      if self.view.debug_mode.load(Ordering::Relaxed) {
        println!("Data sent to {}(pkts {}, bytes {})", self.name,
          self.view.out_pkts.load(Ordering::Relaxed), self.view.out_bytes.load(Ordering::Relaxed));
      }
      Ok(())
    }
    else {
      Err(io::Error::new(io::ErrorKind::BrokenPipe, "fd closed"))
    }
  }

  pub fn is_up(&self) -> bool {
    self.fd.is_some() 
  }
}

impl InterfaceView<'_> {
  pub fn send(&self, frame: &Frame) -> io::Result<()> {
    if let Some(fd) = &self.intf_ro_data.load().fd {
      let data = frame.to_bytes();
      let sent = unsafe { send(fd.as_raw_fd(), data.as_ptr() as *const _, data.len() as usize, 0) };
      if sent != data.len() as isize {
        return Err(io::Error::last_os_error());
      }
      self.out_pkts.fetch_add(1, Ordering::Relaxed);
      self.out_bytes.fetch_add(sent as u64, Ordering::Relaxed);
      if self.debug_mode.load(Ordering::Relaxed) {
        println!("Data sent to {}(pkts {}, bytes {})", self.name,
          self.out_pkts.load(Ordering::Relaxed), self.out_bytes.load(Ordering::Relaxed));
      }
      Ok(())
    }
    else {
      Err(io::Error::new(io::ErrorKind::BrokenPipe, "fd closed"))
    }
  }

  pub fn is_up(&self) -> bool {
    self.intf_ro_data.load().fd.is_some() 
  }

  pub fn set_debug_mode(&self, value: bool) {
    self.debug_mode.store(value, Ordering::Relaxed);
  }

  pub fn reset_counters(&self) {
    self.in_pkts.store(0, Ordering::Relaxed);
    self.out_pkts.store(0, Ordering::Relaxed);
    self.in_bytes.store(0, Ordering::Relaxed);
    self.out_bytes.store(0, Ordering::Relaxed);
  }

  pub fn send_cmd(&self, cmd: IntfCmd) {
    if let Err(err) = self.tx.send(cmd){
      eprintln!("Err: {}", err);
    }
  }
}

fn get_if_index(if_name: &str) -> io::Result<u32> {
    let c_name = CString::new(if_name)?;
    let index = unsafe { if_nametoindex(c_name.as_ptr()) };
    if index == 0 {
      return Err(io::Error::last_os_error());
    }
    Ok(index)
}

impl fmt::Display for InterfaceView<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let ro_data = self.intf_ro_data.load();
    write!(f, "{}\n----------\nStatus: {}\nFd: {:?}\nIn Pkts: {} Out Pkts: {}\nIn bytes: {}, Out bytes: {}\nDebug Mode: {}",
      self.name,
      if let Some(_) = &ro_data.fd { "running" } else { "shutdown" },
      &ro_data.fd,
      self.in_pkts.load(Ordering::Relaxed), self.out_pkts.load(Ordering::Relaxed),
      self.in_bytes.load(Ordering::Relaxed), self.out_bytes.load(Ordering::Relaxed),
      self.debug_mode.load(Ordering::Relaxed))
  }
}
