//! One mu_net session as the game loop sees it: the [`Connector`] (connection, handshake, protected messages), the
//! current game's [`Input`] exchange and [`Transcript`], the pre-match PING / PONG, RESULT, and the peer's INFO.
//! The game side (`melee`'s mu_np / play path) commits its pads, takes the peer's, and reports the end.

use super::conn::{Connector, State};
use super::crypto::Bytes32;
use super::desc::{MAX_GAMES, REVISION};
use super::input::{Input, Transcript};
use super::wire::{Msg, Pad, BYE_LEFT, CH_INPUT};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const PING_EVERY: Duration = Duration::from_millis(250);
/// A hold this long without a new peer frame ends the match as disconnected.
pub const INPUT_TIMEOUT: Duration = Duration::from_secs(7);

pub struct Link {
    pub conn: Connector,
    t0: Instant,
    pub input: Input,
    pub transcript: Option<Transcript>,
    pub game: u32,
    local_slot: usize,
    /// our most games in this session (0 = no limit), sent in INFO
    pub games: u16,
    info_sent: bool,
    /// the peer's INFO: (revision, games)
    pub peer_info: Option<(u8, u16)>,
    next_ping: Instant,
    /// a frame was committed in this game (PING / PONG stop then)
    started: bool,
    /// the peer's RESULT for this game: (frames, digest)
    pub peer_result: Option<(i32, Bytes32)>,
    /// our RESULT was sent
    pub reported: bool,
    /// the peer's (checksum frame, checksum) pairs seen this game
    pub remote_checksums: BTreeMap<i32, u32>,
}

impl Link {
    pub fn new(conn: Connector, local_slot: usize, games: u16, now: Instant) -> Link {
        Link {
            conn,
            t0: now,
            input: Input::new(1, 0),
            transcript: None,
            game: 1,
            local_slot,
            games,
            info_sent: false,
            peer_info: None,
            next_ping: now,
            started: false,
            peer_result: None,
            reported: false,
            remote_checksums: BTreeMap::new(),
        }
    }

    /// Microseconds since the link started (the stamps' clock).
    pub fn now_us(&self) -> u64 {
        self.t0.elapsed().as_micros() as u64
    }

    pub fn ready(&self) -> bool {
        self.conn.ready()
    }

    /// The session is over or never started: the reason.
    pub fn down(&self) -> Option<String> {
        match self.conn.state() {
            State::Failed(w) | State::Closed(w) => Some(w.clone()),
            _ => None,
        }
    }

    /// The games this session plays: the smaller of the two limits (0 = 127); 1 with a peer below revision 1.
    pub fn session_games(&self) -> u32 {
        let lim = |g: u16| if g == 0 { MAX_GAMES } else { g as u32 };
        match self.peer_info {
            Some((rev, g)) if rev >= 1 => lim(self.games).min(lim(g)),
            _ => 1,
        }
    }

    /// Network work: reads every message, answers input messages at once, sends INFO when ready and the
    /// pre-match PINGs.
    pub fn poll(&mut self) {
        let now = Instant::now();
        self.conn.poll(now);
        if self.conn.ready() && !self.info_sent {
            self.info_sent = true;
            self.conn.send(&Msg::Info { revision: REVISION, games: self.games });
            if self.transcript.is_none() {
                self.transcript = self.conn.digest().map(|d| Transcript::new(&d));
            }
        }
        while let Some(m) = self.conn.recv() {
            let us = self.now_us();
            match &m {
                Msg::Info { revision, games } => {
                    if self.peer_info.is_none() {
                        self.peer_info = Some((*revision, *games));
                    }
                }
                Msg::Result { frames, digest } => self.peer_result = Some((*frames, *digest)),
                Msg::Bye { .. } | Msg::Next { .. } | Msg::Verify { .. } | Msg::Unknown { .. } => {}
                m if m.channel() == CH_INPUT => {
                    if let Some(r) = self.input.on_msg(m, us) {
                        self.conn.send(&r);
                    }
                    if let Msg::Commit(_) = m {
                        let (f, ck) = self.input.ck_remote;
                        if f > 0 {
                            self.remote_checksums.insert(f, ck);
                        }
                    }
                }
                _ => {}
            }
        }
        if self.conn.ready() && !self.started && now >= self.next_ping {
            self.next_ping = now + PING_EVERY;
            let p = self.input.build_ping(self.now_us());
            self.conn.send(&p);
        }
        self.conn.flush();
    }

    /// Commits our pad of game frame `f` (in order from 1).
    pub fn commit(&mut self, f: i32, pad: Pad) -> bool {
        self.started = true;
        self.input.commit(f, pad)
    }

    /// This tick's COMMIT (one per simulation tick, held ticks too), sent now.
    pub fn send_commit(&mut self) {
        let us = self.now_us();
        if let Some(c) = self.input.build_commit(us) {
            self.conn.send(&c);
        }
        self.conn.flush();
    }

    /// Hashes every frame both pads are known for into the transcript.
    pub fn advance_transcript(&mut self) {
        let Some(t) = self.transcript.as_mut() else { return };
        loop {
            let f = t.frames + 1;
            let (Some(l), Some(r)) = (self.input.local_pad(f), self.input.remote_pad(f)) else { break };
            let (a, b) = if self.local_slot == 0 { (l, r) } else { (r, l) };
            t.push(&a, &b);
        }
    }

    /// No new peer frame for [`INPUT_TIMEOUT`] while frames are being played.
    pub fn input_timed_out(&self) -> bool {
        self.started && self.now_us().saturating_sub(self.input.last_progress_us) > INPUT_TIMEOUT.as_micros() as u64
    }

