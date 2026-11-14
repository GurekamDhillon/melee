//! envoy_ui: the Envoy run's presentation (gd-ui-v1), placeholder looks only: plain kit slices and text, no art.
//! It reads the run view the sim mod publishes (`gd_ui_envoy`) and the transition record (`gd_ui_transition`), both
//! read-only, and never writes the sim: the reward pick is made with the synced sim pad inside envoy_sim.
//! * HUD in a fight: fight n/5 and its kind, the stock pool, each player's pair and tag cooldown, the next stage
//!   shift, and an elite's or boss's drives (named on its card).
//! * the drives in the world (envoy-core-v2.md 3): each player's orbiters (Twin Soul's bench set too), the keystone
//!   halo, orbs, statuses, foes' orbiters, the drops in the owner's colour, each fire's effect and its Brawl sound.
//! * reward screen (sim frozen): three cards per player (co-op: both rows at once) with NEW / LEVEL rows per fighter,
//!   the cursor and the 20 s timer.
//! * tag cinematic (transition kind 1, the record's length) and the stage shift's wipe and flash (kind 2).
//! * between fights: the run line; RUN CLEAR / RUN OVER.
//! * the door (envoy-redesign.md 1.2-1.5): the main menu's Envoy row (the host draws it natively; this mod adds its hover
//!   stickers), and the Envoy camp scene (kit nodes): Continue / New run / Party / Abandon / Back on the main menu, and after
//!   a fight (the camp replaces the VS CSS while the run is armed) the run's line with Continue / Party / Abandon /
//!   Leave, or RUN CLEAR / RUN OVER. Placeholder look only: kit panels and text.
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

mod camp;
mod hud;
mod pick;
mod shop;
mod kit;
mod party;
mod picker;
mod fx;
mod items_ui;

use envoy_data::view::*;
use envoy_data::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_transition(out: *mut i32) -> i32;
    fn gd_ui_stage_live(out: *mut i32) -> i32;
    fn gd_ui_envoy(out: *mut i32, n: u32) -> i32;
    fn gd_ui_scene_register(s: *const u8, len: u32) -> u32;
    fn gd_ui_scene_replace_when(major: i32, minor: i32, scene: u32, cond: u32) -> i32;
    fn gd_ui_scene_goto(to: u32) -> i32;
    fn gd_ui_retail_scene() -> i32;
    fn gd_ui_envoy_door(scene: u32) -> i32;
    fn gd_ui_envoy_state() -> i32;
    fn gd_ui_pad_present(port: u32) -> i32;
    fn gd_ui_envoy_saved(out: *mut i32, n: u32) -> i32;
    fn gd_ui_envoy_best() -> i32;
    fn gd_ui_envoy_arm(op: i32) -> i32;
    fn gd_ui_node_free(n: u32);
    fn gd_ui_focus(n: u32) -> i32;
    fn gd_ui_petey(out: *mut i32, n: u32) -> i32;
    fn gd_ui_fight_hud(out: *mut i32, n: u32) -> i32;
    fn gd_ui_world_model(model: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, glow: f32, alpha: f32) -> i32;
    fn gd_ui_world_fx(kind: i32, x: f32, y: f32, z: f32, a: f32, b: f32, c: f32, rgba: u32) -> i32;
    fn gd_ui_model_tex(model: i32) -> u32;
    fn gd_ui_brawl_sfx(s: *const u8, len: u32) -> i32;
}

const SCENE_RETAIL: u32 = 0x8000_0000;
const GM_VS: i32 = 2;
const GS_CSS: i32 = 8;
const ST_ARMED: i32 = 1;
const ST_1P: i32 = 2;
/// The player's Brawl files are missing: Envoy is blocked (full-game plan Q3).
const ST_NO_BRAWL: i32 = 32;
/// Envoy's select is in the CSS state (the retail select confirms itself under the cover).
const ST_BYPASS: i32 = 128;
const ARM_OFF: i32 = 0;
const ARM_NEW: i32 = 1;
const ARM_CONTINUE: i32 = 2;
const ARM_ABANDON: i32 = 3;
const ARM_LAUNCH: i32 = 0x10;

// the camp's buttons
const B_CONTINUE: usize = 0;
const B_NEW: usize = 1;
const B_PARTY: usize = 2;
const B_ABANDON: usize = 3;
const B_BACK: usize = 4;
const B_DONE: usize = 5;
const NB: usize = 6;

struct Camp {
    scene: u32,
    open: bool,
    cap: u32,
    root: u32,
    btn: [u32; NB],
    /// Abandon was pressed once (asks again).
    confirm: bool,
    party: bool,
    /// On the VS CSS (after a fight) rather than the main menu.
    after_fight: bool,
    /// Presents since the camp opened: the press that opened it must not also choose a row.
    age: u32,
    /// The camp is closing back to the main menu: the capture is held until every button is up, so the press that
    /// closed it (B, or A on Back / Leave) never reaches the retail menu as a fresh press (B there goes to the title).
    closing: bool,
}
static mut CAMP: Camp = Camp { closing: false, age: 0, scene: 0, open: false, cap: 0, root: 0, btn: [0; NB], confirm: false, party: false, after_fight: false };

fn label(n: u32, t: &str) {
    unsafe { gd_ui_node_label(n, t.as_ptr(), t.len() as u32) };
}

/// The best loop any run on this save reached (endless NG+; 0 / 1: none past the first).
pub(crate) unsafe fn best_loop() -> i32 {
    gd_ui_envoy_best()
}

unsafe fn saved() -> Option<[i32; REC_WORDS]> {
    let mut r = [0i32; REC_WORDS];
    if gd_ui_envoy_saved(r.as_mut_ptr(), REC_WORDS as u32) == REC_WORDS as i32 { Some(r) } else { None }
}

/// envoy_sim's view (envoy_data::view, 512 words) and the run record inside it.
pub(crate) const VW: usize = VIEW_WORDS;
/// The view read each present (a static: 6 KiB is too much for the mod's 16 KiB stack).
static mut VBUF: [i32; VW] = [0; VW];

/// "FOX: Ember 2, Frost" for one fighter's drives (`ids`: id + 1, 0 empty; `lv`: levels).
fn drive_line(b: &mut Buf, who: &str, ids: &[i32], lv: &[i32]) {
    b.s(who).s(": ");
    let mut any = false;
    for (k, &c) in ids.iter().enumerate() {
        if (1..=POOL_LEN as i32).contains(&c) {
            if any {
                b.s(", ");
            }
            b.s(POOL[c as usize - 1].name);
            if lv[k] > 1 {
                b.s(" ").i(lv[k]);
            }
            any = true;
        }
    }
    if !any {
        b.s("no drives");
    }
}

fn keys_line(b: &mut Buf, ids: &[i32]) {
    b.s("KEYSTONES: ");
    let mut any = false;
    for &c in ids {
        if (1..=POOL_LEN as i32).contains(&c) {
            if any {
                b.s(", ");
            }
            b.s(POOL[c as usize - 1].name);
            any = true;
        }
    }
    if !any {
        b.s("none");
    }
}

/// A player's block in the view.
fn pb(p: usize) -> usize {
    V_P0 + p * V_PW
}
/// A match slot's block in the view.
fn sb(s: usize) -> usize {
    V_S0 + s * V_SW
}

/// (Re)build the camp's node tree for what is true now.
unsafe fn camp_build() {
    let c = &mut CAMP;
    if c.root != 0 {
        gd_ui_hide(c.root);
        gd_ui_node_free(c.root);
    }
    c.btn = [0; NB];
    let root = gd_ui_node(0, NODE_PANEL, 1);
    label(root, "ENVOY CAMP");
    gd_ui_node_rect(root, 220.0, 165.0, 200.0, 0.0);
    let list = gd_ui_node(root, NODE_LIST, 2);
    let v: &mut [i32; VW] = &mut *core::ptr::addr_of_mut!(VBUF);
    let view = gd_ui_envoy(v.as_mut_ptr(), VW as u32) == 0 && v[V_MAGIC] == VIEW_MAGIC;
    let ended = view && (v[V_STATE] == 2 || v[V_STATE] == 3);
    let has = saved().is_some();
    let no_brawl = gd_ui_envoy_state() & ST_NO_BRAWL != 0;
    if no_brawl {
        // Envoy needs the player's own Brawl disc (Petey, Brawl music); without it the whole mode is closed
        for (k, t) in ["Envoy needs your Brawl disc.", "Add your Brawl disc image", "and restart the game."].iter().enumerate() {
            let n = gd_ui_node(list, NODE_LABEL, 30 + k as u32);
            label(n, t);
        }
    }
    let rows: &[(usize, &str)] = if no_brawl {
        &[(B_BACK, "Back")]
    } else if c.after_fight && ended {
        &[(B_DONE, "Back to the character select"), (B_NEW, "New run")]
    } else if c.after_fight {
        &[(B_CONTINUE, "Continue"), (B_PARTY, "Party"), (B_ABANDON, "Abandon run"), (B_BACK, "Leave (the run is kept)")]
    } else if has {
        &[(B_CONTINUE, "Continue"), (B_NEW, "New run"), (B_PARTY, "Party"), (B_ABANDON, "Abandon run"), (B_BACK, "Back")]
    } else {
        &[(B_NEW, "New run"), (B_PARTY, "Party"), (B_BACK, "Back")]
    };
    for (k, &(b, t)) in rows.iter().enumerate() {
        let n = gd_ui_node(list, NODE_BUTTON, 10 + k as u32);
        label(n, if b == B_ABANDON && c.confirm { "Abandon run? Press A again" } else { t });
        c.btn[b] = n;
    }
    c.root = root;
    gd_ui_show(root, LAYER_MENU, 0);
    if let Some(&(b, _)) = rows.first() {
        gd_ui_focus(c.btn[b]);
    }
}

/// Back to the retail scene under the camp (the main menu, or the VS CSS after a fight).
unsafe fn camp_close() {
    let r = gd_ui_retail_scene() as u32;
    gd_ui_scene_goto(SCENE_RETAIL | (r & 0xFFFF));
}

