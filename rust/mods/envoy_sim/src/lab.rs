//! The Envoy Item Lab, sim half (envoy_data::lab): a sandbox session where every item, modifier and effect can be
//! equipped, stacked and fired on demand. The session's setup is read once at arm time (`gd_envoy_lab`, like the
//! saved run record); after that everything the Lab changes arrives as synced input: the UI streams 24-bit commands
//! one bit per logic input on two reserved pad bits (envoy_data::lab SIG_CLK / SIG_DATA), so the commands land on
//! the same frame for every resimulation and in the synctest. All of it is state in `L` (hashed with the rest).
//! A Lab session never writes the run record: envoy_store skips it, and the sim publishes no record words for it.
//!
//! Item-shaped behaviour reads `ITEMS` (every id, trigger, tag, cap), so items and triggers added to envoy_data show
//! up here with no change, except the one-line effect hook in `fire` for an item whose effect needs a context
//! (slice 2: add its arm next to the others; an item without one logs that the Lab cannot fire it yet).
use super::*;
use envoy_data::lab::*;

#[link(wasm_import_module = "gd")]
extern "C" {
    fn gd_envoy_lab(out: *mut i32, cap: u32) -> i32;
    fn gd_fighter_set_pos(h: u32, x: f32, y: f32, z: f32);
    fn gd_fighter_set_vel(h: u32, x: f32, y: f32, z: f32);
}

#[repr(C)]
pub(crate) struct Lab {
    pub on: i32,
    pub cfg: [i32; LAB_WORDS],
    pub off: [u32; 2],
    pub cats: i32,
    pub crowd: i32,
    pub cmds: i32,
    pub last: i32,
    /// The command reader per port: bits read (0: waiting for a start bit), the bits so far.
    pub rd_n: [i32; 4],
    pub rd_acc: [u32; 4],
    pub tlog: [[i32; TL_W]; TLOG],
    pub tlog_n: i32,
    /// `MELEE_ENVOY_TEST_DRIVES` lab script (`lab@frame:op:a:b`) entries already run.
    pub script_at: i32,
}
pub(crate) static mut L: Lab = unsafe { core::mem::zeroed() };

pub(crate) fn on() -> bool {
    unsafe { L.on != 0 }
}

/// At arm time: a Lab session? Reads the setup words; the saved run is not read (a fresh session every time).
pub(crate) unsafe fn take() -> bool {
    let mut w = [0i32; LAB_WORDS];
    let n = gd_envoy_lab(w.as_mut_ptr(), LAB_WORDS as u32);
    L = core::mem::zeroed();
    if n <= 0 || w[L_ON] == 0 {
        return false;
    }
    L.on = 1;
    L.cfg = w;
    L.crowd = (w[L_FOES] == FOE_CROWD) as i32;
    log(b"envoy: lab session armed, foes*10 + dummy ", w[L_FOES] * 10 + w[L_DUMMY]);
    true
}

/// The fight a Lab session plays (its foe layout): the crowd room for a crowd, room 1 (the opening gauntlet of
/// duels) otherwise.
pub(crate) fn start_fight(dflt: i32) -> i32 {
    if !on() {
        return dflt;
    }
    if unsafe { L.cfg[L_FOES] } == FOE_CROWD { 1 } else { 0 }
}

