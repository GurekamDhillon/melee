//! The Party pack sticker kit (envoy-v3-ui.md, menu/out_envoy/UI-SPEC.md section 3): sprites baked by
//! tools/rust-port/envoy/bake_stickers.py into `<mods>/envoy_ui/*.png`, drawn in band space (640 x 480 game pixels).

use gd_ui_enc::*;

pub const SUNNY: u32 = 0xFFD8_4AFF;
pub const INK: u32 = 0x2A1A_6EFF;
pub const WHITE: u32 = 0xFFFF_FFFF;
pub const CORAL: u32 = 0xFF4D_6DFF;
pub const LAGOON: u32 = 0x00A9_9DFF;
pub const LILAC: u32 = 0x5A4C_99FF;
pub const PALE: u32 = 0xC9BF_F5FF;
pub const DEEP: u32 = 0xC218_3FFF;
pub const VIOLET: u32 = 0x4C3D_9EFF;
pub const TRACK: u32 = 0x170E_45FF;
pub const HEAT_COL: [u32; 4] = [0xB9B2_DEFF, 0xFFBE_2EFF, 0xFF8A_1FFF, 0xFF4A_1CFF];
pub const PCOL: [u32; 2] = [CORAL, LAGOON];

/// Sprite names, in handle order (index = `S_*`), with their size in game pixels.
pub const SPRITES: [(&str, f32, f32); 75] = [
    ("face_white", 48.0, 48.0), ("face_sunny", 48.0, 48.0), ("face_coral", 48.0, 48.0), ("face_lagoon", 48.0, 48.0),
    ("face_faded", 48.0, 48.0), ("face_lilac", 48.0, 48.0), ("face_ink", 48.0, 48.0), ("face_cream", 48.0, 48.0),
    ("face_hover", 48.0, 48.0),
    ("disc", 32.0, 32.0), ("dot_ink", 16.0, 16.0), ("stock", 18.0, 18.0), ("stock_last", 18.0, 18.0), ("stock_spent", 18.0, 18.0),
    ("coin", 16.0, 16.0), ("star_on", 12.0, 12.0), ("star_off", 12.0, 12.0), ("spark", 14.0, 14.0), ("lock", 14.0, 14.0),
    ("burst_coral", 58.0, 58.0), ("burst_sunny", 58.0, 58.0), ("burst_lagoon", 58.0, 58.0), ("burst_white", 58.0, 58.0),
    ("burst16", 40.0, 40.0), ("rays", 200.0, 200.0),
    ("flame_cold", 24.0, 30.0), ("flame_warm", 24.0, 30.0), ("flame_hot", 24.0, 30.0), ("flame_blaze", 24.0, 30.0),
    ("ptr_coral", 28.0, 26.0), ("ptr_lagoon", 28.0, 26.0), ("ptrdown_coral", 26.0, 28.0), ("ptrdown_lagoon", 26.0, 28.0),
    ("dpad_down", 16.0, 16.0), ("dpad_right", 16.0, 16.0), ("dpad_right_lit", 16.0, 16.0), ("emblem", 30.0, 30.0),
    ("nodisc", 22.0, 22.0), ("swap", 22.0, 22.0), ("head", 22.0, 22.0),
    ("label_idle", 222.0, 38.0), ("label_on", 222.0, 38.0), ("label_miss", 222.0, 38.0),
    ("badge_0_0", 40.0, 40.0), ("badge_0_1", 40.0, 40.0), ("badge_0_2", 40.0, 40.0),
    ("badge_1_0", 40.0, 40.0), ("badge_1_1", 40.0, 40.0), ("badge_1_2", 40.0, 40.0),
    ("badge_2_0", 40.0, 40.0), ("badge_2_1", 40.0, 40.0), ("badge_2_2", 40.0, 40.0),
    ("badge_3_0", 40.0, 40.0), ("badge_3_1", 40.0, 40.0), ("badge_3_2", 40.0, 40.0),
    ("badge_4_0", 40.0, 40.0), ("badge_4_1", 40.0, 40.0), ("badge_4_2", 40.0, 40.0),
    ("badge_5_0", 40.0, 40.0), ("badge_5_1", 40.0, 40.0), ("badge_5_2", 40.0, 40.0),
    ("badge_6_0", 40.0, 40.0), ("badge_6_1", 40.0, 40.0), ("badge_6_2", 40.0, 40.0),
    ("badge_7_0", 40.0, 40.0), ("badge_7_1", 40.0, 40.0), ("badge_7_2", 40.0, 40.0),
    ("kem_0", 34.0, 34.0), ("kem_1", 34.0, 34.0), ("kem_2", 34.0, 34.0), ("kem_3", 34.0, 34.0), ("kem_4", 34.0, 34.0), ("kem_5", 34.0, 34.0),
    ("wedge_idle", 186.0, 276.0), ("wedge_on", 186.0, 276.0),
];
pub const S_FACE_WHITE: usize = 0;
pub const S_FACE_SUNNY: usize = 1;
pub const S_FACE_CORAL: usize = 2;
pub const S_FACE_LAGOON: usize = 3;
pub const S_FACE_FADED: usize = 4;
pub const S_FACE_LILAC: usize = 5;
pub const S_FACE_INK: usize = 6;
pub const S_FACE_CREAM: usize = 7;
pub const S_FACE_HOVER: usize = 8;
pub const S_DISC: usize = 9;
pub const S_DOT_INK: usize = 10;
pub const S_STOCK: usize = 11;
pub const S_STOCK_LAST: usize = 12;
pub const S_STOCK_SPENT: usize = 13;
pub const S_COIN: usize = 14;
pub const S_STAR_ON: usize = 15;
pub const S_STAR_OFF: usize = 16;
pub const S_SPARK: usize = 17;
pub const S_LOCK: usize = 18;
pub const S_BURST_CORAL: usize = 19;
pub const S_BURST_SUNNY: usize = 20;
pub const S_BURST_LAGOON: usize = 21;
pub const S_BURST_WHITE: usize = 22;
pub const S_BURST16: usize = 23;
pub const S_RAYS: usize = 24;
pub const S_FLAME: usize = 25; // + state 0..3
pub const S_PTR: usize = 29; // + player
pub const S_PTRDOWN: usize = 31; // + player
pub const S_DPAD_DOWN: usize = 33;
pub const S_DPAD_RIGHT: usize = 34;
pub const S_DPAD_RIGHT_LIT: usize = 35;
pub const S_EMBLEM: usize = 36;
pub const S_NODISC: usize = 37;
pub const S_SWAP: usize = 38;
pub const S_HEAD: usize = 39;
pub const S_LABEL: usize = 40; // + idle / on / miss
pub const S_BADGE: usize = 43; // + family * 3 + tier
pub const S_KEM: usize = 67; // + emblem
pub const S_WEDGE: usize = 73; // + idle / on
pub const N_SPRITES: usize = 75;
pub const S_BG_CAMP: usize = 75;
pub const S_BG_ROAD: usize = 76;

