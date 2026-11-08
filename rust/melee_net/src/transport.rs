//! The pluggable socket (`gw_net_transport`), real UDP, and `gw_net_addr_parse`.
//!
//! IPv4 is the C build's address type (`ip` in host byte order). IPv6 (net3) rides in `ip6`: non-zero means the
//! endpoint is an IPv6 address and `ip` is then a fold of it (only used as a hash input by the netcode).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs, UdpSocket};

/// A UDP endpoint (`gw_net_addr`, widened for IPv6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Addr {
    pub ip: u32,
    pub port: u16,
    /// the IPv6 address (big-endian as a number), 0 for IPv4
    pub ip6: u128,
}

impl Addr {
    pub fn new(ip: u32, port: u16) -> Self {
        Addr { ip, port, ip6: 0 }
    }
    pub fn new6(ip6: u128, port: u16) -> Self {
        let ip = (ip6 as u32) ^ ((ip6 >> 32) as u32) ^ ((ip6 >> 64) as u32) ^ ((ip6 >> 96) as u32);
        Addr { ip, port, ip6 }
    }
    pub fn is_v6(&self) -> bool {
        self.ip6 != 0
    }
    /// The same host on another port.
    pub fn with_port(self, port: u16) -> Self {
        Addr { port, ..self }
    }
    pub fn to_socket(self) -> SocketAddr {
        if self.is_v6() {
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(self.ip6), self.port, 0, 0))
        } else {
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(self.ip), self.port))
        }
    }
    /// An IPv4-mapped IPv6 address (`::ffff:a.b.c.d`) becomes plain IPv4.
    pub fn from_socket(a: SocketAddr) -> Self {
        match a {
            SocketAddr::V4(a) => Addr::new(u32::from(*a.ip()), a.port()),
            SocketAddr::V6(a) => match a.ip().to_ipv4_mapped() {
                Some(v4) => Addr::new(u32::from(v4), a.port()),
                None => Addr::new6(u128::from(*a.ip()), a.port()),
            },
        }
    }
}

impl std::fmt::Display for Addr {
    /// `1.2.3.4:5` / `[2001:db8::1]:5` (what `addr_resolve` reads back).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_socket())
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
    /// A send path another thread can own (early inputs' sampler thread), or None when this transport has none.
    fn thread_sender(&self) -> Option<ThreadSender> {
        None
    }
}

/// Cloned sockets that send from any thread: what the transport stack below the protocol would do with a datagram
/// (the relay's `GDMD` wrapping included), without its single-threaded state.
pub struct ThreadSender {
    pub sock: UdpSocket,
    pub sock6: Option<UdpSocket>,
    /// Relayed through the server: wrap as the rendezvous client does and send to it.
    pub relay: Option<Addr>,
}

impl ThreadSender {
    pub fn send(&self, to: &Addr, data: &[u8]) -> bool {
        let (dst, buf);
        match self.relay {
            Some(srv) => {
                let mut b = b"GDMD".to_vec();
                b.extend_from_slice(data);
                buf = b;
                dst = srv;
            }
            None => {
                buf = data.to_vec();
                dst = *to;
            }
        }
        let r = if dst.is_v6() {
            match &self.sock6 {
                Some(s) => s.send_to(&buf, dst.to_socket()),
                None => return false,
            }
        } else {
            self.sock.send_to(&buf, dst.to_socket())
        };
        r.is_ok()
    }
}

/// Real UDP (the C's Winsock transport, here on `std::net`). Non-blocking. Bound to all interfaces it is two sockets
/// on the same port, IPv4 and IPv6 (Windows' IPv6 sockets are V6ONLY by default and std cannot clear that); a
/// datagram leaves through the socket of its family.
pub struct UdpTransport {
    sock: UdpSocket,
    sock6: Option<UdpSocket>,
    port: u16,
    /// [`UdpTransport::stamp_arrivals`]: reader threads that take each datagram off the socket as it lands.
    rx: Option<RxThreads>,
}

