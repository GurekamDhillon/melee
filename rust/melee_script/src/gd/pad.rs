//! `gd.input`, `gd.release`, `gd.pad`, `gd.input_mask`, `gd.input_chord`, `gd.mirror_pad` (gw_script.c "input").
use super::core::{clampi, field_int};
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::Args;
use crate::engine::Engine;
use crate::game::*;
use mlua::{Result as LuaResult, Table, Value};

/// `gs_buttons`: every accepted name (the first twelve are the distinct ones `gd.pad` reports).
const NAMES: &[(&str, u32)] = &[
    ("A", 0x0100), ("B", 0x0200), ("X", 0x0400), ("Y", 0x0800), ("START", 0x1000), ("L", 0x0040), ("R", 0x0020),
    ("Z", 0x0010), ("UP", 0x0008), ("DOWN", 0x0004), ("LEFT", 0x0001), ("RIGHT", 0x0002), ("DUP", 0x0008),
    ("DDOWN", 0x0004), ("DLEFT", 0x0001), ("DRIGHT", 0x0002),
];

/// `gs_parse_buttons`: nil is none, an integer is the raw mask, a string is names joined by `+ , |` or spaces.
pub fn parse_buttons(a: &Args, i: usize) -> LuaResult<u32> {
    if a.none_or_nil(i) {
        return Ok(0);
    }
    if a.is_integer(i) {
        return Ok(a.check_integer(i)? as u32 & 0xFFFF);
    }
    if let Value::String(s) = a.get(i) {
        let b = s.as_bytes().to_vec();
        let sep = |c: u8| matches!(c, b'+' | b' ' | b',' | b'|');
        let mut bits = 0;
        let mut p = 0;
        while p < b.len() {
            while p < b.len() && sep(b[p]) {
                p += 1;
            }
            let mut tok = Vec::new();
            while p < b.len() && !sep(b[p]) && tok.len() < 15 {
                tok.push(b[p].to_ascii_uppercase());
                p += 1;
            }
            if tok.is_empty() {
                continue;
            }
            let tok = String::from_utf8_lossy(&tok).into_owned();
            match NAMES.iter().find(|(n, _)| *n == tok) {
                Some((_, bit)) => bits |= bit,
                None => {
                    return Err(lua_err(format!(
                        "unknown button \"{tok}\" (A B X Y START L R Z UP DOWN LEFT RIGHT)"
                    )))
                }
            }
        }
        return Ok(bits);
    }
    Err(lua_err("buttons are a string like \"A+B\" or a number"))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "input", |_, st, a| {
        let ch = slot_arg(&a, 1)? as i32;
        let frames = a.opt_integer(3, 1)? as i32;
        let mut s = st.borrow_mut();
        s.require_gameplay("input")?;
        if ch >= 4
            && (!s.match_active || s.game.fighter_i(ch, SI_SLOT_TYPE) != 1 || !players_present(&*s.game, ch as usize))
        {
            return Err(lua_err("gd.input: slots 5-6 require a live CPU fighter in an offline match"));
        }
        let (buttons, mut sx, mut sy, mut cx, mut cy, mut tl, mut tr) = (0u32, 0, 0, 0, 0, 0, 0);
        let buttons = if a.is_table(2) {
            let t = a.check_table(2)?;
            let bv: Value = t.get("buttons")?;
            let view = Args::new("input", mlua::MultiValue::from_iter([bv]));
            let b = parse_buttons(&view, 1)?;
            sx = clampi(field_int(&t, "x", 0), -127, 127);
            sy = clampi(field_int(&t, "y", 0), -127, 127);
            cx = clampi(field_int(&t, "cx", 0), -127, 127);
            cy = clampi(field_int(&t, "cy", 0), -127, 127);
            tl = clampi(field_int(&t, "l", 0), 0, 255);
            tr = clampi(field_int(&t, "r", 0), 0, 255);
            b
        } else {
            let _ = buttons;
            parse_buttons(&a, 2)?
        };
        let owner = s.pad_owner();
        if ch >= 4 {
            let v = [owner, frames.max(1), buttons as i32, sx, sy, cx, cy, tl, tr];
            s.rw_branch();
            for (k, val) in v.iter().enumerate() {
                s.game.virtual_pad_field(ch, k as i32, *val, true);
            }
        } else {
            s.pads.override_pad(ch, owner, buttons, sx, sy, cx, cy, tl, tr, frames.max(1));
            let log: Vec<String> = s.pads.log.drain(..).collect();
            for l in log {
                s.log(&l);
            }
        }
        Ok(())
    })?;
    // "release" and its alias "release_pad" are the same function
    let release = e.native("release", |_, st, a| {
        let ch = slot_arg(&a, 1)? as i32;
        let mut s = st.borrow_mut();
        if ch >= 4 {
            s.require_offline("release_pad")?;
            s.rw_branch();
        }
        let owner = s.pad_owner();
        s.pad_release(ch, owner);
        if let Some(cur) = s.cur.filter(|&c| c < s.scripts.len()) {
            if cur != s.console || s.client_owner == 0 {
                s.pad_release(ch, cur as i32 + 1);
            }
            let owners: Vec<i32> = s.scripts[cur]
                .tasks
                .iter()
                .flatten()
                .filter(|t| cur != s.console || t.client_owner == s.client_owner)
                .map(|t| t.pad_owner)
                .collect();
            for o in owners {
                s.pad_release(ch, o);
            }
        }
        Ok(())
    })?;
    gd.set("release", release.clone())?;
    gd.set("release_pad", release)?;
    reg(e, gd, "pad", |lua, st, a| {
        let ch = slot_arg(&a, 1)? as i32;
        let s = st.borrow();
        let (mut b, mut sx, mut sy, mut cx, mut cy, mut tl, mut tr) = (0u32, 0, 0, 0, 0, 0, 0);
        if ch >= 4 {
            if s.match_active {
                let g = &*s.game;
                b = g.virtual_pad_seen(ch, 0) as u32;
                sx = g.virtual_pad_seen(ch, 1);
                sy = g.virtual_pad_seen(ch, 2);
                cx = g.virtual_pad_seen(ch, 3);
                cy = g.virtual_pad_seen(ch, 4);
                tl = g.virtual_pad_seen(ch, 5);
                tr = g.virtual_pad_seen(ch, 6);
            }
        } else {
            (b, sx, sy, cx, cy, tl, tr) = s.pads.state(ch);
            if a.truthy(2) {
                b = s.pads.raw_buttons(ch);
            }
        }
        let t = lua.create_table()?;
        t.int("buttons", b as i64)?;
        t.int("x", sx as i64)?;
        t.int("y", sy as i64)?;
        t.int("cx", cx as i64)?;
        t.int("cy", cy as i64)?;
        t.int("l", tl as i64)?;
        t.int("r", tr as i64)?;
        for (name, bit) in &NAMES[..12] {
            t.boolean(name, b & bit != 0)?;
        }
        Ok(t)
    })?;
    reg(e, gd, "input_mask", |_, st, a| {
        let ch = slot_arg(&a, 1)? as i32;
        let bits = a.check_integer(2)?;
        let mut s = st.borrow_mut();
        s.require_offline("input_mask")?;
        if ch > 3 || bits < 0 || (bits & !0x100F) != 0 {
            return Err(lua_err("input_mask: port 1-4, D-pad bits 0-15 plus 0x1000 for START"));
        }
        let owner = s.cur.map_or(0, |c| c as i32 + 1);
        Ok(s.pads.mask(ch, owner, bits as u32))
    })?;
    reg(e, gd, "input_chord", |_, st, a| {
        let ch = slot_arg(&a, 1)? as i32;
        let mut s = st.borrow_mut();
        s.require_offline("input_chord")?;
        let bits = parse_buttons(&a, 2)?;
        if ch > 3 {
            return Err(lua_err("input_chord: port 1-4"));
        }
        let owner = s.cur.map_or(0, |c| c as i32 + 1);
        Ok(s.pads.chord(ch, owner, bits))
    })?;
    reg(e, gd, "mirror_pad", |_, st, a| {
        let mut s = st.borrow_mut();
        s.require_offline("mirror_pad")?;
        if a.none_or_nil(1) {
            s.pads.mirror(-1, -1);
            return Ok(());
        }
        let from = slot_arg(&a, 1)? as i32;
        let to = slot_arg(&a, 2)? as i32;
        if from > 3 || to > 3 || from == to {
            return Err(lua_err("gd.mirror_pad: two different ports 1-4"));
        }
        s.pads.mirror(from, to);
        // gd.mirror_pad(from, to, true) also leaves `from` neutral (the Lab dummy's recording)
        s.pads.take(a.truthy(3));
        Ok(())
    })?;
    Ok(())
}
