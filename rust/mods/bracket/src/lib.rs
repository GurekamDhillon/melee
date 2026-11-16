//! bracket: the Brackets screen, a gd-ui-v1.4 presentation mod bundled with the game (wasm-ui-lab-api.md A6.2). It shows
//! the native bracket client's state (rust/melee/src/bracket): the key's fingerprint, the server and whether it answers,
//! linked or not, auto-upload, a link-code entry and the upload queue. Every action is a host request; the key itself
//! never reaches this mod (the host hands out only the fingerprint).
//!
//! Y on the main menu opens it (a SHARED capture watches the pads while closed; open: EXCLUSIVE + KEYBOARD on all
//! ports). Up / down pick a row, A acts (Auto-upload toggles, Link code edits: type it or up / down cycle a character,
//! right adds one, left / Backspace removes, A / Enter sends, B cancels; Check server re-probes), B / Escape closes.
//! Closed it draws nothing but a one-line hint and submits only that; open, one `gd_ui_submit` a present from a fixed
//! buffer, no allocation, lists re-read only when their generation moves.
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

// gd-ui-lab-v1.h GD_UI_LIST_BRACKET* / GD_UI_REQ_BRACKET_*
const L_STATUS: u32 = 4;
const L_BK: u32 = 9;
const L_BKQ: u32 = 10;
const R_STATUS: i32 = 13;
const R_LINK: i32 = 14;
const R_AUTO: i32 = 18;

const DD: u16 = 0x4;
const DU: u16 = 0x8;
const DL: u16 = 0x1;
const DR: u16 = 0x2;
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

const T_BG: usize = 0;
const T_BORDER: usize = 2;
const T_TEXT: usize = 3;
const T_MUTED: usize = 4;
const T_ACCENT: usize = 5;
const T_INK: usize = 6;
const T_DIVIDER: usize = 7;
const T_KEY_BORDER: usize = 8;
const T_KEY_TEXT: usize = 9;

const GM_MENU: i32 = 1;
const MAXR: usize = 24;
const RECB: usize = 256;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";
const CODE: usize = 24;
const ROWS: [&str; 3] = ["Auto-upload", "Link code", "Check server"];
const HINT: &str = "Brackets: upload queue, link code";

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

struct S {
    setup: bool,
    open: bool,
    cap: u32,
    tok: [u32; 26],
    pad: [u8; 16],
    keys: [u32; 32],
    prev: u16,
    sel: usize,
    edit: bool,
    code: [u8; CODE],
    n: usize,
    bk: List,
    q: List,
    st: List,
    enc: Enc<12288>,
}

static mut ST: S = S {
    setup: false, open: false, cap: 0, tok: [0; 26], pad: [0; 16], keys: [0; 32], prev: 0, sel: 0, edit: false,
    code: [0; CODE], n: 0, bk: List::new(), q: List::new(), st: List::new(), enc: Enc::new(),
};

