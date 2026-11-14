//! Envoy slice 2 rooms (docs/rust-port/notes/envoy-items-design.md section 11): each player's own stock limit and
//! stocks, the devil / angel door after the boss, the secret room behind a cracked wall, the fork after the item room
//! (the shop or the campfire), the shop's stock slot, and the floor map the UI draws. All of it is sim state inside
//! `St` (hashed, rolled back); every roll comes from the run seed and the loop.
use super::*;

/// Each player's stock limit and stocks at a run's start (co-op: each player their own).
pub(crate) const LIM_START: i32 = 4;
/// The devil door's chance after a boss: the base, more with no stock lost on the floor, less per deal taken.
const DOOR_BASE: i32 = 35;
const DOOR_CLEAN: i32 = 15;
const DOOR_PER_DEAL: i32 = 20;
/// With no deal taken this run, an opened door is the angel's this often (percent).
const ANGEL_PCT: i32 = 50;
/// The cracked wall: hits it takes, the frames between two hits from one player, its reach, its x (inside every
/// stage's main floor: the narrowest ledge is 56).
pub(crate) const SECRET_HP: i32 = 3;
const SECRET_CD: i32 = 18;
const WALL_X: f32 = 44.0;
const WALL_REACH_X: f32 = 18.0;
const WALL_H: f32 = 26.0;
/// The shop's stock slot: how often a shop has one (percent), its price (+40 a loop past the first).
const SLOT_SALE_PCT: i32 = 30;
const SLOT_PRICE: i32 = 160;
/// The devil room's wait (frames) before its unbought pedestals vanish.
pub(crate) const DEVIL_TIME: i32 = 900;

// ---- stocks (Isaac's hearts, per player)

pub(crate) unsafe fn lim(p: usize) -> i32 {
    S.lims[p.min(NP - 1)]
}

/// Player `p` gains `n` stock slots and `n` stocks (to the hard cap): Spare Heart, the shop's slot.
pub(crate) unsafe fn add_slot(p: usize, n: i32) {
    if p >= NP {
        return;
    }
    let before = S.lims[p];
    S.lims[p] = (S.lims[p] + n).min(items::LIM_CAP);
    let got = S.lims[p] - before;
    S.pools[p] = (S.pools[p] + got.max(0)).min(S.lims[p]);
    if S.live != 0 && got > 0 && gd_player_stocks(p as i32) > 0 {
        gd_envoy_apply(p as i32, AP_STOCKS, (gd_player_stocks(p as i32) + got).min(S.lims[p]));
    }
    ev(EV_SLOT, p as i32, S.lims[p]);
    log(b"envoy: stock slot gained, player*100 + limit\0", p as i32 * 100 + S.lims[p]);
    make_plan();
}

/// Player `p` pays `c` stock slots for a deal: the limit drops, stocks above it are lost. The last slot is never paid.
pub(crate) unsafe fn pay_slots(p: usize, c: i32) -> bool {
    if p >= NP || c <= 0 || S.lims[p] - c < 1 {
        return false;
    }
    S.lims[p] -= c;
    S.pools[p] = S.pools[p].min(S.lims[p]);
    if S.live != 0 && gd_player_stocks(p as i32) > S.lims[p] {
        gd_envoy_apply(p as i32, AP_STOCKS, S.lims[p]);
    }
    ev(EV_SLOT, p as i32, S.lims[p]);
    log(b"envoy: stock slots paid, player*100 + limit now\0", p as i32 * 100 + S.lims[p]);
    log(b"envoy: stock limits now, p1*100 + p2\0", S.lims[0] * 100 + S.lims[1]);
    true
}

/// Whether player `p` can pay `c` slots (never their last).
pub(crate) unsafe fn can_pay(p: usize, c: i32) -> bool {
    p < NP && S.lims[p] - c >= 1
}

/// The campfire: each player's stocks back up to their own limit (never past it).
pub(crate) unsafe fn campfire() {
    S.camp_at = S.play.max(1);
    S.camp_gain = 0;
    for p in 0..nh() as usize {
        let gain = (S.lims[p] - S.pools[p]).max(0);
        S.pools[p] = S.lims[p];
        S.camp_gain += gain;
        if S.live != 0 {
            gd_envoy_apply(p as i32, AP_STOCKS, S.lims[p]);
        }
        ev(EV_CAMPFIRE, p as i32, gain);
        log(b"envoy: campfire, player*100 + stocks refilled to\0", p as i32 * 100 + S.pools[p]);
    }
    log(b"envoy: campfire, stocks refilled to\0", S.pools[0]);
    make_plan();
}