static mut H: [u32; N_SPRITES + 2] = [0; N_SPRITES + 2];

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_tex_load(s: *const u8, len: u32) -> u32;
}

/// Envoy model sprites (`em_<prefix><key>.png`, docs/rust-port/notes/envoy-models1.md), loaded on first use: up to
/// EM_MAX distinct ones a session. A sprite is told by its two strings' addresses (table strings: `Synergy::key`,
/// a literal), so nothing is compared or copied per frame. Handle 0 after a failed load (not retried).
const EM_MAX: usize = 48;
static mut EM_TEX: [(usize, usize, u32); EM_MAX] = [(0, 0, 0); EM_MAX];
static mut EM_N: usize = 0;

pub fn em_tex(prefix: &'static str, key: &'static str) -> u32 {
    let id = (prefix.as_ptr() as usize, key.as_ptr() as usize);
    unsafe {
        let t = &mut *core::ptr::addr_of_mut!(EM_TEX);
        for e in t.iter().take(EM_N) {
            if (e.0, e.1) == id {
                return e.2;
            }
        }
        if EM_N >= EM_MAX {
            return 0;
        }
        let mut name = [0u8; 72];
        let mut n = 0;
        for part in ["em_", prefix, key, ".png"] {
            for &c in part.as_bytes() {
                if n < name.len() {
                    name[n] = c;
                    n += 1;
                }
            }
        }
        let h = gd_ui_tex_load(name.as_ptr(), n as u32);
        t[EM_N] = (id.0, id.1, h);
        EM_N += 1;
        h
    }
}

