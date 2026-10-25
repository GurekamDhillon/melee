//! Staging deadlines (`gw_script_deadline.inc`): gd.deadline, gd.deadline_done and the `on_deadline` hook. Deadlines count
//! completed live logic frames, not PADReads, wall time, rewind or paused draws. Namespace = script slot + its stage-owner generation.
use super::{lua_err, reg};
use crate::args::Args;
use crate::engine::{Engine, Shared};
use mlua::{MultiValue, Table, Value};

type LuaResult<T> = mlua::Result<T>;

pub const GS_DEADLINES: usize = 128;

#[derive(Clone, Default)]
pub struct Deadline {
    used: bool,
    owner: usize,
    generation: i32,
    reported: bool,
    waited: u32,
    limit: u32,
    serial: u32,
    name: String,
}
#[derive(Default)]
pub struct Deadlines {
    t: Vec<Deadline>,
    serial: u32,
    active: bool,
    last: String,
}

impl Deadlines {
    fn publish(&mut self) -> (i32, i32, String) {
        let (mut pending, mut expired) = (0, 0);
        for d in self.t.iter().filter(|d| d.used) {
            if d.reported {
                expired += 1;
            } else {
                pending += 1;
            }
        }
        self.active = pending + expired > 0;
        (pending, expired, self.last.clone())
    }
    /// `gs_deadline_release`
    pub fn release(&mut self, owner: usize) -> (i32, i32, String) {
        for d in self.t.iter_mut().filter(|d| d.used && d.owner == owner) {
            d.used = false;
        }
        self.publish()
    }
}

fn deadline(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let name = a.check_string(1)?;
    let n = a.check_integer(2)?;
    let mut s = st.borrow_mut();
    let cur = s.cur.unwrap_or(0);
    let generation = s.scripts.get(cur).map_or(0, |x| x.stage_owner);
    if name.is_empty() || name.len() >= 128 || n < 1 || n > i32::MAX as i64 {
        return Err(lua_err("gd.deadline: name 1..127 bytes, frames 1..2147483647"));
    }
    if s.deadlines.t.is_empty() {
        s.deadlines.t = vec![Deadline::default(); GS_DEADLINES];
    }
    let mut slot = None;
    for (i, d) in s.deadlines.t.iter().enumerate() {
        if d.used && d.owner == cur && d.generation == generation && d.name == name {
            // repeated calls in staging must not silently extend its deadline
            return Ok(false);
        }
        if !d.used && slot.is_none() {
            slot = Some(i);
        }
    }
    let Some(slot) = slot else { return Err(lua_err("gd.deadline: table full (128 deadlines)")) };
    s.deadlines.serial += 1;
    let serial = s.deadlines.serial;
    s.deadlines.t[slot] = Deadline { used: true, owner: cur, generation, reported: false, waited: 0, limit: n as u32, serial, name };
    let (p, e, l) = s.deadlines.publish();
    s.host.borrow_mut().hang_script_state(p, e, &l);
    Ok(true)
}
fn deadline_done(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let name = a.check_string(1)?;
    let mut s = st.borrow_mut();
    let cur = s.cur.unwrap_or(0);
    let generation = s.scripts.get(cur).map_or(0, |x| x.stage_owner);
    let found = s.deadlines.t.iter().position(|d| d.used && d.owner == cur && d.generation == generation && d.name == name);
    if let Some(i) = found {
        s.deadlines.t[i].used = false;
        let (p, e, l) = s.deadlines.publish();
        s.host.borrow_mut().hang_script_state(p, e, &l);
        return Ok(true);
    }
    Ok(false)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "deadline", deadline)?;
    reg(e, gd, "deadline_done", deadline_done)
}

impl Engine {
    /// `gs_deadline_frame`: one completed live logic frame
    pub fn deadline_frame(&self) {
        let cutoff = {
            let s = self.st.borrow();
            if !s.deadlines.active {
                return;
            }
            s.deadlines.serial
        };
        for i in 0..GS_DEADLINES {
            let fire = {
                let mut s = self.st.borrow_mut();
                let Some(d) = s.deadlines.t.get(i).cloned() else { break };
                if !d.used || d.reported || d.serial > cutoff {
                    continue;
                }
                let alive = s.scripts.get(d.owner).is_some_and(|x| x.used && x.stage_owner == d.generation);
                if !alive {
                    s.deadlines.t[i].used = false;
                    continue;
                }
                s.deadlines.t[i].waited += 1;
                let waited = s.deadlines.t[i].waited;
                if waited >= d.limit {
                    s.deadlines.t[i].reported = true;
                    s.deadlines.last = d.name.clone();
                    let id = s.script_id(Some(d.owner));
                    s.log(&format!("script [{id}] deadline expired name={} frames_waited={waited}", d.name));
                    // publish before invoking user code: the callback may stall, cancel or re-arm
                    let (p, e, l) = s.deadlines.publish();
                    s.host.borrow_mut().hang_script_state(p, e, &l);
                    Some((d.owner, d.name, waited))
                } else {
                    None
                }
            };
            if let Some((owner, name, waited)) = fire {
                if let Some(f) = self.get_hook(owner, "on_deadline") {
                    let args = MultiValue::from_iter([
                        Value::String(self.lua.create_string(&name).unwrap()),
                        Value::Integer(waited as i64),
                    ]);
                    self.pcall(owner, &f, args, "on_deadline");
                }
            }
        }
        let mut s = self.st.borrow_mut();
        let (p, e, l) = s.deadlines.publish();
        s.host.borrow_mut().hang_script_state(p, e, &l);
    }
}
