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
}

const SCENE_RETAIL: u32 = 0x8000_0000;
const GM_VS: i32 = 2;
const GS_CSS: i32 = 8;
const ST_ARMED: i32 = 1;
const ST_1P: i32 = 2;
const ST_HOVER: i32 = 4;
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

unsafe fn saved() -> Option<[i32; 16]> {
    let mut r = [0i32; 16];
    if gd_ui_envoy_saved(r.as_mut_ptr(), 16) == 16 { Some(r) } else { None }
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
    let mut v = [0i32; 64];
    let view = gd_ui_envoy(v.as_mut_ptr(), 64) == 0 && v[0] == 0x454E_5659;
    let ended = view && (v[1] == 2 || v[1] == 3);
    let has = saved().is_some();
    let rows: &[(usize, &str)] = if c.after_fight && ended {
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
    let mut v = [0i32; 64];
    let view = gd_ui_envoy(v.as_mut_ptr(), 64) == 0 && v[0] == 0x454E_5659;
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
const UP_NAMES: [&str; 6] = ["POWER", "GUARD", "EXTRA STOCK", "HEAVY", "SWIFT", "SPRING"];
const UP_STAT: [&str; 6] = ["Damage dealt", "Damage taken", "Stock pool", "Weight", "Run speed", "Jump height"];
const UP_TEXT: [&str; 6] = [
    "Both fighters hit harder.",
    "Both fighters take less damage.",
    "One more shared stock, now.",
    "Both fighters are harder to launch.",
    "Both fighters walk and dash faster.",
    "Both fighters jump higher.",
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
    b: [u8; 96],
    n: usize,
}
impl Buf {
    fn new() -> Self {
        Buf { b: [0; 96], n: 0 }
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
    /// x1000 as "x1.20"
    fn x(&mut self, v: i32) -> &mut Self {
        self.s("x").i(v / 1000).s(".");
        let f = (v % 1000).abs() / 10;
        if f < 10 {
            self.s("0");
        }
        self.i(f)
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
        let mut v = [0i32; 64];
        // only an armed run has a HUD or a between-fights line: with the mods loaded and no run armed, VS is retail
        if gd_ui_envoy(v.as_mut_ptr(), 64) != 0 || v[0] != 0x454E_5659 || st & ST_ARMED == 0 {
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
            hud(e, &v, vw);
            if v[10] != 0 {
                reward(e, &v, vw, vh);
            } else if v[30] >= 0 && v[31] < 150 {
                let mut s = Buf::new();
                s.s("TOOK ").s(UP_NAMES[(v[30].clamp(0, 5)) as usize]);
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

unsafe fn hud(e: &mut Enc<16384>, v: &[i32; 64], _vw: f32) {
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
    if v[27] >= 0 {
        let mut s = Buf::new();
        s.s("STAGE SHIFT IN ").i((v[27] + 59) / 60).s("s");
        e.text(x + 14.0, y + 106.0, ROLE_LABEL, DIM, s.str());
    }
}

unsafe fn reward(e: &mut Enc<16384>, v: &[i32; 64], vw: f32, vh: f32) {
    e.rect(0.0, 0.0, vw, vh, 0x0000_00B0);
    let cx = vw * 0.5;
    let mut s = Buf::new();
    s.s("FIGHT ").i(v[2]).s(" CLEARED  -  CHOOSE AN UPGRADE");
    e.text_ex(cx, 200.0, ROLE_TITLE, WHITE, ALIGN_CENTER, vw - 40.0, s.str());
    let secs = (v[12] + 59) / 60;
    let mut s = Buf::new();
    s.i(secs).s("s");
    e.text_ex(cx, 240.0, ROLE_VALUE, if secs <= 5 { ACCENT } else { DIM }, ALIGN_CENTER, 0.0, s.str());
    let gap = 24.0;
    let w = ((vw - 60.0 - 2.0 * gap) / 3.0).min(340.0);
    let h = 260.0;
    let x0 = cx - (3.0 * w + 2.0 * gap) * 0.5;
    let y = 290.0;
    for i in 0..3 {
        let id = v[13 + i].clamp(0, 5) as usize;
        let x = x0 + i as f32 * (w + gap);
        let sel = v[11] == i as i32;
        e.slice(SLICE_FILL_R14, x, y, w, h, if sel { 0x1C1C_30F0 } else { 0x0E0E_18E8 });
        if sel {
            e.slice(SLICE_LINE_R14_3, x, y, w, h, ACCENT);
        }
        e.text_ex(x + w * 0.5, y + 24.0, ROLE_TITLE, if sel { WHITE } else { DIM }, ALIGN_CENTER, w - 20.0, UP_NAMES[id]);
        e.text_ex(x + w * 0.5, y + 90.0, ROLE_LABEL, DIM, ALIGN_CENTER, w - 20.0, UP_STAT[id]);
        let (bf, af) = (v[16 + i], v[19 + i]);
        let mut s = Buf::new();
        if id == 2 {
            s.i(bf).s("  ->  ").i(af);
        } else {
            s.x(bf).s("  ->  ").x(af);
        }
        e.text_ex(x + w * 0.5, y + 120.0, ROLE_VALUE, GOOD, ALIGN_CENTER, w - 20.0, s.str());
        e.text_ex(x + w * 0.5, y + 180.0, ROLE_BODY, DIM, ALIGN_CENTER, w - 30.0, UP_TEXT[id]);
        if i == 0 {
            e.text_ex(x + w * 0.5, y + h - 30.0, ROLE_LABEL, 0x6060_70FF, ALIGN_CENTER, 0.0, "(taken on timeout)");
        }
    }
    e.text_ex(cx, y + h + 40.0, ROLE_LABEL, DIM, ALIGN_CENTER, 0.0, "LEFT / RIGHT choose     A take");
}

unsafe fn between(e: &mut Enc<16384>, v: &[i32; 64], vw: f32, _vh: f32) {
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
            l2.s(ck_name(v[6])).s(" + ").s(ck_name(v[7])).s("   -  any stage, Start");
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