impl S {
    fn request(&self, op: i32, arg: &[u8]) {
        unsafe { gd_ui_request(op, arg.as_ptr(), arg.len() as u32) };
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
        self.cap = unsafe { gd_ui_capture(0xF, CAP_EXCLUSIVE | CAP_KEYBOARD) };
        self.prev = 0xFFFF;
        // the first bracket request starts the host's service (key, queue, a probe of the server)
        self.request(R_STATUS, b"");
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

    /// 1 up 2 down 3 left 4 right 5 a 6 b 7 backspace, 0x100 | char
    fn act(&mut self, a: u32) {
        if self.edit {
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
                    let ch = (v & 0xFF) as u8;
                    let ch = ch.to_ascii_uppercase();
                    if self.n < CODE && CHARSET.contains(&ch) {
                        self.code[self.n] = ch;
                        self.n += 1;
                    }
                }
                _ => {}
            }
            return;
        }
        match a {
            1 => self.sel = (self.sel + ROWS.len() - 1) % ROWS.len(),
            2 => self.sel = (self.sel + 1) % ROWS.len(),
            5 => match self.sel {
                0 => {
                    let on = self.bk.f(0, 4) == "1";
                    self.request(R_AUTO, if on { b"0" } else { b"1" });
                }
                1 => self.edit = true,
                _ => self.request(R_STATUS, b""),
            },
            6 => self.close_now(),
            _ => {}
        }
    }

    fn frame(&mut self) {
        if !self.setup {
            self.setup = true;
            unsafe {
                gd_ui_theme(0);
                for i in 0..26 {
                    self.tok[i] = gd_ui_token(i as i32);
                }
                gd_ui_capture(0xF, CAP_SHARED);
            }
        }
        let on_menu = unsafe { gd_ui_retail_scene() } >> 8 == GM_MENU;
        if !self.open {
            if !on_menu {
                self.prev = 0;
                return;
            }
            let (held, edge, _) = self.read_input();
            self.prev = held;
            if edge & BY != 0 {
                self.open_now();
            } else {
                return self.draw_hint();
            }
        }
        if !on_menu {
            return self.close_now();
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
        self.bk.sync(L_BK);
        self.q.sync(L_BKQ);
        self.st.sync(L_STATUS);
        self.draw();
    }

    fn draw_hint(&mut self) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        let (vw, vh) = (b[4] / sc, b[5] / sc);
        let kw = measure("Y", ROLE_LABEL) + 12.0;
        let tw = measure(HINT, ROLE_VALUE);
        // bottom-right: the browser's hint holds the bottom-left corner
        let x = vw - 24.0 - kw - 8.0 - tw;
        let y = vh - 34.0;
        let t = self.tok;
        let e = &mut self.enc;
        e.clear();
        e.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
        e.slice(SLICE_FILL_R4, x - 6.0, y - 5.0, kw + 16.0 + tw + 6.0, 28.0, (t[T_BG] & 0xFFFF_FF00) | 0xCC);
        e.slice(SLICE_LINE_R4, x, y, kw, 18.0, t[T_KEY_BORDER]);
        e.text_ex(x + kw * 0.5, y + 4.0, ROLE_LABEL, t[T_KEY_TEXT], ALIGN_CENTER, 0.0, "Y");
        e.text(x + kw + 8.0, y + 3.0, ROLE_VALUE, t[T_MUTED], HINT);
        e.submit();
    }

    fn draw(&mut self) {
        let mut b = [0f32; 6];
        unsafe { gd_ui_band_rect(b.as_mut_ptr()) };
        let sc = (b[5] / 720.0).min(b[4] / 1280.0).max(0.25);
        let vw = b[4] / sc;
        let t = self.tok;
        let (w, h) = (760.0f32, 600.0f32);
        let (x, y) = ((vw - w) * 0.5, 60.0);
        let e = &mut self.enc;
        e.clear();
        e.layer(LAYER_MODAL, 0).space(SPACE_WINDOW, sc);
        e.slice(SLICE_FILL_R6, x, y, w, h, (t[T_BG] & 0xFFFF_FF00) | 0xF2).slice(SLICE_LINE_R6, x, y, w, h, t[T_BORDER]);
        e.text(x + 20.0, y + 16.0, ROLE_TITLE, t[T_TEXT], "Brackets");
        e.text_ex(x + w - 20.0, y + 22.0, ROLE_LABEL, t[T_MUTED], ALIGN_RIGHT, 0.0, "B close");
        // the state record: fingerprint, linked, reachable, server, auto, event, message, error
        let bk = &self.bk;
        let reach = match bk.f(0, 2) { "1" => "reachable", "2" => "not reachable", _ => "checking..." };
        let rows: [(&str, &str); 4] = [
            ("Key", if bk.f(0, 0).is_empty() { "making the key..." } else { bk.f(0, 0) }),
            ("Server", bk.f(0, 3)),
            ("Status", reach),
            ("Account", if bk.f(0, 1) == "1" { "linked" } else { "not linked" }),
        ];
        let mut yy = y + 60.0;
        for (k, v) in rows {
            e.text(x + 20.0, yy, ROLE_VALUE, t[T_MUTED], k);
            e.text_ex(x + 140.0, yy, ROLE_VALUE | TEXT_MONO, t[T_TEXT], ALIGN_LEFT, w - 160.0, v);
            yy += 24.0;
        }
        yy += 6.0;
        e.rect(x + 16.0, yy, w - 32.0, 1.0, t[T_DIVIDER]);
        yy += 10.0;
        for (i, name) in ROWS.iter().enumerate() {
            let on = i == self.sel;
            if on {
                e.slice(SLICE_FILL_R4, x + 14.0, yy - 4.0, w - 28.0, 28.0, t[T_ACCENT]);
            }
            let (tx, mu) = if on { (t[T_INK], t[T_INK]) } else { (t[T_TEXT], t[T_MUTED]) };
            e.text(x + 24.0, yy + 1.0, ROLE_BODY, tx, name);
            let val: &str = match i {
                0 => if bk.f(0, 4) == "1" { "on: netplay games upload" } else { "off" },
                1 => {
                    if self.edit || self.n > 0 { utf8(&self.code[..self.n]) } else { "A to type the code" }
                }
                _ => "A",
            };
            e.text_ex(x + 220.0, yy + 1.0, ROLE_VALUE | TEXT_MONO, mu, ALIGN_LEFT, w - 240.0, val);
            if i == 1 && self.edit {
                let cx = x + 220.0 + measure(utf8(&self.code[..self.n]), ROLE_VALUE | TEXT_MONO) + 2.0;
                e.rect(cx, yy, 2.0, 18.0, tx);
            }
            yy += 30.0;
        }
        // the last request's result (STATUS), else the service's own message
        let (msg, err) = if self.st.n > 0 && !self.st.f(0, 1).is_empty() { (self.st.f(0, 1), self.st.f(0, 0) == "1") } else { (bk.f(0, 6), bk.f(0, 7) == "1") };
        e.text_ex(x + 20.0, yy + 4.0, ROLE_VALUE, if err { t[T_ACCENT] } else { t[T_MUTED] }, ALIGN_LEFT, w - 40.0, msg);
        yy += 34.0;
        e.rect(x + 16.0, yy, w - 32.0, 1.0, t[T_DIVIDER]);
        yy += 10.0;
        e.text(x + 20.0, yy, ROLE_LABEL, t[T_MUTED], "Upload queue");
        yy += 22.0;
        if self.q.n == 0 {
            e.text(x + 20.0, yy, ROLE_VALUE, t[T_MUTED], "Nothing queued.");
        }
        // records: file, state, attempts, event/set/game, reason (newest first)
        let left = ((y + h - 16.0 - yy) / 22.0) as usize;
        for i in 0..self.q.n.min(left) {
            let q = &self.q;
            e.text_ex(x + 20.0, yy, ROLE_VALUE | TEXT_MONO, t[T_TEXT], ALIGN_LEFT, 250.0, q.f(i, 0));
            e.text(x + 280.0, yy, ROLE_VALUE, if q.f(i, 1) == "failed" { t[T_ACCENT] } else { t[T_TEXT] }, q.f(i, 1));
            e.text_ex(x + 380.0, yy, ROLE_VALUE, t[T_MUTED], ALIGN_LEFT, w - 400.0, if q.f(i, 4).is_empty() { q.f(i, 3) } else { q.f(i, 4) });
            yy += 22.0;
        }
        e.submit();
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
    magic: 0x444D_4447, struct_size: 124, abi_min: 1 << 16, abi_max: 1 << 16, hook_table_version: 1, mod_version: 0x0001_0000,
    required_features: FEATURE_UI_DRAW | FEATURE_UI_INPUT | FEATURE_UI_HOST,
    optional_features: 0, state_bytes: 0, hook_count: 0,
    hooks: core::ptr::null(), mod_id: text(b"gd.bracket"), mod_name: text(b"Brackets"),
};
