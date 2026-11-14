//! Envoy items, the first slice (docs/rust-port/notes/envoy-items-design.md 8): twelve hand-made items, each tied to
//! one thing you do in Melee, each with its own visible effect, firing straight off its trigger, stacking without a
//! slot limit. Everything here is sim state inside `St` (hashed every frame, rolled back with the game): the
//! inventory (copies per item id), a fixed projectile pool, the one-shot burst ring the UI draws, the pedestals and
//! the Isaac-style stock limit. Hitboxes go through `gd_envoy_blast` (a real launching hit on the engine path) or the
//! percent hook (a nick); one effect per item per frame, player 1 before player 2.
use super::*;
use envoy_data::procs::{self, Proc, ProcIn, PROC_ITEMS, PROC_MAX, EVOLUTIONS, SYNERGIES};

/// The stock limit's hard cap (heart containers; each player's start is rooms::LIM_START).
pub(crate) const LIM_CAP: i32 = 8;
/// Pedestals wait this long for their owner to walk in, then fly to them.
pub(crate) const PED_TIME: i32 = 900;
/// The shop's stock refill price (before the per-loop rise past loop 5).
pub(crate) const STOCK_REFILL: i32 = 60;

const ORDER: [i32; 38] = [
    IT_SPARK_RING, IT_GLIDE_WAKE, IT_SHINE_BELL, IT_HOP_SHOT, IT_LEDGE_SPIKES, IT_WHIFF_ECHO, IT_ROLL_MINES, IT_SMASH_QUAKE,
    IT_MOON_DROPS, IT_COIN_MAGNET, IT_HOMING_INK, IT_FIRE_LACE,
    // slice 2
    IT_LASER_PRISM, IT_CROUCH_THORNS, IT_PERFECT_GUARD, IT_BRIMSTONE_JAB, IT_DEMON_WINGS, IT_CRIMSON_COUNT, IT_BLOODLUST,
    IT_HALO, IT_GUARDIAN_FEATHER, IT_ICE_GLASS, IT_CHAIN_SPARK, IT_TWIN_SHOT, IT_GRAVITY_WELL, IT_BIG_BANG, IT_BOUNCY,
    IT_SPARE_HEART, IT_DODGE_ORB, IT_NAIR_RING, IT_DASH_ORB, IT_SHIELD_ORBS,
    // the synergy lane
    IT_EMBER_SEED, IT_STATIC_COIL, IT_FROST_BUD, IT_KIN_TETHER, IT_RALLY_BANNER, IT_KINDRED_SPARK,
];
/// Walk steps coprime with ORDER's 38 (a roll visits every item).
const STEPS: [usize; 6] = [1, 3, 5, 9, 11, 13];
/// Slice 2 cooldowns per player (frames): Brimstone Jab, Bloodlust, Crouch Thorns, Perfect Guard, Laser Prism.
const CD_BRIM: usize = 0;
const CD_BLOOD: usize = 1;
const CD_THORNS: usize = 2;
const CD_GUARD: usize = 3;
const CD_PRISM: usize = 4;
const CD_LEN: [i32; 5] = [24, 30, 20, 30, 10];
/// The guardian feather: a hit counts as KO strength at this percent (whole) after it, with at least this damage.
const GUARD_PCT: f32 = 100.0;
const GUARD_DMG: f32 = 8.0;
/// Action states: the jabs, the powershield (GuardReflect).
const AS_JAB: (i32, i32) = (44, 46);
const AS_GUARD_REFLECT: i32 = 182;
/// Item kinds of the Fox / Falco blaster shots.
pub(crate) const IK_LASERS: [i32; 2] = [54, 55];

// action states the triggers read
const AS_AIR_N: i32 = 65;
const AS_ATTACK_LAST: i32 = 69;
const AS_CLIFF_CATCH: i32 = 252;
const AS_ESCAPE_F: i32 = 233;
const AS_ESCAPE_B: i32 = 234;
const AS_SHINE: [i32; 2] = [360, 365];
/// FighterKind: Fox, Falco (the shine's owners); CharacterKind (the run's pair): Fox, Falco.
const FT_FOX: i32 = 1;
const FT_FALCO: i32 = 22;
const CK_FOX: i32 = 2;
const CK_FALCO: i32 = 20;

const TAU: f32 = 6.283_185_5;

pub(crate) fn sinf(x: f32) -> f32 {
    let mut x = x % TAU;
    if x > TAU * 0.5 {
        x -= TAU;
    } else if x < -TAU * 0.5 {
        x += TAU;
    }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}
pub(crate) fn cosf(x: f32) -> f32 {
    sinf(x + TAU * 0.25)
}
/// A square root without std (Newton from a bit guess; deterministic).
fn sqrtf(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut y = f32::from_bits((x.to_bits() >> 1) + 0x1FBD_1DF5);
    for _ in 0..3 {
        y = 0.5 * (y + x / y);
    }
    y
}

pub(crate) unsafe fn copies(p: usize, id: i32) -> i32 {
    if p < NP && (1..ITEM_IDS as i32).contains(&id) { super::lab::gate(id, S.inv[p][id as usize]) } else { 0 }
}

unsafe fn has(p: usize, id: i32) -> bool {
    copies(p, id) > 0
}

/// x1000 of the item's stack curve for player `p`'s copies.
unsafe fn mul(p: usize, id: i32) -> i32 {
    stack_mul(id, copies(p, id))
}

/// `more`: the count (copies, capped).
unsafe fn more(p: usize, id: i32) -> i32 {
    item(id).map_or(1, |it| copies(p, id).clamp(1, it.cap as i32))
}

/// The look flags the player's modifiers put on everything they make.
unsafe fn look(p: usize) -> i32 {
    let f = |id: i32, fl: i32| if has(p, id) { fl } else { 0 };
    f(IT_HOMING_INK, PF_INK) | f(IT_FIRE_LACE, PF_LACE) | f(IT_ICE_GLASS, PF_ICE) | f(IT_CHAIN_SPARK, PF_CHAIN) | f(IT_BOUNCY, PF_BOUNCY)
        | f(IT_GRAVITY_WELL, PF_WELL) | f(IT_BIG_BANG, PF_BIG) | f(IT_HALO, PF_HALO)
}

/// Player `p` gains a copy of item `id` (`how`: a G_* source). Logged; the UI flashes it over the fighter.
pub(crate) unsafe fn give(p: usize, id: i32, how: i32) {
    if p >= NP || !(1..ITEM_IDS as i32).contains(&id) || item(id).is_none() {
        return;
    }
    S.inv[p][id as usize] = (S.inv[p][id as usize] + 1).min(99);
    let n = S.gain[p][5] + 1;
    S.gain[p] = [S.play, id, 0, S.inv[p][id as usize], how, n];
    log_item(b"item taken", id, p as i32 * 100 + S.inv[p][id as usize]);
    evo_check(p);
    if id == IT_SPARE_HEART {
        // on pickup: +1 stock limit and +1 stock (to the cap)
        rooms::add_slot(p, 1);
        log_item(b"item fires", id, p as i32 * 100 + rooms::lim(p));
    }
}

/// `envoy: <what> <item key>` (value: as given).
pub(crate) fn log_item(what: &[u8], id: i32, v: i32) {
    let key = item(id).map_or("?", |it| it.key);
    let mut b = [0u8; 80];
    let mut n = 0;
    for &x in b"envoy: ".iter().chain(what.iter()).chain(b" ".iter()).chain(key.as_bytes().iter()) {
        if n < b.len() - 1 {
            b[n] = x;
            n += 1;
        }
    }
    unsafe { gd_log(b.as_ptr(), v) }
}

/// Whether item `id` may be offered to player `p` from pool bits `pools`: in the pool, not held (unless repeatable),
/// and usable by the pair (Shine Bell only with Fox or Falco in it).
unsafe fn offer_ok(p: usize, id: i32, pools: u8) -> bool {
    let Some(it) = item(id) else { return false };
    if it.pools & pools == 0 || (has(p, id) && !it.repeatable) {
        return false;
    }
    if (id == IT_SHINE_BELL || id == IT_LASER_PRISM) && ![CK_FOX, CK_FALCO].iter().any(|k| S.pair[p].contains(k)) {
        return false;
    }
    // co-op items only in a 2P run
    if it.trigger == TR_COOP && nh() < 2 {
        return false;
    }
    // a pool roll never hands out more Spare Hearts than the limit can take
    if id == IT_SPARE_HEART && rooms::lim(p) >= LIM_CAP {
        return false;
    }
    true
}

/// A seeded pick from `pools` for player `p` (`salt` separates rolls); 0 when nothing is left.
pub(crate) unsafe fn roll(p: usize, pools: u8, salt: u32, skip: &[i32]) -> i32 {
    let mut h = mix(S.seed as u32 ^ 0x17E5_0000 ^ fkey() << 4 ^ salt << 12 ^ (p as u32) << 24);
    let start = (h % ORDER.len() as u32) as usize;
    h = mix(h.wrapping_add(0x6D2B_79F5));
    let step = STEPS[(h % STEPS.len() as u32) as usize];
    for pass in 0..2 {
        for k in 0..ORDER.len() {
            let i = if pass == 0 { (start + k * step) % ORDER.len() } else { (start + k) % ORDER.len() };
            let id = ORDER[i];
            if offer_ok(p, id, pools) && !skip.contains(&id) {
                return id;
            }
        }
    }
    0
}

// ---- the projectile pool and the burst ring

unsafe fn spawn(kind: i32, p: usize, at: P, vx: f32, vy: f32, life: i32, a: i32, b: i32) -> Option<usize> {
    let i = S.proj.iter().position(|q| q[0] == 0)?;
    S.proj[i] = [kind, p as i32, x100(at.x), x100(at.y), 0, life, a, b];
    S.pv[i] = [x100(vx), x100(vy)];
    S.pj_hit[i] = 0;
    S.pf[i] = look(p);
    S.pj_bounce[i] = 0;
    Some(i)
}

/// A moving shot (sparks, stars, laser bolts, feathers): spawned, and with Twin Shot one twin per copy at a small
/// angle, each twin hitting for 60% (its PF_TWIN flag).
unsafe fn shot(kind: i32, p: usize, at: P, vx: f32, vy: f32, life: i32, skip: i32) {
    if let Some(i) = spawn(kind, p, at, vx, vy, life, 0, 0) {
        S.pj_hit[i] = skip;
    }
    for k in 0..copies(p, IT_TWIN_SHOT).min(2) {
        let a = 0.22 * (k + 1) as f32 * if k % 2 == 0 { 1.0 } else { -1.0 };
        let (c, sn) = (cosf(a), sinf(a));
        if let Some(i) = spawn(kind, p, P { x: at.x, y: at.y + 2.0 }, vx * c - vy * sn, vx * sn + vy * c, life, 0, 0) {
            S.pj_hit[i] = skip;
            S.pf[i] |= PF_TWIN;
        }
    }
}

/// A one-shot visual for the UI (a ring, a spike column, a swing, a pop): presentation only.
unsafe fn burst(id: i32, p: usize, at: P, size: f32, dir: i32) {
    let i = S.burst_n as usize % BURSTS;
    S.burst[i] = [S.play, id, p as i32, x100(at.x), x100(at.y), x100(size), look(p) | (dir + 1) & 3];
    S.burst_n += 1;
    if p < NP {
        S.proc_at[p] = S.play;
    }
}

/// Frames since player p's last item proc (a worn look shows only for a short while after one).
pub(crate) unsafe fn proc_age(p: usize) -> i32 {
    if S.proc_at[p] <= 0 { 9999 } else { S.play - S.proc_at[p] }
}

/// The build flourish (fight intro, victory): the items flash around the fighter for a second (UI side).
pub(crate) unsafe fn flourish() {
    S.flourish_at = [S.play.max(1); NP];
}

/// The rarest item player p holds (devil / angel / secret, then boss, then quality): the aura tint's id.
unsafe fn rarest(p: usize) -> i32 {
    let mut best = (0, 0);
    for id in 1..ITEM_IDS as i32 {
        if S.inv[p][id as usize] <= 0 {
            continue;
        }
        let Some(it) = envoy_data::items::item(id) else { continue };
        let r = if it.pools & (PL_DEVIL | PL_ANGEL | PL_SECRET) != 0 { 30 } else if it.pools & PL_BOSS != 0 { 20 } else { 10 } + it.quality as i32;
        if r > best.0 {
            best = (r, id);
        }
    }
    best.1
}

fn pj_pos(q: &[i32; PJ_W]) -> P {
    P { x: q[2] as f32 / 100.0, y: q[3] as f32 / 100.0 }
}

/// Damage on a foe without a launch (a nick): through the foe's toughness, into the cash-out tally; Fire Lace burns.
unsafe fn nick(p: usize, s: i32, milli: i32, src: i32) {
    if !enemy_of(p * 2, s) || milli <= 0 {
        return;
    }
    let m = if s >= nh() { (milli as i64 * foe_def(s) as i64 / 1000) as i32 } else { milli };
    gd_envoy_percent(s, 0, m);
    deal(p, s, m, 1000, T_FIRE, -1, CM_NONE, 0, 0);
    lace(p, s);
    after_hit(p, s, src);
}

