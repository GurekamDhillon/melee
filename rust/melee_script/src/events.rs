//! Engine events (`gw_Script_GameEvent` and friends, `gs_dispatch_events`, `gw_script_spine_events.inc`).
//! The game reports events from inside a logic frame; they are only queued, and the hooks run after the frame
//! (`Engine::frame_post`), never mid-frame and never for a resimulated frame.
use crate::engine::{Engine, State};
use crate::game::lab::{HF_COUNT, HI_COUNT};
use crate::gd::entity::actor_text;
use crate::gd::players::{element_name, push_hit_fields};
use crate::gd::TableExt;
use mlua::{MultiValue, Table, Value};

pub const EV_ACTION: i32 = 1;
pub const EV_HIT: i32 = 2;
pub const EV_HITLAG: i32 = 3;
pub const EV_LAND: i32 = 4;
pub const EV_TARGET_BROKEN: i32 = 5;
pub const EV_ALL_TARGETS_BROKEN: i32 = 6;
pub const EV_BOSS_DEFEATED: i32 = 7;
pub const EV_ENEMY_DEFEATED: i32 = 8;
pub const EV_ENEMY_REMOVED: i32 = 9;
pub const EV_ENEMY_HIT: i32 = 10;
pub const EV_CLANK: i32 = 11;
pub const EV_ITEM_COLLECT: i32 = 12;
pub const EV_ITEM_EXPIRE: i32 = 13;
pub const EV_KO: i32 = 14;
pub const EV_STOCK_LOST: i32 = 15;
pub const EV_ACTION_SIGNAL: i32 = 16;
pub const EV_ARMOR: i32 = 17;
pub const EV_SKILL: i32 = 18;
pub const EV_CRIT: i32 = 19;

pub const HIT_INDEX_MASK: i32 = 0xFF;
pub const HIT_ATTACKER_SUB: i32 = 0x100;
pub const HIT_VICTIM_SUB: i32 = 0x200;
pub const HIT_BY_ITEM: i32 = 0x400;

pub const MAX_EVENTS: usize = 256;
pub const HIT_RULE_CONTEXT_MAX: usize = 96;
pub const ENEMY_NAMES: [&str; 7] = ["goomba", "koopa", "redead", "like_like", "octorok", "polar_bear", "topi"];

/// One field of a producer-built event payload table (clank, item, armor, skill and crit events)
#[derive(Clone, Debug)]
pub enum Pv {
    Int(i64),
    Num(f64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Debug, Default)]
pub struct Spine {
    pub frame: i32,
    pub phase: i32,
    pub epoch: u32,
    pub sequence: u32,
    pub cause: u32,
    pub actor: String,
    pub target: String,
}

#[derive(Clone, Debug)]
pub struct GsEvent {
    pub what: i32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub spine: Spine,
    pub hit: [i32; HI_COUNT as usize],
    pub hitf: [f32; HF_COUNT as usize],
    pub has_hit: bool,
    pub context_valid: bool,
    pub context_element: i32,
    pub context_action: i32,
    pub context_move: i32,
    pub context_original_element: i32,
    pub hit_rules: Vec<(i32, i32)>, // (id, owner)
    pub context_attacker_ground: i32,
    pub context_victim_ground: i32,
    pub context_attacker_damage: i32,
    pub context_victim_damage: i32,
    pub source_enemy: i32,
    pub reaction: bool,
    pub payload: Option<Vec<(String, Pv)>>,
}
impl GsEvent {
    pub fn new(what: i32, a: i32, b: i32, c: i32, d: i32) -> GsEvent {
        GsEvent {
            what,
            a,
            b,
            c,
            d,
            spine: Spine::default(),
            hit: [0; HI_COUNT as usize],
            hitf: [0.0; HF_COUNT as usize],
            has_hit: false,
            context_valid: false,
            context_element: 0,
            context_action: 0,
            context_move: 0,
            context_original_element: 0,
            hit_rules: Vec::new(),
            context_attacker_ground: 0,
            context_victim_ground: 0,
            context_attacker_damage: 0,
            context_victim_damage: 0,
            source_enemy: 0,
            reaction: false,
            payload: None,
        }
    }
}

