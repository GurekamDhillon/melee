//! The Party pack sticker kit (envoy-v3-ui.md, menu/out_envoy/UI-SPEC.md section 3), drawn in band space (640 x 480
//! game pixels). Sticker faces, discs and dots are analytic shapes the host evaluates per pixel (exact at every
//! resolution, one quad each: `face`, `disc`, `spr`); the illustrated pieces (badges, bursts, pointers, flames ..)
//! are sprites baked at 4x by tools/rust-port/envoy/bake_stickers.py into `<mods>/envoy_ui/*.png`.

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
        // the faces, the disc, the ink dot and the stock dots are analytic shapes (`face`, `spr`): no bitmap
        if i <= S_FACE_HOVER || matches!(i, S_DISC | S_DOT_INK | S_STOCK | S_STOCK_LAST | S_STOCK_SPENT) {
            continue;
        }
        one(i, s.0);
    }
    one(S_BG_CAMP, "bg_camp");
    one(S_BG_ROAD, "bg_road");
    n + load_items()
}

// ---- the item sprites (envoy-models1.md): em_<item.key>.png per item, em_frame_<pool>.png behind it, the hearts
static mut H_ITEM: [u32; envoy_data::items::ITEM_IDS] = [0; envoy_data::items::ITEM_IDS];
/// devil, angel, secret, boss, shop, item (the aura's order, design 11.11)
static mut H_FRAME: [u32; 6] = [0; 6];
/// heart_container, heart_full
pub static mut H_HEART: [u32; 2] = [0; 2];

fn tex(key: &str) -> u32 {
    let mut name = [0u8; 48];
    let mut n = 0;
    for part in [b"em_".as_slice(), key.as_bytes(), b".png".as_slice()] {
        if n + part.len() > name.len() {
            return 0;
        }
        name[n..n + part.len()].copy_from_slice(part);
        n += part.len();
    }
    unsafe { gd_ui_tex_load(name.as_ptr(), n as u32) }
}

fn load_items() -> u32 {
    let mut n = 0;
    unsafe {
        for it in envoy_data::items::ITEMS.iter() {
            let h = tex(it.key);
            H_ITEM[it.id as usize] = h;
            n += (h != 0) as u32;
        }
        for (i, k) in ["frame_devil", "frame_angel", "frame_secret", "frame_boss", "frame_shop", "frame_item"].iter().enumerate() {
            H_FRAME[i] = tex(k);
        }
        H_HEART = [tex("heart_container"), tex("heart_full")];
    }
    n
}

/// The frame an item's pool takes (devil, angel, secret, then boss, shop, item).
fn frame_of(it: &envoy_data::items::Item) -> u32 {
    use envoy_data::items::*;
    let i = if it.pools & PL_DEVIL != 0 { 0 } else if it.pools & PL_ANGEL != 0 { 1 } else if it.pools & PL_SECRET != 0 { 2 } else if it.pools & PL_BOSS != 0 { 3 } else if it.pools & PL_SHOP != 0 { 4 } else { 5 };
    unsafe { H_FRAME[i] }
}

/// An item's icon centred on (cx, cy), `size` px: its pool frame behind at 4/3 the size, then its sprite; the
/// glyph icon (icons.rs) when the item has no sprite yet.
pub fn item_icon(e: &mut E, id: i32, cx: f32, cy: f32, size: f32, a: f32) {
    let Some(it) = envoy_data::items::item(id) else { return };
    let f = frame_of(it);
    if f != 0 {
        let w = size * 4.0 / 3.0;
        e.tex(f, cx - w * 0.5, cy - w * 0.5, w, w, [0.0, 0.0, 1.0, 1.0], alpha(WHITE, a));
    }
    let h = unsafe { H_ITEM.get(id as usize).copied().unwrap_or(0) };
    if h != 0 {
        e.tex(h, cx - size * 0.5, cy - size * 0.5, size, size, [0.0, 0.0, 1.0, 1.0], alpha(WHITE, a));
    } else {
        // no sprite yet: the slice-2 glyph icon (icons.rs)
        crate::icons::icon_a(e, id, cx, cy, size * if f != 0 { 0.8 } else { 1.0 }, a);
    }
}

