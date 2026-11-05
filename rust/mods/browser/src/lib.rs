//! browser: the in-game browser as a gd-ui-v1.2 presentation mod (docs/rust-port/notes/wasm-ui-lab-api.md A6), in the
//! LAB's theme A parts (panel, key caps, rows, accent selection; rust/mods/lab).
//!
//! Open it on the main menu with Z (any port); closed, a one-line hint in the bottom-left corner says so. Sections (L / R, or Tab): Modes, Mods, Replays, Skins, Online.
//! D-pad / stick / arrows move (hold repeats 15 then every 4), A / Enter activates, B / Escape goes back or closes.
//! Skins: left / right picks the skin for the slot, A applies. Online: A on Join / Spectate edits the code (up / down
//! cycle the character, left / right move, or type it), A sends, B cancels.
//! Everything the game does is a host request (`gd_ui_request`); the catalogs are host lists (`gd_ui_list_*`).
//!
//! Performance: closed, nothing is drawn and nothing is submitted (one scene query and, on the main menu only, the
//! pads); open, one `gd_ui_submit` from a fixed buffer, no allocation (no_std, no alloc), lists re-read only when their
//! generation changes, the description re-wrapped only when the selection changes.
#![no_std]
#![allow(static_mut_refs)]

mod txt;

use gd_ui_enc::*;
use txt::Txt;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_keys(out: *mut u32, max: u32) -> i32;
    fn gd_ui_retail_scene() -> i32;
    fn gd_ui_list_info(kind: u32) -> i32;
    fn gd_ui_list_get(kind: u32, i: u32, out: *mut u8, cap: u32) -> i32;
    fn gd_ui_request(op: i32, s: *const u8, len: u32) -> i32;
}

const FEATURE_UI_HOST: u32 = 1 << 16;
const UI_ABI_1_2: u32 = (1 << 16) | 2;

// lists and requests (gd-ui-lab-v1.h GD_UI_LIST_* / GD_UI_REQ_*)
const L_MODES: u32 = 0;
const L_MODS: u32 = 1;
const L_REPLAYS: u32 = 2;
const L_SKINS: u32 = 3;
const L_STATUS: u32 = 4;
const L_ONLINE: u32 = 5;
const R_LAUNCH: i32 = 1;
const R_MOD_SET: i32 = 2;
const R_REPLAY: i32 = 3;
const R_SKIN: i32 = 4;
const R_HOST: i32 = 5;
const R_JOIN: i32 = 6;
const R_SPECTATE: i32 = 7;
const R_REFRESH: i32 = 8;

// pad
const DL: u16 = 0x1;
const DR: u16 = 0x2;
const DD: u16 = 0x4;
const DU: u16 = 0x8;
const BZ: u16 = 0x10;
const BR: u16 = 0x20;
const BL: u16 = 0x40;
const BA: u16 = 0x100;
const BB: u16 = 0x200;
const BY: u16 = 0x800;
// keys (GD_UI_KEY_*)
const K_ENTER: u32 = 1;
const K_ESC: u32 = 2;
const K_BACK: u32 = 3;
const K_UP: u32 = 4;
const K_DOWN: u32 = 5;
const K_LEFT: u32 = 6;
const K_RIGHT: u32 = 7;
const K_TAB: u32 = 8;

// theme tokens
const T_BG: usize = 0;
const T_BORDER: usize = 2;
const T_TEXT: usize = 3;
const T_MUTED: usize = 4;
const T_ACCENT: usize = 5;
const T_INK: usize = 6;
const T_DIVIDER: usize = 7;
const T_KEY_BORDER: usize = 8;
const T_KEY_TEXT: usize = 9;
const T_P1: usize = 10;

const R_TITLE: i32 = ROLE_TITLE | TEXT_MONO;
const R_NUM: i32 = ROLE_VALUE | TEXT_MONO;

const GM_MENU: i32 = 1;
const MAXR: usize = 64;
const RECB: usize = 256;
const ROWS: usize = 13;
const ROW_H: f32 = 30.0;
const SECTIONS: [&str; 5] = ["Modes", "Mods", "Replays", "Skins", "Online"];
const LISTS: [u32; 5] = [L_MODES, L_MODS, L_REPLAYS, L_SKINS, L_ONLINE];
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.:-";
const CODE: usize = 24;
/// The closed browser's main-menu hint (draw_hint).
const HINT: &str = "Browser: modes, LAB, replays, mods, skins, online";

struct List {
    gen: i32,
    n: usize,
    rec: [[u8; RECB]; MAXR],
    len: [u16; MAXR],
}

