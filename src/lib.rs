use std::collections::HashMap;
use std::sync::Arc;
use std::{thread, time};
use crossbeam_channel::{unbounded, Receiver};

use network::interface::{Interface, InterfaceView, IntfCmd};
use network::frame::Frame;
use fib::Fib;
use cli::cli_run;

pub mod network;
pub mod fib;
pub mod cli;

pub struct Switch<'a> {
  interfaces: Vec<Interface<'a>>,
  intfs_view: HashMap<&'a str, Arc<InterfaceView<'a>>>,
  intfs_rx: HashMap<&'a str, Receiver<IntfCmd>>,
  fib: Arc<Fib<'a>>,
}

impl Switch<'_> {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{interfaces: Vec::new(),
      intfs_view: HashMap::new(),
      intfs_rx: HashMap::new(),
      fib: Arc::new(Fib::new())};
    for name in interfaces_name {
      let (tx, rx) = unbounded::<IntfCmd>();
      let mut intf = Interface::init(name, tx);
      if let Err(err) = intf.open() {
        eprintln!("Error: {}", err);
      }
      switch.intfs_rx.insert(&name, rx);
      switch.intfs_view.insert(&name, Arc::clone(&intf.view));
      switch.interfaces.push(intf);
    }
    switch
  }

  pub fn start(&mut self) {
    thread::scope(|scope| {
      let interfaces = std::mem::take(&mut self.interfaces);

      for mut ing_intf in interfaces {
        let rx = self.intfs_rx[&ing_intf.name.as_str()].clone();
        let mut egr_intfs = self.intfs_view.clone();
        egr_intfs.remove(&ing_intf.name[..]);
        let fib = Arc::clone(&self.fib);

        let handle = scope.spawn( move || {
          loop {

            // Control plane
            match rx.try_recv() {
              Ok(IntfCmd::Shutdown) => ing_intf.close(),
              Ok(IntfCmd::NoShutdown) => {
                if let Err(err) = ing_intf.open() {
                  eprintln!("Error: {}", err)
                }
              },
              Err(crossbeam_channel::TryRecvError::Empty) => (),
              Err(_) => (),
            }


            if !ing_intf.is_up() {
              thread::sleep(time::Duration::from_millis(200));
              continue;
            }

            // Data plane
            match ing_intf.receive() {
              Ok(Some(frame)) => {
                fib.learn(&frame.src_mac, Arc::clone(&ing_intf.view));
                if !frame.is_broadcast() &&
                  let Some(egr_intf) = fib.lookup(&frame.dst_mac) &&
                  egr_intf.is_up() {
                  unicast(&egr_intf, &frame);
                } else {
                  flood(&egr_intfs, &frame);
                }
              }
              Ok(None) => continue, // no data, timeout
              Err(err) => {
                eprintln!("Error: {}", err);
              }
            }
          }
        });
      }
      cli_run(&self.intfs_view, &self.fib);
    });
  }
}

// Frame flooding
pub fn flood(intfs: &HashMap<&str, Arc<InterfaceView>>, frame: &Frame) {
  for (_, intf) in intfs {
    if intf.is_up() {
      if let Err(err) = intf.send(&frame) {
        eprintln!("Error: {}", err);
      }
    }
  }
}

pub fn unicast(intf: &Arc<InterfaceView>, frame: &Frame) {
  if let Err(err) = intf.send(&frame) {
    eprintln!("Error: {}", err);
  }
}
