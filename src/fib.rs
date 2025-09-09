use std::fmt;
use std::sync::Arc;
use dashmap::{DashMap, Entry};
use macaddr::MacAddr6;

use super::network::interface::InterfaceView;

pub struct Fib<'a> {
  table: DashMap<MacAddr6, Arc<InterfaceView<'a>>>,
}

impl<'a> Fib<'a> {
  pub fn new() -> Self {
    Self {table: DashMap::new()}
  }

  pub fn lookup(&self, mac: &MacAddr6) -> Option<Arc<InterfaceView<'a>>> {
     self.table.get(mac).map(|g| Arc::clone(g.value()))
  }

  pub fn learn(&self, mac: &MacAddr6, intf: Arc<InterfaceView<'a>>) {
    match self.table.entry(*mac) {
      Entry::Occupied(mut entry) => {
        if !Arc::ptr_eq(entry.get(), &intf) {
          *entry.get_mut() = intf;
        }
      },
      Entry::Vacant(entry) => {
        entry.insert(intf);
      },
    }
  }
}

impl fmt::Display for Fib<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    for entry in &self.table {
      write!(f, "{} {}\n", entry.key(), entry.value().name)?
    }
    Ok(())
  }
}
