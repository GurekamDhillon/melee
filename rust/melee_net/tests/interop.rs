//! Rust <-> C over real UDP on 127.0.0.1: the Rust port against the shipping gw_net.c
//! (oracle/net_peer.c, built by oracle/build.sh into rust/target/net-oracle/). Both directions:
//! Rust host + C guest, C host + Rust guest; 300 frames of 11-byte payloads each way, checksums
//! exchanged (a mismatch fails the C side), match config (rules, Envoy word, blob) carried.
//! Skipped (with a message) when the C peer has not been built.

use melee_net::net::{Callbacks, Config, Net};
use melee_net::*;
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::rc::Rc;

const FIRST: i32 = -123;
const FRAMES: u32 = 300;
const HOST_SLOTS: u8 = 0x13;
const GUEST_SLOTS: u8 = 0x44;
const PB: usize = 11;

fn mk_payload(frame: i32, slot: usize, who: usize, out: &mut [u8]) {
    let mut h = (frame as u32).wrapping_mul(2654435761) ^ (slot as u32).wrapping_mul(40503) ^ (who as u32 + 1).wrapping_mul(0x9E3779B1);
    for (i, o) in out.iter_mut().enumerate().take(PB) {
        h ^= h >> 15;
        h = h.wrapping_mul(2246822519);
        h ^= h >> 13;
        *o = (h >> (i & 7)) as u8;
    }
}

#[derive(Default)]
struct St {
    exp: i32,
    idx: usize,
    delivered: u32,
    bad: bool,
    desync: bool,
    rslots: Vec<usize>,
    sender: usize,
}
struct Cb(Rc<RefCell<St>>);
impl Callbacks for Cb {
    fn remote_input(&mut self, frame: i32, slot: usize, payload: &[u8]) {
        let mut s = self.0.borrow_mut();
        if s.bad {
            return;
        }
        let mut want = [0u8; PB];
        mk_payload(frame, slot, s.sender, &mut want);
        if frame != s.exp || slot != s.rslots[s.idx] || payload != want {
            s.bad = true;
            return;
        }
        s.idx += 1;
        if s.idx == s.rslots.len() {
            s.idx = 0;
            s.exp += 1;
            s.delivered += 1;
        }
    }
    fn desync(&mut self, _f: i32, _l: u32, _r: u32) {
        self.0.borrow_mut().desync = true;
    }
}

fn peer_exe() -> Option<std::path::PathBuf> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/net-oracle/net_peer.exe");
    p.exists().then_some(p)
}

fn cfg() -> Config {
    Config {
        exe_hash: 0x1111,
        iso_hash: 0x2222,
        mods_hash: 0x3333,
        first_frame: FIRST,
        payload_bytes: PB as u8,
        seed: 0xC0FFEE11,
        input_delay: 2,
        host_slots: HOST_SLOTS,
        guest_slots: GUEST_SLOTS,
        rules: 0x5FB,
        envoy: ENVOY_MODE_V1,
        match_blob: (0..64).map(|i| (i * 5 + 1) as u8).collect(),
        ..Config::default()
    }
}

fn drive(mut net: Net, st: Rc<RefCell<St>>, me: usize) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut frame = FIRST;
    while std::time::Instant::now() < deadline {
        net.poll(frame);
        if matches!(net.state(), net::State::Dead | net::State::Refused) {
            break;
        }
        if net.started() && net.unacked() < 30 && frame < FIRST + FRAMES as i32 {
            let mut inp = [[0u8; MAX_PAYLOAD]; MAX_SLOTS];
            for (s, b) in inp.iter_mut().enumerate() {
                if net.local_slots() & (1 << s) != 0 {
                    mk_payload(frame, s, me, b);
                }
            }
            if net.submit_local(frame, &inp) == 0 {
                net.report_checksum(frame, (frame as u32).wrapping_mul(2654435761) ^ 0x5BD1E995);
                frame += 1;
            }
        }
        if st.borrow().delivered >= FRAMES && frame >= FIRST + FRAMES as i32 && net.unacked() == 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let rc = net.remote_config().expect("config");
    assert_eq!((rc.rules, rc.envoy, rc.match_blob.len()), (0x5FB, ENVOY_MODE_V1, 64));
    // keep answering briefly so the C side's last frames are acknowledged
    let t = std::time::Instant::now();
    while t.elapsed() < std::time::Duration::from_millis(500) {
        net.poll(frame);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let s = st.borrow();
    assert!(!s.bad, "a payload from the C peer arrived out of order or corrupted");
    assert!(!s.desync, "checksum mismatch reported");
    assert!(s.delivered >= FRAMES, "delivered {} of {} frames from the C peer (state {:?}, {})", s.delivered, FRAMES, net.state(), net.last_reason());
}

fn state(host: bool) -> Rc<RefCell<St>> {
    let rmask = if host { GUEST_SLOTS } else { HOST_SLOTS };
    Rc::new(RefCell::new(St {
        exp: FIRST,
        rslots: (0..8).filter(|i| rmask & (1 << i) != 0).collect(),
        sender: if host { 1 } else { 0 },
        ..St::default()
    }))
}

fn finish(child: std::process::Child) {
    let out = child.wait_with_output().expect("C peer");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "C peer failed: {}", text);
    assert!(text.contains("OK "), "C peer: {}", text);
}

#[test]
fn rust_host_c_guest() {
    let Some(exe) = peer_exe() else {
        eprintln!("SKIPPED: build the C peer with rust/melee_net/oracle/build.sh");
        return;
    };
    let tp = UdpTransport::open(0x7F000001, 0).unwrap();
    let port = tp.local_port();
    let st = state(true);
    let net = Net::host(&cfg(), Box::new(Cb(st.clone())), Box::new(tp)).unwrap();
    let child = Command::new(exe).arg("guest").arg(port.to_string()).stdout(Stdio::piped()).spawn().unwrap();
    drive(net, st, 0);
    finish(child);
}

#[test]
fn c_host_rust_guest() {
    let Some(exe) = peer_exe() else {
        eprintln!("SKIPPED: build the C peer with rust/melee_net/oracle/build.sh");
        return;
    };
    let mut child = Command::new(exe).arg("host").stdout(Stdio::piped()).spawn().unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.as_mut().unwrap()).read_line(&mut line).unwrap();
    let port: u16 = line.trim().strip_prefix("PORT ").expect("PORT line").parse().unwrap();
    let tp = UdpTransport::open(0x7F000001, 0).unwrap();
    let st = state(false);
    let mut gc = cfg();
    gc.rules_expect = true;
    gc.envoy_expect = true;
    gc.match_blob.clear();
    let net = Net::join(&gc, Box::new(Cb(st.clone())), Box::new(tp), Addr::new(0x7F000001, port)).unwrap();
    drive(net, st, 1);
    finish(child);
}
