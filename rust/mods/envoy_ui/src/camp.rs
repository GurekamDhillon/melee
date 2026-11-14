//! The camp, C1 (UI-SPEC 2.2): one sticker list, one cursor (player 1), the party and run status on the right; player
//! 2 joins with Start. After a fight the same scene is the road between fights (the map, N2, UI-SPEC 2.4): the next
//! stop's poster and A to go on (the character select that follows stays, with its Envoy banner).

use crate::kit::*;
use crate::party::*;
use crate::{ck_name, Buf, VW};
use envoy_data::view::*;
use envoy_data::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_envoy_coop(on: i32) -> i32;
    fn gd_ui_pass(buttons: u32) -> i32;
    fn gd_ui_pad_present(port: u32) -> i32;
}

pub unsafe fn coop(on: i32) {
    gd_ui_envoy_coop(on);
}

pub const R_BIG: u8 = 0; // Continue run / Start run
pub const R_NEW: u8 = 1;
pub const R_FIGHTERS: u8 = 2;
pub const R_LENGTH: u8 = 3;
pub const R_TEAM: u8 = 4;
pub const R_NG: u8 = 5;
pub const R_COLLECTION: u8 = 6;
pub const R_BACK: u8 = 7;
pub const R_TAGBTN: u8 = 8;
// the road between fights
pub const R_GO: u8 = 10;
pub const R_ABANDON: u8 = 11;
pub const R_LEAVE: u8 = 12;
pub const R_DONE: u8 = 13;

pub struct Ui {
    pub cur: usize,
    pub prev_cur: usize,
    pub moved_at: i32,
    pub age: i32,
    pub dialog: bool,
    pub dcur: usize,
    pub p2: bool,
    pub p2_at: i32,
    pub p2_hold: i32,
    pub stick: [i8; 2],
    pub rep: [i32; 2],
    pub msg: &'static str,
    pub msg_at: i32,
    pub team: bool,
    /// A new run is armed from the camp and its road (the map and the fork) shows before fight 1.
    pub pre_road: bool,
    pub pre_at: i32,
}
pub static mut UI: Ui = Ui { cur: 0, prev_cur: 0, moved_at: -100, age: 0, dialog: false, dcur: 0, p2: false, p2_at: -100, p2_hold: 0, stick: [0; 2], rep: [0; 2], msg: "", msg_at: -100, team: false, pre_road: false, pre_at: 0 };

/// What the controllers did this frame (edges, with stick repeat).
#[derive(Clone, Copy, Default)]
pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub b: bool,
    pub b_held: bool,
    pub start: bool,
}

/// Any button or trigger held on `port` (a pad that cannot be read counts as nothing held).
pub unsafe fn any_held(port: usize) -> bool {
    let mut o = [0u8; 16];
    if gd_ui_pad(port as u32, o.as_mut_ptr()) != 0 {
        return false;
    }
    u16::from_le_bytes([o[0], o[1]]) & 0x1F7F != 0
}

pub unsafe fn read_pad(port: usize) -> Pad {
    let mut o = [0u8; 16];
    if gd_ui_pad(port as u32, o.as_mut_ptr()) != 0 {
        return Pad::default();
    }
    let held = u16::from_le_bytes([o[0], o[1]]);
    let edge = u16::from_le_bytes([o[12], o[13]]);
    let sy = o[3] as i8;
    let sx = o[2] as i8;
    let dir: i8 = if sy > 50 { 1 } else if sy < -50 { 2 } else if sx < -50 { 3 } else if sx > 50 { 4 } else { 0 };
    let u = &mut UI;
    let mut fire = 0i8;
    if dir != u.stick[port] {
        u.stick[port] = dir;
        u.rep[port] = 0;
        fire = dir;
    } else if dir != 0 {
        u.rep[port] += 1;
        if u.rep[port] > 18 && u.rep[port] % 6 == 0 {
            fire = dir;
        }
    }
    Pad {
        up: edge & 0x8 != 0 || fire == 1,
        down: edge & 0x4 != 0 || fire == 2,
        left: edge & 0x1 != 0 || fire == 3,
        right: edge & 0x2 != 0 || fire == 4,
        a: edge & 0x100 != 0,
        b: edge & 0x200 != 0,
        b_held: held & 0x200 != 0,
        start: edge & 0x1000 != 0,
    }
}

/// The rows for what is true now.
pub fn rows(after_fight: bool, ended: bool, has: bool, no_brawl: bool) -> &'static [u8] {
    if no_brawl {
        &[R_BACK]
    } else if after_fight && ended {
        &[R_DONE, R_NEW]
    } else if after_fight {
        &[R_GO, R_ABANDON, R_LEAVE]
    } else if has {
        &[R_BIG, R_NEW, R_FIGHTERS, R_TAGBTN, R_COLLECTION, R_BACK]
    } else {
        &[R_BIG, R_FIGHTERS, R_TAGBTN, R_COLLECTION, R_BACK]
    }
}

fn row_text(r: u8, has: bool, p2: bool) -> (&'static str, &'static str) {
    match r {
        R_BIG => (if has { "Continue run" } else { "Start run" }, ""),
        R_NEW => ("New run", ""),
        R_FIGHTERS => ("Fighters", if has { "Set for this run" } else if p2 { "2 each" } else { "2" }),
        R_LENGTH => ("Run length", "Short, 5 fights"),
        R_TEAM => ("Team attack", "Off"),
        R_NG => ("NG+ level", "0 only"),
        R_COLLECTION => ("How to play", ""),
        R_TAGBTN => ("Tag button", unsafe { ["Y", "X", "D-pad (classic)"][TAG_BUTTON.clamp(0, 2) as usize] }),
        R_BACK => ("Back", ""),
        R_GO => ("On to the next fight", ""),
        R_ABANDON => ("Abandon run", ""),
        R_LEAVE => ("Leave (the run is kept)", ""),
        R_DONE => ("Back to camp", if p2 { "" } else { "" }),
        _ => ("", ""),
    }
}

