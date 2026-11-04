//! The 1P coverage scenarios (docs/rust-port/golden/coverage/scenarios.tsv kind `1p`): Classic / Adventure / All-Star
//! booted through the boot scene into the mode at a step (`MELEE_SCENE=mode=classic;p1=ck:N;step=S;difficulty=D`, as the
//! C build launches them), driven by the `randabs_<tag>.lua` pad (tools/rust-port/golden/coverage/gen_pads.py `abs_rand`),
//! hashed like the C build's `MELEE_NEUTRAL_HASHLOG`: one row per live match frame.
//!
//! The C side these mirror (pc/platform/gw_script.c, the Lua harness, not game code):
//! - `gw_Script_FramePost`, once per logic frame: the pad holds count one frame (`gw_Script_PadFrameConsumed`), `gs.frame`
//!   goes up, a match goes live on the first frame a fighter is present (`gs.match_frame` 0) and counts up after it; a live
//!   frame writes its hash row (`frame` = `gs.match_frame`); then the script's task resumes (`gs_run_tasks`).
//! - a scene change ends the match (`gs.match_active = 0`).
//! - `gd.input(1, {..}, 1)` claims port 1 for one sample (`gw_script_pad_override`): `PADRead` then returns the claimed
//!   pad (connected, the given buttons and stick, triggers 0) while the hold lasts, and a neutral connected pad after it
//!   (`gw_Script_PadApply`). Ports 2-4 have no controller.
//!
//! The randabs task: while `gd.frame() < FRAMES`, inside a live match a random human-style action
//! (an LCG over the ACTIONS table), elsewhere A / Start taps on a 24-frame cycle; then `gd.wait(1)`; at FRAMES it quits.

use melee_types::PADStatus;

/// `ACTIONS` of gen_pads.py: {buttons, stick x, stick y, min frames, max frames}.
const ACTIONS: [(u16, i8, i8, i64, i64); 22] = [
    (0, 0, 0, 10, 40),
    (0, 100, 0, 10, 45),
    (0, -100, 0, 10, 45),
    (0, 55, 0, 8, 30),
    (0, -55, 0, 8, 30),
    (0x400, 0, 0, 3, 4),
    (0x400, 100, 0, 3, 4),
    (0x400, -100, 0, 3, 4),
    (0x100, 0, 0, 3, 5),
    (0x100, 100, 0, 3, 8),
    (0x100, 0, 127, 3, 8),
    (0x100, 0, -127, 3, 8),
    (0x200, 0, 0, 3, 40),
    (0x200, 100, 0, 3, 8),
    (0x200, 0, 127, 3, 8),
    (0x200, 0, -127, 3, 20),
    (0x20, 0, 0, 5, 30),
    (0x20, 100, 0, 3, 8),
    (0x10, 0, 0, 3, 5),
    (0x100, -100, 0, 3, 8),
    (0, 0, -100, 5, 25),
    (0, 0, 100, 3, 6),
];

/// The randabs script's state plus the harness's (`gs.frame`, the match flags, port 1's claim).
pub struct RandAbs {
    frames: i64,
    s: i64,
    left: i64,
    cur: (u16, i8, i8),
    k: i64,
    /// port 1's claim: the pad and the samples it still holds
    claim: Option<((u16, i8, i8), i32)>,
    pub gframe: i64,
    pub active: bool,
    pub match_frame: i64,
}

impl RandAbs {
    /// From the pad file (`local FRAMES = N`, `new_player(SEED)`); None when it is not a randabs script.
    pub fn from_file(path: &str) -> Option<RandAbs> {
        let t = std::fs::read_to_string(path).ok()?;
        if !t.contains("coverage absolute-frame rand pad") {
            return None;
        }
        let num = |key: &str, end: char| -> Option<i64> {
            let i = t.find(key)? + key.len();
            t[i..].split(end).next()?.trim().parse().ok()
        };
        Some(RandAbs {
            frames: num("local FRAMES =", '\n')?,
            s: num("local p1 = new_player(", ')')?,
            left: 0,
            cur: (0, 0, 0),
            k: 0,
            claim: None,
            gframe: 0,
            active: false,
            match_frame: 0,
        })
    }

