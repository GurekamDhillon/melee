//! gw_net_tests.c, ported: two real Net instances over the simulated network in deterministic
//! virtual time (loss, duplication, jitter, a clock that wraps), plus one real-UDP loopback test.

mod common;
use common::*;
use melee_net::net::{Event, Net, State};
use melee_net::sim::{SimEndpoint, SimNet};
use melee_net::*;

#[test]
fn hash_addr() {
    assert_eq!(hash64(b"", 0), 0xcbf29ce484222325);
    assert_eq!(hash64(b"a", 0), 0xaf63dc4c8601ec8c);
    let a = addr_parse("192.168.1.20:5555", 1).unwrap();
    assert_eq!((a.ip, a.port), (0xC0A80114, 5555));
    assert_eq!(addr_parse("10.0.0.1", 7777).unwrap().port, 7777);
    assert!(addr_parse("10.0.0", 7777).is_none());
    assert!(addr_parse("300.1.1.1", 7777).is_none());
}

#[test]
fn handshake() {
    let s = SimNet::shared(1);
    let (mut h, mut g) = make_pair(&s, 0xFFFFF000, false, 9, false); // the guest's clock wraps 4 s in
    for _ in 0..400 {
        if h.n().started() && g.n().started() {
            break;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    assert!(h.n().started() && g.n().started(), "states {:?}/{:?}", h.n().state(), g.n().state());
    for (p, ev) in [(&g, Event::Accepted), (&h, Event::Starting), (&g, Event::Starting), (&h, Event::Started), (&g, Event::Started)] {
        assert!(p.has_event(ev), "missing {:?}", ev);
    }
    let rc = g.n().remote_config().unwrap();
    assert_eq!(rc.seed, 0xC0FFEE11);
    assert_eq!(rc.input_delay, 2);
    assert_eq!((rc.host_slots, rc.guest_slots), (HOST_SLOTS, GUEST_SLOTS));
    assert_eq!(rc.first_frame, FIRST);
    assert_eq!(rc.payload_bytes, 9);
    assert_eq!(rc.match_blob, blob());
    assert_eq!((g.n().local_slots(), g.n().remote_slots()), (GUEST_SLOTS, HOST_SLOTS));
    let hs = h.n().start_time_ms() as i32;
    let gs = g.n().start_time_ms().wrapping_sub(0xFFFFF000) as i32;
    assert!((hs - gs).abs() <= 6, "start times differ by {} ms", hs - gs);
    let rtt = g.n().rtt_ms();
    assert!((40..=100).contains(&rtt), "guest rtt {}", rtt);
}

fn refuse_case(which: i32, needle: &str) {
    let s = SimNet::shared(7);
    let (hc, gc) = refuse_cfgs(which);
    let mut h = Peer::new(&s, 0, 0, 9, false);
    let mut g = Peer::new(&s, 1, 0, 9, false);
    h.net = Net::host(&hc, h.cb(false, false), SimEndpoint::new(&s, 0));
    g.net = Net::join(&gc, g.cb(false, false), SimEndpoint::new(&s, 1), SimNet::addr(0));
    for _ in 0..100 {
        if g.n().state() != State::Connecting {
            break;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    assert_eq!(g.n().state(), State::Refused, "mismatch {} was not refused", which);
    let why = g.event_msg(Event::Refused);
    assert!(why.contains(needle), "refusal reason {:?} does not mention {:?}", why, needle);
    assert_eq!(h.n().state(), State::Listening, "host left LISTENING after refusing");
    // a refused guest does not use up the slot; a third is told the session is full
    let mut g2 = Peer::new(&s, 2, 0, 9, false);
    g2.st.borrow_mut().sender_id = 0;
    let g2c = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    g2.net = Net::join(&g2c, g2.cb(false, false), SimEndpoint::new(&s, 2), SimNet::addr(0));
    for _ in 0..400 {
        if h.n().started() && g2.n().started() {
            break;
        }
        s.borrow_mut().step();
        session_tick(&mut h);
        session_tick(&mut g2);
    }
    assert!(g2.n().started(), "a matching guest could not join after a refusal");
    let mut g3 = Peer::new(&s, 1, 0, 9, false);
    let g3c = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    g3.net = Net::join(&g3c, g3.cb(false, false), SimEndpoint::new(&s, 1), SimNet::addr(0));
    for _ in 0..100 {
        if g3.n().state() != State::Connecting {
            break;
        }
        s.borrow_mut().step();
        session_tick(&mut h);
        session_tick(&mut g2);
        session_tick(&mut g3);
    }
    assert_eq!(g3.n().state(), State::Refused);
    assert!(g3.event_msg(Event::Refused).contains("full"));
}

#[test]
fn refuse_exe() {
    refuse_case(0, "different game build");
}
#[test]
fn refuse_iso() {
    refuse_case(1, "disc");
}
#[test]
fn refuse_mods() {
    refuse_case(2, "different global game data; host has: plco#12ab,itco#34cd");
}
#[test]
fn refuse_mods_long_list_is_cut() {
    let s = SimNet::shared(7);
    let (hc, gc) = refuse_cfgs(9);
    let mut h = Peer::new(&s, 0, 0, 9, false);
    let mut g = Peer::new(&s, 1, 0, 9, false);
    h.net = Net::host(&hc, h.cb(false, false), SimEndpoint::new(&s, 0));
    g.net = Net::join(&gc, g.cb(false, false), SimEndpoint::new(&s, 1), SimNet::addr(0));
    for _ in 0..100 {
        pair_tick(&s, &mut h, &mut g);
    }
    let why = g.event_msg(Event::Refused);
    assert_eq!(why.len(), 230);
    assert!(why.ends_with("...") && why.starts_with("different global game data; host has: aaa"));
}
#[test]
fn refuse_settings() {
    refuse_case(3, "netplay settings");
}
#[test]
fn refuse_rules() {
    refuse_case(4, "different match rules (host 000005fb, you 00000000)");
}
#[test]
fn refuse_rules_nearmiss() {
    refuse_case(5, "different match rules (host 000005fb, you 000005fa)");
}
#[test]
fn refuse_envoy() {
    refuse_case(6, "different Envoy rules (host 45560001, you 00000000)");
}
#[test]
fn refuse_envoy_other_way() {
    refuse_case(7, "different Envoy rules (host 00000000, you 45560001)");
}
#[test]
fn refuse_envoy_unknown_word() {
    refuse_case(8, "different Envoy rules (host 45560001, you 00001234)");
}

#[test]
fn refuse_old_heap_layout() {
    for pass in 1..=2 {
        let s = SimNet::shared(37 + pass as u64);
        s.borrow_mut().old_heap_peer = pass;
        let (mut h, mut g) = make_pair(&s, 0, false, 9, false);
        for _ in 0..400 {
            if g.n().state() == State::Refused {
                break;
            }
            pair_tick(&s, &mut h, &mut g);
        }
        assert_eq!(g.n().state(), State::Refused);
        assert!(g.event_msg(Event::Refused).contains("protocol version"));
        assert!(!h.has_event(Event::Started) && !g.has_event(Event::Started));
    }
}

fn agree(set: impl Fn(&mut net::Config, &mut net::Config), check: impl Fn(&net::RemoteConfig)) {
    let s = SimNet::shared(9);
    let mut h = Peer::new(&s, 0, 0, 9, false);
    let mut g = Peer::new(&s, 1, 0, 9, false);
    let mut hc = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    let mut gc = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    set(&mut hc, &mut gc);
    h.net = Net::host(&hc, h.cb(false, false), SimEndpoint::new(&s, 0));
    g.net = Net::join(&gc, g.cb(false, false), SimEndpoint::new(&s, 1), SimNet::addr(0));
    for _ in 0..400 {
        if h.n().started() && g.n().started() {
            break;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    assert!(h.n().started() && g.n().started());
    check(&g.n().remote_config().unwrap());
}

#[test]
fn rules_agree() {
    agree(|h, _| h.rules = 0x5FB, |rc| assert_eq!(rc.rules, 0x5FB));
    agree(
        |h, g| {
            h.rules = 0x5FB;
            g.rules = 0x5FB;
            g.rules_expect = true
        },
        |rc| assert_eq!(rc.rules, 0x5FB),
    );
}

#[test]
fn envoy_agree() {
    agree(|h, _| h.envoy = ENVOY_MODE_V1, |rc| assert_eq!((rc.envoy, rc.rules), (ENVOY_MODE_V1, 0)));
    agree(
        |h, g| {
            h.envoy = ENVOY_MODE_V1;
            g.envoy = ENVOY_MODE_V1;
            g.envoy_expect = true
        },
        |rc| assert_eq!(rc.envoy, ENVOY_MODE_V1),
    );
}

fn inputs_case(loss: i32, jitter: u32, dup: i32, frames: i32, seed: u64, pb: usize, pull: bool) {
    let s = SimNet::shared(seed);
    let (mut h, mut g) = make_pair(&s, 0xFFFFF000, pull, pb, false);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = loss;
        sm.jitter_ms = jitter;
        sm.dup_pct = dup;
    }
    for p in [&h, &g] {
        let mut st = p.st.borrow_mut();
        st.limit = FIRST + frames;
        st.report_checksums = false;
    }
    for _ in 0..(frames as u32 + 4000) {
        pair_tick(&s, &mut h, &mut g);
        if h.delivered() >= frames as u32 && g.delivered() >= frames as u32 {
            break;
        }
    }
    for p in [&h, &g] {
        let st = p.st.borrow();
        assert_eq!(st.bad, 0, "{}", st.bad_msg);
    }
    assert_eq!((h.delivered(), g.delivered()), (frames as u32, frames as u32));
    assert_eq!(h.n().remote_confirmed_frame(), FIRST + frames - 1);
    let hs = h.n().stats();
    if loss >= 20 {
        assert!(hs.resent_frames > 0);
    }
    if dup > 0 {
        assert!(hs.duplicates > 0);
    }
}

#[test]
fn inputs_clean() {
    inputs_case(0, 0, 0, 10000, 11, 9, false);
}
#[test]
fn inputs_lossy() {
    inputs_case(20, 50, 10, 10000, 22, 28, false);
}
#[test]
fn inputs_hostile() {
    inputs_case(50, 150, 10, 10000, 33, 9, false);
}
#[test]
fn inputs_pull() {
    inputs_case(10, 30, 5, 3000, 44, 28, true);
}
#[test]
fn inputs_raw_pad_payload() {
    inputs_case(15, 40, 5, 3000, 55, NP_PAYLOAD, false);
}

fn sync_case(use_sync: bool) -> (i32, i32) {
    let s = SimNet::shared(5);
    let (mut h, mut g) = make_pair(&s, 0x1000, false, 9, false);
    {
        let mut sm = s.borrow_mut();
        sm.base_ms = 50;
        sm.jitter_ms = 10;
        sm.loss_pct = 5;
    }
    g.st.borrow_mut().speed = 0.85;
    for p in [&h, &g] {
        let mut st = p.st.borrow_mut();
        st.use_sync = use_sync;
        st.report_checksums = false;
    }
    let mut worst = 0;
    for t in 0..3600 {
        pair_tick(&s, &mut h, &mut g);
        if t >= 2400 {
            worst = worst.max((h.st.borrow().frame - g.st.borrow().frame).abs());
        }
    }
    let stalls = h.st.borrow().stalls_taken + g.st.borrow().stalls_taken;
    (worst, stalls)
}

#[test]
fn frame_sync() {
    let (wg, st) = sync_case(true);
    let (wg0, _) = sync_case(false);
    assert!(wg0 >= 200, "control gap only {}", wg0);
    assert!(st > 0, "time sync never asked anyone to wait");
    assert!(wg <= 6, "with sync the gap still reached {} (control {})", wg, wg0);
}

#[test]
fn interrupt_resume() {
    let s = SimNet::shared(9);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, false);
    for t in 0..1500 {
        if t == 300 {
            s.borrow_mut().blackhole = true;
        }
        if t == 390 {
            s.borrow_mut().blackhole = false;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    for p in [&h, &g] {
        assert!(p.has_event(Event::Interrupted) && p.has_event(Event::Resumed));
        assert_eq!(p.n().state(), State::Running);
        assert!(p.delivered() >= 600);
        assert_eq!(p.st.borrow().bad, 0);
    }
}

#[test]
fn peer_timeout() {
    let s = SimNet::shared(4);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, false);
    let mut dead_at = 0;
    for t in 0..900u32 {
        if t == 200 {
            s.borrow_mut().blackhole = true;
        }
        pair_tick(&s, &mut h, &mut g);
        if dead_at == 0 && h.n().state() == State::Dead {
            dead_at = t;
        }
    }
    assert_eq!((h.n().state(), g.n().state()), (State::Dead, State::Dead));
    assert!((480..=520).contains(&dead_at), "disconnect {} ticks after the cut", dead_at - 200);
    assert!(h.event_msg(Event::Disconnected).contains("timed out"));
}

fn desync_case(pull: bool) {
    let s = SimNet::shared(3);
    let (mut h, mut g) = make_pair(&s, 0, pull, 9, false);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = 20;
        sm.jitter_ms = 40;
    }
    g.st.borrow_mut().diverge_at = FIRST + 700;
    for _ in 0..1400 {
        pair_tick(&s, &mut h, &mut g);
    }
    assert_eq!((h.st.borrow().desync_calls, g.st.borrow().desync_calls), (1, 1));
    assert_eq!((h.st.borrow().desync_frame, g.st.borrow().desync_frame), (FIRST + 700, FIRST + 700));
    assert_eq!(h.n().desync_frame(), FIRST + 700);
    drop((h, g));
    let s = SimNet::shared(3);
    let (mut h, mut g) = make_pair(&s, 0, pull, 9, false);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = 30;
        sm.jitter_ms = 60;
    }
    for _ in 0..1500 {
        pair_tick(&s, &mut h, &mut g);
    }
    assert_eq!((h.st.borrow().desync_calls, g.st.borrow().desync_calls), (0, 0));
    assert_eq!(h.n().desync_frame(), NO_FRAME);
}

#[test]
fn desync() {
    desync_case(false);
}
#[test]
fn desync_pull() {
    desync_case(true);
}

#[test]
fn quit() {
    let s = SimNet::shared(6);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, false);
    for _ in 0..200 {
        pair_tick(&s, &mut h, &mut g);
    }
    g.net = None;
    for _ in 0..40 {
        s.borrow_mut().step();
        session_tick(&mut h);
    }
    assert_eq!(h.n().state(), State::Dead);
    assert!(h.event_msg(Event::Disconnected).contains("quit"));
}

#[test]
fn lobby_and_hold() {
    let s = SimNet::shared(12);
    let (mut h, mut g) = make_pair_x(&s, 0x12345, false, true, 9, false);
    {
        let mut sm = s.borrow_mut();
        sm.loss_pct = 25;
        sm.jitter_ms = 30;
    }
    for t in 0..700 {
        if t == 20 {
            for k in 0..5usize {
                let m: Vec<u8> = (0..10 + k * 7).map(|j| (k * 31 + j) as u8).collect();
                assert_eq!(h.net().lobby_send(&m), 0);
                assert_eq!(g.net().lobby_send(&m[..3 + k]), 0);
            }
        }
        if t == 150 {
            assert_eq!(h.n().state(), State::Accepted, "held: no start before release");
            g.net().release();
        }
        if t == 260 {
            assert_eq!(h.n().state(), State::Accepted, "the host still holds");
            h.net().release();
        }
        if t == 400 {
            h.net().lobby_send(&[9, 8, 7]);
        }
        pair_tick(&s, &mut h, &mut g);
    }
    assert!(h.n().started() && g.n().started());
    assert_eq!(h.st.borrow().lobby_rx, 5);
    assert_eq!(g.st.borrow().lobby_rx, 6, "the message sent while RUNNING is delivered too");
    assert_eq!(h.net().lobby_send(&[0u8; LOBBY_MAX + 1]), -1);
}

#[test]
fn api_misc() {
    let s = SimNet::shared(2);
    let (mut h, mut g) = make_pair(&s, 0, false, 9, false);
    for _ in 0..400 {
        if h.n().started() {
            break;
        }
        pair_tick(&s, &mut h, &mut g);
    }
    let f = h.st.borrow().frame;
    h.st.borrow_mut().limit = f;
    let inp = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
    assert!(h.net().submit_local(f + 5, &inp) < 0);
    assert!(h.net().submit_local(f - 1, &inp) < 0);
    assert_eq!(h.net().submit_local(f, &inp), 0);
    s.borrow_mut().blackhole = true;
    let mut fr = f + 1;
    let mut i = 0;
    while i < 400 {
        if h.net().submit_local(fr, &inp) < 0 {
            break;
        }
        fr += 1;
        i += 1;
    }
    assert!(i < 400, "send window never filled");
    let mk = |hs: u8, gs: u8, pb: u8| net::Config { host_slots: hs, guest_slots: gs, payload_bytes: pb, ..Default::default() };
    let ep = || SimEndpoint::new(&s, 0);
    assert!(Net::host(&mk(0x03, 0x02, 0), Box::new(net::NoCallbacks), ep()).is_none());
    assert!(Net::host(&mk(0x01, 0x00, 0), Box::new(net::NoCallbacks), ep()).is_none());
    assert!(Net::host(&mk(0x01, 0x02, MAX_PAYLOAD as u8 + 1), Box::new(net::NoCallbacks), ep()).is_none());
    let big = net::Config { host_slots: 1, guest_slots: 2, match_blob: vec![0; MAX_BLOB + 1], ..Default::default() };
    assert!(Net::host(&big, Box::new(net::NoCallbacks), ep()).is_none());
}

/// The real socket path: two UDP sockets on 127.0.0.1, the wall clock, real time.
#[test]
fn udp_loopback() {
    let ht = UdpTransport::open(0x7F000001, 0).expect("udp open");
    let gt = UdpTransport::open(0x7F000001, 0).expect("udp open");
    let haddr = Addr::new(0x7F000001, ht.local_port());
    assert_ne!(haddr.port, 0);
    let s = SimNet::shared(1); // only for the peers' bookkeeping
    let mut h = Peer::new(&s, 0, 0, 9, false);
    let mut g = Peer::new(&s, 1, 0, 9, false);
    let hc = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    let gc = peer_cfg(9, 0x1111, 0x2222, 0x3333);
    let real = |p: &Peer| Box::new(PeerCb { st: p.st.clone(), pull: false, lobby: false, real_clock: true });
    h.net = Net::host(&hc, real(&h), Box::new(ht));
    g.net = Net::join(&gc, real(&g), Box::new(gt), haddr);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
    let (mut fh, mut fg) = (FIRST, FIRST);
    while std::time::Instant::now() < deadline {
        h.net().poll(fh);
        g.net().poll(fg);
        for (p, f, mask, id) in [(&mut h, &mut fh, HOST_SLOTS, 0usize), (&mut g, &mut fg, GUEST_SLOTS, 1)] {
            if p.n().started() && p.n().unacked() < 30 && *f < FIRST + 200 {
                let mut inp = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
                for (slot, b) in inp.iter_mut().enumerate() {
                    if mask & (1 << slot) != 0 {
                        mk_payload(9, *f, slot, id, b);
                    }
                }
                p.net().submit_local(*f, &inp);
                *f += 1;
            }
        }
        if h.delivered() >= 200 && g.delivered() >= 200 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(h.st.borrow().bad + g.st.borrow().bad, 0);
    assert!(h.delivered() >= 200 && g.delivered() >= 200, "delivered {}/{}", h.delivered(), g.delivered());
}
