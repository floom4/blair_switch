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

switch.send_cmd("interface if1-sw")
switch.send_cmd("switchport access vlan 33")
switch.send_cmd("exit")
switch.send_cmd("interface if3-sw")
switch.send_cmd("switchport access vlan 33")
time.sleep(0.2)

#Test Vlan access
print("\nTest custom Vlan 33")
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

switch.send_cmd("exit\nshow interfaces\nshow fib")
print(switch.read_output())

switch.terminate()
