//! The Geno Lab reads (docs/geno.md 14.3): `gd.hitboxes`, `hurtboxes`, `joints`, `dobjs`, `project`, `attrs`,
//! `motion_name`, the two debug-draw switches and `set_percent`. They never write the game except the draw
//! switches (offline only) and the percent.
use super::players::{body_state, motion_name_at, push_hitbox, push_hitbox_list};
use super::{reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::{Engine, Shared};
use crate::game::{lab::*, *};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};

/// `gs_present_arg`: the slot, or -1 when no fighter is there.
fn present_arg(st: &Shared, a: &Args, i: usize) -> LuaResult<i32> {
    let slot = slot_arg(a, i)?;
    Ok(if players_present(&*st.borrow().game, slot) { slot as i32 } else { -1 })
}

/// `gs_project`: world -> the 640x480 script screen through the match camera (the maths of
/// `lbVector_WorldToScreen`). `Ok((sx, sy, depth))` in front of the camera; `Err(None)` with no camera, `Err(Some(depth))` otherwise.
pub fn project(g: &dyn ScriptGame, canvas_w: f32, x: f32, y: f32, z: f32) -> Result<(f32, f32, f32), Option<f32>> {
    if g.camera_f(CAM_OK) == 0.0 {
        return Err(None);
    }
    let c: Vec<f32> = (0..CAM_COUNT).map(|k| g.camera_f(k)).collect();
    let m = &c[CAM_VIEW as usize..CAM_VIEW as usize + 12];
    let ex = m[0] * x + m[1] * y + m[2] * z + m[3];
    let ey = m[4] * x + m[5] * y + m[6] * z + m[7];
    let ez = m[8] * x + m[9] * y + m[10] * z + m[11];
    let proj = c[CAM_PROJ as usize] as i32;
    let (xc, yc, wc);
    if proj == 2 {
        // ortho: top, bottom, left, right
        let (t, b, l, r) = (c[CAM_P0 as usize], c[CAM_P1 as usize], c[CAM_P2 as usize], c[CAM_P3 as usize]);
        if r == l || t == b {
            return Err(Some(0.0));
        }
        xc = ex * (2.0 / (r - l)) - (r + l) / (r - l);
        yc = ey * (2.0 / (t - b)) - (t + b) / (t - b);
        wc = 1.0f32;
    } else if proj == 0 {
        // perspective: fov (degrees), aspect
        let (fov, aspect) = (c[CAM_P0 as usize], c[CAM_P1 as usize]);
        if ez > -0.01 || aspect == 0.0 {
            return Err(Some(-ez)); // at or behind the camera
        }
        let cot = 1.0f32 / (fov * 0.5 * 3.14159265f32 / 180.0).tan();
        xc = ex * (cot / aspect);
        yc = ey * cot;
        wc = 1.0 / -ez;
    } else {
        return Err(Some(0.0)); // frustum cameras: not used by a match
    }
    let vx = c[CAM_VP_XMIN as usize];
    let vy = c[CAM_VP_YMIN as usize];
    let vw = c[CAM_VP_XMAX as usize] - vx;
    let vh = c[CAM_VP_YMAX as usize] - vy;
    let mut sx = vw * 0.5 + vx + wc * xc * vw * 0.5;
    let sy = vh * 0.5 + vy - wc * yc * vh * 0.5;
    // the overlay canvas grows with the window: world->overlay x stretches by canvas/640
    sx *= canvas_w / 640.0;
    Ok((sx, sy, -ez))
}