fn row_say(r: u8, has: bool, p2: bool) -> &'static str {
    match r {
        R_BIG if has => "Pick up the saved run with the saved party.",
        R_BIG if p2 => "Player 2 is in: 5 stocks together. Straight into fight 1.",
        R_BIG => "Three rooms and a boss. Straight into room 1.",
        R_FIGHTERS if has => "The party is set for this run. Start a new run to change it.",
        R_NEW => "Start over from fight 1. The saved run is lost.",
        R_FIGHTERS => "Bring two fighters each. You swap between your two mid-fight.",
        R_LENGTH => "A session is three rooms and a boss.",
        R_TEAM => "Your attacks never hit your partner in this build.",
        R_NG => "NG+ opens when a run is cleared. Not in this build yet.",
        R_COLLECTION => "The goal, your pair, Heat, drives and the buttons, on one card.",
        R_TAGBTN => "Y and X tag at once, no cutscene. The D-pad is the classic tag with its cutscene.",
        R_BACK => "Back to the main menu.",
        R_GO => "The fights of this session follow back to back.",
        R_ABANDON => "End this run for good.",
        R_LEAVE => "Back to the main menu. The run waits at the camp.",
        R_DONE => "The run is over. Back to the camp.",
        _ => "",
    }
}

/// One list row sticker (idle: white, hovered: sunny, moved right and level).
fn list_row(e: &mut E, i: usize, x: f32, y: f32, w: f32, h: f32, on: bool, t: i32, label: &str, value: &str, big: bool, sub: &str) {
    const ROT: [f32; 8] = [-1.2, 0.9, -0.7, 1.0, -0.9, 0.7, -1.1, 1.2];
    let (_, a) = enter(t - i as i32 * 2);
    if a <= 0.0 {
        return;
    }
    let dx = if on { 12.0 } else { 0.0 };
    let deg = if on { 0.0 } else { ROT[i % 8] };
    let x = x + dx;
    sticker(e, if on { S_FACE_HOVER } else { S_FACE_WHITE }, x, y, w, h, deg, alpha(WHITE, a), |e| {
        if big {
            e.text_ex(x + 14.0, y + 6.0, d(23), INK, ALIGN_LEFT, w - 20.0, label);
            e.text_ex(x + 14.0, y + 34.0, bs(13), if on { INK } else { LILAC }, ALIGN_LEFT, w - 20.0, sub);
        } else {
            e.text_ex(x + 12.0, y + h * 0.5 - 8.0, d(15), INK, ALIGN_LEFT, 0.0, label);
            if !value.is_empty() {
                e.text_ex(x + w - 12.0, y + h * 0.5 - 7.0, bs(12), if on { INK } else { LILAC }, ALIGN_RIGHT, 0.0, value);
            }
        }
    });
}

/// A fighter card (72 x 94): head, name; dashed "Pick" when not chosen yet.
fn fcard(e: &mut E, x: f32, y: f32, deg: f32, ck: i32, tag: i32, col: u32, ids: &[i32], lvs: &[i32]) {
    // any real fighter (retail or an m-ex slot); only an unset seat or an unrolled Random is the "?" card
    let known = ck >= 0 && ck != crate::picker::RANDOM;
    sticker(e, if known { S_FACE_WHITE } else { S_FACE_CREAM }, x, y, 72.0, 94.0, deg, WHITE, |e| {
        if known {
            spr_c(e, S_HEAD, x + 36.0, y + 26.0, 1.6, WHITE);
            e.text_ex(x + 36.0, y + 46.0, d(13), INK, ALIGN_CENTER, 66.0, if (0..26).contains(&ck) { ck_name(ck) } else { crate::picker::short(ck) });
            let mut n = 0;
            for k in 0..ids.len() {
                if (1..=POOL_LEN as i32).contains(&ids[k]) && n < 4 {
                    chip(e, x + 12.0 + n as f32 * 16.0, y + 74.0, ids[k] as usize - 1, lvs[k], 13.0);
                    n += 1;
                }
            }
        } else {
            text_edge(e, x + 36.0, y + 14.0, d(30), SUNNY, ALIGN_CENTER, 1.4, "?");
            e.text_ex(x + 36.0, y + 56.0, d(13), LILAC, ALIGN_CENTER, 66.0, "Random");
        }
        if tag > 0 {
            face(e, if tag == 1 { S_FACE_CORAL } else { S_FACE_LAGOON }, x - 6.0, y - 8.0, 26.0, 18.0, WHITE);
            e.text_ex(x + 7.0, y - 6.0, d(12), WHITE, ALIGN_CENTER, 0.0, if tag == 1 { "P1" } else { "P2" });
        }
        let _ = col;
    });
}

/// The camp: input (returns what to do) and drawing. `has`: a saved run; `v`: the live view, when there is one.
pub const DO_NOTHING: i32 = 0;
pub const DO_CLOSE: i32 = 1;
pub const DO_ARM: i32 = 2; // + op << 8
/// gd_ui_pass: D-pad left / right and the main stick's X (the host's PASS_STICK_X).
const PASS_LR: u32 = 0x0001 | 0x0002 | 0x1_0000;