/// The Lab's plan over the run's: its stage, 99 stocks a side (the gauntlet keeps its foes' stocks so the next
/// duel comes in), its foes.
pub(crate) unsafe fn plan(p: &mut [i32; PLAN_N]) {
    if !on() {
        return;
    }
    let c = &L.cfg;
    if STAGES.contains(&c[L_STAGE]) {
        p[1] = c[L_STAGE];
    }
    if c[L_MAIN] >= 0 {
        p[2] = c[L_MAIN];
    }
    if c[L_PARTNER] >= 0 {
        p[3] = c[L_PARTNER];
    }
    p[4] = 99;
    let lv = c[L_LEVEL].clamp(1, 9);
    match c[L_FOES] {
        FOE_NONE | FOE_SUBSPACE => p[7] = 0,
        FOE_DUMMY | FOE_CPU => p[7] = 1,
        FOE_CROWD => p[7] = 3,
        _ => {}
    }
    for i in 0..3 {
        if c[L_FOES] != FOE_GAUNTLET {
            p[14 + i] = 99;
            p[32 + i] = -1;
        }
        p[11 + i] = if c[L_FOES] == FOE_DUMMY && c[L_DUMMY] != DUMMY_CPU { 1 } else { lv };
        p[17 + i] = 1000;
        p[20 + i] = 1000;
        p[23 + i] = 1000;
    }
    if c[L_FOE_CK] >= 0 && c[L_FOES] != FOE_CROWD {
        p[8] = c[L_FOE_CK];
    }
    p[26] = 0;
    if c[L_FOES] != FOE_GAUNTLET {
        for k in 0..6 {
            p[PL_GAUNT + k] = -1;
        }
        p[PL_ELITE_WAVE] = 0;
    }
}

/// The Subspace enemy kind of a Lab session with Subspace foes (test_waves' code), or -1.
pub(crate) fn waves() -> i32 {
    unsafe {
        if on() && L.cfg[L_FOES] == FOE_SUBSPACE {
            if L.cfg[L_SUBSPACE] > 0 { L.cfg[L_SUBSPACE] } else { 0 }
        } else {
            -1
        }
    }
}

/// Copies of item `id` that count for player `p` (0 while the Lab switched it or its category off).
pub(crate) unsafe fn gate(id: i32, n: i32) -> i32 {
    if L.on == 0 || n == 0 {
        return n;
    }
    if (0..64).contains(&id) && L.off[id as usize / 32] >> (id % 32) & 1 != 0 {
        return 0;
    }
    if L.cats != 0 {
        if let Some(it) = item(id) {
            if it.trigger != TR_MODIFIER && item_cats(it) & L.cats != 0 && item_cats(it) & !L.cats == 0 {
                return 0;
            }
        }
    }
    n
}

/// A trigger happened for player `p` (TR_*; LT_FIRE + id for a fire on demand): the live trigger log.
pub(crate) unsafe fn trig(p: usize, tr: i32) {
    if L.on == 0 {
        return;
    }
    let mut fired = 0;
    if tr < LT_FIRE {
        for it in ITEMS.iter() {
            if it.trigger as i32 == tr && items::copies(p, it.id) > 0 {
                fired += 1;
            }
        }
    } else {
        fired = 1;
    }
    let i = L.tlog_n as usize % TLOG;
    L.tlog[i] = [S.play, p as i32, tr, fired];
    L.tlog_n += 1;
}

/// The crowd knockback (x1000) on a foe hit: the Lab's toggle.
pub(crate) unsafe fn crowd(dflt: i32) -> i32 {
    if L.on == 0 {
        return dflt;
    }
    if L.crowd != 0 { dflt.max(CROWD_KB) } else { 1000 }
}

/// Once per frame (live or frozen, before anything reads the inventory): read every port's command bits.
pub(crate) unsafe fn frame() {
    if L.on == 0 {
        return;
    }
    for port in 0..4usize {
        let b = gd_pad_held(port as i32);
        if b & SIG_CLK == 0 {
            L.rd_n[port] = 0;
            continue;
        }
        let bit = (b & SIG_DATA != 0) as u32;
        if L.rd_n[port] == 0 {
            if bit == 1 {
                L.rd_n[port] = 1;
                L.rd_acc[port] = 0;
            }
            continue;
        }
        L.rd_acc[port] = L.rd_acc[port] << 1 | bit;
        L.rd_n[port] += 1;
        if L.rd_n[port] > CMD_BITS as i32 {
            L.rd_n[port] = 0;
            let p = if port < nh() as usize { port } else { 0 };
            run(p, L.rd_acc[port] & 0xFF_FFFF);
        }
    }
    script();
}

