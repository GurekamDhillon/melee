//! Body-group tints (docs/rust-port/notes/envoy-tints.md): a fighter's draw objects split into body groups (head,
//! torso, arms, hands, legs, feet) from its joint tree, and a per-port, per-group additive colour drawn as one extra
//! TEV stage after the material's (`colour = material + K`).
//!
//! In the spirit of the C-build roguelite (visuals.lua `gd.dobj_tint` + script_game.c `part_joint_region`), not a
//! copy: groups come from `ftPartsTable` (the same named bones for every fighter, m-ex/ACE slots included), a skinned
//! PObj votes by its envelope weights, and the colour is set per PObj, so one big skinned DObj still splits.
//!
//! Callers queue tints once a present (`begin` / `add` / `commit`, like drive_draw); several callers' tints on the
//! same group add up (saturating). Presentation only: nothing here is read by a sim, so it is rollback-safe.
#![allow(static_mut_refs, clippy::missing_safety_doc)]
use super::script_game as sg;
// the GX calls the engine actually wires to the renderer (script_game's own ext GX entries are stubs in the port)
use melee_runtime::tu::lbspdisplay::ext as gx;
use melee_runtime::tu::tev_native::HSD_StateGetCommittedTevStages;
use melee_types::*;
use std::sync::Mutex;
use std::vec::Vec;


/// Body groups (bit = 1 << group).
pub const G_HEAD: u8 = 0;
pub const G_TORSO: u8 = 1;
pub const G_ARMS: u8 = 2;
pub const G_HANDS: u8 = 3;
pub const G_LEGS: u8 = 4;
pub const G_FEET: u8 = 5;
pub const GROUPS: usize = 6;
pub const ALL: u32 = (1 << GROUPS) - 1;
const NONE: u8 = 0xFF;
const PORTS: usize = 6;

/// Named bones → group; the nearest named ancestor of a joint wins (order is priority within one joint).
const ROLE: [(Fighter_Part, u8); 17] = [
    (FtPart_HeadN, G_HEAD),
    (FtPart_NeckN, G_HEAD),
    (FtPart_LHandN, G_HANDS),
    (FtPart_RHandN, G_HANDS),
    (FtPart_LFootJ, G_FEET),
    (FtPart_RFootJ, G_FEET),
    (FtPart_LArmJ, G_ARMS),
    (FtPart_LShoulderJ, G_ARMS),
    (FtPart_RArmJ, G_ARMS),
    (FtPart_RShoulderJ, G_ARMS),
    (FtPart_LLegJ, G_LEGS),
    (FtPart_LKneeJ, G_LEGS),
    (FtPart_RLegJ, G_LEGS),
    (FtPart_RKneeJ, G_LEGS),
    (FtPart_BustN, G_TORSO),
    (FtPart_WaistN, G_TORSO),
    (FtPart_HipN, G_TORSO),
];

/// Per port, per group: additive RGB (0..255).
type Table = [[[u16; 3]; GROUPS]; PORTS];
static BUILD: Mutex<Table> = Mutex::new([[[0; 3]; GROUPS]; PORTS]);
static mut SHOW: Table = [[[0; 3]; GROUPS]; PORTS];
static mut ANY: bool = false;

/// A present begins: no tints queued.
pub fn begin() {
    if let Ok(mut b) = BUILD.lock() {
        *b = [[[0; 3]; GROUPS]; PORTS];
    }
}

