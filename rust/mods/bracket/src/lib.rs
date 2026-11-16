//! bracket: the Brackets screens, a gd-ui-v1.4 presentation mod bundled with the game (wasm-ui-lab-api.md A6.2,
//! docs/rust-port/notes/brackets-ui.md). The Party pack look: sticker panels (white paper, a grape-ink outline and a
//! hard drop shadow), sunny #FFD84A, grape ink #2A1A6E, Bricolage Grotesque for display text and Source Sans 3 for body.
//!
//! Screens: the main-menu BRACKETS tile (Y opens; a star badge when your set is called), Brackets home (the next-thing
//! card, MY EVENTS, BROWSE EVENTS / ACCOUNT; signed out: the link-account card), Account (key fingerprint, server,
//! auto-upload, link code, upload queue), Browse events, and the set flow (ready, character select, stage strike /
//! ban / pick, game, result card, set end) under a set-score bar. All data is host lists and every action a host
//! request; the key never reaches this mod (only its fingerprint). Live set data needs the server's (b) API; until
//! then `MELEE_BRACKET_MOCK=1` makes the host serve made-up data in the same record shapes.
//!
//! Input: d-pad / stick / arrows move, A / Enter acts, B / Escape goes back (home closes). Closed: only the tile.
//! One `gd_ui_submit` a present from a fixed buffer, no allocation, lists re-read only when their generation moves.
#![no_std]
#![allow(static_mut_refs)]

use gd_ui_enc::*;

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
const UI_ABI_1_4: u32 = (1 << 16) | 4;

// gd-ui-lab-v1.h GD_UI_LIST_* / GD_UI_REQ_*
const L_STATUS: u32 = 4;
const L_BK: u32 = 9;
const L_BKQ: u32 = 10;
const L_EVENTS: u32 = 11;
const L_NEXT: u32 = 12;
const L_SET: u32 = 13;
const L_BROWSE: u32 = 14;
const L_EVENT: u32 = 15;
const R_STATUS: i32 = 13;
const R_LINK: i32 = 14;
const R_MY_SETS: i32 = 15;
const R_READY: i32 = 16;
const R_AUTO: i32 = 18;
const R_STAGE: i32 = 19;
const R_EVENTS: i32 = 20;
const R_MOCK_STEP: i32 = 21;
const R_WATCH: i32 = 22;
const R_EVENT_VIEW: i32 = 23;
const R_START: i32 = 24;
const R_JOIN: i32 = 17;

const DL: u16 = 0x1;
const DR: u16 = 0x2;
const DD: u16 = 0x4;
const DU: u16 = 0x8;
const BA: u16 = 0x100;
const BB: u16 = 0x200;
const BY: u16 = 0x800;
const K_ENTER: u32 = 1;
const K_ESC: u32 = 2;
const K_BACK: u32 = 3;
const K_UP: u32 = 4;
const K_DOWN: u32 = 5;
const K_LEFT: u32 = 6;
const K_RIGHT: u32 = 7;

// the Party pack palette (RRGGBBAA)
const SUNNY: u32 = 0xFFD8_4AFF;
const GRAPE: u32 = 0x2A1A_6EFF;
const PAPER: u32 = 0xFFFF_FFFF;
const CREAM: u32 = 0xFFF6_DCFF;
const CORAL: u32 = 0xFF6B_5BFF;
const LIME: u32 = 0x7ED9_57FF;
const FADED: u32 = 0x8E84_B0FF;
const DIM: u32 = 0x1A10_46D8;

const DISPLAY: i32 = ROLE_TITLE | TEXT_DISPLAY;
const BIG: i32 = ROLE_BIG | TEXT_DISPLAY;
const LABEL: i32 = ROLE_LABEL | TEXT_DISPLAY;
const BODY: i32 = ROLE_BODY | TEXT_BODY;
const VALUE: i32 = ROLE_VALUE | TEXT_BODY;
const MONO: i32 = ROLE_VALUE | TEXT_MONO;