#[derive(Default)]
pub struct EventState {
    pub queue: Vec<GsEvent>,
    pub dropped: i32,
    /// `gs.want_events`: some script defines one of the event hooks
    pub want: bool,
    pub in_event: bool,
    epoch: u32,
    frame: i32,
    sequence: u32,
    /// `gs_spine_post_pending`: FramePost has retired the simulation gate but `gs.frame` has not advanced yet
    pub post_pending: bool,
}

const WANT_HOOKS: [&str; 46] = [
    "on_action_change", "on_hit", "on_hitlag", "on_land", "on_boss_defeated", "on_1p_boss_defeated", "on_enemy_hit", "on_clank",
    "on_ko", "on_stock_lost", "on_jump", "on_air_jump", "on_ledge_grab", "on_grab", "on_throw", "on_taunt", "on_shield_hit",
    "on_perfect_shield", "on_armor", "on_skill", "on_crit", "on_lcancel", "on_lcancel_miss", "on_auto_cancel", "on_lcancel_hit",
    "on_wavedash", "on_waveland", "on_ledge_dash", "on_air_dodge", "on_perfect_shield_skill", "on_tech", "on_tech_miss",
    "on_dash_dance", "on_short_hop", "on_full_hop", "on_fast_fall", "on_shield_drop", "on_jump_cancel_grab",
    "on_jump_cancel_usmash", "on_sdi", "on_combo", "on_combo_end", "on_shock", "on_shock_hit", "on_shock_end", "on_event",
];
const HOOK_NAMES: [&str; 20] = [
    "", "on_action_change", "on_hit", "on_hitlag", "on_land", "on_target_broken", "on_all_targets_broken", "on_boss_defeated",
    "on_enemy_defeated", "on_enemy_removed", "on_enemy_hit", "on_clank", "on_item_collect", "on_item_expire", "on_ko",
    "on_stock_lost", "on_action_signal", "on_armor", "on_skill", "on_crit",
];
const ACTION_HOOKS: [&str; 9] =
    ["", "on_jump", "on_air_jump", "on_ledge_grab", "on_grab", "on_throw", "on_taunt", "on_shield_hit", "on_perfect_shield"];
const MOVES: [&str; 10] = ["unknown", "jab", "dash_attack", "tilt", "smash", "aerial", "grab", "throw", "special", "projectile"];

fn element_tag(e: i32) -> &'static str {
    match e {
        0 => "normal",
        1 => "fire",
        2 => "electric",
        5 => "ice",
        13 => "darkness",
        _ => "unknown",
    }
}

impl State {
    /// `gs_spine_event_capture`: one epoch/frame/sequence clock for every event row
    pub fn spine_capture(&mut self, e: &mut GsEvent) {
        let epoch = self.game.entity_epoch() as u32;
        let frame = self.frame + (self.rw.in_frame || self.ev.post_pending) as i32;
        let ev = &mut self.ev;
        e.spine = Spine::default();
        if ev.epoch != epoch || ev.frame != frame {
            ev.epoch = epoch;
            ev.frame = frame;
            ev.sequence = 0;
        }
        ev.sequence += 1;
        e.spine.frame = frame;
        e.spine.epoch = epoch;
        e.spine.sequence = ev.sequence;
        let w = e.what;
        e.spine.phase = if w == 17 {
            3
        } else if w == EV_HIT || w == 11 || w == 19 {
            2
        } else if (5..=9).contains(&w) || (12..=15).contains(&w) {
            4
        } else {
            1
        };
        let g = &*self.game;
        match w {
            EV_HIT => e.spine.target = actor_text(g, e.b, e.c & HIT_VICTIM_SUB != 0),
            EV_ACTION | EV_LAND | 16 => e.spine.actor = actor_text(g, e.a, e.d != 0),
            EV_HITLAG | 15 => e.spine.actor = actor_text(g, e.a, e.c != 0),
            14 => {
                e.spine.actor = actor_text(g, e.a, false);
                e.spine.target = actor_text(g, e.b, false);
            }
            17 | 18 => {
                let entity = e.a - 1;
                e.spine.actor = actor_text(g, entity.rem_euclid(6), entity / 6 != 0);
            }
            19 => {
                let (entity, v) = (e.a - 1, e.b - 1);
                if entity >= 0 {
                    e.spine.actor = actor_text(g, entity % 6, entity / 6 != 0);
                }
                if v >= 0 {
                    e.spine.target = actor_text(g, v % 6, v / 6 != 0);
                }
            }
            _ => {}
        }
    }

