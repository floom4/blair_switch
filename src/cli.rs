use std::process;
use rustyline::error::ReadlineError;
use super::network::interface::Interface;

enum CliMode<'a> {
  General,
  Interface(&'a Interface),
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

pub fn cli_run(interfaces: &Vec<Interface>) {
  let mut rl = rustyline::DefaultEditor::new().unwrap();
  let mut mode = CliMode::General;

  loop {
    let prompt = generate_prompt(&mode);
    let input = rl.readline(&prompt);
    match mode {
      CliMode::General => {
        match input {
          Ok(cmd) => {
            rl.add_history_entry(&cmd);
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens.as_slice() {
              ["show", "interfaces"] => {
                println!("Interfaces:\n==========\n");
                for interface in interfaces {
                  println!("{}\n", interface);
                }
              },
              ["interface", intf_name] => {
                for intf in interfaces {
                  if intf.name == *intf_name {
                    mode = CliMode::Interface(intf);
                    break;
                  }
                }
                if matches!(mode, CliMode::General) {
                  println!("Interface {} not found", intf_name);
                }
              },
              ["debug"] => {
                for interface in interfaces {
                  interface.set_debug_mode(true);
                }
              },
              ["no", "debug"] => {
                for interface in interfaces {
                  interface.set_debug_mode(false);
                }
              },
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
      CliMode::Interface(ref interface) => {
        match input {
          Ok(cmd) => {
            rl.add_history_entry(&cmd);
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens.as_slice() {
              ["show"] => println!("{}", interface),
              ["debug"] => interface.set_debug_mode(true), 
              ["no", "debug"] => interface.set_debug_mode(false),
              ["disable"] => {},
              ["no", "disable"] => {},
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