const GM_MENU: i32 = 1;
const MAXR: usize = 24;
const RECB: usize = 256;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";
const CODE: usize = 24;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scr {
    Home,
    Account,
    Browse,
    Event,
    Set,
}

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
    fn sync(&mut self, kind: u32) {
        let info = unsafe { gd_ui_list_info(kind) };
        if info < 0 || (info >> 16) + 1 == self.gen {
            return;
        }
        self.gen = (info >> 16) + 1;
        self.n = ((info & 0xFFFF) as usize).min(MAXR);
        for i in 0..self.n {
            let r = unsafe { gd_ui_list_get(kind, i as u32, self.rec[i].as_mut_ptr(), RECB as u32) };
            self.len[i] = r.max(0) as u16;
        }
    }
    fn f(&self, i: usize, k: usize) -> &str {
        if i >= self.n {
            return "";
        }
        let b = &self.rec[i][..self.len[i] as usize];
        let (mut s, mut idx) = (0, 0);
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

fn log(s: &str) {
    unsafe { gd_ui_log(s.as_ptr(), s.len() as u32) };
}

/// Field `k` of a '|'-separated field.
fn part(s: &str, k: usize) -> &str {
    s.split('|').nth(k).unwrap_or("")
}

fn num(s: &str) -> u32 {
    s.bytes().take_while(|c| c.is_ascii_digit()).fold(0u32, |a, c| a.wrapping_mul(10).wrapping_add((c - b'0') as u32))
}

/// A small text builder (no allocation).
struct Buf {
    b: [u8; 128],
    n: usize,
}
impl Buf {
    const fn new() -> Buf {
        Buf { b: [0; 128], n: 0 }
    }
    fn clear(&mut self) -> &mut Self {
        self.n = 0;
        self
    }
    fn s(&mut self, t: &str) -> &mut Self {
        for &c in t.as_bytes() {
            if self.n < self.b.len() {
                self.b[self.n] = c;
                self.n += 1;
            }
        }
        self
    }
    fn u(&mut self, mut v: u32) -> &mut Self {
        let mut d = [0u8; 10];
        let mut k = 0;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            k += 1;
            v /= 10;
            if v == 0 {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            let c = [d[k]];
            self.s(utf8(&c));
        }
        self
    }
    fn as_str(&self) -> &str {
        utf8(&self.b[..self.n])
    }
}

struct S {
    setup: bool,
    open: bool,
    cap: u32,
    pad: [u8; 16],
    keys: [u32; 32],
    prev: u16,
    scr: Scr,
    sel: usize,
    /// the stage cursor in the strike / ban / pick steps
    stage: usize,
    /// the last BRACKET_WATCH sent (0 nothing, 1 menu, 2 home, 3 set)
    watching: u8,
    /// the BRACKET generation that opened the screens by itself (back from a game)
    opened_gen: i32,
    edit: bool,
    code: [u8; CODE],
    n: usize,
    /// presents since open (the cards' little entrance)
    t: u32,
    bk: List,
    q: List,
    st: List,
    ev: List,
    next: List,
    set: List,
    browse: List,
    event: List,
    enc: Enc<16384>,
    b: Buf,
}

static mut ST: S = S {
    setup: false, open: false, cap: 0, pad: [0; 16], keys: [0; 32], prev: 0, scr: Scr::Home, sel: 0, stage: 0, watching: 0, opened_gen: 0,
    edit: false, code: [0; CODE], n: 0, t: 0, bk: List::new(), q: List::new(), st: List::new(), ev: List::new(),
    next: List::new(), set: List::new(), browse: List::new(), event: List::new(), enc: Enc::new(), b: Buf::new(),
};

// ---------------------------------------------------------------------------------------------------- drawing

/// A sticker panel: a hard grape drop shadow, a grape outline, the face.
fn sticker(e: &mut Enc<16384>, x: f32, y: f32, w: f32, h: f32, face: u32) {
    e.round_rect(x + 5.0, y + 6.0, w, h, 16.0, GRAPE);
    e.shape(SHAPE_BOX, x, y, w, h, 16.0, 3.0, 0.0, face, GRAPE);
}

/// A button sticker; `on` is the hovered one (sunny, lifted).
fn button(e: &mut Enc<16384>, x: f32, y: f32, w: f32, label: &str, on: bool) {
    let lift = if on { -3.0 } else { 0.0 };
    e.round_rect(x + 4.0, y + 5.0, w, 44.0, 22.0, GRAPE);
    e.shape(SHAPE_BOX, x, y + lift, w, 44.0, 22.0, 3.0, 0.0, if on { SUNNY } else { PAPER }, GRAPE);
    e.text_ex(x + w * 0.5, y + 11.0 + lift, LABEL, GRAPE, ALIGN_CENTER, 0.0, label);
}

fn pill(e: &mut Enc<16384>, x: f32, y: f32, label: &str, fill: u32, ink: u32) -> f32 {
    let w = measure(label, LABEL) + 22.0;
    e.shape(SHAPE_BOX, x, y, w, 26.0, 13.0, 2.0, 0.0, fill, GRAPE);
    e.text(x + 11.0, y + 4.0, LABEL, ink, label);
    w
}

fn star(e: &mut Enc<16384>, cx: f32, cy: f32, d: f32) {
    e.circle(cx + 2.0, cy + 3.0, d, GRAPE);
    e.shape(SHAPE_BOX, cx - d * 0.5, cy - d * 0.5, d, d, d, 2.5, 0.0, CORAL, GRAPE);
    e.text_ex(cx, cy - d * 0.33, LABEL, PAPER, ALIGN_CENTER, 0.0, "\u{2605}");
}

impl S {
    fn request(&self, op: i32, arg: &[u8]) {
        unsafe { gd_ui_request(op, arg.as_ptr(), arg.len() as u32) };
    }

    /// Tell the host what is on screen (it polls the server only for that).
    fn watch(&mut self, w: u8) {
        if self.watching != w {
            self.watching = w;
            self.request(R_WATCH, &[b'0' + w]);
        }
    }

    fn linked(&self) -> bool {
        self.bk.f(0, 1) == "1"
    }
    fn called(&self) -> bool {
        self.bk.f(0, 8) == "1"
    }
    fn phase(&self) -> &str {
        self.set.f(0, 0)
    }
    /// The set flow is on (ready sent, not yet back home).
    fn in_set(&self) -> bool {
        !matches!(self.phase(), "" | "called")
    }

    fn read_input(&mut self) -> (u16, u16, usize) {
        let mut held = 0u16;
        for p in 0..4 {
            if unsafe { gd_ui_pad(p, self.pad.as_mut_ptr()) } == 0 {
                held |= u16::from_le_bytes([self.pad[0], self.pad[1]]);
                let (x, y) = (self.pad[2] as i8, self.pad[3] as i8);
                if x > 50 { held |= DR }
                if x < -50 { held |= DL }
                if y > 50 { held |= DU }
                if y < -50 { held |= DD }
            }
        }
        let nk = if self.open { unsafe { gd_ui_keys(self.keys.as_mut_ptr(), 32) }.max(0) as usize } else { 0 };
        (held, held & !self.prev, nk)
    }

    fn open_now(&mut self) {
        self.open = true;
        self.edit = false;
        self.scr = if self.in_set() { Scr::Set } else { Scr::Home };
        self.sel = 0;
        self.t = 0;
        self.cap = unsafe { gd_ui_capture(0xF, CAP_EXCLUSIVE | CAP_KEYBOARD) };
        self.prev = 0xFFFF;
        // the first bracket request starts the host's service (key, queue, a probe of the server)
        self.request(R_STATUS, b"");
        self.request(R_MY_SETS, b"");
        log("bracket: open");
    }

    fn close_now(&mut self) {
        self.open = false;
        if self.cap != 0 {
            unsafe { gd_ui_release(self.cap) };
            self.cap = 0;
        }
        log("bracket: closed");
    }

    fn go(&mut self, s: Scr) {
        self.scr = s;
        self.sel = 0;
        self.t = 0;
        self.edit = false;
        let name = match s { Scr::Home => "home", Scr::Account => "account", Scr::Browse => "browse events", Scr::Event => "event", Scr::Set => "set" };
        self.b.clear().s("bracket: screen ").s(name);
        log(self.b.as_str());
    }

    /// The home screen's rows: the next card's button (when it has one), then BROWSE EVENTS, ACCOUNT. Signed out:
    /// the link card's code entry, then BROWSE EVENTS, ACCOUNT.
    fn home_rows(&self) -> usize {
        3
    }

    /// 1 up 2 down 3 left 4 right 5 a 6 b 7 backspace, 0x100 | char
    fn act(&mut self, a: u32) {
        if self.edit {
            return self.act_edit(a);
        }
        match self.scr {
            Scr::Home => match a {
                1 => self.sel = (self.sel + self.home_rows() - 1) % self.home_rows(),
                2 => self.sel = (self.sel + 1) % self.home_rows(),
                3 | 4 if self.sel >= 1 => self.sel = if self.sel == 1 { 2 } else { 1 },
                5 => match self.sel {
                    0 if !self.linked() => self.edit = true,
                    0 if self.next.f(0, 0) == "called" => {
                        self.request(R_READY, b"");
                        self.go(Scr::Set);
                    }
                    0 if self.next.f(0, 0) == "playing" => self.go(Scr::Set),
                    0 if self.in_set() => self.go(Scr::Set),
                    0 => self.request(R_MY_SETS, b""),
                    1 => {
                        self.request(R_EVENTS, b"");
                        self.go(Scr::Browse);
                    }
                    _ => self.go(Scr::Account),
                },
                6 => self.close_now(),
                _ => {}
            },
            Scr::Account => match a {
                1 => self.sel = (self.sel + 2) % 3,
                2 => self.sel = (self.sel + 1) % 3,
                5 => match self.sel {
                    0 => {
                        let on = self.bk.f(0, 4) == "1";
                        self.request(R_AUTO, if on { b"0" } else { b"1" });
                    }
                    1 => self.edit = true,
                    _ => self.request(R_STATUS, b""),
                },
                6 => self.go(Scr::Home),
                _ => {}
            },
            Scr::Browse => {
                let n = self.browse.n.max(1);
                match a {
                    1 => self.sel = (self.sel + n - 1) % n,
                    2 => self.sel = (self.sel + 1) % n,
                    5 if self.browse.n > 0 => {
                        let mut b = Buf::new();
                        b.s(self.browse.f(self.sel, 0));
                        self.request(R_EVENT_VIEW, &b.b[..b.n]);
                        self.go(Scr::Event);
                    }
                    6 => self.go(Scr::Home),
                    _ => {}
                }
            }
            Scr::Event => match a {
                5 if self.event.f(0, 2) == "signup" => {
                    let mut b = Buf::new();
                    b.s(self.event.f(0, 0));
                    self.request(R_JOIN, &b.b[..b.n]);
                }
                6 => {
                    self.go(Scr::Browse);
                    self.request(R_EVENTS, b"");
                }
                _ => {}
            },
            #[allow(unreachable_patterns)]
            Scr::Browse => {
                if a == 6 || a == 5 {
                    self.go(Scr::Home);
                }
            }
            Scr::Set => self.act_set(a),
        }
    }

    fn act_edit(&mut self, a: u32) {
        match a {
            1 | 2 => {
                if self.n == 0 {
                    self.code[0] = CHARSET[0];
                    self.n = 1;
                }
                let c = self.code[self.n - 1];
                let i = CHARSET.iter().position(|&x| x == c).unwrap_or(0);
                let k = if a == 1 { (i + 1) % CHARSET.len() } else { (i + CHARSET.len() - 1) % CHARSET.len() };
                self.code[self.n - 1] = CHARSET[k];
            }
            4 if self.n < CODE => {
                self.code[self.n] = CHARSET[0];
                self.n += 1;
            }
            3 | 7 => self.n = self.n.saturating_sub(1),
            5 => {
                self.edit = false;
                let c = self.code;
                self.request(R_LINK, &c[..self.n]);
                log("bracket: link code sent");
            }
            6 => self.edit = false,
            v if v & 0x100 != 0 => {
                let ch = ((v & 0xFF) as u8).to_ascii_uppercase();
                if self.n < CODE && CHARSET.contains(&ch) {
                    self.code[self.n] = ch;
                    self.n += 1;
                }
            }
            _ => {}
        }
    }

    fn stage_open(&self, i: usize) -> bool {
        self.set.f(0, 10).as_bytes().get(i) == Some(&b'0')
    }

    /// The set record's phase as a static string (so it does not hold a borrow of the list).
    fn ph(&self) -> &'static str {
        match self.phase() {
            "wait_ready" => "wait_ready",
            "css" => "css",
            "strike" => "strike",
            "ban" => "ban",
            "pick" => "pick",
            "game" => "game",
            "result" => "result",
            "done" => "done",
            "called" => "called",
            "judging" => "judging",
            "review" => "review",
            _ => "",
        }
    }

    fn act_set(&mut self, a: u32) {
        let ph = self.ph();
        match ph {
            "strike" | "ban" | "pick" => {
                let n = self.set.f(0, 9).split('|').count().max(1);
                match a {
                    3 => self.stage = (self.stage + n - 1) % n,
                    4 => self.stage = (self.stage + 1) % n,
                    1 => self.stage = (self.stage + 3 * n - 3) % n,
                    2 => self.stage = (self.stage + 3) % n,
                    5 if self.set.f(0, 7) == "1" && self.stage_open(self.stage) => {
                        self.b.clear().u(self.stage as u32);
                        let b = Buf { b: self.b.b, n: self.b.n };
                        self.request(R_STAGE, &b.b[..b.n]);
                    }
                    6 => self.go(Scr::Home),
                    _ => {}
                }
            }
            "css" if a == 5 => self.request(R_MOCK_STEP, b"css"),
            "game" if a == 5 => self.request(R_START, b""),
            "done" if a == 5 || a == 6 => self.go(Scr::Home),
            _ if a == 6 => self.go(Scr::Home),
            _ => {}
        }
    }

    fn frame(&mut self) {
        if !self.setup {
            self.setup = true;
            unsafe {
                gd_ui_theme(0);
                gd_ui_capture(0xF, CAP_SHARED);
            }
        }
        let on_menu = unsafe { gd_ui_retail_scene() } >> 8 == GM_MENU;
        if !on_menu {
            if self.open {
                self.close_now();
            }
            self.watch(0);
            self.prev = 0;
            return;
        }
        let w = if !self.open { 1 } else if self.scr == Scr::Set { 3 } else { 2 };
        self.watch(w);
        self.bk.sync(L_BK);
        if !self.open {
            let (held, edge, _) = self.read_input();
            self.prev = held;
            // back from a bracket game: the set flow opens by itself (BRACKET field 9, once)
            if edge & BY != 0 || (self.bk.f(0, 9) == "1" && self.bk.gen != self.opened_gen) {
                self.opened_gen = self.bk.gen;
                self.open_now();
            } else {
                return self.draw_tile();
            }
        }
        let (held, edge, nk) = self.read_input();
        self.prev = held;
        for (b, a) in [(DU, 1), (DD, 2), (DL, 3), (DR, 4), (BA, 5), (BB, 6)] {
            if edge & b != 0 {
                self.act(a);
                if !self.open {
                    return;
                }
            }
        }
        for i in 0..nk {
            let k = self.keys[i];
            let (kind, v) = (k >> 24, k & 0xFF_FFFF);
            let a = if kind == 1 {
                match v { K_UP => 1, K_DOWN => 2, K_LEFT => 3, K_RIGHT => 4, K_ENTER => 5, K_ESC => 6, K_BACK => 7, _ => 0 }
            } else if kind == 3 && self.edit && v < 128 {
                0x100 | v
            } else {
                0
            };
            if a != 0 {
                self.act(a);
                if !self.open {
                    return;
                }
            }
        }
        self.q.sync(L_BKQ);
        self.st.sync(L_STATUS);
        self.ev.sync(L_EVENTS);
        self.next.sync(L_NEXT);
        self.browse.sync(L_BROWSE);
        self.event.sync(L_EVENT);
        let before = self.set.gen;
        self.set.sync(L_SET);
        if self.set.gen != before {
            self.b.clear().s("bracket: set ").s(self.set.f(0, 0)).s(" ").s(self.set.f(0, 1)).s("-").s(self.set.f(0, 2));
            log(self.b.as_str());
            // a stage step starts on the first open stage
            if !self.stage_open(self.stage) {
                self.stage = (0..8).find(|&i| self.stage_open(i)).unwrap_or(0);
            }
        }
        self.t += 1;
        self.draw();
    }

    fn frame_rect(&self) -> (f32, f32, f32) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        (sc, b[4] / sc, b[5] / sc)
    }

    /// The main-menu BRACKETS tile (bottom right; the browser's hint holds the bottom left), a star when the set is up.
    fn draw_tile(&mut self) {
        let (sc, vw, vh) = self.frame_rect();
        let called = self.called();
        let e = &mut self.enc;
        e.clear();
        e.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
        let (w, h) = (196.0, 58.0);
        let (x, y) = (vw - w - 28.0, vh - h - 28.0);
        sticker(e, x, y, w, h, SUNNY);
        e.text(x + 18.0, y + 9.0, DISPLAY, GRAPE, "BRACKETS");
        e.shape(SHAPE_BOX, x + w - 40.0, y + 16.0, 26.0, 24.0, 6.0, 2.0, 0.0, PAPER, GRAPE);
        e.text_ex(x + w - 27.0, y + 19.0, LABEL, GRAPE, ALIGN_CENTER, 0.0, "Y");
        if called {
            star(e, x + w - 4.0, y - 2.0, 34.0);
            e.text_ex(x + w * 0.5, y - 26.0, LABEL, PAPER, ALIGN_CENTER, 0.0, "Your set is up!");
        }
        e.submit();
    }

    fn draw(&mut self) {
        let (sc, vw, vh) = self.frame_rect();
        {
            let e = &mut self.enc;
            e.clear();
            e.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
            e.rect(0.0, 0.0, vw, vh, DIM);
        }
        match self.scr {
            Scr::Home => self.draw_home(vw),
            Scr::Account => self.draw_account(vw),
            Scr::Browse => self.draw_browse(vw),
            Scr::Event => self.draw_event(vw),
            Scr::Set => self.draw_set(vw),
        }
        self.draw_status(vw, vh);
        self.enc.submit();
    }

    fn header(&mut self, vw: f32, title: &str, back: &str) {
        let e = &mut self.enc;
        let x = (vw - 820.0) * 0.5;
        let w = measure(title, BIG) + 40.0;
        sticker(e, x, 34.0, w, 62.0, SUNNY);
        e.text(x + 20.0, 44.0, BIG, GRAPE, title);
        e.text_ex(x + 820.0, 58.0, LABEL, PAPER, ALIGN_RIGHT, 0.0, back);
    }

    /// The last request's answer (STATUS), else the service's own message, on a small paper strip at the bottom.
    fn draw_status(&mut self, vw: f32, vh: f32) {
        let (msg, err) = if self.st.n > 0 && !self.st.f(0, 1).is_empty() { (self.st.f(0, 1), self.st.f(0, 0) == "1") } else { (self.bk.f(0, 6), self.bk.f(0, 7) == "1") };
        if msg.is_empty() || msg == "ready" {
            return;
        }
        let w = (measure(msg, VALUE) + 30.0).min(820.0);
        let e = &mut self.enc;
        let x = (vw - w) * 0.5;
        e.shape(SHAPE_BOX, x, vh - 52.0, w, 32.0, 16.0, 2.0, 0.0, if err { CORAL } else { CREAM }, GRAPE);
        e.text_ex(x + 15.0, vh - 46.0, VALUE, GRAPE, ALIGN_LEFT, w - 30.0, msg);
    }

    fn draw_home(&mut self, vw: f32) {
        self.header(vw, "BRACKETS", "B  close");
        let x = (vw - 820.0) * 0.5;
        let mut y = 122.0;
        let rise = if self.t < 10 { (10 - self.t) as f32 * 3.0 } else { 0.0 };
        if !self.linked() {
            // signed out: the link-account card
            let on = self.sel == 0;
            let e = &mut self.enc;
            sticker(e, x, y + rise, 820.0, 200.0, PAPER);
            e.text(x + 28.0, y + 22.0 + rise, BIG, GRAPE, "Link your account");
            e.text_ex(x + 28.0, y + 74.0 + rise, BODY, GRAPE, ALIGN_LEFT, 760.0, "Get a link code on the bracket site, then type it here. Your game key stays on this PC.");
            e.shape(SHAPE_BOX, x + 28.0, y + 120.0 + rise, 420.0, 52.0, 12.0, 3.0, 0.0, if on { SUNNY } else { CREAM }, GRAPE);
            let shown = if self.edit || self.n > 0 { utf8(&self.code[..self.n]) } else { "A  type the code" };
            e.text(x + 44.0, y + 133.0 + rise, if self.n > 0 { MONO } else { LABEL }, GRAPE, shown);
            if self.edit && (self.t / 20) % 2 == 0 {
                let cx = x + 46.0 + measure(utf8(&self.code[..self.n]), MONO);
                e.rect(cx, y + 132.0 + rise, 3.0, 26.0, GRAPE);
            }
            if self.edit {
                e.text(x + 470.0, y + 136.0 + rise, VALUE, FADED, "A send   B cancel");
            }
            y += 232.0;
        } else {
            y = self.draw_next_card(x, y + rise) + 24.0;
            // MY EVENTS
            let n = self.ev.n.min(4);
            let h = 56.0 + n.max(1) as f32 * 40.0;
            let e = &mut self.enc;
            sticker(e, x, y, 820.0, h, PAPER);
            e.text(x + 24.0, y + 14.0, LABEL, FADED, "MY EVENTS");
            if n == 0 {
                e.text(x + 24.0, y + 50.0, BODY, FADED, "No events yet. Browse events to sign up.");
            }
            for i in 0..n {
                let ry = y + 48.0 + i as f32 * 40.0;
                if i > 0 {
                    e.rect(x + 20.0, ry - 6.0, 780.0, 2.0, CREAM);
                }
                e.text(x + 24.0, ry, BODY, GRAPE, self.ev.f(i, 1));
                e.text_ex(x + 796.0, ry + 2.0, VALUE, FADED, ALIGN_RIGHT, 380.0, self.ev.f(i, 2));
            }
            y += h + 24.0;
        }
        let s1 = self.sel == 1;
        let s2 = self.sel == 2;
        let e = &mut self.enc;
        button(e, x, y, 300.0, "BROWSE EVENTS", s1);
        button(e, x + 324.0, y, 220.0, "ACCOUNT", s2);
    }

    /// The next-thing card. Returns its bottom.
    fn draw_next_card(&mut self, x: f32, y: f32) -> f32 {
        let kind = self.next.f(0, 0);
        let on = self.sel == 0;
        let (event, round, opp, eta) = (self.next.f(0, 1), self.next.f(0, 2), self.next.f(0, 3), num(self.next.f(0, 4)));
        let e = &mut self.enc;
        match kind {
            "called" | "playing" => {
                let h = 210.0;
                sticker(e, x, y, 820.0, h, SUNNY);
                star(e, x + 790.0, y + 8.0, 46.0);
                e.text(x + 28.0, y + 18.0, LABEL, GRAPE, event);
                e.text(x + 28.0, y + 44.0, BIG, GRAPE, if kind == "called" { "YOUR SET IS UP" } else { "YOUR SET IS ON" });
                self.b.clear().s(round).s("  \u{00B7}  vs ").s(opp);
                e.text(x + 28.0, y + 100.0, DISPLAY, GRAPE, self.b.as_str());
                button(e, x + 28.0, y + 146.0, 220.0, if kind == "called" { "READY" } else { "BACK TO SET" }, on);
                y + h
            }
            "upcoming" => {
                let h = 170.0;
                sticker(e, x, y, 820.0, h, CREAM);
                e.text(x + 28.0, y + 18.0, LABEL, FADED, "YOUR NEXT MATCH");
                self.b.clear().s(round).s("  \u{00B7}  vs ").s(opp);
                e.text(x + 28.0, y + 44.0, DISPLAY, GRAPE, self.b.as_str());
                self.b.clear().s(event).s("  \u{00B7}  in about ").u((eta + 59) / 60).s(" min");
                e.text(x + 28.0, y + 84.0, BODY, GRAPE, self.b.as_str());
                button(e, x + 28.0, y + 112.0, 200.0, "REFRESH", on);
                y + h
            }
            "review" => {
                let h = 130.0;
                sticker(e, x, y, 820.0, h, CREAM);
                e.text(x + 28.0, y + 18.0, LABEL, FADED, event);
                e.text(x + 28.0, y + 44.0, DISPLAY, GRAPE, "Your set is with the organiser");
                e.text(x + 28.0, y + 88.0, BODY, FADED, "Wait here: the decision shows up on its own.");
                y + h
            }
            _ => {
                let h = 130.0;
                sticker(e, x, y, 820.0, h, CREAM);
                e.text(x + 28.0, y + 18.0, LABEL, FADED, "NEXT");
                e.text(x + 28.0, y + 44.0, DISPLAY, GRAPE, "Nothing to play right now");
                e.text(x + 28.0, y + 88.0, BODY, FADED, "Your sets show up here when they are called.");
                y + h
            }
        }
    }

    fn draw_account(&mut self, vw: f32) {
        self.header(vw, "ACCOUNT", "B  back");
        let x = (vw - 820.0) * 0.5;
        let y = 122.0;
        let bk = &self.bk;
        let reach = match bk.f(0, 2) { "1" => "reachable", "2" => "not reachable", _ => "checking..." };
        let rows: [(&str, &str); 4] = [
            ("Key", if bk.f(0, 0).is_empty() { "making the key..." } else { bk.f(0, 0) }),
            ("Server", bk.f(0, 3)),
            ("Status", reach),
            ("Account", if bk.f(0, 1) == "1" { "linked" } else { "not linked" }),
        ];
        let e = &mut self.enc;
        sticker(e, x, y, 820.0, 430.0, PAPER);
        let mut yy = y + 22.0;
        for (k, v) in rows {
            e.text(x + 26.0, yy, LABEL, FADED, k);
            e.text_ex(x + 160.0, yy, MONO, GRAPE, ALIGN_LEFT, 640.0, v);
            yy += 28.0;
        }
        yy += 10.0;
        const ROWS: [&str; 3] = ["Auto-upload", "Link code", "Check server"];
        for (i, name) in ROWS.iter().enumerate() {
            let on = i == self.sel;
            e.shape(SHAPE_BOX, x + 18.0, yy - 6.0, 784.0, 36.0, 10.0, if on { 2.5 } else { 0.0 }, 0.0, if on { SUNNY } else { PAPER }, GRAPE);
            e.text(x + 30.0, yy, BODY, GRAPE, name);
            let val: &str = match i {
                0 => if self.bk.f(0, 4) == "1" { "ON  netplay games upload" } else { "OFF" },
                1 => if self.edit || self.n > 0 { utf8(&self.code[..self.n]) } else { "A  type the code" },
                _ => "A",
            };
            e.text_ex(x + 250.0, yy + 2.0, if i == 1 && self.n > 0 { MONO } else { VALUE }, GRAPE, ALIGN_LEFT, 540.0, val);
            yy += 40.0;
        }
        yy += 6.0;
        e.text(x + 26.0, yy, LABEL, FADED, "UPLOAD QUEUE");
        yy += 26.0;
        if self.q.n == 0 {
            e.text(x + 26.0, yy, VALUE, FADED, "Nothing queued.");
        }
        let left = ((y + 418.0 - yy) / 24.0).max(0.0) as usize;
        for i in 0..self.q.n.min(left) {
            let q = &self.q;
            let state = q.f(i, 1);
            e.text_ex(x + 26.0, yy, MONO, GRAPE, ALIGN_LEFT, 280.0, q.f(i, 0));
            let c = match state { "failed" => CORAL, "done" => LIME, _ => SUNNY };
            e.shape(SHAPE_BOX, x + 316.0, yy - 1.0, 96.0, 22.0, 11.0, 2.0, 0.0, c, GRAPE);
            e.text_ex(x + 364.0, yy + 1.0, ROLE_LABEL | TEXT_BODY, GRAPE, ALIGN_CENTER, 0.0, state);
            e.text_ex(x + 424.0, yy, VALUE, FADED, ALIGN_LEFT, 376.0, if q.f(i, 4).is_empty() { q.f(i, 3) } else { q.f(i, 4) });
            yy += 24.0;
        }
    }

    fn draw_browse(&mut self, vw: f32) {
        self.header(vw, "BROWSE EVENTS", "B  back");
        let x = (vw - 820.0) * 0.5;
        let e = &mut self.enc;
        if self.browse.n == 0 {
            sticker(e, x, 122.0, 820.0, 120.0, CREAM);
            e.text(x + 28.0, 142.0, DISPLAY, GRAPE, "No events listed");
            e.text_ex(x + 28.0, 190.0, BODY, GRAPE, ALIGN_LEFT, 760.0, "Events the server runs show here, sign-up first.");
            return;
        }
        let n = self.browse.n.min(8);
        sticker(e, x, 122.0, 820.0, 24.0 + n as f32 * 62.0, PAPER);
        for i in 0..n {
            let y = 134.0 + i as f32 * 62.0;
            if i == self.sel {
                e.shape(SHAPE_BOX, x + 12.0, y - 4.0, 796.0, 58.0, 12.0, 2.5, 0.0, SUNNY, GRAPE);
            } else if i > 0 {
                e.rect(x + 20.0, y - 6.0, 780.0, 2.0, CREAM);
            }
            let b = &self.browse;
            e.text(x + 26.0, y + 2.0, BODY, GRAPE, b.f(i, 1));
            e.text_ex(x + 26.0, y + 28.0, VALUE, FADED, ALIGN_LEFT, 560.0, b.f(i, 5));
            let (label, fill) = match b.f(i, 2) {
                "signup" => ("SIGN-UP OPEN", LIME),
                "running" => ("RUNNING", SUNNY),
                _ => ("DONE", CREAM),
            };
            let w = measure(label, LABEL) + 22.0;
            pill(e, x + 796.0 - w, y + 2.0, label, fill, GRAPE);
            self.b.clear().s(b.f(i, 3)).s(" players");
            e.text_ex(x + 796.0, y + 32.0, VALUE, FADED, ALIGN_RIGHT, 0.0, self.b.as_str());
        }
    }

    fn draw_event(&mut self, vw: f32) {
        self.header(vw, "EVENT", "B  back");
        let x = (vw - 820.0) * 0.5;
        let ev = &self.event;
        let e = &mut self.enc;
        if ev.n == 0 {
            sticker(e, x, 122.0, 820.0, 100.0, CREAM);
            e.text(x + 28.0, 150.0, DISPLAY, GRAPE, "Loading...");
            return;
        }
        let open = ev.f(0, 2) == "signup";
        sticker(e, x, 122.0, 820.0, 360.0, PAPER);
        e.text(x + 28.0, 140.0, BIG, GRAPE, ev.f(0, 1));
        e.text_ex(x + 28.0, 196.0, BODY, GRAPE, ALIGN_LEFT, 760.0, ev.f(0, 5));
        self.b.clear().s("Best of ").s(ev.f(0, 6)).s("  \u{00B7}  ").s(ev.f(0, 7)).s(" stocks  \u{00B7}  ").u(num(ev.f(0, 8)) / 60).s(" min  \u{00B7}  ").s(ev.f(0, 3)).s(" players");
        e.text(x + 28.0, 236.0, LABEL, GRAPE, self.b.as_str());
        e.text(x + 28.0, 274.0, LABEL, FADED, "STAGES");
        let mut sx = x + 28.0;
        let mut sy = 300.0;
        for name in ev.f(0, 9).split('|').filter(|s| !s.is_empty()) {
            let w = measure(name, VALUE) + 22.0;
            if sx + w > x + 796.0 {
                sx = x + 28.0;
                sy += 32.0;
            }
            e.shape(SHAPE_BOX, sx, sy, w, 26.0, 13.0, 2.0, 0.0, CREAM, GRAPE);
            e.text(sx + 11.0, sy + 4.0, VALUE, GRAPE, name);
            sx += w + 8.0;
        }
        e.text_ex(x + 28.0, sy + 44.0, VALUE, FADED, ALIGN_LEFT, 760.0, ev.f(0, 10));
        button(e, x + 28.0, 500.0, 220.0, if open { "JOIN" } else { "SIGN-UP CLOSED" }, open);
    }

    /// The set flow, the set score on top throughout.
    fn draw_set(&mut self, vw: f32) {
        let x = (vw - 820.0) * 0.5;
        let s = &self.set;
        let (me, opp, bo, game) = (num(s.f(0, 1)), num(s.f(0, 2)), num(s.f(0, 3)).max(1), num(s.f(0, 4)).max(1));
        let ph = self.ph();
        {
            let e = &mut self.enc;
            // the score bar
            sticker(e, x, 30.0, 820.0, 76.0, PAPER);
            e.text(x + 26.0, 40.0, LABEL, FADED, s.f(0, 15));
            e.text(x + 26.0, 60.0, DISPLAY, GRAPE, s.f(0, 5));
            e.text_ex(x + 794.0, 60.0, DISPLAY, GRAPE, ALIGN_RIGHT, 0.0, s.f(0, 6));
            self.b.clear().u(me).s("  -  ").u(opp);
            e.shape(SHAPE_BOX, x + 330.0, 42.0, 160.0, 54.0, 14.0, 3.0, 0.0, SUNNY, GRAPE);
            e.text_ex(x + 410.0, 48.0, BIG, GRAPE, ALIGN_CENTER, 0.0, self.b.as_str());
            self.b.clear().s("Best of ").u(bo).s("  \u{00B7}  Game ").u(game);
            e.text_ex(x + 410.0, 104.0 + 6.0, LABEL, PAPER, ALIGN_CENTER, 0.0, self.b.as_str());
        }
        let y = 150.0;
        match ph {
            "wait_ready" => self.card(x, y, "READY!", "Waiting for your opponent to ready up...", None),
            "css" => self.card(x, y, "PICK YOUR CHARACTER", "Both players go to the normal character select.", Some("A  characters locked (mock)")),
            "called" => self.card(x, y, "YOUR SET IS UP", "Say ready from Brackets home.", None),
            "judging" => self.card(x, y, "CHECKING THE GAME", "Both replays are in; the server is judging the game.", None),
            "review" => self.card(x, y, "WITH THE ORGANISER", "The set is being reviewed. Hang tight.", None),
            "strike" | "ban" | "pick" => self.draw_stages(x, y, ph),
            "game" => {
                let st = num(self.set.f(0, 11)) as usize;
                let name = part(self.set.f(0, 9), st);
                self.b.clear().s("Game ").u(game).s(" on ").s(name);
                let t = Buf { b: self.b.b, n: self.b.n };
                let mock = self.set.f(0, 19) == "1";
                let mut body = Buf::new();
                body.s(if mock { "Good luck! Both players go to the character select; the stage is set." } else { self.set.f(0, 14) });
                self.card(x, y, t.as_str(), body.as_str(), Some(if mock { "A  START THE GAME (mock: plays it)" } else { "A  START THE GAME" }));
            }
            "result" => self.draw_result(x, y, me, opp),
            "done" => self.draw_done(x, y, me, opp),
            _ => self.card(x, y, "SET", "Waiting for the server...", None),
        }
    }

    fn card(&mut self, x: f32, y: f32, title: &str, body: &str, button_label: Option<&str>) {
        let e = &mut self.enc;
        sticker(e, x, y, 820.0, if button_label.is_some() { 220.0 } else { 160.0 }, PAPER);
        e.text(x + 28.0, y + 22.0, BIG, GRAPE, title);
        e.text_ex(x + 28.0, y + 84.0, BODY, GRAPE, ALIGN_LEFT, 760.0, body);
        if let Some(b) = button_label {
            button(e, x + 28.0, y + 140.0, measure(b, LABEL) + 48.0, b, true);
        }
    }

    fn draw_stages(&mut self, x: f32, y: f32, ph: &str) {
        let mine = self.set.f(0, 7) == "1";
        let left = num(self.set.f(0, 8));
        let opp = self.set.f(0, 6);
        let names = self.set.f(0, 9);
        let gone = self.set.f(0, 10).as_bytes();
        let verb = match ph { "strike" => "STRIKE", "ban" => "BAN", _ => "PICK" };
        if mine {
            self.b.clear().s(verb).s(if ph == "pick" { " THE STAGE" } else if left == 1 { " 1 STAGE" } else { " 2 STAGES" });
        } else {
            self.b.clear().s(opp).s(if ph == "pick" { " is picking..." } else if ph == "ban" { " is banning..." } else { " is striking..." });
        }
        let t = Buf { b: self.b.b, n: self.b.n };
        let e = &mut self.enc;
        sticker(e, x, y, 820.0, 64.0, if mine { SUNNY } else { CREAM });
        e.text(x + 24.0, y + 10.0, BIG, GRAPE, t.as_str());
        if ph == "strike" {
            e.text_ex(x + 796.0, y + 22.0, LABEL, GRAPE, ALIGN_RIGHT, 0.0, "Game 1: strike 1-2-1");
        }
        // the stage grid: 3 per row
        for (i, name) in names.split('|').enumerate() {
            let (cx, cy) = (x + (i % 3) as f32 * 276.0, y + 90.0 + (i / 3) as f32 * 120.0);
            let open = gone.get(i) == Some(&b'0');
            let on = mine && i == self.stage;
            let lift = if on { -4.0 } else { 0.0 };
            e.round_rect(cx + 5.0, cy + 6.0, 260.0, 100.0, 14.0, GRAPE);
            e.shape(SHAPE_BOX, cx, cy + lift, 260.0, 100.0, 14.0, if on { 4.0 } else { 3.0 }, 0.0, if !open { FADED } else if on { SUNNY } else { PAPER }, GRAPE);
            e.text_ex(cx + 130.0, cy + 36.0 + lift, LABEL, if open { GRAPE } else { CREAM }, ALIGN_CENTER, 240.0, name);
            if !open {
                // a struck stage: a coral slash
                e.rot(-0.35, cx + 130.0, cy + 50.0);
                e.round_rect(cx + 30.0, cy + 46.0, 200.0, 8.0, 4.0, CORAL);
                e.rot(0.0, 0.0, 0.0);
            }
        }
    }

    fn draw_result(&mut self, x: f32, y: f32, me: u32, opp: u32) {
        let last = self.set.f(0, 12);
        let won = part(last, 0) == "1";
        let (stocks, pct) = (num(part(last, 1)), num(part(last, 2)));
        let who = if won { "You take it!" } else { self.set.f(0, 6) };
        self.b.clear().s(who).s(if won { "" } else { " takes it" });
        let t = Buf { b: self.b.b, n: self.b.n };
        let next = self.set.f(0, 14);
        let e = &mut self.enc;
        sticker(e, x + 110.0, y, 600.0, 280.0, if won { SUNNY } else { PAPER });
        e.text_ex(x + 410.0, y + 22.0, BIG, GRAPE, ALIGN_CENTER, 0.0, t.as_str());
        self.b.clear().u(me).s("  -  ").u(opp);
        e.text_ex(x + 410.0, y + 82.0, BIG, GRAPE, ALIGN_CENTER, 0.0, self.b.as_str());
        self.b.clear().u(stocks).s(if stocks == 1 { " stock" } else { " stocks" }).s(" left at ").u(pct).s("%");
        e.text_ex(x + 410.0, y + 150.0, BODY, GRAPE, ALIGN_CENTER, 0.0, self.b.as_str());
        if !next.is_empty() {
            let w = measure(next, LABEL) + 30.0;
            e.shape(SHAPE_BOX, x + 410.0 - w * 0.5, y + 200.0, w, 40.0, 20.0, 3.0, 0.0, PAPER, GRAPE);
            e.text_ex(x + 410.0, y + 209.0, LABEL, GRAPE, ALIGN_CENTER, 0.0, next);
        }
    }

    fn draw_done(&mut self, x: f32, y: f32, me: u32, opp: u32) {
        let won = me > opp;
        let verified = self.set.f(0, 13) == "1";
        let next = self.set.f(0, 14);
        let e = &mut self.enc;
        sticker(e, x, y, 820.0, 330.0, if won { SUNNY } else { PAPER });
        e.text(x + 28.0, y + 20.0, LABEL, GRAPE, "FINAL");
        e.text(x + 28.0, y + 44.0, BIG, GRAPE, if won { "SET WON" } else { "SET LOST" });
        self.b.clear().u(me).s("  -  ").u(opp);
        e.text_ex(x + 790.0, y + 30.0, BIG, GRAPE, ALIGN_RIGHT, 0.0, self.b.as_str());
        if verified {
            pill(e, x + 28.0, y + 104.0, "\u{2713} verified", LIME, GRAPE);
        } else {
            pill(e, x + 28.0, y + 104.0, "checking the replays...", CREAM, FADED);
        }
        e.rect(x + 24.0, y + 150.0, 772.0, 2.0, GRAPE);
        e.text(x + 28.0, y + 164.0, LABEL, FADED, "NEXT");
        e.text_ex(x + 28.0, y + 190.0, DISPLAY, GRAPE, ALIGN_LEFT, 760.0, next);
        button(e, x + 28.0, y + 258.0, 280.0, "BRACKETS HOME", true);
    }
}

#[no_mangle]
pub extern "C" fn gd_ui_frame(_present: u32) {
    unsafe { (*core::ptr::addr_of_mut!(ST)).frame() };
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
pub static gdmod_ui_abi: u32 = UI_ABI_1_4;

#[no_mangle]
pub static gd_mod_manifest: Manifest = Manifest {
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0002_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_INPUT | FEATURE_UI_HOST,
    optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.bracket"), mod_name: text(b"Brackets"),
};
