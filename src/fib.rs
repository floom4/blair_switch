use std::fmt;
use std::sync::Arc;
use dashmap::{DashMap, Entry};
use macaddr::MacAddr6;

use super::network::interface::InterfaceView;

#[derive(Hash, PartialEq, Eq)]
struct FibKey {
  vlan: u16,
  mac_addr: MacAddr6,
}

pub struct Fib<'a> {
  table: DashMap<FibKey, Arc<InterfaceView<'a>>>,
}

impl<'a> Fib<'a> {
  pub fn new() -> Self {
    Self {table: DashMap::new()}
  }

  pub fn lookup(&self, vlan: u16, mac: &MacAddr6) -> Option<Arc<InterfaceView<'a>>> {
     self.table.get(&FibKey{ vlan: vlan, mac_addr: *mac}).map(|g| Arc::clone(g.value()))
  }

  pub fn learn(&self, vlan: u16, mac: &MacAddr6, intf: Arc<InterfaceView<'a>>) {
    match self.table.entry(FibKey{vlan: vlan, mac_addr: *mac}) {
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

impl fmt::Display for FibKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.vlan, self.mac_addr)
  }
}
