use std::fmt;

use macaddr::MacAddr6;

pub struct Frame {
  pub src_mac: MacAddr6,
  pub dst_mac: MacAddr6,
  ether_type: u16,
  data: Vec<u8>,
}

impl Frame {
  pub fn build(bytes: &[u8], size: usize) -> Frame {
    if size < 13 {
      panic!("Array too small to contain valid frame")
    }
    let dst_mac = MacAddr6::new(bytes[0],bytes[1],bytes[2],bytes[3], bytes[4], bytes[5]);
    let src_mac = MacAddr6::new(bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]);
    let ether_type = ((bytes[12] as u16) << 8) | bytes[13] as u16;
    let data = bytes[14..size].to_vec();

    Frame{dst_mac: dst_mac, src_mac: src_mac, ether_type: ether_type, data: data}
  }
}

impl Frame {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(self.dst_mac.as_bytes());
    bytes.extend(self.src_mac.as_bytes());
    bytes.push((self.ether_type >> 8) as u8);
    bytes.push(self.ether_type as u8);
    bytes.extend(self.data.clone());
    bytes
  }

  pub fn get_eth_type(&self) -> &str {
    match self.ether_type {
      0x0800 => "IPv4",
      0x0806 => "ARP",
      0x86dd => "IPv6",
      _ => "UNKNOWN",
    }
  }

  pub fn is_broadcast(&self) -> bool {
    self.dst_mac.is_broadcast()
  }
}

impl fmt::Display for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut data_str = String::new();
    for byte in &self.data {
      data_str += &format!("{:X}", &byte);
    }
    write!(f, "Src MAC: {}, Dest MAC: {}, EthType: {}, Data: {}", self.src_mac, self.dst_mac, self.get_eth_type(), data_str)
  }
}

