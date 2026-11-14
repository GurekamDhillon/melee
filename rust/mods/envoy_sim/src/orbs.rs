//! Orbitals (envoy-items-design.md 11.7, owner-approved rework): every orb is a real small hit on the engine path
//! (light knockback, hitstun and hitlag, the item's own hit spark and Melee hit sound: the Spark Ring bar). A foe's
//! attack or projectile shatters an orb; it comes back after ORB_RESPAWN frames, or at once when its owner lands a
//! hit. An orb that touches a foe projectile blocks it (the projectile is removed) and shatters. Stacks add orbs and
//! widen the orbit a little.
//!
//! The families: Moon Drops (always out) and the stance orbs, which exist only while the fighter is in one movement
//! state and take their shape from it: Dodge Orb (leads an airdodge, stays in front through the wavedash slide),
//! Nair Ring (circles the fighter for the nair), Dash Orb (leads a dash or run), Shield Orbs (circle the shield).
//! All of it is sim state inside `St` (hashed, rolled back).
use super::*;

pub(crate) const FAM: [i32; FAMS] = [IT_MOON_DROPS, IT_DODGE_ORB, IT_NAIR_RING, IT_DASH_ORB, IT_SHIELD_ORBS];
pub(crate) const FAMS: usize = 5;
pub(crate) const ORB_MAX: usize = 8;
/// Frames a shattered orb stays gone.
pub(crate) const ORB_RESPAWN: i32 = 240;
/// Frames before one orb family can hit the same foe again.
const ORB_HIT_CD: i32 = 30;
/// An orb's reach (hit and block), and how near a foe's attack must come to shatter it.
const ORB_R: f32 = 7.0;
const SHATTER_R: f32 = 11.0;
const AS_DASH: (i32, i32) = (20, 21);
const AS_GUARD: (i32, i32) = (178, 182);
const AS_ESC_AIR: i32 = 236;
const AS_LAND_SPECIAL: i32 = 43;
const AS_NAIR: i32 = 65;

/// Orbs of family `f` player `p` has out (0: none: no copy, or not in the family's state).
unsafe fn count(p: usize, f: usize, st: i32) -> i32 {
    let id = FAM[f];
    let c = items::copies(p, id);
    if c <= 0 {
        return 0;
    }
    let more = c.min(3);
    match f {
        0 => {
            // the owner: items reveal themselves during their use. Moons are out while attacking (their hit window)
            // and for a short while after any item proc.
            let active = (AS_ATTACK0..=74).contains(&st) || st >= 295 || items::proc_age(p) < 40;
            if active { 2 * more * if items::copies(p, IT_TWIN_SHOT) > 0 { 2 } else { 1 } } else { 0 }
        }
        1 => {
            let dodge = st == AS_ESC_AIR || (st == AS_LAND_SPECIAL && S.play - S.orb_dodge_at[p] <= 14);
            if dodge { more } else { 0 }
        }
        2 => if st == AS_NAIR { 3 + more } else { 0 },
        3 => if (AS_DASH.0..=AS_DASH.1).contains(&st) { more } else { 0 },
        _ => if (AS_GUARD.0..=AS_GUARD.1).contains(&st) { 2 + more } else { 0 },
    }
}

/// Where orb `k` of `n` of family `f` is around player `p` at `me` (feet) facing `face`.
unsafe fn place(p: usize, f: usize, k: i32, n: i32, me: P, face: f32) -> P {
    let more = items::copies(p, FAM[f]).clamp(1, 3) as f32;
    let t = S.play as f32;
    let ring = |r: f32, speed: f32, cy: f32| {
        let a = t * speed + k as f32 * TAU / n.max(1) as f32;
        P { x: me.x + items::cosf(a) * r, y: me.y + cy + items::sinf(a) * r * 0.8 }
    };
    match f {
        0 => {
            // Moon Drops: the old orbit, a little wider per copy
            let a = t * 0.075 + k as f32 * TAU / n.max(1) as f32;
            let r = 14.0 + 2.0 * (more - 1.0);
            P { x: me.x + items::cosf(a) * r, y: me.y + 9.0 + items::sinf(a) * r * 0.8 }
        }
        1 => {
            // the dodge's direction (taken when it began), orbs side by side across it
            let (dx, dy) = (S.orb_dir[p][0] as f32 / 100.0, S.orb_dir[p][1] as f32 / 100.0);
            let lead = 15.0 + 2.0 * (more - 1.0);
            let side = (k as f32 - (n - 1) as f32 * 0.5) * 7.0;
            P { x: me.x + dx * lead - dy * side, y: me.y + 8.0 + dy * lead + dx * side }
        }
        2 => ring(13.0 + 2.0 * (more - 1.0), 0.32, 8.0),
        3 => P { x: me.x + face * (15.0 + 8.0 * k as f32), y: me.y + 6.0 },
        _ => ring(17.0 + 1.5 * (more - 1.0), 0.12, 9.0),
    }
}