#[no_mangle]
pub extern "C" fn gd_ui_scene_enter(scene: u32) {
    unsafe {
        let c = &mut CAMP;
        if scene != c.scene {
            return;
        }
        c.open = true;
        c.closing = false;
        c.age = 0;
        c.confirm = false;
        c.party = false;
        c.after_fight = gd_ui_retail_scene() >> 8 == GM_VS;
        c.cap = gd_ui_capture(0xF, CAP_EXCLUSIVE);
        let u = &mut *core::ptr::addr_of_mut!(camp::UI);
        u.age = 0;
        u.cur = 0;
        u.dialog = false;
        u.msg = "";
        if !c.after_fight {
            u.p2 = false;
            camp::coop(0);
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_scene_leave(scene: u32) {
    unsafe {
        let c = &mut CAMP;
        if scene != c.scene {
            return;
        }
        c.open = false;
        if c.root != 0 {
            gd_ui_hide(c.root);
            gd_ui_node_free(c.root);
            c.root = 0;
        }
        if c.cap != 0 {
            gd_ui_release(c.cap);
            c.cap = 0;
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_event(node: u32, ev: i32, _value: i32) {
    unsafe {
        let c = &mut CAMP;
        // V3: the camp reads the pads itself (camp.rs); no host nodes
        if c.open || !c.open {
            let _ = (node, ev);
            return;
        }
        if !c.open {
            return;
        }
        if ev == EV_BACK && c.age >= 3 {
            // after a fight B does nothing: a B mashed through the results must not drop the run (Leave is a row)
            if c.after_fight {
                return;
            }
            return camp_close();
        }
        if ev != EV_ACTIVATE || node == 0 || c.age < 3 {
            return;
        }
        let Some(b) = c.btn.iter().position(|&n| n == node) else { return };
        if b != B_ABANDON {
            c.confirm = false;
        }
        let launch = if c.after_fight { 0 } else { ARM_LAUNCH };
        match b {
            B_CONTINUE => {
                gd_ui_envoy_arm(ARM_CONTINUE | launch);
                camp_close();
            }
            B_NEW => {
                if launch != 0 {
                    picker::send(camp::p2_in());
                }
                gd_ui_envoy_arm(ARM_NEW | launch);
                camp_close();
            }
            B_PARTY => c.party = !c.party,
            B_ABANDON => {
                if !c.confirm {
                    c.confirm = true;
                    camp_build();
                    gd_ui_focus(c.btn[B_ABANDON]);
                } else {
                    c.confirm = false;
                    gd_ui_envoy_arm(ARM_ABANDON);
                    if c.after_fight {
                        camp_close();
                    } else {
                        camp_build();
                    }
                }
            }
            B_BACK | B_DONE => {
                if c.after_fight {
                    gd_ui_envoy_arm(ARM_OFF);
                }
                camp_close();
            }
            _ => {}
        }
    }
}

/// The camp's backdrop and text (the buttons are host nodes).
unsafe fn camp_draw(e: &mut kit::E, vw: f32, vh: f32) {
    let c = &CAMP;
    e.layer(LAYER_SCENE, 0);
    e.rect(0.0, 0.0, vw, vh, 0x0B0B_12FF);
    let cx = vw * 0.5;
    e.text_ex(cx, 90.0, ROLE_TITLE, WHITE, ALIGN_CENTER, 0.0, "ENVOY");
    let v: &mut [i32; VW] = &mut *core::ptr::addr_of_mut!(VBUF);
    let view = gd_ui_envoy(v.as_mut_ptr(), VW as u32) == 0 && v[V_MAGIC] == VIEW_MAGIC;
    let mut l1 = Buf::new();
    let mut l2 = Buf::new();
    let mut col = WHITE;
    let pair = |b: &mut Buf, m: i32, p: i32| {
        b.s(ck_name(m)).s(" + ").s(ck_name(p));
    };
    if c.after_fight && view && v[V_STATE] == 2 {
        l1.s("RUN CLEAR  -  pool left ").i(v[V_POOL]);
        l2.s("won all ").i(v[V_FIGHTS]).s(" fights");
        col = GOOD;
    } else if c.after_fight && view && v[V_STATE] == 3 {
        l1.s("RUN OVER at ");
        if v[V_LOOP] > 1 {
            l1.s("loop ").i(v[V_LOOP]).s(", ");
        }
        l1.s("fight ").i(v[V_FIGHT] + 1).s("/").i(v[V_FIGHTS]);
        l2.s("The shared stock pool ran out");
        col = ACCENT;
    } else if c.after_fight && view && v[V_STATE] == 1 {
        l1.s("Next: ");
        if v[V_LOOP] > 1 {
            l1.s("loop ").i(v[V_LOOP]).s(" ");
        }
        l1.s("fight ").i(v[V_FIGHT] + 1).s("/").i(v[V_FIGHTS]).s("   pool ").i(v[V_POOL]);
        pair(&mut l2, v[pb(0) + P_MAIN], v[pb(0) + P_PARTNER]);
        if v[V_HUMANS] == 2 {
            l2.s("   &   ");
            pair(&mut l2, v[pb(1) + P_MAIN], v[pb(1) + P_PARTNER]);
        }
        l2.s("  -  Continue, any stage, Start");
    } else if let Some(r) = saved() {
        l1.s("Saved run: ");
        if rec_loop(r[R_LOOP]) > 1 {
            l1.s("loop ").i(rec_loop(r[R_LOOP])).s(" ");
        }
        l1.s("fight ").i(r[R_FIGHT] + 1).s("/5   pool ").i(r[R_POOL]);
        if r[R_COOP] != 0 {
            l1.s("   CO-OP");
        }
        pair(&mut l2, r[R_PAIRS], r[R_PAIRS + 1]);
        if r[R_COOP] != 0 {
            l2.s("   &   ");
            pair(&mut l2, r[R_PAIRS + 2], r[R_PAIRS + 3]);
        }
    } else {
        l1.s("No run saved");
        l2.s("New run: pick your pair on the character select, any stage, Start");
    }
    e.text_ex(cx, 160.0, ROLE_VALUE, col, ALIGN_CENTER, 0.0, l1.str());
    e.text_ex(cx, 190.0, ROLE_LABEL, DIM, ALIGN_CENTER, vw - 80.0, l2.str());
    // the pairs' drives (the live run's, else the saved run's): [player][fighter] ids and levels, keystones, pairs
    let mut ids = [[0i32; SLOTS]; 4];
    let mut lvs = [[0i32; SLOTS]; 4];
    let mut keys = [[0i32; KEYSTONES]; 2];
    let mut pairs = [-1i32; 4];
    let (have, players, shared) = if c.after_fight && view && v[V_STATE] != 0 {
        for set in 0..4 {
            let b = pb(set / 2) + (set % 2) * SLOTS;
            ids[set].copy_from_slice(&v[b + P_DRIVES..b + P_DRIVES + SLOTS]);
            lvs[set].copy_from_slice(&v[b + P_LEVELS..b + P_LEVELS + SLOTS]);
        }
        for p in 0..2 {
            keys[p].copy_from_slice(&v[pb(p) + P_KEYS..pb(p) + P_KEYS + KEYSTONES]);
            pairs[p * 2] = v[pb(p) + P_MAIN];
            pairs[p * 2 + 1] = v[pb(p) + P_PARTNER];
        }
        (true, v[V_HUMANS].clamp(1, 2) as usize, v[V_SHARED] != 0)
    } else if let Some(r) = saved() {
        for set in 0..4 {
            for k in 0..SLOTS {
                (ids[set][k], lvs[set][k]) = rec_drive(r[R_DRIVES + set * SLOTS + k], r[1]);
            }
        }
        for p in 0..2 {
            keys[p].copy_from_slice(&r[R_KEYS + p * KEYSTONES..R_KEYS + (p + 1) * KEYSTONES]);
        }
        pairs.copy_from_slice(&r[R_PAIRS..R_PAIRS + 4]);
        (true, if r[R_COOP] != 0 { 2 } else { 1 }, r[R_SHARED] != 0)
    } else {
        (false, 1, false)
    };
    if have {
        let mut y = 222.0;
        for p in 0..players {
            let sets = if shared { 1 } else { 2 };
            for f in 0..sets {
                let set = p * 2 + f;
                let mut b = Buf::new();
                if players == 2 {
                    b.s(if p == 0 { "P1 " } else { "P2 " });
                }
                drive_line(&mut b, if sets == 1 { "PAIR" } else { ck_name(pairs[set]) }, &ids[set], &lvs[set]);
                e.text_ex(cx, y, ROLE_LABEL, if p == 0 { DIM } else { 0x8CB4_FFFF }, ALIGN_CENTER, vw - 80.0, b.str());
                // the drives as their models, turning
                let n = ids[set].iter().filter(|&&d| d > 0).count() as f32;
                let mut x = cx - n * 16.0;
                for k in 0..SLOTS {
                    let id = ids[set][k];
                    if id > 0 {
                        sprite(e, POOL[id as usize - 1].family as i32, x, y + 14.0, 28.0);
                        x += 32.0;
                    }
                }
                y += if players == 2 { 40.0 } else { 48.0 };
            }
            let mut b = Buf::new();
            keys_line(&mut b, &keys[p]);
            e.text_ex(cx, y, ROLE_LABEL, DIM, ALIGN_CENTER, vw - 80.0, b.str());
            y += 30.0;
        }
    }
    if c.party {
        let (x, y, w) = (cx - 380.0, vh - 190.0, 760.0);
        e.slice(SLICE_FILL_R6, x, y, w, 120.0, PANEL);
        e.text_ex(cx, y + 14.0, ROLE_VALUE, WHITE, ALIGN_CENTER, 0.0, "PARTY");
        e.text_ex(cx, y + 46.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, "P1's pick on the character select is your main; the P2 slot (human or CPU, any character) is your partner.");
        e.text_ex(cx, y + 72.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, "Co-op (MELEE_ENVOY_COOP=1): P1 + P3 are player 1's pair, P2 + P4 player 2's. Zelda, Sheik and Ice Climbers fall back to Fox / Marth.");
    }
    e.text_ex(cx, vh - 40.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, "A: choose   B: back");
}

static mut ENC: kit::E = Enc::new();

const REVEAL: i32 = 18;
/// FighterKind order (the transition record's from / to).
const FT_NAMES: [&str; 27] = [
    "MARIO", "FOX", "CAPTAIN FALCON", "DONKEY KONG", "KIRBY", "BOWSER", "LINK", "SHEIK", "NESS", "PEACH", "POPO",
    "NANA", "PIKACHU", "SAMUS", "YOSHI", "JIGGLYPUFF", "MEWTWO", "LUIGI", "MARTH", "ZELDA", "YOUNG LINK", "DR. MARIO",
    "FALCO", "PICHU", "MR. GAME & WATCH", "GANONDORF", "ROY",
];
/// CharacterKind order (the CSS ids: the run's pair).
const CK_NAMES: [&str; 26] = [
    "CAPTAIN FALCON", "DONKEY KONG", "FOX", "MR. GAME & WATCH", "KIRBY", "BOWSER", "LINK", "LUIGI", "MARIO", "MARTH",
    "MEWTWO", "NESS", "PEACH", "PIKACHU", "ICE CLIMBERS", "JIGGLYPUFF", "SAMUS", "YOSHI", "ZELDA", "SHEIK", "FALCO",
    "YOUNG LINK", "DR. MARIO", "ROY", "PICHU", "GANONDORF",
];
/// Gr_Kind -> stage name for the shift card.
fn stage_name(k: i32) -> &'static str {
    match k {
        36 => "BATTLEFIELD",
        10 => "YOSHI'S STORY",
        28 => "DREAM LAND",
        12 => "FOUNTAIN OF DREAMS",
        37 => "FINAL DESTINATION",
        _ => "STAGE",
    }
}

const WHITE: u32 = 0xFFFF_FFFF;
const DIM: u32 = 0x9090_A0FF;
const ACCENT: u32 = 0xE8_3A3A_FF;
const GOOD: u32 = 0x5AD0_7AFF;
const PANEL: u32 = 0x0A0A_14D8;

pub(crate) struct Buf {
    b: [u8; 160],
    n: usize,
}
impl Buf {
    pub(crate) fn new() -> Self {
        Buf { b: [0; 160], n: 0 }
    }
    pub(crate) fn s(&mut self, s: &str) -> &mut Self {
        for &c in s.as_bytes() {
            if self.n < self.b.len() {
                self.b[self.n] = c;
                self.n += 1;
            }
        }
        self
    }
    pub(crate) fn i(&mut self, v: i32) -> &mut Self {
        if v < 0 {
            self.s("-");
        }
        let mut u = v.unsigned_abs();
        let mut t = [0u8; 10];
        let mut k = 0;
        loop {
            t[k] = b'0' + (u % 10) as u8;
            k += 1;
            u /= 10;
            if u == 0 {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            let c = [t[k]];
            self.s(unsafe { core::str::from_utf8_unchecked(&c) });
        }
        self
    }
    pub(crate) fn str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.b[..self.n]) }
    }
}

fn alpha(c: u32, a: f32) -> u32 {
    let a = if a < 0.0 { 0.0 } else if a > 1.0 { 1.0 } else { a };
    (c & 0xFFFF_FF00) | ((((c & 0xFF) as f32) * a) as u32)
}

fn ease(t: f32) -> f32 {
    let t = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    t * t * (3.0 - 2.0 * t)
}

fn ft_name(k: i32) -> &'static str {
    if (0..27).contains(&k) { FT_NAMES[k as usize] } else { "FIGHTER" }
}
pub(crate) fn pad_present(port: u32) -> i32 {
    unsafe { gd_ui_pad_present(port) }
}

pub(crate) fn ck_name(k: i32) -> &'static str {
    if (0..26).contains(&k) { CK_NAMES[k as usize] } else { "?" }
}

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe {
        gd_ui_theme(0);
        kit::load();
        let n = b"envoy.camp";
        let sc = gd_ui_scene_register(n.as_ptr(), n.len() as u32);
        CAMP.scene = sc;
        if sc != 0 {
            gd_ui_envoy_door(sc);
            // the VS CSS is the camp while a run is armed and has fought (the results land there); retail otherwise
            gd_ui_scene_replace_when(GM_VS, GS_CSS, sc, 1);
            // Envoy's own select scene (melee_modes envoy::GS_ENVOY_SEL, GS_COUNT + 2) holds the road the same way
            gd_ui_scene_replace_when(GM_VS, 47, sc, 1);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe {
        let st = gd_ui_envoy_state();
        // Envoy's select: the retail character select loads and confirms itself under this cover (a few frames)
        if st & ST_BYPASS != 0 && !CAMP.open {
            let e = &mut *core::ptr::addr_of_mut!(ENC);
            e.clear();
            e.space(SPACE_BAND, 1.0);
            e.layer(LAYER_MENU, 0);
            e.rect(-200.0, -20.0, 1040.0, 520.0, 0x1A10_4AFF);
            kit::text_edge(e, 320.0, 210.0, kit::d(30), kit::SUNNY, ALIGN_CENTER, 2.2, "Get ready");
            e.submit();
            return;
        }
        if CAMP.open || st & ST_1P != 0 {
            let mut b = [0f32; 6];
            gd_ui_band_rect(b.as_mut_ptr());
            let sc = (b[5] / 720.0).max(0.25);
            let vw = b[4] / sc;
            let e = &mut *core::ptr::addr_of_mut!(ENC);
            e.clear();
            e.space(SPACE_WINDOW, sc);
            if CAMP.open && CAMP.closing {
                if !camp::any_held(0) && !camp::any_held(1) {
                    CAMP.closing = false;
                    camp_close();
                }
            } else if CAMP.open {
                CAMP.age += 1;
                e.space(SPACE_BAND, 1.0);
                let v: &mut [i32; VW] = &mut *core::ptr::addr_of_mut!(VBUF);
                let view = gd_ui_envoy(v.as_mut_ptr(), VW as u32) == 0 && v[V_MAGIC] == VIEW_MAGIC && v[V_STATE] != 0;
                let act = camp::frame(e, CAMP.after_fight, if view { Some(&v) } else { None }, saved(), st & ST_NO_BRAWL != 0);
                if act == camp::DO_CLOSE {
                    CAMP.closing = true;
                } else if act & 0xFF == camp::DO_ARM {
                    let op = act >> 8;
                    let launch = if CAMP.after_fight { 0 } else { ARM_LAUNCH };
                    match op {
                        1 => {
                            if launch != 0 {
                                picker::send(camp::p2_in());
                            }
                            gd_ui_envoy_arm(ARM_NEW | launch);
                        }
                        2 => { gd_ui_envoy_arm(ARM_CONTINUE | launch); }
                        3 => { gd_ui_envoy_arm(ARM_ABANDON); }
                        4 => {
                            gd_ui_envoy_arm(4);
                            if !CAMP.after_fight {
                                // Leave (the run kept): back to the main menu, the A press held back from it too
                                CAMP.closing = true;
                            }
                        }
                        _ => { gd_ui_envoy_arm(ARM_OFF); }
                    }
                    if !CAMP.closing {
                        camp_close();
                    }
                }
            } else {
                e.space(SPACE_BAND, 1.0);
                party::menu(e, st, saved());
            }
            e.submit();
            return;
        }
        let v: &mut [i32; VW] = &mut *core::ptr::addr_of_mut!(VBUF);
        // only an armed run has a HUD or a between-fights line: with the mods loaded and no run armed, VS is retail
        if gd_ui_envoy(v.as_mut_ptr(), VW as u32) != 0 || v[V_MAGIC] != VIEW_MAGIC || st & ST_ARMED == 0 {
            return;
        }
        let mut t = [0i32; 10];
        if gd_ui_transition(t.as_mut_ptr()) != 0 {
            t = [0; 10];
        }
        let mut b = [0f32; 6];
        gd_ui_band_rect(b.as_mut_ptr());
        let sc = (b[5] / 720.0).max(0.25);
        let vw = b[4] / sc;
        let vh = 720.0;
        let e = &mut *core::ptr::addr_of_mut!(ENC);
        e.clear();
        e.layer(LAYER_OVERLAY, 12).space(SPACE_WINDOW, sc);
        let in_match = v[V_LIVE] != 0;
        if in_match {
            world(&v);
            fx::heat_screen(&v);
            fx::moments(&v);
            sounds(&v);
            items_ui::sounds(&v);
            let mut pv = [0i32; 12];
            let petey = gd_ui_petey(pv.as_mut_ptr(), 12) == 0 && pv[0] != 0;
            // V3 HUD in band space (640 x 480; `ex` the extra width each side of the 4:3 band)
            let ex = if b[2] > 0.0 { b[0] / (b[2] / 640.0) } else { 0.0 };
            e.space(SPACE_BAND, 1.0);
            if gd_ui_fight_hud(core::ptr::addr_of_mut!(hud::FH) as *mut i32, 16) != 0 {
                hud::FH = [0; 16];
            }
            hud::draw(e, &v, ex, if petey { Some(&pv) } else { None });
            e.space(SPACE_WINDOW, sc);
            if v[V_RW] == RW_SHOP {
                shop::draw(e, &v, ex, vw, vh, sc);
            } else if v[V_RW] != 0 {
                e.space(SPACE_BAND, 1.0);
                pick::draw(e, &v, ex);
                e.space(SPACE_WINDOW, sc);
            }
            if v[V_RW] != 0 {
                e.space(SPACE_BAND, 1.0);
                pick::confirm(e, &v);
                e.space(SPACE_WINDOW, sc);
            } else if v[V_SINCE_PICK] < 150 {
                e.space(SPACE_BAND, 1.0);
                hud::picked(e, &v);
                e.space(SPACE_WINDOW, sc);
            }
            // the cinematic only for the classic D-pad tag; Quick Tag (Y / X) pops in place (hud::tag_pop)
            if t[1] == 1 && v[V_P0 + P_TAG_BUTTON] == 2 {
                tag_cinematic(e, &t, vw, vh);
            } else if t[1] == 2 {
                stage_shift(e, &t, vw, vh);
            }
            let mut l = [0i32; 8];
            if gd_ui_stage_live(l.as_mut_ptr()) == 0 && l[0] == 1 {
                stage_rift(e, &l, vw, vh);
            }
        } else {
            between(e, &v, vw, vh);
        }
        e.submit();
    }
}

/// The player colours: P1 red, P2 blue (co-op drops, orbit rings, HUD).
const PLAYER_COL: [u32; 2] = [0xFF5A_5AFF, 0x5A9C_FFFF];

fn fighter_name(v: &[i32; VW], p: usize, f: i32) -> &'static str {
    ck_name(if f == 1 { v[pb(p) + P_PARTNER] } else { v[pb(p) + P_MAIN] })
}

/// "TAKES Ember" after a pick, a line per player.
unsafe fn picked_line(e: &mut kit::E, v: &[i32; VW], vw: f32) {
    let a = 1.0 - ease((v[V_SINCE_PICK] as f32 - 110.0) / 40.0);
    for p in 0..v[V_HUMANS].clamp(1, 2) as usize {
        let id = v[pb(p) + P_LAST_PICK];
        if id < 0 {
            continue;
        }
        let mut s = Buf::new();
        if v[V_HUMANS] == 2 {
            s.s(if p == 0 { "P1  " } else { "P2  " });
        }
        if id == OFFER_STOCK {
            s.s("TOOK +1 STOCK");
        } else {
            let d = &POOL[id.clamp(0, POOL_LEN as i32 - 1) as usize];
            if d.kind == K_KEYSTONE {
                s.s("KEYSTONE: ").s(d.name);
            } else {
                s.s(if v[V_SHARED] != 0 { "PAIR" } else { fighter_name(v, p, v[pb(p) + P_LAST_WEARER]) }).s(" TAKES ").s(d.name);
            }
        }
        let y = 96.0 + p as f32 * 50.0;
        e.slice(SLICE_FILL_R6, vw * 0.5 - 180.0, y, 360.0, 44.0, alpha(PANEL, a));
        e.text_ex(vw * 0.5, y + 8.0, ROLE_VALUE, alpha(GOOD, a), ALIGN_CENTER, 0.0, s.str());
    }
}

unsafe fn hud(e: &mut kit::E, v: &[i32; VW], vw: f32) {
    // co-op with one controller: player 2 (port 2) stands idle; say why
    if v[V_HUMANS] == 2 && gd_ui_pad_present(1) == 0 {
        e.slice(SLICE_FILL_R6, vw * 0.5 - 220.0, 150.0, 440.0, 44.0, PANEL);
        e.text_ex(vw * 0.5, 158.0, ROLE_VALUE, PLAYER_COL[1], ALIGN_CENTER, 0.0, "P2: CONNECT A SECOND CONTROLLER");
    }
    let (fight, total, pool) = (v[V_FIGHT] + 1, v[V_FIGHTS], v[V_POOL]);
    let x = 24.0;
    let y = 24.0;
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let h = 70.0 + 88.0 * humans as f32 + if v[V_STAGE_IN] >= 0 { 22.0 } else { 0.0 };
    e.slice(SLICE_FILL_R6, x, y, 320.0, h, PANEL);
    let kind = v[V_FIGHT_KIND];
    let mut s = Buf::new();
    s.s("ENVOY  FIGHT ").i(fight).s("/").i(total);
    s.s(match kind {
        FK_ELITE => "  ELITE",
        FK_BOSS => "  BOSS",
        FK_PACK => "  PACK",
        _ => "",
    });
    e.text(x + 14.0, y + 8.0, ROLE_LABEL, if kind >= FK_ELITE { ACCENT } else { WHITE }, s.str());
    let mut s = Buf::new();
    s.s("STOCK POOL ").i(pool);
    e.text(x + 14.0, y + 34.0, ROLE_LABEL, WHITE, s.str());
    for i in 0..pool.clamp(0, 12) {
        e.slice(SLICE_FILL_R4, x + 170.0 + i as f32 * 12.0, y + 38.0, 9.0, 9.0, GOOD);
    }
    let mut yy = y + 60.0;
    for p in 0..humans {
        let b = pb(p);
        let mut s = Buf::new();
        if humans == 2 {
            s.s(if p == 0 { "P1 " } else { "P2 " });
        }
        s.s(ck_name(v[b + P_MAIN])).s(" + ").s(ck_name(v[b + P_PARTNER]));
        e.text(x + 14.0, yy, ROLE_LABEL, if humans == 2 { PLAYER_COL[p] } else { DIM }, s.str());
        let mut s = Buf::new();
        let cd = v[b + P_TAG_CD];
        if v[b + P_PARTNER_KIND] < 0 {
            s.s("TAG  -");
        } else if cd > 0 {
            s.s("TAG  ").i(cd / 60).s(".").i((cd % 60) / 6).s("s");
        } else {
            s.s("TAG  READY (D-pad down)");
        }
        e.text(x + 14.0, yy + 20.0, ROLE_LABEL, if cd > 0 { DIM } else { GOOD }, s.str());
        // V3 Heat (a text stand-in for the new HUD): the meter, its state, the purse, Volley
        let heat = v[b + P_HEAT];
        let st = v[b + P_HEAT_STATE].clamp(0, 3) as usize;
        const ST_NAME: [&str; 4] = ["COLD", "WARM", "HOT", "BLAZING"];
        const ST_COL: [u32; 4] = [0x8CB4_FFFF, 0xFFD2_3CFF, 0xFF8C_2AFF, 0xFF3A_2AFF];
        let cap = v[b + P_HEAT_CAP].max(1) as f32;
        e.slice(SLICE_FILL_R4, x + 14.0, yy + 44.0, 292.0, 10.0, 0x2020_30FF);
        e.slice(SLICE_FILL_R4, x + 14.0, yy + 44.0, (292.0 * heat as f32 / cap).clamp(2.0, 292.0), 10.0, ST_COL[st]);
        for m in 0..3 {
            e.slice(SLICE_FILL_R4, x + 14.0 + 292.0 * v[b + P_HEAT_MARKS + m] as f32 / cap, yy + 42.0, 2.0, 14.0, WHITE);
        }
        let mut s = Buf::new();
        s.s("HEAT ").i(heat / 1000).s(" ").s(ST_NAME[st]).s("  x").i(v[b + P_HEAT_SCALE] / 1000).s(".").i(v[b + P_HEAT_SCALE] % 1000 / 10);
        if v[b + P_GAIN_MUL] > 1000 {
            s.s("  fill x").i(v[b + P_GAIN_MUL] / 1000).s(".").i(v[b + P_GAIN_MUL] % 1000 / 100);
        }
        e.text(x + 14.0, yy + 58.0, ROLE_LABEL, ST_COL[st], s.str());
        let mut s = Buf::new();
        s.s("COINS ").i(v[V_COINS]).s("  VOLLEY ").i(v[b + P_PIPS]).s("/").i(v[b + P_PIPS_NEED]);
        if v[b + P_ACT_READY] != 0 {
            s.s(" (D-pad right)");
        }
        e.text(x + 160.0, yy + 20.0, ROLE_LABEL, if v[b + P_ACT_READY] != 0 { GOOD } else { DIM }, s.str());
        yy += 88.0;
    }
    if v[V_STAGE_IN] >= 0 {
        let mut s = Buf::new();
        s.s("STAGE SHIFT IN ").i((v[V_STAGE_IN] + 59) / 60).s("s");
        e.text(x + 14.0, yy, ROLE_LABEL, DIM, s.str());
    }
    // the elite's / boss's card: what it carries (Q7: named on the card)
    for sl in 0..4 {
        let b = sb(sl);
        if v[b + S_FOE] == 0 || v[b + S_DRIVES] <= 0 {
            continue;
        }
        let mut s = Buf::new();
        s.s(if kind == FK_BOSS { "BOSS CARRIES  " } else { "ELITE CARRIES  " });
        drive_line(&mut s, "", &v[b + S_DRIVES..b + S_DRIVES + 2], &v[b + S_LEVELS..b + S_LEVELS + 2]);
        let w = 420.0;
        let cx = vw - w - 24.0;
        e.slice(SLICE_FILL_R6, cx, y, w, 40.0, PANEL);
        e.text(cx + 14.0, y + 10.0, ROLE_LABEL, ACCENT, s.str());
        break;
    }
}

/// Petey's HP bar (top centre) and the two cage bars under it.
unsafe fn petey_bars(e: &mut kit::E, p: &[i32; 12], vw: f32) {
    let w = 420.0;
    let x = vw * 0.5 - w * 0.5;
    let y = 24.0;
    e.slice(SLICE_FILL_R6, x - 10.0, y - 6.0, w + 20.0, 92.0, PANEL);
    let mut s = Buf::new();
    s.s(if p[11] != 0 { "PETEY PIRANHA" } else { "PETEY (STAND-IN)" }); // word 11: the real model is drawn
    if p[7] != 0 {
        s.s("  PHASE 2");
    }
    if p[6] == 7 {
        s.s(if p[8] == 2 { "  CAGE BROKEN: PARTNER FREE" } else { "  DEFEATED" });
    } else if p[6] == 6 {
        s.s("  STUNNED");
    }
    e.text(x, y, ROLE_LABEL, if p[7] != 0 { ACCENT } else { WHITE }, s.str());
    let bar = |e: &mut kit::E, bx: f32, by: f32, bw: f32, v: i32, m: i32, c: u32| {
        e.rect(bx, by, bw, 12.0, 0x2020_30FF);
        let f = (v as f32 / m.max(1) as f32).clamp(0.0, 1.0);
        e.rect(bx, by, bw * f, 12.0, c);
    };
    bar(e, x, y + 28.0, w, p[1], p[2], if p[7] != 0 { ACCENT } else { GOOD });
    let cw = w * 0.5 - 8.0;
    bar(e, x, y + 58.0, cw, p[3], p[5], DIM);
    bar(e, x + w - cw, y + 58.0, cw, p[4], p[5], DIM);
    e.text(x, y + 42.0, ROLE_LABEL, DIM, "CAGE");
    e.text_ex(x + w, y + 42.0, ROLE_LABEL, DIM, ALIGN_RIGHT, 0.0, "CAGE");
}

/// Family colours (RGBA) for effects: RED, GREEN, BLUE, YELLOW, PURPLE, WHITE.
const FAM_COL: [u32; 6] = [0xFF5A_2AFF, 0x3CFF_8CFF, 0x5A9C_FFFF, 0xFFD2_3CFF, 0xC864_FFFF, 0xF0F0_FFFF];
/// Model ids (melee_modes::tu::drive_draw): 0-5 the family bodies, 6 the glass shell, 7-9 the ring overlays.
const M_GLASS: i32 = 6;
const M_RING2: i32 = 7;
const M_RING3: i32 = 8;
const M_CROWN: i32 = 9;
/// The Party pack set (DRIVE-MODELS.md, melee_modes::tu::drive_draw): the manifest make_drives.py writes, one key a
/// line, model id = PARTY_FIRST + line. Tier rings and the ascended overlay share the bodies' frame.
const PARTY_KEYS: &str = include_str!("../../../../pc/scripts/examples/envoy_drives/models/drive_party.txt");
const PARTY_FIRST: i32 = 10;
const M_RING_RARE: i32 = 10;
const M_RING_EPIC: i32 = 11;
const M_ASCENDED: i32 = 12;
/// Rock: 12 degrees about the vertical axis, a 2.4 s period (144 presents), each slot a fifth of a period apart.
const ROCK: f32 = 0.209_44;
const ROCK_PERIOD: f32 = 144.0;
/// Each pool record's model (0 = not looked up yet, -1 none: the family body).
static mut MODEL_OF: [i32; POOL_LEN] = [0; POOL_LEN];

/// The drive's own model by key, falling back to its family's legacy body.
fn drive_model(id: i32) -> i32 {
    let i = (id - 1).clamp(0, POOL_LEN as i32 - 1) as usize;
    unsafe {
        if MODEL_OF[i] == 0 {
            MODEL_OF[i] = PARTY_KEYS.lines().position(|k| k == POOL[i].key).map_or(-1, |n| PARTY_FIRST + n as i32);
        }
        if MODEL_OF[i] > 0 { MODEL_OF[i] } else { POOL[i].family as i32 }
    }
}

/// The facing yaw: 0 faces the camera; most drives rock, a few hold still or snap (the spec's orbit column, the
/// cheap ones: Bolt snaps 8 degrees every 20 frames; Spire, Gatekeeper, Meteor, Leech, Affliction, Clean Landing
/// face the camera without rocking).
fn facing(id: i32, slot: usize) -> f32 {
    let key = POOL[(id - 1).clamp(0, POOL_LEN as i32 - 1) as usize].key;
    let t = unsafe { T } as f32;
    match key {
        "spire" | "gatekeeper" | "meteor" | "leech" | "affliction" | "clean_landing" => 0.0,
        "bolt" => if (unsafe { T } / 20) % 2 == 0 { 0.1396 } else { -0.1396 },
        _ => ROCK * sin(TAU * (t / ROCK_PERIOD + slot as f32 * 0.2)),
    }
}

/// Level is size: x1.0, x1.15, x1.3 (levels past 3 keep the level-3 size).
fn level_size(lv: i32) -> f32 {
    match lv { i32::MIN..=1 => 1.0, 2 => 1.15, _ => 1.3 }
}

/// A drive's model with its treatments at one place: the tier ring under (Rare white, Epic foil with the spark), the
/// body, the ascended halo and star over it (level 5 and up). All share the body's frame, yaw and scale.
unsafe fn drive_at(id: i32, lv: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, glow: f32, flags: i32, p: [f32; 4]) {
    let m = drive_model(id);
    let s = scale * level_size(lv);
    // the body: the glow material with the ink edge (drive_draw::is_drive_body; not the rings, overlay or glass)
    fx::model(m, x, y, z, yaw, s, 1.0, fx::MAT_GLOW | fx::MAT_OUTLINE | flags, [glow, p[1], p[2], p[3]], FAM_COL[POOL[(id - 1).clamp(0, POOL_LEN as i32 - 1) as usize].family as usize]);
    drive_rings(id, lv, x, y, z, yaw, scale);
}

/// The treatments over and under a Party body (plain models, no outline).
unsafe fn drive_rings(id: i32, lv: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32) {
    let m = drive_model(id);
    let s = scale * level_size(lv);
    if m >= PARTY_FIRST {
        let tier = POOL[(id - 1).clamp(0, POOL_LEN as i32 - 1) as usize].tier;
        if tier == 1 {
            model(M_RING_RARE, x, y, z, yaw, s, 0.6, 1.0);
        } else if tier >= 2 {
            model(M_RING_EPIC, x, y, z, yaw + T as f32 * TAU / 180.0, s, 0.9, 1.0);
        }
        if lv >= 5 {
            model(M_ASCENDED, x, y, z, yaw, s, 0.9, 1.0);
        }
    }
}
const FX_RING: i32 = 1;
const FX_DISC: i32 = 2;
const FX_SPIKES: i32 = 3;
const FX_BOLT: i32 = 4;
const FX_BEAM: i32 = 5;
const FX_LINE: i32 = 6;
/// One fire lives this many frames on screen (envoy-core-v2.md 3.3); the model's dart out and back.
const FIRE_LIFE: i32 = 30;
const DART: i32 = 6;
const BACK: i32 = 8;
const TAU: f32 = 6.283_185_5;

/// The sound for each presentation key (Brawl wave sounds from the player's own disc; silent without it).
const SFX: [(&str, &str); 16] = [
    ("fire_red", "snd_se_common_fire_m"),
    ("fire_bolt", "snd_se_item_pasaran_spark"),
    ("fire_spire", "snd_se_item_StarRod_l"),
    ("fire_green", "snd_se_common_dash_start"),
    ("fire_blue", "snd_se_common_JustShield"),
    ("fire_heal", "snd_se_item_Heart_get"),
    ("fire_yellow", "snd_se_common_bomb_m"),
    ("fire_air", "snd_se_common_jump_air"),
    ("fire_purple", "snd_se_item_Freezer_landing"),
    ("fire_shatter", "snd_se_item_Ice_Crash"),
    ("fire_white", "snd_se_item_StarRod_m"),
    ("pickup", "snd_se_item_Item_Get"),
    ("levelup", "snd_se_common_LifeUp"),
    ("keystone", "snd_se_item_Superstar_get"),
    ("absorb", "snd_se_common_Guard_on"),
    ("combobreak", "snd_se_item_StarRod_m"),
];

pub(crate) fn sfx(key: &str) {
    if let Some(&(_, name)) = SFX.iter().find(|s| s.0 == key) {
        unsafe { gd_ui_brawl_sfx(name.as_ptr(), name.len() as u32) };
    }
}

/// Presents since load: the orbit's and the sprites' clock (presentation only).
static mut T: u32 = 0;
static mut TEXS: [u32; 10] = [0; 10];

/// A drive model's turning sprite at top-left (x, y), `size` square.
unsafe fn sprite(e: &mut kit::E, model: i32, x: f32, y: f32, size: f32) {
    let m = model.clamp(0, 9) as usize;
    if TEXS[m] == 0 {
        TEXS[m] = gd_ui_model_tex(m as i32);
    }
    if TEXS[m] == 0 {
        return;
    }
    let fr = (T / 5 % 12) as f32;
    e.tex(TEXS[m], x, y, size, size, [fr / 12.0, 0.0, (fr + 1.0) / 12.0, 1.0], WHITE);
}

fn fv(v: &[i32; VW], i: usize) -> f32 {
    f32::from_bits(v[i] as u32)
}

fn sin(x: f32) -> f32 {
    // a small polynomial sine (no_std): x wrapped to [-pi, pi]
    let mut x = x % TAU;
    if x > TAU * 0.5 {
        x -= TAU;
    } else if x < -TAU * 0.5 {
        x += TAU;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}
fn cos(x: f32) -> f32 {
    sin(x + TAU * 0.25)
}

unsafe fn model(m: i32, x: f32, y: f32, z: f32, yaw: f32, scale: f32, glow: f32, a: f32) {
    gd_ui_world_model(m, x, y, z, yaw, scale, glow, a);
}
/// Each drop's last drawn place and its trail (the fly-in).
static mut DROP_LAST: [[(f32, f32); DROPS]; 2] = [[(0.0, 0.0); DROPS]; 2];
static mut DROP_TRAIL: [[[(f32, f32); 8]; DROPS]; 2] = [[[(0.0, 0.0); 8]; DROPS]; 2];

unsafe fn fx(kind: i32, x: f32, y: f32, z: f32, a: f32, b: f32, c: f32, rgba: u32) {
    gd_ui_world_fx(kind, x, y, z, a, b, c, rgba);
}

/// Where orbiter `k` of `n` sits now around (cx, cy), on a ring of radius `r` (each player its own; Twin Soul's
/// bench set a wider one).
unsafe fn orbit_at(cx: f32, cy: f32, k: usize, n: usize, r: f32, phase: f32) -> (f32, f32, f32) {
    let a = T as f32 * 0.045 + phase + k as f32 * TAU / n.max(1) as f32;
    (cx + cos(a) * r, cy + 10.0 + sin(a * 2.0) * 1.2, sin(a) * r)
}

fn log_entry(v: &[i32; VW], e: usize) -> [i32; LOG_W] {
    let mut o = [0; LOG_W];
    o.copy_from_slice(&v[V_LOG + e * LOG_W..V_LOG + (e + 1) * LOG_W]);
    o
}

/// The position a fire's owner draws from: a player's fighter in play, or a foe.
fn owner_pos(v: &[i32; VW], owner: i32) -> Option<(f32, f32)> {
    if (0..OWNER_FOE).contains(&owner) {
        let b = pb(owner as usize / 2);
        if v[b + P_HAVE_POS] != 0 { Some((fv(v, b + P_X), fv(v, b + P_Y))) } else { None }
    } else if (OWNER_FOE..OWNER_FOE + 4).contains(&owner) {
        let b = sb((owner - OWNER_FOE) as usize);
        if v[b + S_ALIVE] != 0 { Some((fv(v, b + S_X), fv(v, b + S_Y))) } else { None }
    } else {
        None
    }
}

/// One orbiting drive: darts to its latest fire's target, flashes, comes back dim; rings for its level.
unsafe fn orbiter(v: &[i32; VW], owner: i32, id: i32, lv: i32, charge: i32, (mut x, mut y, mut z): (f32, f32, f32), cx: f32, cy: f32, scale: f32, dim: f32, slot: usize) {
    let d = &POOL[id as usize - 1];
    let play = v[V_PLAY];
    let ch = charge as f32 / 1000.0;
    let mut glow = 0.1 + 0.6 * ch;
    if ch >= 1.0 || d.charge == C_PASSIVE {
        glow = 1.0 + 0.4 * sin(T as f32 * 0.3);
    }
    for e in 0..LOG {
        let l = log_entry(v, e);
        let age = play - l[0];
        if l[1] == id && l[2] == owner && l[7] & (FL_ECHO | FL_PASSIVE) == 0 && (0..DART + BACK).contains(&age) {
            let (tx, ty) = if l[3] >= 0 { (l[4] as f32 / 100.0, l[5] as f32 / 100.0 + 8.0) } else { (cx, cy + 10.0) };
            let t = if age < DART { ease(age as f32 / DART as f32) } else { 1.0 - ease((age - DART) as f32 / BACK as f32) };
            x += (tx - x) * t;
            y += (ty - y) * t;
            z *= 1.0 - t;
            glow = if age < DART + 2 { 1.6 } else { 0.1 };
        }
    }
    // faces the camera and rocks (DRIVE-MODELS.md 3: these are reliefs 0.4 deep, edge-on they vanish); level is
    // size, tier under, ascension over. envoy-fx.md: the glow material (pulses with the charge, brighter with the
    // wearer's Heat), a light trail, the ink edge on the body
    let heat = if (0..OWNER_FOE).contains(&owner) { fx::heat_state(v, owner as usize / 2) as f32 } else { 0.0 };
    if drive_model(id) >= PARTY_FIRST {
        drive_at(id, lv, x, y, z, facing(id, slot), scale, glow * dim, fx::MAT_TRAIL, [glow * dim, ch.min(1.0), heat, 0.0]);
    } else {
        let yaw = T as f32 * 0.06 + id as f32;
        // Epic foil, Unique (tier 3) holographic foil on the family bodies
        let (mat, extra) = match d.tier { 2 => (fx::MAT_FOIL, 1.0), t if t >= 3 => (fx::MAT_FOIL, 2.0), _ => (fx::MAT_GLOW, 0.0) };
        fx::model(d.family as i32, x, y, z, yaw, scale, 1.0, mat | fx::MAT_TRAIL | fx::MAT_OUTLINE, [glow * dim, ch.min(1.0), heat, extra], FAM_COL[d.family as usize]);
        if lv >= 2 {
            model(if lv >= 3 { M_RING3 } else { M_RING2 }, x, y, z, yaw, scale * 1.05, 0.8 * dim, 1.0);
        }
    }
}

/// The drives in the world: each player's orbiters (their live sets; Twin Soul's bench set on a wider ring), the
/// keystone halo, orbs and Haste; foes' orbiters (elites and bosses) and every slot's statuses; each player's drops
/// in the owner's colour; each fire's stand-in effect for its 30 frames.
unsafe fn world(v: &[i32; VW]) {
    T = T.wrapping_add(1);
    let play = v[V_PLAY];
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    for p in 0..humans {
        let b = pb(p);
        if v[b + P_HAVE_POS] == 0 {
            continue;
        }
        let (mx, my) = (fv(v, b + P_X), fv(v, b + P_Y));
        let act = v[b + P_ACTIVE].clamp(0, 1) as usize;
        let shared = v[V_SHARED] != 0;
        let live = v[b + P_LIVE_SETS];
        for f in 0..2usize {
            let set_f = if shared { 0 } else { f };
            if shared && f == 1 {
                continue;
            }
            if live >> f & 1 == 0 && !(shared && f == 0) {
                continue;
            }
            let bench = !shared && f != act;
            let r = 8.5 + p as f32 * 1.6 + if bench { 4.0 } else { 0.0 };
            let ids = &v[b + P_DRIVES + set_f * SLOTS..b + P_DRIVES + set_f * SLOTS + SLOTS];
            let n = ids.iter().filter(|&&d| d > 0).count();
            let mut i = 0;
            for k in 0..SLOTS {
                let id = ids[k];
                if id <= 0 {
                    continue;
                }
                let at = orbit_at(mx, my, i, n, r, p as f32 * 0.9 + if bench { 0.4 } else { 0.0 });
                i += 1;
                let owner = (p * 2 + set_f) as i32;
                orbiter(v, owner, id, v[b + P_LEVELS + set_f * SLOTS + k], v[b + P_CHARGE + set_f * SLOTS + k], at, mx, my, if bench { 2.8 } else { 3.6 }, if bench { 0.6 } else { 1.0 }, k);
            }
            // Bulwark's orb while armed; Haste: spikes at the feet
            if v[b + P_SHIELD + set_f] != 0 {
                model(M_GLASS, mx, my + 8.0, 0.0, T as f32 * 0.02, 17.0, 0.4, 0.3);
                fx(FX_RING, mx, my + 8.0, 0.0, 9.0, 0.8, 0.0, alpha(FAM_COL[2], 0.6));
            }
            if v[b + P_HASTE + set_f] > 0 && !bench {
                fx(FX_SPIKES, mx, my + 1.0, 0.0, 2.0, 7.0, T as f32 * 0.2, alpha(FAM_COL[1], 0.7));
            }
        }
        // the keystone halo: one crown per keystone, stacked over the head
        for j in 0..KEYSTONES {
            if v[b + P_KEYS + j] > 0 {
                model(M_CROWN, mx, my + 21.0 + j as f32 * 2.0, 0.0, T as f32 * 0.03 + j as f32, 7.0 - j as f32, 0.9, 1.0);
            }
        }
        // co-op: a thin ring in the player's colour under the fighter
        if humans == 2 {
            fx(FX_RING, mx, my + 0.5, 0.0, 6.0, 0.6, 0.0, alpha(PLAYER_COL[p], 0.5));
        }
    }
    // every slot: a foe's orbiters, statuses (Burning flickering spikes, Chilled a cold ring, Marked a violet ring)
    for s in 0..4 {
        let b = sb(s);
        if v[b + S_ALIVE] == 0 {
            continue;
        }
        let (qx, qy) = (fv(v, b + S_X), fv(v, b + S_Y));
        let n = (0..2).filter(|&k| v[b + S_DRIVES + k] > 0).count();
        let mut i = 0;
        for k in 0..2 {
            let id = v[b + S_DRIVES + k];
            if id > 0 {
                let at = orbit_at(qx, qy, i, n, 10.0, 1.7);
                i += 1;
                orbiter(v, OWNER_FOE + s as i32, id, v[b + S_LEVELS + k], v[b + S_CHARGE + k], at, qx, qy, 3.6, 0.9, k);
            }
        }
        if v[b + S_SHIELD] != 0 {
            model(M_GLASS, qx, qy + 8.0, 0.0, T as f32 * 0.02, 17.0, 0.4, 0.3);
        }
        if v[b + S_HASTE] > 0 {
            fx(FX_SPIKES, qx, qy + 1.0, 0.0, 2.0, 7.0, T as f32 * 0.2, alpha(FAM_COL[1], 0.6));
        }
        let st = v[b + S_BURN];
        if st > 0 {
            fx(FX_SPIKES, qx, qy + 8.0, 0.0, 3.0, 6.0 + st as f32 * 2.5 + 1.5 * sin(T as f32 * 0.7), T as f32 * 0.1, alpha(0xFF7A_20FF, 0.55));
        }
        if v[b + S_CHILL] > 0 {
            fx(FX_RING, qx, qy + 8.0, 0.0, 9.5, 1.2, 0.0, alpha(0x8CE6_FFFF, 0.7));
        }
        if v[b + S_MARK] > 0 {
            fx(FX_RING, qx, qy + 16.0, 0.0, 3.0 + 0.4 * sin(T as f32 * 0.25), 1.0, 0.0, alpha(0xC864_FFFF, 0.8));
        }
    }
    // Assist Strike: a pop outlined in the player's colour as the partner jumps in and as it leaves
    for p in 0..humans {
        for kind in [EV_ASSIST, EV_ASSIST_END] {
            let a = hud::tag_age(v, p, kind);
            if (0..8).contains(&a) {
                if let Some((x, y)) = owner_pos(v, (p * 2) as i32) {
                    let k = a as f32 / 8.0;
                    fx(FX_RING, x, y + 8.0, 0.0, 8.0 + 12.0 * k, 2.4, 0.0, alpha(PLAYER_COL[p], 1.0 - k));
                }
            }
        }
    }
    // Quick Tag: a six-frame pop on the fighter coming in
    for p in 0..humans {
        if let Some(k) = hud::tag_pop(v, p) {
            if let Some((x, y)) = owner_pos(v, (p * 2) as i32) {
                fx(FX_RING, x, y + 8.0, 0.0, 6.0 + 10.0 * k, 2.0, 0.0, alpha(0xFFD8_4AFF, 1.0 - k));
                fx(FX_DISC, x, y + 8.0, 0.0, 9.0 * (1.0 - k), 0.0, 0.0, alpha(0xFFFF_FFFF, 0.6 * (1.0 - k)));
            }
        }
    }
    // items: the floor points hold the item room's and the boss's pedestals; every item effect
    if v[V_ITEMS_ON] == 1 {
        items_ui::pedestals(v);
        items_ui::world(v);
    }
    // the drops: the model in its glass shell with a beam in the owner's colour (co-op), else the family's
    let mut drops_seen = [0u32; 2];
    for p in 0..if v[V_ITEMS_ON] == 1 { 0 } else { humans } {
        for k in 0..DROPS {
            let d = pb(p) + P_DROPS + k * 4;
            if v[d] != 1 || v[d + 1] <= 0 {
                DROP_LAST[p][k] = (0.0, 0.0);
                continue;
            }
            let fam = POOL[v[d + 1] as usize - 1].family as usize;
            let (x, y) = (v[d + 2] as f32 / 100.0, v[d + 3] as f32 / 100.0 + 6.0 + sin(T as f32 * 0.08 + k as f32));
            // flying in (it moved since the last frame): a bright sunny trail back along its path, unmissable at
            // the end-of-match zoom
            let prev = DROP_LAST[p][k];
            if prev.0 != 0.0 && ((prev.0 - x).abs() + (prev.1 - y).abs()) > 0.3 {
                let tr = &mut DROP_TRAIL[p][k];
                tr.rotate_right(1);
                tr[0] = (x, y);
                for j in 0..7 {
                    let (ax, ay) = tr[j];
                    let (bx, by) = tr[j + 1];
                    if bx != 0.0 {
                        fx(FX_LINE, ax, ay, 0.0, bx, by, 2.6 - j as f32 * 0.3, alpha(0xFFD8_4AFF, 1.0 - j as f32 / 8.0));
                    }
                }
                fx(FX_DISC, x, y, 0.0, 9.0, 0.0, 0.0, alpha(0xFFF0_A0FF, 0.75));
                fx(FX_RING, x, y, 0.0, 12.0 + 3.0 * sin(T as f32 * 0.5), 1.4, 0.0, alpha(0xFFD8_4AFF, 0.9));
            } else if prev.0 == 0.0 {
                DROP_TRAIL[p][k] = [(0.0, 0.0); 8];
            }
            DROP_LAST[p][k] = (x, y);
            let beam = if humans == 2 { PLAYER_COL[p] } else { FAM_COL[fam] };
            fx(FX_BEAM, x, y - 6.0, 0.0, 70.0, 7.0, 0.0, alpha(beam, 0.35));
            if humans == 2 {
                fx(FX_RING, x, y, 0.0, 8.5, 1.0, 0.0, alpha(PLAYER_COL[p], 0.8));
            }
            // the drive's own model in the badge's place, rocking, tier ring showing; the glass shell only for a drive
            // still drawn as a family body; the fx fly-in / pickup dissolve on either
            let id = v[d + 1];
            let m = drive_model(id);
            if m >= PARTY_FIRST {
                let yaw = facing(id, k);
                fx::drop_model(T, p, k, m, x, y, yaw, false, true);
                drive_rings(id, 1, x, y, 0.0, yaw, 5.0);
            } else {
                fx::drop_model(T, p, k, fam as i32, x, y, T as f32 * 0.05, true, true);
            }
            drops_seen[p] |= 1 << k;
        }
    }
    fx::drop_ghosts(T, drops_seen);
    // each fire's effect, by its drive's look, in its family colour
    for e in 0..LOG {
        let l = log_entry(v, e);
        let age = play - l[0];
        if l[1] <= 0 || !(0..FIRE_LIFE).contains(&age) {
            continue;
        }
        let Some((mx, my)) = owner_pos(v, l[2]) else { continue };
        let d = &POOL[l[1] as usize - 1];
        let p = age as f32 / FIRE_LIFE as f32;
        let k = (1.0 - p) * if l[7] & FL_ECHO != 0 { 0.5 } else { 1.0 };
        let col = alpha(FAM_COL[d.family as usize], k);
        let (tx, ty) = (l[4] as f32 / 100.0, l[5] as f32 / 100.0 + 8.0);
        let big = 1.0 + 0.35 * (l[6] - 1) as f32;
        match d.effect {
            E_BURST | E_WILDFIRE => {
                let r = if d.effect == E_WILDFIRE { 20.0 } else { 12.0 };
                fx(FX_DISC, tx, ty, 0.0, r * big * ease(p * 3.0), 0.0, 0.0, alpha(0xFFB0_40FF, k));
                fx(FX_SPIKES, tx, ty, 0.0, 2.0, (6.0 + 16.0 * ease(p * 2.0)) * big, 0.3, col);
                if d.effect == E_WILDFIRE && age < 12 {
                    fx(FX_LINE, mx, my + 10.0, 0.0, tx, ty, 1.2, alpha(FAM_COL[0], 1.0 - age as f32 / 12.0));
                }
            }
            E_NOVA => {
                fx(FX_RING, mx, my + 8.0, 0.0, 4.0 + 34.0 * ease(p * 1.5), 3.0 * big, 0.0, col);
                if age < 8 {
                    fx(FX_DISC, mx, my + 8.0, 0.0, 16.0, 0.0, 0.0, alpha(0xFFE0_C0FF, 1.0 - age as f32 / 8.0));
                }
            }
            E_SPIRE => {
                fx(FX_BEAM, tx, ty - 10.0, 0.0, 90.0 * ease(p * 3.0), 9.0 * big, 0.0, alpha(0xFFF0_D0FF, k));
                fx(FX_SPIKES, tx, ty - 6.0, 0.0, 2.0, 10.0 * big, 0.0, col);
            }
            E_BOLT => {
                if age < 14 {
                    fx(FX_BOLT, tx, ty + 70.0, 0.0, tx, ty, 2.4 * big, alpha(0xFFF6_A0FF, k));
                    fx(FX_BOLT, tx + 2.0, ty + 70.0, 0.0, tx, ty, 0.8, alpha(WHITE, k));
                }
                fx(FX_SPIKES, tx, ty, 0.0, 1.0, 10.0 * big, T as f32 * 0.4, alpha(0xFFF6_A0FF, k));
            }
            E_FEATHERS => {
                for j in 0..5 {
                    let ox = (j as f32 - 2.0) * 4.0;
                    let fall = 40.0 * (1.0 - p) + j as f32 * 3.0;
                    fx(FX_LINE, tx + ox, ty + fall + 6.0, 0.0, tx + ox + 1.5, ty + fall, 1.4, col);
                }
            }
            E_HASTE | E_HEAL => fx(FX_RING, mx, my + 8.0, 0.0, 9.0 + 8.0 * p, 2.0, 0.0, col),
            E_SHIELD => fx(FX_RING, mx, my + 8.0, 0.0, 16.0 - 7.0 * ease(p * 2.0), 2.5, 0.0, col),
            E_AIRJUMP => fx(FX_RING, mx, my + 2.0, 0.0, 4.0 + 10.0 * p, 1.6, 0.0, col),
            E_SHOCKWAVE => {
                fx(FX_RING, mx, my + 1.0, 0.0, 4.0 + 30.0 * ease(p * 1.5), 3.0 * big, 0.0, col);
                fx(FX_SPIKES, mx, my + 1.0, 0.0, 2.0, 8.0 + 10.0 * p, 0.0, col);
            }
            E_CHILL => fx(FX_SPIKES, tx, ty, 0.0, 3.0, (6.0 + 12.0 * ease(p * 2.0)) * big, 0.4, alpha(0x8CE6_FFFF, k)),
            E_MARK => fx(FX_RING, tx, ty + 8.0, 0.0, 12.0 - 8.0 * ease(p * 2.0), 1.6, 0.0, alpha(0xC864_FFFF, k)),
            E_SHATTER | E_THORNS => {
                fx(FX_SPIKES, tx, ty, 0.0, 2.0, (8.0 + 18.0 * ease(p * 2.0)) * big, T as f32 * 0.05, alpha(0xE6FA_FFFF, k));
                if l[7] & FL_LINK != 0 && age < 12 {
                    // the link flash: a short line from the owner to the struck foe
                    fx(FX_LINE, mx, my + 10.0, 0.0, tx, ty, 1.2, alpha(FAM_COL[d.family as usize], 1.0 - age as f32 / 12.0));
                }
            }
            E_TAILWIND => {
                if age < 10 {
                    fx(FX_LINE, mx, my + 10.0, 0.0, tx, ty, 0.8, alpha(FAM_COL[1], 1.0 - age as f32 / 10.0));
                }
            }
            E_RELAY | E_REPEAT | E_SURGE => fx(FX_RING, mx, my + 8.0, 0.0, 6.0 + 26.0 * ease(p * 2.0), 2.0, 0.0, col),
            _ => {}
        }
    }
}

/// Sounds keyed on the fire log and the gain records: each (frame, drive, owner) plays once, so a rollback that
/// replays a fire does not play it twice.
static mut PLAYED: [i32; 32] = [-1; 32];
static mut PLAYED_N: usize = 0;
static mut GAIN_SEEN: [i32; 2] = [-1; 2];

unsafe fn sounds(v: &[i32; VW]) {
    let play = v[V_PLAY];
    let mut n = 0;
    for e in 0..LOG {
        let l = log_entry(v, e);
        let age = play - l[0];
        if l[1] <= 0 || !(0..4).contains(&age) || l[7] & FL_PASSIVE != 0 {
            continue;
        }
        let key = l[0].wrapping_mul(64) ^ (l[1] * 8 + l[2]) ^ (l[7] << 20);
        if PLAYED.contains(&key) || n >= 2 {
            continue;
        }
        PLAYED[PLAYED_N % 32] = key;
        PLAYED_N += 1;
        n += 1;
        sfx(POOL[l[1] as usize - 1].sound);
    }
    // gains (a drop, a pick, a level-up, a keystone): keyed on each player's gain count
    for p in 0..v[V_HUMANS].clamp(1, 2) as usize {
        let g = pb(p) + P_GAIN;
        let key = v[g + 5];
        if key > 0 && key != GAIN_SEEN[p] {
            GAIN_SEEN[p] = key;
            sfx(if v[g + 4] == 3 { "keystone" } else if v[g + 3] > 1 { "levelup" } else { "pickup" });
        }
    }
}

/// The level fighter `f` of player `p` has of drive `id` (0: not worn).
fn level_of(v: &[i32; VW], p: usize, f: usize, id: i32) -> i32 {
    let set = if v[V_SHARED] != 0 { 0 } else { f };
    let b = pb(p);
    (0..SLOTS).find(|&k| v[b + P_DRIVES + set * SLOTS + k] == id + 1).map_or(0, |k| v[b + P_LEVELS + set * SLOTS + k])
}

/// The reward screen: per player, three drive cards (each model turning, its rule, NEW or LEVEL n per fighter) then
/// the fighter that wears it; or three keystone cards with their price; a `+1 stock` card when the pool is at 1. In
/// co-op the two players pick at once: P1's row on top, P2's below, each with its own cursor.
unsafe fn reward(e: &mut kit::E, v: &[i32; VW], vw: f32, vh: f32) {
    T = T.wrapping_add(1);
    e.rect(0.0, 0.0, vw, vh, 0x0000_00B0);
    let cx = vw * 0.5;
    let keys = v[V_RW] == 2;
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let mut s = Buf::new();
    if keys {
        s.s("CHOOSE A KEYSTONE");
    } else {
        s.s("FIGHT ").i(v[V_FIGHT]).s(" CLEARED  -  CHOOSE A DRIVE");
    }
    e.text_ex(cx, if humans == 2 { 40.0 } else { 120.0 }, ROLE_TITLE, WHITE, ALIGN_CENTER, vw - 40.0, s.str());
    let secs = (v[V_TIMER] + 59) / 60;
    let mut s = Buf::new();
    s.i(secs).s("s");
    e.text_ex(cx, if humans == 2 { 76.0 } else { 158.0 }, ROLE_VALUE, if secs <= 5 { ACCENT } else { DIM }, ALIGN_CENTER, 0.0, s.str());
    let gap = 24.0;
    let w = ((vw - 60.0 - 2.0 * gap) / 3.0).min(340.0);
    let h = if humans == 2 { 262.0 } else { 380.0 };
    let x0 = cx - (3.0 * w + 2.0 * gap) * 0.5;
    for p in 0..humans {
        let b = pb(p);
        let y = if humans == 2 { 104.0 + p as f32 * (h + 40.0) } else { 190.0 };
        let compact = humans == 2;
        if humans == 2 {
            let mut s = Buf::new();
            s.s(if p == 0 { "P1  " } else { "P2  " }).s(ck_name(v[b + P_MAIN])).s(" + ").s(ck_name(v[b + P_PARTNER]));
            if v[b + P_RW_DONE] != 0 {
                s.s("  -  READY");
            }
            e.text(x0, y - 22.0, ROLE_LABEL, PLAYER_COL[p], s.str());
        }
        let done = v[b + P_RW_DONE] != 0;
        for i in 0..3 {
            let id = v[b + P_OFFERS + i];
            let x = x0 + i as f32 * (w + gap);
            let sel = v[b + P_CURSOR] == i as i32 && !done;
            e.slice(SLICE_FILL_R14, x, y, w, h, if sel { 0x1C1C_30F0 } else { 0x0E0E_18E8 });
            if sel {
                e.slice(SLICE_LINE_R14_3, x, y, w, h, if humans == 2 { PLAYER_COL[p] } else { ACCENT });
            }
            let tx = x + w * 0.5;
            let (ty_name, ty_kind, ty_rule, ty_rows) = if compact { (78.0, 106.0, 132.0, 196.0) } else { (120.0, 158.0, 190.0, 262.0) };
            if id == OFFER_STOCK || !(0..POOL_LEN as i32).contains(&id) {
                e.text_ex(tx, y + ty_name, ROLE_TITLE, if sel { WHITE } else { DIM }, ALIGN_CENTER, w - 20.0, "+1 STOCK");
                e.text_ex(tx, y + ty_kind, ROLE_LABEL, DIM, ALIGN_CENTER, w - 20.0, "SHARED POOL");
                let mut s = Buf::new();
                s.i(v[V_POOL]).s("  ->  ").i(v[V_POOL] + 1);
                e.text_ex(tx, y + ty_rule, ROLE_VALUE, GOOD, ALIGN_CENTER, w - 20.0, s.str());
                continue;
            }
            let d = &POOL[id as usize];
            let m = if d.kind == K_KEYSTONE { M_CROWN } else { d.family as i32 };
            let ms = if compact { 64.0 } else { 96.0 };
            sprite(e, m, tx - ms * 0.5, y + 10.0, ms);
            e.text_ex(tx, y + ty_name, ROLE_TITLE, if sel { WHITE } else { DIM }, ALIGN_CENTER, w - 20.0, d.name);
            if d.kind == K_KEYSTONE {
                e.text_ex(tx, y + ty_kind, ROLE_LABEL, 0xFAC8_46FF, ALIGN_CENTER, w - 20.0, "KEYSTONE");
                e.text_ex(tx, y + ty_rule, ROLE_BODY, WHITE, ALIGN_CENTER, w - 30.0, d.rule);
                let mut s = Buf::new();
                s.s("PRICE: ").s(PRICES[d.price as usize]);
                e.text_ex(tx, y + ty_rows, ROLE_VALUE, ACCENT, ALIGN_CENTER, w - 20.0, s.str());
                e.text_ex(tx, y + ty_rows + 30.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, PRICE_TEXT[d.price as usize]);
            } else {
                let mut s = Buf::new();
                s.s(FAMILIES[d.family as usize]).s("  -  ").s(FAMILY_MEANING[d.family as usize]);
                e.text_ex(tx, y + ty_kind, ROLE_LABEL, FAM_COL[d.family as usize], ALIGN_CENTER, w - 20.0, s.str());
                e.text_ex(tx, y + ty_rule, ROLE_BODY, WHITE, ALIGN_CENTER, w - 30.0, d.rule);
                // NEW / LEVEL n for each fighter (or the pair's one set)
                let sets = if v[V_SHARED] != 0 { 1 } else { 2 };
                for f in 0..sets {
                    let lv = level_of(v, p, f, id);
                    let ok = v[b + P_CAN_TAKE] >> (i * 2 + f) & 1 != 0;
                    let mut s = Buf::new();
                    s.s(if sets == 1 { "PAIR" } else { fighter_name(v, p, f as i32) }).s(": ");
                    if !ok {
                        s.s(if lv >= 3 { "MAX" } else { "FULL" });
                    } else if lv == 0 {
                        s.s("NEW");
                    } else {
                        s.s("LEVEL ").i(lv + 1);
                    }
                    e.text_ex(tx, y + ty_rows + f as f32 * 22.0, ROLE_LABEL, if ok { GOOD } else { DIM }, ALIGN_CENTER, w - 20.0, s.str());
                }
                let word = if d.gives != W_NONE { d.gives } else { d.reads };
                if word != W_NONE && !compact {
                    e.text_ex(tx, y + 318.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, GLOSSARY[word as usize]);
                }
            }
            if i == 0 && !compact {
                e.text_ex(tx, y + h - 24.0, ROLE_LABEL, 0x6060_70FF, ALIGN_CENTER, 0.0, "(taken on timeout)");
            }
        }
        if v[b + P_STEP] == 1 && !done {
            // step 2: which fighter wears it
            let i = v[b + P_CURSOR].clamp(0, 2) as usize;
            let (bw, bh) = (300.0, 50.0);
            let by = y + h - bh - 10.0;
            e.rect(x0, by - 6.0, 3.0 * w + 2.0 * gap, bh + 12.0, 0x0000_00D0);
            for f in 0..2 {
                let bx = cx - bw - 12.0 + f as f32 * (bw + 24.0);
                let ok = v[b + P_CAN_TAKE] >> (i * 2 + f) & 1 != 0;
                let on = v[b + P_FCURSOR] == f as i32;
                e.slice(SLICE_FILL_R6, bx, by, bw, bh, if on { 0x2A2A_44F0 } else { PANEL });
                if on {
                    e.slice(SLICE_LINE_R6, bx, by, bw, bh, if ok { GOOD } else { DIM });
                }
                let mut s = Buf::new();
                s.s(fighter_name(v, p, f as i32));
                if !ok {
                    s.s(" (FULL)");
                }
                e.text_ex(bx + bw * 0.5, by + 14.0, ROLE_VALUE, if ok { WHITE } else { DIM }, ALIGN_CENTER, bw - 10.0, s.str());
            }
        }
    }
    for p in 0..humans {
        let b = pb(p);
        if v[b + P_STEP] == 2 && v[b + P_RW_DONE] == 0 {
            let f = v[b + P_FCURSOR].clamp(0, 1) as usize;
            let mut s = Buf::new();
            s.s("ROW FULL - REPLACE: ");
            for k in 0..v[V_OPEN].clamp(0, SLOTS as i32) as usize {
                let id = v[b + P_DRIVES + f * SLOTS + k];
                let on = v[b + P_SCURSOR] == k as i32;
                s.s(if on { "[" } else { " " }).s(if id > 0 { POOL[id as usize - 1].name } else { "-" }).s(if on { "]" } else { " " });
            }
            e.text_ex(cx, vh - 70.0 - p as f32 * 30.0, ROLE_VALUE, ACCENT, ALIGN_CENTER, vw - 40.0, s.str());
        }
    }
    let hint = if (0..humans).any(|p| v[pb(p) + P_STEP] == 2) {
        "REPLACE WHICH?   LEFT / RIGHT choose     A replace     B back"
    } else if (0..humans).any(|p| v[pb(p) + P_STEP] == 1) {
        "WHO WEARS IT?   LEFT / RIGHT choose     A wear     B back"
    } else {
        "LEFT / RIGHT choose     A take"
    };
    e.text_ex(cx, vh - 30.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, hint);
}

/// V3's shop as plain text (a stand-in until the new UI lands): the rows, the cursor, the coins, the last result.
pub(crate) unsafe fn shop_text(e: &mut kit::E, v: &[i32; VW], vw: f32, vh: f32) {
    e.rect(0.0, 0.0, vw, vh, 0x0000_00C0);
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let mut s = Buf::new();
    s.s("SHOP   COINS ").i(v[V_COINS]).s("   ").i((v[V_TIMER] + 59) / 60).s("s");
    e.text_ex(vw * 0.5, 60.0, ROLE_TITLE, WHITE, ALIGN_CENTER, 0.0, s.str());
    for p in 0..humans {
        let b = pb(p);
        let x = if humans == 2 { 40.0 + p as f32 * vw * 0.5 } else { vw * 0.5 - 300.0 };
        let mut y = 120.0;
        let f = v[b + P_SHOP_FIGHTER].clamp(0, 1) as usize;
        let mut s = Buf::new();
        s.s(if p == 0 { "P1 " } else { "P2 " }).s("ROW OF ").s(fighter_name(v, p, f as i32)).s(":");
        for k in 0..v[V_OPEN].clamp(0, SLOTS as i32) as usize {
            let id = v[b + P_DRIVES + f * SLOTS + k];
            let on = v[b + P_SHOP_STEP] >= 1 && v[b + P_SCURSOR] == k as i32;
            s.s(if on { " [" } else { "  " }).s(if id > 0 { POOL[id as usize - 1].name } else { "-" });
            if id > 0 {
                s.s(" ").i(v[b + P_LEVELS + f * SLOTS + k]);
            }
            s.s(if on { "]" } else { "" });
        }
        e.text_ex(x, y, ROLE_LABEL, WHITE, ALIGN_LEFT, vw * 0.5 - 40.0, s.str());
        y += 34.0;
        if v[b + P_SHOP_DONE] != 0 {
            e.text(x, y, ROLE_VALUE, DIM, "DONE");
            continue;
        }
        for row in 0..SHOP_ROWS {
            let mut s = Buf::new();
            let on = v[b + P_SHOP_CURSOR] == row;
            s.s(if on { "> " } else { "  " });
            match row {
                0..=2 => {
                    let id = v[b + P_SHOP_OFFERS + row as usize];
                    if id >= 0 {
                        let d = &POOL[id as usize];
                        s.s(d.name).s("  ").i(v[b + P_SHOP_PRICE + row as usize]).s("c  ").s(TIERS[d.tier.min(2) as usize]);
                    } else {
                        s.s("(sold)");
                    }
                }
                SHOP_LEVEL => {
                    s.s("Level a drive  60c / 120c");
                }
                SHOP_STOCK => {
                    s.s("+1 stock  100c");
                }
                SHOP_REROLL => {
                    s.s("Reroll  ").i(v[V_REROLL_COST]).s("c");
                }
                SHOP_REORDER => {
                    s.s("Reorder the row (free)");
                }
                _ => {
                    s.s("Leave");
                }
            }
            e.text(x, y, ROLE_LABEL, if on { GOOD } else { WHITE }, s.str());
            if on && (0..3).contains(&row) {
                let id = v[b + P_SHOP_OFFERS + row as usize];
                if id >= 0 {
                    e.text_ex(x + 20.0, y + 20.0, ROLE_LABEL, DIM, ALIGN_LEFT, vw * 0.5 - 60.0, POOL[id as usize].rule);
                    y += 20.0;
                }
            }
            y += 26.0;
        }
        const MSG: [&str; 10] = ["", "BOUGHT", "NOT ENOUGH COINS", "LEVELLED UP", "STOCK RESTORED", "REROLLED", "MOVED", "ROW FULL: CHOOSE A SLOT TO REPLACE", "REPLACED", "CAN'T"];
        e.text(x, y + 10.0, ROLE_VALUE, ACCENT, MSG[v[b + P_SHOP_MSG].clamp(0, 9) as usize]);
    }
    let hint = match v[pb(0) + P_SHOP_STEP] {
        1 => "LEFT / RIGHT slot     A confirm     B back",
        2 => "LEFT / RIGHT pick a drive     A lift     B back",
        3 => "LEFT / RIGHT move it     A / B put it down",
        _ => "UP / DOWN choose     A buy     X / Y switch fighter",
    };
    e.text_ex(vw * 0.5, vh - 30.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, hint);
}

unsafe fn between(e: &mut kit::E, v: &[i32; VW], vw: f32, _vh: f32) {
    let cx = vw * 0.5;
    let humans = v[V_HUMANS].clamp(1, 2) as usize;
    let (w, h) = (760.0, 62.0 + 34.0 * humans as f32);
    let y = 18.0;
    let mut l1 = Buf::new();
    let mut col = WHITE;
    match v[V_STATE] {
        2 => {
            l1.s("ENVOY  -  RUN CLEAR!  pool left ").i(v[V_POOL]);
            col = GOOD;
        }
        3 => {
            l1.s("ENVOY  -  RUN OVER at fight ").i(v[V_FIGHT] + 1).s("/").i(v[V_FIGHTS]);
            col = ACCENT;
        }
        1 => {
            l1.s("ENVOY  -  NEXT: FIGHT ").i(v[V_FIGHT] + 1).s("/").i(v[V_FIGHTS]).s("   pool ").i(v[V_POOL]);
        }
        _ => {
            l1.s("ENVOY RUN  -  pick your pair, any stage, Start");
        }
    }
    e.slice(SLICE_FILL_R6, cx - w * 0.5, y, w, h, PANEL);
    e.text_ex(cx, y + 12.0, ROLE_VALUE, col, ALIGN_CENTER, w - 20.0, l1.str());
    for p in 0..humans {
        let mut l2 = Buf::new();
        let b = pb(p);
        if v[V_STATE] == 1 {
            if humans == 2 {
                l2.s(if p == 0 { "P1  " } else { "P2  " });
            }
            if v[V_SHARED] != 0 {
                drive_line(&mut l2, "PAIR", &v[b + P_DRIVES..b + P_DRIVES + 4], &v[b + P_LEVELS..b + P_LEVELS + 4]);
            } else {
                drive_line(&mut l2, ck_name(v[b + P_MAIN]), &v[b + P_DRIVES..b + P_DRIVES + 4], &v[b + P_LEVELS..b + P_LEVELS + 4]);
                l2.s("   ");
                drive_line(&mut l2, ck_name(v[b + P_PARTNER]), &v[b + P_DRIVES + 4..b + P_DRIVES + 8], &v[b + P_LEVELS + 4..b + P_LEVELS + 8]);
            }
        } else if v[V_STATE] == 0 {
            l2.s("P1's pick = your main, P2's slot (any) = your partner");
        } else {
            l2.s("The camp has the result");
        }
        e.text_ex(cx, y + 48.0 + p as f32 * 30.0, ROLE_LABEL, if humans == 2 { PLAYER_COL[p] } else { DIM }, ALIGN_CENTER, w - 20.0, l2.str());
        if v[V_STATE] != 1 {
            break;
        }
    }
}

unsafe fn tag_cinematic(e: &mut kit::E, t: &[i32; 10], vw: f32, vh: f32) {
    let (phase, slot, frame, len, since, from, to, skipped) = (t[0], t[2], t[3] as f32, t[4].max(2) as f32, t[5], t[7], t[8], t[9] != 0);
    let frozen = phase == 1;
    if !frozen && since >= REVEAL {
        return;
    }
    let port = ACCENT;
    let _ = slot;
    let k = len / 66.0;
    if frozen {
        let bar = 84.0 * ease(frame / (18.0 * k));
        e.rect(0.0, 0.0, vw, bar, 0x0000_00E6).rect(0.0, vh - bar, vw, bar, 0x0000_00E6);
        if frame < 10.0 {
            e.rect(0.0, 0.0, vw, vh, alpha(WHITE, 0.85 * (1.0 - frame / 10.0)));
        }
        let (a0, a1) = (8.0 * k, 48.0 * k);
        if frame >= a0 && frame < a1 + 8.0 * k {
            let a = ease((frame - a0) / (8.0 * k)) * (1.0 - ease((frame - a1) / (8.0 * k)));
            let y = vh * 0.5 - 40.0;
            let cx = vw * 0.5;
            e.rect_shear(cx - 380.0, y - 8.0, 760.0, 96.0, alpha(0x0A0A_14FF, 0.82 * a), -0.25);
            e.rect(cx - 380.0, y - 8.0, 8.0, 96.0, alpha(port, a));
            e.text(cx - 360.0, y + 2.0, ROLE_LABEL, alpha(port, a), "TAG");
            e.text_ex(cx - 200.0, y + 24.0, ROLE_TITLE, alpha(0xB8B8_C8FF, a), ALIGN_CENTER, 0.0, ft_name(from));
            e.text_ex(cx, y + 24.0, ROLE_TITLE, alpha(port, a), ALIGN_CENTER, 0.0, ">>");
            let in_t = ease((frame - 20.0 * k) / (12.0 * k));
            e.text_ex(cx + 200.0 + (1.0 - in_t) * 260.0, y + 24.0, ROLE_TITLE, alpha(WHITE, a * in_t), ALIGN_CENTER, 0.0, ft_name(to));
            e.text_ex(cx, y + 70.0, ROLE_LABEL, alpha(DIM, a), ALIGN_CENTER, 0.0, "TAG IN  -  D-pad down to skip");
        }
        let w0 = len * 0.7;
        if frame >= w0 {
            let p = ease((frame - w0) / (len - 1.0 - w0));
            let w = (vw + 400.0) * p;
            e.rect_shear(-200.0, 0.0, w, vh, port, -0.35);
            e.rect_shear(-200.0, 0.0, w - 40.0, vh, 0x0A0A_14FF, -0.35);
        }
    } else if !skipped || since < REVEAL {
        let p = ease(since as f32 / REVEAL as f32);
        let x = -200.0 + (vw + 400.0) * p;
        e.rect_shear(x, 0.0, vw + 400.0, vh, port, -0.35);
        e.rect_shear(x + 40.0, 0.0, vw + 400.0, vh, 0x0A0A_14FF, -0.35);
        let bar = 84.0 * (1.0 - p);
        e.rect(0.0, 0.0, vw, bar, 0x0000_00E6).rect(0.0, vh - bar, vw, bar, 0x0000_00E6);
    }
}

/// The stage shift: a wipe closes over the first half, a white flash at the switch (the midpoint), the wipe opens
/// over the second half; the destination's name rides the cover.
unsafe fn stage_shift(e: &mut kit::E, t: &[i32; 10], vw: f32, vh: f32) {
    let (phase, frame, len, since, to) = (t[0], t[3] as f32, t[4].max(2) as f32, t[5], t[8]);
    if phase != 1 {
        if since < 10 {
            e.rect(0.0, 0.0, vw, vh, alpha(WHITE, 0.6 * (1.0 - since as f32 / 10.0)));
        }
        return;
    }
    let mid = len * 0.5;
    let cover = if frame < mid { ease(frame / (mid - 6.0)) } else { 1.0 - ease((frame - mid - 6.0) / (len - mid - 8.0)) };
    let w = (vw + 400.0) * cover;
    if frame < mid {
        e.rect_shear(-200.0, 0.0, w, vh, 0x1020_40FF, -0.3);
        e.rect_shear(-200.0, 0.0, w - 30.0, vh, 0x0A0A_14FF, -0.3);
    } else {
        e.rect_shear(vw + 200.0 - w, 0.0, w, vh, 0x1020_40FF, -0.3);
        e.rect_shear(vw + 230.0 - w, 0.0, w - 30.0, vh, 0x0A0A_14FF, -0.3);
    }
    let d = frame - mid;
    if (-4.0..14.0).contains(&d) {
        let a = if d < 0.0 { (d + 4.0) / 4.0 } else { 1.0 - d / 14.0 };
        e.rect(0.0, 0.0, vw, vh, alpha(WHITE, 0.9 * a));
    }
    if cover > 0.6 {
        let a = (cover - 0.6) / 0.4;
        e.text_ex(vw * 0.5, vh * 0.5 - 50.0, ROLE_LABEL, alpha(DIM, a), ALIGN_CENTER, 0.0, "STAGE SHIFT");
        e.text_ex(vw * 0.5, vh * 0.5 - 20.0, ROLE_TITLE, alpha(WHITE, a), ALIGN_CENTER, 0.0, stage_name(to));
    }
}

/// A stage's signature colour (the rift's glow and the tint on its side of the sweep).
fn stage_col(k: i32) -> u32 {
    match k {
        36 => 0x3FD0_C8FF,
        10 => 0x9BE1_5DFF,
        28 => 0x7EC8_FFFF,
        12 => 0xB58C_FFFF,
        37 => 0xC04B_FFFF,
        _ => 0xFFFF_FFFF,
    }
}

/// The live stage transition (Fights 2, envoy-items-design.md section 10): the fight never stops. A rift (a bright
/// sheared band in the destination's colour) sweeps across the screen from left to right over the whole record;
/// the side it has passed is veiled in the destination's colour before the hand-off and the side still ahead in the
/// old stage's colour after it, so the screen reads as one stage giving way to the other. At the hand-off (record
/// frame `at`, the rift's core over the middle) the core flares wide for a few frames with a soft flash and a ring,
/// covering the one-frame swap. Tints stay at or under 22% and clear of the centre's fighters' silhouettes (low alpha,
/// no blur), the flare is one flash. Reads only the engine record; never writes back.
unsafe fn stage_rift(e: &mut kit::E, l: &[i32; 8], vw: f32, vh: f32) {
    let (done, from, to, frame, len, at) = (l[1], l[2], l[3], l[4] as f32, l[5].max(2) as f32, l[6].max(1) as f32);
    // the rift's x: from off the left edge to the middle at the hand-off, then on off the right edge
    let x = if frame < at {
        -0.25 * vw + 0.75 * vw * ease(frame / at)
    } else {
        0.5 * vw + 0.75 * vw * ease((frame - at) / (len - at).max(1.0))
    };
    let shear = -0.22;
    let (cf, ct) = (stage_col(from), stage_col(to));
    let fade_in = (frame / 20.0).min(1.0);
    let fade_out = ((len - frame) / 20.0).clamp(0.0, 1.0);
    if done == 0 {
        // before the hand-off: the passed side carries the destination's veil
        e.rect_shear(-400.0, 0.0, x + 400.0, vh, alpha(ct, 0.20 * fade_in), shear);
    } else if done == 1 {
        // after: the side still ahead holds the old stage's veil, thinning as the rift runs out
        e.rect_shear(x, 0.0, vw + 400.0 - x, vh, alpha(cf, 0.18 * fade_out), shear);
    }
    // the rift: a wide soft glow, a narrower glow, the white core
    let pulse = 0.85 + 0.15 * kit_sin(frame * 0.35);
    let near = 1.0 - ((frame - at).abs() / 10.0).min(1.0);
    let core = 10.0 + 120.0 * near * near;
    let a = fade_in.min(fade_out.max(if done == 0 { 1.0 } else { 0.0 }));
    e.rect_shear(x - 110.0, 0.0, 220.0, vh, alpha(ct, 0.16 * a), shear);
    e.rect_shear(x - 44.0, 0.0, 88.0, vh, alpha(ct, 0.38 * a * pulse), shear);
    e.rect_shear(x - core * 0.5, 0.0, core, vh, alpha(WHITE, (0.75 + 0.2 * near) * a), shear);
    // sparks riding the edge (deterministic from the record frame: presentation only)
    for i in 0..9 {
        let fi = i as f32;
        let y = (fi * 83.0 + frame * (5.0 + fi)) % vh;
        let dx = kit_sin(frame * 0.2 + fi) * 18.0 + shear * -y;
        e.rect(x + dx - 3.0 + shear * y, y, 6.0, 6.0, alpha(if i % 2 == 0 { WHITE } else { ct }, 0.8 * a));
    }
    if (frame - at).abs() < 1.0 && done == 1 {
        // the hand-off frame: one soft flash and a ring from the middle (fx post; at most one flash)
        fx::handoff(vw, vh, ct);
    }
    if (frame - at).abs() < 8.0 {
        e.rect(0.0, 0.0, vw, vh, alpha(WHITE, 0.35 * near));
    }
    // the caption: where the rift leads
    let ca = fade_in * fade_out * 0.9;
    e.text_ex(vw * 0.5, 92.0, ROLE_LABEL, alpha(DIM, ca), ALIGN_CENTER, 0.0, if done == 0 { "SHIFTING TO" } else { "NOW ON" });
    e.text_ex(vw * 0.5, 118.0, ROLE_TITLE, alpha(ct | 0xFF, ca), ALIGN_CENTER, 0.0, stage_name(to));
}

/// A cheap sine for presentation (no libm in the mod).
fn kit_sin(x: f32) -> f32 {
    let q = x / 6.283_185;
    let t = q - (q as i32 as f32) + if q < 0.0 { 1.0 } else { 0.0 };
    let u = t * 2.0 - 1.0;
    // a parabola through the sine's shape (-1..1), close enough for a shimmer
    let y = 4.0 * u * (1.0 - u.abs());
    -y
}

#[repr(C)]
pub struct Manifest {
    magic: u32, struct_size: u32, abi_min: u32, abi_max: u32, hook_table_version: u32, mod_version: u32,
    required_features: u32, optional_features: u32, state_bytes: u32, hook_count: u32, hooks: *const u8,
    mod_id: [u8; 32], mod_name: [u8; 48],
}
unsafe impl Sync for Manifest {}

const fn text<const N: usize>(s: &[u8]) -> [u8; N] {
    let mut o = [0u8; N];
    let mut i = 0;
    while i < s.len() {
        o[i] = s[i];
        i += 1;
    }
    o
}

#[no_mangle]
pub static gdmod_abi_version: u32 = 1 << 16;
#[no_mangle]
pub static gdmod_ui_abi: u32 = UI_ABI;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_MENU | FEATURE_UI_INPUT | FEATURE_UI_SCENE, optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.envoy_ui"), mod_name: text(b"Envoy run (presentation)"),
};