// ---- the floor (one loop): the secret wall's fight, the door chance

/// A new floor (a new run, or the loop after a boss): no fork taken, no secret found, no stock lost yet; the wall's
/// fight rolled from the seed (fights 1-3, never the boss).
pub(crate) unsafe fn floor_start() {
    S.fork = 0;
    S.fork_pick = 0;
    S.path = [-1; 8];
    S.map_cur = 1;
    S.map_wish = -1;
    S.secret_state = 0;
    S.secret_hits = 0;
    S.floor_lost = 0;
    S.door = 0;
    S.door_at = 0;
    let forced = test_secret();
    S.secret_fight = if forced > 0 { forced - 1 } else { (mix(S.seed as u32 ^ 0x5EC2_E700 ^ (lp() as u32) << 8) % 3) as i32 };
    // the wall sits on one node of that fight row (a path that goes elsewhere never sees it); a test knob puts it
    // on every lane
    S.secret_lane = if forced > 0 { -1 } else { (mix(S.seed as u32 ^ 0x5EC2_1A4E ^ (lp() as u32) << 8) % LANES as u32) as i32 };
    log(b"envoy: floor map, loop*10 + secret wall fight\0", lp() * 10 + S.secret_fight + 1);
}

static mut T_SECRET: i32 = -1;
/// Test knob `secret:N`: the cracked wall in fight N (1-3).
unsafe fn test_secret() -> i32 {
    if T_SECRET < 0 {
        T_SECRET = 0;
        for n in 1..=3 {
            let k = match n {
                1 => "secret:1",
                2 => "secret:2",
                _ => "secret:3",
            };
            if test_flag(k) {
                T_SECRET = n;
            }
        }
    }
    T_SECRET
}

/// The chance the door opens after this floor's boss (percent).
pub(crate) unsafe fn door_pct() -> i32 {
    (DOOR_BASE + if S.floor_lost == 0 { DOOR_CLEAN } else { 0 } - DOOR_PER_DEAL * S.deals_any).clamp(0, 100)
}

/// After the boss: roll the door (DOOR_*; 0 none). Test knobs `door:devil` / `door:angel` force it.
unsafe fn door_roll() -> i32 {
    if test_flag("door:devil") {
        return DOOR_DEVIL;
    }
    if test_flag("door:angel") {
        return DOOR_ANGEL;
    }
    if test_flag("nodoor") {
        return 0;
    }
    let h = mix(S.seed as u32 ^ 0xD00E_0000 ^ (lp() as u32) << 4);
    let pct = door_pct();
    if (h % 100) as i32 >= pct {
        log(b"envoy: no door after the boss, chance\0", pct);
        return 0;
    }
    let angel = S.deals_any == 0 && (mix(h ^ 0xA9E1) % 100) < ANGEL_PCT as u32;
    if angel { DOOR_ANGEL } else { DOOR_DEVIL }
}

// ---- the act map (envoy-items-design.md 11.4): a Slay the Spire style map per loop
//
// Rows from the bottom: fight rows 0, 2, 4 (the loop's fights 1-3), room rows 1, 3, 5 (rooms run in the held match
// after the fight below them), the boss (row 6) and the door after it (row 7). Three lanes a row; every node leads
// to the same lane above and, seeded, to one neighbour (never two edges crossing). Made from the run seed and the
// loop only: the same on both co-op players' screens, on both netplay peers and on replay.

pub(crate) const LANES: usize = 3;
pub(crate) const ROWS: usize = 6; // rows 0-5 hold LANES nodes each; the boss and the door follow
pub(crate) const N_BOSS: usize = ROWS * LANES; // 18
pub(crate) const N_DOOR: usize = N_BOSS + 1;
pub(crate) const N_SECRET: usize = N_BOSS + 2;

/// The act's seed word.
unsafe fn act_h(salt: u32) -> u32 {
    mix(S.seed as u32 ^ 0xAC7_0000 ^ (lp() as u32) << 8 ^ salt)
}

/// A shuffle of three kinds by a seed word.
fn three(k: [i32; 3], h: u32) -> [i32; 3] {
    let o = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]][(h % 6) as usize];
    [k[o[0]], k[o[1]], k[o[2]]]
}

