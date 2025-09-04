use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};

use network::interface::{Interface, InterfaceView, IntfCmd};
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
      intf.open().expect("test");
      switch.intfs_rx.insert(&name, rx);
      switch.intfs_view.insert(&name, Arc::clone(&intf.view));
      switch.interfaces.push(intf);
    }
    switch
  }

  pub fn start(&mut self) {
    thread::scope(|scope| {
      let egress_interfaces = &self.intfs_view;
      let intfs_rx = &self.intfs_rx;

      let interfaces = std::mem::take(&mut self.interfaces);

      for mut ingress_interface in interfaces {
        let rx = intfs_rx[&ingress_interface.name.as_str()].clone();

        let handle = scope.spawn( move || {
          loop {

            // Control plane
            match rx.try_recv() {
              Ok(IntfCmd::Shutdown) => ingress_interface.close(),
              Ok(IntfCmd::NoShutdown) => {
                if let Err(err) = ingress_interface.open() {
                  eprintln!("Error: {}", err)
                }
              },
              Err(crossbeam_channel::TryRecvError::Empty) => (),
              Err(_) => (),
            }

            // Data plane
            if ingress_interface.is_up() {
              match ingress_interface.receive() {
                Ok(Some(frame)) => {
                  // Frame flooding
                  for (egr_name, egress_interface) in egress_interfaces {
                    if **egr_name != *ingress_interface.name && egress_interface.is_up() {
                      if let Err(err) = egress_interface.send(&frame) {
                        eprintln!("Error: {}", err);
                      }
                    }
                  }
                }
                Ok(None) => (), // no data, timeout
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
