use std::{ffi, fmt, io, mem};
use std::collections::HashSet;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, BorrowedFd};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;

use libc;
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
  tpacket_auxdata,
  AF_PACKET,
  ETH_P_ALL,
  PACKET_AUXDATA,
  SOCK_RAW,
  SOL_PACKET,
  SOL_SOCKET,
  SO_RCVTIMEO,
};

use super::frame::Frame;

pub const DEFAULT_VLAN : u16 = 1;

pub enum IntfCmd {
  Shutdown,
  NoShutdown,
  PortModeAccess,
  PortModeMonitoring(String),
  PortModeTrunk,
  PortAccessVlan(u16),
  PortTrunkAddVlans(Vec<u16>),
  PortTrunkRemoveVlans(Vec<u16>),
}

#[derive(Debug,Clone)]
pub enum PortMode {
  Access { vlan: u16 },
  Trunk { vlans: HashSet<u16>},
  Monitoring(String),
}

#[derive(Clone, Debug)]
pub struct InterfaceRoData<'a> {
  //TODO move fd out of here and have one egr fd for each sender thread intf
  fd: Option<BorrowedFd<'a>>,
  mode: PortMode,
}

#[derive(Debug)]
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
      intf_ro_data: ArcSwap::from_pointee(InterfaceRoData{ fd: None, mode: PortMode::Access{vlan: 1 }})
    };
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
    let packet_auxdata = 1;
    unsafe {
      if bind(fd, &addr as *const _ as *const sockaddr, mem::size_of::<sockaddr_ll>() as u32) < 0 {
          let e = io::Error::last_os_error();
          close(fd);
          return Err(e);
      }
      if setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout as *const _ as *const c_void, size_of::<timeval>() as u32) < 0 {
        let e = io::Error::last_os_error();
        close(fd);
        return Err(e);
      }
      if setsockopt(fd, SOL_PACKET, PACKET_AUXDATA, &packet_auxdata as *const _ as *const c_void, size_of::<u32>() as u32) < 0 {
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
    let mut buf : Vec<u8> = vec![0; 4096];
    let mut ctrl_msg : Vec<u8> = unsafe { vec![0; libc::CMSG_SPACE(500) as usize] };
    let mut msghdr : libc::msghdr = unsafe { mem::zeroed() };
    let iov = [
      std::io::IoSliceMut::new(&mut buf),
    ];
    msghdr.msg_iov = iov.as_ptr() as *mut libc::iovec;
    msghdr.msg_iovlen = iov.len();
    msghdr.msg_control = ctrl_msg.as_ptr() as *mut c_void;
    msghdr.msg_controllen = ctrl_msg.len();

    if let Some(fd) = &self.fd {
      let n = unsafe { libc::recvmsg(fd.as_raw_fd(), &mut msghdr as *mut libc::msghdr, 0) };
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

      let mut aux_data = None;
      let mut cmsg = unsafe { libc::CMSG_FIRSTHDR(&mut msghdr as *mut libc::msghdr) };
      while !cmsg.is_null() {
        unsafe {
          if (*cmsg).cmsg_level != SOL_PACKET ||
            (*cmsg).cmsg_type != PACKET_AUXDATA {
            cmsg = libc::CMSG_NXTHDR(&msghdr as *const libc::msghdr, cmsg);
            continue;
          }
        }

        let auxdata = unsafe { libc::CMSG_DATA(cmsg) as *const tpacket_auxdata };
        aux_data = unsafe { Some(*auxdata) };
        break;
      }

      let frame = Frame::parse(&buf, n as usize, aux_data);

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

  pub fn ing_process_frame(&self, mut frame: Frame) -> Option<Frame> {
    match self.view.intf_ro_data.load().mode {
      PortMode::Access{vlan} => {
        if frame.get_vlan() != 0 {
          if self.view.debug_mode.load(Ordering::Relaxed) {
            println!("Dropping tagged frame ingressing on access port");
          }
          return None; // Drop tagged frame
        }
        frame.tag(vlan);
      },
      PortMode::Trunk{ref vlans} => {
        let vlan = frame.get_vlan();
        if vlan == 0 {
          if self.view.debug_mode.load(Ordering::Relaxed) {
            println!("Dropping untagged frame ingressing on trunk port");
          }
          return None; // Drop untagged & bad vlan frame
        } else if !vlans.contains(&vlan) {
          if self.view.debug_mode.load(Ordering::Relaxed) {
            println!("Dropping frame taggued {} ingressing on trunk port allowing {:?}", vlan, vlans);
          }
          return None; // Drop untagged & bad vlan frame
        }
      }
      PortMode::Monitoring(_) => return None, // Drop ingress on monitoring ports
    }
    Some(frame)
  }

  pub fn is_up(&self) -> bool {
    self.fd.is_some() 
  }

  pub fn set_port_mode_monitoring(&self, target: &String) {
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    intf_ro_data.mode = PortMode::Monitoring(target.clone());
    self.view.intf_ro_data.store(Arc::new(intf_ro_data));
  }

  pub fn get_monitoring_targets(&self) -> Option<String> {
    if let PortMode::Monitoring(target) = &self.view.intf_ro_data.load().mode {
      return Some(target.clone());
    }
    None
  }

  pub fn set_port_mode_access_vlan(&self, vlan: u16) {
    debug_assert!(vlan > 0 && vlan < 4096);
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    intf_ro_data.mode = PortMode::Access{vlan: vlan};
    self.view.intf_ro_data.store(Arc::new(intf_ro_data));
  }

  pub fn set_port_mode_trunk_vlan(&self) {
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    intf_ro_data.mode = PortMode::Trunk{vlans: HashSet::new()};
    self.view.intf_ro_data.store(Arc::new(intf_ro_data));
  }

  pub fn add_trunk_allowed_vlan(&self, vlans: &Vec<u16>) {
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    debug_assert!(vlans.into_iter().all(| x | *x > 0 && *x < 4096));
    if let PortMode::Trunk{vlans: ref mut allowed_vlans} = intf_ro_data.mode  {
      for vlan in vlans {
        allowed_vlans.insert(*vlan);
      }
      self.view.intf_ro_data.store(Arc::new(intf_ro_data));
    } else {
      debug_assert!(false);
    }
  }

  pub fn remove_trunk_allowed_vlan(&self, vlans: &Vec<u16>) {
    let mut intf_ro_data = self.view.intf_ro_data.load().as_ref().clone();
    debug_assert!(vlans.into_iter().all(| x | *x > 0 && *x < 4096));
    if let PortMode::Trunk{vlans: ref mut allowed_vlans} = intf_ro_data.mode  {
      for vlan in vlans {
        allowed_vlans.remove(vlan);
      }
      self.view.intf_ro_data.store(Arc::new(intf_ro_data));
    } else {
      debug_assert!(false);
    }
  }
}

impl InterfaceView<'_> {

  pub fn send(&self, frame: Frame) -> io::Result<()> {
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

  pub fn egr_process_frame(&self, mut frame: Frame) -> Frame {
     if let PortMode::Access{vlan} = self.intf_ro_data.load().mode {
       debug_assert!(vlan == frame.get_vlan()); //vlan should be checked before
       frame.untag()
     }
     frame
  }

  pub fn is_up(&self) -> bool {
    self.intf_ro_data.load().fd.is_some() 
  }

  pub fn get_port_mode(&self) -> PortMode {
    self.intf_ro_data.load().as_ref().clone().mode
  }

  pub fn is_monitoring(&self) -> bool {
    matches!( self.intf_ro_data.load().mode, PortMode::Monitoring(_))
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

  pub fn allows_vlan(&self, vlan: u16) -> bool {
    match &self.intf_ro_data.load().mode {
      PortMode::Access{vlan: port_vlan} => *port_vlan == vlan,
      PortMode::Trunk{vlans} => vlans.contains(&vlan),
      PortMode::Monitoring(_) => panic!("Unexpected path")
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
    let mut output = format!("{}\n----------\nStatus: {}\nMode: {}\n",
      self.name,
      if let Some(_) = &ro_data.fd { "running" } else { "shutdown" },
      match ro_data.mode {
        PortMode::Access{..} => "Access",
        PortMode::Trunk{..} => "Trunk",
        PortMode::Monitoring(_) => "Monitoring",
      }
    );

    if let PortMode::Access{vlan} = &ro_data.mode {
      output += &format!("Vlan: {}\n", vlan);
    }
    if let PortMode::Trunk{vlans} = &ro_data.mode {
      output += &format!("Allowed Vlans: {:?}\n", vlans);
    }
    if let PortMode::Monitoring(target) = &ro_data.mode {
      output += &format!("Monitoring: {}\n", target);
    }

    output += &format!("Mode Debug: {}\n", self.debug_mode.load(Ordering::Relaxed));
    output += &format!("\nIn Pkts: {}, Out Pkts: {}\nIn bytes: {}, Out bytes: {}\n",
      self.in_pkts.load(Ordering::Relaxed), self.out_pkts.load(Ordering::Relaxed),
      self.in_bytes.load(Ordering::Relaxed), self.out_bytes.load(Ordering::Relaxed));
    write!(f, "{}", output)
  }
}

impl fmt::Display for PortMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",
    match self {
      PortMode::Access{..} => "access",
      PortMode::Trunk{..} => "trunk",
      PortMode::Monitoring(_) => "monitoring",
    })
  }
}
