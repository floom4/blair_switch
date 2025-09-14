use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use rustyline::error::ReadlineError;
use super::network::interface::{InterfaceView, IntfCmd};
use super::fib::Fib;

enum CliMode<'a> {
  General,
  Interface(Arc<InterfaceView<'a>>),
}

fn generate_prompt(mode: &CliMode) -> String {
  let mut prompt = String::new();
  prompt += "blair-switch";
  match *mode {
    CliMode::Interface(ref interface) => {
      prompt = format!("{}({})", prompt, interface.name);
    },
    _ => (),
  };
  prompt += "#";
  prompt
}

pub fn cli_run(intfs_view: & HashMap<& str, Arc<InterfaceView>>, fib: &Arc<Fib>) {
  let mut rl = rustyline::DefaultEditor::new().unwrap();
  let mut mode = CliMode::General;

  loop {
    let prompt = generate_prompt(&mode);
    let input = rl.readline(&prompt);
    match mode {
      CliMode::General => {
        match input {
          Ok(cmd) => {
            if let Err(err) = rl.add_history_entry(&cmd) {
              eprintln!("Err: {}", err);
            }
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens.as_slice() {
              ["show", "interfaces"] => {
                let mut keys: Vec<_> = intfs_view.keys().cloned().collect();
                keys.sort();
                println!("Interfaces:\n==========\n");
                for intf in keys {
                  println!("{}\n", intfs_view[intf]);
                }
              },
              ["show", "fib"] => {
                println!("FIB:\n====\n{}", fib)
              }
              ["interface", intf_name] => {
                  if let Some(intf) = intfs_view.get(*intf_name) {
                    mode = CliMode::Interface(intf.clone());
                  } else {
                    println!("Interface {} not found", intf_name);
                  }
              },
              ["debug"] => {
                for (_, view) in intfs_view {
                  view.set_debug_mode(true);
                }
              },
              ["no", "debug"] => {
                for (_, view) in intfs_view {
                  view.set_debug_mode(false);
                }
              },
              ["counters", "reset"] => {
                for (_, view) in intfs_view {
                  view.reset_counters();
                }
              }
              ["config", "save"] => {
              },
              ["config", "load"] => {
              },
              ["exit"] => process::exit(0),
              _ => println!("Unknown command"),
            }
          },
          Err(ReadlineError::Interrupted) => println!("^C"),
          Err(ReadlineError::Eof) => process::exit(0),
          Err(_) => println!("No input"),
        }
      },
      CliMode::Interface(ref intf) => {
        match input {
          Ok(cmd) => {
            if let Err(err) = rl.add_history_entry(&cmd) {
              eprintln!("Err: {}", err);
            }
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens.as_slice() {
              ["show"] => println!("{}", intf),
              ["debug"] => intf.set_debug_mode(true), 
              ["no", "debug"] => intf.set_debug_mode(false),
              ["shutdown"] => {intf.send_cmd(IntfCmd::Shutdown);},
              ["no", "shutdown"] => {intf.send_cmd(IntfCmd::NoShutdown);},
              ["counters", "reset"] => {intf.reset_counters()},
              ["switchport", "mode", "access"] => {intf.send_cmd(IntfCmd::PortModeAccess)},
              ["switchport", "access", "vlan", vlan_str] => {
                match vlan_str.parse::<u16>() {
                  Ok(vlan) => {
                    if vlan > 0 && vlan < 4096 {
                      intf.send_cmd(IntfCmd::PortAccessVlan(vlan));
                    } else {
                      eprintln!("Error: invalid vlan \"{}\". Must be an number between 1 and 4095", vlan_str);
                    }
                  },
                  Err(err) => eprintln!("Error: invalid vlan format \"{}\". Must be number between 1 and 4095", vlan_str),
                }
              },
              ["no", "switchport", "access", "vlan"] => {intf.send_cmd(IntfCmd::PortModeAccess)},
              ["exit"] => mode = CliMode::General,
              _ => println!("Unknown command"),
            }
          },
          Err(ReadlineError::Interrupted) => println!("^C"),
          Err(ReadlineError::Eof) => mode = CliMode::General,
          Err(_) => println!("No input"),
        }
      },
    }
  }
}