/// Add `rgb * strength` (strength 0..1) to every group in `mask` of `port`'s fighter this present.
/// The public entry for every caller (Envoy item groups, the rarest-item aura): tints add up.
pub fn add(port: usize, mask: u32, rgb: u32, strength: f32) {
    if port >= PORTS || !(strength > 0.0) {
        return;
    }
    let s = strength.min(1.0);
    let c = [(rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF].map(|v| (v as f32 * s + 0.5) as u16);
    let Ok(mut b) = BUILD.lock() else { return };
    for g in 0..GROUPS {
        if mask & (1 << g) != 0 {
            for k in 0..3 {
                b[port][g][k] = (b[port][g][k] + c[k]).min(255);
            }
        }
    }
}

/// The present ended: its tints are what the fighters draw with until the next one.
pub fn commit() {
    let Ok(b) = BUILD.lock() else { return };
    // MELEE_BODY_TINT=0 turns the draw off (before/after comparisons); read once
    static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let off = *OFF.get_or_init(|| {
        // MELEE_BODY_TINT=k: the stage runs with K = 0 everywhere (a check that the stage alone changes nothing)
        ZERO_K.store(std::env::var("MELEE_BODY_TINT").is_ok_and(|v| v == "k"), std::sync::atomic::Ordering::Relaxed);
        std::env::var("MELEE_BODY_TINT").is_ok_and(|v| v == "0")
    });
    unsafe {
        SHOW = if off { [[[0; 3]; GROUPS]; PORTS] } else { *b };
        let any = SHOW.iter().flatten().flatten().any(|&v| v != 0);
        if any && !ANY {
            // installed on the first present that tints anything: otherwise the draw path is untouched
            melee_runtime::tu::dobj::hsd::BODY_TINT =
                Some(melee_runtime::tu::dobj::hsd::BodyTint { begin: tint_begin, pobj: tint_pobj });
        }
        ANY = any;
        // MELEE_BODY_TINT_PERF=1: the draw cost (hook time, all fighters) per present, logged every 600 presents
        static PERF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if *PERF.get_or_init(|| std::env::var("MELEE_BODY_TINT_PERF").is_ok_and(|v| v == "1")) {
            STAT.0 += 1;
            STAT.1 += NS;
            STAT.2 = STAT.2.max(NS);
            if STAT.0 == 600 {
                eprintln!("body tint: draw cost over 600 presents: mean {:.4} ms, max {:.4} ms ({} DObjs tinted, {} passed by)", STAT.1 as f64 / 600.0 / 1e6, STAT.2 as f64 / 1e6, DBG[1], DBG[0]);
                DBG = [0; 2];
                STAT = (0, 0, 0);
            }
        }
        NS = 0;
    }
}
static ZERO_K: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static mut NS: u64 = 0;
static mut DBG: [u64; 2] = [0; 2];
static mut STAT: (u64, u64, u64) = (0, 0, 0);

/// The committed tint of one port's group (tests, logs).
pub fn shown(port: usize, group: u8) -> [u16; 3] {
    unsafe { SHOW.get(port).map_or([0; 3], |p| p[group as usize]) }
}

/// Per fighter: the group of each PObj, built once a model (root joint, kind) and rebuilt when it changes.
struct Cache {
    root: usize,
    kind: u32,
    pobj: Vec<(usize, u8)>,
}
static mut CACHE: [Option<Cache>; PORTS] = [const { None }; PORTS];
/// The draw in progress: its port and the TEV K register in use.
static mut CUR: (usize, i32) = (0, -1);

unsafe fn joint_group(fp: *mut Fighter, idx: &[(usize, u8)], mut j: *mut HSD_JObj) -> u8 {
    let mut guard = 0;
    while !j.is_null() && guard < 128 {
        if let Ok(k) = idx.binary_search_by_key(&(j as usize), |e| e.0) {
            return idx[k].1;
        }
        j = (*j).parent;
        guard += 1;
    }
    let _ = fp;
    NONE
}

unsafe fn build(fp: *mut Fighter) -> Option<Cache> {
    let gobj = (*fp).gobj;
    let root = if gobj.is_null() { 0 } else { (*gobj).hsd_obj as usize };
    let kind = (*fp).kind as u32;
    let tab = sg::ftPartsTable;
    let n = sg::lab_joint_count(fp);
    if tab.is_null() || n <= 0 || (*fp).parts.is_null() {
        return Some(Cache { root, kind, pobj: Vec::new() });
    }
    let t = *tab.add(kind as usize);
    if t.is_null() || (*t).part_to_joint.is_null() {
        return Some(Cache { root, kind, pobj: Vec::new() });
    }
    // named joints → group (the first role a joint carries wins)
    let mut idx: Vec<(usize, u8)> = Vec::new();
    for (part, g) in ROLE {
        let ji = *(*t).part_to_joint.add(part as usize) as i32;
        if ji == 0xFF || ji >= n {
            continue;
        }
        let j = (*(*fp).parts.add(ji as usize)).joint;
        if !j.is_null() && !idx.iter().any(|e| e.0 == j as usize) {
            idx.push((j as usize, g));
        }
    }
    idx.sort_by_key(|e| e.0);
    let mut out: Vec<(usize, u8)> = Vec::new();
    for i in 0..n as usize {
        let j = (*(*fp).parts.add(i)).joint;
        if j.is_null() || (*j).flags & (0x20 | 0x4000) != 0 {
            continue; // particles / splines carry no DObjs
        }
        let own = joint_group(fp, &idx, j);
        let mut d = (*j).u.dobj;
        let mut dg = 0;
        while !d.is_null() && dg < 256 {
            let mut p = (*d).pobj;
            let mut pg = 0;
            while !p.is_null() && pg < 256 {
                out.push((p as usize, pobj_group(fp, &idx, p, own)));
                p = (*p).next;
                pg += 1;
            }
            d = (*d).next;
            dg += 1;
        }
    }
    out.sort_by_key(|e| e.0);
    out.dedup_by_key(|e| e.0);
    let mut cnt = [0u32; GROUPS + 1];
    for e in &out {
        cnt[(e.1 as usize).min(GROUPS)] += 1;
    }
    eprintln!("body tint: model kind {kind}: {} pobjs, head/torso/arms/hands/legs/feet/none {:?}", out.len(), cnt);
    Some(Cache { root, kind, pobj: out })
}

/// A PObj's group: a skinned one by its envelope weights (the heaviest group), a rigid one by its joint.
unsafe fn pobj_group(fp: *mut Fighter, idx: &[(usize, u8)], p: *mut HSD_PObj, own: u8) -> u8 {
    match (*p).flags & 0x3000 {
        0x2000 => {
            let mut w = [0f32; GROUPS];
            let mut l = (*p).u.envelope_list;
            let mut guard = 0;
            while !l.is_null() && guard < 64 {
                let mut e = (*l).data as *mut HSD_Envelope;
                let mut g2 = 0;
                while !e.is_null() && g2 < 64 {
                    let g = joint_group(fp, idx, (*e).jobj);
                    if (g as usize) < GROUPS && (*e).weight.is_finite() {
                        w[g as usize] += (*e).weight.max(0.0);
                    }
                    e = (*e).next;
                    g2 += 1;
                }
                l = (*l).next;
                guard += 1;
            }
            let mut best = (0.0f32, own);
            for (g, v) in w.iter().enumerate() {
                if *v > best.0 {
                    best = (*v, g as u8);
                }
            }
            best.1
        }
        0 if !(*p).u.jobj.is_null() => match joint_group(fp, idx, (*p).u.jobj) {
            NONE => own,
            g => g,
        },
        _ => own,
    }
}

unsafe fn tint_begin(dobj: *mut HSD_DObj) -> bool {
    if !ANY {
        return false;
    }
    let t0 = std::time::Instant::now();
    let r = tint_begin_in(dobj);
    NS += t0.elapsed().as_nanos() as u64;
    DBG[r as usize] += 1;
    r
}

unsafe fn tint_begin_in(dobj: *mut HSD_DObj) -> bool {
    let g = melee_fighter::tu::ftdrawcommon::DRAW_FIGHTER;
    if g.is_null() || (*g).user_data.is_null() {
        return false;
    }
    let fp = (*g).user_data as *mut Fighter;
    let port = (*fp).player_idx as usize;
    if port >= PORTS || SHOW[port].iter().all(|c| c == &[0; 3]) {
        return false;
    }
    let root = (*(*fp).gobj).hsd_obj as usize;
    let stale = CACHE[port].as_ref().is_none_or(|c| c.root != root || c.kind != (*fp).kind as u32);
    if stale {
        CACHE[port] = build(fp);
    }
    if CACHE[port].as_ref().is_none_or(|c| c.pobj.is_empty()) {
        return false;
    }
    // after every stage the material set: HSD's committed count misses stages a render callback added (the
    // fighter colour overlay), so take the renderer's live count when the engine provides it
    let (live, kused) = melee_runtime::tu::dobj::hsd::GX_NUM_TEV.map_or((0, 0), |f| f());
    let stage = HSD_StateGetCommittedTevStages().max(live as i32);
    if !(1..15).contains(&stage) {
        return false;
    }
    let reg = free_kreg(dobj, kused);
    if reg < 0 {
        return false;
    }
    CUR = (port, reg);
    let st = stage as u32;
    gx::GXSetTevOrder(st, GX_TEXCOORD_NULL as u32, GX_TEXMAP_NULL as u32, GX_COLOR_NULL as u32);
    gx::GXSetTevKColorSel(st, GX_TEV_KCSEL_K0 as u32 + reg as u32);
    // colour = CPREV + K (a = 0, b = K, c = 1, d = CPREV), clamped; alpha passes through
    gx::GXSetTevColorIn(st, GX_CC_ZERO as u32, GX_CC_KONST as u32, GX_CC_ONE as u32, GX_CC_CPREV as u32);
    gx::GXSetTevColorOp(st, GX_TEV_ADD as u32, GX_TB_ZERO as u32, GX_CS_SCALE_1 as u32, 1, GX_TEVPREV as u32);
    gx::GXSetTevAlphaIn(st, GX_CA_ZERO as u32, GX_CA_ZERO as u32, GX_CA_ZERO as u32, GX_CA_APREV as u32);
    gx::GXSetTevAlphaOp(st, GX_TEV_ADD as u32, GX_TB_ZERO as u32, GX_CS_SCALE_1 as u32, 1, GX_TEVPREV as u32);
    gx::GXSetTevSwapMode(st, GX_TEV_SWAP0 as u32, GX_TEV_SWAP0 as u32);
    gx::GXSetNumTevStages((stage + 1) as u8);
    true
}

/// The highest konst colour register the DObj's material does not use (script_game's `part_tint_register` scan,
/// from the top: the fighter colour overlay takes the lowest free one, lbGetFreeColorRegister).
unsafe fn free_kreg(dobj: *mut HSD_DObj, live_used: u32) -> i32 {
    if dobj.is_null() || (*dobj).mobj.is_null() {
        return -1;
    }
    // the live stages' selections (TExp materials and the colour overlay never show in the tevdesc list)
    let mut used = live_used;
    let td = (*(*dobj).mobj).tevdesc;
    let mut d = if td.is_null() { core::ptr::null_mut() } else { &raw mut (*td).desc as *mut HSD_TevDesc };
    let mut n = 0;
    while !d.is_null() {
        n += 1;
        if n >= 16 {
            return -1;
        }
        if (*d).flags & TEVCONF_MODE as u32 != 0 {
            let (c, a) = ((*d).u.tevconf.kcsel as i32, (*d).u.tevconf.kasel as i32);
            if (12..=15).contains(&c) {
                used |= 1 << (c - 12);
            } else if (16..=31).contains(&c) {
                used |= 1 << ((c - 16) % 4);
            }
            if (16..=31).contains(&a) {
                used |= 1 << ((a - 16) % 4);
            }
        }
        d = (*d).next as *mut HSD_TevDesc;
    }
    (0..4).rev().find(|k| used & (1 << k) == 0).unwrap_or(-1)
}

unsafe fn tint_pobj(p: *mut HSD_PObj) {
    let t0 = std::time::Instant::now();
    tint_pobj_in(p);
    NS += t0.elapsed().as_nanos() as u64;
}

unsafe fn tint_pobj_in(p: *mut HSD_PObj) {
    let (port, reg) = CUR;
    let g = CACHE[port].as_ref().and_then(|c| c.pobj.binary_search_by_key(&(p as usize), |e| e.0).ok().map(|k| c.pobj[k].1));
    let c = match g {
        Some(g) if (g as usize) < GROUPS => SHOW[port][g as usize],
        _ => [0; 3],
    };
    let c = if ZERO_K.load(std::sync::atomic::Ordering::Relaxed) { [0; 3] } else { c };
    let color = GXColor { r: c[0] as u8, g: c[1] as u8, b: c[2] as u8, a: 0 };
    gx::GXSetTevKColor(reg as u32, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_saturates_and_commit_shows() {
        begin();
        add(1, 1 << G_ARMS | 1 << G_HANDS, 0xFF8000, 0.5);
        add(1, ALL, 0xFF0000, 1.0);
        add(9, ALL, 0xFFFFFF, 1.0); // out of range: ignored
        commit();
        assert_eq!(shown(1, G_ARMS), [255, 64, 0]);
        assert_eq!(shown(1, G_HEAD), [255, 0, 0]);
        assert_eq!(shown(0, G_HEAD), [0, 0, 0]);
        begin();
        commit();
        assert_eq!(shown(1, G_ARMS), [0, 0, 0]);
    }
}
