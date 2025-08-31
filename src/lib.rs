use std::thread;
use std::sync::mpsc::channel;

use network::interface::Interface;
use cli::cli_run;

pub mod network;
pub mod cli;

pub struct Switch {
  interfaces: Vec<Interface>,
}

impl Switch {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{interfaces: Vec::new()};
    for name in interfaces_name {
      let mut intf = Interface::init(name);
      intf.open().expect("test");
      switch.interfaces.push(intf);
    }
    switch
  }

  pub fn start(&mut self) {
    thread::scope(|scope| {
      let egress_interfaces = &self.interfaces;
      for ingress_interface in &self.interfaces {
        let handle = scope.spawn(move || {
          loop {
            let frame = ingress_interface.receive().unwrap();
            for egress_interface in egress_interfaces {
              if *egress_interface != *ingress_interface {
                egress_interface.send(&frame).unwrap();
              }
            }
          }
        });
      }
      cli_run(&self);
    });
  }
}