/// The node kind (MN_*) at row `r`, lane `l`. The rules (Slay the Spire's, at this act size): the first fights are
/// duels and a crowd, never an elite; the first room is an item room (no shop or campfire that early); elites in
/// the middle and late fights; the middle room mixes a shop, a campfire and an item room (one shop a path at most);
/// the row before the boss is all campfires.
pub(crate) unsafe fn node_kind(r: usize, l: usize) -> i32 {
    let l = l.min(LANES - 1);
    match r {
        0 => three([MN_DUEL, MN_DUEL, MN_CROWD], act_h(0x10))[l],
        1 => MN_ITEM,
        2 => three([MN_ELITE, MN_CROWD, MN_DUEL], act_h(0x12))[l],
        3 => three([MN_SHOP, MN_CAMPFIRE, MN_ITEM], act_h(0x13))[l],
        4 => three([MN_ELITE, MN_DUEL, MN_CROWD], act_h(0x14))[l],
        5 => MN_CAMPFIRE,
        _ => MN_BOSS,
    }
}

/// The lanes of row `r + 1` lane `l` leads to (bits).
pub(crate) unsafe fn links(r: usize, l: usize) -> i32 {
    if r + 1 >= ROWS {
        return 0; // the top room row leads to the boss
    }
    let mut b = 1 << l;
    for g in 0..LANES - 1 {
        // between lanes g and g + 1: none, up-right (g -> g + 1) or up-left (g + 1 -> g); never both
        match act_h(0x100 + (r * 4 + g) as u32) % 3 {
            1 if l == g => b |= 1 << (g + 1),
            2 if l == g + 1 => b |= 1 << g,
            _ => {}
        }
    }
    b
}

/// Whether row `r` lane `l` can be reached from the run's path (row 0: every lane).
pub(crate) unsafe fn reachable(r: usize, l: usize) -> bool {
    if r == 0 {
        return true;
    }
    let pl = S.path[r - 1];
    pl >= 0 && links(r - 1, pl as usize) >> l & 1 != 0
}

/// The template (SCHEDULE / ROOM_MIX index) of fight `f` (0-3): the chosen (or the cursor's) fight node's kind.
pub(crate) unsafe fn template(f: usize) -> usize {
    if f >= 3 {
        return 3;
    }
    let r = f * 2;
    let l = if S.path[r] >= 0 { S.path[r] } else { cursor_lane(r) };
    match node_kind(r, l as usize) {
        MN_CROWD => 1,
        MN_ELITE => 2,
        _ => 0,
    }
}

/// The cursor on row `r`: a reachable lane (the remembered cursor when it is reachable, else the first).
pub(crate) unsafe fn cursor_lane(r: usize) -> i32 {
    let c = S.map_cur.clamp(0, LANES as i32 - 1);
    if reachable(r, c as usize) {
        return c;
    }
    (0..LANES as i32).find(|&l| reachable(r, l as usize)).unwrap_or(0)
}

/// The road screen before fight `f` (out of a match): P1's left / right walks the reachable fight nodes of its row
/// (the plan follows the cursor); P2's plants a wish. The node is taken when the match starts.
pub(crate) unsafe fn map_road_frame(f: usize) {
    if f >= 3 {
        return;
    }
    let r = f * 2;
    for p in 0..(1 + (gd_envoy_coop() != 0 || S.nh == 2) as i32) {
        let pad = gd_pad_pressed(p);
        let d = (pad & (DPAD_RIGHT | STICK_RIGHT) != 0) as i32 - (pad & (DPAD_LEFT | STICK_LEFT) != 0) as i32;
        if d == 0 {
            continue;
        }
        if p == 0 {
            let mut c = cursor_lane(r);
            for _ in 0..LANES {
                c = (c + d).rem_euclid(LANES as i32);
                if reachable(r, c as usize) {
                    break;
                }
            }
            S.map_cur = c;
            log(b"envoy: map cursor, row*10 + lane\\0", r as i32 * 10 + c);
            make_plan();
        } else {
            S.map_wish = (S.map_wish.max(0) + d).rem_euclid(LANES as i32);
        }
    }
}

