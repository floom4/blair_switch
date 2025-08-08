use rawsock::open_best_library;
use macaddr::MacAddr6;

struct Frame {
  src_mac: MacAddr6,
  dest_mac: MacAddr6,
  data: Vec<u8>,
}

fn main() {
    let lib = open_best_library().expect("Could not open library");
    let interf_name = "if1-1-sw";
    let interf_name2 = "if2-1-sw";
    let mut interf = lib.open_interface(&interf_name).expect("Could not open network interface");
    let mut interf2 = lib.open_interface(&interf_name2).expect("Could not open network interface");
    println!("Interface opened, data link: {}", interf.data_link());
    loop {
      let mut packet = interf.receive().expect("Could not receive packet").to_vec();
      let frame = Frame{dest_mac: MacAddr6::new(packet[0],packet[1],packet[2],packet[3], packet[4], packet[5]), src_mac: MacAddr6::new(packet[6], packet[7], packet[8], packet[9], packet[10], packet[11]), data: Vec<u8>::new(packet[12..])};
      frame.data.clone_from_slice(&packet[12..]);
      println!("Received packet: intf: {}, Src MAC: {}, Dest MAC: {}, Data: {:?}", interf_name, frame.src_mac, frame.dest_mac, packet);
      packet[6] = 0x4a;
      packet[7] = 0x80;
      packet[8] = 0x2f;
      packet[9] = 0x78;
      packet[10] = 0x03;
      packet[11] = 0xec;
      interf2.send(&packet);
      println!("Packet sent to {}", interf_name2);
    }
}