impl List {
    const fn new() -> List {
        List { gen: 0, n: 0, rec: [[0; RECB]; MAXR], len: [0; MAXR] }
    }
    /// Re-read the list when its generation moved. True when it changed.
    fn sync(&mut self, kind: u32) -> bool {
        let info = unsafe { gd_ui_list_info(kind) };
        if info < 0 || (info >> 16) + 1 == self.gen {
            return false;
        }
        self.gen = (info >> 16) + 1;
        self.n = ((info & 0xFFFF) as usize).min(MAXR);
        for i in 0..self.n {
            let r = unsafe { gd_ui_list_get(kind, i as u32, self.rec[i].as_mut_ptr(), RECB as u32) };
            self.len[i] = r.max(0) as u16;
        }
        true
    }
    /// Field `k` of record `i` ("" when absent).
    fn f(&self, i: usize, k: usize) -> &str {
        if i >= self.n {
            return "";
        }
        let b = &self.rec[i][..self.len[i] as usize];
        let mut s = 0;
        let mut idx = 0;
        for (p, &c) in b.iter().enumerate() {
            if c == 0 {
                if idx == k {
                    return utf8(&b[s..p]);
                }
                idx += 1;
                s = p + 1;
            }
        }
        if idx == k { utf8(&b[s..]) } else { "" }
    }
}

fn utf8(b: &[u8]) -> &str {
    match core::str::from_utf8(b) {
        Ok(s) => s,
        Err(e) => unsafe { core::str::from_utf8_unchecked(&b[..e.valid_up_to()]) },
    }
}

fn measure(s: &str, role: i32) -> f32 {
    gd_ui_enc::measure(s, role)
}

struct Code {
    b: [u8; CODE],
    n: usize,
    cur: usize,
}

impl Code {
    const fn new() -> Code {
        Code { b: [0; CODE], n: 0, cur: 0 }
    }
    fn as_str(&self) -> &str {
        utf8(&self.b[..self.n])
    }
}

struct B {
    setup: bool,
    tok: [u32; 26],
    shared: u32,
    cap: u32,
    open: bool,
    closing: bool,
    sec: usize,
    sel: [usize; 5],
    top: [usize; 5],
    lists: [List; 5],
    status: List,
    status_gen: i32,
    // skins: distinct slots (first record index of each) and the picked choice (0 vanilla, k = k-th skin of the slot)
    slots: [u8; MAXR],
    nslots: usize,
    pick: [u8; MAXR],
    // online: 0 host, 1 join, 2 spectate; editing row (0 none) and the codes
    edit: usize,
    codes: [Code; 2],
    // input
    pad: [u8; 16],
    prev: u16,
    hold: u16,
    rep: u32,
    keys: [u32; 32],
    // cached widths and the wrapped description of the selection
    tab_w: [f32; 5],
    wrap_key: u32,
    wrap: [(u16, u16); 8],
    nwrap: usize,
    enc: Enc<12288>,
    t: Txt,
}

static mut S: B = B {
    setup: false,
    tok: [0; 26],
    shared: 0,
    cap: 0,
    open: false,
    closing: false,
    sec: 0,
    sel: [0; 5],
    top: [0; 5],
    lists: [List::new(), List::new(), List::new(), List::new(), List::new()],
    status: List::new(),
    status_gen: 0,
    slots: [0; MAXR],
    nslots: 0,
    pick: [0; MAXR],
    edit: 0,
    codes: [Code::new(), Code::new()],
    pad: [0; 16],
    prev: 0,
    hold: 0,
    rep: 0,
    keys: [0; 32],
    tab_w: [0.0; 5],
    wrap_key: 0,
    wrap: [(0, 0); 8],
    nwrap: 0,
    enc: Enc::new(),
    t: Txt::new(),
};

impl B {
    fn tk(&self, i: usize) -> u32 {
        self.tok[i]
    }

    fn init(&mut self) {
        unsafe {
            gd_ui_theme(0);
            for i in 0..26 {
                self.tok[i] = gd_ui_token(i as i32);
            }
            self.shared = gd_ui_capture(0xF, CAP_SHARED);
        }
        for i in 0..5 {
            self.tab_w[i] = measure(SECTIONS[i], ROLE_LABEL) + 24.0;
        }
    }

    /// Rows of the current section.
    fn rows(&self) -> usize {
        match self.sec {
            3 => self.nslots,
            4 => 3,
            s => self.lists[s].n,
        }
    }

    fn request(&mut self, op: i32, arg: &[u8]) {
        unsafe { gd_ui_request(op, arg.as_ptr(), arg.len() as u32) };
    }

