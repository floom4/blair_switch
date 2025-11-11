use super::shell::CliMode;

struct Command<'a> {
  pattern: &'a [&'a str] ,
  description: &'a str,
  //handler: fn 
}

const GENERAL_COMMANDS: &[Command] = &[
  Command {
    pattern: &["show", "interfaces"],
    description: "Show all interfaces information",
  },
  Command {
    pattern: &["show", "fib"],
    description: "Display FIB entries",
  },
  Command {
    pattern: &["interface", "<intf>"],
    description: "Enter in interfate configuration mode for given target",
  },
  Command {
    pattern: &["debug"],
    description: "Set entire device in debug mode",
  },
  Command {
    pattern: &["debug"],
    description: "Set entire device and all interfaces in debug mode",
  },
  Command {
    pattern: &["no", "debug"],
    description: "Disable debug mode for entire device and interfaces",
  },
  Command {
    pattern: &["counters", "reset"],
    description: "Reset all counters on the device",
  },
  Command {
    pattern: &["show", "config"],
    description: "Display current running configuration",
  },
  Command {
    pattern: &["config", "save", "<filename>"],
    description: "Save current running configuration at <filename>",
  },
  Command {
    pattern: &["config", "load", "<filename>"],
    description: "Replace running configuration with config stored at <filename>",
  },
  Command {
    pattern: &["help"],
    description: "Display this help menu with available commandes",
  },
  Command {
    pattern: &["exit"],
    description: "Exit and shutdown program",
  },
];

const INTF_COMMANDS: &[Command] = &[
  Command {
    pattern: &["show"],
    description: "Show interface information",
  },
  Command {
    pattern: &["debug"],
    description: "Enable debug mode on interface",
  },
  Command {
    pattern: &["no", "debug"],
    description: "Disable debug mode on interface",
  },
  Command {
    pattern: &["shutdown"],
    description: "Shut the interface off stopping ingress/egress traffic",
  },
  Command {
    pattern: &["no", "shutdown"],
    description: "Bring interface up re-establshing ingress/egress traffic",
  },
  Command {
    pattern: &["counters", "reset"],
    description: "Reset all interface counters to 0",
  },
  Command {
    pattern: &["switchport", "mode", "access"],
    description: "Set interface in vlan access mode",
  },
  Command {
    pattern: &["switchport", "mode", "monitor", "<target_intf>"],
    description: "Set interface in monitor mode to mirror traffic from target interfaces",
  },
  Command {
    pattern: &["switchport", "access", "vlan", "<vlan>"],
    description: "Set vlan group for interface",
  },
  Command {
    pattern: &["no", "switchport", "access", "vlan"],
    description: "Reset vlan group for interface to default group 1",
  },
  Command {
    pattern: &["help"],
    description: "Display this help menu with available commandes",
  },
  Command {
    pattern: &["exit"],
    description: "Exit interface configuration mode",
  },
];

pub fn display_help_menu(mode: &CliMode) {
  let mut cmds =  match mode {
    CliMode::General => GENERAL_COMMANDS,
    CliMode::Interface(_) => INTF_COMMANDS,
    _ => panic!(),
  };
  for cmd in cmds {
    println!("{:<40} {}", cmd.pattern.join(" "), cmd.description)
  }
}
