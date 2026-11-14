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
    fn gd_ui_tex_size(h: u32) -> i32;
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

/// The part of a pool frame's cell its opening takes (the frame art's border is about a sixth of the cell on every
/// side) and the part of an item's cell its drawing fills: an item sits inside its frame at `FRAME_FIT` of the
/// frame's cell.
const FRAME_FIT: f32 = 0.6;

/// An item's icon centred on (cx, cy): its pool frame over a cell of 4/3 `size`, and its sprite inside the frame's
/// opening (uiremake: the sprite was drawn at `size`, 3/4 of the frame's cell, while the opening is about 2/3 of it
/// and the sprites fill their own cells, so every item ran over its frame's border). The glyph icon (icons.rs)
/// when the item has no sprite yet. Icons in a row need a step of `size` * 1.2 or more to stand clear of each other
/// (`item_step`).
pub fn item_icon(e: &mut E, id: i32, cx: f32, cy: f32, size: f32, a: f32) {
    let Some(it) = envoy_data::items::item(id) else { return };
    let f = frame_of(it);
    let w = size * 4.0 / 3.0;
    if f != 0 {
        e.tex(f, cx - w * 0.5, cy - w * 0.5, w, w, [0.0, 0.0, 1.0, 1.0], alpha(WHITE, a));
    }
    let s = if f != 0 { w * FRAME_FIT } else { size };
    let h = unsafe { H_ITEM.get(id as usize).copied().unwrap_or(0) };
    if h != 0 {
        e.tex(h, cx - s * 0.5, cy - s * 0.5, s, s, [0.0, 0.0, 1.0, 1.0], alpha(WHITE, a));
    } else {
        // no sprite yet: the slice-2 glyph icon (icons.rs)
        crate::icons::icon_a(e, id, cx, cy, s, a);
    }
}

/// The step between `item_icon`s of `size` in a row: the frames (visible over 5/6 of their 4/3 cell) stand 2 apart.
pub fn item_step(size: f32) -> f32 {
    size * (4.0 / 3.0) * (5.0 / 6.0) + 2.0
}