    fn open_now(&mut self) {
        self.open = true;
        self.closing = false;
        self.edit = 0;
        self.cap = unsafe { gd_ui_capture(0xF, CAP_EXCLUSIVE | CAP_KEYBOARD) };
        self.prev = 0xFFFF; // the press that opened it is not an edge inside
        self.refresh_all();
        log("browser: open");
        self.log_section();
    }

    fn close_now(&mut self) {
        self.open = false;
        self.closing = false;
        if self.cap != 0 {
            unsafe { gd_ui_release(self.cap) };
            self.cap = 0;
        }
        log("browser: closed");
    }

    fn refresh_all(&mut self) {
        for s in 0..5 {
            if self.lists[s].sync(LISTS[s]) && s == 3 {
                self.group_skins();
            }
        }
    }

    fn log_section(&mut self) {
        let n = self.rows() as i32;
        self.t.clear().s("browser: section ").s(SECTIONS[self.sec]).s(", ").i(n).s(" rows");
        log(self.t.as_str());
    }

    fn group_skins(&mut self) {
        let l = &self.lists[3];
        self.nslots = 0;
        for i in 0..l.n {
            if i == 0 || l.f(i, 0) != l.f(i - 1, 0) {
                self.slots[self.nslots] = i as u8;
                self.pick[self.nslots] = 0;
                self.nslots += 1;
            }
            if l.f(i, 3) == "1" {
                let s = self.nslots - 1;
                self.pick[s] = (i - self.slots[s] as usize + 1) as u8;
            }
        }
    }

    fn slot_count(&self, s: usize) -> usize {
        let end = if s + 1 < self.nslots { self.slots[s + 1] as usize } else { self.lists[3].n };
        end - self.slots[s] as usize
    }

    // ---------------------------------------------------------------------------------------------- input

    fn read_input(&mut self) -> (u16, u16, usize) {
        let mut held = 0u16;
        let mut sx = 0i8;
        let mut sy = 0i8;
        for p in 0..4 {
            if unsafe { gd_ui_pad(p, self.pad.as_mut_ptr()) } == 0 {
                held |= u16::from_le_bytes([self.pad[0], self.pad[1]]);
                let (x, y) = (self.pad[2] as i8, self.pad[3] as i8);
                if x.unsigned_abs() > sx.unsigned_abs() { sx = x }
                if y.unsigned_abs() > sy.unsigned_abs() { sy = y }
            }
        }
        if sx > 50 { held |= DR }
        if sx < -50 { held |= DL }
        if sy > 50 { held |= DU }
        if sy < -50 { held |= DD }
        let nk = if self.open { unsafe { gd_ui_keys(self.keys.as_mut_ptr(), 32) }.max(0) as usize } else { 0 };
        (held, held & !self.prev, nk)
    }

    fn frame(&mut self) {
        if !self.setup {
            self.setup = true;
            self.init();
        }
        let scene = unsafe { gd_ui_retail_scene() };
        let on_menu = scene >> 8 == GM_MENU;
        if !self.open {
            if !on_menu {
                self.prev = 0;
                return;
            }
            let (held, edge, _) = self.read_input();
            self.prev = held;
            if edge & BZ != 0 {
                self.open_now();
            } else {
                return self.draw_hint();
            }
        }
        if !on_menu {
            // the game left the main menu (a mode started): nothing stays on screen
            return self.close_now();
        }
        let (held, edge, nk) = self.read_input();
        self.prev = held;
        if self.closing {
            if held == 0 {
                self.close_now();
            }
            return;
        }
        // directional repeat: first press, then after 15 frames every 4
        let dirs = held & (DU | DD | DL | DR);
        let mut fire = edge & (DU | DD | DL | DR);
        if dirs != 0 && dirs == self.hold {
            self.rep += 1;
            if self.rep >= 15 && (self.rep - 15) % 4 == 0 {
                fire |= dirs;
            }
        } else {
            self.rep = 0;
        }
        self.hold = dirs;
        let mut act = 0u32; // 1 up 2 down 3 left 4 right 5 a 6 b 7 prev section 8 next section 9 refresh
        let mut q = [0u32; 40];
        let mut nq = 0;
        let push = |q: &mut [u32; 40], nq: &mut usize, v: u32| {
            if *nq < 40 {
                q[*nq] = v;
                *nq += 1;
            }
        };
        for (b, a) in [(DU, 1), (DD, 2), (DL, 3), (DR, 4)] {
            if fire & b != 0 {
                push(&mut q, &mut nq, a);
            }
        }
        for (b, a) in [(BA, 5), (BB, 6), (BL, 7), (BR, 8), (BY, 9)] {
            if edge & b != 0 {
                push(&mut q, &mut nq, a);
            }
        }
        for i in 0..nk {
            let k = self.keys[i];
            let (kind, v) = (k >> 24, k & 0xFF_FFFF);
            if kind == 1 {
                let a = match v { K_UP => 1, K_DOWN => 2, K_LEFT => 3, K_RIGHT => 4, K_ENTER => 5, K_ESC => 6, K_TAB => 8, K_BACK => 10, _ => 0 };
                if a != 0 {
                    push(&mut q, &mut nq, a);
                }
            } else if kind == 3 && self.edit != 0 {
                push(&mut q, &mut nq, 0x100 | v);
            }
        }
        for i in 0..nq {
            act = q[i];
            self.act(act);
            if !self.open || self.closing {
                break;
            }
        }
        let _ = act;
        if !self.open {
            return;
        }
        // lists and the status line (one query each; records re-read only on a change)
        if self.lists[self.sec].sync(LISTS[self.sec]) {
            if self.sec == 3 {
                self.group_skins();
            }
            self.wrap_key = u32::MAX;
            let r = self.rows();
            if self.sel[self.sec] >= r {
                self.sel[self.sec] = r.saturating_sub(1);
            }
        }
        if self.status.sync(L_STATUS) {
            if self.status_gen > 0 {
                self.t.clear().s("browser: status ").s(self.status.f(0, 1));
                log(self.t.as_str());
                if self.status.f(0, 0) == "2" {
                    self.closing = true;
                }
            }
            self.status_gen = self.status.gen;
        }
        self.draw();
    }

