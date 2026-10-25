//! Fighter modifiers (`gw_script_fighter_mod.inc`: fighter_mod) and the timed logic freeze (`gw_script_presentation.inc`:
//! hitstop, hitstop_cancel). The modifier record lives in the game half (`ScriptGame::fighter_mod*`); hitstop is a host
//! presentation deadline, never simulation state: frames mean 1/60 second and it does not touch manual pause/step state.
use super::arena::cur_owner;
use super::camera::{bitsf, fbits};
use super::caps::{key_str, neg1, one, pairs};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, GS_MAX_SCRIPTS};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub const MOD_KEYS: [&str; 11] = [
    "damage_dealt", "damage_taken", "run_speed", "air_speed", "shield_max", "jump_height", "air_jump_height", "knockback_taken",
    "fall_speed", "weight", "shield_regen",
];

fn fighter_mod(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Value> {
    let slot = slot_arg(&a, 1)?;
    if a.len() == 1 {
        let s = st.borrow();
        if s.game.fighter_mod_owner(slot as i32) == 0 {
            return Ok(Value::Nil);
        }
        let t = lua.create_table()?;
        for (i, k) in MOD_KEYS.iter().enumerate() {
            t.num(k, bitsf(s.game.fighter_mod_read(slot as i32, i as i32)) as f64)?;
        }
        return Ok(Value::Table(t));
    }
    let enabled = !a.none_or_nil(2);
    let mut s = st.borrow_mut();
    s.require_offline("fighter_mod")?;
    if !s.match_active {
        return Err(lua_err("fighter_mod requires an active match"));
    }
    let owner = s.game.fighter_mod_owner(slot as i32);
    let me = cur_owner(&s);
    if owner != 0 && owner != me {
        return Err(lua_err("fighter modifiers owned by another script"));
    }
    let mut v = [1.0f32; 11];
    if enabled {
        let t = a.check_table(2)?;
        for (k, val) in pairs(&t)? {
            let n = neg1(one("fighter_mod", &val).check_number(1))?;
            let Some(i) = key_str(&k).and_then(|k| MOD_KEYS.iter().position(|m| *m == k)) else {
                return Err(lua_err("unknown fighter modifier"));
            };
            if !n.is_finite() {
                return Err(lua_err("fighter modifier must be finite"));
            }
            v[i] = (if n < 0.1 { 0.1 } else if n > 4.0 { 4.0 } else { n }) as f32;
        }
    }
    s.rw_branch();
    let b: Vec<i32> = v.iter().map(|f| fbits(*f)).collect();
    let ok = s.game.fighter_mod(slot as i32, me, enabled, &b[..8]) != 0;
    if enabled && ok {
        s.game.fighter_mod_extra(slot as i32, me, b[8], b[9], b[10]);
    }
    Ok(Value::Boolean(ok))
}

/// `GwPresentationTimer`
#[derive(Clone, Copy, Default)]
pub struct Timer {
    pub owner: i32,
    pub used: bool,
    pub start: f64,
    pub end: f64,
}
impl Timer {
    pub fn start(&mut self, owner: i32, now: f64, frames: i32) {
        *self = Timer { used: true, owner, start: now, end: now + frames as f64 * (1000.0 / 60.0) };
    }
    pub fn live(&self, now: f64) -> bool {
        self.used && now < self.end
    }
    pub fn progress(&self, now: f64) -> f64 {
        let p = if self.end > self.start { (now - self.start) / (self.end - self.start) } else { 1.0 };
        p.clamp(0.0, 1.0)
    }
    pub fn cancel(&mut self, owner: i32) -> bool {
        if !self.used || self.owner != owner {
            return false;
        }
        self.used = false;
        true
    }
}
pub struct Hitstop {
    pub t: Vec<Timer>,
    pub active: i32, // bit 1 freeze, bit 2 owned post deadlines
}
impl Default for Hitstop {
    fn default() -> Self {
        Hitstop { t: vec![Timer::default(); GS_MAX_SCRIPTS + 1], active: 0 }
    }
}

fn presentation_owner(s: &crate::engine::State) -> i32 {
    s.cur.and_then(|c| s.scripts.get(c)).map_or(0, |x| x.stage_owner)
}
fn hitstop(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let n = a.check_integer(1)?;
    let mut s = st.borrow_mut();
    s.require_offline("hitstop")?;
    if !(1..=36000).contains(&n) {
        return Err(lua_err("gd.hitstop: frames must be 1..36000"));
    }
    if !s.match_active {
        return Ok(false);
    }
    let owner = presentation_owner(&s);
    let now = s.host.borrow().now_ms();
    let mut slot = None;
    for (i, t) in s.hitstop.t.iter().enumerate() {
        if t.used && t.owner == owner {
            slot = Some(i);
            break;
        }
        if !t.used && slot.is_none() {
            slot = Some(i);
        }
    }
    if let Some(i) = slot {
        s.hitstop.t[i].start(owner, now, n as i32);
        s.hitstop.active |= 1;
        return Ok(true);
    }
    Ok(false)
}
fn hitstop_cancel(_: &mlua::Lua, st: &Shared, _a: Args) -> LuaResult<bool> {
    let mut s = st.borrow_mut();
    s.require_offline("hitstop_cancel")?;
    let owner = presentation_owner(&s);
    let mut changed = false;
    for t in s.hitstop.t.iter_mut() {
        changed |= t.cancel(owner);
    }
    Ok(changed)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "fighter_mod", fighter_mod)?;
    reg(e, gd, "hitstop", hitstop)?;
    reg(e, gd, "hitstop_cancel", hitstop_cancel)?;
    Ok(())
}

impl crate::engine::State {
    /// `gs_presentation_release`: a script unloaded
    pub fn presentation_release(&mut self, owner: i32) {
        for t in self.hitstop.t.iter_mut() {
            t.cancel(owner);
        }
    }
    /// `gs_hitstop_live`: the logic is frozen by a timed hitstop (offline only)
    pub fn hitstop_live(&self) -> bool {
        if self.hitstop.active & 1 == 0 {
            return false;
        }
        let now = self.host.borrow().now_ms();
        if self.host.borrow().session() || self.game.resimulating() {
            return false;
        }
        self.hitstop.t.iter().any(|t| t.live(now))
    }
    /// `gs_presentation_tick`: retire expired timers
    pub fn presentation_tick(&mut self) {
        if self.hitstop.active == 0 {
            return;
        }
        let now = self.host.borrow().now_ms();
        if !self.match_active || self.host.borrow().session() {
            for t in self.hitstop.t.iter_mut() {
                *t = Timer::default();
            }
        }
        let mut active = 0;
        for t in self.hitstop.t.iter_mut() {
            if !t.live(now) {
                t.used = false;
            } else {
                active |= 1;
            }
        }
        self.hitstop.active = active | (self.hitstop.active & 2);
    }
}
