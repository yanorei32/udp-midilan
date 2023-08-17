# UDP MIDILAN
It's a simple MIDI over UDP software.

## Architecture

```
MIDI  MIDI
 In   Out
 ^     ^
 |     |
udp-midilan server <Port>
 |     ^
 |     |
 v     |
udp-midilan client <ServerIP>:<Port>
 |     ^
 v     |
MIDI  MIDI
Out    In
```
For further specifications, see [protocol.md](docs/protocol.md).

### IMPORTANT
This software will send **one** MIDI message as **one** UDP packet.
It may impact system network performance.

