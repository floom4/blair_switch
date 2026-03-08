```
           ██▓███   ██▀███   ▒█████   ▄▄▄██▀▀▀▓█████  ▄████▄  ▄▄▄█████▓
          ▓██░  ██▒▓██ ▒ ██▒▒██▒  ██▒   ▒██   ▓█   ▀ ▒██▀ ▀█  ▓  ██▒ ▓▒
          ▓██░ ██▓▒▓██ ░▄█ ▒▒██░  ██▒   ░██   ▒███   ▒▓█    ▄ ▒ ▓██░ ▒░
          ▒██▄█▓▒ ▒▒██▀▀█▄  ▒██   ██░▓██▄██▓  ▒▓█  ▄ ▒▓▓▄ ▄██▒░ ▓██▓ ░ 
          ▒██▒ ░  ░░██▓ ▒██▒░ ████▓▒░ ▓███▒   ░▒████▒▒ ▓███▀ ░  ▒██▒ ░ 
          ▒▓▒░ ░  ░░ ▒▓ ░▒▓░░ ▒░▒░▒░  ▒▓▒▒░   ░░ ▒░ ░░ ░▒ ▒  ░  ▒ ░░   
          ░▒ ░       ░▒ ░ ▒░  ░ ▒ ▒░  ▒ ░▒░    ░ ░  ░  ░  ▒       ░    
          ░░         ░░   ░ ░ ░ ░ ▒   ░ ░ ░      ░   ░          ░      
            ░         ░ ░   ░   ░      ░  ░░ ░               
                                           ░                 

 ▄▄▄▄    ██▓    ▄▄▄       ██▓ ██▀███       ██████  █     █░ ██▓▄▄▄█████▓ ▄████▄   ██░ ██ 
▓█████▄ ▓██▒   ▒████▄    ▓██▒▓██ ▒ ██▒   ▒██    ▒ ▓█░ █ ░█░▓██▒▓  ██▒ ▓▒▒██▀ ▀█  ▓██░ ██▒
▒██▒ ▄██▒██░   ▒██  ▀█▄  ▒██▒▓██ ░▄█ ▒   ░ ▓██▄   ▒█░ █ ░█ ▒██▒▒ ▓██░ ▒░▒▓█    ▄ ▒██▀▀██░
▒██░█▀  ▒██░   ░██▄▄▄▄██ ░██░▒██▀▀█▄       ▒   ██▒░█░ █ ░█ ░██░░ ▓██▓ ░ ▒▓▓▄ ▄██▒░▓█ ░██ 
░▓█  ▀█▓░██████▒▓█   ▓██▒░██░░██▓ ▒██▒   ▒██████▒▒░░██▒██▓ ░██░  ▒██▒ ░ ▒ ▓███▀ ░░▓█▒░██▓
░▒▓███▀▒░ ▒░▓  ░▒▒   ▓▒█░░▓  ░ ▒▓ ░▒▓░   ▒ ▒▓▒ ▒ ░░ ▓░▒ ▒  ░▓    ▒ ░░   ░ ░▒ ▒  ░ ▒ ░░▒░▒
▒░▒   ░ ░ ░ ▒  ░ ▒   ▒▒ ░ ▒ ░  ░▒ ░ ▒░   ░ ░▒  ░ ░  ▒ ░ ░   ▒ ░    ░      ░  ▒    ▒ ░▒░ ░
 ░    ░   ░ ░    ░   ▒    ▒ ░  ░░   ░    ░  ░  ░    ░   ░   ▒ ░  ░      ░         ░  ░░ ░
 ░          ░  ░     ░  ░ ░     ░              ░      ░     ░           ░ ░       ░  ░  ░
      ░                                                                 ░                

```

This is a project to learn rust language which aim to implement a virtual switch.

Usage:
======

```
Usage:
	blair_switch [OPTIONS] [INTERFACES...]
Arguments:
	INTERFACES	List of interfaces to attach to the switch
Options:
	-h, --help	Show this help message
```

Quick simple virtual topology setup and test:
```
sh# make init # Creates namespaces for switch and 4 hosts with corresponding links
sh# make run &# Runs switch in "sw" namespace with corresponding interfaces as arguments
sh# scripts/host-exec host2 arping -c 1 192.168.10.11
ARPING 192.168.10.11 from 192.168.10.12 if2-host2
Unicast reply from 192.168.10.11 [8A:D4:28:A8:87:6E]  0.693ms
Sent 1 probes (1 broadcast(s))
Received 1 response(s)
```

CLI
===

The command line interface has multiple mods:
  - General mod: General configuration of the switch
  - Interface mod: Configuration of the selectet interface

Example:
```
blair-switch# // General mode prompt
blair-switch# interface Port1 // Switching to interface mod on Port1
blair-switch(Port1)# // Intreface mode prompt
```


Most of the command do have a corresponding "no" command undoing it.
Example:
```
blair-switch# debug // Enabling debug mod on all interfaces
blair-switch# no debug // Disabling debug mod on all interfaces
```

Auto-complete as well as a listing of available commands is available when pressing tab.

General mod
-----------


| Command | Action |
|---------|--------|
| show interfaces | Display all interfaces with their configurations |
| show fib | Display MAC table entries |
| debug | Enable debug mod on all interfaces |
| no debug | Disable debug mod on all interfaces |
| interface {interface\_name} | Set cli in "interface mode" on given interface |
| help | Display available commands |
| exit | Exit program |


Interface mod
-------------

| Command | Action |
|---------|--------|
| show | Display interface configuration & counters |
| debug | Enable debug mod on interface |
| no debug | Disable debug mod on interface |
| shutdown | Disable interface |
| no shutdown | Enable interface |
| switchport mode vlan | Change interface mode to access port |
| switchport access vlan {vlan\_id} | Change access port vlan to {vlan\_id}. vlan\_id must be a number between 1 and 4095 |
| switchport mode trunk | Set interface in Vlan trunk mode |
| switchport trunk vlans add {vlan} | Add allowed vlans for interface |
| switchport trunk vlans remove {vlans} | Remove allowed vlans for interface |
| switchport mode dot1q-tunnel | Set interface in Vlan tunnel mode |
| no switchport trunk vlans | Remove all allowed vlans for interface |
| switchport mode monitor {if\_name}| Configure interface to mirror egress on given port |
| no switchport access vlan | Revert access port vlan to default (1) |
| help | Display available commands |
| exit | Exit interface mod and goes back to general mod |

Supported Features
==================

| Feature | Support |
|---------|---------|
| Frame flooding | X |
| Frame switching | X |
| Basic Port mirroring | X |
| Advanced Port mirroring | |
| Vlan mapping | |
| 802.1q (Vlan) | X |
| 802.1ad (QinQ) | X |
| 802.1ab (LLDP) | |
| 802.1ax (LACP) | |
| 802.1ak (MRVP) | |
| 802.1d  (STP) | |

