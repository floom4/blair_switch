use std::process;
use rustyline::error::ReadlineError;
use super::network::interface::Interface;

enum CliMode {
  General,
  Interface(String),
}

pub fn cli_run(interfaces: &Vec<Interface>) {
  let mut rl = rustyline::DefaultEditor::new().unwrap();
  let mut mode = CliMode::General;

  loop {
    let mut prompt = String::new();
    prompt += "blair-switch";
    match mode {
      CliMode::Interface(ref interface) => {
        prompt = format!("{}({})", prompt, interface);
      },
      _ => (),
    };
    prompt += "#";

    let input = rl.readline(&prompt);
    match mode {
      CliMode::General => {
        match input {
          Ok(cmd) => {
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens[0] {
              "interface" => {
                if tokens.len() >= 2 {
                  mode = CliMode::Interface(tokens[1].to_string());
                }
              },
              "show" => {
                match tokens[1] {
                  "interfaces" => {
                    println!("Interfaces:\n==========\n");
                    for interface in interfaces {
                      println!("{}\n", interface);
                    }
                  },
                  "mode" => {
                    println!("general");
                  },
                  &_ => println!("Unknown command"),
                }
              },
              "debug" => {
                for interface in interfaces {
                  interface.set_debug_mode(true);
                }
              },
              "no" => {
                match tokens[1] {
                  "debug" => {
                    for interface in interfaces {
                      interface.set_debug_mode(false);
                    }
                  },
                  &_ => println!("Unknown command"),
                }
              },
              "config" => {
                match tokens[1] {
                  "save" => {
                  },
                  "load" => {
                  },
                  &_ => println!("Unknown command"),
                }
              },
              "exit" => process::exit(0),
              &_ => println!("Unknown command"),
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
            let tokens : Vec<&str> = cmd.split(" ").collect();
            match tokens[0] {
              "show" => {
                match tokens[1] {
                  "mode" => {
                    println!("interface {}", interface);
                  },
                  &_ => println!("Unknown command"),
                }
              },
              //"debug" => interface.set_debug_mode(true),
              &_ => println!("Unknown command"),
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