/// What every item hit carries on: Crimson Count's sigils burn, Chain Spark arcs on to the next foe (never from an
/// arc: no chain of chains).
static mut IN_CHAIN: bool = false;
unsafe fn after_hit(p: usize, s: i32, src: i32) {
    if S.sigils[p] > 0 {
        burn(s, BURN_STACKS + S.sigils[p].min(3) - 1, true, p as i32);
    }
    if has(p, IT_CHAIN_SPARK) && !IN_CHAIN {
        if let Some(from) = pos(s) {
            let from = P { x: from.x, y: from.y + 8.0 };
            IN_CHAIN = true;
            let mut skip = 1 << s;
            for _ in 0..more(p, IT_CHAIN_SPARK) {
                let t = foe_near(p, from, 70.0, skip);
                let Some(to) = t.ge(&0).then(|| pos(t)).flatten() else { break };
                skip |= 1 << t;
                let to = P { x: to.x, y: to.y + 8.0 };
                spawn(PJ_ARC, p, from, 0.0, 0.0, 12, x100(to.x), x100(to.y));
                tap(p, t, to, 2, IT_CHAIN_SPARK);
                log_item(b"item fires", IT_CHAIN_SPARK, p as i32 * 100 + t);
            }
            IN_CHAIN = false;
        }
    }
    procs_off(p, s, src);
    kindred(p, s, src);
}

unsafe fn lace(p: usize, s: i32) {
    if has(p, IT_FIRE_LACE) {
        let to = BURN_STACKS + copies(p, IT_FIRE_LACE).min(3) - 1;
        burn(s, to, true, p as i32);
    }
}

/// A launching hit at `at` (radius `r`, `pct` percent, knockback angle / growth / base) on the engine's hit path;
/// when no capsule is free, a nick on `target` instead. Fire Lace burns whoever stands in it.
unsafe fn boom(p: usize, at: P, r: f32, pct: i32, angle: i32, kbg: i32, bkb: i32, target: i32) {
    // the hit's source item (its look and its procs); taken and cleared, so a stale id never carries into a later
    // boom (BOOM_ID is not sim state: a rollback must not see a value from a frame it rewound)
    let src = BOOM_ID;
    BOOM_ID = 0;
    // Gravity Well: the burst pulls in first (a well that bursts at the end of its life, proj_frame)
    if has(p, IT_GRAVITY_WELL) && !IN_WELL {
        let rr = r * mul(p, IT_GRAVITY_WELL) as f32 / 1000.0;
        if let Some(i) = spawn(PJ_WELL, p, at, 0.0, 0.0, 16, x100(rr), pct & 0xFF | (kbg & 0xFF) << 8 | (bkb & 0xFF) << 16) {
            S.pv[i] = [angle, target];
            log_item(b"item fires", IT_GRAVITY_WELL, p as i32 * 100 + rr as i32);
            return;
        }
    }
    // Big Bang: twice the radius (more per copy), a quake, a fifth less damage
    let (r, pct) = if has(p, IT_BIG_BANG) {
        S.shake = S.play.max(1);
        (r * (1.5 + 0.5 * copies(p, IT_BIG_BANG).min(2) as f32), (pct * 4 / 5).max(1))
    } else {
        (r, pct)
    };
    let rc = gd_envoy_blast(p as i32, at.x, at.y, r, pct, angle, kbg, bkb);
    if rc == 0 {
        S.blasts += 1;
        let (el, k, sev) = hit_look(src);
        gd_envoy_blast_look(p as i32, el, k, sev);
        if has(p, IT_FIRE_LACE) {
            for s in 0..4 {
                if enemy_of(p * 2, s) && pos(s).is_some_and(|q| d2(P { x: q.x, y: q.y + 8.0 }, at) < (r + 6.0) * (r + 6.0)) {
                    lace(p, s);
                }
            }
        }
    } else if target >= 0 {
        nick(p, target, pct * 1000, src);
        return;
    }
    if target >= 0 && rc == 0 {
        after_hit(p, target, src);
    }
}
static mut IN_WELL: bool = false;

/// The item whose hit is being made now (its look: the hit spark and the Melee hit sound, `hit_look`).
static mut BOOM_ID: i32 = 0;

/// The Spark Ring bar (envoy-items-design.md 11.6): every item hit lands as an engine hit with the item's own hit
/// element (its spark: 0 normal, 1 fire, 2 electric, 3 slash) and Melee-disc hit sound family (`sfx_kind`) and
/// severity, so it reads and sounds without the Brawl disc. (element, sfx kind, severity).
fn hit_look(id: i32) -> (i32, i32, i32) {
    match id {
        IT_SPARK_RING | IT_LASER_PRISM => (2, 6, 0),
        IT_GLIDE_WAKE => (1, 7, 0),
        IT_SHINE_BELL => (2, 6, 1),
        IT_HOP_SHOT => (0, 3, 1),
        IT_LEDGE_SPIKES => (3, 2, 1),
        IT_WHIFF_ECHO => (0, 0, 1),
        IT_ROLL_MINES => (1, 7, 2),
        IT_SMASH_QUAKE => (0, 4, 2),
        IT_MOON_DROPS => (0, 5, 0),
        IT_CROUCH_THORNS => (3, 2, 0),
        IT_PERFECT_GUARD => (2, 3, 2),
        IT_BRIMSTONE_JAB => (1, 7, 2),
        IT_DEMON_WINGS => (3, 2, 0),
        IT_BLOODLUST => (3, 1, 1),
        IT_CHAIN_SPARK => (2, 6, 1),
        IT_ICE_GLASS => (0, 2, 0),
        IT_GRAVITY_WELL => (0, 4, 2),
        // the synergy lane: each its own spark and sound family
        IT_EMBER_SEED => (1, 5, 1),
        IT_STATIC_COIL => (2, 2, 0),
        IT_FROST_BUD => (0, 2, 2),
        IT_KIN_TETHER => (2, 5, 0),
        IT_KINDRED_SPARK => (3, 6, 1),
        _ => (0, 0, 1),
    }
}

/// Moon Drops' hit (and every light item hit: sparks, arcs, shards, feathers): percent, knockback growth, base.
/// A light flinch with a small push (tunable); it lands on the engine hit path, so it rolls back with the fighter.
const TAP_KBG: i32 = 18;
const TAP_BKB: i32 = 22;

/// An orb's hit (orbs.rs): the light engine hit with the orb item's look.
pub(crate) unsafe fn orb_hit(p: usize, s: i32, at: P, pct: i32, id: i32) {
    if id == IT_MOON_DROPS && evo_on(p, procs::EVO_ECLIPSE) {
        // Eclipse: black moons with gold rims, twice as hard
        tap(p, s, at, pct * 2, id);
        burst(VFX_EVO + procs::EVO_ECLIPSE as i32, p, at, 9.0, 0);
    } else {
        tap(p, s, at, pct, id);
        burst(id, p, at, 5.0, 0);
    }
    if id == IT_MOON_DROPS && syn_on(p, procs::SY_BLOOD_MOON) {
        burn(s, BURN_STACKS, true, p as i32);
        burst(VFX_SYN + procs::SY_BLOOD_MOON as i32, p, at, 6.0, 0);
    }
}

/// An orb shattered: its pop (the UI's burst of shards; the event carries the sound).
pub(crate) unsafe fn orb_burst(id: i32, p: usize, at: P) {
    burst(id, p, at, 9.0, 3);
}

/// A light item hit on foe `s` at `at`: an engine hit (spark, sound, flinch) with item `id`'s look; a nick when no
/// hit capsule is free. Never a well or a Big Bang (those change bursts, not shots).
unsafe fn tap(p: usize, s: i32, at: P, pct: i32, id: i32) {
    if !enemy_of(p * 2, s) {
        return;
    }
    let c = pos(s).map_or(at, |q| P { x: q.x, y: q.y + 8.0 });
    let pct = pct + rallied(p) as i32;
    if gd_envoy_blast(p as i32, c.x, c.y, 5.0, pct.max(1), 361, TAP_KBG, TAP_BKB) == 0 {
        S.blasts += 1;
        let (el, k, sev) = hit_look(id);
        gd_envoy_blast_look(p as i32, el, k, sev);
        lace(p, s);
        after_hit(p, s, id);
    } else {
        nick(p, s, pct * 1000, id);
    }
}

/// The live foe of player `p` nearest `at` (centre of mass), within `reach`.
unsafe fn foe_near(p: usize, at: P, reach: f32, skip_bits: i32) -> i32 {
    let mut best = -1;
    let mut bd = reach * reach;
    for s in 0..4 {
        if skip_bits >> s & 1 != 0 || !enemy_of(p * 2, s) {
            continue;
        }
        if let Some(q) = pos(s) {
            let d = d2(P { x: q.x, y: q.y + 8.0 }, at);
            if d < bd {
                bd = d;
                best = s;
            }
        }
    }
    best
}

// ---- triggers

/// A hit player `p` landed on foe `v` (a move record: tags, the attacker's action state). Spark Ring on a nair hit
/// (once per nair), Smash Quake on a smash hit (once per smash); and the whiff watch learns the move connected.
pub(crate) unsafe fn on_hit(p: usize, v: i32, tags: i32, ast: i32) {
    if p >= nh() as usize || !is_foe(v) && v != S.boss_slot {
        return;
    }
    S.it_atk_hit[p] = 1;
    orbs::on_owner_hit(p);
    let Some(vp) = pos(v) else { return };
    let vc = P { x: vp.x, y: vp.y + 8.0 };
    if ast == AS_AIR_N && S.it_nair[p] == 0 {
        S.it_nair[p] = 1;
        super::lab::trig(p, TR_NAIR_HIT as i32);
        if has(p, IT_SPARK_RING) {
            item_spark_ring(p, v, vc);
        }
    }
    if tags & T_SMASH != 0 && S.it_smash[p] == 0 {
        S.it_smash[p] = 1;
        super::lab::trig(p, TR_SMASH_HIT as i32);
        if has(p, IT_SMASH_QUAKE) {
            item_smash_quake(p, v, vp);
        }
    }
    if (AS_JAB.0..=AS_JAB.1).contains(&ast) {
        log(b"envoy: technique jab hit, player\0", p as i32);
        if has(p, IT_BRIMSTONE_JAB) && cd_ok(p, CD_BRIM) {
            item_brimstone_jab(p);
        }
    }
    if tags & T_LASER != 0 {
        log(b"envoy: technique laser hit, player*10 + victim\0", p as i32 * 10 + v);
        if has(p, IT_LASER_PRISM) && cd_ok(p, CD_PRISM) {
            item_laser_prism(p, v, vc);
        }
    }
}

/// A cooldown slot ready (and taken now).
unsafe fn cd_ok(p: usize, k: usize) -> bool {
    if S.play < S.it_cd[p][k] {
        return false;
    }
    S.it_cd[p][k] = S.play + CD_LEN[k];
    true
}

/// Player `p` took a hit from foe `a` (the frame's records): Bloodlust lashes back.
pub(crate) unsafe fn on_taken(p: usize, a: i32) {
    if p >= nh() as usize || !has(p, IT_BLOODLUST) || a < 0 || !cd_ok(p, CD_BLOOD) {
        return;
    }
    item_bloodlust(p, a);
    if has(p, IT_CROUCH_THORNS) && syn_on(p, procs::SY_IRON_MAIDEN) {
        if let Some(me) = pos(p as i32) {
            item_crouch_thorns(p, me);
            burst(VFX_SYN + procs::SY_IRON_MAIDEN as i32, p, P { x: me.x, y: me.y + 9.0 }, 14.0, 0);
        }
    }
}

/// Player `p` is about to take `d` percent: the Guardian Feather eats a KO-strength hit, once a stock per copy.
/// True: the hit is cancelled (the pre hook skips it).
pub(crate) unsafe fn guard_eats(p: usize, h: u32, d: f32) -> bool {
    if p >= nh() as usize || !has(p, IT_GUARDIAN_FEATHER) || S.guard_used[p] >= more(p, IT_GUARDIAN_FEATHER) || d < GUARD_DMG {
        return false;
    }
    let mut w = [0u32; 20];
    gd_fighter_read(h, w.as_mut_ptr());
    let pct = f32::from_bits(w[10]);
    if pct + d < GUARD_PCT {
        return false;
    }
    S.guard_used[p] += 1;
    let left = more(p, IT_GUARDIAN_FEATHER) - S.guard_used[p];
    ev(EV_GUARD, p as i32, left);
    if let Some(me) = pos(p as i32) {
        burst(IT_GUARDIAN_FEATHER, p, P { x: me.x, y: me.y + 9.0 }, 18.0, 0);
    }
    log_item(b"item fires", IT_GUARDIAN_FEATHER, p as i32 * 100 + left);
    log(b"envoy: guardian feather ate a KO hit, percent*100 + damage\0", (pct as i32) * 100 + d as i32);
    true
}

