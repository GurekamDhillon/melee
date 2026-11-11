//! envoy_ui: the Envoy run's presentation (gd-ui-v1), placeholder looks only: plain kit slices and text, no art.
//! It reads the run view the sim mod publishes (`gd_ui_envoy`) and the transition record (`gd_ui_transition`), both
//! read-only, and never writes the sim: the reward pick is made with the synced sim pad inside envoy_sim.
//! * HUD in a fight: fight n/5, the stock pool, tag cooldown, the next stage shift.
//! * reward screen (sim frozen): three cards with before -> after compare rows, the cursor and the 20 s timer.
//! * tag cinematic (transition kind 1, the record's length) and the stage shift's wipe and flash (kind 2).
//! * between fights: the run line; RUN CLEAR / RUN OVER.
//! * the door (envoy-redesign.md 1.2-1.5): the 1-P menu's Envoy row (the host moves the cursor onto it, this mod draws
//!   it), and the Envoy camp scene (kit nodes): Continue / New run / Party / Abandon / Back on the main menu, and after
//!   a fight (the camp replaces the VS CSS while the run is armed) the run's line with Continue / Party / Abandon /
//!   Leave, or RUN CLEAR / RUN OVER. Placeholder look only: kit panels and text.
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

use envoy_data::*;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_transition(out: *mut i32) -> i32;
    fn gd_ui_envoy(out: *mut i32, n: u32) -> i32;
    fn gd_ui_scene_register(s: *const u8, len: u32) -> u32;
    fn gd_ui_scene_replace_when(major: i32, minor: i32, scene: u32, cond: u32) -> i32;
    fn gd_ui_scene_goto(to: u32) -> i32;
    fn gd_ui_retail_scene() -> i32;
    fn gd_ui_envoy_door(scene: u32) -> i32;
    fn gd_ui_envoy_state() -> i32;
    fn gd_ui_envoy_saved(out: *mut i32, n: u32) -> i32;
    fn gd_ui_envoy_arm(op: i32) -> i32;
    fn gd_ui_node_free(n: u32);
    fn gd_ui_focus(n: u32) -> i32;
    fn gd_ui_petey(out: *mut i32, n: u32) -> i32;
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
const ST_HOVER: i32 = 4;
/// The player's Brawl files are missing: Envoy is blocked (full-game plan Q3).
const ST_NO_BRAWL: i32 = 32;
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
}
static mut CAMP: Camp = Camp { age: 0, scene: 0, open: false, cap: 0, root: 0, btn: [0; NB], confirm: false, party: false, after_fight: false };

fn label(n: u32, t: &str) {
    unsafe { gd_ui_node_label(n, t.as_ptr(), t.len() as u32) };
}

unsafe fn saved() -> Option<[i32; REC_WORDS]> {
    let mut r = [0i32; REC_WORDS];
    if gd_ui_envoy_saved(r.as_mut_ptr(), REC_WORDS as u32) == REC_WORDS as i32 { Some(r) } else { None }
}

