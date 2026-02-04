#!/usr/bin/python3

from utils import *

switch = Switch()

# Test ARP flooding
print("Test ARP flooding")
frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[1].ip)

exp = expect_frame(hosts[2], raw(frame).hex())
exp2 = expect_frame(hosts[3], raw(frame).hex())
exp3 = expect_frame(hosts[4], raw(frame).hex())

send_frame(hosts[1], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

# Test unicast
print("\nTest unicast")
frame = Ether(src=hosts[2].mac, dst=hosts[1].mac)/IP(dst=hosts[1].ip, src=hosts[2].ip)/ICMP()
exp = expect_frame(hosts[1], raw(frame).hex())
exp2 = expect_frame(hosts[3], raw(frame).hex(), failure=True)
exp3 = expect_frame(hosts[4], raw(frame).hex(), failure=True)

send_frame(hosts[2], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

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

exp = expect_frame(hosts[2], raw(frame).hex(), failure=True)
exp2 = expect_frame(hosts[3], raw(frame).hex())
exp3 = expect_frame(hosts[4], raw(frame).hex(), failure=True)

send_frame(hosts[1], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

print("\nTest default Vlan 1")
frame = Ether(src=hosts[4].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[4].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[4].ip)

exp = expect_frame(hosts[1], raw(frame).hex(), failure=True)
exp2 = expect_frame(hosts[2], raw(frame).hex())
exp3 = expect_frame(hosts[3], raw(frame).hex(), failure=True)

send_frame(hosts[4], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

# Test Vlan Trunk
switch.send_cmds([
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
time.sleep(1)

print("\nTest ingress trunk Vlan 33")
frame = Ether(src=hosts[5].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[5].mac, hwdst="00:00:00:00:00:00", pdst=hosts[1].ip, psrc=hosts[5].ip)

exp = expect_frame(hosts[1], raw(frame).hex())
exp2 = expect_frame(hosts[2], raw(frame).hex(), failure=True)
exp3 = expect_frame(hosts[3], raw(frame).hex(), failure=True)
exp4 = expect_frame(hosts[4], raw(frame).hex(), failure=True)
exp5 = expect_frame(hosts[5], raw(frame).hex(), vlan=42, failure=True)

send_frame(hosts[5], raw(frame).hex(), vlan=33)

exp.receive()
exp2.receive()
exp3.receive()
exp4.receive()
exp5.receive()

print("\nTest egress trunk Vlan 33")
frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[5].ip, psrc=hosts[1].ip)

exp = expect_frame(hosts[2], raw(frame).hex(), failure=True)
exp2 = expect_frame(hosts[3], raw(frame).hex(), failure=True)
exp3 = expect_frame(hosts[4], raw(frame).hex(), failure=True)
exp4 = expect_frame(hosts[5], raw(frame).hex(), vlan=33)
exp5 = expect_frame(hosts[5], raw(frame).hex(), vlan=42, failure=True)

send_frame(hosts[1], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()
exp4.receive()
exp5.receive()

switch.send_cmd("show interfaces\nshow fib")
print(switch.read_output())

switch.terminate()
