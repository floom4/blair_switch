use rawsock::open_best_library;
use blair_switch::network::frame::Frame;

fn main() {
    let lib = open_best_library().expect("Could not open library");
    let interf_name = "if1-1-sw";
    let interf_name2 = "if2-1-sw";
    let mut interf = lib.open_interface(&interf_name).expect("Could not open network interface");
    let mut interf2 = lib.open_interface(&interf_name2).expect("Could not open network interface");
    println!("Interface opened, data link: {}", interf.data_link());
    loop {
      let mut packet = interf.receive().expect("Could not receive packet").to_vec();
      let frame = Frame::build(&packet);
      println!("Received frame: intf: {}, {}", interf_name, frame);
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
