use std::collections::HashMap;
use std::sync::Arc;
use std::{thread, time};
use crossbeam_channel::{unbounded, Receiver};
use dashmap::DashMap;

use network::interface::{Interface, InterfaceView, IntfCmd, DEFAULT_VLAN};
use network::frame::Frame;
use fib::Fib;
use cli::shell::cli_run;

pub mod network;
pub mod fib;
pub mod cli;

pub struct Switch<'a> {
  interfaces: Vec<Interface<'a>>,
  intfs_view: HashMap<&'a str, Arc<InterfaceView<'a>>>,
  intfs_rx: HashMap<&'a str, Receiver<IntfCmd>>,
  fib: Arc<Fib<'a>>,
  mirrors: DashMap<String, Vec<Arc<InterfaceView<'a>>>>,
}

impl Switch<'_> {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{ interfaces: Vec::new(),
      intfs_view: HashMap::new(),
      intfs_rx: HashMap::new(),
      fib: Arc::new(Fib::new()),
      mirrors: DashMap::new()
    };
    for name in interfaces_name {
      let (tx, rx) = unbounded::<IntfCmd>();
      let mut intf = Interface::init(name, tx);
      if let Err(err) = intf.open() {
        eprintln!("Error: {}", err);
      }
      switch.intfs_rx.insert(&name, rx);
      switch.intfs_view.insert(&name, Arc::clone(&intf.view));
      switch.interfaces.push(intf);
      switch.mirrors.insert(name.clone(), Vec::new());
    }
    switch
  }

  pub fn start(&mut self) {
    thread::scope(|scope| {
      let interfaces = std::mem::take(&mut self.interfaces);

      for ing_intf in interfaces {
        let rx = self.intfs_rx[&ing_intf.name.as_str()].clone();
        let mut egr_intfs = self.intfs_view.clone();
        egr_intfs.remove(&ing_intf.name[..]);
        let fib = Arc::clone(&self.fib);
        let mirrors = &self.mirrors;

        let _ = scope.spawn( move || {
          run_interface_worker(ing_intf, rx, egr_intfs, fib, mirrors);
        });
      }

      cli_run(&self.intfs_view, &self.fib);
    });
  }
}

pub fn run_interface_worker<'a>(mut ing_intf: Interface<'a>, rx: Receiver<IntfCmd>,
  egr_intfs: HashMap<&str, Arc<InterfaceView<'a>>>, fib: Arc<Fib<'a>>,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView<'a>>>>,) {
  loop {

    // Control plane
    match rx.try_recv() {
      // TODO Delete intf_view from collection on shutdown
      Ok(IntfCmd::Shutdown) => ing_intf.close(),
      Ok(IntfCmd::NoShutdown) => {
        if let Err(err) = ing_intf.open() {
          eprintln!("Error: {}", err)
        }
      },
      Ok(IntfCmd::PortAccessVlan(vlan)) => {
          ing_intf.set_port_mode_access_vlan(vlan)
        },
      Ok(IntfCmd::PortModeAccess) => {
        remove_monitoring_session(&ing_intf, &mirrors);
        ing_intf.set_port_mode_access_vlan(DEFAULT_VLAN)
      },
      Ok(IntfCmd::PortModeTrunk) => {
        ing_intf.set_port_mode_trunk_vlan()
      },
      Ok(IntfCmd::PortTrunkAddVlans(vlans)) => {
        ing_intf.add_trunk_allowed_vlan(&vlans);
      },
      Ok(IntfCmd::PortTrunkRemoveVlans(vlans)) => {
        ing_intf.remove_trunk_allowed_vlan(&vlans);
      },
      // TODO Delete intf_view from collection on monitoring
      Ok(IntfCmd::PortModeMonitoring(target)) => {
        // Remove eventual previous monitoring session
        remove_monitoring_session(&ing_intf, &mirrors);

        add_monitoring_session(&ing_intf, &target, &mirrors);
      },
      Err(crossbeam_channel::TryRecvError::Empty) => (),
      Err(err) => eprintln!("Error: {}", err),
    }


    if !ing_intf.is_up() || ing_intf.view.is_monitoring() {
      thread::sleep(time::Duration::from_millis(200));
      continue;
    }

    // Data plane
    match ing_intf.receive() {
      Ok(Some(frame)) => {

        if let Some(frame) = ing_intf.ing_process_frame(frame.clone()) {
          fib.learn(frame.get_vlan(), &frame.src_mac, Arc::clone(&ing_intf.view));

          if !frame.is_broadcast() &&
            let Some(egr_intf) = fib.lookup(frame.get_vlan(), &frame.dst_mac) &&
            egr_intf.is_up() && !egr_intf.is_monitoring() &&
            egr_intf.allows_vlan(frame.get_vlan()) {
            // Unicast
            egr_process_and_send(&egr_intf, &frame, mirrors);
          } else {
            flood(&egr_intfs, &frame, &mirrors);
          }
        } else { // frame dropped
          continue
        }
      }
      Ok(None) => continue, // no data, timeout
      Err(err) => {
        eprintln!("Error: {}", err);
      }
    }
  }
}

// Frame flooding
pub fn flood(intfs: &HashMap<&str, Arc<InterfaceView>>, frame: &Frame,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>>) {

  for (_, intf) in intfs {
    if intf.is_up() && !intf.is_monitoring() && intf.allows_vlan(frame.get_vlan()) {
      egr_process_and_send(intf, &frame, mirrors);
    }
  }
}


pub fn egr_process_and_send(egr_intf: &InterfaceView, frame: &Frame,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>>) {

  // untag frame
  let out_frame = egr_intf.egr_process_frame(frame.clone());

  if let Err(err) = egr_intf.send(out_frame.clone()) {
    eprintln!("Error: {}", err);
  }

  mirror_frame(mirrors, egr_intf, &out_frame);
}

pub fn mirror_frame(mirrors: &DashMap<String, Vec<Arc<InterfaceView>>>, src: &InterfaceView, frame: &Frame) {
  if let Some(targets) = mirrors.get(&src.name) {
    for mirror in targets.iter() {
      debug_assert!(mirror.is_monitoring());
      if let Err(err) = mirror.send(frame.clone()) {
        eprintln!("Error: {}", err);
      }
    }
  }
}

pub fn add_monitoring_session<'a>(monitoring_intf: &Interface<'a>, target_intf: &String,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView<'a>>>> ) {

  monitoring_intf.set_port_mode_monitoring(target_intf);
  mirrors.alter(target_intf, |_, mut v| {v.push(monitoring_intf.view.clone()); v});
}

pub fn remove_monitoring_session(monitoring_intf: &Interface,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>> ) {

  if let Some(target_intf) = monitoring_intf.get_monitoring_targets() {
    mirrors.alter(&target_intf, |_, mut v| {
      let index = v.iter().position(|intf| intf.name == monitoring_intf.name).unwrap();
      v.remove(index);
      v
    });
  }
}