pub unsafe fn frame(e: &mut E, after_fight: bool, view: Option<&[i32; VW]>, rec: Option<[i32; REC_WORDS]>, no_brawl: bool) -> i32 {
    let u = &mut UI;
    u.age += 1;
    let t = u.age;
    if t == 1 {
        TAG_BUTTON = gd_ui_envoy_set_tag_button(-1).clamp(0, 2);
    }
    let ended = view.is_some_and(|v| v[V_STATE] == 2 || v[V_STATE] == 3);
    // the forked road: the cursor is the sim's (synced input, so co-op and netplay agree), but this scene holds the
    // game's pads; left / right (D-pad and the main stick) are handed through while the fork is on screen
    let pre = UI.pre_road && !after_fight;
    let fork_up = (after_fight || pre) && view.is_some_and(|v| v[V_ROAD_N] > 1 && v[V_SESS_NEXT] != 1) && !ended && !crate::picker::is_open();
    gd_ui_pass(if fork_up { PASS_LR } else { 0 });
    // sessions: the fight was won and the next is in the same session: no road, the banner, then on at once
    if after_fight {
        if let Some(v) = view.filter(|v| v[V_SESS_NEXT] == 1) {
            e.layer(LAYER_MENU, 0);
            e.rect(-107.0, 0.0, 854.0, 480.0, 0x1A10_4AFF);
            session_banner(e, v, 1.0, 190.0);
            return if t >= 40 { DO_ARM | (2 << 8) } else { DO_NOTHING };
        }
    }
    let has = rec.is_some();
    if crate::picker::is_open() {
        if !crate::picker::frame(e) {
            u.age = 1; // back on the camp: the closing press must not choose a row
        }
        return DO_NOTHING;
    }
    // the How to play card (help.rs) over the camp: it closes on A, B or Start
    if crate::help::is_open() {
        let pad = read_pad(0);
        if !crate::help::frame(e, &pad, TAG_BUTTON) {
            u.age = 1; // back on the camp: the closing press must not choose a row
        }
        return DO_NOTHING;
    }
    let rs: &[u8] = if pre { &[R_GO] } else { rows(after_fight, ended, has, no_brawl) };
    if u.cur >= rs.len() {
        u.cur = 0;
    }
    // ---- input (the press that opened the camp must not also choose a row)
    let p1 = read_pad(0);
    let p2 = read_pad(1);
    let mut act = DO_NOTHING;
    // after a fight the road holds 40 frames: an A mashed through the results must not skip it
    if t > if after_fight { 40 } else { 3 } {
        if !after_fight && !no_brawl {
            // a port that mirrors port 1 (one controller read twice) must not join: P2's Start alone joins
            if p2.start && !p1.start && !u.p2 && gd_ui_pad_present(1) != 0 {
                u.p2 = true;
                u.p2_at = t;
                gd_ui_envoy_coop(1);
            }
            if u.p2 && p2.b_held {
                u.p2_hold += 1;
                if u.p2_hold >= 30 {
                    u.p2 = false;
                    u.p2_hold = 0;
                    gd_ui_envoy_coop(0);
                }
            } else {
                u.p2_hold = 0;
            }
        }
        if u.dialog {
            if p1.left || p1.right || p1.up || p1.down {
                u.dcur ^= 1;
            }
            if p1.b {
                u.dialog = false;
            } else if p1.a {
                u.dialog = false;
                if u.dcur == 1 {
                    // abandon it: a new run (on the main menu) or the run ends (after a fight)
                    act = if after_fight { DO_ARM | (3 << 8) } else { DO_ARM | (1 << 8) };
                }
            }
        } else {
            if p1.up {
                u.cur = (u.cur + rs.len() - 1) % rs.len();
            }
            if p1.down {
                u.cur = (u.cur + 1) % rs.len();
            }
            if p1.up || p1.down {
                u.prev_cur = u.cur;
                u.moved_at = t;
            }
            let r = rs[u.cur];
            if p1.b && pre {
                // back from the road: the run is disarmed again (nothing was played)
                u.pre_road = false;
                act = DO_ARM | (7 << 8);
            } else if p1.b && !after_fight {
                act = DO_CLOSE;
            }
            let go = p1.a || (p1.start && !after_fight && !no_brawl);
            if go {
                let r = if p1.start { R_BIG } else { r };
                match r {
                    // a new run: arm it here and show its road before fight 1 (op 5); A on the road launches (op 6)
                    R_BIG => act = DO_ARM | (if has { 2 } else { 5 } << 8),
                    R_NEW => {
                        if has && !after_fight {
                            u.dialog = true;
                            u.dcur = 0;
                        } else {
                            act = DO_ARM | (1 << 8);
                        }
                    }
                    R_BACK => act = DO_CLOSE,
                    R_FIGHTERS if !has => crate::picker::open(u.p2),
                    R_COLLECTION => crate::help::open(),
                    R_TAGBTN => {
                        TAG_BUTTON = (TAG_BUTTON + 1) % 3;
                        gd_ui_envoy_set_tag_button(TAG_BUTTON);
                        u.msg = row_say(r, has, u.p2);
                        u.msg_at = t;
                    }
                    R_GO if pre => act = DO_ARM | (6 << 8),
                    R_GO => act = DO_ARM | (2 << 8),
                    R_ABANDON => {
                        u.dialog = true;
                        u.dcur = 0;
                    }
                    R_LEAVE => act = DO_ARM | (4 << 8), // no character select to go back to: the main menu, the run kept
                    R_DONE => act = DO_ARM | (4 << 8),
                    _ => {
                        u.msg = row_say(r, has, u.p2);
                        u.msg_at = t;
                    }
                }
            }
        }
    }
    // ---- drawing
    e.layer(LAYER_MENU, 0);
    let ex = 107.0;
    spr(e, S_BG_CAMP, -ex, 0.0, 854.0, 480.0, WHITE);
    if after_fight || pre {
        road(e, t, view, rec.as_ref());
    } else {
        // title
        spr_c(e, S_EMBLEM, 36.0, 30.0, 1.0, WHITE);
        text_edge(e, 56.0, 15.0, d(23), WHITE, ALIGN_LEFT, 2.2, "Envoy camp");
        status(e, t, has, rec.as_ref(), u.p2);
    }
    // the list
    let lx = 30.0;
    let forked = view.is_some_and(|v| v[V_ROAD_N] > 1 && !ended);
    let mut y = if after_fight || pre { if forked { 298.0 } else { 288.0 } } else { 58.0 };
    for (i, &r) in rs.iter().enumerate() {
        let on = i == u.cur && !u.dialog;
        let (label, value) = row_text(r, has, u.p2);
        if r == R_BIG {
            let sub = if let Some(rc) = rec.as_ref() {
                let mut b = Buf::new();
                loop_fight(&mut b, rec_loop(rc[R_LOOP]), rc[R_FIGHT], 4);
                b.s(". ").i(rc[R_POOL]).s(" stocks left.");
                b
            } else {
                let mut b = Buf::new();
                b.s(if u.p2 { "Two players, from fight 1." } else { "A new run, from fight 1." });
                b
            };
            list_row(e, i, lx, y, 234.0, 54.0, on, t, label, "", true, sub.str());
            if on {
                spr(e, S_PTR, 6.0, y + 14.0, 0.0, 0.0, WHITE);
            }
            y += 62.0;
        } else {
            let w = if r == R_BACK { 112.0 } else if after_fight { 280.0 } else { 234.0 };
            list_row(e, i, lx, y, w, 30.0, on, t, label, value, false, "");
            if on {
                spr(e, S_PTR, 4.0, y + 2.0, 0.0, 0.0, WHITE);
            }
            y += if after_fight { 34.0 } else { 36.0 };
        }
    }
    // strip and hints
    let r = rs[u.cur];
    let say = if t - u.msg_at < 150 && !u.msg.is_empty() { u.msg } else if no_brawl { "Envoy needs your Brawl disc. Add it in the launcher." } else { row_say(r, has, u.p2) };
    strip(e, 24.0, 590.0, say);
    if pre {
        hints(e, &[("Left / Right", "Road"), ("A", "Start fight 1"), ("B", "Back to camp")]);
    } else if after_fight {
        hints(e, &[("Up / Down", "Choose"), ("A", if r == R_GO { "Start the fight" } else { "Choose" })]);
    } else {
        let a = match r {
            R_BIG => if has { "Continue run" } else { "Start run" },
            R_NEW => "New run",
            R_BACK => "Main menu",
            _ => "Open",
        };
        hints(e, &[("Up / Down", "Choose"), ("A", a), ("B", "Main menu"), ("Start", if has { "Continue run" } else { "Start run" }), ("P2: Start", if u.p2 { "hold B: Leave" } else { "Join" })]);
    }
    if u.dialog {
        e.rect(-ex, 0.0, 854.0, 480.0, 0x1A10_4AB0);
        let (x, y) = (148.0, 132.0);
        let (s, _) = enter(4);
        let _ = s;
        sticker(e, S_FACE_WHITE, x, y, 344.0, 176.0, -1.0, WHITE, |e| {
            let title = if after_fight { "Abandon this run?" } else { "Abandon the saved run?" };
            e.text_ex(x + 20.0, y + 16.0, d(21), INK, ALIGN_LEFT, 0.0, title);
            let line = if let Some(rc) = rec.as_ref() {
                let mut b = Buf::new();
                if rec_loop(rc[R_LOOP]) > 1 {
                    b.s("Loop ").i(rec_loop(rc[R_LOOP])).s(", f");
                } else {
                    b.s("F");
                }
                b.s("ight ").i(rc[R_FIGHT] + 1).s(" and its ").i(rc[R_POOL]).s(" stocks will be gone for good.");
                b
            } else {
                let mut b = Buf::new();
                b.s("The run and its drives will be gone for good.");
                b
            };
            e.text_ex(x + 20.0, y + 52.0, bs(14), LILAC, ALIGN_LEFT, 300.0, line.str());
            for k in 0..2 {
                let bx = x + 20.0 + k as f32 * 156.0;
                let on = UI.dcur == k;
                face(e, if on { S_FACE_HOVER } else if k == 1 { S_FACE_CORAL } else { S_FACE_WHITE }, bx, y + 116.0, 144.0, 34.0, WHITE);
                e.text_ex(bx + 72.0, y + 124.0, d(15), INK, ALIGN_CENTER, 0.0, if k == 0 { "Keep the run" } else { "Abandon it" });
            }
        });
        hints(e, &[("Left / Right", "Choose"), ("A", if UI.dcur == 0 { "Keep the run" } else { "Abandon it" }), ("B", "Keep the run")]);
    }
    act
}

