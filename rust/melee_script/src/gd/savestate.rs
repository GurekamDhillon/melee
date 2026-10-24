//! `gd.savestate` / `gd.loadstate`: the four quick slots (gw_script.c `l_savestate`, `l_loadstate`, and the slot
//! part of `gs_apply_pending`). A request is queued and carried out at the next frame boundary.
use super::{lua_err, reg};
use crate::engine::{Engine, State};
use mlua::{MultiValue, Result as LuaResult, Table, Value};

pub const GS_SAVE_SLOTS: usize = 4;

/// `GsSaveSlot`: the script bookkeeping saved beside the game snapshot.
#[derive(Clone, Copy, Default, Debug)]
pub struct SaveSlot {
    pub used: bool,
    pub scene_epoch: i32,
    pub frame: i32,
    pub match_frame: i32,
    pub last_action: [i32; 6],
    pub state_frame: [i32; 6],
}

impl State {
    /// `gs_slot_capture`
    pub fn slot_capture(&mut self, i: usize) {
        self.slot[i] = SaveSlot {
            used: true,
            scene_epoch: self.scene_epoch,
            frame: self.frame,
            match_frame: self.match_frame,
            last_action: self.last_action,
            state_frame: self.state_frame,
        };
    }

    /// `gs_slot_restore`
    pub fn slot_restore(&mut self, s: SaveSlot) {
        self.match_frame = s.match_frame;
        self.last_action = s.last_action;
        self.state_frame = s.state_frame;
    }

    /// `gs_stage_state_refuse`
    fn stage_state_refuse(&mut self, lua: &mlua::Lua, op: &str) -> LuaResult<MultiValue> {
        let why = "stage slots active: savestate/rewind cannot restore the native director and cache";
        self.log(&format!("{op} refused: {why}"));
        Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
    }
}

impl Engine {
    /// `gs_apply_pending` (quick slots): at a frame boundary, save or load what was requested.
    pub fn apply_pending(&self, _at_tick: bool) {
        let mut hooks: Vec<(&'static str, i64)> = Vec::new();
        {
            let mut s = self.st.borrow_mut();
            let session = s.host.borrow().session();
            if (s.pending_save != 0 || s.pending_load != 0) && s.game.stage_slots_active() {
                s.print(
                    crate::engine::GS_RED,
                    "stage slot session active: savestate/rewind is refused (native queue/cache lifetime)",
                );
                s.log("pending savestate/rewind refused: stage slots active; pending requests cleared");
                s.pending_save = 0;
                s.pending_load = 0;
            }
            if (s.pending_save != 0 || s.pending_load != 0) && session {
                s.print(crate::engine::GS_RED, "savestates are off during a netplay/rollback session");
                s.pending_save = 0;
                s.pending_load = 0;
            }
            if s.pending_save != 0 {
                let slot = (s.pending_save - 1) as usize;
                s.pending_save = 0;
                if !s.snap_ready {
                    s.snap_ready = s.game.snap_reserve(GS_SAVE_SLOTS as i32) >= GS_SAVE_SLOTS as i32;
                }
                if s.snap_ready {
                    s.game.snap_save_index(slot as i32, -1 - slot as i32);
                    s.slot_capture(slot);
                    let f = s.frame;
                    s.print(crate::engine::GS_GREEN, &format!("saved state {} (frame {f})", slot + 1));
                    hooks.push(("on_savestate", slot as i64 + 1));
                } else {
                    s.print(crate::engine::GS_RED, "savestates unavailable (snapshot memory could not be allocated)");
                }
            }
            if s.pending_load != 0 {
                let slot = (s.pending_load - 1) as usize;
                s.pending_load = 0;
                let sl = s.slot[slot];
                if sl.used && sl.scene_epoch == s.scene_epoch && s.game.snap_load_index(slot as i32) == 0 {
                    s.slot_restore(sl);
                    s.print(crate::engine::GS_GREEN, &format!("loaded state {}", slot + 1));
                    hooks.push(("on_loadstate", slot as i64 + 1));
                } else {
                    s.print(crate::engine::GS_RED, &format!("could not load state {}", slot + 1));
                }
            }
        }
        for (name, slot) in hooks {
            self.hook_all(name, &[Value::Integer(slot), Value::Integer(0)][..1]);
        }
    }
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    for (name, save) in [("savestate", true), ("loadstate", false)] {
        reg(e, gd, name, move |lua, st, a| {
            let mut s = st.borrow_mut();
            if s.game.onep_state_active() {
                return Err(lua_err("retail 1P state active: snapshot/rewind unavailable"));
            }
            let slot = a.opt_integer(1, 1)?;
            s.require_gameplay(name)?;
            if s.game.stage_slots_active() {
                return s.stage_state_refuse(lua, name);
            }
            if slot < 1 || slot > GS_SAVE_SLOTS as i64 {
                return Err(lua_err(format!("savestate slots are 1-{GS_SAVE_SLOTS}")));
            }
            if save {
                s.pending_save = slot as i32;
            } else {
                let sl = s.slot[slot as usize - 1];
                if !sl.used {
                    return Err(lua_err(format!("savestate slot {slot} is empty")));
                }
                if sl.scene_epoch != s.scene_epoch {
                    return Err(lua_err(format!("savestate slot {slot} was saved in another scene")));
                }
                s.pending_load = slot as i32;
            }
            Ok(MultiValue::new())
        })?;
    }
    Ok(())
}
