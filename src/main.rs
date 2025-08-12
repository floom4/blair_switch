use std::env;
use std::process;

use blair_switch::network::interface::Interface;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
      eprintln!("Usage: blair_switch {{int_name}} {{int_name2}}");
      process::exit(1);
    }

    let mut interf = Interface::open(&args[1]);
    let mut interf2 = Interface::open(&args[2]);
    loop {
      let mut frame = interf.receive();
      interf2.send(&frame);
    }
}
