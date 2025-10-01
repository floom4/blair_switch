#!/usr/bin/python3

import subprocess
import time
import textwrap
from scapy.all import ARP, Ether, IP, ICMP, raw

class Host:
  def __init__(self, name, iface, mac, ip):
    self.name = name
    self.iface = iface
    self.mac = mac
    self.ip = ip

hosts = [
  None,
  Host("host1", "if1-host1", "aa:aa:aa:aa:aa:aa", "192.168.10.11"),
  Host("host2", "if2-host2", "bb:bb:bb:bb:bb:bb", "192.168.10.12"),
  Host("host3", "if3-host3", "cc:cc:cc:cc:cc:cc", "192.168.10.13"),
  Host("host4", "if4-host4", "dd:dd:dd:dd:dd:dd", "192.168.10.14"),
]

def start_switch():
  cmd = ["sudo", "scripts/host-exec", "sw", "target/debug/blair_switch" ]
  for host in hosts[1:]:
    cmd.append(host.iface.split('-')[0] + "-sw")
  p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
  return p

def run_cmd(cmd):
  subprocess.run(cmd, shell=True, check=True)

def run_cmd_on_host(host, cmd):
  run_cmd("sudo scripts/host-exec " + host + " " + cmd)


class Receiver:
  def __init__(self, host, expected, failure, process):
    self.host = host
    self.expected = expected
    self.process = process
    self.failure = failure

  def receive(self):
    self.process.wait()
    ok = self.process.returncode == 0
    if ok == self.failure:
      print(f"{self.host.iface}@{self.host.name} FAILURE")
      print("Return Code: ", self.process.returncode)
      stdout, stderr = self.process.communicate()
      print("==== STDOUT =====\n", stdout)
      print("==== STDERR =====\n", stderr)
      assert(False)
    print(f"{self.host.iface}@{self.host.name} OK")

def send_frame(host, data):
  time.sleep(1) #Wait for all receiver to be setup
  script = f"""
from scapy.all import sendp
frame = bytes.fromhex("{data}")
sendp(frame, iface="{host.iface}")
"""
  run_cmd_on_host(host.name, f"python3 - <<'PY'\n{script}\nPY")

def expect_frame(host, expected_bytes, timeout = 5, failure=False):
  script = f"""
from scapy.all import sniff
import sys, binascii
exp = binascii.unhexlify("{expected_bytes}")
frames = sniff(iface="{host.iface}", timeout={timeout}, stop_filter=lambda frame : bytes(frame) == exp, count=0)
if not len(frames):
  print("No packets received")
  sys.exit(1)
if bytes(frames[-1]) != exp:
  for frame in frames:
    print(frame)
  sys.exit(1)
sys.exit(0)
  """
  p = subprocess.Popen(
      f"sudo scripts/host-exec {host.name} python3 - << 'PY'\n{script}\nPY", stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, shell=True)
  return Receiver(host, expected_bytes, failure, p)
