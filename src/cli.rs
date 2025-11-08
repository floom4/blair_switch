use std::collections::{HashMap, HashSet};
use std::fs;
use std::process;
use std::sync::Arc;
use dashmap::{DashMap, Entry};
use rustyline::error::ReadlineError;

use super::network::interface::{InterfaceView, IntfCmd};
use super::fib::Fib;

#[derive(PartialOrd, Ord, Clone, Eq, PartialEq, Hash)]
enum CliMode {
  General,
  Interface(String), //Arc<InterfaceView<'a>>),
}

fn generate_prompt(mode: &CliMode) -> String {
  let mut prompt = String::new();
  prompt += "blair-switch";
  match *mode {
    CliMode::Interface(ref if_name) => {
      prompt = format!("{}({})", prompt, if_name);
    },
    _ => (),
  };
  prompt += "#";
  prompt
}

pub fn cli_run(intfs_view: &HashMap<&str, Arc<InterfaceView>>, fib: &Arc<Fib>,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>> ) {

  let mut rl = rustyline::DefaultEditor::new().unwrap();
  let mut mode = CliMode::General;
  let mut config = HashMap::new();

  loop {
    let prompt = generate_prompt(&mode);
    let input = rl.readline(&prompt);
    match mode.clone() {
      CliMode::General => {
        handle_general_cmd(&input, intfs_view, fib, &mut mode, &config);
      },
      CliMode::Interface(ref if_name) => {
        handle_interface_cmd(&input, intfs_view, fib, &mut mode, &mut config, if_name);
      },
    }
    if let Ok(cmd) = input && let Err(err) = rl.add_history_entry(&cmd) {
      eprintln!("Err: {}", err);
    }
  }
}

fn handle_general_cmd(input: &rustyline::Result<String>, intfs_view: &HashMap<&str, Arc<InterfaceView>>,
  fib: &Arc<Fib>, mode: &mut CliMode, config: &HashMap<CliMode, HashSet<String>>) {

  match input {
    Ok(cmd) => {
      /*if let Err(err) = rl.add_history_entry(&cmd) {
        eprintln!("Err: {}", err);
      }*/
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
            //if let Some(intf) = intfs_view.get(*intf_name) {
            if intfs_view.contains_key(*intf_name) {
              //mode = CliMode::Interface(intf.clone());
              *mode = CliMode::Interface(intf_name.to_string());
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
        ["show", "config"] => {
          print!("{}", config_to_str(config));
        }
        ["config", "save", filepath] => {
          if let Err(err) = fs::write(filepath, config_to_str(config)) {
            eprintln!("Error: Failure to write config at \"{}\": {}", filepath, err);
          } else {
            println!("Config saved at {}", filepath);
          }
        },
        ["config", "load", filepath] => {
        },
        ["exit"] => process::exit(0),
        _ => println!("Unknown command"),
      }
    },
    Err(ReadlineError::Interrupted) => println!("^C"),
    Err(ReadlineError::Eof) => process::exit(0),
    Err(_) => println!("No input"),
  }
}

fn handle_interface_cmd(input: &rustyline::Result<String>, intfs_view: &HashMap<&str, Arc<InterfaceView>>,
  fib: &Arc<Fib>, mode: &mut CliMode, config: &mut HashMap<CliMode, HashSet<String>>, if_name: &String) {

  let intf = &intfs_view[&if_name[..]];
  match input {
    Ok(cmd) => {
      //if let Err(err) = rl.add_history_entry(&cmd) {
       // eprintln!("Err: {}", err);
      //}
      let tokens : Vec<&str> = cmd.split(" ").collect();
      match tokens.as_slice() {
        ["show"] => println!("{}", intf),
        ["debug"] => {
          intf.set_debug_mode(true);
          config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
        },
        ["no", "debug"] => intf.set_debug_mode(false),
        ["shutdown"] => {
          intf.send_cmd(IntfCmd::Shutdown);
          config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
        },
        ["no", "shutdown"] => {intf.send_cmd(IntfCmd::NoShutdown);},
        ["counters", "reset"] => {intf.reset_counters()},
        ["switchport", "mode", "access"] => {intf.send_cmd(IntfCmd::PortModeAccess)},
        ["switchport", "mode", "monitor", src_intf_str] => {
            if intfs_view.contains_key(*src_intf_str) {
              intf.send_cmd(IntfCmd::PortModeMonitoring(src_intf_str.to_string()));
            } else {
              println!("Interface {} not found", src_intf_str);
            }
        }
        ["switchport", "access", "vlan", vlan_str] => {
          match vlan_str.parse::<u16>() {
            Ok(vlan) => {
              if vlan > 0 && vlan < 4096 {
                intf.send_cmd(IntfCmd::PortAccessVlan(vlan));
                config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
              } else {
                eprintln!("Error: invalid vlan \"{}\". Must be an number between 1 and 4095", vlan_str);
              }
            },
            Err(err) => eprintln!("Error: invalid vlan format \"{}\". Must be number between 1 and 4095", vlan_str),
          }
        },
        ["no", "switchport", "access", "vlan"] => {intf.send_cmd(IntfCmd::PortModeAccess)},
        ["exit"] => *mode = CliMode::General,
        _ => println!("Unknown command"),
      }
    },
    Err(ReadlineError::Interrupted) => println!("^C"),
    Err(ReadlineError::Eof) => *mode = CliMode::General,
    Err(_) => println!("No input"),
  }
}

fn config_to_str(config: &HashMap<CliMode, HashSet<String>>) -> String {
  let mut config_str = String::new();
  let mut keys: Vec<_> = config.keys().cloned().collect();
  keys.sort();
  for mode in keys {
    match mode {
      CliMode::General => {
      },
      CliMode::Interface(ref if_name) => {
        config_str += &format!("interface {}\n", if_name)[..];
      },
    }
    for cmd in &config[&mode] {
      config_str += &format!("  {}\n", cmd)[..];
    }
  }
  config_str
}