/// The camp's status board: both players' fighters, the run strip, stocks, keystones, the locked rules.
unsafe fn status(e: &mut E, t: i32, has: bool, rec: Option<&[i32; REC_WORDS]>, p2: bool) {
    let tb = t - 16;
    if tb <= 0 {
        return;
    }
    let party = rec.map(|r| party_from_record(r));
    let coop = party.as_ref().map_or(p2, |p| p.players == 2);
    // P1 fighters
    let lp = crate::picker::LAST;
    let (ck0, ck1) = party.as_ref().map_or((lp[0], lp[1]), |p| (p.pairs[0], p.pairs[1]));
    let z = [0i32; SLOTS];
    let (i0, l0, i1, l1) = party.as_ref().map_or((z, z, z, z), |p| (p.ids[0], p.lvs[0], p.ids[1], p.lvs[1]));
    fcard(e, 292.0, 54.0, -3.0, ck0, 1, CORAL, &i0, &l0);
    fcard(e, 368.0, 60.0, 2.0, ck1, 0, CORAL, &i1, &l1);
    spr_c(e, S_SWAP, 365.0, 103.0, 1.0, WHITE);
    if coop {
        let (c2, c3) = party.as_ref().map_or((lp[2], lp[3]), |p| (p.pairs[2], p.pairs[3]));
        let (i2, l2, i3, l3) = party.as_ref().map_or((z, z, z, z), |p| (p.ids[2], p.lvs[2], p.ids[3], p.lvs[3]));
        fcard(e, 468.0, 58.0, -2.0, c2, 2, LAGOON, &i2, &l2);
        fcard(e, 544.0, 54.0, 3.0, c3, 0, LAGOON, &i3, &l3);
        spr_c(e, S_SWAP, 541.0, 103.0, 1.0, WHITE);
        if p2 && UI.p2_at > 0 && t - UI.p2_at < 90 {
            burst(e, S_BURST_LAGOON, 566.0, 26.0, 74.0, t - UI.p2_at, "P2 joined", 12);
        }
    } else {
        let (x, y) = (462.0, 58.0);
        sticker(e, S_FACE_CREAM, x, y, 152.0, 92.0, 2.0, WHITE, |e| {
            e.text_ex(x + 76.0, y + 14.0, d(17), LILAC, ALIGN_CENTER, 0.0, "Player 2");
            face(e, S_FACE_WHITE, x + 26.0, y + 44.0, 100.0, 22.0, WHITE);
            e.text_ex(x + 76.0, y + 47.0, d(13), INK, ALIGN_CENTER, 0.0, "Start: Join");
            e.text_ex(x + 76.0, y + 72.0, bs(11), LILAC, ALIGN_CENTER, 0.0, "on controller 2");
        });
    }
    // run strip
    let (x, y) = (290.0, 182.0);
    sticker(e, if has { S_FACE_SUNNY } else { S_FACE_WHITE }, x, y, 312.0, 46.0, -1.0, WHITE, |e| {
        face(e, S_FACE_INK, x + 10.0, y - 12.0, 86.0, 18.0, WHITE);
        e.text_ex(x + 53.0, y - 10.0, d(11), WHITE, ALIGN_CENTER, 0.0, if has { "Saved run" } else { "A full run" });
        let done = party.as_ref().map_or(-1, |p| p.fight);
        marks(e, x + 30.0, y + 24.0, 5, done, 60.0, 7.0);
        if let Some(lp) = party.as_ref().map(|p| p.lp).filter(|&l| l > 1) {
            let mut s = Buf::new();
            s.s("Loop ").i(lp);
            e.text_ex(x + 300.0, y - 10.0, d(11), INK, ALIGN_RIGHT, 0.0, s.str());
        }
    });
    // the best loop reached on this save (endless NG+)
    let best = crate::best_loop();
    if best > 1 {
        let mut s = Buf::new();
        s.s("Best: loop ").i(best);
        e.text_ex(x + 156.0, y + 52.0, d(13), WHITE, ALIGN_CENTER, 0.0, s.str());
    }
    // stocks
    let pool = party.as_ref().map_or(if coop { 5 } else { 4 }, |p| p.pool);
    stocks(e, 294.0, 244.0, 128.0, pool, if coop { 5 } else { 4 }, if coop { "Team stocks" } else { "Solo stocks" }, 2.0);
    if p2 && UI.p2_at > 0 && t - UI.p2_at < 90 && !has {
        burst(e, S_BURST_LAGOON, 420.0, 248.0, 40.0, t - UI.p2_at, "+1", 14);
    }
    if let Some(p) = party.as_ref() {
        let (ks, kn) = party_keys(p);
        for i in 0..kn {
            kpill(e, 440.0 + if i == 1 { 6.0 } else { 0.0 }, 242.0 + i as f32 * 29.0, ks[i], if i % 2 == 0 { -2.0 } else { 2.0 });
        }
        let (x, y) = (296.0, 304.0);
        sticker(e, S_FACE_LILAC, x, y, 124.0, 52.0, -2.0, WHITE, |e| {
            spr_c(e, S_LOCK, x + 16.0, y + 15.0, 1.0, WHITE);
            e.text_ex(x + 28.0, y + 7.0, d(13), INK, ALIGN_LEFT, 0.0, "Set for this run");
            e.text_ex(x + 12.0, y + 29.0, bs(11), INK, ALIGN_LEFT, 0.0, "Short run, team attack off");
        });
    }
}

