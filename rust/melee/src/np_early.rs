//! Early inputs for `melee play --netplay` (melee_net::early has the rules, the buffer and the trust rule; the
//! design is Arte's subframe rollback, as in Hero's Melee Unlocked; notes: docs/rust-port/notes/net.md).
//!
//! A sampler thread reads the local controller about every millisecond between the game's polls (a
//! [`melee_platform::pad_read::PadSampler`], the same pipeline as the game's read) and, when the pad changes in a way
//! the game acts on, sends it at once to the peer as an `MEI1` datagram stamped with the session frame it will first
//! appear in. It sends through cloned sockets ([`melee_net::transport::ThreadSender`]), so it never waits for the
//! game thread. The game thread tells it which local frames have been scheduled (`after_tick`) and when the controller
//! was read (`note_poll`), and feeds received early pads to the session (`RbSession::submit_hint`).
//!
//! Settings: on by default; `--np-no-early` (or `--np-early 0`), `MELEE_EARLY_INPUTS=0`, or `early_inputs=0` in
//! settings.cfg turn it off. Test / measurement switches: `MELEE_EARLY_RULES=presses`, `MELEE_TEST_EARLY_LEAD_MS=N`
//! (a pad script changes exactly at a poll, so nothing ever changes between polls: with this the sampler sends the
//! script's next pad N ms before the poll that takes it), `MELEE_TEST_EARLY_LIE=1` (send made-up early pads; the
//! peer's trust rule must drop them).

use melee_net::early::{self, Pad, Rules, SenderState};
use melee_net::transport::{Addr, ThreadSender};
use melee_platform::pad_read::PadSampler;
use melee_types::PADStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Whether early inputs are enabled for this run: `--np-no-early` / `--np-early 0|1`, else `MELEE_EARLY_INPUTS`,
/// else settings.cfg `early_inputs` (default on).
pub fn enabled(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--np-no-early") {
        return false;
    }
    if let Some(v) = args.iter().position(|a| a == "--np-early").and_then(|i| args.get(i + 1)) {
        return v != "0";
    }
    crate::cfg::flag("early_inputs", true)
}

fn rules() -> Rules {
    match std::env::var("MELEE_EARLY_RULES") {
        Ok(v) if v.starts_with('p') => Rules::Presses,
        _ => Rules::All,
    }
}

fn lead_ms() -> Option<f64> {
    let v: f64 = std::env::var("MELEE_TEST_EARLY_LEAD_MS").ok()?.parse().ok()?;
    (v > 0.0 && v < 16.0).then_some(v)
}

/// The pad in its wire form, as `RbSession::pad_latch` + `np_encode` would carry it (err 0).
pub fn pad_bytes(p: &PADStatus) -> Pad {
    let b = p.button.to_be_bytes();
    [b[0], b[1], p.stickX as u8, p.stickY as u8, p.substickX as u8, p.substickY as u8, p.triggerLeft, p.triggerRight, p.analogA, p.analogB, 0]
}

/// What the game thread publishes for the test lead: when the controller was last read and the pad the script will
/// give at the next read.
#[derive(Clone, Copy)]
struct Lead {
    polled: Instant,
    next: PADStatus,
}

pub struct EarlySender {
    state: Arc<Mutex<SenderState>>,
    lead: Arc<Mutex<Option<Lead>>>,
    run: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    /// the newest local frame told to the state
    last: Option<i32>,
    no_start: bool,
}

impl EarlySender {
    /// Start the sampler: `ch` the room's number (the receiver ignores other rooms' pads), `port` the local in-game
    /// port.
    pub fn start(mut sampler: PadSampler, tx: ThreadSender, peer: Addr, ch: u8, port: u8) -> EarlySender {
        let state = Arc::new(Mutex::new(SenderState::new(rules())));
        let lead: Arc<Mutex<Option<Lead>>> = Arc::new(Mutex::new(None));
        let run = Arc::new(AtomicBool::new(true));
        let no_start = std::env::var("MELEE_NP_SCRIPT_NO_START").map_or(false, |v| v == "1");
        let lie = std::env::var("MELEE_TEST_EARLY_LIE").map_or(false, |v| v == "1");
        let lead_ms = lead_ms();
        let (st2, ld2, run2) = (state.clone(), lead.clone(), run.clone());
        let thread = std::thread::Builder::new()
            .name("early-inputs".into())
            .spawn(move || {
                let t0 = Instant::now();
                let mut handled: Option<Instant> = None;
                while run2.load(Ordering::Acquire) {
                    std::thread::sleep(Duration::from_millis(1));
                    let pad = if let Some(ms) = lead_ms {
                        let Some(l) = *ld2.lock().unwrap() else { continue };
                        if handled == Some(l.polled) || l.polled.elapsed().as_secs_f64() * 1e3 < 1000.0 / 60.0 - ms {
                            continue;
                        }
                        handled = Some(l.polled);
                        l.next
                    } else {
                        let mut st = [melee_platform::pad::pad_disconnected(); melee_platform::pad::PAD_CHANMAX];
                        sampler.read(&mut st);
                        if st[0].err != 0 {
                            continue;
                        }
                        st[0]
                    };
                    let mut p = pad;
                    if no_start {
                        p.button &= !0x1000;
                    }
                    let mut bytes = pad_bytes(&p);
                    let target = st2.lock().unwrap().judge(&bytes, t0.elapsed().as_micros() as u64);
                    if let Some(t) = target {
                        if lie {
                            bytes[0] ^= 0x1F;
                        }
                        if tx.send(&peer, &early::encode(ch, t, port, &bytes)) {
                            st2.lock().unwrap().stats.packets += 1;
                        }
                    }
                }
            })
            .ok();
        crate::play_net::log_line(&format!(
            "netplay: early inputs on (rules {:?}{})",
            rules(),
            lead_ms.map_or(String::new(), |m| format!(", test: the script's next pad {m} ms before its poll"))
        ));
        EarlySender { state, lead, run, thread, last: None, no_start }
    }

    /// The game read the controller for this tick (before the session schedules its pad). `script_next`: the pad the
    /// pad script gives at the next read (the test lead).
    pub fn note_poll(&mut self, script_next: Option<PADStatus>) {
        self.state.lock().unwrap().note_poll();
        if let Some(n) = script_next {
            *self.lead.lock().unwrap() = Some(Lead { polled: Instant::now(), next: n });
        }
    }

    /// After the session's tick: the local frames newly scheduled (`pad_of(frame)`), or a skip.
    pub fn after_tick(&mut self, newest: i32, pad_of: &dyn Fn(i32) -> Option<Pad>) {
        let from = self.last.map_or(newest - 16, |l| l + 1);
        let mut got = None;
        for f in from..=newest {
            if let Some(p) = pad_of(f) {
                got = Some((f, p));
            }
        }
        let mut st = self.state.lock().unwrap();
        match got {
            Some((f, mut p)) => {
                if self.no_start {
                    p[0] &= !0x10;
                }
                st.note_sampled(f, &p);
                self.last = Some(f);
            }
            None => st.note_skipped(),
        }
    }

    pub fn stats(&self) -> early::SendStats {
        self.state.lock().unwrap().stats
    }

    pub fn stop(&mut self) {
        self.run.store(false, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for EarlySender {
    fn drop(&mut self) {
        self.stop();
    }
}