    fn act(&mut self, a: u32) {
        let rows = self.rows();
        let sec = self.sec;
        if self.edit != 0 {
            let c = &mut self.codes[self.edit - 1];
            match a {
                1 | 2 => {
                    if c.cur >= c.n {
                        if c.n < CODE {
                            c.b[c.n] = b'A';
                            c.n += 1;
                        } else {
                            return;
                        }
                    }
                    let p = CHARSET.iter().position(|&x| x == c.b[c.cur]).unwrap_or(0);
                    let l = CHARSET.len();
                    c.b[c.cur] = CHARSET[if a == 1 { (p + 1) % l } else { (p + l - 1) % l }];
                }
                3 => c.cur = c.cur.saturating_sub(1),
                4 => c.cur = (c.cur + 1).min(c.n).min(CODE - 1),
                10 => {
                    if c.cur > 0 && c.cur <= c.n {
                        c.b.copy_within(c.cur..c.n, c.cur - 1);
                        c.n -= 1;
                        c.cur -= 1;
                    }
                }
                5 => {
                    let row = self.edit;
                    self.edit = 0;
                    let mut buf = [0u8; CODE];
                    let n = self.codes[row - 1].n;
                    buf[..n].copy_from_slice(&self.codes[row - 1].b[..n]);
                    self.t.clear().s("browser: request ").s(if row == 1 { "join " } else { "spectate " }).s(utf8(&buf[..n]));
                    log(self.t.as_str());
                    self.request(if row == 1 { R_JOIN } else { R_SPECTATE }, &buf[..n]);
                }
                6 => self.edit = 0,
                v if v & 0x100 != 0 => {
                    let ch = (v & 0xFF) as u8;
                    if (ch.is_ascii_alphanumeric() || b".:-_/".contains(&ch)) && c.n < CODE {
                        let ch = ch.to_ascii_uppercase();
                        let at = c.cur.min(c.n);
                        c.b.copy_within(at..c.n, at + 1);
                        c.b[at] = ch;
                        c.n += 1;
                        c.cur = at + 1;
                    }
                }
                _ => {}
            }
            return;
        }
        match a {
            1 | 2 if rows > 0 => {
                let s = &mut self.sel[sec];
                *s = if a == 1 { (*s + rows - 1) % rows } else { (*s + 1) % rows };
            }
            3 | 4 if sec == 3 && self.nslots > 0 => {
                let s = self.sel[3];
                let n = self.slot_count(s) + 1;
                let p = self.pick[s] as usize;
                self.pick[s] = (if a == 3 { (p + n - 1) % n } else { (p + 1) % n }) as u8;
            }
            3 | 4 if sec == 1 && rows > 0 => self.toggle_mod(a == 4),
            7 | 8 | 3 | 4 => {
                self.sec = if a == 7 || a == 3 { (sec + 4) % 5 } else { (sec + 1) % 5 };
                self.wrap_key = u32::MAX;
                if self.lists[self.sec].sync(LISTS[self.sec]) && self.sec == 3 {
                    self.group_skins();
                }
                self.log_section();
            }
            5 if rows > 0 => self.activate(),
            6 => self.closing = true,
            9 => {
                let k = [b'0' + LISTS[sec] as u8];
                self.request(R_REFRESH, &k);
            }
            _ => {}
        }
    }

