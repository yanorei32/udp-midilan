# UDP MIDILAN Protocol Specification

<br />


## System Messages [0x00, ...]

### KeepAlive [0x00, 0x00]
The client will send an this message for connection stability if the connection is inactive for the 60s.

- Refresh the TTL of NAPT translation entries.
- Suppress the power-saving feature of the LTE modem.


<br />

## MIDI Messages [0x10, ...]
Messages beginning with `0x10` transmit midi messages.  
Raw MIDI messages follow after.

```
Examples
[0x10, 0x90, 0x3c, 0x40]: Note On  C4 (Velocity 64)
[0x10, 0x80, 0x3c, 0x00]: Note Off C4

```