/// A player lost a stock: the Guardian Feather is back for the next one.
pub(crate) unsafe fn on_stock_lost(p: usize) {
    if p < NP {
        S.guard_used[p] = 0;
    }
}

/// A foe was KO'd (lost a stock): Coin Magnet pays every player who holds it.
pub(crate) unsafe fn on_ko(s: i32) {
    for p in 0..nh() as usize {
        super::lab::trig(p, TR_KO as i32);
        if has(p, IT_COIN_MAGNET) {
            item_coin_magnet(p, s);
        }
        if has(p, IT_CRIMSON_COUNT) {
            item_crimson_count(p);
        }
        if has(p, IT_RALLY_BANNER) {
            item_rally_banner(p);
        }
    }
}

/// Per unfrozen live frame, after the records: the polled triggers (wavedash, shine, short hop apex, ledge grab,
/// roll, whiff), then every projectile and the moons. `ev_bits`: each player's engine event bits this frame.
pub(crate) unsafe fn frame(ev_bits: [u32; NP]) {
    for p in 0..nh() as usize {
        poll(p, ev_bits[p]);
    }
    if demo_on() {
        demo_frame();
    }
    proj_frame();
    orbs::frame();
    for p in 0..nh() as usize {
        if has(p, IT_DEMON_WINGS) {
            item_demon_wings(p);
        }
        if has(p, IT_KIN_TETHER) {
            item_kin_tether(p);
        }
    }
}

/// A wavedash was registered for player `p` (tech_frame's detection).
pub(crate) unsafe fn on_wavedash(p: usize) {
    super::lab::trig(p, TR_WAVEDASH as i32);
    if has(p, IT_GLIDE_WAKE) {
        item_glide_wake(p);
    }
}

unsafe fn poll(p: usize, bits: u32) {
    let h = gd_fighter_from_slot(p as i32);
    if h == 0 || !alive(p as i32) {
        S.it_prev[p] = -1;
        return;
    }
    let st = gd_fighter_get_action_state(h);
    let prev = S.it_prev[p];
    S.it_prev[p] = st;
    let me = P { x: gd_fighter_get_pos_x(h), y: gd_fighter_get_pos_y(h) };
    // the engine's shield-hit bit: the attack touched a shield, so it is not a whiff
    if bits & 8 != 0 && S.it_atk[p] > 0 && S.it_atk_hit[p] == 0 {
        S.it_atk_hit[p] = 2;
    }
    // the engine's crouch-cancel bit: a hit's knockback was cut by crouching
    if bits & 32 != 0 {
        log(b"envoy: technique crouch cancel, player\0", p as i32);
        if has(p, IT_CROUCH_THORNS) && cd_ok(p, CD_THORNS) {
            item_crouch_thorns(p, me);
        }
    }
    // the short hop: armed at the jump (the engine's bit), fired at the apex (vertical speed turns down)
    if bits & 2 != 0 {
        S.it_hop[p] = 1;
        S.it_hop_t[p] = 0;
    } else if bits & 4 != 0 {
        S.it_hop[p] = 0;
    }
    if S.it_hop[p] != 0 {
        S.it_hop_t[p] += 1;
        let mut w = [0u32; 20];
        gd_fighter_read(h, w.as_mut_ptr());
        let vy = f32::from_bits(w[4]);
        if S.it_hop_t[p] > 2 && vy <= 0.0 {
            S.it_hop[p] = 0;
            log(b"envoy: technique short hop apex, player\0", p as i32);
            super::lab::trig(p, TR_SHORT_HOP as i32);
            if has(p, IT_HOP_SHOT) {
                item_hop_shot(p, me);
            }
        } else if S.it_hop_t[p] > 45 {
            S.it_hop[p] = 0;
        }
    }
    if st == prev {
        return;
    }
    // a new move: the nair / smash once-per-move flags reset
    S.it_nair[p] = 0;
    S.it_smash[p] = 0;
    // the whiff watch: an attack state ended (into anything else) with no hit registered
    let watched = S.it_atk[p] - 1;
    if watched >= AS_ATTACK0 && st != watched {
        if S.it_atk_hit[p] == 0 {
            log(b"envoy: technique whiff, player*1000 + move\0", p as i32 * 1000 + watched);
            super::lab::trig(p, TR_WHIFF as i32);
            if has(p, IT_WHIFF_ECHO) {
                item_whiff_echo(p);
            }
        } else if S.it_atk_hit[p] == 2 {
            log(b"envoy: attack ended on a shield, not a whiff, player*1000 + move\0", p as i32 * 1000 + watched);
        }
        S.it_atk[p] = 0;
    }
    if (AS_ATTACK0..=AS_ATTACK_LAST).contains(&st) {
        S.it_atk[p] = st + 1;
        S.it_atk_hit[p] = 0;
        S.it_atk_at[p] = [x100(me.x), x100(me.y), if gd_fighter_get_facing(h) < 0.0 { -1 } else { 1 }];
    }
    let kind = gd_fighter_kind(h);
    if AS_SHINE.contains(&st) && (kind == FT_FOX || kind == FT_FALCO) {
        log(b"envoy: technique shine, player\0", p as i32);
        super::lab::trig(p, TR_SHINE as i32);
        if has(p, IT_SHINE_BELL) {
            item_shine_bell(p, me);
        }
    }
    if st == AS_CLIFF_CATCH {
        log(b"envoy: technique ledge grab, player\0", p as i32);
        super::lab::trig(p, TR_LEDGE_GRAB as i32);
        if has(p, IT_LEDGE_SPIKES) {
            item_ledge_spikes(p, me);
        }
    }
    if st == AS_GUARD_REFLECT {
        log(b"envoy: technique powershield, player\0", p as i32);
        if has(p, IT_PERFECT_GUARD) && cd_ok(p, CD_GUARD) {
            item_perfect_guard(p, me);
        }
    }
    if st == AS_ESCAPE_F || st == AS_ESCAPE_B {
        log(b"envoy: technique roll, player\0", p as i32);
        super::lab::trig(p, TR_ROLL as i32);
        if has(p, IT_ROLL_MINES) {
            item_roll_mines(p, me);
        }
    }
}

// ---- the twelve effects (item_<name>: the design's one hand-written function per item)

/// 1 Spark Ring: the nair hit pops a ring of 6 sparks (6 more per copy) that fly out and nick anyone they touch.
unsafe fn item_spark_ring(p: usize, v: i32, at: P) {
    let n = 6 * more(p, IT_SPARK_RING);
    let turn = (S.play % 7) as f32 * 0.13;
    for k in 0..n {
        let a = turn + k as f32 * TAU / n as f32;
        shot(PJ_SPARK, p, at, cosf(a) * 1.7, sinf(a) * 1.7, 28, 1 << v); // the nair's own victim is not nicked again
    }
    burst(IT_SPARK_RING, p, at, 10.0, 0);
    log_item(b"item fires", IT_SPARK_RING, p as i32 * 100 + n);
    if evo_on(p, procs::EVO_STORM_CROWN) {
        // Storm Crown: a crown of lightning strikes up to three foes near the nair's victim
        let mut skip = 0;
        for _ in 0..3 {
            let t = foe_near(p, at, 60.0, skip);
            let Some(to) = t.ge(&0).then(|| pos(t)).flatten() else { break };
            skip |= 1 << t;
            let to = P { x: to.x, y: to.y + 8.0 };
            spawn(PJ_ARC, p, P { x: to.x, y: to.y + 40.0 }, 0.0, 0.0, 14, x100(to.x), x100(to.y));
            tap(p, t, to, 3, IT_CHAIN_SPARK);
        }
        burst(VFX_EVO + procs::EVO_STORM_CROWN as i32, p, at, 16.0, 0);
        log_syn(b"evolution fires", EVOLUTIONS[procs::EVO_STORM_CROWN].key, p as i32);
    }
}

/// 2 Glide Wake: the wavedash leaves a burning streak on the floor for 1.5 s; foes in it flinch and burn.
unsafe fn item_glide_wake(p: usize) {
    let Some(me) = pos(p as i32) else { return };
    let half = 12.0 * mul(p, IT_GLIDE_WAKE) as f32 / 1000.0;
    spawn(PJ_WAKE, p, P { x: me.x, y: me.y + 0.5 }, 0.0, 0.0, 90, x100(half), 0);
    burst(IT_GLIDE_WAKE, p, me, half, 0);
    log_item(b"item fires", IT_GLIDE_WAKE, p as i32 * 100 + half as i32);
}

/// 3 Shine Bell: each shine rings a shock ring that pushes foes out (1.5x the shine's reach, bigger per copy).
unsafe fn item_shine_bell(p: usize, me: P) {
    BOOM_ID = IT_SHINE_BELL;
    let r = 15.0 * mul(p, IT_SHINE_BELL) as f32 / 1000.0;
    let c = P { x: me.x, y: me.y + 7.0 };
    boom(p, c, r, 3, 361, 70, 50, foe_near(p, c, r, 0));
    burst(IT_SHINE_BELL, p, c, r, 0);
    log_item(b"item fires", IT_SHINE_BELL, p as i32 * 100 + r as i32);
    // Thunder Bell (with Chain Spark): the ring arcs to every foe within 2.5x its reach
    if has(p, IT_CHAIN_SPARK) && foe_near(p, c, r * 2.5, 0) >= 0 && syn_on(p, procs::SY_THUNDER_BELL) {
        let mut skip = 0;
        loop {
            let t = foe_near(p, c, r * 2.5, skip);
            let Some(to) = t.ge(&0).then(|| pos(t)).flatten() else { break };
            skip |= 1 << t;
            let to = P { x: to.x, y: to.y + 8.0 };
            spawn(PJ_ARC, p, c, 0.0, 0.0, 12, x100(to.x), x100(to.y));
            tap(p, t, to, 2, IT_CHAIN_SPARK);
        }
        burst(VFX_SYN + procs::SY_THUNDER_BELL as i32, p, c, r * 2.5, 0);
    }
}

/// 4 Hop Shot: at the top of a short hop a star arcs toward the nearest foe (one more per copy, fanned).
unsafe fn item_hop_shot(p: usize, me: P) {
    // Seeker Stars (with Homing Ink): one more star
    let n = more(p, IT_HOP_SHOT) + (has(p, IT_HOMING_INK) && syn_on(p, procs::SY_SEEKER_STARS)) as i32;
    let c = P { x: me.x, y: me.y + 9.0 };
    let t = foe_near(p, c, 400.0, 0);
    let (dx, dy) = match t.ge(&0).then(|| pos(t)).flatten() {
        Some(q) => (q.x - c.x, q.y + 8.0 - c.y + 10.0),
        None => {
            let h = gd_fighter_from_slot(p as i32);
            (if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 }, 0.2)
        }
    };
    let len = sqrtf(dx * dx + dy * dy).max(0.001);
    for k in 0..n {
        let spread = (k as f32 - (n - 1) as f32 * 0.5) * 0.22;
        let (ux, uy) = (dx / len, dy / len);
        let (cs, sn) = (cosf(spread), sinf(spread));
        let (vx, vy) = ((ux * cs - uy * sn) * 2.6, (ux * sn + uy * cs) * 2.6);
        shot(PJ_STAR, p, c, vx, vy, 80, 0);
    }
    burst(IT_HOP_SHOT, p, c, 6.0, 0);
    log_item(b"item fires", IT_HOP_SHOT, p as i32 * 100 + n);
}

/// 5 Ledge Spikes: grabbing ledge stabs a column of ice spikes up from the lip, launching anyone above it.
unsafe fn item_ledge_spikes(p: usize, me: P) {
    BOOM_ID = IT_LEDGE_SPIKES;
    let g = gd_stage_grkind();
    let lx = STAGES.iter().position(|&k| k == g).map_or(me.x, |i| if me.x < 0.0 { -LEDGE_X[i] + 6.0 } else { LEDGE_X[i] - 6.0 });
    let at = P { x: lx, y: 10.0 };
    let m = mul(p, IT_LEDGE_SPIKES);
    boom(p, at, 11.0, 6 * m / 1000, 90, 60, 45, foe_near(p, at, 16.0, 0));
    burst(IT_LEDGE_SPIKES, p, P { x: lx, y: 0.0 }, 30.0 * m as f32 / 1000.0, 0);
    log_item(b"item fires", IT_LEDGE_SPIKES, p as i32 * 100 + m / 100);
}

/// 6 Whiff Echo: a whiffed attack leaves a ghost of the fighter that swings again 20 frames later (one more ghost
/// per copy, 8 frames apart).
unsafe fn item_whiff_echo(p: usize) {
    let [x, y, face] = S.it_atk_at[p];
    let at = P { x: x as f32 / 100.0, y: y as f32 / 100.0 };
    for k in 0..more(p, IT_WHIFF_ECHO) {
        spawn(PJ_GHOST, p, at, 0.0, 0.0, 34 + 8 * k, face, 20 + 8 * k);
    }
    log_item(b"item fires", IT_WHIFF_ECHO, p as i32 * 100 + more(p, IT_WHIFF_ECHO));
}