    fn lcg(s: i64) -> i64 {
        (s * 48271) % 2147483647
    }

    /// The script's `step(p1)`.
    fn step(&mut self) -> (u16, i8, i8) {
        if self.left <= 0 {
            self.s = Self::lcg(self.s);
            let a = ACTIONS[((self.s / 64) % ACTIONS.len() as i64) as usize];
            self.s = Self::lcg(self.s);
            self.left = a.3 + (self.s / 64) % (a.4 - a.3 + 1);
            self.cur = (a.0, a.1, a.2);
        }
        self.left -= 1;
        self.cur
    }

    /// One pass of the task's loop body (`gd.input` for the next frame). False once it has quit.
    pub fn run_task(&mut self) -> bool {
        if self.gframe >= self.frames {
            return false;
        }
        let pad = if self.active {
            self.step()
        } else {
            let ph = self.k % 24;
            self.k += 1;
            if ph < 4 {
                (0x100, 0, 0)
            } else if (12..16).contains(&ph) {
                (0x1000, 0, 0)
            } else {
                (0, 0, 0)
            }
        };
        self.claim = Some((pad, 1));
        true
    }

    /// `gw_Script_PadApply` for the next PADRead.
    pub fn pads(&self) -> [PADStatus; 4] {
        let mut p = [melee_platform::pad::pad_disconnected(); 4];
        if let Some(((b, x, y), n)) = self.claim {
            p[0] = PADStatus::zeroed();
            if n > 0 {
                p[0].button = b;
                p[0].stickX = x;
                p[0].stickY = y;
            }
        }
        p
    }

    /// `gw_Script_FramePost` up to the hash row: the hold, `gs.frame`, the match flags. Returns whether the frame is live.
    pub fn frame_post(&mut self, any_fighter: bool) -> bool {
        // gs_1p_start_dispatch in Script_FramePre (classic / adventure): after a stage's `Script_OnePStage`, the first frame
        // with a fighter goes live before the frame runs (match_frame 0), so its FramePost row is frame 1. The fighters of a
        // stage exist from the scene's enter on, so their presence at FramePre is their presence now.
        let onep = unsafe { &mut *core::ptr::addr_of_mut!(melee_modes::tu::gm_17c0::ONEP_STAGE_EVENT) };
        if any_fighter && *onep {
            *onep = false;
            if !self.active {
                self.active = true;
                self.match_frame = 0;
            }
        }
        if let Some((_, n)) = self.claim.as_mut() {
            if *n > 0 {
                *n -= 1;
            }
        }
        self.gframe += 1;
        if any_fighter && !self.active {
            self.active = true;
            self.match_frame = 0;
        } else if self.active {
            self.match_frame += 1;
        }
        self.active
    }

    /// A scene change (`gs.match_active = 0`).
    pub fn scene_changed(&mut self) {
        self.active = false;
    }
}

/// `gs_players_present` for any slot: a slot's fighter in the fighter GObj list (script_game.c `script_fighter`).
pub fn any_fighter() -> bool {
    unsafe {
        let head = melee_runtime::tu::gobj_native::HSD_GObjPLinkHead;
        if head.is_null() {
            return false;
        }
        (0..6).any(|slot| {
            let g = melee_fighter::player::get_entity(slot);
            if g.is_null() {
                return false;
            }
            let mut cur = *head.add(melee_types::HSD_GOBJ_PLINK_FIGHTER as usize);
            while !cur.is_null() {
                if cur == g {
                    return true;
                }
                cur = (*cur).next;
            }
            false
        })
    }
}

/// The run a [`frame_post_hook`] drives (one per process).
struct Run {
    ra: RandAbs,
    rows: usize,
    written: usize,
    out: std::fs::File,
    quit: bool,
}
static mut RUN: Option<Run> = None;

/// `melee diff`: the match frame whose state frame_post captures (-1: none).
pub static mut DIFF_CAPTURE: i64 = -1;

