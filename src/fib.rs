use std::fmt;
use std::sync::Arc;
use dashmap::{DashMap, DashSet, Entry};
use macaddr::MacAddr6;
use std::time::{Duration, SystemTime};

use super::network::interface::InterfaceView;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct FibKey {
  vlan: u16,
  mac_addr: MacAddr6,
}

pub struct Fib<'a> {
  table: DashMap<FibKey, Arc<InterfaceView<'a>>>,
  reverse_table: DashMap<String, DashMap<u16, DashSet<MacAddr6>>>,
}

impl<'a> Fib<'a> {
  pub fn new() -> Self {
    Self {table: DashMap::new(), reverse_table: DashMap::new()}
  }

  pub fn lookup(&self, vlan: u16, mac: &MacAddr6) -> Option<Arc<InterfaceView<'a>>> {
     self.table.get(&FibKey{ vlan: vlan, mac_addr: *mac}).map(|g| Arc::clone(g.value()))
  }

  pub fn learn(&self, vlan: u16, mac: &MacAddr6, intf: Arc<InterfaceView<'a>>) {
    let fib_key = FibKey{vlan: vlan, mac_addr: *mac};
    match self.table.entry(fib_key.clone()) {
      Entry::Occupied(mut entry) => {
        if !Arc::ptr_eq(entry.get(), &intf) {
          *entry.get_mut() = intf;
        }
      },
      Entry::Vacant(entry) => {
        entry.insert(intf.clone());
        match self.reverse_table.entry(intf.name.clone()) {
          Entry::Occupied(mut rev_entry) => {
            match rev_entry.get_mut().entry(vlan) {
              Entry::Occupied(mut rev_vlan_entry) => {
                _ = rev_vlan_entry.get_mut().insert(*mac)
              }
              Entry::Vacant(entry) => {
                let set = DashSet::new();
                set.insert(*mac);
                entry.insert(set);
              }
            }
          },
          Entry::Vacant(rev_entry) => {
            let set = DashSet::new();
            set.insert(*mac);
            let vlan_map = DashMap::new();
            vlan_map.insert(vlan, set);
            rev_entry.insert(vlan_map);
          }
        }
      },
    }
  }

  pub fn remove_entry(&self, vlan: u16, mac: &MacAddr6) {
    let result = self.table.remove(&FibKey{vlan: vlan, mac_addr: *mac});
    if let Some(entry) = result {
      match self.reverse_table.entry(entry.1.name.clone()) {
        Entry::Occupied(mut intf_entry) => {
          match intf_entry.get_mut().entry(vlan) {
            Entry::Occupied(mut vlan_entry) => {
              _ = vlan_entry.get_mut().remove(mac)
            }
            Entry::Vacant(_) => ()
          }
        }
        Entry::Vacant(_) => ()
      }
    }
  }

  pub fn remove_intf_vlan_entries(&self, if_name: String, vlan: u16) {
    match self.reverse_table.entry(if_name) {
      Entry::Occupied(mut intf_entry) => {
        match intf_entry.get().entry(vlan) {
          Entry::Occupied(vlan_entry) => {
            for mac_entry in vlan_entry.get().iter() {
              self.table.remove(&FibKey{vlan: vlan, mac_addr: *mac_entry});
            }
          }
          Entry::Vacant(_) => ()
        }
        intf_entry.get_mut().remove(&vlan);
      }
      Entry::Vacant(_) => ()
    }
  }

  pub fn remove_intf_entries(&self, if_name: String) {
    match self.reverse_table.entry(if_name.clone()) {
      Entry::Occupied(intf_entry) => {
        for vlan_entry in intf_entry.get().iter() {
          for mac_entry in vlan_entry.value().iter() {
            self.table.remove(&FibKey{vlan: *vlan_entry.key(), mac_addr: *mac_entry});
          }
        }
      }
      Entry::Vacant(_) => ()
    }
    self.reverse_table.remove(&if_name);
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

impl fmt::Debug for Fib<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "table = {{\n{}\n}}\n", self)?;

    write!(f, "reverse = {{\n")?;
    for entry in &self.reverse_table {
      write!(f, "{}: [", entry.key())?;
      for vlan_entry in entry.value() {
        for mac_entry in vlan_entry.value().iter() {
          write!(f, "({},{}), ", vlan_entry.key(), *mac_entry)?;
        }
      }
      write!(f, "],\n")?;
    }
    write!(f, "}}\n")?;
    Ok(())
  }
}

impl fmt::Display for FibKey {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.vlan, self.mac_addr)
  }
}