/// Per unfrozen live frame (after the projectiles): every orb out hits, shatters or blocks.
pub(crate) unsafe fn frame() {
    for p in 0..nh() as usize {
        let h = gd_fighter_from_slot(p as i32);
        if h == 0 || !alive(p as i32) {
            continue;
        }
        let st = gd_fighter_get_action_state(h);
        if st == AS_ESC_AIR && S.orb_prev[p] != AS_ESC_AIR {
            // a new airdodge: its direction (the stick's, read as the dodge's speed a frame in; facing when still)
            let mut w = [0u32; 20];
            gd_fighter_read(h, w.as_mut_ptr());
            let (vx, vy) = (f32::from_bits(w[3]), f32::from_bits(w[4]));
            let l = (vx * vx + vy * vy).max(1e-6);
            let (dx, dy) = if l < 0.01 { (if gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 }, 0.0) } else { let r = inv_sqrt(l); (vx * r, vy * r) };
            S.orb_dir[p] = [x100(dx), x100(dy)];
            S.orb_dodge_at[p] = S.play;
        }
        if st == AS_ESC_AIR {
            S.orb_dodge_at[p] = S.play; // the slide window runs from the dodge's last frame
        }
        S.orb_prev[p] = st;
        let Some(me) = pos(p as i32) else { continue };
        let face = if gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 };
        for f in 0..FAMS {
            let n = count(p, f, st).min(ORB_MAX as i32);
            if n <= 0 {
                continue;
            }
            for k in 0..n as usize {
                if S.orb_broken[p][f][k] != 0 {
                    if S.play >= S.orb_broken[p][f][k] {
                        S.orb_broken[p][f][k] = 0;
                    } else {
                        continue;
                    }
                }
                let o = place(p, f, k as i32, n, me, face);
                if orb_touch(p, f, k, o) {
                    continue; // shattered this frame
                }
            }
        }
    }
}

/// One live orb at `o`: a foe's attack or projectile shatters it; else it hits a foe it touches.
unsafe fn orb_touch(p: usize, f: usize, k: usize, o: P) -> bool {
    let id = FAM[f];
    // a foe projectile touching it: blocked, and the orb shatters
    let n = gd_item_count();
    for i in 0..n.min(32) {
        let ih = gd_item_at(i);
        if ih == 0 {
            continue;
        }
        let mut w = [0u32; 13];
        gd_item_read(ih, w.as_mut_ptr());
        let owner = w[8] as i32;
        if owner < 0 || owner > 3 || !enemy_of(p * 2, owner) {
            continue;
        }
        let ip = P { x: f32::from_bits(w[0]), y: f32::from_bits(w[1]) };
        if d2(ip, o) < (ORB_R + 3.0) * (ORB_R + 3.0) {
            gd_envoy_item_block(ih);
            ev(EV_ORB_BLOCK, p as i32, id);
            items::log_item(b"orb blocked a projectile", id, p as i32 * 100 + owner);
            shatter(p, f, k, o);
            return true;
        }
    }
    for s in 0..4 {
        if !enemy_of(p * 2, s) {
            continue;
        }
        let Some(q) = pos(s) else { continue };
        let fc = P { x: q.x, y: q.y + 8.0 };
        let hs = gd_fighter_from_slot(s);
        let st = if hs != 0 { gd_fighter_get_action_state(hs) } else { -1 };
        let attacking = (AS_ATTACK0..=74).contains(&st) || st >= 295;
        if attacking && d2(fc, o) < SHATTER_R * SHATTER_R {
            items::log_item(b"orb shattered by an attack", id, p as i32 * 100 + s);
            shatter(p, f, k, o);
            return true;
        }
        if d2(fc, o) < (ORB_R + 3.0) * (ORB_R + 3.0) && S.play >= S.orb_cd[p][f][s as usize] {
            S.orb_cd[p][f][s as usize] = S.play + ORB_HIT_CD;
            let light = f == 0 && items::copies(p, IT_TWIN_SHOT) > 0;
            items::orb_hit(p, s, o, if light { 1 } else { 2 }, id);
            items::log_item(b"orb hit", id, p as i32 * 100 + s);
        }
    }
    false
}

unsafe fn shatter(p: usize, f: usize, k: usize, o: P) {
    S.orb_broken[p][f][k] = S.play + ORB_RESPAWN;
    ev(EV_ORB_BREAK, p as i32, FAM[f]);
    items::orb_burst(FAM[f], p, o);
}

/// The owner landed a hit: every shattered orb is back at once.
pub(crate) unsafe fn on_owner_hit(p: usize) {
    if p < NP && S.orb_broken[p].iter().any(|f| f.iter().any(|&b| b != 0)) {
        S.orb_broken[p] = [[0; ORB_MAX]; FAMS];
        log(b"envoy: orbs back (the owner landed a hit), player\0", p as i32);
    }
}

/// A new match: every orb whole.
pub(crate) unsafe fn reset_match() {
    S.orb_broken = [[[0; ORB_MAX]; FAMS]; NP];
    S.orb_cd = [[[0; 4]; FAMS]; NP];
    S.orb_prev = [-1; NP];
}

/// The orbs out, for the UI (PJ_ORB: index, item id; life 1 while shattered).
pub(crate) unsafe fn publish(v: &mut [i32; VIEW_WORDS], n: &mut usize, look: impl Fn(usize) -> i32) {
    for p in 0..nh() as usize {
        let h = gd_fighter_from_slot(p as i32);
        if h == 0 || !alive(p as i32) {
            continue;
        }
        let st = gd_fighter_get_action_state(h);
        let Some(me) = pos(p as i32) else { continue };
        let face = if gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 };
        for f in 0..FAMS {
            let c = count(p, f, st).min(ORB_MAX as i32);
            for k in 0..c {
                if *n >= PROJ {
                    return;
                }
                let o = place(p, f, k, c, me, face);
                let b = V_PROJ + *n * PJ_W;
                let broken = (S.orb_broken[p][f][k as usize] != 0) as i32;
                v[b..b + PJ_W].copy_from_slice(&[PJ_ORB, p as i32 | look(p), x100(o.x), x100(o.y), S.play, broken, k, FAM[f]]);
                *n += 1;
            }
        }
    }
}

/// 1 / sqrt(x) without std (two Newton steps from a bit guess; deterministic).
fn inv_sqrt(x: f32) -> f32 {
    let mut y = f32::from_bits(0x5F37_59DF - (x.to_bits() >> 1));
    for _ in 0..2 {
        y *= 1.5 - 0.5 * x * y * y;
    }
    y
}
