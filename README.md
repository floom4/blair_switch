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


Most of the command do have a corresponding "no" command undoing it.
Example:
```
blair_switch# debug // Enabling debug mod on all interfaces
blair_switch# no debug // Disabling debug mod on all interfaces
```

General mod
-----------


| Command | Action |
|---------|--------|
| show interfaces | Display all interfaces with their configurations |
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
| exit | Exit interface mod and goes back to general mod |

Supported Features
==================

| Feature | Support |
|---------|---------|
| Frame flooding |  X |
| Frame switching | |
| Port mirroring | |
| Vlan mapping | |
| 802.1q (Vlan) | |
| 802.1ad (QinQ) | |
| 802.1ab (LLDP) | |
| 802.1ax (LACP) | |
| 802.1ak (MRVP) | |
| 802.1d  (STP) | |