/// Between fights: the road (map N2): run strip, the next stop's poster, the party.
unsafe fn road(e: &mut E, t: i32, view: Option<&[i32; VW]>, rec: Option<&[i32; REC_WORDS]>) {
    let Some(v) = view else { return };
    let p = party_from_view(v);
    let next = v[V_FIGHT];
    let ended = v[V_STATE] == 2 || v[V_STATE] == 3;
    // endless loops: the boss fell and the run goes on at fight 1 of the next loop (the clear moment)
    let looped = v[V_STATE] == 1 && next == 0 && v[V_LOOP] > 1;
    let _ = rec;
    // run strip
    let act = crate::map::has(v) && !ended;
    if act {
        // the act map (Slay the Spire style, the sim's words): rows up to the boss, P1's cursor picks the next fight
        sticker(e, S_FACE_INK, 16.0, 8.0, 300.0, 272.0, -0.4, WHITE, |e| {
            face(e, S_FACE_INK, 16.0, 8.0, 300.0, 272.0, WHITE);
            crate::map::draw(e, v, 40.0, 14.0, 236.0, 262.0, 10.0);
        });
        let picking = v[V_MAP_PICK] == 1;
        let mut s = Buf::new();
        if looped {
            s.s("Act ").i(v[V_LOOP]).s(" begins.");
        } else {
            s.s(if picking { "Choose the next fight" } else { "The road ahead" });
        }
        text_edge(e, 340.0, 10.0, d(20), SUNNY, ALIGN_LEFT, 1.8, s.str());
        if let Some((_, k)) = crate::map::cursor(v) {
            let line = match k {
                MN_DUEL => "Duels: one foe at a time.",
                MN_CROWD => "Crowd: three foes at once.",
                MN_ELITE => "Elite: strong foes, better rewards.",
                MN_BOSS => "The boss of the act.",
                _ => crate::map::kind_word(k),
            };
            text_edge(e, 340.0, 38.0, bs(14), WHITE, ALIGN_LEFT, 1.4, line);
        }
        if picking {
            let hint = if v[V_HUMANS] > 1 { "P1 Left / Right picks, A goes. P2 Left / Right flags a wish." } else { "Left / Right picks the lane, A goes." };
            text_edge(e, 340.0, 58.0, bs(12), PALE, ALIGN_LEFT, 1.2, hint);
        }
    }
    if act {
    } else if crate::map::has(v) {
    } else {
    let (x, y) = (16.0, 8.0);
    sticker(e, S_FACE_SUNNY, x, y, 608.0, 46.0, -0.4, WHITE, |e| {
        let n = v[V_FIGHTS].clamp(1, 8);
        let gap = 540.0 / (n - 1).max(1) as f32;
        marks(e, x + 34.0, y + 23.0, n, next, gap, 7.0);
        if !ended {
            spr(e, S_PTRDOWN, x + 34.0 + next as f32 * gap - 13.0, y + 34.0, 26.0, 18.0, WHITE);
        }
    });
    }
    if v[V_ROAD_N] > 1 && !ended && !act {
        fork(e, t, v, looped);
        return;
    }
    if act {
    } else if ended || looped {
        let won = v[V_STATE] == 2 || looped;
        let (x, y) = (20.0, 70.0);
        sticker(e, S_FACE_WHITE, x, y, 300.0, 200.0, -1.0, WHITE, |e| {
            e.rect(x + 1.5, y + 1.5, 297.0, 56.0, if won { SUNNY } else { CORAL });
            e.text_ex(x + 16.0, y + 12.0, d(28), INK, ALIGN_LEFT, 0.0, if won { "Run cleared!" } else { "The run ended" });
            let mut s = Buf::new();
            if looped {
                s.s("Loop ").i(v[V_LOOP]).s(" begins. Everything kept, the foes hit harder. ").i(v[V_POOL]).s(" stocks.");
            } else if won {
                s.s("All ").i(v[V_FIGHTS]).s(" fights, ").i(v[V_POOL]).s(" stocks to spare.");
            } else if v[V_LOOP] > 1 {
                s.s("Reached loop ").i(v[V_LOOP]).s(", fight ").i(next + 1).s(".");
            } else {
                s.s("Fight ").i(next + 1).s(" took the last stock.");
            }
            e.text_ex(x + 16.0, y + 76.0, bs(14), INK, ALIGN_LEFT, 270.0, s.str());
            let mut s = Buf::new();
            s.s("Drives fired: ").i(v[V_FIRES]);
            e.text_ex(x + 16.0, y + 110.0, d(17), INK, ALIGN_LEFT, 0.0, s.str());
            // the build the run ended with, a row a player (11.11)
            let any = (0..p.players).any(|q| p.items[q].iter().any(|&c| c > 0));
            if any {
                e.text_ex(x + 16.0, y + 136.0, d(13), LILAC, ALIGN_LEFT, 0.0, "Build");
                for q in 0..p.players {
                    let ry = y + 150.0 + q as f32 * 24.0;
                    if p.players == 2 {
                        e.text_ex(x + 16.0, ry - 5.0, d(11), if q == 0 { CORAL } else { LAGOON }, ALIGN_LEFT, 0.0, if q == 0 { "P1" } else { "P2" });
                    }
                    build_row(e, &p.items[q], x + 40.0, ry + 2.0, 250.0, 18.0);
                }
            }
        });
        if won {
            spr_rot(e, S_RAYS, 170.0, 160.0, 1.6, (t as f32) * 0.07, alpha(WHITE, 0.13));
            burst(e, S_BURST_CORAL, 300.0, 84.0, 80.0, t - 10, "Cleared", 14);
        }
    } else {
        // the poster: what the next stop is
        let kind = v[V_FIGHT_KIND];
        let (kname, kline, ifwin) = match kind {
            FK_PACK => ("Pack", "Two fighters at once.", "The next fight follows; the pick waits for the stop."),
            FK_ELITE => ("Elite", "A strong fighter with drives of its own.", "The next fight follows; the pick waits for the stop."),
            FK_BOSS => ("Boss", "The last fight of the loop.", "The run is cleared, and the next loop begins."),
            _ => ("Duel", "One fighter, nothing extra.", "The next fight follows; the pick waits for the stop."),
        };
        let (x, y) = (20.0, 70.0);
        let (_, a) = enter(t - 2);
        if a > 0.0 {
            sticker(e, S_FACE_WHITE, x, y, 300.0, 200.0, -1.0, alpha(WHITE, a), |e| {
                e.rect(x + 1.5, y + 1.5, 297.0, 56.0, if kind == FK_BOSS { CORAL } else { SUNNY });
                e.text_ex(x + 16.0, y + 6.0, d(28), INK, ALIGN_LEFT, 0.0, kname);
                let mut s = Buf::new();
                loop_fight(&mut s, v[V_LOOP], next, v[V_FIGHTS]);
                e.text_ex(x + 16.0, y + 36.0, bs(13), INK, ALIGN_LEFT, 0.0, s.str());
                // the fight was left from the pause (V_OUTCOME 3): the run is kept, the fight starts over
                let kline = if v[V_OUTCOME] == 3 { "You left this fight. It starts over." } else { kline };
                e.text_ex(x + 16.0, y + 70.0, bs(14), INK, ALIGN_LEFT, 270.0, kline);
                e.rect(x + 16.0, y + 104.0, 268.0, 2.0, PALE);
                e.text_ex(x + 16.0, y + 116.0, d(15), INK, ALIGN_LEFT, 0.0, "If you win");
                e.text_ex(x + 16.0, y + 138.0, bs(13), LILAC, ALIGN_LEFT, 270.0, ifwin);
                let mut s = Buf::new();
                s.s("Pool: ").i(v[V_POOL]).s(" stocks. Next: ");
                session_line(&mut s, v[V_SESS_KINDS]);
                s.s(".");
                e.text_ex(x + 16.0, y + 168.0, bs(13), LILAC, ALIGN_LEFT, 270.0, s.str());
            });
        }
    }
    // party (fighter stickers 140 x 80)
    let sets = if p.shared { 1 } else { 2 };
    let mut k = 0;
    const ROT: [f32; 4] = [-0.6, 0.5, 0.4, -0.5];
    for q in 0..p.players {
        for f in 0..sets {
            let set = q * 2 + f;
            let fx = 340.0 + (k % 2) as f32 * 146.0;
            let fy = if act { 84.0 } else { 72.0 } + (k / 2) as f32 * 90.0;
            let (_, a) = enter(t - 6 - k as i32 * 3);
            if a > 0.0 {
                fighter_sticker(e, fx, fy, ROT[k % 4], &p, set, f == 0, alpha(WHITE, a));
            }
            k += 1;
        }
    }
    let sy = if act { 84.0 } else { 72.0 } + ((k + 1) / 2) as f32 * 90.0;
    stocks(e, 342.0, sy, 128.0, p.pool, if p.players == 2 { 5 } else { 4 }, "Team stocks", -1.5);
    let (ks, kn) = party_keys(&p);
    for i in 0..kn {
        kpill(e, 344.0 + (i % 2) as f32 * 142.0, sy + 62.0 + (i / 2) as f32 * 28.0, ks[i], 0.0);
    }
}

