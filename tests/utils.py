import pytest
import select
import subprocess
import time
import textwrap
from scapy.all import ARP, Dot1Q, Ether, IP, ICMP, raw
from inspect import cleandoc

container_prefix = "bs-lab"

class Host:
  def __init__(self, name, iface, mac, ip):
    self.name = name
    self.iface = iface
    self.mac = mac
    self.ip = ip

hosts = [
  None,
  Host("host1", "if1-host1", "aa:aa:aa:aa:aa:aa", "192.168.10.11"),
  Host("host2", "if2-host2", "aa:aa:aa:aa:aa:bb", "192.168.10.12"),
  Host("host3", "if3-host3", "aa:aa:aa:aa:aa:cc", "192.168.10.13"),
  Host("host4", "if4-host4", "aa:aa:aa:aa:aa:dd", "192.168.10.14"),
  Host("host5", "if5-host5", "aa:aa:aa:aa:aa:ee", "192.168.10.15"),
]

@pytest.fixture(scope="module")
def ctx():
  switch = Switch()
  
  yield {
    "switch": switch
  }

  switch.terminate()

def run_cmd(cmd):
  subprocess.run(cmd, shell=True, check=True)

def run_cmd_on_host(host, cmd):
  run_cmd(f"docker exec {container_prefix}-{host} {cmd}")

class Switch:
  def __init__(self):
    cmd = ["docker", "exec", "-i", f"{container_prefix}-sw", "/app/blair_switch" ]
    for host in hosts[1:]:
      cmd.append(host.iface.split('-')[0] + "-sw")
    self.process = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)

  def send_cmd(self, cmd):
    self.process.stdin.write(cmd.strip() + "\n")
    self.process.stdin.flush()

  def send_cmds(self, cmds):
    for cmd in cmds:
      self.send_cmd(cmd)
      time.sleep(0.2)
    time.sleep(0.3)

  def read_output(self):
    timeout = 0.1
    output = ""

    while True:
      r, _, _ = select.select([self.process.stdout], [], [], timeout)
      if r:
        chunk = self.process.stdout.buffer.read1(4096)
        if not chunk:
          break
        output += chunk.decode('utf-8')
      else:
        break

    return output

  def terminate(self):
    self.process.stdin.close()
    self.process.terminate()
    self.process.wait(timeout=0.2)


class Receiver:
  def __init__(self, host, vlan, expected, failure, process):
    self.host = host
    self.interface = self.host.iface
    if vlan:
      self.interface += f".{vlan}"
    self.expected = expected
    self.process = process
    self.failure = failure

  def receive(self):
    self.process.wait()
    ok = self.process.returncode == 0
    if ok == self.failure:
      print(f"{self.interface}@{self.host.name} FAILURE")
      print("Return Code: ", self.process.returncode)
      stdout, stderr = self.process.communicate()
      print("==== STDOUT =====\n", stdout)
      print("==== STDERR =====\n", stderr)
      assert(False)
    print(f"{self.interface}@{self.host.name} OK")

def send_frame(host, frame, vlan=None):
  data = raw(frame).hex()
  interface=host.iface
  if vlan:
    interface += f".{vlan}"

  time.sleep(0.5) #Wait for all receiver to be setup

  script = cleandoc(f"""
    from scapy.all import sendp
    frame = bytes.fromhex("{data}")
    sendp(frame, iface="{interface}")
  """)
  run_cmd_on_host(host.name, f"python3 -c \'{script}\'")

def expect_frame(host, frame, timeout = 5, failure=False, vlan=None):
  expected_bytes = raw(frame).hex()
  interface=host.iface
  if vlan:
    interface += f".{vlan}"

  script = cleandoc(f"""
    from scapy.all import sniff
    import sys, binascii

    exp = binascii.unhexlify("{expected_bytes}")
    frames = sniff(iface="{interface}", timeout={timeout},
      stop_filter=lambda frame : bytes(frame) == exp, count=0)
    if not len(frames):
      print("No packets received")
      sys.exit(1)
    if bytes(frames[-1]) != exp:
      for frame in frames:
        print(frame)
      sys.exit(1)
    sys.exit(0)
  """)
  p = subprocess.Popen(
      f"docker exec -i {container_prefix}-{host.name} python3 -c \'{script}\'",
      stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, shell=True)
  return Receiver(host, vlan, expected_bytes, failure, p)