/// The run record's words (envoy_sim): 32; keystones at 7..10 (id + 1), drives at 14..22 ((id + 1) * 4 + level).
const REC_WORDS: usize = 32;
const REC_KEYS: usize = 7;
const REC_DRIVES: usize = 14;
/// envoy_sim's view (256 words); the layout is documented at envoy_sim `publish`.
const VW: usize = 256;
const OFFER_STOCK: i32 = 1000;
const SLOTS: usize = 4;

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
    let mut v = [0i32; VW];
    let view = gd_ui_envoy(v.as_mut_ptr(), VW as u32) == 0 && v[0] == 0x454E_5659;
    let ended = view && (v[1] == 2 || v[1] == 3);
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
        c.age = 0;
        c.confirm = false;
        c.party = false;
        c.after_fight = gd_ui_retail_scene() >> 8 == GM_VS;
        c.cap = gd_ui_capture(0xF, CAP_EXCLUSIVE);
        camp_build();
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
        if !c.open {
            return;
        }
        if ev == EV_BACK && c.age >= 3 {
            if c.after_fight {
                gd_ui_envoy_arm(ARM_OFF);
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
unsafe fn camp_draw(e: &mut Enc<16384>, vw: f32, vh: f32) {
    let c = &CAMP;
    e.layer(LAYER_SCENE, 0);
    e.rect(0.0, 0.0, vw, vh, 0x0B0B_12FF);
    let cx = vw * 0.5;
    e.text_ex(cx, 90.0, ROLE_TITLE, WHITE, ALIGN_CENTER, 0.0, "ENVOY");
    let mut v = [0i32; VW];
    let view = gd_ui_envoy(v.as_mut_ptr(), VW as u32) == 0 && v[0] == 0x454E_5659;
    let mut l1 = Buf::new();
    let mut l2 = Buf::new();
    let mut col = WHITE;
    if c.after_fight && view && v[1] == 2 {
        l1.s("RUN CLEAR  -  pool left ").i(v[4]);
        l2.s(ck_name(v[6])).s(" + ").s(ck_name(v[7])).s(" won all ").i(v[3]).s(" fights");
        col = GOOD;
    } else if c.after_fight && view && v[1] == 3 {
        l1.s("RUN OVER at fight ").i(v[2] + 1).s("/").i(v[3]);
        l2.s("The shared stock pool ran out");
        col = ACCENT;
    } else if c.after_fight && view && v[1] == 1 {
        l1.s("Next: fight ").i(v[2] + 1).s("/").i(v[3]).s("   pool ").i(v[4]);
        l2.s(ck_name(v[6])).s(" + ").s(ck_name(v[7])).s("  -  Continue, any stage, Start");
    } else if let Some(r) = saved() {
        l1.s("Saved run: fight ").i(r[4] + 1).s("/5   pool ").i(r[5]);
        l2.s(ck_name(r[11])).s(" + ").s(ck_name(r[12]));
    } else {
        l1.s("No run saved");
        l2.s("New run: pick your pair on the character select, any stage, Start");
    }
    e.text_ex(cx, 160.0, ROLE_VALUE, col, ALIGN_CENTER, 0.0, l1.str());
    e.text_ex(cx, 190.0, ROLE_LABEL, DIM, ALIGN_CENTER, vw - 80.0, l2.str());
    // the pair's drives (the live run's, else the saved run's): ids, levels, keystones, the pair, shared
    let mut w = [0i32; 23];
    let have = if c.after_fight && view && v[1] != 0 {
        w[..8].copy_from_slice(&v[22..30]);
        w[8..16].copy_from_slice(&v[77..85]);
        w[16..19].copy_from_slice(&v[93..96]);
        w[19] = v[6];
        w[20] = v[7];
        w[21] = v[18];
        true
    } else if let Some(r) = saved() {
        for k in 0..8 {
            w[k] = r[REC_DRIVES + k] >> 2;
            w[8 + k] = r[REC_DRIVES + k] & 3;
        }
        w[16..19].copy_from_slice(&r[REC_KEYS..REC_KEYS + 3]);
        w[19] = r[11];
        w[20] = r[12];
        w[21] = r[6];
        true
    } else {
        false
    };
    if have {
        let sets = if w[21] != 0 { 1 } else { 2 };
        for f in 0..sets {
            let mut b = Buf::new();
            drive_line(&mut b, if sets == 1 { "PAIR" } else { ck_name(w[19 + f]) }, &w[f * SLOTS..f * SLOTS + SLOTS], &w[8 + f * SLOTS..8 + f * SLOTS + SLOTS]);
            let y = 222.0 + f as f32 * 48.0;
            e.text_ex(cx, y, ROLE_LABEL, DIM, ALIGN_CENTER, vw - 80.0, b.str());
            // the drives as their models, turning
            let n = w[f * SLOTS..f * SLOTS + SLOTS].iter().filter(|&&d| d > 0).count() as f32;
            let mut x = cx - n * 20.0;
            for k in 0..SLOTS {
                let id = w[f * SLOTS + k];
                if id > 0 {
                    sprite(e, POOL[id as usize - 1].family as i32, x, y + 14.0, 32.0);
                    x += 40.0;
                }
            }
        }
        let mut b = Buf::new();
        keys_line(&mut b, &w[16..19]);
        e.text_ex(cx, 330.0, ROLE_LABEL, DIM, ALIGN_CENTER, vw - 80.0, b.str());
    }
    if c.party {
        let (x, y, w) = (cx - 380.0, vh - 190.0, 760.0);
        e.slice(SLICE_FILL_R6, x, y, w, 120.0, PANEL);
        e.text_ex(cx, y + 14.0, ROLE_VALUE, WHITE, ALIGN_CENTER, 0.0, "PARTY");
        e.text_ex(cx, y + 46.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, "P1's pick on the character select is your main; the P2 slot (human or CPU, any character) is your partner.");
        e.text_ex(cx, y + 72.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, "Zelda, Sheik and Ice Climbers cannot tag: they fall back to Fox / Marth. A saved run keeps its pair.");
    }
    e.text_ex(cx, vh - 40.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, "A: choose   B: back");
}

/// The 1-P menu's Envoy row (the cursor itself is the host's: melee_modes::tu::envoy::DOOR_HOVER).
unsafe fn door_draw(e: &mut Enc<16384>, st: i32, vw: f32, vh: f32) {
    let hover = st & ST_HOVER != 0;
    let (x, y, w, h) = (vw * 0.5 - 330.0, vh - 128.0, 300.0, 44.0);
    e.layer(LAYER_OVERLAY, 10);
    e.slice(SLICE_FILL_R6, x, y, w, h, if hover { 0xC83A_3AF0 } else { PANEL });
    if hover {
        e.slice(SLICE_LINE_R6, x, y, w, h, WHITE);
    }
    e.text_ex(x + 18.0, y + 11.0, ROLE_VALUE, WHITE, ALIGN_LEFT, 0.0, "ENVOY");
    e.text_ex(x + w - 14.0, y + 14.0, ROLE_LABEL, if hover { WHITE } else { DIM }, ALIGN_RIGHT, 0.0, if hover { "A: open the camp" } else { "below Training" });
}

static mut ENC: Enc<16384> = Enc::new();

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

struct Buf {
    b: [u8; 160],
    n: usize,
}
impl Buf {
    fn new() -> Self {
        Buf { b: [0; 160], n: 0 }
    }
    fn s(&mut self, s: &str) -> &mut Self {
        for &c in s.as_bytes() {
            if self.n < self.b.len() {
                self.b[self.n] = c;
                self.n += 1;
            }
        }
        self
    }
    fn i(&mut self, v: i32) -> &mut Self {
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
    fn str(&self) -> &str {
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
fn ck_name(k: i32) -> &'static str {
    if (0..26).contains(&k) { CK_NAMES[k as usize] } else { "?" }
}

#[no_mangle]
pub extern "C" fn gd_ui_init() -> i32 {
    unsafe {
        gd_ui_theme(0);
        let n = b"envoy.camp";
        let sc = gd_ui_scene_register(n.as_ptr(), n.len() as u32);
        CAMP.scene = sc;
        if sc != 0 {
            gd_ui_envoy_door(sc);
            // the VS CSS is the camp while a run is armed and has fought (the results land there); retail otherwise
            gd_ui_scene_replace_when(GM_VS, GS_CSS, sc, 1);
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe {
        let st = gd_ui_envoy_state();
        if CAMP.open || st & ST_1P != 0 {
            let mut b = [0f32; 6];
            gd_ui_band_rect(b.as_mut_ptr());
            let sc = (b[5] / 720.0).max(0.25);
            let vw = b[4] / sc;
            let e = &mut *core::ptr::addr_of_mut!(ENC);
            e.clear();
            e.space(SPACE_WINDOW, sc);
            if CAMP.open {
                CAMP.age += 1;
                camp_draw(e, vw, 720.0);
            } else {
                door_draw(e, st, vw, 720.0);
            }
            e.submit();
            return;
        }
        let mut v = [0i32; VW];
        // only an armed run has a HUD or a between-fights line: with the mods loaded and no run armed, VS is retail
        if gd_ui_envoy(v.as_mut_ptr(), VW as u32) != 0 || v[0] != 0x454E_5659 || st & ST_ARMED == 0 {
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
        let in_match = v[5] != 0;
        if in_match {
            world(&v);
            sounds(&v);
            let mut pv = [0i32; 12];
            let petey = gd_ui_petey(pv.as_mut_ptr(), 12) == 0 && pv[0] != 0;
            let mut hv = v;
            if petey {
                hv[76] = -1; // no stage shifts in the Petey fight: no countdown
            }
            hud(e, &hv, vw);
            if petey {
                petey_bars(e, &pv, vw);
            }
            if v[10] != 0 {
                reward(e, &v, vw, vh);
            } else if v[30] >= 0 && v[31] < 150 {
                let mut s = Buf::new();
                if v[30] == OFFER_STOCK {
                    s.s("TOOK +1 STOCK");
                } else {
                    let d = &POOL[v[30].clamp(0, POOL_LEN as i32 - 1) as usize];
                    if d.kind == K_KEYSTONE {
                        s.s("KEYSTONE: ").s(d.name);
                    } else {
                        s.s(if v[18] != 0 { "PAIR" } else { wearer_name(&v, v[39]) }).s(" TAKES ").s(d.name);
                    }
                }
                let a = 1.0 - ease((v[31] as f32 - 110.0) / 40.0);
                e.slice(SLICE_FILL_R6, vw * 0.5 - 160.0, 96.0, 320.0, 44.0, alpha(PANEL, a));
                e.text_ex(vw * 0.5, 104.0, ROLE_VALUE, alpha(GOOD, a), ALIGN_CENTER, 0.0, s.str());
            }
            if t[1] == 1 {
                tag_cinematic(e, &t, vw, vh);
            } else if t[1] == 2 {
                stage_shift(e, &t, vw, vh);
            }
        } else {
            between(e, &v, vw, vh);
        }
        e.submit();
    }
}

fn wearer_name(v: &[i32; VW], w: i32) -> &'static str {
    ck_name(if w == 1 { v[7] } else { v[6] })
}

unsafe fn hud(e: &mut Enc<16384>, v: &[i32; VW], _vw: f32) {
    let (fight, total, pool) = (v[2] + 1, v[3], v[4]);
    let x = 24.0;
    let y = 24.0;
    e.slice(SLICE_FILL_R6, x, y, 300.0, 128.0, PANEL);
    let mut s = Buf::new();
    s.s("ENVOY  FIGHT ").i(fight).s("/").i(total);
    if v[29] != 0 {
        s.s("  BOSS");
    }
    e.text(x + 14.0, y + 8.0, ROLE_LABEL, if v[29] != 0 { ACCENT } else { WHITE }, s.str());
    let mut s = Buf::new();
    s.s("STOCK POOL ").i(pool);
    e.text(x + 14.0, y + 34.0, ROLE_LABEL, WHITE, s.str());
    for i in 0..pool.clamp(0, 12) {
        e.slice(SLICE_FILL_R4, x + 170.0 + i as f32 * 14.0, y + 38.0, 10.0, 10.0, GOOD);
    }
    let mut s = Buf::new();
    if v[34] < 0 {
        s.s("TAG  -");
    } else if v[8] > 0 {
        s.s("TAG  ").i(v[8] / 60).s(".").i((v[8] % 60) / 6).s("s");
    } else {
        s.s("TAG  READY (D-pad down)");
    }
    e.text(x + 14.0, y + 60.0, ROLE_LABEL, if v[8] > 0 { DIM } else { GOOD }, s.str());
    let mut s = Buf::new();
    s.s(ck_name(v[6])).s(" + ").s(ck_name(v[7]));
    e.text(x + 14.0, y + 84.0, ROLE_LABEL, DIM, s.str());
    if v[76] >= 0 {
        let mut s = Buf::new();
        s.s("STAGE SHIFT IN ").i((v[76] + 59) / 60).s("s");
        e.text(x + 14.0, y + 106.0, ROLE_LABEL, DIM, s.str());
    }
}

/// Petey's HP bar (top centre) and the two cage bars under it.
unsafe fn petey_bars(e: &mut Enc<16384>, p: &[i32; 12], vw: f32) {
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
    let bar = |e: &mut Enc<16384>, bx: f32, by: f32, bw: f32, v: i32, m: i32, c: u32| {
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
const LOG_W: usize = 8;
const FL_ECHO: i32 = 1;
const FL_LINK: i32 = 4;
const TAU: f32 = 6.283_185_5;

/// The sound for each presentation key (Brawl wave sounds from the player's own disc; silent without it).
const SFX: [(&str, &str); 11] = [
    ("fire_red", "snd_se_common_fire_m"),
    ("fire_bolt", "snd_se_item_pasaran_spark"),
    ("fire_green", "snd_se_common_dash_start"),
    ("fire_blue", "snd_se_common_JustShield"),
    ("fire_yellow", "snd_se_common_bomb_m"),
    ("fire_purple", "snd_se_item_Freezer_landing"),
    ("fire_shatter", "snd_se_item_Ice_Crash"),
    ("fire_white", "snd_se_item_StarRod_m"),
    ("pickup", "snd_se_item_Item_Get"),
    ("levelup", "snd_se_common_LifeUp"),
    ("keystone", "snd_se_item_Superstar_get"),
];

fn sfx(key: &str) {
    if let Some(&(_, name)) = SFX.iter().find(|s| s.0 == key) {
        unsafe { gd_ui_brawl_sfx(name.as_ptr(), name.len() as u32) };
    }
}

/// Presents since load: the orbit's and the sprites' clock (presentation only).
static mut T: u32 = 0;
static mut TEXS: [u32; 10] = [0; 10];

/// A drive model's turning sprite at top-left (x, y), `size` square.
unsafe fn sprite(e: &mut Enc<16384>, model: i32, x: f32, y: f32, size: f32) {
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
unsafe fn fx(kind: i32, x: f32, y: f32, z: f32, a: f32, b: f32, c: f32, rgba: u32) {
    gd_ui_world_fx(kind, x, y, z, a, b, c, rgba);
}

/// Where orbiter `k` of `n` sits now around (cx, cy).
unsafe fn orbit_at(cx: f32, cy: f32, k: usize, n: usize) -> (f32, f32, f32) {
    let a = T as f32 * 0.045 + k as f32 * TAU / n.max(1) as f32;
    (cx + cos(a) * 8.5, cy + 10.0 + sin(a * 2.0) * 1.2, sin(a) * 8.5)
}

/// The drives in the world: the active fighter's orbiters (dart out on a fire), the keystone halo, the orb, the
/// statuses on foes, the floor drop, and each fire's stand-in effect for its 30 frames.
unsafe fn world(v: &[i32; VW]) {
    T = T.wrapping_add(1);
    let play = v[73];
    let set = if v[18] != 0 { 0 } else { v[38].clamp(0, 1) as usize };
    let have_me = v[98] != 0;
    let (mx, my) = (fv(v, 96), fv(v, 97));
    let log = |e: usize| -> [i32; LOG_W] {
        let mut o = [0; LOG_W];
        o.copy_from_slice(&v[128 + e * LOG_W..128 + (e + 1) * LOG_W]);
        o
    };
    if have_me {
        let n = (0..SLOTS).filter(|&k| v[22 + set * SLOTS + k] > 0).count();
        let mut i = 0;
        for k in 0..SLOTS {
            let id = v[22 + set * SLOTS + k];
            if id <= 0 {
                continue;
            }
            let d = &POOL[id as usize - 1];
            let (mut x, mut y, mut z) = orbit_at(mx, my, i, n);
            i += 1;
            let ch = v[85 + set * SLOTS + k] as f32 / 1000.0;
            let mut glow = 0.1 + 0.6 * ch;
            if ch >= 1.0 {
                glow = 1.0 + 0.4 * sin(T as f32 * 0.3);
            }
            // the latest fire of this drive: dart to the target, flash, come back dim
            for e in 0..8 {
                let l = log(e);
                let age = play - l[0];
                if l[1] == id && l[2] == set as i32 && l[7] & FL_ECHO == 0 && (0..DART + BACK).contains(&age) {
                    let (tx, ty) = if l[3] >= 0 { (l[4] as f32 / 100.0, l[5] as f32 / 100.0 + 8.0) } else { (mx, my + 10.0) };
                    let t = if age < DART { ease(age as f32 / DART as f32) } else { 1.0 - ease((age - DART) as f32 / BACK as f32) };
                    x += (tx - x) * t;
                    y += (ty - y) * t;
                    z *= 1.0 - t;
                    glow = if age < DART + 2 { 1.6 } else { 0.1 };
                }
            }
            let yaw = T as f32 * 0.06 + k as f32;
            model(d.family as i32, x, y, z, yaw, 3.6, glow, 1.0);
            let lv = v[77 + set * SLOTS + k];
            if lv >= 2 {
                model(if lv >= 3 { M_RING3 } else { M_RING2 }, x, y, z, yaw, 3.8, 0.8, 1.0);
            }
        }
        // the keystone halo: one crown per keystone, stacked over the head
        for j in 0..3 {
            if v[93 + j] > 0 {
                model(M_CROWN, mx, my + 21.0 + j as f32 * 2.0, 0.0, T as f32 * 0.03 + j as f32, 7.0 - j as f32, 0.9, 1.0);
            }
        }
        // Bulwark's orb while armed
        if v[114 + set] != 0 {
            model(M_GLASS, mx, my + 8.0, 0.0, T as f32 * 0.02, 17.0, 0.4, 0.3);
            fx(FX_RING, mx, my + 8.0, 0.0, 9.0, 0.8, 0.0, alpha(FAM_COL[2], 0.6));
        }
        // Haste: green spikes at the feet
        if v[112 + v[38].clamp(0, 1) as usize] > 0 {
            fx(FX_SPIKES, mx, my + 1.0, 0.0, 2.0, 7.0, T as f32 * 0.2, alpha(FAM_COL[1], 0.7));
        }
    }
    // foes: Burning (flickering spikes), Chilled (a cold ring)
    for i in 0..3 {
        if v[192 + i] == 0 {
            continue;
        }
        let (qx, qy) = (fv(v, 99 + i * 2), fv(v, 100 + i * 2));
        let st = v[106 + i];
        if st > 0 {
            fx(FX_SPIKES, qx, qy + 8.0, 0.0, 3.0, 6.0 + st as f32 * 2.5 + 1.5 * sin(T as f32 * 0.7), T as f32 * 0.1, alpha(0xFF7A_20FF, 0.55));
        }
        if v[109 + i] > 0 {
            fx(FX_RING, qx, qy + 8.0, 0.0, 9.5, 1.2, 0.0, alpha(0x8CE6_FFFF, 0.7));
        }
    }
    // the floor drop: the model in its glass shell with a beam
    if v[116] == 1 && v[117] > 0 {
        let fam = POOL[v[117] as usize - 1].family as usize;
        let (x, y) = (v[118] as f32 / 100.0, v[119] as f32 / 100.0 + 6.0 + sin(T as f32 * 0.08));
        fx(FX_BEAM, x, y - 6.0, 0.0, 70.0, 7.0, 0.0, alpha(FAM_COL[fam], 0.35));
        model(fam as i32, x, y, 0.0, T as f32 * 0.05, 5.0, 1.0, 1.0);
        model(M_GLASS, x, y, 0.0, T as f32 * 0.05, 7.5, 0.5, 0.3);
    }
    // each fire's effect
    for e in 0..8 {
        let l = log(e);
        let age = play - l[0];
        if l[1] <= 0 || !(0..FIRE_LIFE).contains(&age) {
            continue;
        }
        let d = &POOL[l[1] as usize - 1];
        let p = age as f32 / FIRE_LIFE as f32;
        let k = (1.0 - p) * if l[7] & FL_ECHO != 0 { 0.5 } else { 1.0 };
        let col = alpha(FAM_COL[d.family as usize], k);
        let (tx, ty) = (l[4] as f32 / 100.0, l[5] as f32 / 100.0 + 8.0);
        let big = 1.0 + 0.35 * (l[6] - 1) as f32;
        match d.effect {
            E_BURST => {
                fx(FX_DISC, tx, ty, 0.0, 12.0 * big * ease(p * 3.0), 0.0, 0.0, alpha(0xFFB0_40FF, k));
                fx(FX_SPIKES, tx, ty, 0.0, 2.0, (6.0 + 16.0 * ease(p * 2.0)) * big, 0.3, col);
            }
            E_NOVA => {
                fx(FX_RING, mx, my + 8.0, 0.0, 4.0 + 34.0 * ease(p * 1.5), 3.0 * big, 0.0, col);
                if age < 8 {
                    fx(FX_DISC, mx, my + 8.0, 0.0, 16.0, 0.0, 0.0, alpha(0xFFE0_C0FF, 1.0 - age as f32 / 8.0));
                }
            }
            E_BOLT => {
                if age < 14 {
                    fx(FX_BOLT, tx, ty + 70.0, 0.0, tx, ty, 2.4 * big, alpha(0xFFF6_A0FF, k));
                    fx(FX_BOLT, tx + 2.0, ty + 70.0, 0.0, tx, ty, 0.8, alpha(WHITE, k));
                }
                fx(FX_SPIKES, tx, ty, 0.0, 1.0, 10.0 * big, T as f32 * 0.4, alpha(0xFFF6_A0FF, k));
            }
            E_HASTE => fx(FX_RING, mx, my + 8.0, 0.0, 9.0 + 8.0 * p, 2.0, 0.0, col),
            E_SHIELD => fx(FX_RING, mx, my + 8.0, 0.0, 16.0 - 7.0 * ease(p * 2.0), 2.5, 0.0, col),
            E_SHOCKWAVE => {
                fx(FX_RING, mx, my + 1.0, 0.0, 4.0 + 30.0 * ease(p * 1.5), 3.0 * big, 0.0, col);
                fx(FX_SPIKES, mx, my + 1.0, 0.0, 2.0, 8.0 + 10.0 * p, 0.0, col);
            }
            E_CHILL => fx(FX_SPIKES, tx, ty, 0.0, 3.0, (6.0 + 12.0 * ease(p * 2.0)) * big, 0.4, alpha(0x8CE6_FFFF, k)),
            E_SHATTER => {
                fx(FX_SPIKES, tx, ty, 0.0, 2.0, (8.0 + 18.0 * ease(p * 2.0)) * big, T as f32 * 0.05, alpha(0xE6FA_FFFF, k));
                if l[7] & FL_LINK != 0 && age < 12 && have_me {
                    // the link flash: a short line from the fighter's orbit to the shattered foe
                    fx(FX_LINE, mx, my + 10.0, 0.0, tx, ty, 1.2, alpha(0xC864_FFFF, 1.0 - age as f32 / 12.0));
                }
            }
            E_RELAY => fx(FX_RING, mx, my + 8.0, 0.0, 6.0 + 26.0 * ease(p * 2.0), 2.0, 0.0, col),
            _ => {}
        }
    }
}

/// Sounds keyed on the fire log and the gain record: each (frame, drive, set) plays once, so a rollback that replays
/// a fire does not play it twice.
static mut PLAYED: [i32; 32] = [-1; 32];
static mut PLAYED_N: usize = 0;
static mut GAIN_SEEN: i32 = -1;

unsafe fn sounds(v: &[i32; VW]) {
    let play = v[73];
    let mut n = 0;
    for e in 0..8 {
        let l = &v[128 + e * LOG_W..128 + (e + 1) * LOG_W];
        let age = play - l[0];
        if l[1] <= 0 || !(0..4).contains(&age) {
            continue;
        }
        let key = l[0].wrapping_mul(64) ^ (l[1] * 2 + l[2]) ^ (l[7] << 20);
        if PLAYED.contains(&key) || n >= 2 {
            continue;
        }
        PLAYED[PLAYED_N % 32] = key;
        PLAYED_N += 1;
        n += 1;
        sfx(POOL[l[1] as usize - 1].sound);
    }
    // gains (a drop, a pick, a level-up, a keystone): keyed on (frame, id, count)
    let gkey = v[120].wrapping_mul(64) ^ v[121] ^ (v[125] << 16);
    if v[125] > 0 && gkey != GAIN_SEEN {
        GAIN_SEEN = gkey;
        sfx(if v[124] == 3 { "keystone" } else if v[123] > 1 { "levelup" } else { "pickup" });
    }
}

/// The level fighter `f` has of drive `id` (0: not worn).
fn level_of(v: &[i32; VW], f: usize, id: i32) -> i32 {
    let set = if v[18] != 0 { 0 } else { f };
    (0..SLOTS).find(|&k| v[22 + set * SLOTS + k] == id + 1).map_or(0, |k| v[77 + set * SLOTS + k])
}

/// The reward screen: three drive cards (each model turning, its rule, NEW or LEVEL n per fighter) then the fighter
/// that wears it; or three keystone cards with their price; a `+1 stock` card when the pool is at 1.
unsafe fn reward(e: &mut Enc<16384>, v: &[i32; VW], vw: f32, vh: f32) {
    T = T.wrapping_add(1);
    e.rect(0.0, 0.0, vw, vh, 0x0000_00B0);
    let cx = vw * 0.5;
    let keys = v[10] == 2;
    let mut s = Buf::new();
    if keys {
        s.s("CHOOSE A KEYSTONE");
    } else {
        s.s("FIGHT ").i(v[2]).s(" CLEARED  -  CHOOSE A DRIVE");
    }
    e.text_ex(cx, 120.0, ROLE_TITLE, WHITE, ALIGN_CENTER, vw - 40.0, s.str());
    let secs = (v[12] + 59) / 60;
    let mut s = Buf::new();
    s.i(secs).s("s");
    e.text_ex(cx, 158.0, ROLE_VALUE, if secs <= 5 { ACCENT } else { DIM }, ALIGN_CENTER, 0.0, s.str());
    let gap = 24.0;
    let w = ((vw - 60.0 - 2.0 * gap) / 3.0).min(340.0);
    let h = 380.0;
    let x0 = cx - (3.0 * w + 2.0 * gap) * 0.5;
    let y = 190.0;
    for i in 0..3 {
        let id = v[13 + i];
        let x = x0 + i as f32 * (w + gap);
        let sel = v[11] == i as i32;
        e.slice(SLICE_FILL_R14, x, y, w, h, if sel { 0x1C1C_30F0 } else { 0x0E0E_18E8 });
        if sel {
            e.slice(SLICE_LINE_R14_3, x, y, w, h, ACCENT);
        }
        let tx = x + w * 0.5;
        if id == OFFER_STOCK || !(0..POOL_LEN as i32).contains(&id) {
            e.text_ex(tx, y + 120.0, ROLE_TITLE, if sel { WHITE } else { DIM }, ALIGN_CENTER, w - 20.0, "+1 STOCK");
            e.text_ex(tx, y + 170.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 20.0, "SHARED POOL");
            let mut s = Buf::new();
            s.i(v[19]).s("  ->  ").i(v[19] + 1);
            e.text_ex(tx, y + 200.0, ROLE_VALUE, GOOD, ALIGN_CENTER, w - 20.0, s.str());
            e.text_ex(tx, y + 250.0, ROLE_BODY, DIM, ALIGN_CENTER, w - 30.0, "One more shared stock, now.");
            continue;
        }
        let d = &POOL[id as usize];
        let m = if d.kind == K_KEYSTONE { M_CROWN } else { d.family as i32 };
        sprite(e, m, tx - 48.0, y + 14.0, 96.0);
        e.text_ex(tx, y + 120.0, ROLE_TITLE, if sel { WHITE } else { DIM }, ALIGN_CENTER, w - 20.0, d.name);
        if d.kind == K_KEYSTONE {
            e.text_ex(tx, y + 158.0, ROLE_LABEL, 0xFAC8_46FF, ALIGN_CENTER, w - 20.0, "KEYSTONE");
            e.text_ex(tx, y + 190.0, ROLE_BODY, WHITE, ALIGN_CENTER, w - 30.0, d.rule);
            let mut s = Buf::new();
            s.s("PRICE: ").s(PRICES[d.price as usize]);
            e.text_ex(tx, y + 262.0, ROLE_VALUE, ACCENT, ALIGN_CENTER, w - 20.0, s.str());
            e.text_ex(tx, y + 292.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, PRICE_TEXT[d.price as usize]);
        } else {
            let mut s = Buf::new();
            s.s(FAMILIES[d.family as usize]).s("  -  ").s(FAMILY_MEANING[d.family as usize]);
            e.text_ex(tx, y + 158.0, ROLE_LABEL, FAM_COL[d.family as usize], ALIGN_CENTER, w - 20.0, s.str());
            e.text_ex(tx, y + 190.0, ROLE_BODY, WHITE, ALIGN_CENTER, w - 30.0, d.rule);
            // NEW / LEVEL n for each fighter (or the pair's one set)
            let sets = if v[18] != 0 { 1 } else { 2 };
            for f in 0..sets {
                let lv = level_of(v, f, id);
                let ok = v[127] >> (i * 2 + f) & 1 != 0;
                let mut s = Buf::new();
                s.s(if sets == 1 { "PAIR" } else { ck_name(v[6 + f]) }).s(": ");
                if !ok {
                    s.s(if lv >= 3 { "MAX" } else { "FULL" });
                } else if lv == 0 {
                    s.s("NEW");
                } else {
                    s.s("LEVEL ").i(lv + 1);
                }
                e.text_ex(tx, y + 262.0 + f as f32 * 22.0, ROLE_LABEL, if ok { GOOD } else { DIM }, ALIGN_CENTER, w - 20.0, s.str());
            }
            let word = if d.gives != W_NONE { d.gives } else { d.reads };
            if word != W_NONE {
                e.text_ex(tx, y + 318.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 30.0, GLOSSARY[word as usize]);
            }
        }
        if i == 0 {
            e.text_ex(tx, y + h - 24.0, ROLE_LABEL, 0x6060_70FF, ALIGN_CENTER, 0.0, "(taken on timeout)");
        }
    }
    if v[16] == 1 {
        // step 2: which fighter wears it
        let i = v[11].clamp(0, 2) as usize;
        let (bw, bh) = (300.0, 54.0);
        let by = y + h + 18.0;
        e.rect(0.0, by - 8.0, vw, bh + 16.0, 0x0000_00C0);
        for f in 0..2 {
            let bx = cx - bw - 12.0 + f as f32 * (bw + 24.0);
            let ok = v[127] >> (i * 2 + f) & 1 != 0;
            let on = v[17] == f as i32;
            e.slice(SLICE_FILL_R6, bx, by, bw, bh, if on { 0x2A2A_44F0 } else { PANEL });
            if on {
                e.slice(SLICE_LINE_R6, bx, by, bw, bh, if ok { GOOD } else { DIM });
            }
            let mut s = Buf::new();
            s.s(ck_name(v[6 + f]));
            if !ok {
                s.s(" (FULL)");
            }
            e.text_ex(bx + bw * 0.5, by + 16.0, ROLE_VALUE, if ok { WHITE } else { DIM }, ALIGN_CENTER, bw - 10.0, s.str());
        }
        e.text_ex(cx, by + bh + 14.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, "WHO WEARS IT?   LEFT / RIGHT choose     A wear     B back");
    } else {
        e.text_ex(cx, y + h + 30.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, "LEFT / RIGHT choose     A take");
    }
}

unsafe fn between(e: &mut Enc<16384>, v: &[i32; VW], vw: f32, _vh: f32) {
    let cx = vw * 0.5;
    let (w, h) = (720.0, 92.0);
    let y = 18.0;
    let mut l1 = Buf::new();
    let mut l2 = Buf::new();
    let mut col = WHITE;
    match v[1] {
        2 => {
            l1.s("ENVOY  -  RUN CLEAR!  pool left ").i(v[4]);
            l2.s("The camp has the result");
            col = GOOD;
        }
        3 => {
            l1.s("ENVOY  -  RUN OVER at fight ").i(v[2] + 1).s("/").i(v[3]);
            l2.s("The camp has the result");
            col = ACCENT;
        }
        1 => {
            l1.s("ENVOY  -  NEXT: FIGHT ").i(v[2] + 1).s("/").i(v[3]).s("   pool ").i(v[4]);
            if v[18] != 0 {
                drive_line(&mut l2, "PAIR", &v[22..26], &v[77..81]);
            } else {
                drive_line(&mut l2, ck_name(v[6]), &v[22..26], &v[77..81]);
                l2.s("   ");
                drive_line(&mut l2, ck_name(v[7]), &v[26..30], &v[81..85]);
            }
        }
        _ => {
            l1.s("ENVOY RUN  -  pick your pair, any stage, Start");
            l2.s("P1's pick = your main, P2's slot (any) = your partner");
        }
    }
    e.slice(SLICE_FILL_R6, cx - w * 0.5, y, w, h, PANEL);
    e.text_ex(cx, y + 12.0, ROLE_VALUE, col, ALIGN_CENTER, w - 20.0, l1.str());
    e.text_ex(cx, y + 54.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 20.0, l2.str());
}

unsafe fn tag_cinematic(e: &mut Enc<16384>, t: &[i32; 10], vw: f32, vh: f32) {
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
unsafe fn stage_shift(e: &mut Enc<16384>, t: &[i32; 10], vw: f32, vh: f32) {
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