/// A fighter sticker (UI-SPEC 3.5): 140 x 80, head, name, chips.
pub fn fighter_sticker(e: &mut E, x: f32, y: f32, deg: f32, p: &Party, set: usize, first: bool, rgba: u32) {
    sticker(e, S_FACE_WHITE, x, y, 140.0, 80.0, deg, rgba, |e| {
        spr_c(e, S_HEAD, x + 16.0, y + 14.0, 1.0, WHITE);
        e.text_ex(x + 33.0, y + 6.0, d(15), INK, ALIGN_LEFT, 104.0, ck_name(p.pairs[set]));
        let drives = p.ids[set].iter().any(|&id| (1..=POOL_LEN as i32).contains(&id));
        let pl = (set / 2).min(1);
        if !drives && p.items[pl].iter().any(|&c| c > 0) {
            // envoy items: the player's build, in their colours (11.11, showing off a build)
            build_row(e, &p.items[pl], x + 6.0, y + 48.0, 128.0, 18.0);
        } else {
            for k in 0..SLOTS {
                let cx = x + 17.0 + k as f32 * 26.0;
                let id = p.ids[set][k];
                if (1..=POOL_LEN as i32).contains(&id) {
                    chip(e, cx, y + 48.0, id as usize - 1, p.lvs[set][k], 22.0);
                } else {
                    spr_c(e, S_DISC, cx, y + 48.0, 19.0 / 32.0, alpha(PALE, 0.5));
                }
            }
        }
        if first && p.players == 2 {
            let pl = set / 2;
            face(e, if pl == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x - 6.0, y - 9.0, 26.0, 18.0, WHITE);
            e.text_ex(x + 7.0, y - 7.0, d(12), WHITE, ALIGN_CENTER, 0.0, if pl == 0 { "P1" } else { "P2" });
        }
    });
}