    /// Queue a fully built event row (or count it dropped). Returns its index.
    pub fn push_event(&mut self, mut e: GsEvent) -> Option<usize> {
        if self.ev.queue.len() >= MAX_EVENTS {
            self.ev.dropped += 1;
            return None;
        }
        self.spine_capture(&mut e);
        self.ev.queue.push(e);
        Some(self.ev.queue.len() - 1)
    }

    /// `gw_Script_GameEvent`
    pub fn game_event(&mut self, what: i32, a: i32, b: i32, c: i32, d: i32) {
        self.game.mods_game_event(what, a, b, c, d); // online Envoy: the native triggered rules, un-gated
        if !self.ev.want || self.game.resimulating() {
            return;
        }
        if self.ev.queue.len() >= MAX_EVENTS {
            self.ev.dropped += 1;
            return;
        }
        let mut e = GsEvent::new(what, a, b, c, d);
        if what == EV_HIT && (0..6).contains(&a) && (c & HIT_INDEX_MASK) < 5 && c & (HIT_ATTACKER_SUB | HIT_BY_ITEM) == 0 {
            let idx = c & HIT_INDEX_MASK;
            for k in 0..HI_COUNT {
                e.hit[k as usize] = self.game.hit_i(a, idx, k);
            }
            for k in 0..HF_COUNT {
                e.hitf[k as usize] = self.game.hit_f(a, idx, k);
            }
            e.has_hit = true;
        }
        self.spine_capture(&mut e);
        let cause = e.spine.sequence;
        self.ev.queue.push(e);
        if what == EV_ACTION && b != c {
            let category = self.game.action_category(c);
            if category != 0 {
                let next = self.ev.queue.len();
                self.game_event(16, a, c, category, d);
                if self.ev.queue.len() == next + 1 {
                    self.ev.queue[next].spine.cause = cause;
                }
            }
        }
    }

