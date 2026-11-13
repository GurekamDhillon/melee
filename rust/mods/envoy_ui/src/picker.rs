//! The camp's fighter picker (UI-SPEC step 7, D2; screens fighters.png): each joined player picks a main and a
//! partner from one grid with their own cursor (a Random cell; locked fighters faded), then Done, then Ready.
//! A, A, A keeps the last party. The party goes to the engine (`gd_ui_envoy_party`) when a new run starts; the run
//! then goes from the camp straight into fight 1.

use crate::camp::read_pad;
use crate::camp::Pad;
use crate::ck_name;
use crate::kit::*;
use gd_ui_enc::*;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    fn gd_ui_envoy_party(m1: i32, p1: i32, m2: i32, p2: i32) -> i32;
    fn gd_ui_envoy_roster(out: *mut i32, max: i32) -> i32;
    fn gd_ui_envoy_fighter_name(ck: i32, out: *mut u8, cap: i32) -> i32;
    fn gd_ui_fighter_tex(ck: i32, which: i32) -> u32;
}

/// Each fighter's character-select icon from the player's disc (gd_ui_fighter_tex; 0 none: the lettered sticker),
/// asked once per kind. u32::MAX: not asked yet.
static mut ICON: [u32; 0x200] = [u32::MAX; 0x200];

/// Fighter `k`'s icon texture handle (0 when the host or the disc has none).
pub(crate) unsafe fn icon(k: i32) -> u32 {
    if !(0..0x200).contains(&k) {
        return 0;
    }
    let i = k as usize;
    if ICON[i] == u32::MAX {
        ICON[i] = gd_ui_fighter_tex(k, 0);
    }
    ICON[i]
}

/// Each fighter's large portrait (gd_ui_fighter_tex which 1), same caching as ICON.
static mut PORTRAIT: [u32; 0x200] = [u32::MAX; 0x200];

/// Fighter `k`'s portrait texture handle (0 when there is none).
pub(crate) unsafe fn portrait(k: i32) -> u32 {
    if !(0..0x200).contains(&k) {
        return 0;
    }
    let i = k as usize;
    if PORTRAIT[i] == u32::MAX {
        PORTRAIT[i] = gd_ui_fighter_tex(k, 1);
    }
    PORTRAIT[i]
}

pub const RANDOM: i32 = 99;
/// Retail fighters in the character select's order; the build's roster decides which show (m-ex slots follow).
const ORDER: [i32; 26] = [22, 8, 7, 5, 12, 17, 1, 0, 25, 20, 2, 11, 4, 16, 6, 21, 24, 13, 15, 10, 3, 9, 23, 14, 18, 19];
const COLS: usize = 9;
const ROWS: usize = 3;
const MAXG: usize = 128;
/// The grid (filled by `open` from `gd_ui_envoy_roster`); last: Random.
static mut GRID: [i32; MAXG] = [0; MAXG];
static mut NG: usize = 0;
static mut LOCKG: [bool; MAXG] = [false; MAXG];
static mut NAMES: [[u8; 24]; MAXG] = [[0; 24]; MAXG];
static mut NAMEL: [usize; MAXG] = [0; MAXG];
static mut TOP: usize = 0;
static mut PTR_AT: [(f32, f32); 2] = [(0.0, 0.0); 2];
/// Each cell's two letters (unique across the grid: the fallback when there is no icon).
static mut ABBR: [[u8; 2]; MAXG] = [[b' '; 2]; MAXG];

/// The last party (or the default): [P1 main, P1 partner, P2 main, P2 partner].
pub static mut LAST: [i32; 4] = [2, 9, 20, 23];
static mut OPEN: bool = false;
static mut COOP: bool = false;
static mut AGE: i32 = 0;
static mut DONE_AT: i32 = -1;
static mut T: i32 = 0;
/// Per player: step (0 main, 1 partner, 2 Done hovered, 3 Ready), grid cursor, seats, message.
static mut STEP: [usize; 2] = [0; 2];
static mut CUR: [usize; 2] = [0; 2];
static mut SEAT: [[i32; 2]; 2] = [[-1; 2]; 2];
static mut SAY: [&str; 2] = [""; 2];