/// Player 2 has joined at the camp.
pub unsafe fn p2_in() -> bool {
    UI.p2
}

/// A reward token (RK_*): its word and its face. Data-driven: a new kind is one row here.
fn token_look(k: i32) -> (&'static str, usize) {
    match k {
        RK_PICK => ("Pick", S_FACE_WHITE),
        RK_FAMILY => ("Family pick", S_FACE_LAGOON),
        RK_RARE => ("Rare pick", S_FACE_LILAC),
        RK_LEVEL => ("Level-up", S_FACE_SUNNY),
        RK_COINS => ("Coins", S_FACE_SUNNY),
        RK_STOCK => ("Stock", S_FACE_CORAL),
        RK_ELITE_DRIVE => ("Elite's drive", S_FACE_CORAL),
        RK_KEYSTONE => ("Keystone", S_FACE_INK),
        _ => ("?", S_FACE_WHITE),
    }
}

/// A row of tokens from a packed token word, from (x, y), wrapping at `w`. Returns the height used.
pub fn tokens(e: &mut E, x: f32, y: f32, w: f32, q: i32, n: i32) -> f32 {
    let (mut cx, mut cy) = (x, y);
    for i in 0..n.clamp(0, 8) {
        let k = token(q, i);
        if k == 0 {
            continue;
        }
        let (word, fc) = token_look(k);
        let tw_ = tw(word, d(11)) + 14.0;
        if cx + tw_ > x + w {
            cx = x;
            cy += 22.0;
        }
        face(e, fc, cx, cy, tw_, 19.0, WHITE);
        let ink = if fc == S_FACE_INK || fc == S_FACE_CORAL || fc == S_FACE_LAGOON || fc == S_FACE_LILAC { WHITE } else { INK };
        e.text_ex(cx + tw_ * 0.5, cy + 3.0, d(11), ink, ALIGN_CENTER, 0.0, word);
        cx += tw_ + 5.0;
    }
    cy - y + 22.0
}

fn fight_word(fk: i32) -> &'static str {
    match fk {
        FK_PACK => "Pack",
        FK_ELITE => "Elite",
        FK_BOSS => "Boss",
        _ => "Duel",
    }
}

fn road_name(k: i32) -> &'static str {
    match k {
        ROAD_STEEP => "Steep road",
        ROAD_SWARM => "Swarm road",
        ROAD_SHIFT => "Shifting road",
        _ => "Open road",
    }
}

