#!/usr/bin/env python3

from utils import *

import sys

class TestTrunk:
  def test_ingress_trunk(self, ctx):
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport mode trunk",
      "switchport trunk vlans add 33",
      "switchport trunk vlans add 42",
      "exit",
      "interface if1-sw",
      "switchport access vlan 33",
      "exit",
      "interface if2-sw",
      "switchport access vlan 42",
      "exit"
    ])
    print("\nTest ingress trunk Vlan 33")

    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame),
     expect_frame(hosts[2], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)

    for exp in exps:
      exp.receive()

  def test_egress_trunk(self, ctx):
    print("\nTest egress trunk Vlan 33")
    
    frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[1].ip)
    exps = [
      expect_frame(hosts[2], frame, failure=True),
      expect_frame(hosts[3], frame, failure=True),
      expect_frame(hosts[4], frame, failure=True),
      expect_frame(hosts[5], frame, vlan=33),
      expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[1], frame)

    for exp in exps:
      exp.receive()

  def test_trunk_wrong_tag(self, ctx):
    print("\nTest Vlan Trunk with wrong Vlan tag")
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport trunk vlans remove 33",
      "exit",
    ])

    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)

    for exp in exps:
      exp.receive()

  def test_trunk_vlan_removal(self, ctx):
    print("\nTest other trunk vlan works after removal of one")
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=42)

    for exp in exps:
      exp.receive()

if __name__ == "__main__":
  sys.exit(pytest.main([__file__, "-v", "-s"]))
