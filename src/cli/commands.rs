use std::collections::{HashMap, HashSet};
use std::process;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::fib::Fib;
use crate::network::interface::{InterfaceView, IntfCmd};
use super::shell::{CliMode, IntfsViewMap};

pub struct Command<'a> {
  pub pattern: &'a [&'a str] ,
  description: &'a str,
  handler: fn(&IntfsViewMap, &Arc<Fib>, &ArcSwap<CliMode>, Arc<InterfaceView>, &mut HashMap<CliMode, HashSet<String>>, HashMap<String, String>),
}

pub const GENERAL_COMMANDS: &[Command] = &[
  Command {
    pattern: &["show", "interfaces"],
    description: "Show all interfaces information",
    handler: | intfs_view, _, _, _, _, _ | {
      let mut keys: Vec<_> = intfs_view.keys().cloned().collect();
      keys.sort();
      println!("Interfaces:\n==========\n");
      for intf in keys {
        println!("{}\n", intfs_view[intf]);
      }
    }
  },
  Command {
    pattern: &["show", "fib"],
    description: "Display FIB entries",
    handler: | _, fib, _, _, _, _ | {
      println!("FIB:\n====\n{}", fib)
    }
  },
  Command {
    pattern: &["interface", "<intf>"],
    description: "Enter in interfate configuration mode for given target",
    handler: | intfs_view, _, mode, _, _, args | {
      let intf_name = &args["intf"];
      if intfs_view.contains_key(&intf_name[..]) {
        mode.store(Arc::new(CliMode::Interface(intf_name.to_string())));
      } else {
        println!("Interface {} not found", intf_name);
      }
    },
  },
  Command {
    pattern: &["debug"],
    description: "Set entire device and all interfaces in debug mode",
    handler: | intfs_view, _, _, _, _, _ | {
      for (_, view) in intfs_view {
        view.set_debug_mode(true);
      }
    }
  },
  Command {
    pattern: &["no", "debug"],
    description: "Disable debug mode for entire device and interfaces",
    handler: | intfs_view, _, _, _, _, _ | {
      for (_, view) in intfs_view {
        view.set_debug_mode(false);
      }
    }
  },
  Command {
    pattern: &["counters", "reset"],
    description: "Reset all counters on the device",
    handler: | intfs_view, _, _, _, _, _ | {
      for (_, view) in intfs_view {
        view.reset_counters();
      }
    }
  },
  Command {
    pattern: &["show", "config"],
    description: "Display current running configuration",
    handler: | _, _, _, _, config, _ | {
      //TODO
      //print!("{}", config_to_str(config))
    },
  },
  Command {
    pattern: &["config", "save", "<filename>"],
    description: "Save current running configuration at <filename>",
    handler: | _, _, _, _, config, args | {
      //TODO
      /*let filepath = &args["filename"]
      if let Err(err) = fs::write(filepath, config_to_str(config)) {
        eprintln!("Error: Failure to write config at \"{}\": {}", filepath, err);
      } else {
        println!("Config saved at {}", filepath);
      }*/
    },
  },
  Command {
    pattern: &["config", "load", "<filename>"],
    description: "Replace running configuration with config stored at <filename>",
    handler: | _, _, _, _, _, _ | {
      //TODO
    }
  },
  Command {
    pattern: &["help"],
    description: "Display this help menu with available commandes",
    handler: | _, _, _, _, _, _ | {
      for cmd in GENERAL_COMMANDS {
        println!("{:<40} {}", cmd.pattern.join(" "), cmd.description)
      }
    },
  },
  Command {
    pattern: &["exit"],
    description: "Exit and shutdown program",
    handler: | _, _, _, _, _, _ | {
      process::exit(0)
    }
  },
];

