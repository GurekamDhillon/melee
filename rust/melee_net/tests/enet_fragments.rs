//! Reliable fragments (SEND_FRAGMENT) over real UDP on 127.0.0.1: packets larger than one datagram (the Slippi
//! spectator feed's bigger game events) arrive whole and in order, between small ones.

use melee_net::enet::{Event, Host};
use melee_net::*;
use std::time::{Duration, Instant};

fn blob(n: usize, k: u8) -> Vec<u8> {
    (0..n).map(|i| (i as u32).wrapping_mul(2654435761).rotate_left(k as u32) as u8 ^ k).collect()
}

#[test]
fn big_reliable_packets_arrive_whole_and_in_order() {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let pb = b.local_port();
    let mut ha = Host::new(Box::new(a), 2, 2);
    let mut hb = Host::new(Box::new(b), 2, 2);
    let peer = ha.connect(Addr::new(0x7F000001, pb), 2, 0).unwrap();
    let t0 = Instant::now();
    let mut up = (false, false);
    while !(up.0 && up.1) {
        while let Some(e) = ha.service() {
            up.0 |= matches!(e, Event::Connect { .. });
        }
        while let Some(e) = hb.service() {
            up.1 |= matches!(e, Event::Connect { .. });
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "no connect");
        std::thread::sleep(Duration::from_millis(1));
    }
    let sizes = [100usize, 5000, 1, 1380, 1381, 70000, 3000, 12];
    let sent: Vec<Vec<u8>> = sizes.iter().enumerate().map(|(k, &n)| blob(n, k as u8)).collect();
    for s in &sent {
        assert!(ha.send(peer, 0, s, true), "send {} bytes", s.len());
    }
    let mut got = Vec::new();
    while got.len() < sent.len() {
        ha.service();
        ha.flush();
        while let Some(e) = hb.service() {
            if let Event::Receive { data, channel: 0, .. } = e {
                got.push(data);
            }
        }
        hb.flush();
        assert!(t0.elapsed() < Duration::from_secs(10), "received {} of {}", got.len(), sent.len());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(got, sent);
    // unreliable over the MTU is still refused
    assert!(!ha.send(peer, 0, &blob(5000, 9), false));
}