pub fn short(k: i32) -> &'static str {
    unsafe {
        // Random (99) is not an m-ex slot: it took the m-ex branch and drew an empty name in the seat
        if k >= 0x22 && k != RANDOM {
            if let Some(i) = (0..NG).find(|&i| GRID[i] == k) {
                if let Ok(t) = core::str::from_utf8(&NAMES[i][..NAMEL[i]]) {
                    // a long m-ex name keeps its first word on the 60-wide card ("Diddy Kong" -> "Diddy")
                    if t.len() > 9 {
                        if let Some(w) = t.split(' ').next().filter(|w| w.len() >= 3) {
                            return w;
                        }
                    }
                    return t;
                }
            }
            return "Fighter";
        }
    }
    // one style (title case), short enough for a 60-wide card
    const SHORT: [&str; 26] = ["Falcon", "DK", "Fox", "G&W", "Kirby", "Bowser", "Link", "Luigi", "Mario", "Marth",
        "Mewtwo", "Ness", "Peach", "Pikachu", "Ice Climbers", "Jiggly", "Samus", "Yoshi", "Zelda", "Sheik", "Falco",
        "Y. Link", "Dr. Mario", "Roy", "Pichu", "Ganon"];
    match k {
        RANDOM => "Random",
        0..=25 => SHORT[k as usize],
        _ => ck_name(k),
    }
}

unsafe fn locked(k: i32) -> bool {
    (0..NG).any(|i| GRID[i] == k && LOCKG[i])
}

unsafe fn cell_of(k: i32) -> usize {
    (0..NG).find(|&i| GRID[i] == k).unwrap_or(0)
}

/// Two letters for a fighter with no art: the initials of the first two words, or the first two letters.
fn abbr(s: &str, out: &mut [u8; 2]) -> usize {
    let mut it = s.split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty());
    let up = |x: &str, i: usize| x.as_bytes().get(i).copied().unwrap_or(b' ').to_ascii_uppercase();
    let Some(w0) = it.next() else { return 0 };
    out[0] = up(w0, 0);
    out[1] = match it.next() {
        Some(w1) => up(w1, 0),
        None => up(w0, 1),
    };
    2
}

unsafe fn fill_grid() {
    let mut r = [0i32; MAXG];
    let n = gd_ui_envoy_roster(r.as_mut_ptr(), (MAXG - 1) as i32).max(0) as usize;
    let have = |k: i32| r[..n].iter().find(|&&w| w & 0xFFFF == k).map(|&w| (w >> 16) == 2);
    NG = 0;
    for &k in ORDER.iter() {
        if let Some(lk) = have(k) {
            GRID[NG] = k;
            LOCKG[NG] = lk;
            NG += 1;
        }
    }
    for &w in r[..n].iter() {
        let k = w & 0xFFFF;
        if k >= 0x22 && NG < MAXG - 1 {
            GRID[NG] = k;
            LOCKG[NG] = (w >> 16) == 2;
            NAMEL[NG] = gd_ui_envoy_fighter_name(k, NAMES[NG].as_mut_ptr(), 24).clamp(0, 24) as usize;
            NG += 1;
        }
    }
    if NG == 0 {
        // an older host: the retail tag fighters
        for &k in ORDER[..23].iter() {
            GRID[NG] = k;
            LOCKG[NG] = false;
            NG += 1;
        }
    }
    GRID[NG] = RANDOM;
    LOCKG[NG] = false;
    NG += 1;
    // two letters each, unique: on a clash the second letter walks on through the name (Pichu PI, Pikachu PK)
    for i in 0..NG {
        let mut ab = [b' '; 2];
        abbr(short(GRID[i]), &mut ab);
        let name = short(GRID[i]).as_bytes();
        let mut j = 1;
        while (0..i).any(|q| ABBR[q] == ab) && j < name.len() {
            let c = name[j].to_ascii_uppercase();
            if c.is_ascii_alphabetic() {
                ab[1] = c;
            }
            j += 1;
        }
        ABBR[i] = ab;
    }
}

pub unsafe fn is_open() -> bool {
    OPEN
}

/// Open the picker: each player's cursor on the main they had.
pub unsafe fn open(coop: bool) {
    fill_grid();
    TOP = 0;
    OPEN = true;
    COOP = coop;
    AGE = 0;
    DONE_AT = -1;
    for p in 0..2 {
        STEP[p] = 0;
        SEAT[p] = [LAST[p * 2], LAST[p * 2 + 1]];
        CUR[p] = cell_of(LAST[p * 2]);
        SAY[p] = "";
    }
}

