#!/usr/bin/python3

from utils import *

switch = Switch()

# Test QinQ
print("\nTest QinQ -> access")
switch.send_cmds([
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

print("\nTest QinQ -> Trunk")
switch.send_cmds([
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

print("\nTest QinQ -> QinQ")
switch.send_cmds([
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

switch.terminate()