    fn toggle_mod(&mut self, on: bool) {
        let i = self.sel[1];
        let mut buf = [0u8; 64];
        let id = self.lists[1].f(i, 0).as_bytes();
        let n = id.len().min(60);
        buf[..n].copy_from_slice(&id[..n]);
        buf[n] = 0;
        buf[n + 1] = if on { b'1' } else { b'0' };
        self.t.clear().s("browser: request mod ").s(utf8(&buf[..n])).s(if on { " on" } else { " off" });
        log(self.t.as_str());
        self.request(R_MOD_SET, &buf[..n + 2]);
    }

    fn activate(&mut self) {
        let sec = self.sec;
        let i = self.sel[sec];
        let mut buf = [0u8; 128];
        let put = |buf: &mut [u8; 128], at: usize, s: &str| -> usize {
            let b = s.as_bytes();
            let n = b.len().min(buf.len() - at);
            buf[at..at + n].copy_from_slice(&b[..n]);
            at + n
        };
        match sec {
            0 => {
                let n = put(&mut buf, 0, self.lists[0].f(i, 0));
                self.t.clear().s("browser: request launch ").s(utf8(&buf[..n]));
                log(self.t.as_str());
                self.request(R_LAUNCH, &buf[..n]);
            }
            1 => {
                let on = self.lists[1].f(i, 4) != "1";
                self.toggle_mod(on);
            }
            2 => {
                let n = put(&mut buf, 0, self.lists[2].f(i, 0));
                self.t.clear().s("browser: request replay ").s(utf8(&buf[..n]));
                log(self.t.as_str());
                self.request(R_REPLAY, &buf[..n]);
            }
            3 => {
                let r0 = self.slots[i] as usize;
                let p = self.pick[i] as usize;
                let mut n = put(&mut buf, 0, self.lists[3].f(r0, 0));
                buf[n] = 0;
                n += 1;
                n = put(&mut buf, n, if p == 0 { "none" } else { self.lists[3].f(r0 + p - 1, 2) });
                self.t.clear().s("browser: request skin ").s(self.lists[3].f(r0, 1));
                log(self.t.as_str());
                self.request(R_SKIN, &buf[..n]);
            }
            _ => match i {
                0 => {
                    log("browser: request host");
                    self.request(R_HOST, &[]);
                }
                r => {
                    self.edit = r;
                    let c = &mut self.codes[r - 1];
                    c.cur = c.n.min(CODE - 1);
                }
            },
        }
    }

    // ---------------------------------------------------------------------------------------------- drawing