/// 7 Roll Mines: rolling drops a blinking mine where the roll began; it pops after 1 s or when a foe touches it.
unsafe fn item_roll_mines(p: usize, me: P) {
    let n = more(p, IT_ROLL_MINES);
    for k in 0..n {
        let off = (k as f32 - (n - 1) as f32 * 0.5) * 7.0;
        spawn(PJ_MINE, p, P { x: me.x + off, y: me.y + 1.5 }, 0.0, 0.0, 60 + 6 * k, 0, 0);
    }
    log_item(b"item fires", IT_ROLL_MINES, p as i32 * 100 + n);
}

/// 12 Smash Quake: a smash that lands cracks the floor; a shockwave runs along the stage both ways.
unsafe fn item_smash_quake(p: usize, v: i32, vp: P) {
    let me = pos(p as i32).unwrap_or(vp);
    let m = mul(p, IT_SMASH_QUAKE);
    let tect = has(p, IT_BIG_BANG) && syn_on(p, procs::SY_TECTONIC);
    let size = 7.0 * m as f32 / 1000.0 * if tect { 1.5 } else { 1.0 };
    let life = (26 + 8 * (m / 500 - 2)) * if tect { 2 } else { 1 };
    if tect {
        burst(VFX_SYN + procs::SY_TECTONIC as i32, p, P { x: vp.x, y: me.y }, size * 3.0, 0);
    }
    for dir in [-1, 1] {
        if let Some(i) = spawn(PJ_QUAKE, p, P { x: vp.x, y: me.y }, 3.2 * dir as f32, 0.0, life, dir, x100(size)) {
            S.pj_hit[i] = 1 << v;
        }
    }
    burst(IT_SMASH_QUAKE, p, P { x: vp.x, y: me.y }, size * 2.0, 0);
    log_item(b"item fires", IT_SMASH_QUAKE, p as i32 * 100 + m / 100);
}

/// 15 Moon Drops: two moons (two more per copy) orbit the fighter and nick foes they pass through.
#[allow(dead_code)]
unsafe fn item_moon_drops(p: usize) {
    let Some(me) = pos(p as i32) else { return };
    let n = moons(p);
    for s in 0..4 {
        if !enemy_of(p * 2, s) || S.play < S.moon_cd[p][s as usize] {
            continue;
        }
        let Some(q) = pos(s) else { continue };
        let fc = P { x: q.x, y: q.y + 8.0 };
        for k in 0..n {
            let m = moon_at(me, S.play, k, n);
            if d2(m, fc) < 8.0 * 8.0 {
                S.moon_cd[p][s as usize] = S.play + 30;
                // the Spark Ring bar: an engine hit with its own spark, sound and a light flinch; Twin Shot: twice the
                // moons, each lighter
                tap(p, s, m, if has(p, IT_TWIN_SHOT) { 1 } else { 2 }, IT_MOON_DROPS);
                burst(IT_MOON_DROPS, p, m, 4.0, 0);
                log_item(b"item nick", IT_MOON_DROPS, s);
                break;
            }
        }
    }
}

/// Moon Drops' moons: two a copy, doubled by Twin Shot.
unsafe fn moons(p: usize) -> i32 {
    2 * more(p, IT_MOON_DROPS) * if has(p, IT_TWIN_SHOT) { 2 } else { 1 }
}

/// Moon `k` of `n` around a fighter at `me` on play frame `t` (the UI is handed the same places).
pub(crate) fn moon_at(me: P, t: i32, k: i32, n: i32) -> P {
    let a = t as f32 * 0.075 + k as f32 * TAU / n.max(1) as f32;
    P { x: me.x + cosf(a) * 14.0, y: me.y + 9.0 + sinf(a) * 11.0 }
}

/// 18 Coin Magnet: a KO'd foe bursts into coins that fly to the player (+5 coins a KO, x copies to 3).
unsafe fn item_coin_magnet(p: usize, s: i32) {
    let c = 5 * more(p, IT_COIN_MAGNET);
    HH.purse += c as i64 * COIN_UNIT;
    HH.earned[p] += c as i64 * COIN_UNIT;
    ev(EV_COINS, p as i32, c);
    let from = pos(s).map_or(P { x: 0.0, y: 40.0 }, |q| P { x: q.x.clamp(-110.0, 110.0), y: q.y.clamp(0.0, 110.0) });
    for k in 0..6 {
        let a = k as f32 * TAU / 6.0;
        spawn(PJ_COIN, p, from, cosf(a) * 2.0, sinf(a) * 2.0 + 1.0, 50, p as i32, 0);
    }
    burst(IT_COIN_MAGNET, p, from, 12.0, 0);
    log_item(b"item coins", IT_COIN_MAGNET, c);
}

// ---- slice 2's effects

/// 8 Laser Prism: a laser that hits splits into three thin bolts fanning past the target (three more per copy).
unsafe fn item_laser_prism(p: usize, v: i32, at: P) {
    let dir = pos(p as i32).map_or(1.0, |m| if at.x >= m.x { 1.0 } else { -1.0 });
    let spectrum = has(p, IT_HALO) && syn_on(p, procs::SY_SPECTRUM);
    let n = if spectrum { 5 } else { 3 } * more(p, IT_LASER_PRISM);
    if spectrum {
        burst(VFX_SYN + procs::SY_SPECTRUM as i32, p, at, 12.0, 0);
    }
    for k in 0..n {
        let a = (k as f32 - (n - 1) as f32 * 0.5) * 0.28;
        shot(PJ_PRISM, p, at, cosf(a) * 3.4 * dir, sinf(a) * 3.4, 30, 1 << v);
    }
    burst(IT_LASER_PRISM, p, at, 8.0, dir as i32);
    log_item(b"item fires", IT_LASER_PRISM, p as i32 * 100 + n);
}

/// 21 Crouch Thorns: a crouch-cancelled hit grows thorns from the back that strike everyone close (the attacker).
unsafe fn item_crouch_thorns(p: usize, me: P) {
    let size = 13.0 * mul(p, IT_CROUCH_THORNS) as f32 / 1000.0;
    let h = gd_fighter_from_slot(p as i32);
    let face = if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1 } else { 1 };
    spawn(PJ_THORNS, p, P { x: me.x, y: me.y + 6.0 }, 0.0, 0.0, 22, face, x100(size));
    burst(IT_CROUCH_THORNS, p, P { x: me.x, y: me.y + 8.0 }, size, face);
    log_item(b"item fires", IT_CROUCH_THORNS, p as i32 * 100 + size as i32);
}

/// 26 Perfect Guard: a powershield fires a gold beam at the nearest foe's side.
unsafe fn item_perfect_guard(p: usize, me: P) {
    let c = P { x: me.x, y: me.y + 9.0 };
    let t = foe_near(p, c, 400.0, 0);
    let dir = t.ge(&0).then(|| pos(t)).flatten().map_or(1.0, |f| if f.x >= me.x { 1.0 } else { -1.0 });
    spawn(PJ_BEAM, p, c, 0.0, 0.0, 16, x100(110.0 * dir), BEAM_GOLD);
    burst(IT_PERFECT_GUARD, p, c, 12.0, dir as i32);
    log_item(b"item fires", IT_PERFECT_GUARD, p as i32 * 100 + t);
}

/// 27 Brimstone Jab: a jab that hits fires a blood-red beam across the stage (longer per copy).
unsafe fn item_brimstone_jab(p: usize) {
    let Some(me) = pos(p as i32) else { return };
    let h = gd_fighter_from_slot(p as i32);
    let dir = if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 };
    let len = 80.0 * mul(p, IT_BRIMSTONE_JAB) as f32 / 1000.0;
    let c = P { x: me.x + dir * 6.0, y: me.y + 9.0 };
    spawn(PJ_BEAM, p, c, 0.0, 0.0, 18, x100(len * dir), BEAM_BRIMSTONE);
    burst(IT_BRIMSTONE_JAB, p, c, 10.0, dir as i32);
    log_item(b"item fires", IT_BRIMSTONE_JAB, p as i32 * 100 + len as i32);
}

/// 28 Demon Wings: one more air jump per copy; each extra jump rains a fan of feathers down.
unsafe fn item_demon_wings(p: usize) {
    let j = gd_envoy_jumps(p as i32);
    if j < 0 {
        return;
    }
    let (used, max) = (j & 0xFF, j >> 8 & 0xFF);
    if used <= 1 {
        S.wings[p] = 0; // on the ground (or the first jump only): the wings are ready again
        return;
    }
    let extra = more(p, IT_DEMON_WINGS);
    if used >= max && S.wings[p] < 2 * extra {
        if S.wings[p] % 2 == 0 {
            // the last jump is spent: one comes back (the extra jump)
            if gd_envoy_air_jump(p as i32) == 0 {
                S.wings[p] += 1;
                log(b"envoy: demon wings, an extra air jump, player\0", p as i32);
            }
        } else {
            // the extra jump was used: the feather fan
            S.wings[p] += 1;
            if let Some(me) = pos(p as i32) {
                for k in 0..5 {
                    let a = -1.57 + (k as f32 - 2.0) * 0.3;
                    shot(PJ_FEATHER, p, P { x: me.x, y: me.y + 4.0 }, cosf(a) * 1.6, sinf(a) * 1.6, 50, 0);
                }
                burst(IT_DEMON_WINGS, p, P { x: me.x, y: me.y + 10.0 }, 14.0, 0);
            }
            log_item(b"item fires", IT_DEMON_WINGS, p as i32 * 100 + S.wings[p]);
        }
    }
}

/// 29 Crimson Count: every KO adds a red sigil (to 3 a copy, for the fight); each sigil burns on every item hit.
unsafe fn item_crimson_count(p: usize) {
    S.sigils[p] = (S.sigils[p] + 1).min(3 * more(p, IT_CRIMSON_COUNT));
    if let Some(me) = pos(p as i32) {
        burst(IT_CRIMSON_COUNT, p, P { x: me.x, y: me.y + 9.0 }, 12.0, 0);
    }
    log_item(b"item fires", IT_CRIMSON_COUNT, p as i32 * 100 + S.sigils[p]);
}

/// 30 Bloodlust: when hit, red tendrils lash the attacker (a launching hit 6 frames later).
unsafe fn item_bloodlust(p: usize, a: i32) {
    let Some(me) = pos(p as i32) else { return };
    let c = P { x: me.x, y: me.y + 9.0 };
    if let Some(i) = spawn(PJ_TENDRIL, p, c, 0.0, 0.0, 14, a, 6) {
        S.pj_hit[i] = 0;
    }
    log_item(b"item fires", IT_BLOODLUST, p as i32 * 100 + a);
}

// 31 Halo, 37 Ice Glass, 38 Chain Spark, 39 Twin Shot, 40 Gravity Well, 41 Big Bang and 42 Bouncy are modifiers:
// Halo pierces (proj_frame) and is worn (publish), Ice Glass shatters shots (shot_hit), Chain Spark arcs on every
// item hit (after_hit), Twin Shot twins shots (shot) and moons, Gravity Well and Big Bang change bursts (boom),
// Bouncy bounces shots (proj_frame). 32 Guardian Feather eats a KO hit (guard_eats); 43 Spare Heart is its pickup.

// 35 Homing Ink and 36 Fire Lace are modifiers: Homing Ink steers every projectile (proj_frame), Fire Lace burns on
// every item hit (nick / boom) and puts embers on every effect (the PF_LACE look flag).

// ---- the synergy lane (envoy-synergy.md): proc chains, synergies, evolutions, co-op items

/// Burst ids for the lane's own visuals (the UI draws them in the synergy's / evolution's colour).
pub(crate) const VFX_SYN: i32 = 100;
pub(crate) const VFX_EVO: i32 = 120;

/// Inside a chain's own hits: they never start another chain (resolve already worked the whole chain out). Set and
/// cleared within one call, so never carried across frames (not sim state, nothing for a rollback to restore).
static mut IN_PROC: bool = false;

/// A partner's Rally Banner is lifting player p now.
unsafe fn rallied(p: usize) -> bool {
    p < NP && S.rally_until[p] > S.play
}

/// The resolver's input for player p: the run seed, the frame, the nonce, the copies and the bonuses.
unsafe fn proc_in(p: usize) -> ProcIn {
    let mut inp = ProcIn { seed: S.seed as u32, frame: S.play, nonce: S.proc_n as u32, player: p as u8, ..Default::default() };
    for (k, q) in PROC_ITEMS.iter().enumerate() {
        inp.copies[k] = copies(p, q.id).clamp(0, 3) as u8;
        if rallied(p) {
            inp.bonus[k] += 500;
        }
    }
    // Wildfire (Ember Seed + Fire Lace): Ember Seed procs twice as often; Inferno Seed (evolved) +50% more
    if has(p, IT_FIRE_LACE) && has(p, IT_EMBER_SEED) {
        inp.bonus[0] += 1000;
        if evo_on(p, procs::EVO_INFERNO_SEED) {
            inp.bonus[0] += 500;
        }
    }
    inp
}