    /// `gw_Script_HitContext`: scalar payload captured by the collision site for the HIT row just queued
    #[allow(clippy::too_many_arguments)]
    pub fn hit_context(
        &mut self,
        attacker: i32,
        victim: i32,
        element: i32,
        action: i32,
        mv: i32,
        attacker_ground: i32,
        victim_ground: i32,
        attacker_damage: i32,
        victim_damage: i32,
    ) {
        if !self.ev.want || self.ev.queue.is_empty() || self.ev.queue.len() >= MAX_EVENTS || self.game.resimulating() {
            return;
        }
        let e = self.ev.queue.last_mut().unwrap();
        if e.what != EV_HIT || e.b != victim {
            return;
        }
        e.a = attacker;
        e.spine.actor.clear(); // throw credit carries no sub identity: do not guess a live sibling
        e.context_valid = true;
        e.context_element = element;
        e.context_action = action;
        e.context_move = mv;
        e.context_original_element = element;
        e.context_attacker_ground = attacker_ground;
        e.context_victim_ground = victim_ground;
        e.context_attacker_damage = attacker_damage;
        e.context_victim_damage = victim_damage;
    }
    /// `gw_Script_HitOriginalElement`
    pub fn hit_original_element(&mut self, original: i32) {
        if !self.ev.want || self.ev.queue.is_empty() || self.ev.queue.len() >= MAX_EVENTS || self.game.resimulating() {
            return;
        }
        let e = self.ev.queue.last_mut().unwrap();
        if e.what == EV_HIT && e.context_valid {
            e.context_original_element = original;
        }
    }
    /// `gw_Script_HitRuleContext`
    pub fn hit_rule_context(&mut self, victim: i32, id: i32, mv: i32, grounded: i32, owner: i32) {
        if !self.ev.want || self.ev.queue.is_empty() || self.ev.queue.len() >= MAX_EVENTS || self.game.resimulating() {
            return;
        }
        let e = self.ev.queue.last_mut().unwrap();
        if e.what != EV_HIT || e.b != victim {
            return;
        }
        e.context_move = mv;
        e.context_attacker_ground = grounded;
        if e.hit_rules.contains(&(id, owner)) {
            return;
        }
        if e.hit_rules.len() < HIT_RULE_CONTEXT_MAX {
            e.hit_rules.push((id, owner));
        }
    }
    /// `gw_Script_EnemyGameEvent`
    pub fn enemy_game_event(&mut self, handle: i32, attacker: i32, victim: i32, flags: i32, damage: i32, reaction: i32) {
        let before = self.ev.queue.len();
        self.game_event(EV_HIT, attacker, victim, flags, damage);
        if self.ev.queue.len() == before + 1 {
            self.ev.queue[before].source_enemy = handle;
            self.ev.queue[before].reaction = reaction != 0;
        }
    }
    /// `gw_Script_TargetBroken`
    pub fn target_broken(&mut self, handle: i32, remaining: i32) {
        if self.game.resimulating() {
            return;
        }
        if self.ev.queue.len() >= MAX_EVENTS - (remaining == 0) as usize {
            self.ev.dropped += 1;
            return;
        }
        self.push_event(GsEvent::new(5, handle, remaining, 0, 0));
        if remaining == 0 {
            self.push_event(GsEvent::new(6, 0, 0, 0, 0));
        }
    }
    /// `gw_Script_EnemyDefeated`
    pub fn enemy_defeated(&mut self, which: i32, handle: i32) {
        if self.game.resimulating() || !(0..ENEMY_NAMES.len() as i32).contains(&which) {
            return;
        }
        if self.push_event(GsEvent::new(8, which, handle, 0, 0)).is_some() {
            self.log(&format!("script enemy: queued defeat kind={} handle={handle}", ENEMY_NAMES[which as usize]));
        }
    }
    /// `gw_Script_EnemyRemoved`: reason 1 explicit, 2 generic
    pub fn enemy_removed(&mut self, which: i32, handle: i32, reason: i32) {
        if self.game.resimulating() || !(0..ENEMY_NAMES.len() as i32).contains(&which) {
            return;
        }
        if self.push_event(GsEvent::new(9, which, handle, reason, 0)).is_some() {
            self.log(&format!(
                "script enemy: queued removed kind={} handle={handle} reason={}",
                ENEMY_NAMES[which as usize],
                if reason == 1 { "explicit_remove" } else { "item_destroyed" }
            ));
        }
    }
    /// `gw_Script_EnemyHit`: event 10 (event 9 is on_enemy_removed)
    pub fn enemy_hit(&mut self, handle: i32, from: i32, damage: i32) {
        if !self.ev.want || self.game.resimulating() || !(0..6).contains(&from) || damage <= 0 {
            return;
        }
        self.push_event(GsEvent::new(10, handle, from + 1, damage, 0));
    }
}

fn payload_table(lua: &mlua::Lua, p: &Option<Vec<(String, Pv)>>) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (k, v) in p.iter().flatten() {
        match v {
            Pv::Int(n) => t.int(k, *n)?,
            Pv::Num(n) => t.num(k, *n)?,
            Pv::Bool(b) => t.boolean(k, *b)?,
            Pv::Str(s) => t.string(k, s)?,
        }
    }
    Ok(t)
}

/// `gs_spine_event_push`: the common `on_event` envelope
pub fn spine_push(lua: &mlua::Lua, e: &GsEvent, hook: &str) -> mlua::Result<Table> {
    const PHASES: [&str; 7] = ["unknown", "action", "collision", "reaction", "lifecycle", "world", "run"];
    let id = format!("v1:{}:{}:{}", e.spine.epoch, e.spine.frame, e.spine.sequence);
    let t = lua.create_table()?;
    t.int("version", 1)?;
    t.string("kind", hook)?;
    t.string("id", &id)?;
    t.int("frame", e.spine.frame as i64)?;
    t.int("sequence", e.spine.sequence as i64)?;
    let ph = e.spine.phase;
    t.string("phase", PHASES[if (1..=6).contains(&ph) { ph as usize } else { 0 }])?;
    t.string("delivery", "post_frame")?;
    if e.spine.cause != 0 {
        t.string("cause", &format!("v1:{}:{}:{}", e.spine.epoch, e.spine.frame, e.spine.cause))?;
    }
    if !e.spine.actor.is_empty() {
        t.string("actor_ref", &e.spine.actor)?;
    }
    if !e.spine.target.is_empty() {
        t.string("target_ref", &e.spine.target)?;
    }
    let l = lua.create_table()?;
    l.int("code", e.what as i64)?;
    l.int("a", e.a as i64)?;
    l.int("b", e.b as i64)?;
    l.int("c", e.c as i64)?;
    l.int("d", e.d as i64)?;
    t.set("legacy", l)?;
    Ok(t)
}

