//! Envoy items, the first slice (docs/rust-port/notes/envoy-items-design.md 8): twelve hand-made items, each tied to
//! one thing you do in Melee, each with its own visible effect, firing straight off its trigger, stacking without a
//! slot limit. Everything here is sim state inside `St` (hashed every frame, rolled back with the game): the
//! inventory (copies per item id), a fixed projectile pool, the one-shot burst ring the UI draws, the pedestals and
//! the Isaac-style stock limit. Hitboxes go through `gd_envoy_blast` (a real launching hit on the engine path) or the
//! percent hook (a nick); one effect per item per frame, player 1 before player 2.
use super::*;

/// The stock limit (heart containers): where a run starts, the hard cap.
pub(crate) const LIM_START: [i32; 2] = [4, 5];
pub(crate) const LIM_CAP: i32 = 8;
/// Pedestals wait this long for their owner to walk in, then fly to them.
pub(crate) const PED_TIME: i32 = 900;
/// The shop's stock refill price (before the per-loop rise past loop 5).
pub(crate) const STOCK_REFILL: i32 = 60;

const ORDER: [i32; 12] = [
    IT_SPARK_RING, IT_GLIDE_WAKE, IT_SHINE_BELL, IT_HOP_SHOT, IT_LEDGE_SPIKES, IT_WHIFF_ECHO, IT_ROLL_MINES, IT_SMASH_QUAKE,
    IT_MOON_DROPS, IT_COIN_MAGNET, IT_HOMING_INK, IT_FIRE_LACE,
];

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
    (if has(p, IT_HOMING_INK) { PF_INK } else { 0 }) | (if has(p, IT_FIRE_LACE) { PF_LACE } else { 0 })
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
    if id == IT_SHINE_BELL && ![CK_FOX, CK_FALCO].iter().any(|k| S.pair[p].contains(k)) {
        return false;
    }
    true
}

/// A seeded pick from `pools` for player `p` (`salt` separates rolls); 0 when nothing is left.
pub(crate) unsafe fn roll(p: usize, pools: u8, salt: u32, skip: &[i32]) -> i32 {
    let mut h = mix(S.seed as u32 ^ 0x17E5_0000 ^ fkey() << 4 ^ salt << 12 ^ (p as u32) << 24);
    let start = (h % ORDER.len() as u32) as usize;
    h = mix(h.wrapping_add(0x6D2B_79F5));
    let step = 1 + (h % 5) as usize; // 1-5, coprime with 12 only for 1 and 5; walk all with a fallback pass
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
    Some(i)
}

/// A one-shot visual for the UI (a ring, a spike column, a swing, a pop): presentation only.
unsafe fn burst(id: i32, p: usize, at: P, size: f32, dir: i32) {
    let i = S.burst_n as usize % BURSTS;
    S.burst[i] = [S.play, id, p as i32, x100(at.x), x100(at.y), x100(size), look(p) | (dir + 1) & 3];
    S.burst_n += 1;
}

fn pj_pos(q: &[i32; PJ_W]) -> P {
    P { x: q[2] as f32 / 100.0, y: q[3] as f32 / 100.0 }
}

