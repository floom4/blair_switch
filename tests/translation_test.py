#!/usr/bin/env python3

from utils import *

import sys

class TestVlanTranslation:
  def test_two_way_translation(self, ctx):
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport mode trunk",
      "switchport vlan translation 33 5",
      "switchport vlan translation 42 10",
      "exit",
      "interface if2-sw",
      "switchport access vlan 5",
      "exit",
      "interface if1-sw",
      "switchport access vlan 33",
      "exit",
      "interface if3-sw",
      "switchport access vlan 10",
      "exit"
    ])

    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)

    for exp in exps:
      exp.receive()

    frame = Ether(src=hosts[2].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[2].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[2], frame)

    for exp in exps:
      exp.receive()

  def test_translation_single_deletion(self, ctx):
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "no switchport vlan translation 33 5"
    ])

    # Check that packet are not translated anymore nor forwarded to previously matching interface
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[5].ip)
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

    ctx["switch"].send_cmds([
      "switchport trunk vlans add 33",
    ])

    # Check that packet are not translated nor forwarded to previously matching interface and dropped
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[5].ip)
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

    # Check that packet are forwarded and not translated
    frame = Ether(src=hosts[2].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[2].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[2], frame)

    for exp in exps:
      exp.receive()

    # Check that non deleted translation still works
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[3].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame, failure=True),
     expect_frame(hosts[3], frame),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=42)

    for exp in exps:
      exp.receive()

    ctx["switch"].send_cmds([
      "switchport trunk vlans remove 33",
      "exit"
    ])

  def test_all_translation_deletion(self, ctx):
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport vlan translation 33 5",
      "no switchport vlan translation",
      "exit"
    ])

    # Check that translation on 42 doesn't work on ingress
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[3].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=42)

    for exp in exps:
      exp.receive()

    # Check that translation on 42 doesn't work on egress
    frame = Ether(src=hosts[3].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[3].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[3].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[2], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[3], frame)

    for exp in exps:
      exp.receive()

    # Check that translation on 33 doesn't work on ingress
    frame = Ether(src=hosts[2].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[3].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[2].ip)
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

    # Check that translation on 33 doesn't work on egress
    frame = Ether(src=hosts[2].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[2].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[2].ip)
    exps = [
     expect_frame(hosts[1], frame, failure=True),
     expect_frame(hosts[3], frame, failure=True),
     expect_frame(hosts[4], frame, failure=True),
     expect_frame(hosts[5], frame, vlan=33, failure=True),
     expect_frame(hosts[5], frame, vlan=42, failure=True)
    ]

    send_frame(hosts[2], frame)

    for exp in exps:
      exp.receive()

if __name__ == "__main__":
  sys.exit(pytest.main([__file__, "-v", "-s"]))
