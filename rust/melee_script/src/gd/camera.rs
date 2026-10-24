//! The script camera: `gd.camera_get/detach/attach/set/move/path/follow/shake/bounds` (gw_script.c "camera").
//! The game camera is reached through [`crate::game::ScriptGame`] `camera_*`; floats cross as bit patterns, as in C.
use super::{lua_err, reg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared, State};
use crate::game::lab::*;
use mlua::{Result as LuaResult, Table, Value};

pub fn fbits(f: f32) -> i32 {
    f.to_bits() as i32
}
pub fn bitsf(b: i32) -> f32 {
    f32::from_bits(b as u32)
}

/// `gs_camera_claim`
fn claim(s: &mut State, fname: &str) -> LuaResult<()> {
    s.require_offline(fname)?;
    let cur = s.cur.map_or(0, |c| c as i32 + 1);
    if s.camera_owner != 0 && s.camera_owner != cur {
        return Err(lua_err(format!("gd.{fname}: camera is owned by another script")));
    }
    s.rw_branch();
    if s.game.camera_script_state() == 2 {
        s.game.camera_script_reset();
    }
    if !s.game.camera_script_detach() {
        return Err(lua_err(format!("gd.{fname}: no game camera in this scene")));
    }
    if s.camera_owner == 0 {
        s.camera_owner = cur;
        s.camera_task_owner = if s.input_owner > (crate::engine::GS_MAX_SCRIPTS + crate::engine::GS_CLIENTS) as i32 {
            s.input_owner
        } else {
            0
        };
        s.camera_completion = s.game.camera_script_completion();
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera detached"));
    }
    Ok(())
}

/// `gs_camera_number`
fn number(a: &Args, i: usize, name: &str) -> LuaResult<f32> {
    let v = a.check_number(i)?;
    if v.is_nan() || v > 1_000_000.0 || v < -1_000_000.0 {
        return Err(lua_err(format!("camera {name} must be finite and within +/-1000000")));
    }
    Ok(v as f32)
}

fn tnum(t: &Table, k: &str, name: &str) -> LuaResult<f32> {
    // a table field read through luaL_checknumber's rules
    let v: Value = t.get(k)?;
    let view = Args::new("camera", mlua::MultiValue::from_iter([v]));
    number(&view, 1, name)
}

/// `gs_camera_vec`
fn vec3(t: &Table) -> LuaResult<[f32; 3]> {
    Ok([tnum(t, "x", "x")?, tnum(t, "y", "y")?, tnum(t, "z", "z")?])
}

/// `gs_camera_check_pose`: validates a pose table and returns which parts it sets (1 eye, 2 interest, 4 fov, 8 roll).
fn check_pose(s: &State, pose: &Table, required: bool) -> LuaResult<i32> {
    let mut mask = 0;
    let mut eye = [0f32; 3];
    let mut interest = [0f32; 3];
    for i in 0..3 {
        eye[i] = bitsf(s.game.camera_script_get_bits(i as i32));
        interest[i] = bitsf(s.game.camera_script_get_bits(i as i32 + 3));
    }
    for (i, name) in ["eye", "interest"].iter().enumerate() {
        let v: Value = pose.get(*name)?;
        if !matches!(v, Value::Nil) {
            let Value::Table(t) = v else {
                return Err(lua_err("bad argument #1 to 'camera' (table expected)"));
            };
            let vv = vec3(&t)?;
            if i == 0 { eye = vv } else { interest = vv }
            mask |= 1 << i;
        } else if required {
            return Err(lua_err(format!("camera key needs {name}")));
        }
    }
    let fov: Value = pose.get("fov")?;
    if !matches!(fov, Value::Nil) {
        let view = Args::new("camera", mlua::MultiValue::from_iter([fov]));
        let f = number(&view, 1, "fov")?;
        if f <= 1.0 || f >= 179.0 {
            return Err(lua_err("camera fov must be between 1 and 179 degrees"));
        }
        mask |= 4;
    } else if required {
        return Err(lua_err("camera key needs fov"));
    }
    let roll: Value = pose.get("roll")?;
    if !matches!(roll, Value::Nil) {
        let view = Args::new("camera", mlua::MultiValue::from_iter([roll]));
        number(&view, 1, "roll")?;
        mask |= 8;
    }
    if mask == 0 {
        return Err(lua_err("camera pose needs eye, interest, fov, or roll"));
    }
    if (mask & 3) == 3 || ((mask & 3) != 0 && s.game.camera_f(CAM_OK) != 0.0) {
        let (dx, dy, dz) = ((eye[0] - interest[0]) as f64, (eye[1] - interest[1]) as f64, (eye[2] - interest[2]) as f64);
        if dx * dx + dy * dy + dz * dz < 0.0001 {
            return Err(lua_err("camera eye and interest must be distinct"));
        }
    }
    Ok(mask)
}