/// Damage on a foe without a launch (a nick): through the foe's toughness, into the cash-out tally; Fire Lace burns.
unsafe fn nick(p: usize, s: i32, milli: i32) {
    if !enemy_of(p * 2, s) || milli <= 0 {
        return;
    }
    let m = if s >= nh() { (milli as i64 * foe_def(s) as i64 / 1000) as i32 } else { milli };
    gd_envoy_percent(s, 0, m);
    deal(p, s, m, 1000, T_FIRE, -1, CM_NONE, 0, 0);
    lace(p, s);
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
    let rc = gd_envoy_blast(p as i32, at.x, at.y, r, pct, angle, kbg, bkb);
    if rc == 0 {
        S.blasts += 1;
        if has(p, IT_FIRE_LACE) {
            for s in 0..4 {
                if enemy_of(p * 2, s) && pos(s).is_some_and(|q| d2(P { x: q.x, y: q.y + 8.0 }, at) < (r + 6.0) * (r + 6.0)) {
                    lace(p, s);
                }
            }
        }
    } else if target >= 0 {
        nick(p, target, pct * 1000);
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
}

/// A foe was KO'd (lost a stock): Coin Magnet pays every player who holds it.
pub(crate) unsafe fn on_ko(s: i32) {
    for p in 0..nh() as usize {
        super::lab::trig(p, TR_KO as i32);
        if has(p, IT_COIN_MAGNET) {
            item_coin_magnet(p, s);
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
    for p in 0..nh() as usize {
        if has(p, IT_MOON_DROPS) {
            item_moon_drops(p);
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
        if let Some(i) = spawn(PJ_SPARK, p, at, cosf(a) * 1.7, sinf(a) * 1.7, 28, 0, 0) {
            S.pj_hit[i] = 1 << v; // the nair's own victim is not nicked again
        }
    }
    burst(IT_SPARK_RING, p, at, 10.0, 0);
    log_item(b"item fires", IT_SPARK_RING, p as i32 * 100 + n);
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
    let r = 15.0 * mul(p, IT_SHINE_BELL) as f32 / 1000.0;
    let c = P { x: me.x, y: me.y + 7.0 };
    boom(p, c, r, 3, 361, 70, 50, foe_near(p, c, r, 0));
    burst(IT_SHINE_BELL, p, c, r, 0);
    log_item(b"item fires", IT_SHINE_BELL, p as i32 * 100 + r as i32);
}

/// 4 Hop Shot: at the top of a short hop a star arcs toward the nearest foe (one more per copy, fanned).
unsafe fn item_hop_shot(p: usize, me: P) {
    let n = more(p, IT_HOP_SHOT);
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
        spawn(PJ_STAR, p, c, vx, vy, 80, 0, 0);
    }
    burst(IT_HOP_SHOT, p, c, 6.0, 0);
    log_item(b"item fires", IT_HOP_SHOT, p as i32 * 100 + n);
}

/// 5 Ledge Spikes: grabbing ledge stabs a column of ice spikes up from the lip, launching anyone above it.
unsafe fn item_ledge_spikes(p: usize, me: P) {
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
    let size = 7.0 * m as f32 / 1000.0;
    for dir in [-1, 1] {
        if let Some(i) = spawn(PJ_QUAKE, p, P { x: vp.x, y: me.y }, 3.2 * dir as f32, 0.0, 26 + 8 * (m / 500 - 2), dir, x100(size)) {
            S.pj_hit[i] = 1 << v;
        }
    }
    burst(IT_SMASH_QUAKE, p, P { x: vp.x, y: me.y }, size * 2.0, 0);
    log_item(b"item fires", IT_SMASH_QUAKE, p as i32 * 100 + m / 100);
}

/// 15 Moon Drops: two moons (two more per copy) orbit the fighter and nick foes they pass through.
unsafe fn item_moon_drops(p: usize) {
    let Some(me) = pos(p as i32) else { return };
    let n = 2 * more(p, IT_MOON_DROPS);
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
                nick(p, s, 2000);
                burst(IT_MOON_DROPS, p, m, 4.0, 0);
                log_item(b"item nick", IT_MOON_DROPS, s);
                break;
            }
        }
    }
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

// 35 Homing Ink and 36 Fire Lace are modifiers: Homing Ink steers every projectile (proj_frame), Fire Lace burns on
// every item hit (nick / boom) and puts embers on every effect (the PF_LACE look flag).

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
            IT_HOMING_INK | IT_FIRE_LACE => {
                demo_fire(IT_SPARK_RING);
                demo_fire(IT_HOP_SHOT);
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
        IT_MOON_DROPS => burst(id, p, me, 14.0, 0),
        IT_HOMING_INK | IT_FIRE_LACE => {
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
            PJ_SPARK | PJ_STAR => {
                let (mut vx, mut vy) = (S.pv[i][0] as f32 / 100.0, S.pv[i][1] as f32 / 100.0);
                if q[0] == PJ_STAR {
                    vy -= 0.03;
                }
                if has(p, IT_HOMING_INK) {
                    let t = foe_near(p, at, 160.0, S.pj_hit[i]);
                    if let Some(f) = t.ge(&0).then(|| pos(t)).flatten() {
                        let (dx, dy) = (f.x - at.x, f.y + 8.0 - at.y);
                        let l = sqrtf(dx * dx + dy * dy).max(0.001);
                        let sp = sqrtf(vx * vx + vy * vy).max(1.2);
                        let k = 0.08 * mul(p, IT_HOMING_INK) as f32 / 1000.0;
                        vx += (dx / l * sp - vx) * k;
                        vy += (dy / l * sp - vy) * k;
                    }
                }
                let nx = at.x + vx;
                let ny = at.y + vy;
                S.pv[i] = [x100(vx), x100(vy)];
                S.proj[i][2] = x100(nx);
                S.proj[i][3] = x100(ny);
                let now = P { x: nx, y: ny };
                let r = if q[0] == PJ_STAR { 7.0 } else { 6.0 };
                let s = foe_near(p, now, r + 4.0, S.pj_hit[i]);
                if s >= 0 {
                    if q[0] == PJ_STAR {
                        boom(p, now, 7.0, 5, 50, 45, 35, s);
                        burst(IT_HOP_SHOT, p, now, 9.0, 0);
                        proj_end(i);
                    } else {
                        S.pj_hit[i] |= 1 << s;
                        nick(p, s, 2000);
                        burst(IT_SPARK_RING, p, now, 4.0, 0);
                    }
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
            _ => proj_end(i),
        }
    }
}

unsafe fn mine_pop(p: usize, i: usize, at: P) {
    let c = P { x: at.x, y: at.y + 5.0 };
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
    if sched().kind == FK_BOSS {
        PED_BOSS
    } else if session_last(f) && test_waves() < 0 {
        PED_ITEM_ROOM
    } else {
        0
    }
}

pub(crate) unsafe fn ped_open(kind: i32) {
    S.ped_kind = kind;
    S.ped_left = PED_TIME;
    let coop = nh() == 2;
    for p in 0..nh() as usize {
        let base = if coop { if p == 0 { -32.0 } else { 32.0 } } else { 0.0 };
        S.drop[p] = [[0; 4]; DROPS];
        if kind == PED_ITEM_ROOM {
            let id = roll(p, PL_ITEM, 0x11, &[]);
            if id > 0 {
                S.drop[p][0] = [1, id, x100(base), 0];
            }
        } else {
            // the boss's choice of two: a new item, and a copy of one already held when there is one
            let a = roll(p, PL_ITEM | PL_BOSS, 0x22, &[]);
            let mut b = 0;
            let held: [i32; 12] = ORDER;
            let start = (mix(S.seed as u32 ^ 0xB055_0000 ^ fkey() << 4 ^ p as u32) % 12) as usize;
            for k in 0..12 {
                let id = held[(start + k) % 12];
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
                log_item(if kind == PED_BOSS { b"boss drop on a pedestal" } else { b"item room pedestal" }, S.drop[p][k][1], p as i32);
            }
        }
    }
    log(if kind == PED_BOSS { b"envoy: boss drop, choose one\0" } else { b"envoy: item room open\0" }, S.fight + 1);
}

/// A pedestal walked into (or flown in): the item is the player's; the boss's other choice is gone.
pub(crate) unsafe fn ped_take(p: usize, k: usize, how: i32) {
    let id = S.drop[p][k][1];
    S.gain_from[p] = [S.drop[p][k][2], S.drop[p][k][3], p as i32];
    S.drop[p][k][0] = 2;
    give(p, id, how);
    if S.ped_kind == PED_BOSS {
        for j in 0..DROPS {
            if j != k && S.drop[p][j][0] == 1 {
                S.drop[p][j][0] = 0;
                log_item(b"boss drop left behind", S.drop[p][j][1], p as i32);
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
    if S.ped_left > 0 {
        S.ped_left -= 1;
        if S.ped_left % 300 == 0 {
            log(b"envoy: pedestal wait, frames left ", S.ped_left);
        }
        return true;
    }
    if S.ped_kind == PED_BOSS {
        for p in 0..NP {
            if S.drop[p][0][0] == 1 && S.drop[p][1][0] == 1 {
                S.drop[p][1][0] = 0;
            }
        }
    }
    false
}

/// The campfire (after the boss): stocks back up to the limit, never past it.
pub(crate) unsafe fn campfire() {
    let gain = (S.lim - S.pool).max(0);
    S.pool = S.lim;
    S.camp_at = S.play.max(1);
    S.camp_gain = gain;
    for h in 0..nh() {
        gd_envoy_apply(h, AP_STOCKS, S.pool);
    }
    log(b"envoy: campfire, stocks refilled to\0", S.pool);
}

// ---- the record (v9) and the view

/// The item words of the record: player p's copies, a byte per id, in R_DRIVES + p * 10 .. + 10.
pub(crate) unsafe fn rec_put(r: &mut [i32; REC_WORDS]) {
    for w in R_DRIVES..R_DRIVES + 20 {
        r[w] = 0;
    }
    for p in 0..NP {
        for id in 1..ITEM_IDS {
            let c = S.inv[p][id].clamp(0, 255);
            let w = R_DRIVES + p * 10 + id / 4;
            r[w] |= c << (8 * (id % 4));
        }
    }
    for k in 0..6 {
        r[R_KEYS + k] = 0;
    }
    r[R_KEYS] = S.lim;
}

pub(crate) unsafe fn rec_get(r: &[i32; REC_WORDS]) {
    S.inv = [[0; ITEM_IDS]; NP];
    for p in 0..NP {
        for id in 1..ITEM_IDS {
            let c = (r[R_DRIVES + p * 10 + id / 4] as u32 >> (8 * (id % 4)) & 0xFF) as i32;
            if item(id as i32).is_some() {
                S.inv[p][id] = c;
            }
        }
    }
    S.lim = r[R_KEYS].clamp(1, LIM_CAP);
}

pub(crate) unsafe fn publish(v: &mut [i32; VIEW_WORDS]) {
    v[V_POOL_LIM] = S.lim;
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
        if matches!(q[0], PJ_SPARK | PJ_STAR | PJ_COIN) {
            v[b + 6] = S.pv[i][0];
            v[b + 7] = S.pv[i][1];
        }
        n += 1;
    }
    // the moons, after the pool's own entries (the UI draws them the same way)
    for p in 0..nh() as usize {
        if !has(p, IT_MOON_DROPS) || !alive(p as i32) {
            continue;
        }
        let Some(me) = pos(p as i32) else { continue };
        let m = 2 * more(p, IT_MOON_DROPS);
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
    for p in 0..NP {
        for id in 0..ITEM_IDS {
            v[V_INV + p * ITEM_IDS + id] = S.inv[p][id];
        }
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
