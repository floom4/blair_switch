use std::fmt;
use macaddr::MacAddr6;

#[derive(Debug,Clone)]
struct Tag {
  tpid: u16,
  tci: u16,
}

impl Tag {
  pub fn parse(bytes: &[u8]) -> Self {
    debug_assert!(bytes.len() == 4);
    let tpid = ((bytes[0] as u16) << 8) | bytes[1] as u16;
    let tci = ((bytes[2] as u16) << 8) | bytes[3] as u16;
    Tag{tpid: tpid, tci: tci}
  }

  pub fn build(pcp : u8, dei: bool, vlan: u16) -> Tag {
    debug_assert!(vlan < 4096);
    let mut tci = 0 as u16;
    tci |= (pcp as u16) << 13;
    if dei {
      tci |= (1 as u16) << 12;
    }
    tci |= vlan;
    Tag{tpid: 0x8100, tci: tci}
  }
}

#[derive(Clone)]
pub struct Frame {
  pub dst_mac: MacAddr6,
  pub src_mac: MacAddr6,
  tag: Option<Tag>,
  ether_type: u16,
  data: Vec<u8>,
}

impl Frame {
  pub fn parse(bytes: &[u8], size: usize) -> Frame {
    if size < 13 {
      panic!("Array too small to contain valid frame")
    }
    let dst_mac = MacAddr6::new(bytes[0],bytes[1],bytes[2],bytes[3], bytes[4], bytes[5]);
    let src_mac = MacAddr6::new(bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]);
    let mut ether_type = ((bytes[12] as u16) << 8) | bytes[13] as u16;
    let mut cursor = 14;
    let mut tag = None;
    if ether_type == 0x8100 { // dot1q handling
      tag = Some(Tag::parse(&bytes[12..16]));
      ether_type = ((bytes[16] as u16) << 8) | bytes[17] as u16;
      cursor = 18;
    }
    let data = bytes[cursor..size].to_vec();

    Frame{dst_mac: dst_mac, src_mac: src_mac, tag: tag, ether_type: ether_type, data: data}
  }
}

impl Frame {
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(self.dst_mac.as_bytes());
    bytes.extend(self.src_mac.as_bytes());
    if let Some(tag) = &self.tag {
      bytes.push((tag.tpid >> 8) as u8);
      bytes.push(tag.tpid as u8);
      bytes.push((tag.tci >> 8) as u8);
      bytes.push(tag.tci as u8);
    }
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

  pub fn tag(&mut self, vlan: u16) {
    debug_assert!(vlan < 4096);
    self.tag = Some(Tag::build(0, false, vlan));
  }

  pub fn untag(&mut self) {
    self.tag = None
  }

  pub fn get_vlan(&self) -> u16 {
    if let Some(tag) = &self.tag {
      tag.tci & 0x1FFF
    } else {
      0
    }
  }
}

impl fmt::Display for Frame {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut data_str = String::new();
    for byte in &self.data {
      data_str += &format!("{:X}", &byte);
    }
    write!(f, "Src MAC: {}, Dest MAC: {}, ", self.src_mac, self.dst_mac);
    if let Some(tag) = &self.tag {
      write!(f, "Tag: {}, ", tag);
    }
    write!(f, "EthType: {}, Data: {}", self.get_eth_type(), data_str)
  }
}

impl fmt::Display for Tag{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{{ tpid: 0x{:X}, vlan: {}}}", self.tpid, self.tci & 0x1FFF)
  }
}