/// The forked road (step 8, M1): two or three posters side by side; P1's cursor (the sim's V_ROAD_CUR) is the one
/// A takes, P2's lagoon flag is a wish. Everything on a poster is known.
unsafe fn fork(e: &mut E, t: i32, v: &[i32; VW], looped: bool) {
    let n = v[V_ROAD_N].clamp(2, ROADS_MAX as i32) as usize;
    let cur = v[V_ROAD_CUR].clamp(0, n as i32 - 1) as usize;
    let pw = if n == 2 { 296.0 } else { 194.0 };
    let gap = (608.0 - pw * n as f32) / (n - 1) as f32;
    let mut title = Buf::new();
    if looped {
        title.s("Loop ").i(v[V_LOOP]).s(" begins. Choose the road.");
    } else {
        title.s("Choose the road. ");
        loop_fight(&mut title, v[V_LOOP], v[V_FIGHT], v[V_FIGHTS]);
    }
    text_edge(e, 20.0, 64.0, d(16), WHITE, ALIGN_LEFT, 1.6, title.str());
    // how to choose: P1's left / right moves the pointer (A takes it); in co-op P2's left / right plants the flag
    let hint = if v[V_HUMANS] > 1 { "P1 Left / Right picks, A takes it. P2 Left / Right flags a wish." } else { "Left / Right picks the road, A takes it." };
    text_edge(e, 624.0, 66.0, bs(12), WHITE, ALIGN_RIGHT, 1.4, hint);
    for r in 0..n {
        let b = V_ROAD0 + r * V_ROADW;
        let on = r == cur;
        let x = 16.0 + r as f32 * (pw + gap);
        let y = if on { 92.0 } else { 96.0 };
        let (_, a) = enter(t - 2 - r as i32 * 3);
        if a <= 0.0 {
            continue;
        }
        let deg = if on { 0.0 } else { [-1.0, 0.8, -0.6][r] };
        let kind = v[b + RD_KIND];
        let danger = v[b + RD_DANGER];
        if on {
            face(e, S_FACE_CORAL, x - 4.0, y - 4.0, pw + 8.0, 168.0 + 8.0, alpha(WHITE, a));
        }
        sticker(e, S_FACE_WHITE, x, y, pw, 168.0, deg, alpha(WHITE, a), |e| {
            let head = if danger == DANGER_STEEP { CORAL } else if danger == DANGER_ROUGH { SUNNY } else { 0xBFEB_E3FF };
            e.rect(x + 1.5, y + 1.5, pw - 3.0, 40.0, head);
            e.text_ex(x + 12.0, y + 6.0, d(19), if danger == DANGER_STEEP { WHITE } else { INK }, ALIGN_LEFT, pw - 90.0, road_name(kind));
            let dw = match danger { DANGER_STEEP => "steep", DANGER_ROUGH => "rough", _ => "calm" };
            let ww = tw(dw, d(11)) + 14.0;
            face(e, S_FACE_INK, x + pw - ww - 10.0, y + 11.0, ww, 19.0, WHITE);
            e.text_ex(x + pw - ww * 0.5 - 10.0, y + 14.0, d(11), WHITE, ALIGN_CENTER, 0.0, dw);
            // the fights
            let nf = v[b + RD_NF].clamp(0, 3);
            let mut s = Buf::new();
            for f in 0..nf {
                if f > 0 {
                    s.s(", ");
                }
                s.s(fight_word(v[b + RD_FIGHTS + f as usize * RD_FW] - 1));
            }
            e.text_ex(x + 12.0, y + 46.0, bs(11), LILAC, ALIGN_LEFT, 0.0, "Fights");
            e.text_ex(x + 12.0, y + 58.0, d(14), INK, ALIGN_LEFT, pw - 24.0, s.str());
            // the twist
            let tws = v[b + RD_TWISTS];
            let mut tl = Buf::new();
            if kind == ROAD_SHIFT || tws & TW_FAST_SHIFT != 0 {
                tl.s("The stage shifts on foe KOs, ").i(v[b + RD_SHIFT_CAP]).s(" a fight, +").i(v[b + RD_SHIFT_COINS]).s(" coins each.");
            } else if tws & TW_FOE_DRIVE != 0 {
                tl.s("Every foe carries one more drive.");
            } else {
                tl.s("Nothing extra.");
            }
            crate::pick::wrap(e, x + 12.0, y + 76.0, pw - 24.0, bs(12), 13.0, INK, ALIGN_LEFT, tl.str(), 2);
            e.rect(x + 12.0, y + 102.0, pw - 24.0, 2.0, PALE);
            e.text_ex(x + 12.0, y + 108.0, bs(11), LILAC, ALIGN_LEFT, 0.0, "Banks");
            tokens(e, x + 12.0, y + 124.0, pw - 24.0, v[b + RD_PRIZE], v[b + RD_PRIZE_N]);
        });
        if on {
            spr(e, S_PTR, x - 30.0, y + 70.0, 0.0, 0.0, WHITE);
        }
        if v[V_ROAD_FLAG] == r as i32 + 1 {
            // P2's wish: a small lagoon flag on the poster's corner
            let (fx, fy) = (x + pw - 22.0, y - 14.0);
            e.rect(fx, fy, 2.5, 26.0, INK);
            face(e, S_FACE_LAGOON, fx + 2.0, fy, 24.0, 15.0, WHITE);
            e.text_ex(fx + 14.0, fy + 1.0, d(10), WHITE, ALIGN_CENTER, 0.0, "P2");
        }
    }
    // what A does
    let b = V_ROAD0 + cur * V_ROADW;
    let mut s = Buf::new();
    s.s("Take the ").s(road_name(v[b + RD_KIND])).s(": ");
    if v[b + RD_KIND] == ROAD_SHIFT || v[b + RD_TWISTS] & TW_FAST_SHIFT != 0 {
        s.s("the stage shifts on foe KOs, +").i(v[b + RD_SHIFT_COINS]).s(" coins each. ");
    } else if v[b + RD_TWISTS] & TW_FOE_DRIVE != 0 {
        s.s("every foe carries one more drive. ");
    } else {
        s.s("the fights as they are. ");
    }
    s.s("Banks ");
    let (mut picks, mut lv, mut other) = (0, 0, 0);
    for i in 0..v[b + RD_PRIZE_N].clamp(0, 8) {
        match token(v[b + RD_PRIZE], i) {
            RK_PICK => picks += 1,
            RK_LEVEL => lv += 1,
            0 => {}
            _ => other += 1,
        }
    }
    s.i(picks).s(if picks == 1 { " pick" } else { " picks" });
    if lv > 0 {
        s.s(" and ").s(if lv == 1 { "a level-up" } else { "level-ups" });
    }
    if other > 0 {
        s.s(" and more");
    }
    s.s(".");
    let w = (tw(s.str(), bs(12)) + 22.0).min(608.0);
    sticker(e, S_FACE_SUNNY, 16.0, 270.0 - 6.0, w, 26.0, -0.4, WHITE, |e| {
        e.text_ex(27.0, 270.0, bs(12), INK, ALIGN_LEFT, w - 22.0, s.str());
    });
}

/// The tag button chosen at the camp (0 Y, 1 X, 2 D-pad classic); sent with gd_ui_envoy_set_tag_button.
pub static mut TAG_BUTTON: i32 = 0;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_envoy_set_tag_button(b: i32) -> i32;
}
