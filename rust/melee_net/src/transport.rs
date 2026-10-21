//! The pluggable socket (`gw_net_transport`), real UDP, and `gw_net_addr_parse`.

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

/// An IPv4 endpoint in host byte order (`gw_net_addr`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Addr {
    pub ip: u32,
    pub port: u16,
}

impl Addr {
    pub fn new(ip: u32, port: u16) -> Self {
        Addr { ip, port }
    }
    pub fn to_socket(self) -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::from(self.ip), self.port)
    }
}

/// The datagram socket a [`crate::Net`] owns.
pub trait Transport {
    /// Sends one datagram; the return value is ignored by the protocol (as in the C).
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32;
    /// Non-blocking: the datagram length, 0 when nothing is waiting, < 0 on a hard error.
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32;
    /// The bound port, 0 when not a real socket (`gw_net_udp_local_port`).
    fn local_port(&self) -> u16 {
        0
    }
}

/// Real UDP (the C's Winsock transport, here on `std::net`). Non-blocking.
pub struct UdpTransport {
    sock: UdpSocket,
    port: u16,
}

impl UdpTransport {
    /// `bind_ip` 0 = all interfaces; `bind_port` 0 = ephemeral (`gw_net_udp_open`).
    pub fn open(bind_ip: u32, bind_port: u16) -> std::io::Result<Self> {
        let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::from(bind_ip), bind_port))?;
        sock.set_nonblocking(true)?;
        let port = sock.local_addr().map(|a| a.port()).unwrap_or(0);
        Ok(UdpTransport { sock, port })
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        match self.sock.send_to(data, to.to_socket()) {
            Ok(n) => n as i32,
            Err(_) => -1,
        }
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        match self.sock.recv_from(buf) {
            Ok((n, std::net::SocketAddr::V4(a))) => {
                *from = Addr::new(u32::from(*a.ip()), a.port());
                n as i32
            }
            Ok(_) => 0, // IPv6 sender on an IPv4 socket: cannot happen; ignore
            Err(e) => {
                use std::io::ErrorKind;
                match e.kind() {
                    // WSAECONNRESET on UDP is an ICMP port-unreachable for an earlier send
                    ErrorKind::WouldBlock | ErrorKind::ConnectionReset => 0,
                    // WSAEMSGSIZE (10040) / EMSGSIZE: an oversized datagram, dropped like the C
                    _ if matches!(e.raw_os_error(), Some(10040) | Some(90)) => 0,
                    _ => -1,
                }
            }
        }
    }
    fn local_port(&self) -> u16 {
        self.port
    }
}

/// `"1.2.3.4:5678"` or `"1.2.3.4"` (then `default_port`), parsed like the C's
/// `sscanf("%u.%u.%u.%u:%u")`: anything after the fields read is ignored.
pub fn addr_parse(text: &str, default_port: u16) -> Option<Addr> {
    let b = text.as_bytes();
    let mut i = 0usize;
    // %u: skip whitespace, optional sign, digits (u32 wrap like strtoul is not reproduced; values
    // over 255 / 65535 are rejected anyway)
    fn num(b: &[u8], i: &mut usize) -> Option<u64> {
        while *i < b.len() && (b[*i] as char).is_ascii_whitespace() {
            *i += 1;
        }
        let start = *i;
        let mut v: u64 = 0;
        while *i < b.len() && b[*i].is_ascii_digit() {
            v = (v * 10 + (b[*i] - b'0') as u64).min(u64::MAX / 16);
            *i += 1;
        }
        if *i == start {
            None
        } else {
            Some(v)
        }
    }
    let mut f = [0u64; 4];
    for (k, slot) in f.iter_mut().enumerate() {
        if k > 0 {
            if i < b.len() && b[i] == b'.' {
                i += 1;
            } else {
                return None;
            }
        }
        *slot = num(b, &mut i)?;
    }
    let mut port = default_port as u64;
    if i < b.len() && b[i] == b':' {
        i += 1;
        if let Some(p) = num(b, &mut i) {
            port = p;
        }
    }
    if f.iter().any(|&x| x > 255) || port == 0 || port > 65535 {
        return None;
    }
    Some(Addr::new(((f[0] << 24) | (f[1] << 16) | (f[2] << 8) | f[3]) as u32, port as u16))
}
