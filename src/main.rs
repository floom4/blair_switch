use std::env;
use std::process;

use blair_switch::Switch;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
      println!("Usage:");
			println!("\tblair_switch [OPTIONS] [INTERFACES...]");
      println!("Arguments:");
      println!("\tINTERFACES\tList of interfaces to attach to the switch");
      println!("Options:");
      println!("\t-h, --help\tShow this help message");
      process::exit(0);
		}

    let mut switch = Switch::build(&args[1..]);
    switch.start()
}