/// Random seats rolled to real fighters (a partner never equals its own main). Done when the picker closes, so the
/// camp shows the fighter the run will get (it showed "Random" before), and again at send for an older LAST.
unsafe fn roll(q: &mut [i32; 4]) {
    let mut seed = (T as u32).wrapping_mul(2654435761).wrapping_add(12345);
    for i in 0..4 {
        if q[i] == RANDOM {
            let avoid = if i % 2 == 1 { q[i - 1] } else { -1 };
            for _ in 0..256 {
                seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                let k = GRID[(seed >> 16) as usize % (NG - 1).max(1)];
                if k != avoid && k != RANDOM && !locked(k) {
                    q[i] = k;
                    break;
                }
            }
        }
    }
}

/// The party for a new run (any Random left rolled now), sent to the engine.
pub unsafe fn send(coop: bool) {
    let mut q = LAST;
    if q.contains(&RANDOM) {
        fill_grid();
        roll(&mut q);
    }
    if !coop {
        q[2] = -1;
        q[3] = -1;
    }
    gd_ui_envoy_party(q[0], q[1], q[2], q[3]);
}

unsafe fn input(p: usize, pad: &Pad) {
    match STEP[p] {
        3 => {
            if pad.b {
                STEP[p] = 2;
            }
            return;
        }
        2 => {
            if pad.a {
                STEP[p] = 3;
            } else if pad.b {
                STEP[p] = 1;
                CUR[p] = cell_of(SEAT[p][1]);
            }
            return;
        }
        _ => {}
    }
    let n = NG;
    let c = CUR[p];
    if pad.left {
        CUR[p] = if c % COLS == 0 { (c + COLS - 1).min(n - 1) } else { c - 1 };
    } else if pad.right {
        CUR[p] = if c % COLS == COLS - 1 || c + 1 >= n { c - c % COLS } else { c + 1 };
    } else if pad.up {
        CUR[p] = if c >= COLS { c - COLS } else { (c + (n / COLS) * COLS).min(n - 1) };
    } else if pad.down {
        CUR[p] = if c + COLS < n { c + COLS } else { c % COLS };
    }
    let k = GRID[CUR[p]];
    if pad.a {
        if locked(k) {
            SAY[p] = "That fighter is still locked.";
        } else if STEP[p] == 1 && k == SEAT[p][0] && k != RANDOM {
            SAY[p] = "That one is your main. Pick another partner.";
        } else {
            SEAT[p][STEP[p]] = k;
            SAY[p] = "";
            STEP[p] += 1;
            if STEP[p] == 1 {
                CUR[p] = cell_of(SEAT[p][1]);
            }
        }
    } else if pad.b {
        if STEP[p] == 0 {
            OPEN = false; // leave, keeping the old party
        } else {
            STEP[p] = 0;
            CUR[p] = cell_of(SEAT[p][0]);
        }
    }
}