/// A stack count under an icon centred on (cx, cy) of `size` ("x2"): an ink pill as wide as its words, centred on
/// the icon's foot (it was a 16 x 11 chip, a squashed oval, or loose digits over the next icon).
pub fn count_chip(e: &mut E, cx: f32, cy: f32, size: f32, n: i32) {
    let mut b = [b'x', 0, 0, 0];
    let n = n.clamp(0, 99) as u8;
    let len = if n >= 10 {
        b[1] = b'0' + n / 10;
        b[2] = b'0' + n % 10;
        3
    } else {
        b[1] = b'0' + n;
        2
    };
    let s = unsafe { core::str::from_utf8_unchecked(&b[..len]) };
    let w = pill_w(11.0, 3.5, d(8), s);
    pill(e, S_FACE_INK, cx - w * 0.5, cy + size * 0.5 - 5.0, 11.0, 3.5, d, 8, SUNNY, WHITE, s);
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

/// The largest rectangle of proportions `aw : ah` inside the box (x, y, w, h), centred: (x, y, w, h). Art placed
/// with this is never stretched: a box of another shape only leaves room beside it (uiremake; ui_aspect checks it).
pub fn contain(x: f32, y: f32, w: f32, h: f32, aw: f32, ah: f32) -> (f32, f32, f32, f32) {
    if aw <= 0.0 || ah <= 0.0 || w <= 0.0 || h <= 0.0 {
        return (x, y, w, h);
    }
    let k = (w / aw).min(h / ah);
    let (fw, fh) = (aw * k, ah * k);
    (x + (w - fw) * 0.5, y + (h - fh) * 0.5, fw, fh)
}

/// A whole texture of `aw : ah` art (any units) contained in the box (x, y, w, h), centred, never stretched.
#[allow(clippy::too_many_arguments)]
pub fn tex_fit(e: &mut E, h: u32, aw: f32, ah: f32, x: f32, y: f32, w: f32, hh: f32, rgba: u32) {
    if h == 0 {
        return;
    }
    let (fx, fy, fw, fh) = contain(x, y, w, hh, aw, ah);
    e.tex(h, fx, fy, fw, fh, [0.0, 0.0, 1.0, 1.0], rgba);
}

/// A whole texture contained in the box (x, y, w, h) at the proportions of its own texels (`gd_ui_tex_size`): for
/// art whose size the mod does not carry (the disc's fighter icons and portraits).
pub fn tex_in(e: &mut E, h: u32, x: f32, y: f32, w: f32, hh: f32, rgba: u32) {
    if h == 0 {
        return;
    }
    let sz = unsafe { gd_ui_tex_size(h) };
    let (aw, ah) = if sz > 0 { ((sz >> 16) as f32, (sz & 0xFFFF) as f32) } else { (w, hh) };
    tex_fit(e, h, aw, ah, x, y, w, hh, rgba);
}

/// A sprite in the box (x, y, w, h) (w 0: its own size at x, y), tinted `rgba`. A kit sprite keeps its art's
/// proportions whatever the box (it is contained and centred, `contain`); only the backgrounds fill their box. The
/// round pieces (the disc, the ink dot, the stock dots) are analytic shapes with the art's own proportions.
pub fn spr(e: &mut E, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
    let (x, y, w, h) = if id >= N_SPRITES {
        (x, y, w, h)
    } else if w <= 0.0 {
        (x, y, SPRITES[id].1, SPRITES[id].2)
    } else {
        contain(x, y, w, h, SPRITES[id].1, SPRITES[id].2)
    };
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

/// The rotation in force (radians, pivot): the host keeps one per pen, so `seg` composes its own with it.
static mut ROT: (f32, f32, f32) = (0.0, 0.0, 0.0);

pub fn rot(e: &mut E, deg: f32, cx: f32, cy: f32) {
    unsafe { ROT = (deg * 0.017_453_3, cx, cy) };
    e.rot(deg * 0.017_453_3, cx, cy);
}
pub fn unrot(e: &mut E) {
    unsafe { ROT = (0.0, 0.0, 0.0) };
    e.rot(0.0, 0.0, 0.0);
}

/// atan2 (no_std, presentation only): a minimax polynomial, about 0.0002 rad off.
pub fn atan2(y: f32, x: f32) -> f32 {
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }
    let (ax, ay) = (if x < 0.0 { -x } else { x }, if y < 0.0 { -y } else { y });
    let a = if ax > ay { ay / ax } else { ax / ay };
    let s = a * a;
    let mut r = ((-0.046_496_47 * s + 0.159_314_22) * s - 0.327_622_76) * s * a + a;
    if ay > ax {
        r = 1.570_796_3 - r;
    }
    if x < 0.0 {
        r = 3.141_592_7 - r;
    }
    if y < 0.0 { -r } else { r }
}

/// A round-capped line from (x0, y0) to (x1, y1), `w` thick: one analytic pill quad turned onto the segment, under
/// whatever sticker tilt is in force (the turn is composed with it and the tilt put back). One 11-word record where
/// the old chain of discs took up to 65 (the act map's roads alone used to fill the frame's command buffer, and
/// the records that no longer fit were dropped mid-frame: missing icons, half lines, a shadow or tilt left on).
pub fn seg(e: &mut E, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, c: u32) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = crate::icons::sqrt(dx * dx + dy * dy);
    let (mut cx, mut cy) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
    let (oa, px, py) = unsafe { ROT };
    if oa != 0.0 {
        // the pill's centre under the outer turn; the pill then turns by both angles about it
        let (cs, sn) = (cos(oa), sin(oa));
        let (ddx, ddy) = (cx - px, cy - py);
        cx = px + ddx * cs - ddy * sn;
        cy = py + ddx * sn + ddy * cs;
    }
    let a = oa + atan2(dy, dx);
    if a != 0.0 {
        e.rot(a, cx, cy);
    }
    e.shape(SHAPE_BOX, cx - (len + w) * 0.5, cy - w * 0.5, len + w, w, w * 0.5, 0.0, 0.0, c, 0);
    if a != 0.0 || oa != 0.0 {
        e.rot(oa, px, py);
    }
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

/// The largest size from `px` down to `min` at which `s` is no wider than `w` in the face `role` (`d` or `bs`).
/// Words are never squeezed or cut to a width (uiremake): a line that is too long is set smaller, down to a floor
/// that still reads, and past that the caller wraps it or gives it more room.
pub fn fit_px(s: &str, role: fn(u32) -> i32, px: u32, min: u32, w: f32) -> u32 {
    let mut p = px;
    while p > min && tw(s, role(p)) > w {
        p -= 1;
    }
    p
}

/// `role` with its size taken down (to two thirds at most, never under 7) until `s` is no wider than `w`.
pub fn fit_role(s: &str, role: i32, w: f32) -> i32 {
    let px = ((role >> 16) & 0xFF) as u32;
    if px == 0 {
        return role;
    }
    let min = (px * 2 / 3).max(7).min(px);
    let mut p = px;
    let at = |p: u32| (role & !0x00FF_0000) | text_size(p);
    while p > min && tw(s, at(p)) > w {
        p -= 1;
    }
    at(p)
}

/// One whole line, at `px` if it fits `w`, else smaller (not under `min`); never cut. `y` is the line top at `px`:
/// a smaller size is moved down so the words keep their baseline. Returns the width drawn.
#[allow(clippy::too_many_arguments)]
pub fn text_fit(e: &mut E, x: f32, y: f32, role: fn(u32) -> i32, px: u32, min: u32, rgba: u32, align: i32, w: f32, s: &str) -> f32 {
    let p = fit_px(s, role, px, min.min(px), w);
    e.text_ex(x, y + (px - p) as f32 * 0.8, role(p), rgba, align, 0.0, s);
    tw(s, role(p))
}

/// Where the first line of `s` ends when lines are `w` wide in `role`: the byte offset of the break (the whole
/// string when it fits; a single word wider than the line stays whole).
fn line_end(s: &str, w: f32, role: i32) -> usize {
    if tw(s, role) <= w {
        return s.len();
    }
    let mut cut = 0;
    for (i, c) in s.char_indices() {
        if c == ' ' {
            if tw(&s[..i], role) <= w {
                cut = i;
            } else {
                break;
            }
        }
    }
    if cut == 0 { s.find(' ').unwrap_or(s.len()) } else { cut }
}

/// Whole words over as many lines as they need, at most `max_lines` (the last line takes all that is left, set
/// smaller if it must): lines `w` wide, `lh` apart, in `role(px)`. Returns the lines used. Nothing is cut.
#[allow(clippy::too_many_arguments)]
pub fn wrap(e: &mut E, x: f32, y: f32, w: f32, role: fn(u32) -> i32, px: u32, lh: f32, rgba: u32, align: i32, s: &str, max_lines: usize) -> usize {
    let mut rest = s.trim();
    let mut line = 0;
    let max = max_lines.max(1);
    while !rest.is_empty() && line < max {
        let cut = if line + 1 == max { rest.len() } else { line_end(rest, w, role(px)) };
        text_fit(e, x, y + line as f32 * lh, role, px, (px * 2 / 3).max(7), rgba, align, w, &rest[..cut]);
        rest = rest[cut..].trim_start();
        line += 1;
    }
    line
}

/// The lines `s` needs at width `w` in `role` (what `wrap` uses, given enough lines).
pub fn wrap_lines(s: &str, w: f32, role: i32) -> usize {
    let mut rest = s.trim();
    let mut n = 0;
    while !rest.is_empty() {
        let cut = line_end(rest, w, role);
        rest = rest[cut..].trim_start();
        n += 1;
    }
    n
}

/// The width `pill` gives `s`: its words plus `pad` each side, never narrower than `h`. A pill between one and
/// 1.25 circles wide reads as a squashed circle, so it is made a circle or a real pill.
pub fn pill_w(h: f32, pad: f32, role: i32, s: &str) -> f32 {
    let w = tw(s, role) + 2.0 * pad;
    if w <= h { h } else if w < h * 1.3 { h * 1.3 } else { w }
}

/// A pill sized from its words: the face `id` at (x, y), `h` high (`pill_w` wide), the words centred in
/// `role(px)`. Returns its width.
#[allow(clippy::too_many_arguments)]
pub fn pill(e: &mut E, id: usize, x: f32, y: f32, h: f32, pad: f32, role: fn(u32) -> i32, px: u32, ink: u32, rgba: u32, s: &str) -> f32 {
    let w = pill_w(h, pad, role(px), s);
    face(e, id, x, y, w, h, rgba);
    e.text_ex(x + w * 0.5, y + (h - px as f32 * 1.25) * 0.5, role(px), ink, ALIGN_CENTER, 0.0, s);
    w
}

/// A tab row over the width `w` from (x, y): every tab as wide as its words need, the room left over shared out
/// evenly; when the words do not fit one row they go on two (never cut, never squeezed). `on`: the open tab,
/// `hover`: the cursor is on the row. Returns the height used.
#[allow(clippy::too_many_arguments)]
pub fn tab_row(e: &mut E, x: f32, y: f32, w: f32, th: f32, px: u32, names: &[&str], on: usize, hover: bool) -> f32 {
    let n = names.len();
    if n == 0 {
        return 0.0;
    }
    let (gap, pad) = (5.0, 7.0);
    let nat = |a: usize, b: usize| names[a..b].iter().map(|s| tw(s, d(px)) + 2.0 * pad).sum::<f32>() + gap * (b - a - 1) as f32;
    let one = [(0, n)];
    let two = [(0, n.div_ceil(2)), (n.div_ceil(2), n)];
    let rows: &[(usize, usize)] = if nat(0, n) <= w || n < 2 { &one } else { &two };
    for (r, &(a, b)) in rows.iter().enumerate() {
        let extra = ((w - nat(a, b)) / (b - a) as f32).max(0.0);
        let mut tx = x;
        for (i, name) in names.iter().enumerate().take(b).skip(a) {
            let tw_ = tw(name, d(px)) + 2.0 * pad + extra;
            let sel = i == on;
            let id = if sel && hover { S_FACE_HOVER } else if sel { S_FACE_SUNNY } else { S_FACE_LILAC };
            let ty = y + r as f32 * (th + 6.0) - if sel { 2.0 } else { 0.0 };
            face(e, id, tx, ty, tw_, th, WHITE);
            e.text_ex(tx + tw_ * 0.5, ty + (th - px as f32 * 1.25) * 0.5, d(px), if sel { INK } else { WHITE }, ALIGN_CENTER, 0.0, name);
            tx += tw_ + gap;
        }
    }
    tab_h(rows.len(), th)
}

/// The height of `rows` tab rows `th` high.
pub fn tab_h(rows: usize, th: f32) -> f32 {
    rows as f32 * (th + 6.0)
}

/// How many rows `tab_row` needs for `names` at width `w`.
pub fn tab_rows(w: f32, px: u32, names: &[&str]) -> usize {
    let nat = names.iter().map(|s| tw(s, d(px)) + 14.0).sum::<f32>() + 5.0 * (names.len().max(1) - 1) as f32;
    if nat <= w || names.len() < 2 { 1 } else { 2 }
}

/// The strip: one white sticker at (x, 403), 30 high, one sentence (UI-SPEC 3.8).
pub fn strip(e: &mut E, x: f32, max_w: f32, s: &str) {
    if s.is_empty() {
        return;
    }
    let w = (tw(s, bs(13)) + 22.0).min(max_w);
    sticker(e, S_FACE_WHITE, x, 403.0, w, 30.0, -0.6, WHITE, |e| {
        text_fit(e, x + 11.0, 403.0 + 8.0, bs, 13, 8, INK, ALIGN_LEFT, w - 22.0, s);
    });
}

/// The hint line at (26, 449): (cap, words) pairs (UI-SPEC 3.8).
pub fn hints(e: &mut E, items: &[(&str, &str)]) {
    let mut x = 26.0;
    for &(cap, words) in items {
        // a one-letter cap is a circle, a word a pill (a 20 x 18 "A" cap read as a squashed circle)
        let cw = pill_w(18.0, 6.0, d(11), cap);
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
/// at most `max_lines`. Returns the lines used. A card sizes itself with `rich_lines` first, so no line is lost.
#[allow(clippy::too_many_arguments)]
pub fn rich(e: &mut E, x: f32, y: f32, w: f32, px: u32, lh: f32, rgba: u32, dark: bool, s: &str, max_lines: i32) -> i32 {
    rich_run(Some(e), x, y, w, px, lh, rgba, dark, s, max_lines).min(max_lines)
}

/// The lines `rich` needs for `s` at width `w` and size `px`.
pub fn rich_lines(w: f32, px: u32, s: &str) -> i32 {
    rich_run(None, 0.0, 0.0, w, px, 0.0, 0, false, s, 0)
}

#[allow(clippy::too_many_arguments)]
fn rich_run(mut e: Option<&mut E>, x: f32, y: f32, w: f32, px: u32, lh: f32, rgba: u32, dark: bool, s: &str, max_lines: i32) -> i32 {
    use envoy_data::keywords::*;
    let body = bs(px);
    let key = d(px);
    let (mut cx, mut line) = (x, 0);
    let space = tw(" ", body).max(px as f32 * 0.25);
    let mut put = |e: &mut Option<&mut E>, word: &str, role: i32, col: u32, cx: &mut f32, line: &mut i32| {
        let ww = tw(word, role);
        // a piece that opens with punctuation (": more orbs." after a keyword) stays on its keyword's line
        if *cx > x && *cx + ww > x + w && !word.starts_with([':', ',', '.', ';', ')']) {
            *line += 1;
            *cx = x;
        }
        if let Some(e) = e.as_deref_mut().filter(|_| *line < max_lines) {
            e.text_ex(*cx, y + *line as f32 * lh, role, col, ALIGN_LEFT, 0.0, word);
        }
        *cx += ww;
    };
    for sp in spans(s) {
        match sp {
            Span::Key(k) => {
                let kw = &KEYWORDS[k];
                let col = if dark { KC_DARK[kw.cat as usize] } else { KC_LIGHT[kw.cat as usize] };
                put(&mut e, kw.word, key, alpha(col, (rgba & 0xFF) as f32 / 255.0), &mut cx, &mut line);
            }
            Span::Text(t) => {
                let mut first = true;
                for word in t.split(' ') {
                    if !first {
                        cx += space;
                    }
                    first = false;
                    if !word.is_empty() {
                        put(&mut e, word, body, rgba, &mut cx, &mut line);
                    }
                }
            }
        }
    }
    line + 1
}

// ---- the clear span: where the HUD's centred cards go while a side panel is up
/// The horizontal span (band units) no side panel covers, this present and as claimed for the next. The pause and
/// Lab panels claim their side while they draw; the HUD's centred cards (the new-item card, the room banners, the
/// callouts), drawn before the panels, centre themselves in last present's span and stay inside it (uiremake: with
/// the Lab panel up, the new-item card ran under the panel and showed half its words).
static mut CLEAR: (f32, f32) = (f32::MIN, f32::MAX);
static mut CLEAR_NEXT: (f32, f32) = (f32::MIN, f32::MAX);
static mut CLEAR_EX: f32 = 0.0;

/// Start of a present: last present's claims become this present's span. `ex`: the extra width each side of the
/// 4:3 band.
pub fn clear_begin(ex: f32) {
    unsafe {
        CLEAR = CLEAR_NEXT;
        CLEAR_NEXT = (f32::MIN, f32::MAX);
        CLEAR_EX = ex;
    }
}
/// A panel covers everything left of `x`.
pub fn claim_left(x: f32) {
    unsafe { CLEAR_NEXT.0 = CLEAR_NEXT.0.max(x) }
}
/// A panel covers everything right of `x`.
pub fn claim_right(x: f32) {
    unsafe { CLEAR_NEXT.1 = CLEAR_NEXT.1.min(x) }
}
/// The clear span (left, right): the 4:3 band with no panel up, else from the panel to the far screen edge.
pub fn clear_span() -> (f32, f32) {
    unsafe {
        if CLEAR.0 == f32::MIN && CLEAR.1 == f32::MAX {
            return (0.0, 640.0);
        }
        let l = if CLEAR.0 == f32::MIN { 8.0 - CLEAR_EX } else { CLEAR.0 };
        let r = if CLEAR.1 == f32::MAX { 632.0 + CLEAR_EX } else { CLEAR.1 };
        if r - l < 160.0 { (0.0, 640.0) } else { (l, r) }
    }
}
/// The centre of the clear span (320 with no panel up).
pub fn mid() -> f32 {
    let (l, r) = clear_span();
    (l + r) * 0.5
}
/// The clear span's width (640 with no panel up).
pub fn clear_w() -> f32 {
    let (l, r) = clear_span();
    r - l
}
