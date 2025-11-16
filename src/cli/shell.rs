use std::collections::{HashMap, HashSet};
use std::fs;
use std::process;
use std::sync::Arc;

use arc_swap::ArcSwap;
use dashmap::{DashMap, Entry};
use rustyline::error::ReadlineError;

use rustyline::{CompletionType, Config, Context, Editor, Result};

use super::helper;
use super::commands;
use crate::network::interface::{InterfaceView, IntfCmd};
use crate::fib::Fib;

pub type IntfsViewMap<'a> = HashMap<&'a str, Arc<InterfaceView<'a>>>;

#[derive(PartialOrd, Ord, Clone, Eq, PartialEq, Hash)]
pub enum CliMode {
  General,
  Interface(String),
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

pub fn cli_run(intfs_view: &IntfsViewMap, fib: &Arc<Fib>,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>> ) {

  let mut rl = rustyline::Editor::new().unwrap();
  let mut mode = &ArcSwap::new(Arc::new(CliMode::General));
  let mut config = HashMap::new();
  let helper= helper::CommandHelper{mode: mode};
  rl.set_helper(Some(helper));

  'main: loop {
    let prompt = generate_prompt(mode.load().as_ref());
    let input = rl.readline(&prompt);

    match mode.load().as_ref() {
      CliMode::General => {
        match input {
          Ok(ref cmd) => {
            for available_cmd in commands::GENERAL_COMMANDS {
              if available_cmd.matches_pattern(&cmd) {
                available_cmd.run(intfs_view, fib, &mut mode,
                  intfs_view.values().next().unwrap().clone(), &mut config, cmd);
                if let Ok(cmd) = input && let Err(err) = rl.add_history_entry(&cmd) {
                  eprintln!("Err: {}", err);
                }
                continue 'main;
              }
            }
            println!("Unknown command")
          },
          Err(ReadlineError::Interrupted) => println!("^C"),
          Err(ReadlineError::Eof) => process::exit(0),
          Err(_) => println!("No input"),
        }
      },
      CliMode::Interface(if_name) => {
        match input {
          Ok(ref cmd) => {
            for available_cmd in commands::INTF_COMMANDS {
              if available_cmd.matches_pattern(&cmd) {
                available_cmd.run(intfs_view, fib, &mut mode,
                  intfs_view[&if_name[..]].clone(), &mut config, cmd);
                if let Ok(cmd) = input && let Err(err) = rl.add_history_entry(&cmd) {
                  eprintln!("Err: {}", err);
                }
                continue 'main;
              }
            }
            println!("Unknown command")
          },
          Err(ReadlineError::Interrupted) => println!("^C"),
          Err(ReadlineError::Eof) => mode.store(Arc::new(CliMode::General)),
          Err(_) => println!("No input"),
        }
      },
    }
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