fn bitsf(i: i32) -> f64 {
    f32::from_bits(i as u32) as f64
}

impl Engine {
    /// `gs_update_want_events` (the hook cache): some script defines one of the event hooks
    pub fn update_want_events(&self) {
        let n = self.st.borrow().scripts.len();
        let want = (0..n).any(|i| WANT_HOOKS.iter().any(|h| self.get_hook(i, h).is_some()));
        self.st.borrow_mut().ev.want = want;
        self.zones_arm();
    }

    /// The Lua arguments of one event's specific hook (the switch of `gs_dispatch_events`)
    fn event_args(&self, e: &GsEvent) -> mlua::Result<MultiValue> {
        let lua = &self.lua;
        let int = |n: i32| Value::Integer(n as i64);
        let mut v: Vec<Value> = Vec::new();
        match e.what {
            17 | 18 | 19 | EV_CLANK | EV_ITEM_COLLECT | EV_ITEM_EXPIRE => v.push(Value::Table(payload_table(lua, &e.payload)?)),
            16 => {
                v.push(int(e.a + 1));
                v.push(int(e.b));
                v.push(Value::Boolean(e.d != 0));
            }
            14 => {
                v.push(if e.a >= 0 { int(e.a + 1) } else { Value::Nil });
                v.push(int(e.b + 1));
            }
            15 => {
                v.push(int(e.a + 1));
                v.push(int(e.b));
            }
            10 => {
                let t = lua.create_table()?;
                t.int("handle", e.a as i64)?;
                t.int("from", e.b as i64)?;
                t.int("damage", e.c as i64)?;
                t.boolean("reaction", false)?;
                v.push(Value::Table(t));
            }
            5 => {
                v.push(int(e.a));
                v.push(int(e.b));
            }
            6 => {}
            EV_BOSS_DEFEATED => {
                let other;
                let kind = match e.a {
                    0x1B => "master_hand",
                    0x1C => "crazy_hand",
                    n => {
                        other = format!("fighter_{n}");
                        other.as_str()
                    }
                };
                let t = lua.create_table()?;
                t.string("kind", kind)?;
                t.int("port", (e.b + 1) as i64)?;
                t.num("x", bitsf(e.c))?;
                t.num("y", bitsf(e.d))?;
                v.push(Value::Table(t));
            }
            8 | 9 => {
                let t = lua.create_table()?;
                t.string("kind", ENEMY_NAMES[(e.a as usize).min(ENEMY_NAMES.len() - 1)])?;
                t.int("handle", e.b as i64)?;
                t.string("reason", if e.what == 8 { "defeated" } else if e.c == 1 { "explicit_remove" } else { "item_destroyed" })?;
                v.push(Value::Table(t));
            }
            EV_ACTION => {
                v.push(int(e.a + 1));
                v.push(int(e.b));
                v.push(int(e.c));
                v.push(Value::Boolean(e.d != 0));
            }
            EV_HIT => {
                v.push(if e.a >= 0 { int(e.a + 1) } else { Value::Nil });
                v.push(int(e.b + 1));
                let t = lua.create_table()?;
                t.num("dealt", bitsf(e.d))?;
                if e.c & HIT_INDEX_MASK != HIT_INDEX_MASK {
                    t.int("hitbox", (e.c & HIT_INDEX_MASK) as i64)?;
                }
                t.boolean("context_valid", e.context_valid)?;
                let ids = lua.create_table()?;
                let owners = lua.create_table()?;
                for (k, (id, owner)) in e.hit_rules.iter().enumerate() {
                    ids.raw_set(k as i64 + 1, *id as i64)?;
                    owners.raw_set(k as i64 + 1, *owner as i64)?;
                }
                t.set("hit_rule_ids", ids)?;
                t.set("hit_rule_owners", owners)?;
                if e.context_valid {
                    t.string("move_tag", MOVES[if (0..10).contains(&e.context_move) { e.context_move as usize } else { 0 }])?;
                    t.string("element_tag", element_tag(e.context_element))?;
                    t.int("element", e.context_element as i64)?;
                    t.string("original_element_tag", element_tag(e.context_original_element))?;
                    t.int("original_element", e.context_original_element as i64)?;
                    t.boolean("element_converted", e.context_original_element != e.context_element)?;
                    t.int("attacker_action", e.context_action as i64)?;
                    if e.context_attacker_ground >= 0 {
                        t.boolean("attacker_grounded", e.context_attacker_ground != 0)?;
                    }
                    t.boolean("victim_grounded", e.context_victim_ground != 0)?;
                    t.num("attacker_damage", bitsf(e.context_attacker_damage))?;
                    t.num("victim_damage", bitsf(e.context_victim_damage))?;
                }
                t.boolean("item", e.c & HIT_BY_ITEM != 0)?;
                if e.source_enemy != 0 {
                    t.int("source_enemy", e.source_enemy as i64)?;
                }
                t.boolean("reaction", e.reaction)?;
                t.boolean("attacker_sub", e.c & HIT_ATTACKER_SUB != 0)?;
                t.boolean("victim_sub", e.c & HIT_VICTIM_SUB != 0)?;
                if e.has_hit {
                    push_hit_fields(&t, &e.hit, &e.hitf)?;
                }
                v.push(Value::Table(t));
            }
            EV_HITLAG => {
                v.push(int(e.a + 1));
                v.push(Value::Boolean(e.b != 0));
                v.push(Value::Boolean(e.c != 0));
            }
            _ => {
                // EV_LAND: (port, motion, sub)
                v.push(int(e.a + 1));
                v.push(int(e.b));
                v.push(Value::Boolean(e.c != 0));
            }
        }
        Ok(MultiValue::from_iter(v))
    }

