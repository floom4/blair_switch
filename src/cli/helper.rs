use std::collections::HashSet;

use arc_swap::ArcSwap;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Helper, Context, Result};

use super::commands;
use super::shell::CliMode;

pub struct CommandHelper<'a> {
  pub mode: &'a ArcSwap<CliMode>,
}

impl Completer for CommandHelper<'_> {
  type Candidate = String;
  fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
    let mut candidates = HashSet::new();
    let tokens : Vec<&str> = line.split(" ").collect();

    let cmds = match self.mode.load().as_ref() {
      CliMode::General => commands::GENERAL_COMMANDS,
      CliMode::Interface(_) => commands::INTF_COMMANDS,
    };

    'main: for cmd in cmds {
      if tokens.len() > cmd.pattern.len() {
        continue
      }
      for i in 0..tokens.len()-1 {
        if tokens[i] != cmd.pattern[i] {
          continue 'main;
        }
      }
      if cmd.pattern[tokens.len() - 1].starts_with(tokens[tokens.len() - 1]) {
        _ = candidates.insert(cmd.pattern[tokens.len() - 1].to_string());
      }
    }

    if candidates.len() > 1 {
      println!("");
      commands::display_candidates_help_menu(self.mode.load().as_ref(), &tokens.join(" "));
    }

    return Ok((pos - tokens[tokens.len() - 1].len(), candidates.into_iter().collect()));
  }
}

impl Helper for CommandHelper<'_> {
}

impl Hinter for CommandHelper<'_> {
  type Hint = String;
}

impl Highlighter for CommandHelper<'_> {
}

impl Validator for CommandHelper<'_> {
}
