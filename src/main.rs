mod tcp;

use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

fn main() {
    let mut connections: HashMap<Quad, tcp::State> = Default::default();

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
            Ok(ipv4_header_slice) => {
                let src = ipv4_header_slice.source_addr();
                let dest = ipv4_header_slice.destination_addr();
                let proto = ipv4_header_slice.protocol();

                if proto != 0x06 {
                    // Not a TCP Protocol
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(
                    &buf[4 + ipv4_header_slice.slice().len()..],
                ) {
                    Ok(tcp_header_slice) => {
                        let datai =
                            4 + ipv4_header_slice.slice().len() + tcp_header_slice.slice().len();

                        connections
                            .entry(Quad {
                                src: (src, tcp_header_slice.source_port()),
                                dst: (dest, tcp_header_slice.destination_port()),
                            })
                            .or_insert(tcp::State {})
                            .on_packet(
                                &mut nic,
                                ipv4_header_slice,
                                tcp_header_slice,
                                &buf[datai..nbytes],
                            );
                    }
                    Err(e) => {
                        eprintln!("Error parsing TCP Header {:?}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("Ignoring weird packet {:?}", e);
            }
        }
    }
}