    /// `gs_dispatch_events`
    pub fn dispatch_events(&self) {
        let batch: Vec<GsEvent> = {
            let mut s = self.st.borrow_mut();
            if s.ev.queue.is_empty() {
                return;
            }
            // hooks may enqueue terminal events: they stay behind this batch
            if s.ev.dropped > 0 {
                let d = s.ev.dropped;
                s.log(&format!("script: {d} engine events dropped (queue full)"));
                s.ev.dropped = 0;
            }
            s.ev.in_event = true;
            std::mem::take(&mut s.ev.queue)
        };
        let n_scripts = self.st.borrow().scripts.len();
        for e in &batch {
            if !(1..=19).contains(&e.what) {
                continue;
            }
            let mut hook = HOOK_NAMES[e.what as usize];
            if e.what == 16 {
                if !(1..=8).contains(&e.c) {
                    continue;
                }
                hook = ACTION_HOOKS[e.c as usize];
            }
            for i in 0..n_scripts {
                let (may_run, gameplay) = {
                    let s = self.st.borrow();
                    (!s.game.resimulating(), s.scripts.get(i).is_some_and(|x| x.gameplay))
                };
                if !may_run || (e.what >= 5 && e.what < EV_ITEM_COLLECT && e.what != EV_CLANK && !gameplay) {
                    continue;
                }
                if let Some(f) = self.get_hook(i, "on_event") {
                    if let Ok(t) = spine_push(&self.lua, e, hook) {
                        self.pcall(i, &f, MultiValue::from_iter([Value::Table(t)]), "on_event");
                    }
                }
                if self.st.borrow().game.resimulating() {
                    continue;
                }
                let Some(f) = self.get_hook(i, hook) else { continue };
                let Ok(args) = self.event_args(e) else { continue };
                self.pcall(i, &f, args, hook);
            }
        }
        // events queued by the hooks stay; anything queued before is gone
        let mut s = self.st.borrow_mut();
        s.ev.in_event = false;
    }

    // ---- the game's entry points (`gw_Script_*`)
    pub fn game_event(&self, what: i32, a: i32, b: i32, c: i32, d: i32) {
        self.st.borrow_mut().game_event(what, a, b, c, d);
    }
}

#[allow(dead_code)]
fn _unused() {
    let _ = element_name(0);
}
