enum State {
    Closed,
    Listen,
    // SynRcvd, // Syn Received
    // Estab,   // Established
}

// State of the Send Sequence Space (RFC 793 S3.2)
//         1         2          3          4
//    ----------|----------|----------|----------
//           SND.UNA    SND.NXT    SND.UNA
//                                +SND.WND
//
// 1 - old sequence numbers which have been acknowledged
// 2 - sequence numbers of unacknowledged data
// 3 - sequence numbers allowed for new data transmission
// 4 - future sequence numbers which are not yet allowed
struct SendSequenceSpace {
    // send unacknowledged
    una: usize,
    // send next
    nxt: usize,
    // send window
    wndm: usize,
    // send urgent pointers
    up: bool,
    // segment sequence number used for last window update
    wl1: usize,
    // segment acknowledgment number used for last window update
    wl2: usize,
    // initial send sequence number
    iss: usize,
}

// State of the Receive Sequence Space (RFC 793 S3.2)
//             1          2          3
//         ----------|----------|----------
//                RCV.NXT    RCV.NXT
//                          +RCV.WND
//
// 1 - old sequence numbers which have been acknowledged
// 2 - sequence numbers allowed for new reception
// 3 - future sequence numbers which are not yet allowed
struct ReceiveSequenceSpace {
    // receive next
    nxt: usize,
    // receive window
    wnd: usize,
    // Receive urgent pointer
    up: bool,
    // Initial receive sequence number
    irs: usize,
}

pub struct Connection {
    state: State,
    // send: SendSequenceSpace,
    // receive: ReceiveSequenceSpace,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            state: State::Listen,
        }
    }
}

impl Connection {
    pub fn on_packet<'a>(
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>,
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a [u8],
    ) {
        match self.state {
            State::Closed => {
                return Ok(0);
            }
            State::Listen => {
                // We only expected SYN packet in Listen mode
                if !tcph.syn() {
                    return Ok(0);
                }

                let mut buf = [0u8; 1500];

                // Send an ack
                let mut syn_ack = etherparse::TcpHeader::new(
                    tcph.destination_port(),
                    tcph.source_port(),
                    unimplemented!(),
                    unimplemented!(),
                );

                syn_ack.syn = true;
                syn_ack.ack = true;

                // Wrap this into an IP packet to send
                let mut ip = etherparse::Ipv4Header::new(
                    syn_ack.header_len(),
                    64,
                    etherparse::IpNumber::Tcp as u8,
                    iph.destination(),
                    iph.source(),
                );

                let unwritten = {
                    let mut unwritten = &mut buf[..];
                    ip.write(&mut unwritten);
                    syn_ack.write(&mut unwritten);
                    unwritten.len()
                };

                nic.send(&buf[..unwritten]);
            }
        }
        eprintln!(
            "{}:{} -> {}:{}, {}b of tcp",
            iph.source_addr(),
            tcph.source_port(),
            iph.destination_addr(),
            tcph.destination_port(),
            data.len(),
        );
    }
}