/// The chain off player p's item hit on foe s made by item `src`: resolved whole (integers only, seeded), then each
/// proc's effect applied in order, each from the foe its parent hit.
unsafe fn procs_off(p: usize, s: i32, src: i32) {
    if IN_PROC || p >= nh() as usize || procs::hit_kind(src) == 0 {
        return;
    }
    let inp = proc_in(p);
    if inp.copies.iter().all(|&c| c == 0) {
        return;
    }
    S.proc_n = S.proc_n.wrapping_add(1);
    let mut out = [Proc::default(); PROC_MAX];
    let n = procs::resolve(&inp, src, &mut out);
    run_chain(p, s, &out[..n]);
}

unsafe fn run_chain(p: usize, s: i32, chain: &[Proc]) {
    if chain.is_empty() {
        return;
    }
    IN_PROC = true;
    let mut tgt = [s; PROC_MAX];
    for (i, q) in chain.iter().enumerate() {
        let from = if q.parent == 255 { s } else { tgt[q.parent as usize] };
        tgt[i] = proc_fire(p, q, from);
        log_item(b"item procs", q.item, p as i32 * 10_000 + q.depth as i32 * 1000 + q.coef as i32);
        // frame-tagged (a resimulated frame logs the same line again: the checks compare the unique set)
        log_item(b"proc at", q.item, S.play * 100 + p as i32 * 10 + q.depth as i32);
    }
    let deep = chain.iter().map(|q| q.depth).max().unwrap_or(0);
    if deep >= 2 {
        log_item(b"proc chain", chain[0].item, p as i32 * 100 + chain.len() as i32 * 10 + deep as i32);
    }
    IN_PROC = false;
}

/// One proc's effect on foe `from`; returns the foe it hit (its children start there).
unsafe fn proc_fire(p: usize, q: &Proc, from: i32) -> i32 {
    let Some(f) = pos(from) else { return from };
    let c = P { x: f.x, y: f.y + 8.0 };
    match q.item {
        IT_EMBER_SEED => {
            // a fire seed blooms on the foe: a small burning hit (Inferno Seed: a launching bloom)
            if evo_on(p, procs::EVO_INFERNO_SEED) {
                BOOM_ID = IT_EMBER_SEED;
                boom(p, c, 9.0, (3 * mul(p, IT_EMBER_SEED) / 1000).max(1), 70, 45, 30, from);
                burst(VFX_EVO + procs::EVO_INFERNO_SEED as i32, p, c, 14.0, 0);
            } else {
                tap(p, from, c, (2 * mul(p, IT_EMBER_SEED) / 1000).max(1), IT_EMBER_SEED);
            }
            burn(from, BURN_STACKS, true, p as i32);
            if has(p, IT_FIRE_LACE) {
                syn_on(p, procs::SY_WILDFIRE);
            }
            burst(IT_EMBER_SEED, p, c, 9.0, 0);
            from
        }
        IT_STATIC_COIL => {
            // an arc jumps to the nearest other foe (the same one when it stands alone)
            let t = foe_near(p, c, 80.0, 1 << from);
            let t = if t >= 0 { t } else { from };
            let to = pos(t).map_or(c, |g| P { x: g.x, y: g.y + 8.0 });
            spawn(PJ_ARC, p, c, 0.0, 0.0, 12, x100(to.x), x100(to.y));
            tap(p, t, to, (2 * mul(p, IT_STATIC_COIL) / 1000).max(1), IT_STATIC_COIL);
            if has(p, IT_FROST_BUD) && syn_on(p, procs::SY_SUPERCONDUCTOR) {
                chill(t, 45);
                burst(VFX_SYN + procs::SY_SUPERCONDUCTOR as i32, p, to, 7.0, 0);
            }
            burst(IT_STATIC_COIL, p, to, 6.0, 0);
            t
        }
        IT_FROST_BUD => {
            // a bud of frost bursts on the foe and chills
            chill(from, 60);
            tap(p, from, c, (2 * mul(p, IT_FROST_BUD) / 1000).max(1), IT_FROST_BUD);
            burst(IT_FROST_BUD, p, c, 11.0, 0);
            if has(p, IT_STATIC_COIL) && syn_on(p, procs::SY_SUPERCONDUCTOR) {
                let t = foe_near(p, c, 80.0, 1 << from);
                if let Some(g) = t.ge(&0).then(|| pos(t)).flatten() {
                    let to = P { x: g.x, y: g.y + 8.0 };
                    spawn(PJ_ARC, p, c, 0.0, 0.0, 12, x100(to.x), x100(to.y));
                    tap(p, t, to, 1, IT_STATIC_COIL);
                }
            }
            from
        }
        _ => from,
    }
}

/// Synergy k held by player p (both items): true, logged; the first time this run, its banner.
unsafe fn syn_on(p: usize, k: usize) -> bool {
    let Some(sy) = SYNERGIES.get(k) else { return false };
    if p >= NP || !has(p, sy.a) || !has(p, sy.b) {
        return false;
    }
    if S.syn_seen[p] >> k & 1 == 0 {
        S.syn_seen[p] |= 1 << k;
        announce(p, BANNER_SYN, k as i32);
        log_syn(b"synergy first", sy.key, p as i32 * 100 + k as i32);
    }
    log_syn(b"synergy fires", sy.key, p as i32 * 100 + k as i32);
    true
}

/// Evolution j formed for player p (its base at the cap, its partner held).
unsafe fn evo_on(p: usize, j: usize) -> bool {
    let Some(e) = EVOLUTIONS.get(j) else { return false };
    p < NP && procs::evolves(e, copies(p, e.base), copies(p, e.partner))
}

/// After an item is gained: any evolution newly formed is announced (once a run).
unsafe fn evo_check(p: usize) {
    for (j, e) in EVOLUTIONS.iter().enumerate() {
        if evo_on(p, j) && S.evo_seen[p] >> j & 1 == 0 {
            S.evo_seen[p] |= 1 << j;
            announce(p, BANNER_EVO, j as i32);
            log_syn(b"evolution formed", e.key, p as i32 * 100 + j as i32);
            if let Some(me) = pos(p as i32) {
                burst(VFX_EVO + j as i32, p, P { x: me.x, y: me.y + 9.0 }, 22.0, 0);
            }
        }
    }
}

unsafe fn announce(p: usize, kind: i32, k: i32) {
    S.banner[p] = [S.play.max(1), kind << 8 | k];
    ev(EV_SYNERGY, p as i32, kind << 8 | k);
}

fn log_syn(what: &[u8], key: &str, v: i32) {
    let mut b = [0u8; 80];
    let mut n = 0;
    for &x in b"envoy: ".iter().chain(what.iter()).chain(b" ".iter()).chain(key.as_bytes().iter()) {
        if n < b.len() - 1 {
            b[n] = x;
            n += 1;
        }
    }
    unsafe { gd_log(b.as_ptr(), v) }
}

/// 51 Kin Tether (2P): a light line between the partners; a foe crossing it is nicked (once each 40 frames).
unsafe fn item_kin_tether(p: usize) {
    if nh() < 2 || !alive(0) || !alive(1) {
        return;
    }
    let (Some(a), Some(b)) = (pos(p as i32), pos(1 - p as i32)) else { return };
    let (a, b) = (P { x: a.x, y: a.y + 9.0 }, P { x: b.x, y: b.y + 9.0 });
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = (dx * dx + dy * dy).max(1.0);
    if l2 > 160.0 * 160.0 {
        return; // too far apart: the tether is slack
    }
    for s in 0..4 {
        if !enemy_of(p * 2, s) || S.play < S.tether_cd[p][s as usize] {
            continue;
        }
        let Some(f) = pos(s) else { continue };
        let fc = P { x: f.x, y: f.y + 8.0 };
        let t = (((fc.x - a.x) * dx + (fc.y - a.y) * dy) / l2).clamp(0.0, 1.0);
        let at = P { x: a.x + dx * t, y: a.y + dy * t };
        if d2(at, fc) < 7.0 * 7.0 {
            S.tether_cd[p][s as usize] = S.play + 40;
            tap(p, s, at, (mul(p, IT_KIN_TETHER) / 1000).max(1), IT_KIN_TETHER);
            burst(IT_KIN_TETHER, p, at, 6.0, 0);
            log_item(b"item fires", IT_KIN_TETHER, p as i32 * 100 + s);
        }
    }
}

/// 52 Rally Banner (2P): a KO rallies the partner for 5 s: their item hits +1% and their procs +50%.
unsafe fn item_rally_banner(p: usize) {
    if nh() < 2 {
        return;
    }
    let q = 1 - p;
    S.rally_until[q] = S.play + 300 * mul(p, IT_RALLY_BANNER) / 1000;
    if let Some(m) = pos(q as i32) {
        burst(IT_RALLY_BANNER, q, P { x: m.x, y: m.y + 22.0 }, 12.0, 0);
    }
    log_item(b"item fires", IT_RALLY_BANNER, p as i32 * 100 + q as i32);
}

/// 53 Kindred Spark (2P): player p's item hit on a foe near the partner jumps a spark from the partner, and the
/// partner's procs roll off it (at half coefficient). Once each 20 frames.
unsafe fn kindred(p: usize, s: i32, src: i32) {
    if IN_PROC || nh() < 2 || p >= NP || !has(p, IT_KINDRED_SPARK) || src == IT_KINDRED_SPARK || S.play < S.kin_cd[p] {
        return;
    }
    let q = 1 - p;
    let (Some(m), Some(f)) = (pos(q as i32), pos(s)) else { return };
    let (from, to) = (P { x: m.x, y: m.y + 9.0 }, P { x: f.x, y: f.y + 8.0 });
    if d2(from, to) > 60.0 * 60.0 || !alive(q as i32) {
        return;
    }
    S.kin_cd[p] = S.play + 20;
    spawn(PJ_ARC, q, from, 0.0, 0.0, 12, x100(to.x), x100(to.y));
    burst(IT_KINDRED_SPARK, q, to, 7.0, 0);
    log_item(b"item fires", IT_KINDRED_SPARK, p as i32 * 100 + s);
    IN_PROC = true;
    tap(q, s, to, 1, IT_KINDRED_SPARK);
    IN_PROC = false;
    let inp = proc_in(q);
    if inp.copies.iter().any(|&c| c > 0) {
        S.proc_n = S.proc_n.wrapping_add(1);
        let mut out = [Proc::default(); PROC_MAX];
        let coef = procs::proc_coef(IT_KINDRED_SPARK) * mul(p, IT_KINDRED_SPARK) as u32 / 1000;
        let n = procs::resolve_from(&inp, procs::hit_kind(src) | procs::HK_SPARK, coef, &mut out);
        run_chain(q, s, &out[..n]);
    }
}

// ---- the showcase (test knob `itemdemo`: screenshots of every effect)

static mut DEMO: i32 = -1;
unsafe fn demo_on() -> bool {
    if DEMO < 0 {
        DEMO = test_flag("itemdemo") as i32;
    }
    DEMO == 1
}

/// Test knob `itemdemo`: every DEMO_GAP frames player 1 is given the next item (its flash shows) and 40 frames later
/// its effect fires once from the fighter, whatever the trigger (Moon Drops just orbit; the two modifiers fire a
/// Spark Ring and a Hop Shot to show their look). Deterministic like any other sim input.
const DEMO_AT: i32 = 240;
const DEMO_GAP: i32 = 150;
unsafe fn demo_frame() {
    let t = S.play - DEMO_AT;
    if t < 0 || t >= DEMO_GAP * ORDER.len() as i32 + 400 {
        return;
    }
    let k = (t / DEMO_GAP) as usize;
    let at = t % DEMO_GAP;
    if k >= ORDER.len() {
        // after the twelve: everything once more, with both modifiers on
        if at == 40 || at == 120 {
            demo_fire(IT_SPARK_RING);
            demo_fire(IT_HOP_SHOT);
            demo_fire(IT_ROLL_MINES);
            demo_fire(IT_SHINE_BELL);
        }
        return;
    }
    let id = ORDER[k];
    if at == 0 {
        give(0, id, G_PICK);
        log_item(b"item demo", id, S.play);
    } else if at == 40 {
        match id {
            IT_HOMING_INK | IT_FIRE_LACE | IT_HALO | IT_ICE_GLASS | IT_CHAIN_SPARK | IT_TWIN_SHOT | IT_BOUNCY => {
                demo_fire(IT_SPARK_RING);
                demo_fire(IT_HOP_SHOT);
            }
            IT_GRAVITY_WELL | IT_BIG_BANG => {
                demo_fire(IT_ROLL_MINES);
                demo_fire(IT_SHINE_BELL);
            }
            _ => demo_fire(id),
        }
    }
}

unsafe fn demo_fire(id: i32) {
    lab_fire(0, id);
}