fn tobj_table(lua: &Lua, g: &dyn ScriptGame, slot: i32, d: i32, t: i32) -> LuaResult<Table> {
    const NAMES: [&str; 11] = ["id", "src", "flags", "tu", "tv", "su", "sv", "frame", "fmt", "w", "h"];
    let out = lua.create_table()?;
    for (f, n) in NAMES.iter().enumerate() {
        out.num(n, g.lab_tobj_f(slot, d, t, f as i32) as f64)?;
    }
    Ok(out)
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "debug_draw", |_, st, a| {
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let v = if a.none_or_nil(2) {
            st.borrow_mut().game.lab_debug_draw(slot, 0, 0)
        } else {
            let flags = a.check_integer(2)? as i32;
            let mut s = st.borrow_mut();
            s.require_offline("debug_draw")?;
            s.game.lab_debug_draw(slot, 1, flags & 0xFF)
        };
        Ok(Value::Integer(v as i64))
    })?;
    reg(e, gd, "debug_stage", |_, st, a| {
        let mut s = st.borrow_mut();
        let v;
        if a.none_or_nil(1) {
            v = s.game.lab_stage_draw(0, 0);
        } else {
            let flags = (a.check_integer(1)? as i32) & 31;
            s.require_offline("debug_stage")?;
            v = s.game.lab_stage_draw(31, flags);
            if v >= 0 {
                s.stage_zones = flags & STAGE_ZONES != 0;
            }
        }
        if v < 0 {
            return Ok(Value::Nil); // no match camera
        }
        Ok(Value::Integer((v | if s.stage_zones { STAGE_ZONES } else { 0 }) as i64))
    })?;
    reg(e, gd, "hitboxes", |lua, st, a| {
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let s = st.borrow();
        if !a.truthy(2) {
            return Ok(Value::Table(push_hitbox_list(lua, &*s.game, slot)?));
        }
        let out = lua.create_table()?;
        for i in 0..5 {
            out.raw_set(i + 1, push_hitbox(lua, &*s.game, slot, i)?)?;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "hurtboxes", |lua, st, a| {
        const HEIGHTS: [&str; 3] = ["low", "mid", "high"];
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let s = st.borrow();
        let g = &*s.game;
        let n = g.lab_i(slot, I_HURTBOXES);
        let out = lua.create_table()?;
        for i in 0..n.min(15) {
            let h = g.hurt_i(slot, i, UI_HEIGHT);
            let t = lua.create_table()?;
            t.int("id", i as i64)?;
            t.int("bone", g.hurt_i(slot, i, UI_BONE) as i64)?;
            t.string("state", body_state(g.hurt_i(slot, i, UI_STATE)))?;
            t.string("height", if (0..=2).contains(&h) { HEIGHTS[h as usize] } else { "?" })?;
            t.boolean("grabbable", g.hurt_i(slot, i, UI_GRABBABLE) == 1)?;
            for (k, f) in [("ax", UF_AX), ("ay", UF_AY), ("az", UF_AZ), ("bx", UF_BX), ("by", UF_BY), ("bz", UF_BZ), ("radius", UF_SIZE)] {
                t.num(k, g.hurt_f(slot, i, f) as f64)?;
            }
            out.raw_set(i + 1, t)?;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "joints", |lua, st, a| {
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        if a.truthy(2) {
            let mut s = st.borrow_mut();
            if s.host.borrow().session() {
                return Err(super::lua_err(
                    "gd.joints(port, true) is offline-only (refused during a netplay/rollback session)",
                ));
            }
            s.game.joints_setup(slot);
        }
        let s = st.borrow();
        let g = &*s.game;
        let cw = s.host.borrow().canvas_width();
        let n = g.lab_i(slot, I_JOINTS);
        let out = lua.create_table()?;
        for i in 0..n.max(0) {
            let (x, y, z) = (g.joint_f(slot, i, 0), g.joint_f(slot, i, 1), g.joint_f(slot, i, 2));
            let parent = g.joint_parent(slot, i);
            let t = lua.create_table()?;
            t.int("index", i as i64)?;
            t.int("parent", parent as i64)?;
            t.boolean("valid", parent != -2)?;
            t.num("x", x as f64)?;
            t.num("y", y as f64)?;
            t.num("z", z as f64)?;
            t.num("scale_x", g.joint_f(slot, i, 3) as f64)?;
            t.num("scale_y", g.joint_f(slot, i, 4) as f64)?;
            t.num("scale_z", g.joint_f(slot, i, 5) as f64)?;
            match if parent != -2 { project(g, cw, x, y, z).ok() } else { None } {
                Some((sx, sy, _)) => {
                    t.num("sx", sx as f64)?;
                    t.num("sy", sy as f64)?;
                    t.boolean("on", true)?;
                }
                None => t.boolean("on", false)?,
            }
            out.raw_set(i + 1, t)?;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "dobjs", |lua, st, a| {
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let mut s = st.borrow_mut();
        let g = &mut *s.game;
        let n = g.lab_dobj_i(slot, 0, DI_COUNT as u32);
        let out = lua.create_table()?;
        let nt = g.lab_dobj_i(slot, 0, DI_MODELS as u32);
        let models = lua.create_table()?;
        for i in 0..nt.min(12) {
            models.raw_set(i + 1, g.lab_dobj_i(slot, i, DI_MODEL_STATE as u32) as i64)?;
        }
        out.set("models", models)?;
        let nt = g.lab_dobj_i(slot, 0, DI_COSTUME_TOBJS as u32);
        let costume = lua.create_table()?;
        for i in 0..nt.min(5) {
            costume.raw_set(i + 1, tobj_table(lua, &*g, slot, -1, i)?)?;
        }
        out.set("costume", costume)?;
        for i in 0..n.max(0) {
            let flags = g.lab_dobj_i(slot, i, DI_FLAGS as u32);
            let d = lua.create_table()?;
            d.int("index", i as i64)?;
            d.boolean("hidden", flags & 1 != 0)?;
            d.int("render", g.lab_dobj_i(slot, i, DI_RENDER as u32) as i64)?;
            let nt = g.lab_dobj_i(slot, i, DI_TOBJS as u32);
            let tobjs = lua.create_table()?;
            for t in 0..nt.max(0) {
                tobjs.raw_set(t + 1, tobj_table(lua, &*g, slot, i, t)?)?;
            }
            d.set("tobjs", tobjs)?;
            out.raw_set(i + 1, d)?;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "project", |_, st, a| {
        let x = a.check_number(1)? as f32;
        let y = a.check_number(2)? as f32;
        let z = a.opt_number(3, 0.0)? as f32;
        let s = st.borrow();
        let cw = s.host.borrow().canvas_width();
        match project(&*s.game, cw, x, y, z) {
            Ok((sx, sy, depth)) => {
                let vis = sx >= 0.0 && sx < cw && sy >= 0.0 && sy < 480.0;
                Ok(MultiValue::from_iter([
                    Value::Number(sx as f64),
                    Value::Number(sy as f64),
                    Value::Boolean(vis),
                    Value::Number(depth as f64),
                ]))
            }
            Err(None) => Ok(MultiValue::from_iter([Value::Nil])),
            Err(Some(depth)) => Ok(MultiValue::from_iter([
                Value::Number(0.0),
                Value::Number(0.0),
                Value::Boolean(false),
                Value::Number(depth as f64),
            ])),
        }
    })?;
    reg(e, gd, "attrs", |lua, st, a| {
        let slot = present_arg(st, &a, 1)?;
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let s = st.borrow();
        let t = lua.create_table()?;
        for i in 0..s.game.lab_attr_count() {
            if let Some(name) = s.game.lab_attr_name(i) {
                t.num(&name, s.game.lab_attr_f(slot, i) as f64)?;
            }
        }
        Ok(Value::Table(t))
    })?;
    reg(e, gd, "motion_name", |lua, st, a| {
        let id = a.check_integer(1)? as i32;
        let (mut kind, mut slot) = (-1, -1);
        if !a.none_or_nil(2) {
            slot = present_arg(st, &a, 2)?;
            if slot >= 0 {
                kind = st.borrow().game.lab_i(slot, I_NAME_KIND);
            }
        }
        let s = st.borrow();
        Ok(lua.create_string(motion_name_at(&*s.game, slot, kind, id, None))?)
    })?;
    reg(e, gd, "set_percent", |_, st, a| {
        let slot = slot_arg(&a, 1)?;
        let p = a.check_number(2)? as i32;
        let mut s = st.borrow_mut();
        s.require_gameplay("set_percent")?;
        s.rw_branch();
        s.game.set_percent(slot as i32, super::core::clampi(p, 0, 999));
        Ok(())
    })?;
    Ok(())
}
