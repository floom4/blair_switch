use network::interface::Interface;

pub mod network;

pub struct Switch {
  interfaces: Vec<Interface>,
}

impl Switch {
  pub fn build(interfaces_name: &[String]) -> Switch {
    let mut switch = Switch{interfaces: Vec::new()};
    for name in interfaces_name {
      switch.interfaces.push(Interface::open(name));
    }
    switch
  }

  pub fn start(&mut self) {
    loop {
      let mut frame = self.interfaces[0].receive();
      self.interfaces[1].send(&frame);
    }
  }
}