    /// The game ended: our RESULT (confirmed frames, transcript digest), sent once.
    pub fn report_end(&mut self) {
        if self.reported {
            return;
        }
        self.advance_transcript();
        if let Some(t) = &self.transcript {
            self.reported = true;
            let m = Msg::Result { frames: t.frames, digest: t.digest() };
            self.conn.send(&m);
            self.conn.flush();
        }
    }

    /// Whether the peer's RESULT agrees with our transcript: Some(true / false) once comparable.
    pub fn peer_agreement(&self) -> Option<bool> {
        let (frames, digest) = self.peer_result?;
        let ours = self.transcript.as_ref()?.digest_at(frames)?;
        Some(ours == digest)
    }

    /// Leaves the session (BYE 0).
    pub fn close(&mut self) {
        self.conn.close(BYE_LEFT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mu_net::crypto::Identity;
    use crate::mu_net::desc::{Descriptor, Slot};
    use crate::transport::{Addr, Transport, UdpTransport};

    const LO: u32 = 0x7f00_0001;

    /// Drops every `n`th datagram sent (after the first 40, so the handshake gets through quickly).
    struct Lossy {
        t: UdpTransport,
        n: u32,
        k: u32,
    }
    impl Transport for Lossy {
        fn send(&mut self, to: &Addr, d: &[u8]) -> i32 {
            self.k += 1;
            if self.k > 40 && self.k % self.n == 0 {
                return d.len() as i32;
            }
            self.t.send(to, d)
        }
        fn recv(&mut self, from: &mut Addr, buf: &mut [u8]) -> i32 {
            self.t.recv(from, buf)
        }
        fn local_port(&self) -> u16 {
            self.t.local_port()
        }
    }

    fn desc() -> Descriptor {
        let mut d = Descriptor { build_id: "rust link test".into(), ..Default::default() };
        d.players[0] = Slot { present: true, name: "A".into(), ..Default::default() };
        d.players[1] = Slot { present: true, name: "B".into(), ..Default::default() };
        d
    }

    /// A fake 2-player game loop on both sides: each tick commits its pad, takes the peer's, and may run frame f only
    /// with the peer's frame f - 7 held (the stall rule); 1200 frames with every 7th datagram lost.
    #[test]
    fn two_links_play_a_lossy_game_and_agree() {
        let ta = UdpTransport::open(LO, 0).unwrap();
        let tb = UdpTransport::open(LO, 0).unwrap();
        let (pa, pb) = (ta.local_port(), tb.local_port());
        let (ia, ib) = (Identity::from_seed(&[1; 32]), Identity::from_seed(&[2; 32]));
        let (ka, kb) = (ia.public_key(), ib.public_key());
        let now = Instant::now();
        let lim = Duration::from_secs(5);
        let ca = Connector::start(Box::new(Lossy { t: ta, n: 7, k: 0 }), ia, desc(), 0, &[Addr::new(LO, pb)], Some(kb), lim, now).unwrap();
        let cb = Connector::start(Box::new(Lossy { t: tb, n: 7, k: 0 }), ib, desc(), 1, &[Addr::new(LO, pa)], Some(ka), lim, now).unwrap();
        let mut links = [Link::new(ca, 0, 1, now), Link::new(cb, 1, 1, now)];
        let end = Instant::now() + Duration::from_secs(8);
        while !(links[0].ready() && links[1].ready()) && Instant::now() < end {
            links.iter_mut().for_each(Link::poll);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(links[0].ready() && links[1].ready());
        let pad = |slot: usize, f: i32| -> Pad { [(f * 3 + slot as i32) as u8, slot as u8, 0, 0, 1, 2, 3, 4] };
        let n = 1200;
        let mut frame = [0i32; 2];
        while (frame[0] < n || frame[1] < n) && Instant::now() < end + Duration::from_secs(20) {
            for (s, l) in links.iter_mut().enumerate() {
                l.poll();
                let f = frame[s] + 1;
                if f <= n && l.input.remote_latest() >= f - 7 {
                    // the pad for frame f + delay(2), frames 1..=2 zero
                    if f == 1 {
                        l.commit(1, [0; 8]);
                        l.commit(2, [0; 8]);
                    }
                    l.commit(f + 2, pad(s, f + 2));
                    frame[s] = f;
                }
                l.input.set_floor(frame[s]);
                l.send_commit();
                l.advance_transcript();
            }
            std::thread::sleep(Duration::from_micros(500));
        }
        assert_eq!(frame, [n, n]);
        // the end: both report, then wait for the other's RESULT
        for l in links.iter_mut() {
            while l.transcript.as_ref().unwrap().frames < n {
                l.poll();
                l.send_commit();
                l.advance_transcript();
                std::thread::sleep(Duration::from_millis(1));
            }
            l.report_end();
        }
        let t = Instant::now();
        while links.iter().any(|l| l.peer_agreement().is_none()) && t.elapsed() < Duration::from_secs(3) {
            // the peer may have counted further (its RESULT names more frames): keep exchanging until we hold them
            for l in links.iter_mut() {
                l.poll();
                l.send_commit();
                l.advance_transcript();
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(links[0].peer_agreement(), Some(true));
        assert_eq!(links[1].peer_agreement(), Some(true));
        assert_eq!(links[0].transcript.as_ref().unwrap().digest(), links[1].transcript.as_ref().unwrap().digest());
        assert_eq!(links[0].session_games(), 1);
        assert_eq!(links[0].peer_info, Some((REVISION, 1)));
        assert!(links[0].input.ping_us().is_some());
        let received: u64 = links.iter().map(|l| l.input.stats.commits_received).sum();
        assert!(received > 0);
    }
}