/// Test knob: `MELEE_ENVOY_TEST_DRIVES` entries `lab@F:op:a:b` run Lab command (op, a, b) at play frame F (the
/// synctest drives the Lab with no UI). Deterministic like any test knob.
unsafe fn script() {
    let mut buf = [0u8; 256];
    let n = gd_envoy_test_drives(buf.as_mut_ptr(), buf.len() as u32);
    if n <= 0 {
        return;
    }
    let s = core::str::from_utf8(&buf[..(n as usize).min(buf.len())]).unwrap_or("");
    let mut k = 0;
    for t in s.split(|c| c == ',' || c == '/' || c == '|' || c == ';') {
        let Some(rest) = t.trim().strip_prefix("lab@") else { continue };
        let mut f = rest.split(':').map(|x| x.parse::<i32>().unwrap_or(0));
        let (at, op, a, b) = (f.next().unwrap_or(0), f.next().unwrap_or(0), f.next().unwrap_or(0), f.next().unwrap_or(0));
        if k >= L.script_at && S.play >= at {
            L.script_at = k + 1;
            run(0, cmd(op, a, b));
        }
        k += 1;
    }
}

unsafe fn set_copies(p: usize, id: i32, n: i32) {
    if item(id).is_none() || !(1..ITEM_IDS as i32).contains(&id) {
        return;
    }
    S.inv[p][id as usize] = n.clamp(0, 99);
    if n > 0 {
        let k = S.gain[p][5] + 1;
        S.gain[p] = [S.play, id, 0, S.inv[p][id as usize], G_PICK, k];
    }
}

/// Percent x1000 of slot `s` set to `pct` (a hit with no knockback).
unsafe fn set_pct(s: i32, pct: i32) {
    if !alive(s) {
        return;
    }
    let cur = gd_envoy_percent(s, 0, 0);
    if cur >= 0 {
        gd_envoy_percent(s, 0, pct.clamp(0, 999) * 1000 - cur);
    }
}

