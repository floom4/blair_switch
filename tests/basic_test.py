#!/usr/bin/python3

from utils import *

p = start_switch()

print("Test ARP flooding") # Test ARP flooding
frame = Ether(src=hosts[1].mac, dst="ff:ff:ff:ff:ff:ff")/ARP(hwsrc=hosts[1].mac, hwdst="00:00:00:00:00:00", pdst=hosts[2].ip, psrc=hosts[1].ip)

exp = expect_frame(hosts[2], raw(frame).hex())
exp2 = expect_frame(hosts[3], raw(frame).hex())
exp3 = expect_frame(hosts[4], raw(frame).hex())

send_frame(hosts[1], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

print("Test unicast") # Test unicast
frame = Ether(src=hosts[2].mac, dst=hosts[1].mac)/IP(dst=hosts[1].ip, src=hosts[2].ip)/ICMP()
exp = expect_frame(hosts[1], raw(frame).hex())
exp2 = expect_frame(hosts[3], raw(frame).hex(), failure=True)
exp3 = expect_frame(hosts[4], raw(frame).hex(), failure=True)

send_frame(hosts[2], raw(frame).hex())

exp.receive()
exp2.receive()
exp3.receive()

data = p.communicate(input='show interfaces\nshow fib')[0]
print(data)

p.terminate()
