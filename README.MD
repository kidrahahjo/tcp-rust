# tcp-rust

TCP Implementation in Rust

# Setup

```bash
./run.sh

# Establish a TCP connection on the subnet `tun0` is pointed to
nc 192.168.0.2 80

# You can use tshark to capture packet data from that network.
tshark -i tun0
```
