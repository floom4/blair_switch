use std::fmt;

use macaddr::MacAddr6;

pub struct Frame {
  src_mac: MacAddr6,
  dst_mac: MacAddr6,
  data: Vec<u8>,
}

impl Frame {
  pub fn build(bytes: &[u8]) -> Frame {
    if bytes.len() < 13 {
      panic!("Array too small to contain valid frame")
    }
    let dst_mac = MacAddr6::new(bytes[0],bytes[1],bytes[2],bytes[3], bytes[4], bytes[5]);
    let src_mac = MacAddr6::new(bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]);
    let data = bytes[12..].to_vec();

    Frame{dst_mac: dst_mac, src_mac: src_mac, data: data}
  }
}

impl fmt::Display for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut data_str = String::new();
    for byte in &self.data {
      data_str += &format!("{:X}", &byte);
    }
    write!(f, "Src MAC: {}, Dest MAC: {}, Data: {}", self.src_mac, self.dst_mac, data_str)
  }
}