/// A fight starts: its node is the cursor's (the path takes it).
pub(crate) unsafe fn take_fight_node() {
    let f = S.fight.clamp(0, 3) as usize;
    if f < 3 {
        let r = f * 2;
        if S.path[r] < 0 {
            S.path[r] = cursor_lane(r);
            log(b"envoy: map node taken, row*10 + lane\\0", r as i32 * 10 + S.path[r]);
            log(b"envoy: map fight kind (MN)\\0", node_kind(r, S.path[r] as usize));
        }
    }
}

/// The next room of the held match after a won fight (outcome_frame's collect, once the open one is done and its
/// pedestals flew in). A fight: the room choice above it (the fork), the room taken (the item room's pedestals;
/// the campfire; the shop opens at the stop), the secret room. The boss: its drop, the door. False when done.
pub(crate) unsafe fn advance() -> bool {
    let boss = sched().kind == FK_BOSS;
    loop {
        let opened = match S.ped_done {
            0 => {
                S.ped_done = 1;
                if boss {
                    items::ped_open(PED_BOSS);
                    true
                } else if test_waves() < 0 && S.no_shop == 0 && S.fight < 3 {
                    fork_open();
                    true
                } else {
                    false
                }
            }
            1 => {
                S.ped_done = 2;
                if boss {
                    if test_waves() < 0 {
                        S.door = door_roll();
                    }
                    if S.door != 0 {
                        S.door_at = S.play.max(1);
                        ev(EV_DOOR, 0, S.door);
                        log(if S.door == DOOR_DEVIL { b"envoy: the red door opens (devil deal)\\0" } else { b"envoy: the white door opens (angel room)\\0" }, door_pct());
                        items::ped_open(if S.door == DOOR_DEVIL { PED_DEVIL } else { PED_ANGEL });
                    }
                    S.door != 0
                } else {
                    room_run()
                }
            }
            2 => {
                S.ped_done = 3;
                if S.secret_state == 2 {
                    S.secret_state = 3;
                    log(b"envoy: secret room open, fight\\0", S.fight + 1);
                    items::ped_open(PED_SECRET);
                    true
                } else {
                    false
                }
            }
            _ => return false,
        };
        if opened {
            S.collect = 1; // the fly-in clock starts again for this room's pedestals
            return true;
        }
    }
}

/// The room row above this fight.
unsafe fn room_row() -> usize {
    (S.fight.clamp(0, 2) as usize) * 2 + 1
}

pub(crate) unsafe fn fork_open() {
    let r = room_row();
    S.fork = 1;
    S.fork_cur = cursor_lane(r);
    S.fork_left = 0;
    log(b"envoy: map room choice open, row\\0", r as i32);
}

/// One frame of the room choice (no timer: the owner's "we shouldn't be rushed on making our build"): P1 walks the
/// reachable room nodes (left / right) and takes one with A. Test knob `autotake` (CPU-driven checks only) takes the
/// cursor's room after a second; `campfire` / `forkshop` / `forkitem` steer it to that kind.
pub(crate) unsafe fn fork_wait() -> bool {
    if S.fork != 1 {
        return false;
    }
    let r = room_row();
    let pad = gd_pad_pressed(0);
    let d = (pad & (DPAD_RIGHT | STICK_RIGHT) != 0) as i32 - (pad & (DPAD_LEFT | STICK_LEFT) != 0) as i32;
    if d != 0 {
        let mut c = S.fork_cur;
        for _ in 0..LANES {
            c = (c + d).rem_euclid(LANES as i32);
            if reachable(r, c as usize) {
                break;
            }
        }
        S.fork_cur = c;
    }
    S.fork_left += 1; // frames open (no countdown)
    let want = if test_flag("campfire") {
        MN_CAMPFIRE
    } else if test_flag("forkshop") {
        MN_SHOP
    } else if test_flag("forkitem") {
        MN_ITEM
    } else {
        0
    };
    if want != 0 {
        if let Some(l) = (0..LANES).find(|&l| reachable(r, l) && node_kind(r, l) == want) {
            S.fork_cur = l as i32;
        }
    }
    let auto = (test_flag("autotake") || want != 0) && S.fork_left > 60;
    if pad & BTN_A == 0 && !auto {
        return true;
    }
    S.fork = 2;
    S.path[r] = S.fork_cur;
    S.map_cur = S.fork_cur;
    let k = node_kind(r, S.fork_cur as usize);
    S.fork_pick = k;
    log(b"envoy: map room taken, row*10 + lane\\0", r as i32 * 10 + S.fork_cur);
    log(b"envoy: map room kind (MN)\\0", k);
    false
}

