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

/// Slippi Dolphin's rule for a link: one client from the select into the match (and through a Direct set). Once
/// linked (`armed` holds the links it had then), every later connection is turned away, a link that dropped too.
fn dolphin_poll(b: &mut Client, armed: &mut Option<Vec<usize>>) {
    b.poll();
    if let Some(ok) = armed {
        ok.retain(|&i| b.host.peers[i].connected());
        for i in 0..b.host.peers.len() {
            if b.host.peers[i].connected() && !ok.contains(&i) {
                b.host.disconnect(i, 0);
            }
        }
    }
}

fn arm(b: &Client) -> Option<Vec<usize>> {
    Some((0..b.host.peers.len()).filter(|&i| b.host.peers[i].connected()).collect())
}

fn linked_pair() -> (Client, Client, u16, Option<Vec<usize>>) {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let (pa, pb) = (a.local_port(), b.local_port());
    // our engine as player 1 (not the decider), the Dolphin-like peer as player 0
    let mut ca = Client::new(Box::new(a), 1, Addr::new(0x7F000001, pb), 0);
    let mut cb = Client::new(Box::new(b), 0, Addr::new(0x7F000001, pa), 1);
    let mut first: Option<Vec<usize>> = None;
    let t0 = Instant::now();
    while !(ca.status == Status::Connected && cb.status == Status::Connected) {
        ca.poll();
        dolphin_poll(&mut cb, &mut first);
        assert!(t0.elapsed() < Duration::from_secs(10), "connect timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    // both lock in on the select (the selections exchanged on this link)
    ca.set_match_selections(Selections { character_id: 20, is_character_selected: true, player_idx: 1, ..Default::default() });
    cb.set_match_selections(Selections { character_id: 2, is_character_selected: true, player_idx: 0, stage_id: 0x20, is_stage_selected: true, rng_offset: 99, ..Default::default() });
    let t0 = Instant::now();
    let mut got = false;
    while !got {
        ca.poll();
        dolphin_poll(&mut cb, &mut first);
        while let Some(n) = ca.next_note() {
            got |= matches!(n, Note::Selections(s) if s.rng_offset == 99);
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "no selections");
        std::thread::sleep(Duration::from_millis(1));
    }
    first = arm(&cb);
    (ca, cb, pa, first)
}

/// The old handoff: the select's link dropped, the match connecting again from the same port. A Dolphin-like peer
/// turns the second connection away, so the match never gets a link.
#[test]
fn reconnect_handoff_fails_against_a_dolphin_like_peer() {
    let (ca, mut cb, pa, mut first) = linked_pair();
    drop(ca);
    let mut a2 = Client::new(Box::new(UdpTransport::open(0x7F000001, pa).unwrap()), 1, Addr::new(0x7F000001, cb.host.local_port()), 0);
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(10) && !matches!(a2.status, Status::Failed | Status::Disconnected) {
        a2.poll();
        dolphin_poll(&mut cb, &mut first);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(a2.status, Status::Failed | Status::Disconnected), "the second connection was taken: {:?}", a2.status);
}

/// The one-link handoff: the select's client exported, its socket closed, the match importing it on the same port.
/// The Dolphin-like peer sees one link going on, and 600 frames of pads go both ways.
#[test]
fn one_link_handoff_passes_against_a_dolphin_like_peer() {
    let (mut ca, mut cb, pa, mut first) = linked_pair();
    // the peer goes into its match at once and sends pads while ours is handing over
    cb.start_game();
    for f in 1..=5 {
        cb.send_pad(f, [f as u8; 9], 0, 0);
    }
    for _ in 0..20 {
        ca.poll();
        dolphin_poll(&mut cb, &mut first);
        std::thread::sleep(Duration::from_millis(1));
    }
    let blob = ca.export();
    drop(ca);
    std::thread::sleep(Duration::from_millis(200));
    let mut a2 = Client::import(Box::new(UdpTransport::open(0x7F000001, pa).unwrap()), &blob).expect("import");
    assert_eq!(a2.status, Status::Connected);
    a2.start_game();
    let (mut got_a, mut got_b, mut f) = (0i32, 0i32, 0i32);
    let mut fb = 5;
    let t0 = Instant::now();
    while got_a < 600 || got_b < 600 {
        assert!(t0.elapsed() < Duration::from_secs(60), "pads stalled at {got_a}/{got_b}");
        a2.poll();
        dolphin_poll(&mut cb, &mut first);
        while let Some(n) = a2.next_note() {
            if let Note::Pad { frame, .. } = n {
                assert_eq!(frame, got_a + 1, "pads in order across the handoff");
                got_a = frame;
            }
            assert!(!matches!(n, Note::Disconnected { .. }), "link dropped");
        }
        while let Some(n) = cb.next_note() {
            if let Note::Pad { frame, .. } = n {
                assert_eq!(frame, got_b + 1);
                got_b = frame;
            }
            assert!(!matches!(n, Note::Disconnected { .. }), "peer saw a disconnect");
        }
        if f < 600 && f - got_b < 7 {
            f += 1;
            a2.send_pad(f, [f as u8; 9], 0, 0);
        } else {
            a2.resend_pads();
        }
        if fb < 600 && fb - got_a < 7 {
            fb += 1;
            cb.send_pad(fb, [fb as u8; 9], 0, 0);
        } else {
            cb.resend_pads();
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    assert_eq!(cb.status, Status::Connected);
    assert!(first.unwrap().iter().any(|&i| cb.host.peers[i].connected()), "the peer's first link carried the match");
}

/// A process boundary on our side: the link exported, its socket closed, imported on the same port (the select's
/// keeper to the match process, or a set's match process back to the next game's select).
fn hand_over(mut ca: Client, pa: u16, cb: &mut Client, first: &mut Option<Vec<usize>>) -> Client {
    for _ in 0..20 {
        ca.poll();
        dolphin_poll(cb, first);
        std::thread::sleep(Duration::from_millis(1));
    }
    let blob = ca.export();
    drop(ca);
    for _ in 0..100 {
        dolphin_poll(cb, first);
        std::thread::sleep(Duration::from_millis(2));
    }
    let a2 = Client::import(Box::new(UdpTransport::open(0x7F000001, pa).unwrap()), &blob).expect("import");
    assert_eq!(a2.status, Status::Connected);
    a2
}

/// One game of `n` frames of pads both ways (pads carry the checksum `frame ^ tag`), then both end it with the link up.
fn play_game(a: &mut Client, b: &mut Client, first: &mut Option<Vec<usize>>, n: i32, tag: u32) {
    a.start_game();
    b.start_game();
    let (mut got_a, mut got_b, mut fa, mut fb) = (0i32, 0i32, 0i32, 0i32);
    let t0 = Instant::now();
    while got_a < n || got_b < n {
        assert!(t0.elapsed() < Duration::from_secs(60), "pads stalled at {got_a}/{got_b}");
        a.poll();
        dolphin_poll(b, first);
        for (c, got, who) in [(&mut *a, &mut got_a, "ours"), (&mut *b, &mut got_b, "the peer")] {
            while let Some(m) = c.next_note() {
                if let Note::Pad { frame, .. } = m {
                    assert_eq!(frame, *got + 1, "{who}: pads in order");
                    *got = frame;
                }
                assert!(!matches!(m, Note::Disconnected { .. }), "{who}: link dropped");
            }
        }
        for (c, f, got) in [(&mut *a, &mut fa, got_b), (&mut *b, &mut fb, got_a)] {
            if *f < n && *f - got < 7 {
                *f += 1;
                c.send_pad(*f, [*f as u8; 9], *f, *f as u32 ^ tag);
            } else {
                c.resend_pads();
            }
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    // the last acks, then the game is over on both sides; the link stays for the set
    for _ in 0..50 {
        a.poll();
        dolphin_poll(b, first);
        std::thread::sleep(Duration::from_millis(1));
    }
    a.end_game();
    b.end_game();
}

/// The next game's lock-in on the same link: each side's selections reach the other (not taken for a restarted peer).
fn lock_in_again(a: &mut Client, b: &mut Client, first: &mut Option<Vec<usize>>, rng: u32) {
    a.set_match_selections(Selections { character_id: 9, is_character_selected: true, player_idx: 1, ..Default::default() });
    b.set_match_selections(Selections { character_id: 2, is_character_selected: true, player_idx: 0, stage_id: 0x1F, is_stage_selected: true, rng_offset: rng, ..Default::default() });
    let (mut got_a, mut got_b) = (false, false);
    let t0 = Instant::now();
    while !(got_a && got_b) {
        assert!(t0.elapsed() < Duration::from_secs(5), "no selections for the next game ({got_a}/{got_b})");
        a.poll();
        dolphin_poll(b, first);
        while let Some(n) = a.next_note() {
            got_a |= matches!(n, Note::Selections(s) if s.rng_offset == rng);
            assert!(!matches!(n, Note::Disconnected { .. }), "ours: link dropped at the next lock-in");
        }
        while let Some(n) = b.next_note() {
            got_b |= matches!(n, Note::Selections(s) if s.character_id == 9);
            assert!(!matches!(n, Note::Disconnected { .. }), "the peer: link dropped at the next lock-in");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// A Direct set of two games against a Dolphin-like peer, on one link: select -> match -> (game 1 over) select ->
/// match, a process boundary on our side at each step. No second connection is made, and game 2's checksums are its
/// own (the last game's are gone with the next lock-in).
#[test]
fn two_game_set_on_one_link_against_a_dolphin_like_peer() {
    let (ca, mut cb, pa, mut first) = linked_pair();
    let armed = first.clone().unwrap();
    let mut ca = hand_over(ca, pa, &mut cb, &mut first);
    play_game(&mut ca, &mut cb, &mut first, 300, 0x1000);
    // game 1 over: the match process hands the link back to the next game's select
    let mut ca = hand_over(ca, pa, &mut cb, &mut first);
    lock_in_again(&mut ca, &mut cb, &mut first, 77);
    assert!(ca.remote_checksums.is_empty(), "game 1's checksums are not game 2's");
    let mut ca = hand_over(ca, pa, &mut cb, &mut first);
    play_game(&mut ca, &mut cb, &mut first, 300, 0x2000);
    assert!(!ca.remote_checksums.is_empty() && ca.remote_checksums.iter().all(|(&f, &v)| v == f as u32 ^ 0x2000), "game 2 compares only game 2's checksums");
    assert_eq!((ca.status, cb.status), (Status::Connected, Status::Connected));
    // (the link set at the select may hold two when both sides connected at once: the duplicate is dropped in game 1)
    assert!(first.as_ref().is_some_and(|f| !f.is_empty() && f.iter().all(|i| armed.contains(i))), "the peer's first link carried the whole set");
    assert_eq!((0..cb.host.peers.len()).filter(|&i| cb.host.peers[i].connected()).count(), 1, "one link at the peer");
}

/// The old set path: game 1 over, the match closes its link and the next game's select connects again from the same
/// port. A Dolphin-like peer turns that connection away.
#[test]
fn set_reconnect_for_game_two_fails_against_a_dolphin_like_peer() {
    let (ca, mut cb, pa, mut first) = linked_pair();
    let mut ca = hand_over(ca, pa, &mut cb, &mut first);
    play_game(&mut ca, &mut cb, &mut first, 120, 0x1000);
    ca.disconnect(0);
    for _ in 0..50 {
        ca.poll();
        dolphin_poll(&mut cb, &mut first);
        std::thread::sleep(Duration::from_millis(2));
    }
    drop(ca);
    let mut a2 = Client::new(Box::new(UdpTransport::open(0x7F000001, pa).unwrap()), 1, Addr::new(0x7F000001, cb.host.local_port()), 0);
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(10) && !matches!(a2.status, Status::Failed | Status::Disconnected) {
        a2.poll();
        dolphin_poll(&mut cb, &mut first);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(a2.status, Status::Failed | Status::Disconnected), "game 2's new connection was taken: {:?}", a2.status);
}

/// Both on the select, linked, then the opponent's process dies (no DISCONNECT sent: its client just stops). ENet's
/// timeout limit and minimum (Slippi Dolphin and Hero on ENet's defaults): the survivor notices in about 5 s, not
/// at the 30 s maximum.
#[test]
fn a_dead_opponent_on_the_select_is_noticed_in_about_five_seconds() {
    let (mut ca, cb, _pa, _first) = linked_pair();
    drop(cb);
    let t0 = Instant::now();
    while ca.status == Status::Connected && t0.elapsed() < Duration::from_secs(40) {
        ca.poll();
        std::thread::sleep(Duration::from_millis(2));
    }
    let took = t0.elapsed();
    eprintln!("dead opponent noticed after {:.2}s", took.as_secs_f32());
    assert_eq!(ca.status, Status::Disconnected);
    assert!(took >= Duration::from_millis(4500) && took < Duration::from_secs(10), "took {took:?}");
    assert!(std::iter::from_fn(|| ca.next_note()).any(|n| matches!(n, Note::Disconnected { .. })));
}

/// The other side of it: a long idle select on a lossy, laggy link (5% loss, 60 ms each way) never drops.
#[test]
fn a_long_idle_select_on_a_lossy_link_stays_up() {
    let a = UdpTransport::open(0x7F000001, 0).unwrap();
    let b = UdpTransport::open(0x7F000001, 0).unwrap();
    let (pa, pb) = (a.local_port(), b.local_port());
    let lossy = |u, s| Box::new(Lossy { inner: u, loss: 5, lag: Duration::from_millis(60), rng: s, q: VecDeque::new() });
    let mut ca = Client::new(lossy(a, 7), 0, Addr::new(0x7F000001, pb), 1);
    let mut cb = Client::new(lossy(b, 9), 1, Addr::new(0x7F000001, pa), 0);
    let t0 = Instant::now();
    while !(ca.status == Status::Connected && cb.status == Status::Connected) {
        ca.poll(); cb.poll();
        assert!(t0.elapsed() < Duration::from_secs(10), "connect timed out");
        std::thread::sleep(Duration::from_millis(1));
    }
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(20) {
        ca.poll(); cb.poll();
        assert_eq!((ca.status, cb.status), (Status::Connected, Status::Connected), "dropped after {:?}", t0.elapsed());
        std::thread::sleep(Duration::from_millis(2));
    }
}