/// The sprite of Envoy model `prefix` + `key` centred on (cx, cy), `size` across (band units), tinted `rgba`.
/// False when the sprite is not there (the caller keeps what it drew before).
pub fn em_spr(e: &mut E, prefix: &'static str, key: &'static str, cx: f32, cy: f32, size: f32, rgba: u32) -> bool {
    let h = em_tex(prefix, key);
    if h == 0 {
        return false;
    }
    e.tex(h, cx - size * 0.5, cy - size * 0.5, size, size, [0.0, 0.0, 1.0, 1.0], rgba);
    true
}

/// Frame `fr` of the animated strip `em_<key>.png` (`n` cells side by side; `key` ends in `_anim`).
pub fn em_anim(e: &mut E, key: &'static str, n: u32, fr: u32, cx: f32, cy: f32, size: f32, rgba: u32) -> bool {
    let h = em_tex("", key);
    if h == 0 || n == 0 {
        return false;
    }
    let f = (fr % n) as f32;
    e.tex(h, cx - size * 0.5, cy - size * 0.5, size, size, [f / n as f32, 0.0, (f + 1.0) / n as f32, 1.0], rgba);
    true
}

/// Load every sprite (call once from gd_ui_init). Returns how many loaded.
pub fn load() -> u32 {
    let mut n = 0;
    let mut name = [0u8; 40];
    let mut one = |i: usize, s: &str| {
        let b = s.as_bytes();
        name[..b.len()].copy_from_slice(b);
        name[b.len()..b.len() + 4].copy_from_slice(b".png");
        let h = unsafe { gd_ui_tex_load(name.as_ptr(), b.len() as u32 + 4) };
        unsafe { H[i] = h };
        if h != 0 {
            n += 1;
        }
    };
    for (i, s) in SPRITES.iter().enumerate() {
        one(i, s.0);
    }
    one(S_BG_CAMP, "bg_camp");
    one(S_BG_ROAD, "bg_road");
    n
}

pub fn ready() -> bool {
    unsafe { H[S_FACE_WHITE] != 0 }
}

pub type E = Enc<32768>;

pub fn alpha(c: u32, a: f32) -> u32 {
    (c & 0xFFFF_FF00) | (((c & 0xFF) as f32 * a.clamp(0.0, 1.0)) as u32)
}

/// A sprite at (x, y) with size (w, h) (0: its own size), tinted `rgba`.
pub fn spr(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    let hd = unsafe { H[id] };
    if hd == 0 {
        return;
    }
    let (w, h) = if w <= 0.0 && id < N_SPRITES { (SPRITES[id].1, SPRITES[id].2) } else { (w, h) };
    e.tex(hd, x, y, w, h, [0.0, 0.0, 1.0, 1.0], rgba);
}

/// A sprite centred on (cx, cy) at `scale`.
pub fn spr_c(e: &mut E, id: usize, cx: f32, cy: f32, scale: f32, rgba: u32) {
    let (w, h) = (SPRITES[id].1 * scale, SPRITES[id].2 * scale);
    spr(e, id, cx - w * 0.5, cy - h * 0.5, w, h, rgba);
}

/// A sprite rotated `deg` about its centre.
pub fn spr_rot(e: &mut E, id: usize, cx: f32, cy: f32, scale: f32, deg: f32, rgba: u32) {
    rot(e, deg, cx, cy);
    spr_c(e, id, cx, cy, scale, rgba);
    unrot(e);
}

pub fn rot(e: &mut E, deg: f32, cx: f32, cy: f32) {
    e.rot(deg * 0.017_453_3, cx, cy);
}
pub fn unrot(e: &mut E) {
    e.rot(0.0, 0.0, 0.0);
}

/// A sticker face (9-slice of a 48 px tile, cuts 14 / 34) over the face rectangle (x, y, w, h); the shadow falls
/// outside it. `rgba` tints (alpha fades).
pub fn face(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    let hd = unsafe { H[id] };
    if hd == 0 {
        return;
    }
    // the tile's face starts 1.5 in: the sticker's outer edge sits on (x, y)
    let (x, y) = (x - 1.5, y - 1.5);
    let (w, h) = (w + 6.0, h + 6.0);
    let (a, b) = (14.0f32, 14.0f32);
    let xs = [x, x + a, x + w - b, x + w];
    let ys = [y, y + a, y + h - b, y + h];
    let us = [0.0, a / 48.0, 1.0 - b / 48.0, 1.0];
    for j in 0..3 {
        for i in 0..3 {
            let (qw, qh) = (xs[i + 1] - xs[i], ys[j + 1] - ys[j]);
            if qw <= 0.0 || qh <= 0.0 {
                continue;
            }
            e.tex(hd, xs[i], ys[j], qw, qh, [us[i], us[j], us[i + 1], us[j + 1]], rgba);
        }
    }
}

