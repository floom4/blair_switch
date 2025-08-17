use std::process;
use std::thread;
use rustyline::error::ReadlineError;

use network::interface::Interface;

pub mod network;

pub struct Switch {
  interfaces: Vec<Interface>,
}

impl Switch {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{interfaces: Vec::new()};
    for name in interfaces_name {
      switch.interfaces.push(Interface::open(name).unwrap());
    }
    switch
  }

  pub fn start(&mut self) {
    thread::scope(|scope| {
      let egress_interfaces = &self.interfaces;
      for ingress_interface in &self.interfaces {
        let _ = scope.spawn(move || {
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
      let mut rl = rustyline::DefaultEditor::new().unwrap();
      loop {
        let input = rl.readline("blair-switch#");
        match input {
          Ok(cmd) => {
            match &cmd[..] {
              "show interfaces" => {
                println!("Interfaces:\n==========\n");
                for interface in &self.interfaces {
                  println!("{}\n", interface);
                }
              },
              default => println!("Line: {:?}", default),
            }
          }
          Err(ReadlineError::Interrupted) => println!("^C"),
          Err(ReadlineError::Eof) =>  process::exit(0),
          Err(_) => println!("No input"),
        }
      }
    });
  }
}
