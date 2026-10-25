//! Fighter writes and debug movement: `gd.set_damage`, `hit`, `impulse`, `set_stocks`, `cpu_mode(s)`, `fly`,
//! `teleport`, `fly_speed`, `fly_solid`, `hold_hitbox`, `play_sound` (gw_script.c).
use super::camera::fbits;
use super::core::clampi;
use super::{lua_err, reg, slot_arg, TableExt};
use crate::engine::Engine;
use crate::game::*;
use mlua::{Result as LuaResult, Table, Value};

/// `gs_fly_err`
fn fly_err(rc: i32) -> &'static str {
    match rc {
        -1 => "no fighter on that port",
        -2 => "that fighter's state cannot fly (dead, held, respawning)",
        _ => "",
    }
}

/// `gs_cpu_mode_words`: (mode, source) for `ScriptGame_CpuModeInfo`
pub fn cpu_mode_words(info: i32) -> Option<(&'static str, &'static str)> {
    const M: [&str; 9] = ["", "fight", "idle", "idle", "idle", "fight", "fight", "idle", "script"];
    const S: [&str; 9] = ["", "retail", "global", "slot", "script", "script", "slot", "retail", "script"];
    if info <= 0 || info > 8 { None } else { Some((M[info as usize], S[info as usize])) }
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "set_damage", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let p = a.check_number(2)? as i32;
        let mut s = st.borrow_mut();
        s.require_gameplay("set_damage")?;
        s.require_offline("set_damage")?;
        s.rw_branch();
        s.game.set_percent(slot, clampi(p, 0, 999));
        Ok(())
    })?;
    reg(e, gd, "set_stocks", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let n = a.check_integer(2)? as i32;
        let mut s = st.borrow_mut();
        s.require_gameplay("set_stocks")?;
        s.rw_branch();
        s.game.set_stocks(slot, clampi(n, 0, 99));
        Ok(())
    })?;
    reg(e, gd, "impulse", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let t = a.check_table(2)?;
        let mut s = st.borrow_mut();
        s.require_offline("impulse")?;
        let num = |k: &str| -> LuaResult<f32> {
            let v: Value = t.get(k)?;
            if matches!(v, Value::Nil) {
                return Ok(0.0);
            }
            Ok(crate::args::Args::new("impulse", mlua::MultiValue::from_iter([v])).check_number(1)? as f32)
        };
        let (x, y) = (num("x")?, num("y")?);
        if !x.is_finite() || !y.is_finite() || x.abs() > 4.0 || y.abs() > 3.0 {
            return Err(lua_err("impulse: finite x in -4..4, y in -3..3 required"));
        }
        s.rw_branch();
        Ok(s.game.impulse(slot, fbits(x), fbits(y)))
    })?;
    reg(e, gd, "hit", |_, st, a| {
        let t = a.check_table(2)?;
        let mut s = st.borrow_mut();
        s.require_offline("hit")?;
        if a.is_table(1) {
            // an {enemy = handle} target needs the stage layer (gs_require_stage), not ported yet
            return Err(lua_err("gd.hit enemy requires a gameplay mod script"));
        }
        let slot = slot_arg(&a, 1)? as i32;
        let geti = |k: &str| -> LuaResult<i64> {
            let v: Value = t.get(k)?;
            crate::args::Args::new("hit", mlua::MultiValue::from_iter([v])).check_integer(1)
        };
        let (damage, angle, kbg, bkb) = (geti("damage")?, geti("angle")?, geti("kbg")?, geti("bkb")?);
        let mut from = -1;
        let fv: Value = t.get("from")?;
        if !matches!(fv, Value::Nil) {
            let view = crate::args::Args::new("hit", mlua::MultiValue::from_iter([fv]));
            from = super::slot_arg(&view, 1)? as i32;
        }
        if !(0..=500).contains(&damage)
            || !(0..=361).contains(&angle)
            || !(0..=1000).contains(&kbg)
            || !(0..=1000).contains(&bkb)
        {
            return Err(lua_err("gd.hit: damage 0-500, angle 0-361, kbg/bkb 0-1000 required"));
        }
        s.rw_branch();
        Ok(s.game.hit(slot, from, damage as i32, angle as i32, kbg as i32, bkb as i32))
    })?;
    reg(e, gd, "cpu_mode", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let Value::String(m) = a.get(2) else {
            return Err(a.err(2, &format!("string expected, got {}", if a.is_none(2) { "no value" } else { "other" })));
        };
        let fight = match &*m.as_bytes() {
            b"stand" => 0,
            b"fight" => 1,
            b"default" => 2, // back to the launch setting
            b"script" => 3,  // gd.cpu_pad / gd.cpu_script drive it
            _ => return Err(lua_err("cpu_mode: expected stand, fight, script or default")),
        };
        let mut s = st.borrow_mut();
        s.require_offline("cpu_mode")?;
        s.rw_branch();
        let ok = if fight == 3 { s.game.cpu_script_mode(slot) } else { s.game.cpu_mode(slot, fight) };
        if ok {
            s.cpu_log_slot(slot);
        }
        Ok(ok)
    })?;
    reg(e, gd, "cpu_modes", |lua, st, _| {
        let s = st.borrow();
        let t = lua.create_table()?;
        for slot in 0..6 {
            let Some((w, src)) = cpu_mode_words(s.game.cpu_mode_info(slot)) else { continue };
            let r = lua.create_table()?;
            r.string("mode", w)?;
            r.string("source", src)?;
            t.raw_set(slot + 1, r)?;
        }
        Ok(t)
    })?;

    // ---- debug movement
    reg(e, gd, "fly", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        if a.none_or_nil(2) {
            return Ok(st.borrow().game.geno_fly_get(slot) > 0);
        }
        let mode = if let Value::Boolean(b) = a.get(2) {
            *b as i32
        } else {
            let m = a.check_string(2)?;
            match m.to_ascii_lowercase().as_str() {
                "on" => 1,
                "off" => 0,
                "place" => 2,
                "toggle" => -1,
                _ => {
                    return Err(lua_err(format!(
                        "gd.fly: mode is true / false / \"on\" / \"off\" / \"place\" / \"toggle\" (got \"{m}\")"
                    )))
                }
            }
        };
        let mut s = st.borrow_mut();
        s.require_offline("fly")?;
        let rc = s.fly_set(slot, mode);
        if rc < 0 {
            return Err(lua_err(format!("gd.fly: {}", fly_err(rc))));
        }
        Ok(s.game.geno_fly_get(slot) > 0)
    })?;
    reg(e, gd, "teleport", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let x = a.check_number(2)? as f32;
        let y = a.check_number(3)? as f32;
        let mut s = st.borrow_mut();
        s.require_offline("teleport")?;
        s.rw_branch();
        let rc = s.game.geno_fly_teleport(slot, fbits(x), fbits(y));
        if rc < 0 {
            return Err(lua_err(format!("gd.teleport: {}", fly_err(rc))));
        }
        Ok(())
    })?;
    reg(e, gd, "fly_speed", |_, st, a| {
        let mut s = st.borrow_mut();
        if !a.none_or_nil(1) {
            let v = a.check_number(1)? as f32;
            s.require_offline("fly_speed")?;
            s.rw_branch();
            s.game.geno_fly_set_speed(fbits(v));
        }
        Ok(s.game.geno_fly_speed() as f64)
    })?;
    reg(e, gd, "fly_solid", |_, st, a| {
        let mut s = st.borrow_mut();
        if !a.none_or_nil(1) {
            s.require_offline("fly_solid")?;
            s.rw_branch();
            s.game.geno_fly_set_solid(a.truthy(1));
        }
        Ok(s.game.geno_fly_solid())
    })?;
    reg(e, gd, "hold_hitbox", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        if a.none_or_nil(2) {
            let s = st.borrow();
            return Ok(mlua::MultiValue::from_iter([
                Value::Boolean(s.game.geno_fly_holding(slot)),
                Value::Integer(s.game.geno_fly_hold_hits(slot) as i64),
            ]));
        }
        let mut s = st.borrow_mut();
        s.require_offline("hold_hitbox")?;
        let (mut action, mut frame) = (0, -1);
        if a.is_table(3) {
            let t = a.check_table(3)?;
            action = match t.get::<Value>("action")? {
                Value::Integer(n) => n as i32,
                Value::Number(n) => n as i32,
                Value::String(n) => {
                    let n = n.to_string_lossy().to_string();
                    match ["nair", "fair", "bair", "uair", "dair", "jab"].iter().position(|x| n.eq_ignore_ascii_case(x)) {
                        Some(i) => i as i32,
                        None => return Err(lua_err(format!("gd.hold_hitbox: unknown action \"{n}\""))),
                    }
                }
                _ => 0,
            };
            if let Value::Integer(f) = t.get::<Value>("frame")? {
                frame = f as i32;
            } else if let Value::Number(f) = t.get::<Value>("frame")? {
                frame = f as i32;
            }
        }
        s.rw_branch();
        let on = a.truthy(2);
        let rc = s.game.geno_fly_hold_hitbox(slot, on, action, frame);
        if rc == 0 {
            s.log(&format!("fly: P{} hold hitbox {}", slot + 1, if on { "on" } else { "off" }));
        }
        if rc == -3 {
            return Err(lua_err("gd.hold_hitbox: the fighter must be flying (gd.fly)"));
        }
        if rc < 0 {
            return Err(lua_err(format!("gd.hold_hitbox: {}", fly_err(rc))));
        }
        Ok(mlua::MultiValue::from_iter([Value::Boolean(s.game.geno_fly_holding(slot))]))
    })?;
    reg(e, gd, "play_sound", |_, st, a| {
        let id = a.check_integer(1)?;
        let (mut volume, mut pitch) = (127, 0);
        if !a.none_or_nil(2) {
            let t = a.check_table(2)?;
            for pair in t.pairs::<Value, Value>() {
                let (k, v) = pair?;
                let Value::String(k) = k else { return Err(lua_err("gd.play_sound: unknown option")) };
                let key = k.to_string_lossy().to_string();
                if key != "volume" && key != "pitch" {
                    return Err(lua_err(format!("gd.play_sound: unknown option {key}")));
                }
                let Value::Integer(n) = v else {
                    return Err(lua_err(format!("gd.play_sound: {key} must be an integer")));
                };
                if key == "volume" {
                    if !(0..=127).contains(&n) {
                        return Err(lua_err("gd.play_sound: volume must be 0..127"));
                    }
                    volume = n as i32;
                } else {
                    if !(-1200..=1200).contains(&n) {
                        return Err(lua_err("gd.play_sound: pitch must be -1200..1200 cents"));
                    }
                    pitch = n as i32;
                }
            }
        }
        let mut s = st.borrow_mut();
        if s.host.borrow().session() {
            return Err(lua_err("gd.play_sound is offline-only (refused during a netplay/rollback session)"));
        }
        if id <= 0 || id > 0x7FFF_FFFF {
            return Err(lua_err("gd.play_sound: sound id out of range"));
        }
        Ok(s.game.play_sound_options(id as i32, volume, pitch) as i64)
    })?;
    Ok(())
}