/// Run the room taken: the item room's pedestals, the campfire; the shop opens at the stop (begin_stop).
unsafe fn room_run() -> bool {
    match S.fork_pick {
        MN_ITEM => {
            items::ped_open(PED_ITEM_ROOM);
            true
        }
        MN_CAMPFIRE => {
            campfire();
            S.skip_shop = 1;
            false
        }
        MN_SHOP => false,
        _ => {
            S.skip_shop = 1;
            false
        }
    }
}

// ---- the secret wall

/// The first live frame of a fight: the cracked wall stands when this is the floor's secret fight.
pub(crate) unsafe fn fight_start() {
    let lane_ok = S.secret_lane < 0 || S.path[(S.fight.clamp(0, 2) * 2) as usize] == S.secret_lane;
    if S.fight == S.secret_fight && lane_ok && S.secret_state == 0 && sched().kind != FK_BOSS && test_waves() < 0 {
        S.secret_state = 1;
        S.secret_hits = 0;
        let side = if mix(S.seed as u32 ^ 0x0A11 ^ (lp() as u32) << 8) & 1 == 0 { -1.0 } else { 1.0 };
        let mut x = side * WALL_X;
        if test_flag("wallnear") {
            // the wall beside player 1's spawn (screenshots, the scripted check)
            x = pos(0).map_or(x, |m| (m.x + 14.0).clamp(-WALL_X, WALL_X));
        }
        S.secret_x = x100(x);
        S.secret_y = 0;
        S.secret_cd = [0; NP];
        log(b"envoy: cracked wall stands, x\0", x as i32);
    }
}

/// Per live frame: a player's attack near the standing wall cracks it; SECRET_HP hits break it (the secret room
/// opens after the fight).
pub(crate) unsafe fn wall_frame() {
    if test_flag("fork") && S.play == 120 && S.fork == 0 {
        fork_open(); // the screenshot knob: the room choice over a live fight
    }
    if S.fork == 1 && S.collect == 0 && fork_wait() == false {
        S.fork = 0; // the knob's choice is only a look: the real one comes after the fight
        S.path[room_row()] = -1;
    }
    if S.secret_state != 1 {
        return;
    }
    if test_flag("wallnear") {
        // the scripted check: the wall keeps beside player 1 (a CPU), so any attack of theirs reaches it
        if let (Some(m), h) = (pos(0), gd_fighter_from_slot(0)) {
            let face = if h != 0 && gd_fighter_get_facing(h) < 0.0 { -1.0 } else { 1.0 };
            S.secret_x = x100((m.x + face * 10.0).clamp(-WALL_X, WALL_X));
            S.secret_y = x100(m.y.max(0.0));
        }
    }
    let (wx, wy) = (S.secret_x as f32 / 100.0, S.secret_y as f32 / 100.0);
    for p in 0..nh() as usize {
        if S.secret_cd[p] > 0 {
            S.secret_cd[p] -= 1;
            continue;
        }
        let h = gd_fighter_from_slot(p as i32);
        if h == 0 {
            continue;
        }
        let st = gd_fighter_get_action_state(h);
        // any attack: the ground and air normals, the smashes and the specials
        let attacking = (AS_ATTACK0..=74).contains(&st) || st >= 295;
        if !attacking {
            continue;
        }
        let Some(me) = pos(p as i32) else { continue };
        if (me.x - wx).abs() < WALL_REACH_X && me.y > wy - 6.0 && me.y < wy + WALL_H {
            S.secret_hits += 1;
            S.secret_cd[p] = SECRET_CD;
            log(b"envoy: cracked wall hit, player*10 + hits\0", p as i32 * 10 + S.secret_hits);
            if S.secret_hits >= SECRET_HP {
                S.secret_state = 2;
                ev(EV_SECRET, p as i32, S.fight);
                log(b"envoy: secret wall broken, fight\0", S.fight + 1);
                return;
            }
        }
    }
}

// ---- the shop's stock slot

pub(crate) unsafe fn roll_slot_sale() {
    let h = mix(S.seed as u32 ^ 0x510_7000 ^ (lp() as u32) << 4 ^ (S.fight as u32) << 12);
    S.slot_sale = if test_flag("slotsale") || ((h % 100) as i32) < SLOT_SALE_PCT { SLOT_PRICE + 40 * lx() } else { 0 };
    if S.slot_sale > 0 {
        log(b"envoy: shop has a stock slot, price\0", S.slot_sale);
    }
}