/// Item `id`'s effect for player `p` now, whatever its trigger (the showcase and the Item Lab's Fire button): from
/// the fighter, at the nearest foe where the effect needs one. Moon Drops just orbit; a modifier fires a Spark Ring
/// and a Hop Shot to show its look. An item with no arm here logs that the Lab cannot fire it yet.
pub(crate) unsafe fn lab_fire(p: usize, id: i32) {
    if p >= NP {
        return;
    }
    let h = gd_fighter_from_slot(p as i32);
    let Some(me) = pos(p as i32) else { return };
    let face = if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 };
    let front = P { x: me.x + face * 14.0, y: me.y + 8.0 };
    let v = foe_near(p, me, 400.0, 0);
    match id {
        IT_SPARK_RING => {
            let n = 6 * more(p, id).max(1);
            for k in 0..n {
                let a = k as f32 * TAU / n as f32;
                spawn(PJ_SPARK, p, front, cosf(a) * 1.7, sinf(a) * 1.7, 28, 0, 0);
            }
            burst(id, p, front, 10.0, 0);
        }
        IT_GLIDE_WAKE => item_glide_wake(p),
        IT_SHINE_BELL => item_shine_bell(p, me),
        IT_HOP_SHOT => item_hop_shot(p, me),
        IT_LEDGE_SPIKES => {
            BOOM_ID = id;
            let at = P { x: front.x, y: me.y };
            boom(p, P { x: at.x, y: at.y + 10.0 }, 11.0, 6, 90, 60, 45, -1);
            burst(id, p, at, 30.0, 0);
        }
        IT_WHIFF_ECHO => {
            S.it_atk_at[p] = [x100(me.x), x100(me.y), face as i32];
            item_whiff_echo(p);
        }
        IT_ROLL_MINES => item_roll_mines(p, P { x: me.x - face * 10.0, y: me.y }),
        IT_SMASH_QUAKE => {
            let vp = P { x: front.x, y: me.y };
            for dir in [-1, 1] {
                spawn(PJ_QUAKE, p, vp, 3.2 * dir as f32, 0.0, 30, dir, 700);
            }
            burst(id, p, vp, 14.0, 0);
        }
        IT_COIN_MAGNET => {
            if v >= 0 {
                item_coin_magnet(p, v);
            }
        }
        IT_LASER_PRISM => item_laser_prism(p, if v >= 0 { v } else { 3 }, front),
        IT_CROUCH_THORNS => item_crouch_thorns(p, me),
        IT_PERFECT_GUARD => item_perfect_guard(p, me),
        IT_BRIMSTONE_JAB => item_brimstone_jab(p),
        IT_DEMON_WINGS => {
            for k in 0..5 {
                let a = -1.57 + (k as f32 - 2.0) * 0.3;
                shot(PJ_FEATHER, p, P { x: me.x, y: me.y + 24.0 }, cosf(a) * 1.6, sinf(a) * 1.6, 50, 0);
            }
            burst(id, p, P { x: me.x, y: me.y + 10.0 }, 14.0, 0);
        }
        IT_CRIMSON_COUNT => item_crimson_count(p),
        IT_BLOODLUST => {
            if v >= 0 {
                item_bloodlust(p, v);
            }
        }
        IT_GUARDIAN_FEATHER => burst(id, p, P { x: me.x, y: me.y + 9.0 }, 18.0, 0),
        // the always-on orbitals: their look at the fighter (the orbs themselves circle whenever they are held)
        IT_MOON_DROPS | IT_DODGE_ORB | IT_NAIR_RING | IT_DASH_ORB | IT_SHIELD_ORBS => burst(id, p, me, 14.0, 0),
        IT_SPARE_HEART => burst(id, p, P { x: me.x, y: me.y + 12.0 }, 14.0, 0),
        // the proc items (synergy lane): one proc straight off the nearest foe, as if a hit of their kind had rolled it
        IT_EMBER_SEED | IT_STATIC_COIL | IT_FROST_BUD => {
            if v >= 0 {
                proc_fire(p, &Proc { item: id, depth: 1, coef: 1000, parent: 255 }, v);
            } else {
                burst(id, p, front, 10.0, 0);
            }
        }
        // the co-op items: their hit on the nearest foe (solo too, so the Lab can show them); Rally Banner rallies
        // the partner in co-op and shows its pennant burst solo
        IT_KIN_TETHER | IT_KINDRED_SPARK => {
            if let Some(f) = (v >= 0).then(|| pos(v)).flatten() {
                let at = P { x: f.x, y: f.y + 8.0 };
                tap(p, v, at, (mul(p, id) / 1000).max(1), id);
                burst(id, p, at, 7.0, 0);
            } else {
                burst(id, p, front, 7.0, 0);
            }
        }
        IT_RALLY_BANNER => {
            if nh() >= 2 {
                item_rally_banner(p);
            } else {
                burst(id, p, P { x: me.x, y: me.y + 22.0 }, 12.0, 0);
            }
        }
        // a modifier changes what the others make: a Spark Ring and a Hop Shot show its look
        IT_HOMING_INK | IT_FIRE_LACE | IT_HALO | IT_ICE_GLASS | IT_CHAIN_SPARK | IT_TWIN_SHOT | IT_GRAVITY_WELL | IT_BIG_BANG | IT_BOUNCY => {
            lab_fire(p, IT_SPARK_RING);
            lab_fire(p, IT_HOP_SHOT);
        }
        _ => {
            log_item(b"lab cannot fire yet", id, S.play);
            return;
        }
    }
    log_item(b"item demo fires", id, S.play);
}

// ---- the projectiles

unsafe fn proj_frame() {
    for i in 0..PROJ {
        let q = S.proj[i];
        if q[0] == 0 {
            continue;
        }
        let p = (q[1].clamp(0, 1)) as usize;
        let age = q[4] + 1;
        S.proj[i][4] = age;
        if age > q[5] {
            proj_end(i);
            continue;
        }
        let at = pj_pos(&q);
        match q[0] {
            PJ_SPARK | PJ_STAR | PJ_PRISM | PJ_FEATHER | PJ_SHARD => {
                let (mut vx, mut vy) = (S.pv[i][0] as f32 / 100.0, S.pv[i][1] as f32 / 100.0);
                if q[0] == PJ_STAR || q[0] == PJ_SHARD {
                    vy -= 0.03;
                }
                if q[0] == PJ_FEATHER {
                    vy = (vy - 0.02).max(-1.6);
                }
                if has(p, IT_HOMING_INK) && q[0] != PJ_SHARD {
                    let t = foe_near(p, at, 160.0, S.pj_hit[i]);
                    if let Some(f) = t.ge(&0).then(|| pos(t)).flatten() {
                        let (dx, dy) = (f.x - at.x, f.y + 8.0 - at.y);
                        let l = sqrtf(dx * dx + dy * dy).max(0.001);
                        let sp = sqrtf(vx * vx + vy * vy).max(1.2);
                        let mut k = 0.08 * mul(p, IT_HOMING_INK) as f32 / 1000.0;
                        if q[0] == PJ_STAR && has(p, IT_HOP_SHOT) {
                            k *= 1.6; // Seeker Stars: stars home harder
                        }
                        vx += (dx / l * sp - vx) * k;
                        vy += (dy / l * sp - vy) * k;
                    }
                }
                let mut nx = at.x + vx;
                let mut ny = at.y + vy;
                // Bouncy: off the main floor and its walls, three times a copy
                if has(p, IT_BOUNCY) && S.pj_bounce[i] < 3 * more(p, IT_BOUNCY) {
                    let edge = 56.0;
                    if ny < 0.0 && at.y >= 0.0 && nx.abs() < edge && vy < 0.0 {
                        vy = -vy * 0.85;
                        ny = 0.0;
                        S.pj_bounce[i] += 1;
                        S.proj[i][5] += 20;
                        burst(IT_BOUNCY, p, P { x: nx, y: 0.0 }, 4.0, 0);
                    } else if ny < 0.0 && at.x.abs() < edge && nx.abs() >= edge {
                        vx = -vx;
                        nx = at.x;
                        S.pj_bounce[i] += 1;
                        S.proj[i][5] += 20;
                        burst(IT_BOUNCY, p, P { x: nx, y: ny }, 4.0, 0);
                    }
                }
                S.pv[i] = [x100(vx), x100(vy)];
                S.proj[i][2] = x100(nx);
                S.proj[i][3] = x100(ny);
                let now = P { x: nx, y: ny };
                if matches!(q[0], PJ_PRISM | PJ_FEATHER | PJ_SHARD) {
                    let s = foe_near(p, now, 9.0, S.pj_hit[i]);
                    if s >= 0 {
                        let twin = S.pf[i] & PF_TWIN != 0;
                        let (id, pct) = match q[0] {
                            PJ_PRISM => (IT_LASER_PRISM, 3),
                            PJ_FEATHER => (IT_DEMON_WINGS, 3),
                            _ => (IT_ICE_GLASS, 2),
                        };
                        tap(p, s, now, if twin { (pct * 6 / 10).max(1) } else { pct }, id);
                        burst(id, p, now, 5.0, 0);
                        S.pj_hit[i] |= 1 << s;
                        if q[0] != PJ_SHARD {
                            shot_hit(p, i, now, s);
                        }
                        if !has(p, IT_HALO) {
                            proj_end(i);
                        }
                    }
                    continue;
                }
                let r = if q[0] == PJ_STAR { 7.0 } else { 6.0 };
                let s = foe_near(p, now, r + 4.0, S.pj_hit[i]);
                if s >= 0 {
                    let twin = S.pf[i] & PF_TWIN != 0;
                    if q[0] == PJ_STAR {
                        BOOM_ID = IT_HOP_SHOT;
                        boom(p, now, 7.0, if twin { 3 } else { 5 }, 50, 45, 35, s);
                        burst(IT_HOP_SHOT, p, now, 9.0, 0);
                        if !has(p, IT_HALO) {
                            S.proj[i][5] = 0; // spent (Halo: holy shots pierce on)
                        }
                    } else {
                        tap(p, s, now, if twin { 1 } else { 2 }, IT_SPARK_RING);
                        burst(IT_SPARK_RING, p, now, 4.0, 0);
                    }
                    S.pj_hit[i] |= 1 << s;
                    shot_hit(p, i, now, s);
                }
            }
            PJ_WAKE => {
                // every 30 frames the streak may bite each foe standing in it again
                if age % 30 == 0 {
                    S.pj_hit[i] = 0;
                }
                let half = q[6] as f32 / 100.0;
                for s in 0..4 {
                    if S.pj_hit[i] >> s & 1 != 0 || !enemy_of(p * 2, s) {
                        continue;
                    }
                    if let Some(f) = pos(s) {
                        if (f.x - at.x).abs() < half && (f.y - at.y).abs() < 9.0 {
                            S.pj_hit[i] |= 1 << s;
                            BOOM_ID = IT_GLIDE_WAKE;
                            boom(p, P { x: f.x, y: f.y + 4.0 }, 6.0, 2, 80, 25, 30, s);
                            burn(s, BURN_STACKS, true, p as i32);
                        }
                    }
                }
            }
            PJ_GHOST => {
                if age == q[7] {
                    let face = if q[6] < 0 { -1.0 } else { 1.0 };
                    let c = P { x: at.x + face * 9.0, y: at.y + 9.0 };
                    BOOM_ID = IT_WHIFF_ECHO;
                    boom(p, c, 12.0, 5, 361, 55, 35, foe_near(p, c, 14.0, 0));
                    burst(IT_WHIFF_ECHO, p, c, 12.0, q[6]);
                    S.proj[i][7] = 0; // swung (age never meets 0 again)
                }
            }
            PJ_MINE => {
                let s = if age > 12 { foe_near(p, P { x: at.x, y: at.y + 6.0 }, 10.0, 0) } else { -1 };
                if s >= 0 || age == q[5] {
                    mine_pop(p, i, at);
                }
            }
            PJ_QUAKE => {
                let dir = q[6] as f32;
                let nx = at.x + 3.2 * dir;
                S.proj[i][2] = x100(nx);
                let size = q[7] as f32 / 100.0;
                for s in 0..4 {
                    if S.pj_hit[i] >> s & 1 != 0 || !enemy_of(p * 2, s) {
                        continue;
                    }
                    if let Some(f) = pos(s) {
                        if (f.x - nx).abs() < size && f.y - at.y < 14.0 && f.y - at.y > -6.0 {
                            S.pj_hit[i] |= 1 << s;
                            BOOM_ID = IT_SMASH_QUAKE;
                            boom(p, P { x: f.x, y: f.y + 4.0 }, size, 5, 80, 55, 40, s);
                        }
                    }
                }
            }
            PJ_COIN => {
                let to = pos(q[6].clamp(0, 1)).map_or(at, |m| P { x: m.x, y: m.y + 10.0 });
                let left = (q[5] - age).max(1) as f32;
                let (mut vx, mut vy) = (S.pv[i][0] as f32 / 100.0, S.pv[i][1] as f32 / 100.0);
                // a spray, then pulled in harder every frame (it always lands on the player at the end)
                let pull = (1.0 - left / q[5] as f32).clamp(0.0, 1.0);
                vx = vx * (1.0 - pull) + (to.x - at.x) / left * pull * 1.6;
                vy = vy * (1.0 - pull) + (to.y - at.y) / left * pull * 1.6;
                S.pv[i] = [x100(vx), x100(vy)];
                S.proj[i][2] = x100(at.x + vx);
                S.proj[i][3] = x100(at.y + vy);
                if d2(P { x: at.x + vx, y: at.y + vy }, to) < 9.0 {
                    proj_end(i);
                }
            }
            PJ_BEAM => {
                // a beam: hits each foe in its band once while it lasts
                let len = q[6] as f32 / 100.0;
                let (x0, x1) = if len >= 0.0 { (at.x, at.x + len) } else { (at.x + len, at.x) };
                let (id, pct, ang) = match q[7] {
                    BEAM_GOLD => (IT_PERFECT_GUARD, 6 * mul(p, IT_PERFECT_GUARD) / 1000, 45),
                    _ => (IT_BRIMSTONE_JAB, 5, 30),
                };
                for s in 0..4 {
                    if S.pj_hit[i] >> s & 1 != 0 || !enemy_of(p * 2, s) {
                        continue;
                    }
                    if let Some(f) = pos(s) {
                        if f.x > x0 && f.x < x1 && (f.y + 8.0 - at.y).abs() < 10.0 {
                            S.pj_hit[i] |= 1 << s;
                            BOOM_ID = id;
                            let a = if len >= 0.0 { ang } else { 180 - ang };
                            boom(p, P { x: f.x, y: f.y + 8.0 }, 8.0, pct.max(1), a, 60, 45, s);
                        }
                    }
                }
            }
            PJ_TENDRIL => {
                if age == 6 {
                    let t = q[6];
                    if let Some(f) = pos(t) {
                        BOOM_ID = IT_BLOODLUST;
                        boom(p, P { x: f.x, y: f.y + 8.0 }, 9.0, (3 * mul(p, IT_BLOODLUST) / 1000).max(1), 361, 55, 35, t);
                        burst(IT_BLOODLUST, p, P { x: f.x, y: f.y + 8.0 }, 10.0, 0);
                    }
                }
            }
            PJ_THORNS => {
                if age == 3 {
                    let size = q[7] as f32 / 100.0;
                    let me = pos(p as i32).unwrap_or(at);
                    let c = P { x: me.x, y: me.y + 9.0 };
                    BOOM_ID = IT_CROUCH_THORNS;
                    boom(p, c, size, (4 * mul(p, IT_CROUCH_THORNS) / 1000).max(1), 361, 60, 40, foe_near(p, c, size + 6.0, 0));
                }
            }
            PJ_WELL => {
                // Gravity Well: the burst pulls foes in for its life (a weak push toward the centre every 4 frames),
                // then bursts with the burst's own numbers
                let r = q[6] as f32 / 100.0;
                if age % 4 == 1 && age < q[5] {
                    for s in 0..4 {
                        if !enemy_of(p * 2, s) {
                            continue;
                        }
                        if let Some(f) = pos(s) {
                            let (dx, dy) = (at.x - f.x, at.y - f.y - 8.0);
                            if dx * dx + dy * dy < (r * 2.4) * (r * 2.4) && dx * dx + dy * dy > 16.0 {
                                let ang = angle_deg(dx, dy);
                                gd_envoy_blast(p as i32, f.x, f.y + 8.0, 4.0, 0, ang, 0, 18);
                            }
                        }
                    }
                }
                if age == q[5] {
                    let w = q[7];
                    IN_WELL = true;
                    BOOM_ID = IT_GRAVITY_WELL;
                    boom(p, at, r, w & 0xFF, S.pv[i][0], w >> 8 & 0xFF, w >> 16 & 0xFF, S.pv[i][1]);
                    IN_WELL = false;
                    burst(IT_GRAVITY_WELL, p, at, r, 0);
                }
            }
            PJ_ARC => {}
            _ => proj_end(i),
        }
    }
}