/// Host effects (envoy-fx.md): the next sticker's face gets `FACE_FX` (foil / holo), the hovered face a glow.
static mut FACE_FX: [u32; 2] = [0; 2];
/// The next `sticker` face shimmers: FX_FOIL (Epic) or FX_HOLO (Unique), in `rgba`.
pub fn face_fx_next(id: u32, rgba: u32) {
    unsafe { FACE_FX = [id, rgba] };
}

/// A sticker rotated `deg` about its centre: the face, then whatever `inner` draws (in the same rotation).
pub fn sticker(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, deg: f32, rgba: u32, inner: impl FnOnce(&mut E)) {
    rot(e, deg, x + w * 0.5, y + h * 0.5);
    let fx = unsafe { core::mem::take(&mut *core::ptr::addr_of_mut!(FACE_FX)) };
    if id == S_FACE_HOVER {
        // the cursor's soft halo behind the hovered sticker (the host's pre-blurred glow, breathing)
        e.effect(FX_GLOW, alpha(SUNNY, (rgba & 0xFF) as f32 / 255.0 * 0.85));
        e.slice(SLICE_GLOW_R6, x - 2.0, y - 2.0, w + 7.0, h + 7.0, WHITE);
        e.effect(FX_NONE, 0);
    }
    if fx[0] != 0 {
        e.effect(fx[0], fx[1]);
    }
    face(e, id, x, y, w, h, rgba);
    if fx[0] != 0 {
        e.effect(FX_NONE, 0);
    }
    inner(e);
    unrot(e);
}

/// Text faces: Bricolage ExtraBold (`D`) and Source Sans 3 Bold (`B`), `px` the line size.
pub const fn d(px: u32) -> i32 {
    ROLE_BODY | TEXT_DISPLAY | text_size(px)
}
pub const fn bs(px: u32) -> i32 {
    ROLE_BODY | TEXT_BODY | text_size(px)
}

pub fn text(e: &mut E, x: f32, y: f32, role: i32, rgba: u32, align: i32, s: &str) {
    e.text_ex(x, y, role, rgba, align, 0.0, s);
}

/// Text with an ink edge (titles, percent): eight ink copies around the fill.
pub fn text_edge(e: &mut E, x: f32, y: f32, role: i32, rgba: u32, align: i32, edge: f32, s: &str) {
    let k = edge * 0.7071;
    for (dx, dy) in [(edge, 0.0), (-edge, 0.0), (0.0, edge), (0.0, -edge), (k, k), (-k, k), (k, -k), (-k, -k)] {
        e.text_ex(x + dx, y + dy, role, INK, align, 0.0, s);
    }
    e.text_ex(x, y, role, rgba, align, 0.0, s);
}

/// Text width.
pub fn tw(s: &str, role: i32) -> f32 {
    unsafe { gd_ui_measure(s.as_ptr(), s.len() as u32, role) }
}

/// The strip: one white sticker at (x, 403), 30 high, one sentence (UI-SPEC 3.8).
pub fn strip(e: &mut E, x: f32, max_w: f32, s: &str) {
    if s.is_empty() {
        return;
    }
    let w = (tw(s, bs(13)) + 22.0).min(max_w);
    sticker(e, S_FACE_WHITE, x, 403.0, w, 30.0, -0.6, WHITE, |e| {
        e.text_ex(x + 11.0, 403.0 + 8.0, bs(13), INK, ALIGN_LEFT, w - 22.0, s);
    });
}

/// The hint line at (26, 449): (cap, words) pairs (UI-SPEC 3.8).
pub fn hints(e: &mut E, items: &[(&str, &str)]) {
    let mut x = 26.0;
    for &(cap, words) in items {
        let cw = tw(cap, d(11)).max(8.0) + 12.0;
        face(e, S_FACE_WHITE, x, 449.0, cw, 18.0, WHITE);
        e.text_ex(x + cw * 0.5, 449.0 + 3.0, d(11), INK, ALIGN_CENTER, 0.0, cap);
        x += cw + 6.0;
        e.text_ex(x, 449.0 + 2.0, bs(13), WHITE, ALIGN_LEFT, 0.0, words);
        x += tw(words, bs(13)) + 16.0;
    }
}

