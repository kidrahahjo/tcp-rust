fn main() {
    let nic =
        tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("failed to create an interface");

    let mut buf = [0u8; 1504];

    loop {
        let nbytes = nic
            .recv(&mut buf[..])
            .expect("Did not receive bytes information");

        // With packet information, Each frame format is:
        // Flags: 2 bytes
        // Proto: 2 bytes
        // Raw protocol (IPv4, IPv6, etc) frame.

        let _flags = u16::from_be_bytes([buf[0], buf[1]]);

        // The next 2 bytes are the link level protocol information.
        let proto = u16::from_be_bytes([buf[2], buf[3]]);

        // EtherType 0x0800 refers to IPv4 and we're only concerened about IPv4 packets
        // Ignore any packet which is not an IPv4
        if proto != 0x0800 {
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes]) {
            Ok(packet) => {
                let src = packet.source_addr();
                let dest = packet.destination_addr();
                let proto = packet.protocol();

                if proto != 0x06 {
                    // Not a TCP Protocol
                    continue;
                }

                eprintln!(
                    "{} -> {}, {} bytes of protocol: {}",
                    src,
                    dest, 
                    packet.payload_len(),
                    proto,
                );
            }
            Err(e) => {
                eprintln!("Ignoring weird packet {:?}", e);
            }
        }
    }
}
