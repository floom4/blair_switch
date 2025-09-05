use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use crossbeam_channel::{unbounded, Receiver};

use network::interface::{Interface, InterfaceView, IntfCmd};
use network::frame::Frame;
use cli::cli_run;

pub mod network;
pub mod cli;

pub struct Switch<'a> {
  interfaces: Vec<Interface<'a>>,
  intfs_view: HashMap<&'a str, Arc<InterfaceView<'a>>>,
  intfs_rx: HashMap<&'a str, Receiver<IntfCmd>>,
}

impl Switch<'_> {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{interfaces: Vec::new(),
      intfs_view: HashMap::new(),
      intfs_rx: HashMap::new()};
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

            // Data plane
            if ing_intf.is_up() {
              match ing_intf.receive() {
                Ok(Some(frame)) => {
                  flood(&egr_intfs, &frame);
                }
                Ok(None) => continue, // no data, timeout
                Err(err) => {
                  eprintln!("Error: {}", err);
                }
              }
            }
          }
        });
      }
      cli_run(&self.intfs_view);
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