/// A shot struck foe `s`: Ice Glass shatters it into three shards (shards never shatter again).
unsafe fn shot_hit(p: usize, i: usize, at: P, s: i32) {
    if has(p, IT_ICE_GLASS) {
        chill(s, 60 * copies(p, IT_ICE_GLASS).min(3));
        for k in 0..3 {
            let a = 0.9 + k as f32 * 0.65;
            if let Some(j) = spawn(PJ_SHARD, p, at, cosf(a) * 1.8, sinf(a) * 1.8, 26, 0, 0) {
                S.pj_hit[j] = 1 << s;
            }
        }
        log_item(b"item fires", IT_ICE_GLASS, p as i32 * 100 + s);
    }
    let _ = i;
}

/// Degrees (0 right, 90 up) of (dx, dy), for a knockback angle (no std atan2: an octant polynomial).
fn angle_deg(dx: f32, dy: f32) -> i32 {
    let (ax, ay) = (dx.abs(), dy.abs());
    if ax < 1e-4 && ay < 1e-4 {
        return 0;
    }
    let (t, swap) = if ay > ax { (ax / ay, true) } else { (ay / ax, false) };
    let mut a = t * (45.0 - (t - 1.0) * (14.0 + 3.8 * t)); // atan(t) in degrees, 0 <= t <= 1
    if swap {
        a = 90.0 - a;
    }
    if dx < 0.0 {
        a = 180.0 - a;
    }
    if dy < 0.0 {
        a = 360.0 - a;
    }
    (a as i32).rem_euclid(360)
}

unsafe fn mine_pop(p: usize, i: usize, at: P) {
    let c = P { x: at.x, y: at.y + 5.0 };
    BOOM_ID = IT_ROLL_MINES;
    boom(p, c, 13.0, 6, 361, 60, 45, foe_near(p, c, 16.0, 0));
    burst(IT_ROLL_MINES, p, c, 14.0, 0);
    S.proj[i] = [0; PJ_W];
}

unsafe fn proj_end(i: usize) {
    S.proj[i] = [0; PJ_W];
    S.pv[i] = [0; 2];
    S.pj_hit[i] = 0;
    S.pf[i] = 0;
}

/// A new match: no projectile carries over.
pub(crate) unsafe fn reset_match() {
    S.proc_at = [0; NP];
    S.ped_near = [0; NP];
    S.pick_prev = [-1; NP];
    flourish();
    S.proj = [[0; PJ_W]; PROJ];
    S.pv = [[0; 2]; PROJ];
    S.pj_hit = [0; PROJ];
    S.pf = [0; PROJ];
    S.it_nair = [0; NP];
    S.it_smash = [0; NP];
    S.it_atk = [0; NP];
    S.it_atk_hit = [0; NP];
    S.it_hop = [0; NP];
    S.it_hop_t = [0; NP];
    S.it_prev = [-1; NP];
    S.moon_cd = [[0; 4]; NP];
    S.pj_bounce = [0; PROJ];
    orbs::reset_match();
    S.sigils = [0; NP];
    S.wings = [0; NP];
    S.guard_used = [0; NP];
    S.it_cd = [[0; 8]; NP];
    S.rally_until = [0; NP];
    S.tether_cd = [[0; 4]; NP];
    S.kin_cd = [0; NP];
    S.ped_cost = [[0; DROPS]; NP];
    S.door = 0;
    S.door_at = 0;
    S.fork = if S.fork == 1 { 0 } else { S.fork };
    S.skip_shop = 0;
    S.ped_kind = 0;
    S.ped_left = 0;
    S.ped_done = 0;
    S.camp_at = 0;
    S.camp_gain = 0;
}

// ---- pedestals: the item room and the boss drop (the drop code's floor points, owner colour and fly-in)

/// The pedestals this won fight puts out: the item room after a session's last ordinary fight, the boss's choice
/// of two after the boss; none after the others.
pub(crate) unsafe fn ped_due() -> i32 {
    let f = S.fight.clamp(0, FIGHTS as i32 - 1);
    let _ = f;
    if sched().kind == FK_BOSS { PED_BOSS } else { 0 }
}

pub(crate) unsafe fn ped_open(kind: i32) {
    S.ped_kind = kind;
    S.ped_left = 0; // frames open (no countdown)
    let coop = nh() == 2;
    for p in 0..nh() as usize {
        let base = if coop { if p == 0 { -32.0 } else { 32.0 } } else { 0.0 };
        S.drop[p] = [[0; 4]; DROPS];
        S.ped_cost[p] = [0; DROPS];
        if kind == PED_ITEM_ROOM || kind == PED_ANGEL {
            let id = if kind == PED_ANGEL { roll(p, PL_ANGEL, 0xA6, &[]) } else { roll(p, PL_ITEM, 0x11, &[]) };
            let id = if id == 0 && kind == PED_ANGEL { roll(p, PL_ITEM | PL_BOSS, 0xA7, &[]) } else { id };
            if id > 0 {
                S.drop[p][0] = [1, id, x100(base), 0];
            }
        } else if kind == PED_DEVIL || kind == PED_SECRET {
            // the devil: one or two deals; the secret room: one item or a choice of two (seeded)
            let pool = if kind == PED_DEVIL { PL_DEVIL } else { PL_SECRET };
            let two = mix(S.seed as u32 ^ 0xDE5_0000 ^ fkey() << 4 ^ (kind as u32) << 20 ^ p as u32) & 1 == 1 || test_flag("two");
            let a = roll(p, pool, 0x31 + kind as u32, &[]);
            let b = if two { roll(p, pool, 0x41 + kind as u32, &[a]) } else { 0 };
            let gap = if coop { 14.0 } else { 22.0 };
            for (k, id) in [a, b].into_iter().enumerate() {
                if id <= 0 {
                    continue;
                }
                let x = if b > 0 { base + if k == 0 { -gap } else { gap } } else { base };
                S.drop[p][k] = [1, id, x100(x), 0];
                if kind == PED_DEVIL {
                    let c = item(id).map_or(1, |it| it.slots.max(1) as i32);
                    S.ped_cost[p][k] = c | if rooms::can_pay(p, c) { 0 } else { COST_LOCKED };
                }
            }
        } else {
            // the boss's choice of two: a new item, and a copy of one already held when there is one
            let a = roll(p, PL_ITEM | PL_BOSS, 0x22, &[]);
            let mut b = 0;
            let held = ORDER;
            let nn = held.len();
            let start = (mix(S.seed as u32 ^ 0xB055_0000 ^ fkey() << 4 ^ p as u32) % nn as u32) as usize;
            for k in 0..nn {
                let id = held[(start + k) % nn];
                if has(p, id) && id != a {
                    b = id;
                    break;
                }
            }
            if b == 0 {
                b = roll(p, PL_ITEM | PL_SHOP, 0x23, &[a]);
            }
            let gap = if coop { 14.0 } else { 22.0 };
            if a > 0 {
                S.drop[p][0] = [1, a, x100(base - gap), 0];
            }
            if b > 0 {
                S.drop[p][1] = [1, b, x100(base + gap), 0];
            }
        }
        for k in 0..DROPS {
            if S.drop[p][k][0] == 1 {
                log_item(
                    match kind {
                        PED_BOSS => b"boss drop on a pedestal",
                        PED_DEVIL => b"devil deal on a pedestal",
                        PED_ANGEL => b"angel room pedestal",
                        PED_SECRET => b"secret room pedestal",
                        _ => b"item room pedestal",
                    },
                    S.drop[p][k][1],
                    p as i32,
                );
            }
        }
    }
    log(
        match kind {
            PED_BOSS => b"envoy: boss drop, choose one\0",
            PED_DEVIL => b"envoy: devil deal open\0",
            PED_ANGEL => b"envoy: angel room open\0",
            PED_SECRET => b"envoy: secret room pedestals\0",
            _ => b"envoy: item room open\0",
        },
        S.fight + 1,
    );
    flourish();
    if kind == PED_DEVIL {
        // test knob `devilbuy`: player 1 takes the first deal it can pay at once (the scripted co-op check)
        if test_flag("devilbuy") {
            for k in 0..DROPS {
                if S.drop[0][k][0] == 1 && S.ped_cost[0][k] & COST_LOCKED == 0 {
                    ped_take(0, k, G_PICK);
                    break;
                }
            }
        }
    }
}

/// Frames a worn look (sigils, feather, halo, wings) stays out after a proc.
const WORN_SHOW: i32 = 45;