/// The SHOP_SLOT row bought by player `p`.
pub(crate) unsafe fn buy_slot(p: usize) -> i32 {
    if S.slot_sale <= 0 || S.lims[p] >= items::LIM_CAP {
        return SM_CANT;
    }
    if !spend(S.slot_sale) {
        return SM_POOR;
    }
    S.slot_sale = 0;
    add_slot(p, 1);
    log(b"envoy: shop bought a stock slot, player\0", p as i32);
    SM_STOCK
}

// ---- the record (R_KEYS[5]: this floor's fork, wall and door state)

pub(crate) unsafe fn rec_word() -> i32 {
    let mut w = 0;
    for r in 0..ROWS {
        w |= (S.path[r] + 1 & 3) << (2 * r);
    }
    w | (S.secret_state & 3) << 12 | (S.floor_lost & 1) << 14 | (S.deals_any & 0xFF) << 16
}

pub(crate) unsafe fn rec_word_get(w: i32) {
    floor_start();
    for r in 0..ROWS {
        S.path[r] = (w >> (2 * r) & 3) - 1;
    }
    S.secret_state = w >> 12 & 3;
    if S.secret_state == 1 {
        S.secret_state = 0; // a standing wall comes back when its fight starts again
    }
    S.floor_lost = w >> 14 & 1;
    S.deals_any = w >> 16 & 0xFF;
}

// ---- the view: per-player stocks, the rooms, the map

pub(crate) unsafe fn publish(v: &mut [i32; VIEW_WORDS]) {
    for p in 0..NP {
        let live = S.live != 0 && (p as i32) < nh();
        v[V_PSTOCK + p] = if live { gd_player_stocks(p as i32).max(0) } else { S.pools[p] };
        v[V_PLIM + p] = S.lims[p];
        v[V_DEALS + p] = S.deals[p];
        for k in 0..DROPS {
            v[V_PED_COST + p * DROPS + k] = S.ped_cost[p][k];
        }
    }
    v[V_DOOR] = S.door;
    v[V_DOOR_AT] = S.door_at;
    v[V_SECRET] = if S.fight == S.secret_fight || S.secret_state == 3 { S.secret_state } else { 0 };
    v[V_SECRET_X] = S.secret_x;
    v[V_SECRET_Y] = S.secret_y;
    v[V_SECRET_HITS] = S.secret_hits;
    v[V_SECRET_HP] = SECRET_HP;
    v[V_FORK] = (S.fork == 1) as i32;
    v[V_FORK_CUR] = S.fork_cur;
    v[V_FORK_LEFT] = 0; // no timer
    v[V_SHOP_SLOT_PRICE] = if S.rw_active == RW_SHOP { S.slot_sale } else { 0 };
    v[V_DOOR_PCT] = door_pct();
    v[V_SHAKE] = S.shake;
    map_publish(v);
}