    fn draw(&mut self) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        let (vw, vh) = (b[4] / sc, b[5] / sc);
        let (w, h) = (980.0f32, 580.0f32);
        let (x, y) = (((vw - w) * 0.5) as i32 as f32, ((vh - h) * 0.5) as i32 as f32);
        self.enc.clear();
        self.enc.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
        // dim the menu behind, then the panel (theme A: panel bg, 1px border, radius 4)
        self.enc.rect(0.0, 0.0, vw, vh, 0x0000_0099);
        let (bg, bd, ac, ink, tx, mu, dv) = (self.tk(T_BG), self.tk(T_BORDER), self.tk(T_ACCENT), self.tk(T_INK), self.tk(T_TEXT), self.tk(T_MUTED), self.tk(T_DIVIDER));
        self.enc.slice(SLICE_FILL_R4, x, y, w, h, bg).slice(SLICE_LINE_R4, x, y, w, h, bd);
        self.enc.text(x + 20.0, y + 18.0, R_TITLE, ac, "BROWSER");
        // tabs
        let mut tx0 = x + w - 20.0;
        for s in (0..5).rev() {
            tx0 -= self.tab_w[s];
            let on = s == self.sec;
            if on {
                self.enc.slice(SLICE_FILL_R4, tx0, y + 14.0, self.tab_w[s] - 6.0, 24.0, ac);
            } else {
                self.enc.slice(SLICE_LINE_R4, tx0, y + 14.0, self.tab_w[s] - 6.0, 24.0, self.tk(T_KEY_BORDER));
            }
            self.enc.text_ex(tx0 + (self.tab_w[s] - 6.0) * 0.5, y + 20.0, ROLE_LABEL, if on { ink } else { self.tk(T_KEY_TEXT) }, ALIGN_CENTER, 0.0, SECTIONS[s]);
        }
        self.enc.rect(x + 20.0, y + 52.0, w - 40.0, 1.0, dv);
        // list
        let (lx, ly, lw) = (x + 20.0, y + 64.0, 560.0);
        let rows = self.rows();
        let sec = self.sec;
        let sel = self.sel[sec];
        if sel < self.top[sec] {
            self.top[sec] = sel;
        }
        if sel >= self.top[sec] + ROWS {
            self.top[sec] = sel + 1 - ROWS;
        }
        let top = self.top[sec];
        if rows == 0 {
            let empty = match sec {
                1 => "No mods (set mods_dir= in settings.cfg).",
                2 => "No replays in the replay folder (MELEE_SLP_DIR, else Replays).",
                3 => "No skins (melee cosmetics import FILE).",
                _ => "Nothing here.",
            };
            self.enc.text(lx + 4.0, ly + 8.0, ROLE_BODY, mu, empty);
        }
        for r in top..(top + ROWS).min(rows) {
            let ry = ly + (r - top) as f32 * ROW_H;
            let on = r == sel;
            if on {
                self.enc.slice(SLICE_FILL_R4, lx, ry, lw, ROW_H - 4.0, (ac & 0xFFFF_FF00) | 0x33).slice(SLICE_LINE_R4, lx, ry, lw, ROW_H - 4.0, ac);
            }
            self.row(r, lx, ry, lw, on);
        }
        if rows > ROWS {
            let th = ROWS as f32 * ROW_H;
            let kh = th * ROWS as f32 / rows as f32;
            let ky = ly + th * top as f32 / rows as f32;
            self.enc.rect(lx + lw + 6.0, ly, 2.0, th, dv).rect(lx + lw + 5.0, ky, 4.0, kh, mu);
        }
        // detail pane
        let (dx, dy, dw) = (x + 600.0, y + 64.0, w - 620.0);
        self.enc.slice(SLICE_LINE_R4, dx, dy, dw, ROWS as f32 * ROW_H - 4.0, bd);
        if rows > 0 {
            self.detail(dx + 14.0, dy + 12.0, dw - 28.0);
        }
        // status and hints
        let fy = y + h - 40.0;
        self.enc.rect(x + 20.0, fy - 10.0, w - 40.0, 1.0, dv);
        let code = self.status.f(0, 0);
        let st = self.status.f(0, 1);
        let col = if code == "1" { 0xFF6B6BFF } else { mu };
        self.enc.text_ex(x + 20.0, fy + 3.0, ROLE_VALUE, col, ALIGN_LEFT, 520.0, st);
        let hints: &[(&str, &str)] = if self.edit != 0 {
            &[("A", "Send"), ("B", "Cancel"), ("+-", "Char")]
        } else {
            match sec {
                1 => &[("A", "Toggle"), ("B", "Close"), ("L R", "Section")],
                3 => &[("A", "Apply"), ("< >", "Skin"), ("B", "Close")],
                _ => &[("A", "Select"), ("B", "Close"), ("L R", "Section")],
            }
        };
        let mut hx = x + w - 20.0;
        for &(k, l) in hints.iter().rev() {
            hx -= measure(l, ROLE_VALUE);
            self.enc.text(hx, fy + 3.0, ROLE_VALUE, mu, l);
            let kw = measure(k, ROLE_LABEL) + 12.0;
            hx -= 5.0 + kw;
            self.enc.slice(SLICE_LINE_R4, hx, fy, kw, 18.0, self.tk(T_KEY_BORDER));
            self.enc.text_ex(hx + kw * 0.5, fy + 4.0, ROLE_LABEL, self.tk(T_KEY_TEXT), ALIGN_CENTER, 0.0, k);
            hx -= 12.0;
        }
        let _ = tx;
        self.enc.submit();
    }

    /// Closed on the main menu: one line in the bottom-left corner, so a player can find the browser (and the LAB in
    /// its Modes list). Key cap and muted text as the panel's hints.
    fn draw_hint(&mut self) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        let vh = b[5] / sc;
        let (x, y) = (24.0f32, vh - 34.0);
        let kw = measure("Z", ROLE_LABEL) + 12.0;
        self.enc.clear();
        self.enc.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
        self.enc.slice(SLICE_FILL_R4, x - 6.0, y - 5.0, kw + 16.0 + measure(HINT, ROLE_VALUE) + 12.0, 28.0, (self.tk(T_BG) & 0xFFFF_FF00) | 0xCC);
        self.enc.slice(SLICE_LINE_R4, x, y, kw, 18.0, self.tk(T_KEY_BORDER));
        self.enc.text_ex(x + kw * 0.5, y + 4.0, ROLE_LABEL, self.tk(T_KEY_TEXT), ALIGN_CENTER, 0.0, "Z");
        self.enc.text(x + kw + 8.0, y + 3.0, ROLE_VALUE, self.tk(T_MUTED), HINT);
        self.enc.submit();
    }

    fn row(&mut self, r: usize, x: f32, y: f32, w: f32, on: bool) {
        let (tx, mu, ac) = (self.tk(T_TEXT), self.tk(T_MUTED), self.tk(T_ACCENT));
        let ty = y + 6.0;
        let l = &self.lists[self.sec];
        match self.sec {
            0 => {
                let avail = l.f(r, 4) == "1";
                self.enc.text(x + 10.0, ty, ROLE_BODY, if avail { tx } else { mu }, l.f(r, 2));
                self.enc.text_ex(x + w - 10.0, ty + 1.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, l.f(r, 1));
            }
            1 => {
                let name = if l.f(r, 1).is_empty() { l.f(r, 0) } else { l.f(r, 1) };
                let en = l.f(r, 4) == "1";
                self.enc.text_ex(x + 10.0, ty, ROLE_BODY, tx, ALIGN_LEFT, w - 90.0, name);
                let c = if en { ac } else { self.tk(T_KEY_BORDER) };
                self.enc.slice(if en { SLICE_FILL_R4 } else { SLICE_LINE_R4 }, x + w - 58.0, y + 4.0, 48.0, 18.0, c);
                self.enc.text_ex(x + w - 34.0, y + 8.0, ROLE_LABEL, if en { self.tk(T_INK) } else { mu }, ALIGN_CENTER, 0.0, if en { "ON" } else { "OFF" });
            }
            2 => {
                self.enc.text_ex(x + 10.0, ty, ROLE_BODY, tx, ALIGN_LEFT, w - 150.0, l.f(r, 4));
                self.enc.text_ex(x + w - 10.0, ty, R_NUM, mu, ALIGN_RIGHT, 0.0, l.f(r, 5));
                self.enc.text_ex(x + w - 60.0, ty + 1.0, ROLE_LABEL, mu, ALIGN_RIGHT, 0.0, l.f(r, 1));
            }
            3 => {
                let r0 = self.slots[r] as usize;
                let p = self.pick[r] as usize;
                let lst = &self.lists[3];
                self.enc.text(x + 10.0, ty, R_NUM, tx, lst.f(r0, 1));
                let v = if p == 0 { "Vanilla" } else { lst.f(r0 + p - 1, 2) };
                let cur = (0..self.slot_count(r)).find(|&k| lst.f(r0 + k, 3) == "1").map_or(0, |k| k + 1);
                self.enc.text_ex(x + w - 30.0, ty, ROLE_BODY, if p == cur { tx } else { ac }, ALIGN_RIGHT, 0.0, v);
                if on {
                    self.enc.text(x + w - 22.0, ty, ROLE_BODY, mu, ">");
                    self.enc.text_ex(x + w - 30.0 - measure(v, ROLE_BODY) - 8.0, ty, ROLE_BODY, mu, ALIGN_RIGHT, 0.0, "<");
                }
            }
            _ => {
                let label = ["Host a match", "Join a match", "Spectate a match"][r];
                self.enc.text(x + 10.0, ty, ROLE_BODY, tx, label);
                if r > 0 {
                    let c = &self.codes[r - 1];
                    let editing = self.edit == r;
                    let bx = x + w - 230.0;
                    self.enc.slice(SLICE_LINE_R4, bx, y + 3.0, 220.0, 20.0, if editing { ac } else { self.tk(T_KEY_BORDER) });
                    let s = if c.n == 0 && !editing { "code" } else { c.as_str() };
                    self.enc.text(bx + 8.0, y + 6.0, R_NUM, if c.n == 0 && !editing { mu } else { tx }, s);
                    if editing {
                        // the cursor: under the character it edits
                        let pre = utf8(&c.b[..c.cur.min(c.n)]);
                        let cx = bx + 8.0 + measure(pre, R_NUM);
                        self.enc.rect(cx, y + 19.0, 8.0, 2.0, self.tk(T_P1));
                    }
                }
            }
        }
    }

    fn detail(&mut self, x: f32, y: f32, w: f32) {
        let (tx, mu) = (self.tk(T_TEXT), self.tk(T_MUTED));
        let sec = self.sec;
        let i = self.sel[sec];
        let key = 1 << 31 | (sec as u32) << 16 | i as u32 | (self.lists[sec].gen as u32 & 0xFF) << 20;
        let dk = match sec { 0 => 3, 1 => 2, _ => 99 };
        if dk != 99 && key != self.wrap_key {
            self.wrap_key = key;
            self.rewrap(sec, i, dk, w);
        }
        // title, description record field, two extra lines
        let (title, desc_k, a, b): (&str, usize, &str, &str) = {
            let l = &self.lists[sec];
            match sec {
                0 => (l.f(i, 2), 3, l.f(i, 1), l.f(i, 5)),
                1 => (if l.f(i, 1).is_empty() { l.f(i, 0) } else { l.f(i, 1) }, 2, l.f(i, 3), l.f(i, 6)),
                2 => (l.f(i, 4), 99, l.f(i, 3), l.f(i, 2)),
                3 => {
                    let r0 = self.slots[i] as usize;
                    (l.f(r0, 1), 99, "", "")
                }
                _ => (["Host a match", "Join a match", "Spectate a match"][i], 99, "", ""),
            }
        };
        self.enc.text_ex(x, y, ROLE_BIG, tx, ALIGN_LEFT, w, title);
        let mut yy = y + 32.0;
        if !a.is_empty() {
            self.enc.text_ex(x, yy, ROLE_LABEL, self.tk(T_ACCENT), ALIGN_LEFT, w, a);
            yy += 22.0;
        }
        if desc_k != 99 {
            for k in 0..self.nwrap {
                let (s, e) = self.wrap[k];
                let d = self.lists[sec].f(i, desc_k);
                let line = &d[s as usize..e as usize];
                self.enc.text(x, yy, ROLE_BODY, tx, line);
                yy += 19.0;
            }
            yy += 6.0;
        }
        match sec {
            1 => {
                let l = &self.lists[1];
                let pending = l.f(i, 4) != l.f(i, 5);
                self.enc.text(x, yy, ROLE_VALUE, mu, if pending { "Changed: applies at the next launch" } else if l.f(i, 4) == "1" { "Enabled at this launch" } else { "Disabled" });
                yy += 20.0;
            }
            2 => {
                let l = &self.lists[2];
                self.enc.text_ex(x, yy, R_NUM, mu, ALIGN_LEFT, w, l.f(i, 6));
                yy += 20.0;
                self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, if l.f(i, 1) == "mrpl" { "A plays it with pause, step, seek and speed." } else { "Slippi: plays from the CLI (melee replays)." });
                yy += 20.0;
            }
            3 => {
                let r0 = self.slots[i] as usize;
                let n = self.slot_count(i);
                self.t.clear().i(n as i32).s(if n == 1 { " skin" } else { " skins" });
                self.enc.text(x, yy, ROLE_VALUE, mu, self.t.as_str());
                yy += 20.0;
                for k in 0..n {
                    let v = self.lists[3].f(r0 + k, 4);
                    if !v.is_empty() {
                        self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, v);
                        yy += 20.0;
                    }
                }
                self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, "Applies from the next match.");
                yy += 20.0;
            }
            4 => {
                let o = self.lists[4].f(0, 1);
                self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, match i { 0 => "Opens a room; the code shows in the game.", 1 => "Type the code the host sent you.", _ => "Watch a match by its spectate code." });
                yy += 22.0;
                self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, o);
                yy += 20.0;
            }
            _ => {}
        }
        if !b.is_empty() {
            self.enc.text_ex(x, yy, ROLE_VALUE, mu, ALIGN_LEFT, w, b);
        }
    }

    /// Word-wrap a description into at most 8 lines (only when the selection changed).
    fn rewrap(&mut self, sec: usize, i: usize, k: usize, w: f32) {
        self.nwrap = 0;
        let d = self.lists[sec].f(i, k);
        let bytes = d.as_bytes();
        let mut start = 0usize;
        let mut last_space = None;
        let mut p = 0usize;
        while p <= bytes.len() && self.nwrap < 8 {
            let end = p == bytes.len();
            if end || bytes[p] == b' ' {
                if measure(&d[start..p], ROLE_BODY) > w {
                    if let Some(sp) = last_space {
                        self.wrap[self.nwrap] = (start as u16, sp as u16);
                        self.nwrap += 1;
                        start = sp + 1;
                        last_space = None;
                        continue;
                    }
                }
                if end {
                    if start < p {
                        self.wrap[self.nwrap] = (start as u16, p as u16);
                        self.nwrap += 1;
                    }
                    break;
                }
                last_space = Some(p);
            }
            p += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe { (*core::ptr::addr_of_mut!(S)).frame() };
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
pub static gdmod_ui_abi: u32 = UI_ABI_1_2;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_INPUT | FEATURE_UI_HOST,
    optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.browser"), mod_name: text(b"Browser"),
};