/// A pedestal walked into (or flown in): the item is the player's; the boss's other choice is gone.
pub(crate) unsafe fn ped_take(p: usize, k: usize, how: i32) {
    let id = S.drop[p][k][1];
    if S.ped_kind == PED_DEVIL {
        // a deal: paid in stock slots (never the last); a locked one stays on its pedestal
        let c = S.ped_cost[p][k] & 0xFF;
        if S.ped_cost[p][k] & COST_LOCKED != 0 || !rooms::pay_slots(p, c) {
            return;
        }
        S.deals[p] += 1;
        S.deals_any += 1;
        ev(EV_DEAL, p as i32, id | c << 8);
        log_item(b"devil deal taken", id, p as i32 * 100 + c);
        // the other deal may now be too dear
        for j in 0..DROPS {
            if j != k && S.drop[p][j][0] == 1 {
                let cj = S.ped_cost[p][j] & 0xFF;
                S.ped_cost[p][j] = cj | if rooms::can_pay(p, cj) { 0 } else { COST_LOCKED };
            }
        }
        S.ped_cost[p][k] = 0;
    }
    S.gain_from[p] = [S.drop[p][k][2], S.drop[p][k][3], p as i32];
    S.drop[p][k][0] = 2;
    give(p, id, how);
    if S.ped_kind == PED_BOSS || S.ped_kind == PED_SECRET {
        for j in 0..DROPS {
            if j != k && S.drop[p][j][0] == 1 {
                S.drop[p][j][0] = 0;
                log_item(b"choice left behind", S.drop[p][j][1], p as i32);
            }
        }
    }
}

/// The pedestals' wait: true while the match should stay held for them (owners walk in; the timer runs). At the end
/// of the timer only the first choice is left out, so the fly-in brings exactly one item per player.
pub(crate) unsafe fn ped_wait() -> bool {
    if S.ped_kind == 0 {
        return false;
    }
    if !drops_on_floor() {
        S.ped_kind = 0;
        return false;
    }
    // no timer (the owner: "we shouldn't be rushed on making our build"): the room waits until every player took
    // their item; the devil room is optional and ends when a player presses D-pad down at the door. A pedestal whose
    // owner is not on the stage (out of stocks) flies in at once. Test knob `autotake` (CPU-driven checks only)
    // keeps the old wait.
    S.ped_left += 1;
    let auto = test_flag("autotake") && S.ped_left > PED_TIME;
    let mut waiting = false;
    for p in 0..nh() as usize {
        if !S.drop[p].iter().any(|d| d[0] == 1) {
            continue;
        }
        if pos(p as i32).is_none() && S.ped_kind != PED_DEVIL {
            continue; // the fly-in takes it
        }
        if S.ped_kind == PED_DEVIL && (gd_pad_pressed(p as i32) & DPAD_DOWN != 0 || S.ped_cost[p].iter().zip(S.drop[p].iter()).all(|(c, d)| d[0] != 1 || c & COST_LOCKED != 0)) {
            for k in 0..DROPS {
                if S.drop[p][k][0] == 1 {
                    S.drop[p][k][0] = 0;
                    log_item(b"devil deal left", S.drop[p][k][1], p as i32);
                }
            }
            log(b"envoy: devil room left, player\0", p as i32);
            continue;
        }
        waiting = true;
    }
    if waiting && !auto {
        if S.ped_left % 600 == 0 {
            log(b"envoy: pedestal wait (no timer), frames open\0", S.ped_left);
        }
        return true;
    }
    if S.ped_kind == PED_BOSS || S.ped_kind == PED_SECRET {
        for p in 0..NP {
            if S.drop[p][0][0] == 1 && S.drop[p][1][0] == 1 {
                S.drop[p][1][0] = 0;
            }
        }
    }
    if S.ped_kind == PED_DEVIL {
        // deals never fly in: what was not bought is gone with the door
        for p in 0..NP {
            for k in 0..DROPS {
                if S.drop[p][k][0] == 1 {
                    S.drop[p][k][0] = 0;
                    log_item(b"devil deal left", S.drop[p][k][1], p as i32);
                }
            }
            S.ped_cost[p] = [0; DROPS];
        }
    }
    S.ped_kind = 0;
    false
}

// ---- the record (v9) and the view

/// The item words of the record: player p's copies, a byte per id, in R_DRIVES + p * 10 .. + 10.
pub(crate) unsafe fn rec_put(r: &mut [i32; REC_WORDS]) {
    for w in R_DRIVES..R_DRIVES + 20 {
        r[w] = 0;
    }
    for k in 0..6 {
        r[R_KEYS + k] = 0;
    }
    for p in 0..NP {
        // ids 48-55: two bits a copy in the spare high halves of R_ACTIVE (player 1) and R_PIPS (player 2)
        let mut hi = 0;
        for id in INV_STRIDE..ITEM_IDS {
            hi |= S.inv[p][id].clamp(0, 3) << (2 * (id - INV_STRIDE));
        }
        let w = if p == 0 { R_ACTIVE } else { R_PIPS };
        r[w] = r[w] & 0xFFFF | hi << 16;
        for id in 1..INV_STRIDE {
            let c = S.inv[p][id].clamp(0, 255);
            // ids 0-39 in R_DRIVES (10 words a player), 40-47 in R_KEYS[1..5] (2 words a player)
            let w = if id < 40 { R_DRIVES + p * 10 + id / 4 } else { R_KEYS + 1 + p * 2 + (id - 40) / 4 };
            r[w] |= c << (8 * (id % 4));
        }
    }
    // slice 2: each player's stock limit (P1 | P2 << 8), the floor word
    r[R_KEYS] = S.lims[0] | S.lims[1] << 8 | (S.deals[0] & 0xF) << 16 | (S.deals[1] & 0xF) << 20;
    r[R_KEYS + 5] = rooms::rec_word();
}

pub(crate) unsafe fn rec_get(r: &[i32; REC_WORDS]) {
    S.inv = [[0; ITEM_IDS]; NP];
    for p in 0..NP {
        let hi = (r[if p == 0 { R_ACTIVE } else { R_PIPS }] as u32 >> 16) as i32;
        for id in INV_STRIDE..ITEM_IDS {
            if item(id as i32).is_some() {
                S.inv[p][id] = hi >> (2 * (id - INV_STRIDE)) & 3;
            }
        }
        for id in 1..INV_STRIDE {
            let w = if id < 40 { R_DRIVES + p * 10 + id / 4 } else { R_KEYS + 1 + p * 2 + (id - 40) / 4 };
            let c = (r[w] as u32 >> (8 * (id % 4)) & 0xFF) as i32;
            if item(id as i32).is_some() {
                S.inv[p][id] = c;
            }
        }
    }
    // a slice-1 record has one limit (P2's byte 0): both players take it
    let l0 = (r[R_KEYS] & 0xFF).clamp(1, LIM_CAP);
    let l1 = r[R_KEYS] >> 8 & 0xFF;
    S.lims = [l0, if l1 > 0 { l1.clamp(1, LIM_CAP) } else { l0 }];
    S.deals = [r[R_KEYS] >> 16 & 0xF, r[R_KEYS] >> 20 & 0xF];
    rooms::rec_word_get(r[R_KEYS + 5]);
}

pub(crate) unsafe fn publish(v: &mut [i32; VIEW_WORDS]) {
    v[V_POOL_LIM] = S.lims[0];
    v[V_PED_KIND] = S.ped_kind;
    v[V_PED_LEFT] = S.ped_left;
    v[V_CAMP_AT] = S.camp_at;
    v[V_CAMP_GAIN] = S.camp_gain;
    v[V_ITEMS_ON] = 1;
    v[V_STOCK_PRICE] = stock_price();
    let mut n = 0;
    for i in 0..PROJ {
        let q = S.proj[i];
        if q[0] == 0 {
            continue;
        }
        let b = V_PROJ + n * PJ_W;
        v[b..b + PJ_W].copy_from_slice(&q);
        // the owner word carries the look flags; a moving shot shows its velocity (x100) in a, b for its trail
        v[b + 1] = q[1] | S.pf[i];
        if matches!(q[0], PJ_SPARK | PJ_STAR | PJ_COIN | PJ_PRISM | PJ_FEATHER | PJ_SHARD) {
            v[b + 6] = S.pv[i][0];
            v[b + 7] = S.pv[i][1];
        }
        n += 1;
    }
    // the orbitals (orbs.rs: Moon Drops and the stance orbs), after the pool's own entries
    orbs::publish(v, &mut n, |p| look(p));
    for p in 0..0usize {
        if !has(p, IT_MOON_DROPS) || !alive(p as i32) {
            continue;
        }
        let Some(me) = pos(p as i32) else { continue };
        let m = moons(p);
        for k in 0..m {
            if n >= PROJ {
                break;
            }
            let at = moon_at(me, S.play, k, m);
            let b = V_PROJ + n * PJ_W;
            v[b..b + PJ_W].copy_from_slice(&[PJ_MOON, p as i32 | look(p), x100(at.x), x100(at.y), S.play, 0, k, 0]);
            n += 1;
        }
    }
    // Kin Tether: the line between the partners
    if nh() == 2 && alive(0) && alive(1) {
        for p in 0..NP {
            if !has(p, IT_KIN_TETHER) || n >= PROJ {
                continue;
            }
            let (Some(a), Some(b)) = (pos(p as i32), pos(1 - p as i32)) else { continue };
            let base = V_PROJ + n * PJ_W;
            v[base..base + PJ_W].copy_from_slice(&[PJ_TETHER, p as i32 | look(p), x100(a.x), x100(a.y + 9.0), S.play, 0, x100(b.x), x100(b.y + 9.0)]);
            n += 1;
            break; // one line, whoever holds it
        }
    }
    // slice 2's worn looks: Crimson Count's sigils, the Guardian Feather, the Halo, the Demon Wings
    for p in 0..nh() as usize {
        if !alive(p as i32) {
            continue;
        }
        if proc_age(p) > WORN_SHOW && S.play - S.flourish_at[p] > FLOURISH_LEN {
            continue;
        }
        let Some(me) = pos(p as i32) else { continue };
        let face = { let h = gd_fighter_from_slot(p as i32); if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1 } else { 1 } };
        let mut put = |w: [i32; PJ_W]| {
            if n < PROJ {
                let b = V_PROJ + n * PJ_W;
                v[b..b + PJ_W].copy_from_slice(&w);
                n += 1;
            }
        };
        for k in 0..S.sigils[p].min(9) {
            let a = S.play as f32 * -0.05 + k as f32 * TAU / S.sigils[p].max(1) as f32;
            put([PJ_SIGIL, p as i32 | look(p), x100(me.x + cosf(a) * 11.0), x100(me.y + 9.0 + sinf(a) * 9.0), S.play, 0, k, S.sigils[p]]);
        }
        if has(p, IT_GUARDIAN_FEATHER) {
            let left = more(p, IT_GUARDIAN_FEATHER) - S.guard_used[p];
            for k in 0..more(p, IT_GUARDIAN_FEATHER) {
                let a = S.play as f32 * 0.04 + k as f32 * TAU / 3.0 + 1.0;
                put([PJ_GUARD, p as i32 | look(p), x100(me.x + cosf(a) * 16.0), x100(me.y + 12.0 + sinf(a) * 6.0), S.play, 0, k, (k >= left) as i32]);
            }
        }
        if has(p, IT_HALO) {
            put([PJ_HALO, p as i32 | look(p), x100(me.x), x100(me.y + 21.0), S.play, 0, mul(p, IT_HALO) / 10, 0]);
        }
        if has(p, IT_DEMON_WINGS) {
            put([PJ_WINGS, p as i32 | look(p), x100(me.x), x100(me.y + 12.0), S.play, 0, face, (S.wings[p] >= 2) as i32]);
        }
    }
    for p in 0..NP {
        for id in 0..ITEM_IDS {
            v[inv_word(p, id)] = S.inv[p][id];
        }
        v[V_BANNER + p] = S.banner[p][1];
        v[V_BANNER_AT + p] = S.banner[p][0];
        let mut bits = 0;
        for (k, sy) in SYNERGIES.iter().enumerate() {
            if has(p, sy.a) && has(p, sy.b) {
                bits |= 1 << k;
            }
        }
        for (j, e) in EVOLUTIONS.iter().enumerate() {
            if procs::evolves(e, copies(p, e.base), copies(p, e.partner)) {
                bits |= 1 << (16 + j);
            }
        }
        v[V_SYN + p] = bits;
        v[V_RALLY + p] = (S.rally_until[p] - S.play).max(0);
        v[V_PED_NEAR + p] = S.ped_near[p];
        v[V_FLOURISH + p] = if S.flourish_at[p] > 0 { (FLOURISH_LEN - (S.play - S.flourish_at[p])).max(0) } else { 0 };
        v[V_AURA + p] = rarest(p);
        v[V_PROC_AGE + p] = proc_age(p).min(255);
    }
    for e in 0..BURSTS {
        v[V_BURST + e * BU_W..V_BURST + (e + 1) * BU_W].copy_from_slice(&S.burst[e]);
    }
    v[V_BURST_N] = S.burst_n;
}

/// MELEE_ENVOY_TEST_DRIVES's item entries: `key[:copies]` gives copies of an item from the run's start.
pub(crate) unsafe fn test_item(p: usize, key: &str, n: i32) -> bool {
    let Some(it) = item_by_key(key) else { return false };
    for _ in 0..n.clamp(1, 9) {
        give(p, it.id, G_PICK);
    }
    true
}
