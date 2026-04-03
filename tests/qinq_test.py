#!/usr/bin/env python3

from utils import *

import sys


class TestQinQ:
  def test_qinq_to_access(self, ctx):
    # Test QinQ
    print("\nTest QinQ -> access")
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport mode dot1q-tunnel",
      "switchport access vlan 5",
      "exit",
      "interface if1-sw",
      "switchport access vlan 5",
      "exit"
    ])

    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exp_frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/Dot1Q(vlan=33)/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], exp_frame),
     expect_frame(hosts[2], exp_frame, failure=True),
     expect_frame(hosts[3], exp_frame, failure=True),
     expect_frame(hosts[4], exp_frame, failure=True),
     expect_frame(hosts[5], exp_frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)
    for exp in exps:
      exp.receive()

  def test_qinq_to_trunk(self, ctx):
    print("\nTest QinQ -> Trunk")
    ctx["switch"].send_cmds([
      "interface if1-sw",
      "switchport mode trunk",
      "switchport trunk vlans add 5",
      "exit"
    ])
    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exp_frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/Dot1Q(vlan=5)/Dot1Q(vlan=33)/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], exp_frame),
     expect_frame(hosts[2], exp_frame, failure=True),
     expect_frame(hosts[3], exp_frame, failure=True),
     expect_frame(hosts[4], exp_frame, failure=True),
     expect_frame(hosts[5], exp_frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)
    for exp in exps:
      exp.receive()

  def test_qinq_to_qinq(self, ctx):
    print("\nTest QinQ -> QinQ")
    ctx["switch"].send_cmds([
      "interface if5-sw",
      "switchport mode dot1q-tunnel",
      "switchport access vlan 5",
      "exit",
      "interface if1-sw",
      "switchport mode dot1q-tunnel",
      "switchport access vlan 5",
      "exit"
    ])

    frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exp_frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/Dot1Q(vlan=33)/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)
    exps = [
     expect_frame(hosts[1], exp_frame),
     expect_frame(hosts[2], exp_frame, failure=True),
     expect_frame(hosts[3], exp_frame, failure=True),
     expect_frame(hosts[4], exp_frame, failure=True),
     expect_frame(hosts[5], exp_frame, vlan=42, failure=True)
    ]

    send_frame(hosts[5], frame, vlan=33)
    for exp in exps:
      exp.receive()

if __name__ == "__main__":
  sys.exit(pytest.main([__file__, "-v", "-s"]))