pub const INTF_COMMANDS: &[Command] = &[
  Command {
    pattern: &["show"],
    description: "Show interface information",
    handler: | _, _, _, intf, _, _ | {
      println!("{}", intf)
    },
  },
  Command {
    pattern: &["debug"],
    description: "Enable debug mode on interface",
    handler: | _, _, mode, curr_intf, config, _ | {
      curr_intf.set_debug_mode(true);
      //config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
    },
  },
  Command {
    pattern: &["no", "debug"],
    description: "Disable debug mode on interface",
    handler: | _, _, _, intf, _, _ | {
      intf.set_debug_mode(false)
    },
  },
  Command {
    pattern: &["shutdown"],
    description: "Shut the interface off stopping ingress/egress traffic",
    handler: | _, _, _, curr_intf, _, _ | {
      curr_intf.send_cmd(IntfCmd::Shutdown);
      //config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
    }
  },
  Command {
    pattern: &["no", "shutdown"],
    description: "Bring interface up re-establshing ingress/egress traffic",
    handler: | _, _, _, intf, _, _ | {
      intf.send_cmd(IntfCmd::NoShutdown)
    },
  },
  Command {
    pattern: &["counters", "reset"],
    description: "Reset all interface counters to 0",
    handler: | _, _, _, intf, _, _ | {
      intf.reset_counters()
    },
  },
  Command {
    pattern: &["switchport", "mode", "access"],
    description: "Set interface in vlan access mode",
    handler: | _, _, _, intf, _, _ | {
      intf.send_cmd(IntfCmd::PortModeAccess)
    }
  },
  Command {
    pattern: &["switchport", "mode", "monitor", "<target_intf>"],
    description: "Set interface in monitor mode to mirror traffic from target interfaces",
    handler: | intfs_view, _, _, intf, _, args | {
      let target = &args["target_intf"];
      if intfs_view.contains_key(&target[..]) {
        intf.send_cmd(IntfCmd::PortModeMonitoring(target.to_string()));
      } else {
        println!("Interface {} not found", target);
      }
    }
  },
  Command {
    pattern: &["switchport", "access", "vlan", "<vlan>"],
    description: "Set vlan group for interface",
    handler: | _, _, _, intf, _, args | {
      let vlan_str = &args["vlan"];
      match vlan_str.parse::<u16>() {
        Ok(vlan) => {
          if vlan > 0 && vlan < 4096 {
            intf.send_cmd(IntfCmd::PortAccessVlan(vlan));
            //config.entry(mode.clone()).or_insert(HashSet::new()).insert(cmd.clone());
          } else {
            eprintln!("Error: invalid vlan \"{}\". Must be an number between 1 and 4095", vlan_str);
          }
        },
        Err(err) => eprintln!("Error: invalid vlan format \"{}\". Must be number between 1 and 4095", vlan_str),
      }
    },
  },
  Command {
    pattern: &["no", "switchport", "access", "vlan"],
    description: "Reset vlan group for interface to default group 1",
    handler: | _, _, _, intf, _, _ | {
      intf.send_cmd(IntfCmd::PortModeAccess)
    },
  },
  Command {
    pattern: &["help"],
    description: "Display this help menu with available commandes",
    handler: | _, _, _, _, _, _ | {
      for cmd in INTF_COMMANDS {
        println!("{:<40} {}", cmd.pattern.join(" "), cmd.description)
      }
    }
  },
  Command {
    pattern: &["exit"],
    description: "Exit interface configuration mode",
    handler: | _, _, mode, _, _, _ | {
      mode.store(Arc::new(CliMode::General))
    }
  },
];

impl Command<'_> {
  pub fn matches_pattern(&self, cmd: &String) -> bool {
    let tokens : Vec<&str> = cmd.split(" ").collect();
    if tokens.len() != self.pattern.len() {
      return false;
    }
    for i in 0..tokens.len() {
      if !(self.pattern[i].starts_with('<') && self.pattern[i].ends_with('>')) && tokens[i] != self.pattern[i] {
        return false;
      }
    }
    true
  }

  pub fn extract_args(&self, cmd: &String) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let tokens : Vec<&str> = cmd.split(" ").collect();
    for i in 0..self.pattern.len() {
      if self.pattern[i].starts_with('<') && self.pattern[i].ends_with('>') {
        result.insert(self.pattern[i].strip_prefix("<").unwrap().strip_suffix(">").unwrap().to_string(), tokens[i].to_string());
      }
    }
    result
  }

  pub fn run(&self, intfs_view: &IntfsViewMap, fib: &Arc<Fib>, mode: &ArcSwap<CliMode>, intf: Arc<InterfaceView>, conf: &mut HashMap<CliMode, HashSet<String>>, cmd: &String ) {
    let args = self.extract_args(cmd);
    (self.handler)(intfs_view, fib, mode, intf, conf, args)
  }
}