/// The act map (view.rs V_MAP*): every node with its kind, place, state and edges.
unsafe fn map_publish(v: &mut [i32; VIEW_WORDS]) {
    let f = S.fight.clamp(0, 3) as usize;
    let won = S.outcome == 1 || S.collect > 0;
    // the row the run is on: the fight in play (or next), the room above it while the held match runs it
    let here_row = if f >= 3 { ROWS } else if won && S.fork >= 1 { f * 2 + 1 } else { f * 2 };
    let picking = if S.live == 0 && S.arm_seen != 0 && f < 3 { 1 } else if S.fork == 1 { 2 } else { 0 };
    let wall_row = if (0..3).contains(&S.secret_fight) { S.secret_fight as usize * 2 } else { 99 };
    v[V_MAP_N] = (N_SECRET + 1) as i32;
    v[V_MAP_LOOP] = lp();
    v[V_MAP_ROWS] = ROWS as i32 + 2;
    v[V_MAP_LANES] = LANES as i32;
    v[V_MAP_PICK] = picking;
    let mut at = -1;
    let mut secret = -1;
    for r in 0..ROWS {
        for l in 0..LANES {
            let i = r * LANES + l;
            let b = V_MAP0 + i * MAP_W;
            let mut st = 0;
            let taken = S.path[r] == l as i32;
            if r < here_row && taken {
                st |= MS_DONE | MS_CHOSEN;
            } else if r < here_row {
                st |= MS_SKIPPED;
            }
            if r == here_row && (taken || (picking == 0 && S.path[r] < 0 && false)) {
                st |= MS_HERE;
                at = i as i32;
            }
            if r == here_row && reachable(r, l) && !taken {
                st |= MS_NEXT;
            }
            let cur = if picking == 2 && r == here_row { S.fork_cur } else if picking == 1 && r == here_row { cursor_lane(r) } else { -1 };
            if cur == l as i32 {
                st |= MS_CURSOR | MS_NEXT;
            }
            if picking == 1 && r == here_row && S.map_wish == l as i32 {
                st |= MS_WISH;
            }
            // the planned path ahead: the cursor's lane carried straight up while it stays connected
            if r > here_row && planned(here_row, r) == l as i32 {
                st |= MS_PLAN;
            }
            if r == wall_row && (S.secret_lane < 0 || S.secret_lane == l as i32) {
                st |= MS_WALL;
                if S.secret_state >= 2 {
                    st |= MS_FOUND;
                }
                if S.secret_lane == l as i32 || (S.secret_lane < 0 && secret < 0) {
                    secret = i as i32;
                }
            }
            v[b + MP_KIND] = node_kind(r, l);
            v[b + MP_POS] = l as i32 | (r as i32) << 8;
            v[b + MP_STATE] = st;
            v[b + MP_LINKS] = if r + 1 < ROWS { links(r, l) << ((r + 1) * LANES) } else { 1 << N_BOSS };
        }
    }
    // the boss, the door, the secret room
    let b = V_MAP0 + N_BOSS * MAP_W;
    v[b + MP_KIND] = MN_BOSS;
    v[b + MP_POS] = 1 | (ROWS as i32) << 8;
    v[b + MP_STATE] = if f >= 3 { if won { MS_DONE } else { MS_HERE } } else { 0 };
    v[b + MP_LINKS] = 1 << N_DOOR;
    if f >= 3 && !won {
        at = N_BOSS as i32;
    }
    let b = V_MAP0 + N_DOOR * MAP_W;
    v[b + MP_KIND] = match S.door {
        DOOR_DEVIL => MN_DEVIL,
        DOOR_ANGEL => MN_ANGEL,
        _ => MN_DOOR,
    };
    v[b + MP_POS] = 1 | (ROWS as i32 + 1) << 8;
    v[b + MP_STATE] = if S.door != 0 { MS_HERE } else if f >= 3 && S.ped_done >= 2 { MS_SKIPPED } else { MS_UNKNOWN };
    if S.door != 0 {
        at = N_DOOR as i32;
    }
    let b = V_MAP0 + N_SECRET * MAP_W;
    let wl = if secret >= 0 { secret as usize % LANES } else { 0 };
    v[b + MP_KIND] = if secret >= 0 { MN_SECRET } else { 0 };
    v[b + MP_POS] = 3 | (wall_row.min(ROWS) as i32) << 8 | (wl as i32) << 16; // lane 3: beside the row; bits 16+: the wall's lane
    v[b + MP_STATE] = if S.secret_state >= 2 { MS_FOUND | if S.secret_state == 3 { MS_DONE } else { 0 } } else { MS_UNKNOWN };
    v[b + MP_LINKS] = 0;
    if secret >= 0 {
        let wb = V_MAP0 + secret as usize * MAP_W;
        v[wb + MP_LINKS] |= 1 << N_SECRET;
    }
    v[V_MAP_AT] = at;
    v[V_MAP_SECRET] = secret;
    // the road screen's pass-through (road1's mechanism): more than one reachable fight node is a fork
    if picking == 1 {
        let r = f * 2;
        let n = (0..LANES).filter(|&l| reachable(r, l)).count() as i32;
        v[V_ROAD_N] = n;
        v[V_ROAD_CUR] = cursor_lane(r);
    }
}

/// The lane the plan reaches on row `r`, walking up from row `from` along the cursor's lane (the same lane while
/// linked, else the first linked one).
unsafe fn planned(from: usize, r: usize) -> i32 {
    let mut l = if from < ROWS { if S.path[from] >= 0 { S.path[from] } else { cursor_lane(from) } } else { return -1 };
    for rr in from..r {
        let ls = links(rr, l as usize);
        if ls >> l & 1 == 0 {
            l = (0..LANES as i32).find(|&k| ls >> k & 1 != 0).unwrap_or(l);
        }
    }
    l
}