/// A heart sprite (0 container, 1 full) centred, `size` px; false when not loaded (draw the old look).
pub fn heart(e: &mut E, full: bool, cx: f32, cy: f32, size: f32, rgba: u32) -> bool {
    let h = unsafe { H_HEART[full as usize] };
    if h != 0 {
        e.tex(h, cx - size * 0.5, cy - size * 0.5, size, size, [0.0, 0.0, 1.0, 1.0], rgba);
    }
    h != 0
}

pub fn ready() -> bool {
    unsafe { H[S_COIN] != 0 }
}

pub type E = Enc<32768>;

pub fn alpha(c: u32, a: f32) -> u32 {
    (c & 0xFFFF_FF00) | (((c & 0xFF) as f32 * a.clamp(0.0, 1.0)) as u32)
}

/// `c` tinted by `t` (per channel, alpha included), the way a texture tint multiplies a sprite.
pub fn mulc(c: u32, t: u32) -> u32 {
    if t == WHITE {
        return c;
    }
    let m = |sh: u32| ((((c >> sh) & 0xFF) * ((t >> sh) & 0xFF) + 127) / 255) << sh;
    m(24) | m(16) | m(8) | m(0)
}

/// A disc of diameter `d` centred on (cx, cy), with an edge `edge` wide in `line` inside its outline (0: none).
/// Analytic (uiquality): exact and round at every size and resolution.
pub fn disc(e: &mut E, cx: f32, cy: f32, d: f32, fill: u32, edge: f32, line: u32) {
    e.shape(SHAPE_BOX, cx - d * 0.5, cy - d * 0.5, d, d, d, edge, 0.0, fill, line);
}