/// `gs_camera_apply_pose`
fn apply_pose(s: &mut State, pose: &Table, mask: i32) -> LuaResult<()> {
    if mask & 1 != 0 {
        let Value::Table(t) = pose.get::<Value>("eye")? else { return Ok(()) };
        for (i, v) in vec3(&t)?.iter().enumerate() {
            s.game.camera_script_set(i as i32, fbits(*v));
        }
    }
    if mask & 2 != 0 {
        let Value::Table(t) = pose.get::<Value>("interest")? else { return Ok(()) };
        for (i, v) in vec3(&t)?.iter().enumerate() {
            s.game.camera_script_set(i as i32 + 3, fbits(*v));
        }
    }
    if mask & 4 != 0 {
        let v = tnum(pose, "fov", "fov")?;
        s.game.camera_script_set(6, fbits(v));
    }
    if mask & 8 != 0 {
        let v = tnum(pose, "roll", "roll")?;
        s.game.camera_script_set(7, fbits(v));
    }
    Ok(())
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "camera_get", |lua, st, _| {
        const MODES: [&str; 9] =
            ["standard", "pause", "training", "clear", "fixed", "free", "boss_intro", "debug_follow", "debug_free"];
        let s = st.borrow();
        if s.game.camera_f(CAM_OK) == 0.0 {
            return Ok(Value::Nil);
        }
        let g = &*s.game;
        let t = lua.create_table()?;
        for (name, base) in [("eye", 0), ("interest", 3)] {
            let v = lua.create_table()?;
            v.num("x", bitsf(g.camera_script_get_bits(base)) as f64)?;
            v.num("y", bitsf(g.camera_script_get_bits(base + 1)) as f64)?;
            v.num("z", bitsf(g.camera_script_get_bits(base + 2)) as f64)?;
            t.set(name, v)?;
        }
        t.num("fov", bitsf(g.camera_script_get_bits(6)) as f64)?;
        t.num("roll", bitsf(g.camera_script_get_bits(7)) as f64)?;
        let mode = g.camera_script_mode();
        t.string(
            "mode",
            match g.camera_script_state() {
                1 => "detached",
                2 => "attaching",
                _ if (0..9).contains(&mode) => MODES[mode as usize],
                _ => "unknown",
            },
        )?;
        Ok(Value::Table(t))
    })?;
    reg(e, gd, "camera_detach", |_, st, _| {
        claim(&mut st.borrow_mut(), "camera_detach")?;
        Ok(true)
    })?;
    reg(e, gd, "camera_attach", |_, st, a| {
        let frames = a.opt_integer(1, 30)? as i32;
        let mut s = st.borrow_mut();
        s.require_offline("camera_attach")?;
        if !(0..=6000).contains(&frames) {
            return Err(lua_err("camera_attach frames must be 0..6000"));
        }
        let cur = s.cur.map_or(0, |c| c as i32 + 1);
        if s.camera_owner != 0 && s.camera_owner != cur {
            return Err(lua_err("gd.camera_attach: camera is owned by another script"));
        }
        s.rw_branch();
        s.game.camera_script_attach(frames);
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera attach ({frames} frames)"));
        if frames == 0 {
            s.camera_owner = 0;
            s.camera_task_owner = 0;
        }
        Ok(())
    })?;
    reg(e, gd, "camera_set", |_, st, a| {
        let pose = a.check_table(1)?;
        let mut s = st.borrow_mut();
        let mask = check_pose(&s, &pose, false)?;
        claim(&mut s, "camera_set")?;
        apply_pose(&mut s, &pose, mask)?;
        s.game.camera_script_commit();
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera set"));
        Ok(())
    })?;
    reg(e, gd, "camera_move", |_, st, a| {
        let t = a.check_table(1)?;
        let to: Value = t.get("to")?;
        let Value::Table(to) = to else { return Err(lua_err("bad argument #1 to 'camera' (table expected)")) };
        let mut s = st.borrow_mut();
        let mask = check_pose(&s, &to, false)?;
        let fv: Value = t.get("frames")?;
        let frames = Args::new("camera_move", mlua::MultiValue::from_iter([fv])).check_integer(1)? as i32;
        if !(1..=6000).contains(&frames) {
            return Err(lua_err("camera_move frames must be 1..6000"));
        }
        let ev: Value = t.get("ease")?;
        let name = if matches!(ev, Value::Nil) {
            "linear".to_string()
        } else {
            Args::new("camera_move", mlua::MultiValue::from_iter([ev])).check_string(1)?
        };
        let ease = match name.as_str() {
            "in" => 1,
            "out" => 2,
            "inout" => 3,
            "linear" => 0,
            _ => return Err(lua_err("camera_move ease is linear, in, out, or inout")),
        };
        claim(&mut s, "camera_move")?;
        s.game.camera_script_move_begin();
        apply_pose(&mut s, &to, mask)?;
        s.game.camera_script_move(frames, ease);
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera move {frames} frames ease={ease}"));
        Ok(())
    })?;
    reg(e, gd, "camera_path", |_, st, a| {
        let t = a.check_table(1)?;
        let n = t.raw_len() as i32;
        if !(2..=64).contains(&n) {
            return Err(lua_err("camera_path needs 2..64 keys"));
        }
        let mut s = st.borrow_mut();
        let mut masks = Vec::new();
        let mut frames = Vec::new();
        let mut last = -1;
        for i in 0..n {
            let Value::Table(k) = t.raw_get::<Value>(i + 1)? else {
                return Err(lua_err("bad argument #1 to 'camera' (table expected)"));
            };
            masks.push(check_pose(&s, &k, true)?);
            let fv: Value = k.get("frame")?;
            let f = Args::new("camera_path", mlua::MultiValue::from_iter([fv])).check_integer(1)? as i32;
            if f <= last || f > 36000 || (i == 0 && f != 0) {
                return Err(lua_err("camera_path frames must start at 0 and increase through 36000"));
            }
            last = f;
            frames.push(f);
        }
        claim(&mut s, "camera_path")?;
        s.game.camera_script_path_clear();
        for i in 0..n as usize {
            let Value::Table(k) = t.raw_get::<Value>(i as i64 + 1)? else { unreachable!() };
            apply_pose(&mut s, &k, masks[i])?;
            if !s.game.camera_script_path_key(frames[i]) {
                return Err(lua_err(format!("camera_path: game rejected key {}", i + 1)));
            }
        }
        s.game.camera_script_path_start();
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera path {n} keys, {last} frames"));
        Ok(())
    })?;
    reg(e, gd, "camera_follow", |_, st, a| {
        let t = a.check_table(1)?;
        let (mut kind, mut id) = (0, 0);
        if let Value::Integer(p) = t.get::<Value>("port")? {
            kind = 1;
            id = p as i32 - 1;
        }
        if kind == 0 {
            if let Value::Integer(i) = t.get::<Value>("id")? {
                kind = 2;
                id = i as i32;
            }
        }
        let mut point = [0f32; 3];
        let mut offset = [0f32; 3];
        if kind == 0 {
            point = vec3(&t)?;
        }
        if !a.none_or_nil(2) {
            offset = vec3(&a.check_table(2)?)?;
        }
        if kind == 1 && !(0..6).contains(&id) {
            return Err(lua_err("camera_follow fighter port must be 1..6"));
        }
        if kind == 2 && id < 0 {
            return Err(lua_err("camera_follow item id is out of range"));
        }
        let mut s = st.borrow_mut();
        claim(&mut s, "camera_follow")?;
        let bits = |v: [f32; 3]| [fbits(v[0]), fbits(v[1]), fbits(v[2])];
        if !s.game.camera_script_follow(kind, id, bits(point), bits(offset)) {
            return Err(lua_err("camera_follow target is no longer live"));
        }
        let sid = s.script_id(s.cur);
        s.log(&format!("script [{sid}] camera follow kind={kind} id={id}"));
        Ok(())
    })?;
    reg(e, gd, "camera_shake", |_, st, a| {
        let intensity = number(&a, 1, "shake intensity")?;
        let frames = a.check_integer(2)? as i32;
        if intensity < 0.0 || !(0..=6000).contains(&frames) {
            return Err(lua_err("camera_shake needs nonnegative intensity and 0..6000 frames"));
        }
        let mut s = st.borrow_mut();
        claim(&mut s, "camera_shake")?;
        s.game.camera_script_shake(fbits(intensity), frames);
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera shake intensity={} frames={frames}", crate::args::fmt_g(intensity as f64, 6)));
        Ok(())
    })?;
    reg(e, gd, "camera_bounds", |_, st, a| {
        let lift = a.truthy(1);
        let mut s = st.borrow_mut();
        claim(&mut s, "camera_bounds")?;
        s.game.camera_script_lift_bounds(lift);
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}] camera bounds {}", if lift { "lifted" } else { "stage" }));
        Ok(())
    })?;
    Ok(())
}

#[allow(dead_code)]
fn _shared(_: &Shared) {}