unsafe fn run(p: usize, c: u32) {
    let (op, a, b) = ((c >> 16 & 0xFF) as i32, (c >> 8 & 0xFF) as i32, (c & 0xFF) as i32);
    L.cmds += 1;
    L.last = c as i32;
    log(b"envoy: lab command op*65536 + a*256 + b ", c as i32);
    match op {
        LC_SET => set_copies(p, a, b),
        LC_ADD => set_copies(p, a, items::copies(p, a) + 1),
        LC_SUB => set_copies(p, a, items::copies(p, a) - 1),
        LC_ALL => {
            for it in ITEMS.iter() {
                set_copies(p, it.id, b.max(1));
            }
        }
        LC_CLEAR => S.inv[p] = [0; ITEM_IDS],
        LC_RANDOM => {
            S.inv[p] = [0; ITEM_IDS];
            let mut h = mix(0x1AB0_0000 ^ (a as u32) << 8 ^ S.play as u32);
            for _ in 0..b.clamp(1, ITEMS_LEN as i32) {
                h = mix(h.wrapping_add(0x9E37_79B9));
                let it = &ITEMS[(h % ITEMS_LEN as u32) as usize];
                set_copies(p, it.id, items::copies(p, it.id) + 1);
            }
        }
        LC_FIRE => {
            trig(p, LT_FIRE + a);
            items::lab_fire(p, a);
        }
        LC_TRIGGER => {
            trig(p, a);
            for it in ITEMS.iter() {
                if it.trigger as i32 == a && items::copies(p, it.id) > 0 {
                    items::lab_fire(p, it.id);
                }
            }
        }
        LC_PCT_ME => {
            for s in 0..nh() {
                set_pct(s, a | b << 8);
            }
        }
        LC_PCT_FOE => {
            for s in 0..4 {
                if is_foe(s) {
                    set_pct(s, a | b << 8);
                }
            }
        }
        LC_DAMAGE => {
            for s in 0..4 {
                if is_foe(s) && alive(s) {
                    gd_envoy_percent(s, 0, a * 1000);
                }
            }
        }
        LC_RESET => {
            let mut k = 0;
            for s in 0..4 {
                if !alive(s) {
                    continue;
                }
                set_pct(s, 0);
                let h = gd_fighter_from_slot(s);
                if h != 0 {
                    let x = if is_foe(s) { 30.0 + 20.0 * k as f32 } else { -30.0 - 20.0 * s as f32 };
                    if is_foe(s) {
                        k += 1;
                    }
                    gd_fighter_set_pos(h, x, 20.0, 0.0);
                    gd_fighter_set_vel(h, 0.0, 0.0, 0.0);
                }
            }
            items::reset_match();
        }
        LC_COINS => {
            HH.purse = (a | b << 8) as i64 * COIN_UNIT;
        }
        LC_LIMIT => {
            S.lim = a.clamp(1, items::LIM_CAP);
            if b > 0 {
                S.pool = b.clamp(1, 99);
                for h in 0..nh() {
                    gd_envoy_apply(h, AP_STOCKS, S.pool);
                }
            }
        }
        LC_OFF => {
            if (0..64).contains(&a) {
                let m = 1u32 << (a % 32);
                if b != 0 {
                    L.off[a as usize / 32] |= m;
                } else {
                    L.off[a as usize / 32] &= !m;
                }
            }
        }
        LC_CATS => L.cats = a & 15,
        LC_BREAK => {
            let s = if a == 0 { p as i32 } else { (0..4).find(|&s| is_foe(s) && alive(s)).unwrap_or(-1) };
            if s >= 0 {
                let r = gd_envoy_combo_break(s, CB_INTANGIBLE, CB_FREEZE);
                log(b"envoy: lab combo break forced, slot*10 + rc ", s * 10 + r);
            }
        }
        LC_CROWD => L.crowd = (a != 0) as i32,
        LC_SHIFT => {
            S.stage_want = SH_WAVE;
            log(b"envoy: lab stage transition asked\0", 0);
        }
        LC_ROOM => match a {
            ROOM_ITEM => items::ped_open(PED_ITEM_ROOM),
            ROOM_BOSS_DROP => items::ped_open(PED_BOSS),
            ROOM_CAMPFIRE => items::campfire(),
            _ => log(b"envoy: lab room not on this build ", a),
        },
        _ => log(b"envoy: lab command unknown ", op),
    }
}

/// The Lab's view words.
pub(crate) unsafe fn publish(v: &mut [i32; VIEW_WORDS]) {
    if L.on == 0 {
        return;
    }
    v[VL_ON] = 1;
    v[VL_OFF] = L.off[0] as i32;
    v[VL_OFF + 1] = L.off[1] as i32;
    v[VL_CATS] = L.cats;
    v[VL_CROWD] = L.crowd;
    v[VL_CMDS] = L.cmds;
    v[VL_LAST] = L.last;
    v[VL_FOES] = L.cfg[L_FOES];
    v[VL_ROOMS] = 1 << ROOM_ITEM | 1 << ROOM_BOSS_DROP | 1 << ROOM_CAMPFIRE;
    v[VL_TLOG_N] = L.tlog_n;
    for i in 0..TLOG {
        v[VL_TLOG + i * TL_W..VL_TLOG + (i + 1) * TL_W].copy_from_slice(&L.tlog[i]);
    }
    // nothing of a Lab session is a run to continue
    for w in V_REC..V_REC + REC_WORDS {
        v[w] = 0;
    }
}

pub(crate) unsafe fn bytes() -> &'static [u8] {
    core::slice::from_raw_parts(core::ptr::addr_of!(L) as *const u8, core::mem::size_of::<Lab>())
}