/// Ease out (fast start) 0..1.
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// "A sticker enters" (UI-SPEC 4.1): scale 0.85 -> 1.04 -> 1.0 over 6 + 3 frames; (scale, opacity) at age `t`.
pub fn enter(t: i32) -> (f32, f32) {
    if t <= 0 {
        return (0.85, 0.0);
    }
    if t < 6 {
        let k = ease(t as f32 / 6.0);
        return (0.85 + 0.19 * k, k);
    }
    if t < 9 {
        return (1.04 - 0.04 * (t - 6) as f32 / 3.0, 1.0);
    }
    (1.0, 1.0)
}

/// A burst (16-point star) with words, popping in: scale 0 -> 1.2 -> 1.0 over 4 + 3 frames, tilted 8 degrees.
pub fn burst(e: &mut E, id: usize, cx: f32, cy: f32, size: f32, t: i32, words: &str, fs: u32) {
    let s = if t < 4 { 1.2 * t as f32 / 4.0 } else if t < 7 { 1.2 - 0.2 * (t - 4) as f32 / 3.0 } else { 1.0 };
    if s <= 0.0 {
        return;
    }
    rot(e, 8.0, cx, cy);
    spr_c(e, id, cx, cy, s * size / 58.0, WHITE);
    if s > 0.6 && !words.is_empty() {
        text_edge(e, cx, cy - fs as f32 * 0.55, d(fs), WHITE, ALIGN_CENTER, 1.5, words);
    }
    unrot(e);
}

/// A charge ring (r, width) clockwise from the top to `frac`, as dots (UI-SPEC 6.5).
pub fn ring(e: &mut E, cx: f32, cy: f32, r: f32, wd: f32, frac: f32, on: u32, off: u32) {
    let n = 28;
    for i in 0..n {
        let a = (i as f32 + 0.5) / n as f32;
        let ang = a * 6.283_185 - 1.570_796;
        let (x, y) = (cx + r * cos(ang), cy + r * sin(ang));
        spr(e, S_DISC, x - wd * 0.5, y - wd * 0.5, wd, wd, if a <= frac { on } else { off });
    }
}

pub fn sin(x: f32) -> f32 {
    // range-reduced Taylor (presentation only)
    let tau = 6.283_185_5f32;
    let mut x = x % tau;
    if x > 3.141_592_7 {
        x -= tau;
    } else if x < -3.141_592_7 {
        x += tau;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}
pub fn cos(x: f32) -> f32 {
    sin(x + 1.570_796_3)
}

/// A card's text in the item keyword language (envoy_data::keywords): plain words in `body` / `rgba`, keywords in
/// the display face in their category's colour (`dark`: the palette for an ink sticker). Wraps at `w`, `lh` a line,
/// at most `max_lines`. Returns the lines used.
#[allow(clippy::too_many_arguments)]
pub fn rich(e: &mut E, x: f32, y: f32, w: f32, px: u32, lh: f32, rgba: u32, dark: bool, s: &str, max_lines: i32) -> i32 {
    use envoy_data::keywords::*;
    let body = bs(px);
    let key = d(px);
    let (mut cx, mut line) = (x, 0);
    let space = tw(" ", body).max(px as f32 * 0.25);
    let mut put = |e: &mut E, word: &str, role: i32, col: u32, cx: &mut f32, line: &mut i32| {
        let ww = tw(word, role);
        if *cx > x && *cx + ww > x + w {
            *line += 1;
            *cx = x;
        }
        if *line < max_lines {
            e.text_ex(*cx, y + *line as f32 * lh, role, col, ALIGN_LEFT, 0.0, word);
        }
        *cx += ww;
    };
    for sp in spans(s) {
        match sp {
            Span::Key(k) => {
                let kw = &KEYWORDS[k];
                let col = if dark { KC_DARK[kw.cat as usize] } else { KC_LIGHT[kw.cat as usize] };
                put(e, kw.word, key, alpha(col, (rgba & 0xFF) as f32 / 255.0), &mut cx, &mut line);
            }
            Span::Text(t) => {
                let mut first = true;
                for word in t.split(' ') {
                    if !first {
                        cx += space;
                    }
                    first = false;
                    if !word.is_empty() {
                        put(e, word, body, rgba, &mut cx, &mut line);
                    }
                }
            }
        }
    }
    (line + 1).min(max_lines)
}
