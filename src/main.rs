use std::env;
use std::process;

use blair_switch::network::interface::Interface;
use blair_switch::Switch;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
      eprintln!("Usage: blair_switch {{int_name}} {{int_name2}}");
      process::exit(1);
    }
    let mut switch = Switch::build(&args[1..]);
    switch.start()
}
