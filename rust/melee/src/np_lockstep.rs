//! Lockstep pads for the scenes outside the match (`melee play --netplay ... --np-css`): the character select, the stage
//! select, the results. Both peers run the same boot and the same menu frames; a frame runs only when both ports' pads
//! for it are known (no prediction, no snapshots: the menus are not under the rollback session, whose snapshot base must
//! not straddle a scene's heap setup). The match itself stays under `RbSession` (play_net.rs).
//!
//! Wire (channel 1 of the run's one socket, np_mux.rs): `MLS1`, u32 ack (the next peer frame this side needs), u32
//! first frame, u8 count, then `count` pads of 12 bytes (button u16 LE, stick x/y, c-stick x/y, L, R, analog A, analog
//! B, err, 0) for frames first.. . Every packet repeats all pads the peer has not acknowledged, so a lost datagram costs
//! nothing but its successor's arrival.
//!
//! Each render tick schedules one local pad (`push_local`) at local frame + delay, the pad script's frame k thus always
//! lands on menu frame k + delay, whatever the stalls; `ready` pops the next frame once the peer's pad for it arrived.

use melee_net::transport::{Addr, Transport};
use melee_types::PADStatus;
use std::time::Instant;

const MAGIC: &[u8; 4] = b"MLS1";
const PAD: usize = 12;
// 13 + 90 * 12 = 1093 bytes: under the 1280-byte IPv6 minimum MTU with the relay and IP/UDP headers
const MAX_PER_PACKET: usize = 90;

pub struct Lockstep {
    tp: Box<dyn Transport>,
    peer: Option<Addr>,
    pub host: bool,
    delay: u32,
    /// local pads by frame (index = frame)
    local: Vec<PADStatus>,
    /// peer pads by frame
    remote: Vec<Option<PADStatus>>,
    /// next frame to run
    pub frame: u32,
    /// the first local frame the peer has not acknowledged
    peer_ack: u32,
    last_heard: Instant,
    pub stalls: u64,
}

fn enc(p: &PADStatus, o: &mut Vec<u8>) {
    o.extend_from_slice(&p.button.to_le_bytes());
    o.extend_from_slice(&[p.stickX as u8, p.stickY as u8, p.substickX as u8, p.substickY as u8, p.triggerLeft, p.triggerRight, p.analogA, p.analogB, p.err as u8, 0]);
}
fn dec(b: &[u8]) -> PADStatus {
    let mut p = PADStatus::zeroed();
    p.button = u16::from_le_bytes([b[0], b[1]]);
    p.stickX = b[2] as i8;
    p.stickY = b[3] as i8;
    p.substickX = b[4] as i8;
    p.substickY = b[5] as i8;
    p.triggerLeft = b[6];
    p.triggerRight = b[7];
    p.analogA = b[8];
    p.analogB = b[9];
    p.err = b[10] as i8;
    p
}

impl Lockstep {
    /// `peer` None (the host): learnt from the first packet.
    pub fn new(tp: Box<dyn Transport>, peer: Option<Addr>, host: bool, delay: u32) -> Lockstep {
        let mut local = Vec::new();
        let mut remote = Vec::new();
        // the first `delay` frames have no input from either side: neutral
        for _ in 0..delay {
            local.push(PADStatus::zeroed());
            remote.push(Some(PADStatus::zeroed()));
        }
        Lockstep { tp, peer, host, delay, local, remote, frame: 0, peer_ack: 0, last_heard: Instant::now(), stalls: 0 }
    }

    /// This tick's local pad: it runs at the next unscheduled frame (frame + delay in a steady state).
    pub fn push_local(&mut self, p: &PADStatus) {
        // a scheduled frame may not run before the peer had the chance to see it: cap the queue at 10 s
        if self.local.len() < (self.frame + self.delay) as usize + 600 {
            self.local.push(*p);
        }
    }

    fn need(&self) -> u32 {
        self.remote.iter().position(|r| r.is_none()).unwrap_or(self.remote.len()) as u32
    }

    /// Receive, then send this side's unacknowledged pads.
    pub fn poll(&mut self) {
        let mut buf = [0u8; 2048];
        loop {
            let mut from = Addr::default();
            let n = self.tp.recv(&mut from, &mut buf);
            if n <= 0 {
                break;
            }
            let d = &buf[..n as usize];
            if d.len() < 13 || &d[..4] != MAGIC {
                continue;
            }
            if self.peer.is_none() {
                self.peer = Some(from);
            }
            self.last_heard = Instant::now();
            let ack = u32::from_le_bytes([d[4], d[5], d[6], d[7]]);
            let first = u32::from_le_bytes([d[8], d[9], d[10], d[11]]);
            let cnt = d[12] as usize;
            self.peer_ack = self.peer_ack.max(ack);
            for k in 0..cnt {
                let o = 13 + k * PAD;
                if o + PAD > d.len() {
                    break;
                }
                let f = (first as usize) + k;
                if self.remote.len() <= f {
                    self.remote.resize(f + 1, None);
                }
                if self.remote[f].is_none() {
                    self.remote[f] = Some(dec(&d[o..o + PAD]));
                }
            }
        }
        let Some(peer) = self.peer else { return };
        let first = self.peer_ack.min(self.local.len() as u32);
        let cnt = (self.local.len() as u32 - first).min(MAX_PER_PACKET as u32);
        let mut o = MAGIC.to_vec();
        o.extend_from_slice(&self.need().to_le_bytes());
        o.extend_from_slice(&first.to_le_bytes());
        o.push(cnt as u8);
        for f in first..first + cnt {
            enc(&self.local[f as usize], &mut o);
        }
        self.tp.send(&peer, &o);
    }

    /// The next frame's pads as (local, remote), or None while the peer's has not arrived.
    pub fn ready(&mut self) -> Option<(PADStatus, PADStatus)> {
        let f = self.frame as usize;
        match (self.local.get(f), self.remote.get(f).copied().flatten()) {
            (Some(l), Some(r)) => {
                self.frame += 1;
                Some((*l, r))
            }
            _ => {
                self.stalls += 1;
                None
            }
        }
    }

    /// Back from a match (the lockstep was idle while the session ran): the silence count starts now.
    pub fn resume(&mut self) {
        self.last_heard = Instant::now();
    }

    /// Seconds since the peer was last heard from.
    pub fn silent_secs(&self) -> u64 {
        self.last_heard.elapsed().as_secs()
    }
}
