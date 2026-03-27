#!/usr/bin/python3

from utils import *

switch = Switch()

# Test ARP flooding
print("Test ARP flooding")

frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[1].ip)
exps = [
  expect_frame(hosts[2], frame),
  expect_frame(hosts[3], frame),
  expect_frame(hosts[4], frame)
]

send_frame(hosts[1], frame)

for exp in exps:
  exp.receive()

# Test unicast
print("\nTest unicast")

frame = Ether(src=hosts[2].mac, dst=hosts[1].mac)/IP(dst=hosts[1].ip, src=hosts[2].ip)/ICMP()
exps = [
  expect_frame(hosts[1], frame),
  expect_frame(hosts[3], frame, failure=True),
  expect_frame(hosts[4], frame, failure=True)
]

send_frame(hosts[2], frame)

for exp in exps:
  exp.receive()

switch.send_cmds([
  "interface if1-sw",
  "switchport access vlan 5",
  "exit",
  "interface if3-sw",
  "switchport access vlan 5",
  "exit"
])
time.sleep(0.2)

# Test Vlan Access
print("\nTest custom Vlan 5")

frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[1].ip)
exps = [
  expect_frame(hosts[2], frame, failure=True),
  expect_frame(hosts[3], frame),
  expect_frame(hosts[4], frame, failure=True)
]

send_frame(hosts[1], frame)

for exp in exps:
  exp.receive()

print("\nTest default Vlan 1")

frame = Ether(src=hosts[4].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[4].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[4].ip)
exps = [
  expect_frame(hosts[1], frame, failure=True),
  expect_frame(hosts[2], frame),
  expect_frame(hosts[3], frame, failure=True)
]

send_frame(hosts[4], frame)

for exp in exps:
  exp.receive()

switch.terminate()