thread_local! {
    static RX_AT: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

/// When the datagram the last `recv` on this thread returned reached this process (None: unknown, treat as now).
/// Set by [`UdpTransport`] when it stamps arrivals, and by wrappers that hold datagrams back (simulated lag) to the
/// datagram's simulated arrival. Display-only: the network round trip shown to the player, never game timing.
pub fn rx_stamp() -> Option<Instant> {
    RX_AT.with(|c| c.get())
}
pub fn set_rx_stamp(t: Option<Instant>) {
    RX_AT.with(|c| c.set(t));
}
/// Milliseconds since the last returned datagram arrived (0 when unknown).
pub fn rx_wait_ms() -> u32 {
    rx_stamp().map_or(0, |t| t.elapsed().as_millis().min(u32::MAX as u128) as u32)
}

type RxQueue = Arc<Mutex<VecDeque<(i32, Addr, Vec<u8>, Instant)>>>;

struct RxThreads {
    q: RxQueue,
    stop: Arc<AtomicBool>,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl RxThreads {
    fn start(socks: Vec<UdpSocket>) -> std::io::Result<RxThreads> {
        let q: RxQueue = Arc::default();
        let stop = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();
        for s in socks {
            // the main thread never reads this socket again, so blocking mode (shared by the clones or not) is safe
            s.set_nonblocking(false)?;
            s.set_read_timeout(Some(Duration::from_millis(50)))?;
            let (q, stop) = (q.clone(), stop.clone());
            threads.push(std::thread::Builder::new().name("udp-rx".into()).spawn(move || {
                let mut buf = vec![0u8; 65536];
                while !stop.load(Ordering::Relaxed) {
                    let r = s.recv_from(&mut buf);
                    let at = Instant::now();
                    let timed_out = matches!(&r, Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut));
                    if timed_out {
                        continue;
                    }
                    let mut from = Addr::default();
                    let n = recv_kind(r, &mut from);
                    if n == 0 {
                        continue; // ICMP reset / oversized: dropped, as the non-blocking read does
                    }
                    let d = if n > 0 { buf[..n as usize].to_vec() } else { Vec::new() };
                    q.lock().unwrap().push_back((n, from, d, at));
                    if n < 0 {
                        break;
                    }
                }
            })?);
        }
        Ok(RxThreads { q, stop, threads })
    }
}

impl Drop for RxThreads {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

fn recv_kind(r: std::io::Result<(usize, SocketAddr)>, from: &mut Addr) -> i32 {
    match r {
        Ok((n, a)) => {
            *from = Addr::from_socket(a);
            n as i32
        }
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

impl UdpTransport {
    /// `bind_ip` 0 = all interfaces (IPv4, plus IPv6 on the same port when the system has it); `bind_port` 0 =
    /// ephemeral (`gw_net_udp_open`).
    pub fn open(bind_ip: u32, bind_port: u16) -> std::io::Result<Self> {
        let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::from(bind_ip), bind_port))?;
        sock.set_nonblocking(true)?;
        let port = sock.local_addr().map(|a| a.port()).unwrap_or(0);
        let sock6 = if bind_ip == 0 { Self::bind6(Ipv6Addr::UNSPECIFIED, port).ok() } else { None };
        Ok(UdpTransport { sock, sock6, port, rx: None })
    }

    /// Bind one address of either family (`--np-bind`). An IPv6 address gets no IPv4 path (its IPv4 socket is a
    /// loopback placeholder).
    pub fn open_at(ip: IpAddr, bind_port: u16) -> std::io::Result<Self> {
        match ip {
            IpAddr::V4(v4) => Self::open(u32::from(v4), bind_port),
            IpAddr::V6(v6) => {
                let s6 = Self::bind6(v6, bind_port)?;
                let port = s6.local_addr().map(|a| a.port()).unwrap_or(0);
                let sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
                sock.set_nonblocking(true)?;
                Ok(UdpTransport { sock, sock6: Some(s6), port, rx: None })
            }
        }
    }

    fn bind6(ip: Ipv6Addr, port: u16) -> std::io::Result<UdpSocket> {
        let s = UdpSocket::bind(SocketAddrV6::new(ip, port, 0, 0))?;
        s.set_nonblocking(true)?;
        Ok(s)
    }

    /// Take datagrams off the socket on reader threads the moment they land, stamping each with its arrival
    /// ([`rx_stamp`]). `recv` stays non-blocking and returns the same datagrams in the same order: what a frame's
    /// poll sees is unchanged; only the arrival time becomes known (for the displayed network round trip).
    pub fn stamp_arrivals(&mut self) -> std::io::Result<()> {
        if self.rx.is_some() {
            return Ok(());
        }
        let mut socks = vec![self.sock.try_clone()?];
        if let Some(s) = &self.sock6 {
            socks.push(s.try_clone()?);
        }
        self.rx = Some(RxThreads::start(socks)?);
        Ok(())
    }

    pub fn has_v6(&self) -> bool {
        self.sock6.is_some()
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, to: &Addr, data: &[u8]) -> i32 {
        let r = if to.is_v6() {
            match &self.sock6 {
                Some(s) => s.send_to(data, to.to_socket()),
                None => return -1,
            }
        } else {
            self.sock.send_to(data, to.to_socket())
        };
        match r {
            Ok(n) => n as i32,
            Err(_) => -1,
        }
    }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        if let Some(rx) = &self.rx {
            let Some((n, a, d, at)) = rx.q.lock().unwrap().pop_front() else { return 0 };
            if n < 0 {
                return n;
            }
            *from = a;
            let k = d.len().min(buf.len());
            buf[..k].copy_from_slice(&d[..k]);
            set_rx_stamp(Some(at));
            return k as i32;
        }
        set_rx_stamp(None);
        let n = recv_kind(self.sock.recv_from(buf), from);
        if n != 0 {
            return n;
        }
        match &self.sock6 {
            Some(s) => recv_kind(s.recv_from(buf), from),
            None => 0,
        }
    }
    fn local_port(&self) -> u16 {
        self.port
    }
    fn thread_sender(&self) -> Option<ThreadSender> {
        Some(ThreadSender { sock: self.sock.try_clone().ok()?, sock6: self.sock6.as_ref().and_then(|s| s.try_clone().ok()), relay: None })
    }
}

/// Any endpoint text: `1.2.3.4[:port]`, `[v6]:port`, `[v6]`, a bare IPv6 address, or `hostname[:port]` (resolved by
/// the system resolver; IPv4 preferred unless `prefer_v6`).
pub fn addr_resolve(text: &str, default_port: u16, prefer_v6: bool) -> Result<Addr, String> {
    let t = text.trim();
    if let Ok(sa) = t.parse::<SocketAddr>() {
        return Ok(Addr::from_socket(sa));
    }
    if let Ok(ip) = t.parse::<IpAddr>() {
        return Ok(Addr::from_socket(SocketAddr::new(ip, default_port)));
    }
    if let Some(inner) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        let ip: Ipv6Addr = inner.parse().map_err(|_| format!("{t}: not an IPv6 address"))?;
        return Ok(Addr::from_socket(SocketAddr::new(IpAddr::V6(ip), default_port)));
    }
    let (host, port) = match t.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().map_err(|_| format!("{t}: bad port"))?),
        None => (t, default_port),
    };
    if host.is_empty() || port == 0 {
        return Err(format!("{t}: not host:port"));
    }
    let all: Vec<SocketAddr> = (host, port).to_socket_addrs().map_err(|e| format!("cannot resolve \"{host}\": {e}"))?.collect();
    let pick = all.iter().find(|a| a.is_ipv6() == prefer_v6).or(all.first()).ok_or_else(|| format!("\"{host}\" has no address"))?;
    Ok(Addr::from_socket(*pick))
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