/// One frame of the picker. Returns false once it has closed.
pub unsafe fn frame(e: &mut E) -> bool {
    T += 1;
    AGE += 1;
    let players = if COOP { 2 } else { 1 };
    if AGE > 4 && DONE_AT < 0 {
        for p in 0..players {
            let pad = read_pad(p);
            input(p, &pad);
            if !OPEN {
                return false;
            }
        }
        if (0..players).all(|p| STEP[p] == 3) {
            DONE_AT = T;
        }
    }
    if DONE_AT >= 0 && T - DONE_AT > 20 {
        for p in 0..players {
            LAST[p * 2] = SEAT[p][0];
            LAST[p * 2 + 1] = SEAT[p][1];
        }
        roll(&mut *core::ptr::addr_of_mut!(LAST));
        OPEN = false;
        return false;
    }
    e.layer(LAYER_MENU, 0);
    e.rect(-107.0, 0.0, 854.0, 480.0, 0x1A10_4AFF);
    text_edge(e, 16.0, 4.0, d(24), WHITE, ALIGN_LEFT, 2.2, "Choose fighters");
    let sub = if COOP { "Each player brings two: a main and a partner." } else { "Bring two: a main and a partner." };
    e.text_ex(250.0, 13.0, bs(12), PALE, ALIGN_LEFT, 0.0, sub);
    // the grid
    // more than three rows (an m-ex roster): the rows scroll to keep P1's cursor in view
    let crow = CUR[0] / COLS;
    if crow < TOP {
        TOP = crow;
    } else if crow >= TOP + ROWS {
        TOP = crow + 1 - ROWS;
    }
    let nrows = (NG + COLS - 1) / COLS;
    if nrows > ROWS {
        let mut b = crate::Buf::new();
        b.s("Rows ").i(TOP as i32 + 1).s("-").i((TOP + ROWS).min(nrows) as i32).s(" of ").i(nrows as i32);
        e.text_ex(616.0, 13.0, bs(11), PALE, ALIGN_RIGHT, 0.0, b.str());
    }
    for i in TOP * COLS..(NG).min((TOP + ROWS) * COLS) {
        let k = GRID[i];
        let j = i - TOP * COLS;
        let (cx, cy) = (26.0 + (j % COLS) as f32 * 66.0, 40.0 + (j / COLS) as f32 * 81.0);
        let hov = [STEP[0] < 2 && CUR[0] == i, COOP && STEP[1] < 2 && CUR[1] == i];
        let lk = locked(k);
        let on = (hov[0] || hov[1]) && !lk;
        let deg = if on { 0.0 } else { [-0.6, 0.4, -0.3, 0.7, -0.5][i % 5] };
        let (_, a) = enter(AGE - (i as i32) / 3);
        let facec = if lk { S_FACE_FADED } else if on { S_FACE_SUNNY } else { S_FACE_WHITE };
        if hov[0] || hov[1] {
            let col = if hov[0] { CORAL } else { LAGOON };
            face(e, S_FACE_WHITE, cx - 3.0, cy - 3.0, 66.0, 81.0, alpha(col, a));
        }
        sticker(e, facec, cx, cy, 60.0, 75.0, deg, alpha(WHITE, a), |e| {
            if k == RANDOM {
                text_edge(e, cx + 30.0, cy + 8.0, d(34), SUNNY, ALIGN_CENTER, 1.4, "?");
                e.text_ex(cx + 30.0, cy + 54.0, d(11), alpha(INK, a), ALIGN_CENTER, 56.0, "Random");
            } else {
                let h = icon(k);
                if h != 0 {
                    // the disc's own select icon (faded when locked)
                    e.tex(h, cx + 5.0, cy + 5.0, 50.0, 44.0, [0.0, 0.0, 1.0, 1.0], alpha(if lk { 0x8080_80FF } else { WHITE }, a * if lk { 0.6 } else { 1.0 }));
                } else {
                    // no disc art: the lettered sticker (the old select's two-letter fallback)
                    let t = core::str::from_utf8(&ABBR[i]).unwrap_or("?");
                    text_edge(e, cx + 30.0, cy + 10.0, d(26), alpha(if lk { PALE } else if k >= 0x22 { LAGOON } else { CORAL }, a), ALIGN_CENTER, 1.4, t);
                }
                if lk {
                    spr_c(e, S_LOCK, cx + 46.0, cy + 12.0, 1.0, WHITE);
                }
                e.text_ex(cx + 30.0, cy + 54.0, d(11), alpha(if lk { LILAC } else { INK }, a), ALIGN_CENTER, 58.0, if lk { "Locked" } else { short(k) });
            }
        });
        // seat tabs and cursors
        for p in 0..players {
            for s in 0..2 {
                if SEAT[p][s] == k && STEP[p] > s {
                    let tx = if p == 0 { cx - 4.0 } else { cx + 24.0 };
                    let ty = cy - 8.0 + s as f32 * 0.0;
                    face(e, if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, tx, ty, 40.0, 15.0, WHITE);
                    e.text_ex(tx + 20.0, ty + 1.0, d(9), WHITE, ALIGN_CENTER, 0.0, if s == 0 { "Main" } else { "Partner" });
                }
            }
            if hov[p] {
                // the pointer sits off the card's lower right corner (it never covers the name)
                PTR_AT[p] = (cx + 62.0, cy + 68.0 - p as f32 * 16.0);
            }
        }
    }
    for p in 0..players {
        if STEP[p] < 2 {
            spr_c(e, S_PTR + p, PTR_AT[p].0, PTR_AT[p].1, 0.62, WHITE);
        }
    }
    // trays
    for p in 0..players {
        let x = if p == 0 { 20.0 } else { 326.0 };
        let y = 290.0;
        let w = if COOP { 294.0 } else { 600.0 };
        sticker(e, S_FACE_WHITE, x, y, w, 92.0, if p == 0 { -0.5 } else { 0.5 }, WHITE, |e| {
            for s in 0..2 {
                let sx = x + 8.0 + s as f32 * 104.0;
                let choosing = STEP[p] == s;
                let shown = if choosing { GRID[CUR[p]] } else { SEAT[p][s] };
                face(e, if choosing { S_FACE_SUNNY } else { S_FACE_CREAM }, sx, y + 9.0, 100.0, 72.0, WHITE);
                e.text_ex(sx + 8.0, y + 13.0, bs(11), LILAC, ALIGN_LEFT, 0.0, if s == 0 { "Main" } else { "Partner" });
                let h = if shown >= 0 && shown != RANDOM { icon(shown) } else { 0 };
                if h != 0 {
                    e.tex(h, sx + 4.0, y + 37.0, 28.0, 25.0, [0.0, 0.0, 1.0, 1.0], WHITE);
                } else {
                    spr_c(e, S_HEAD, sx + 18.0, y + 52.0, 0.95, WHITE);
                }
                e.text_ex(sx + 32.0, y + 45.0, d(13), INK, ALIGN_LEFT, 66.0, if shown >= 0 { short(shown) } else { "?" });
            }
            if !COOP {
                // the large portrait of the fighter being chosen (or the main once both are seated)
                let k = if STEP[p] < 2 { GRID[CUR[p]] } else { SEAT[p][0] };
                let h = if k >= 0 && k != RANDOM && !locked(k) { portrait(k) } else { 0 };
                if h != 0 {
                    face(e, S_FACE_CREAM, x + w - 76.0, y + 4.0, 66.0, 84.0, WHITE);
                    e.tex(h, x + w - 73.0, y + 6.0, 60.0, 80.0, [0.0, 0.0, 1.0, 1.0], WHITE);
                }
            }
            let (dx, dy) = (x + 222.0, y + 28.0);
            let ready = STEP[p] == 3;
            let a = if STEP[p] >= 2 { 1.0 } else { 0.4 };
            let fc = if ready { S_FACE_CORAL } else if STEP[p] == 2 { S_FACE_SUNNY } else { S_FACE_WHITE };
            face(e, fc, dx, dy, 64.0, 34.0, alpha(WHITE, a));
            e.text_ex(dx + 32.0, dy + 8.0, d(14), alpha(if ready { WHITE } else { INK }, a), ALIGN_CENTER, 0.0, if ready { "Ready" } else { "Done" });
        });
        face(e, if p == 0 { S_FACE_CORAL } else { S_FACE_LAGOON }, x - 4.0, y - 10.0, 28.0, 18.0, WHITE);
        e.text_ex(x + 10.0, y - 8.0, d(12), WHITE, ALIGN_CENTER, 0.0, if p == 0 { "P1" } else { "P2" });
        if STEP[p] == 2 {
            spr(e, S_PTR + p, x + 190.0, y + 32.0, 0.0, 0.0, WHITE);
        }
    }
    if DONE_AT >= 0 {
        burst(e, S_BURST_CORAL, 320.0, 250.0, 70.0, T - DONE_AT, "Ready!", 13);
    }
    // the strip: P1's line (P2's when only P2 has something to say)
    let p = if COOP && SAY[0].is_empty() && !SAY[1].is_empty() { 1 } else { 0 };
    let s = if !SAY[p].is_empty() {
        SAY[p]
    } else {
        match STEP[p] {
            0 => "Pick your main. B leaves and keeps the old party.",
            1 => "Pick your partner. B goes back to the main.",
            2 => "Press A on Done. B goes back to the partner.",
            _ => if COOP { "Ready. Waiting for the other player." } else { "Ready." },
        }
    };
    strip(e, 24.0, 590.0, s);
    hints(e, &[("Stick", "Choose"), ("A", "Pick"), ("B", "Back")]);
    true
}
