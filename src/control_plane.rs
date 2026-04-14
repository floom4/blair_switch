use std::sync::Arc;
use crossbeam_channel::Receiver;
use dashmap::DashMap;

use crate::network::interface::{Interface, InterfaceView, IntfCmd, DEFAULT_VLAN};
use crate::fib::Fib;

pub fn handle_control_plane<'a>(ing_intf: &mut Interface<'a>, rx: &Receiver<IntfCmd>,
  fib: &Arc<Fib<'a>>, mirrors: &DashMap<String, Vec<Arc<InterfaceView<'a>>>>,) {
  match rx.try_recv() {
    // TODO Delete intf_view from collection on shutdown
    Ok(IntfCmd::Shutdown) => {
      ing_intf.close();
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::NoShutdown) => {
      if let Err(err) = ing_intf.open() {
        eprintln!("Error: {}", err)
      }
    },
    Ok(IntfCmd::PortAccessVlan(vlan)) => {
      ing_intf.set_port_mode_access_vlan(vlan);
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::PortModeAccess) => {
      remove_monitoring_session(&ing_intf, &mirrors);
      ing_intf.set_port_mode_access_vlan(DEFAULT_VLAN);
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::PortModeVlanTunnel) => {
      ing_intf.set_port_mode_vlan_tunnel(DEFAULT_VLAN);
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::PortModeVlanTunnelSetVlan(vlan)) => {
      ing_intf.set_port_mode_vlan_tunnel(vlan);
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::PortModeTrunk) => {
      ing_intf.set_port_mode_trunk_vlan();
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Ok(IntfCmd::PortTrunkAddVlans(vlans)) => {
      ing_intf.add_trunk_allowed_vlan(&vlans);
    },
    Ok(IntfCmd::PortTrunkRemoveVlans(vlans)) => {
      ing_intf.remove_trunk_allowed_vlan(&vlans);
      for vlan in vlans {
        fib.remove_intf_vlan_entries(ing_intf.name.clone(), vlan);
      }
    },
    // TODO Delete intf_view from collection on monitoring
    Ok(IntfCmd::PortModeMonitoring(target)) => {
      // Remove eventual previous monitoring session
      remove_monitoring_session(&ing_intf, &mirrors);
      add_monitoring_session(&ing_intf, &target, &mirrors);
    },
    Ok(IntfCmd::PortAddVlanTranslation(vlan, new_vlan)) => {
      ing_intf.add_vlan_translation(vlan, new_vlan);
    }
    Ok(IntfCmd::PortRemoveVlanTranslation(vlan, new_vlan)) => {
      ing_intf.remove_vlan_translation(vlan, new_vlan);
      fib.remove_intf_vlan_entries(ing_intf.name.clone(), new_vlan);
    }
    Ok(IntfCmd::PortRemoveAllVlanTranslations) => {
      ing_intf.remove_all_vlan_translations();
      fib.remove_intf_entries(ing_intf.name.clone());
    },
    Err(crossbeam_channel::TryRecvError::Empty) => (),
    Err(err) => eprintln!("Error: {}", err),
  }
}

pub fn add_monitoring_session<'a>(monitoring_intf: &Interface<'a>, target_intf: &String,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView<'a>>>> ) {

  monitoring_intf.set_port_mode_monitoring(target_intf);
  mirrors.alter(target_intf, |_, mut v| {v.push(monitoring_intf.view.clone()); v});
}

pub fn remove_monitoring_session(monitoring_intf: &Interface,
  mirrors: &DashMap<String, Vec<Arc<InterfaceView>>> ) {

  if let Some(target_intf) = monitoring_intf.get_monitoring_targets() {
    mirrors.alter(&target_intf, |_, mut v| {
      let index = v.iter().position(|intf| intf.name == monitoring_intf.name).unwrap();
      v.remove(index);
      v
    });
  }
}