/// A sprite at (x, y) with size (w, h) (0: its own size), tinted `rgba`. The round pieces (the disc, the ink dot,
/// the stock dots) are drawn as analytic shapes with the art's own proportions, not as bitmaps.
pub fn spr(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    let (w, h) = if w <= 0.0 && id < N_SPRITES { (SPRITES[id].1, SPRITES[id].2) } else { (w, h) };
    let (cx, cy) = (x + w * 0.5, y + h * 0.5);
    match id {
        // disc: r 15 in a 32 box
        S_DISC => return disc(e, cx, cy, w * (30.0 / 32.0), rgba, 0.0, 0),
        // ink dot: white r 6 with a 2.5 ink edge, in a 16 box
        S_DOT_INK => return disc(e, cx, cy, w * (14.5 / 16.0), rgba, w * (2.5 / 16.0), mulc(INK, rgba)),
        // stock dot: r 7 with a 2.5 ink edge and a hard ink shadow at (1.5, 2.5), in an 18 box
        S_STOCK | S_STOCK_LAST => {
            let k = w / 18.0;
            e.shadow(1.5 * k, 2.5 * k, 1.25 * k, INK);
            disc(e, cx, cy, 16.5 * k, mulc(if id == S_STOCK { WHITE } else { CORAL }, rgba), 2.5 * k, mulc(INK, rgba));
            e.shadow(0.0, 0.0, 0.0, 0);
            return;
        }
        // spent stock: ink r 4.2 with a 1.5 white edge
        S_STOCK_SPENT => {
            let k = w / 18.0;
            return disc(e, cx, cy, 9.9 * k, mulc(INK, rgba), 1.5 * k, rgba);
        }
        _ => {}
    }
    let hd = unsafe { H[id] };
    if hd == 0 {
        return;
    }
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

/// A sticker face over the face rectangle (x, y, w, h); the shadow falls outside it. `rgba` tints (alpha fades).
///
/// Analytic shapes (uiquality), in the proportions of the baked 48 px tiles they replace (bake_stickers.py): the face
/// (radius 10, a 3 ink edge centred on the rectangle) over a hard ink shadow offset (3, 4). One quad each, so a
/// sticker has no pieces to come apart, its corners are true arcs at every resolution, and a sticker lower than two
/// corners (a 12-unit pill) is a pill instead of two corner tiles pushed into each other. Small stickers take the
/// spec's lighter edge (UI-SPEC 3.3: 2.5 on small pills) and a shadow in proportion.
pub fn face(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    face_fx(e, id, x, y, w, h, rgba, [0; 2]);
}

const CREAM: u32 = 0xFBF6_DDFF;
const FADED: u32 = 0xDAD7_E8FF;

fn face_fx(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32, fx: [u32; 2]) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let m = if w < h { w } else { h };
    // 1 for a full sticker; down to 0.6 for the smallest chips
    let k = (m / 22.0).clamp(0.6, 1.0);
    let edge = if m < 22.0 { 2.5 } else { 3.0 };
    let ink = mulc(INK, rgba);
    let fx_on = |e: &mut E| {
        if fx[0] != 0 {
            e.effect(fx[0], fx[1]);
        }
    };
    let fx_off = |e: &mut E| {
        if fx[0] != 0 {
            e.effect(FX_NONE, 0);
        }
    };
    // Each sticker's shadow is part of its own quad (`Enc::shadow`): face and shadow are composed per pixel, so a
    // sticker that fades in is one translucent piece, never a face with its shadow showing through.
    match id {
        S_FACE_CREAM => {
            // the dashed seat: cream, a 2.5 dashed ink edge (5 on, 4 off), no shadow
            fx_on(e);
            e.shape(SHAPE_BOX, x - 1.25, y - 1.25, w + 2.5, h + 2.5, 11.25, 2.5, 5.0, mulc(CREAM, rgba), ink);
            fx_off(e);
        }
        S_FACE_HOVER => {
            // hovered: a white ring outside the sunny face's edge, shadow (4, 6) under both
            e.shadow(4.0 * k, 5.0 * k, 1.0, INK);
            e.shape(SHAPE_BOX, x - 1.0, y - 1.0, w, h, 12.0, 2.0, 0.0, rgba & !0xFF, rgba);
            e.shadow(0.0, 0.0, 0.0, 0);
            fx_on(e);
            e.shape(SHAPE_BOX, x, y, w - 2.0, h - 2.0, 10.5, edge, 0.0, mulc(SUNNY, rgba), ink);
            fx_off(e);
        }
        S_FACE_INK => {
            // the ink sticker: an ink rim, then a white line, then the ink face (rim and face meet under the line)
            let (inset, line) = ((m * 0.06).min(1.25), (m * 0.12).min(2.5));
            e.shadow(3.0 * k, 4.0 * k, 0.0, INK);
            e.shape(SHAPE_BOX, x, y, w, h, 10.0, inset + line * 0.5, 0.0, ink & !0xFF, ink);
            e.shadow(0.0, 0.0, 0.0, 0);
            fx_on(e);
            e.shape(SHAPE_BOX, x + inset, y + inset, w - 2.0 * inset, h - 2.0 * inset, 10.0 - inset, line, 0.0, ink, rgba);
            fx_off(e);
        }
        _ => {
            let fill = match id {
                S_FACE_SUNNY => SUNNY,
                S_FACE_CORAL => CORAL,
                S_FACE_LAGOON => LAGOON,
                S_FACE_FADED => FADED,
                S_FACE_LILAC => PALE,
                _ => WHITE,
            };
            // the shadow is the face rectangle itself (the outline less half the edge) moved by (3, 4)
            e.shadow(3.0 * k, 4.0 * k, edge * 0.5, INK);
            fx_on(e);
            e.shape(SHAPE_BOX, x - edge * 0.5, y - edge * 0.5, w + edge, h + edge, 10.0 + edge * 0.5, edge, 0.0, mulc(fill, rgba), ink);
            fx_off(e);
            e.shadow(0.0, 0.0, 0.0, 0);
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
    // the shimmer goes on the face alone, not on its shadow
    face_fx(e, id, x, y, w, h, rgba, fx);
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

/// Text with an ink edge (titles, percent): the host outlines the glyphs from their own distance field (one ink
/// pass under the fill, uiquality; it was eight offset ink copies, which smeared at high resolution).
pub fn text_edge(e: &mut E, x: f32, y: f32, role: i32, rgba: u32, align: i32, edge: f32, s: &str) {
    e.outline(edge, INK);
    e.text_ex(x, y, role, rgba, align, 0.0, s);
    e.outline(0.0, 0);
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
