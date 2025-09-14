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

This is a rust project to help learn the language consisting in implementing a virtual switch in
the manner of open vswitch.

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

General mod
-----------


| Command | Action |
|---------|--------|
| show interfaces | Display all interfaces with their configurations |
| show fib | Display MAC table entries |
| debug | Enable debug mod on all interfaces |
| no debug | Disable debug mod on all interfaces |
| interface {interface\_name} | Set cli in "interface mode" on given interface |
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
| no switchport access vlan | Revert access port vlan to default (1) |
| exit | Exit interface mod and goes back to general mod |

Supported Features
==================

| Feature | Support |
|---------|---------|
| Frame flooding |  X |
| Frame switching | X |
| Port mirroring | |
| Vlan mapping | |
| 802.1q (Vlan) | WIP |
| 802.1ad (QinQ) | |
| 802.1ab (LLDP) | |
| 802.1ax (LACP) | |
| 802.1ak (MRVP) | |
| 802.1d  (STP) | |