/// Start a run: the CSV header, the task's first pass (`gd.run` resumes it at once: it sets logic frame 0's pad).
pub fn start(mut ra: RandAbs, rows: usize, out: &str) -> Result<(), String> {
    use std::io::Write;
    let mut f = std::fs::File::create(out).map_err(|e| format!("{out}: {e}"))?;
    let _ = f.write_all(b"frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11\n");
    // MELEE_COV1P_PRERUN=1: resume the task once before logic frame 0. Off by default: the C task first sets a pad at the
    // boot scene's first FramePost, so logic frame 0 reads a neutral pad (with the pre-run, Adventure's intro was skipped
    // one frame early by the Start tap).
    if std::env::var("MELEE_COV1P_PRERUN").is_ok_and(|v| v == "1") {
        ra.run_task();
    }
    melee_engine::Engine::set_next_pads(Some(ra.pads()));
    unsafe {
        RUN = Some(Run { ra, rows, written: 0, out: f, quit: false });
        melee_engine::engine::FRAME_POST_HOOK = Some(frame_post_hook);
    }
    Ok(())
}

/// Every logic frame's `gw_Script_FramePost` (cov1p's header): the hold, the counters, the row, the task.
fn frame_post_hook() {
    use std::io::Write;
    let Some(r) = (unsafe { (*std::ptr::addr_of_mut!(RUN)).as_mut() }) else { return };
    if r.quit {
        return;
    }
    if std::env::var_os("MELEE_CT_LIVE").is_some() { eprintln!("ct --- frame_post gframe {} seed {:08x}", r.ra.gframe, unsafe { *melee_runtime::tu::random::HSD_RandCurrentSeed(melee_runtime::tu::random::global::state()) }); }
    if r.ra.frame_post(any_fighter()) {
        let w = melee_engine::Engine::frame_row();
        let mut line = format!("{},{:08X},{:08X},{},{:08X}", r.ra.match_frame, w[0], w[1], w[2], w[3]);
        for x in &w[4..16] {
            line.push_str(&format!(",{x:08X}"));
        }
        line.push('\n');
        let _ = r.out.write_all(line.as_bytes());
        if std::env::var_os("MELEE_ENGINE_ITEMS").is_some() {
            // the live items' hashed fields (the C side: a MELEE_SNAP_DUMP snapshot of the same frame)
            unsafe {
                let mut g = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(melee_types::HSD_GOBJ_PLINK_ITEM as usize);
                while !g.is_null() {
                    let ip = (*g).user_data as *mut melee_types::Item;
                    if !ip.is_null() {
                        let (p, v) = (&(*ip).pos, &(*ip).x40_vel);
                        eprintln!("f{} item kind {} msid {} pos {:.4} {:.4} {:.4} vel {:.4} {:.4} {:.4}", r.ra.match_frame, (*ip).kind as i32, (*ip).msid as i32, p.x, p.y, p.z, v.x, v.y, v.z);
                    }
                    g = (*g).next;
                }
            }
        }
        // `melee diff` on a 1P scenario: capture the state at this match frame (the C snapshot's frame)
        if unsafe { DIFF_CAPTURE } == r.ra.match_frame as i64 {
            unsafe { melee_engine::engine::CAPTURED = Some(melee_engine::Engine::capture_now()) };
        }
        r.written += 1;
        melee_core::ct::begin_frame(r.ra.match_frame as i64 + 1);
    }
    if r.written >= r.rows || !r.ra.run_task() {
        r.quit = true;
    }
    melee_engine::Engine::set_next_pads(Some(r.ra.pads()));
}

/// The run has its rows or the script quit; (rows written, logic frames).
pub fn done() -> Option<(usize, i64)> {
    let r = unsafe { (*std::ptr::addr_of!(RUN)).as_ref()? };
    r.quit.then_some((r.written, r.ra.gframe))
}

/// A scene began (`Script_SceneBegin`).
pub fn scene_began() {
    if let Some(r) = unsafe { (*std::ptr::addr_of_mut!(RUN)).as_mut() } {
        r.ra.scene_changed();
    }
}
