//! ENet + Slippi client over real loopback UDP, with inbound loss and latency on both sides.
use melee_net::slippi::{Client, Note, Selections, Status};
use melee_net::transport::{Addr, Transport, UdpTransport};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct Lossy { inner: UdpTransport, loss: u32, lag: Duration, rng: u32, q: VecDeque<(Instant, Addr, Vec<u8>)> }
impl Transport for Lossy {
    fn send(&mut self, to: &Addr, d: &[u8]) -> i32 { self.inner.send(to, d) }
    fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
        let mut t = [0u8; 4096];
        loop {
            let mut a = Addr::default();
            let n = self.inner.recv(&mut a, &mut t);
            if n <= 0 { break; }
            self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
            if (self.rng >> 16) % 100 < self.loss { continue; }
            self.q.push_back((Instant::now() + self.lag, a, t[..n as usize].to_vec()));
        }
        if self.q.front().map_or(false, |f| f.0 <= Instant::now()) {
            let (_, a, d) = self.q.pop_front().unwrap();
            *from = a; buf[..d.len()].copy_from_slice(&d); return d.len() as i32;
        }
        0
    }
    fn local_port(&self) -> u16 { self.inner.local_port() }
}

fn run(loss: u32, lag_ms: u64, frames: i32) {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let (pa, pb) = (a.local_port(), b.local_port());
    let lossy = |u, s| Box::new(Lossy { inner: u, loss, lag: Duration::from_millis(lag_ms), rng: s, q: VecDeque::new() });
    let mut ca = Client::new(lossy(a, 1), 0, Addr::new(0x7F000001, pb), 1);
    let mut cb = Client::new(lossy(b, 2), 1, Addr::new(0x7F000001, pa), 0);
    let t0 = Instant::now();
    while !(ca.status == Status::Connected && cb.status == Status::Connected) {
        ca.poll(); cb.poll();
        assert!(t0.elapsed() < Duration::from_secs(10), "connect timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    let sel = Selections { character_id: 2, stage_id: 0x20, is_stage_selected: true, rng_offset: 77, ..Default::default() };
    ca.set_match_selections(sel); cb.set_match_selections(sel);
    let (mut got_a, mut got_b) = (0i32, 0i32);
    let (mut sa, mut sb) = (false, false);
    let mut f = 0;
    let t0 = Instant::now();
    while got_a < frames || got_b < frames {
        assert!(t0.elapsed() < Duration::from_secs(60), "pads stalled at {got_a}/{got_b}");
        ca.poll(); cb.poll();
        for (c, got, sel_seen) in [(&mut ca, &mut got_a, &mut sa), (&mut cb, &mut got_b, &mut sb)] {
            while let Some(n) = c.next_note() {
                match n {
                    Note::Selections(s) => { assert_eq!(s.rng_offset, 77); *sel_seen = true; }
                    Note::Pad { frame, pad, .. } => { assert_eq!(frame, *got + 1, "pads in order"); assert_eq!(pad[0], frame as u8); *got = frame; }
                    _ => {}
                }
            }
        }
        if sa && sb && f < frames && f - got_a.min(got_b) < 7 {
            f += 1;
            let p = [f as u8; 9];
            ca.send_pad(f, p, f - 3, 0x1000 + f as u32);
            cb.send_pad(f, p, f - 3, 0x1000 + f as u32);
        } else {
            // a stalled tick: send the unacked pads again, as the game loops do
            ca.resend_pads();
            cb.resend_pads();
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    assert_eq!(ca.remote_checksums.get(&(frames - 3)), Some(&(0x1000 + frames as u32)));
    eprintln!("loss {loss}% lag {lag_ms}ms: {frames} frames, ping {} us, max queue {}", ca.ping_us(), ca.stats.max_queue);
    ca.disconnect(0);
    let t1 = Instant::now();
    while cb.status != Status::Disconnected && t1.elapsed() < Duration::from_secs(5) { cb.poll(); ca.poll(); std::thread::sleep(Duration::from_millis(1)); }
    assert_eq!(cb.status, Status::Disconnected);
}

#[test]
fn slippi_clean() { run(0, 0, 600); }
#[test]
fn slippi_loss_lag() { run(10, 30, 600); }

/// The live failure against Slippi Dolphin: matchmaking names the opponent's public endpoint, but its NAT maps its
/// port anew for us, so its CONNECT arrives from another port than the one we dial. Slippi Dolphin and Hero accept
/// a connecting peer by host; this client used to demand the exact address and dropped it (connect timed out).
#[test]
fn accepts_the_opponent_from_a_remapped_port() {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let dead = std::net::UdpSocket::bind("127.0.0.1:0").unwrap(); // the advertised endpoint: nothing answers there
    let advertised = Addr::new(0x7F000001, dead.local_addr().unwrap().port());
    drop(dead);
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let pa = a.local_port();
    let mut ca = Client::new(Box::new(a), 0, advertised, 1);
    // "Dolphin": its own client dials our real port from its (different) real port
    let mut cb = Client::new(Box::new(b), 1, Addr::new(0x7F000001, pa), 0);
    let t0 = Instant::now();
    while !(ca.status == Status::Connected && cb.status == Status::Connected) {
        ca.poll();
        cb.poll();
        assert!(ca.status != Status::Failed && cb.status != Status::Failed);
        assert!(t0.elapsed() < Duration::from_secs(5), "the opponent from a remapped port was refused (status {:?}/{:?})", ca.status, cb.status);
        std::thread::sleep(Duration::from_millis(1));
    }
    let s = Selections { character_id: 2, rng_offset: 5, ..Default::default() };
    ca.set_match_selections(s);
    let t0 = Instant::now();
    loop {
        ca.poll();
        cb.poll();
        if cb.remote_selections().is_some() {
            break;
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "no selections over the accepted link");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Hero's public fallback: the LAN address named first does not answer; 2 s in, the public one is dialled too.
#[test]
fn public_fallback_after_two_seconds() {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let pb = b.local_port();
    let mut ca = Client::new(Box::new(a), 0, Addr::new(0x7F000001, 9), 1); // a LAN address nothing answers on
    ca.set_fallback(Some(Addr::new(0x7F000001, pb)));
    // the opponent never dials (a silent listener): only our fallback can make the link
    let mut hb = melee_net::enet::Host::new(Box::new(b), 4, 3);
    let t0 = Instant::now();
    while ca.status != Status::Connected {
        ca.poll();
        while hb.service().is_some() {}
        assert!(t0.elapsed() < Duration::from_secs(6), "the public fallback never connected");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(t0.elapsed() >= Duration::from_millis(1900), "the fallback must wait for the LAN attempt");
}
