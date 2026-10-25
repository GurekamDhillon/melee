//! Action-state browsing and previews: `gd.timeline`, `gd.motion_list`, `gd.set_motion`, `gd.anim_rows`,
//! `gd.anim_preview` (gw_script.c "Stage 2 / Stage E"), plus the frame-boundary `gs_apply_set_motion`.
use super::players::{anim_name, motion_name_at, GENO_MOTION_BASE};
use super::{reg, slot_arg, TableExt};
use crate::engine::Engine;
use crate::game::lab::*;
use crate::game::*;
use mlua::{MultiValue, Result as LuaResult, Table, Value};

/// `GENO_FTCMD_OP` and the sub-ids of `pc/geno/geno.h`
pub const GENO_FTCMD_OP: i32 = 59;
pub const GENO_SUB_ORIG: i32 = 0x13;
pub const GENO_SUB_SKIP: i32 = 0x11;
pub const GENO_SUB_IF: i32 = 0x10;
pub const GENO_SUB_IFV: i32 = 0x12;

/// `gs_cmd_len`: command lengths in words for opcodes 10-58 (ftaction.c ftAction_803C0870)
const CMD_LEN: [u8; 49] = [
    5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 3, 3, 2, 1, 4,
];
/// `gs_cmd_names`
const CMD_NAMES: [&str; 64] = [
    "end", "wait", "wait_until", "loop", "loop_end", "call", "return", "goto", "wait_anim", "bg_flash", "gfx", "hitbox",
    "hitbox_damage", "hitbox_size", "hitbox_flags", "hitbox_remove", "hitboxes_clear", "sfx", "smash_sfx", "cmd_var",
    "throw_flag", "throw_flag_b1", "throw_flag_b2", "iasa", "throw_flag_b0", "air_state", "body_state",
    "hurtboxes_state", "hurtbox_state", "jab_combo", "jab_rapid", "model_state", "models_revert", "models_remove",
    "throw", "item_visibility", "article_visibility", "visibility", "random_sfx", "pitch_sfx", "tex_anim", "part_anim",
    "parasol", "rumble", "rumble_stop", "color_anim", "color_overlay", "color_overlay_off", "flag_221E", "sword_trail",
    "anim_part", "self_damage", "continuation", "flag_2225", "footstep_fx", "landing_fx", "smash_charge", "unk_57",
    "wind", "geno", "op_60", "op_61", "op_62", "op_63",
];

fn geno_sub_name(sub: i32) -> &'static str {
    match sub {
        0x00 => "geno.NOP", 0x01 => "geno.SET", 0x02 => "geno.ADD", 0x03 => "geno.SUB", 0x04 => "geno.MUL",
        0x05 => "geno.SETBIT", 0x06 => "geno.CLRBIT", 0x07 => "geno.DIV", 0x08 => "geno.GET", 0x09 => "geno.PUT",
        0x0A => "geno.RAND", 0x10 => "geno.IF", 0x11 => "geno.SKIP", 0x12 => "geno.IFV", 0x13 => "geno.ORIG",
        0x20 => "geno.CALL", 0x30 => "geno.CHG", 0x31 => "geno.CHGAND", 0x32 => "geno.CHGCLR", 0x38 => "geno.REHIT",
        0x39 => "geno.LINK", 0x3A => "geno.HBDMG", 0x3B => "geno.HBSTUN", 0x3C => "geno.HBFLAGS",
        _ => "geno.unknown",
    }
}

fn ubits(v: u32, shift: u32, n: u32) -> i64 {
    ((v >> shift) & ((1u32 << n) - 1)) as i64
}
/// `gs_sbits`: signed field of `n` bits ending `hi` bits down
fn sbits(v: u32, hi: u32, n: u32) -> i64 {
    let x = (v << hi) as i32;
    (x >> (32 - n)) as i64
}

fn present(g: &dyn ScriptGame, slot: usize) -> i32 {
    if players_present(g, slot) { slot as i32 } else { -1 }
}

/// `gs_special_count`: the decomp's special-state table for the kind, or the m-ex MoveLogic table when longer
fn special_count(g: &dyn ScriptGame, slot: i32) -> i32 {
    let kind = g.lab_i(slot, I_NAME_KIND);
    let n = if kind >= 0 && (kind as usize) < crate::motion_names::SPECIAL.len() {
        crate::motion_names::SPECIAL[kind as usize].len() as i32
    } else {
        0
    };
    n.max(g.mex_move_logic_entries(g.lab_i(slot, I_KIND)))
}

/// `gs_motion_ok`: a motion id the fighter really has a row for
pub fn motion_ok(g: &dyn ScriptGame, slot: i32, motion: i32) -> bool {
    let common = g.lab_common_count(slot);
    if motion < 0 || common <= 0 {
        return false;
    }
    if motion >= GENO_MOTION_BASE {
        let p = g.geno_profile_for_kind(g.lab_i(slot, I_KIND));
        return p >= 0 && motion - GENO_MOTION_BASE < g.geno_state_count(p) && g.lab_motion_anim(slot, motion) >= -1;
    }
    if motion < common {
        return g.lab_motion_anim(slot, motion) >= -1;
    }
    if motion - common < special_count(g, slot) {
        return g.lab_motion_anim(slot, motion) >= -1;
    }
    false
}

/// One command as a Lua table (`gs_push_cmd`); `t` is the frame it runs (1-based).
fn push_cmd(lua: &mlua::Lua, g: &dyn ScriptGame, addr: u32, op: i32, t: f32, len: usize) -> LuaResult<Table> {
    let w0 = g.guest_word(addr);
    let c = lua.create_table()?;
    c.num("frame", (t + 1.0) as f64)?;
    c.int("op", op as i64)?;
    c.string("name", CMD_NAMES[(op & 63) as usize])?;
    c.int("addr", addr as i64)?;
    let words = lua.create_table()?;
    for k in 0..len {
        words.raw_set(k + 1, g.guest_word(addr + 4 * k as u32) as i64)?;
    }
    c.set("words", words)?;
    match op {
        GENO_FTCMD_OP => {
            let sub = ubits(w0, 20, 6) as i32;
            c.string("name", geno_sub_name(sub))?;
            c.int("sub", sub as i64)?;
            c.int("flags", (w0 & 0xffff) as i64)?;
            c.int("var", ubits(w0, 8, 8))?;
            c.boolean("operand_is_var", w0 & 0x80 != 0)?;
            c.int("compare", ubits(w0, 4, 3))?;
            let mut detail = String::new();
            for k in 1..len {
                let operand = g.guest_word(addr + 4 * k as u32);
                c.int(&format!("arg{k}"), operand as i64)?;
                if detail.len() < 192 {
                    detail.push_str(&format!("{}arg{k}=0x{operand:08X}", if k == 1 { "" } else { " " }));
                }
            }
            if len > 1 {
                c.string("detail", &detail)?;
            }
        }
        10 => {
            c.int("bone", ubits(w0, 18, 8))?;
            c.int("gfx", ubits(g.guest_word(addr + 4), 16, 16))?;
        }
        11 => {
            let (w1, w2, w3, w4) = (g.guest_word(addr + 4), g.guest_word(addr + 8), g.guest_word(addr + 12), g.guest_word(addr + 16));
            c.int("id", ubits(w0, 23, 3))?;
            c.int("group", ubits(w0, 20, 3))?;
            c.int("bone", ubits(w0, 11, 8))?;
            c.int("damage", ubits(w0, 0, 10))?;
            c.num("size", ubits(w1, 16, 16) as f64 / 256.0)?;
            c.num("oz", sbits(w1, 16, 16) as f64 / 256.0)?;
            c.num("oy", sbits(w2, 0, 16) as f64 / 256.0)?;
            c.num("ox", sbits(w2, 16, 16) as f64 / 256.0)?;
            c.int("angle", ubits(w3, 23, 9))?;
            c.int("kbg", ubits(w3, 14, 9))?;
            c.int("wbk", ubits(w3, 5, 9))?;
            c.int("bkb", ubits(w4, 23, 9))?;
            c.int("element", ubits(w4, 18, 5))?;
            c.string("element_name", super::players::element_name(ubits(w4, 18, 5) as i32))?;
            c.int("shield_damage", sbits(w4, 14, 8))?;
            c.boolean("hit_ground", ubits(w4, 1, 1) != 0)?;
            c.boolean("hit_air", ubits(w4, 0, 1) != 0)?;
        }
        12 | 13 => {
            c.int("id", ubits(w0, 23, 3))?;
            c.int("value", ubits(w0, 0, 23))?;
        }
        15 => c.int("id", ubits(w0, 0, 26))?,
        17 => c.int("sfx", g.guest_word(addr + 4) as i64)?,
        19 => {
            c.int("index", ubits(w0, 24, 2))?;
            c.int("value", ubits(w0, 0, 24))?;
        }
        26 | 27 | 37 | 36 | 35 | 25 | 29 | 30 => c.int("value", ubits(w0, 0, 26))?,
        28 => {
            c.int("bone", ubits(w0, 18, 8))?;
            c.int("value", ubits(w0, 0, 18))?;
        }
        _ => {}
    }
    Ok(c)
}

/// `gs_walk_script`: walk the script the way ftAction's interpreter times it. Returns (events, last frame, truncated).
fn walk_script(lua: &mlua::Lua, g: &dyn ScriptGame, start: u32) -> LuaResult<(Table, f32, i32)> {
    let events = lua.create_table()?;
    let (mut pc, mut t, mut steps, mut n) = (start, 0.0f32, 0, 0);
    let mut ret = [0u32; 8];
    let (mut loop_body, mut loop_left) = ([0u32; 8], [0i32; 8]);
    let (mut nret, mut nloop) = (0usize, 0usize);
    let mut truncated = 0;
    while g.guest_ok(pc, 4) {
        let w0 = g.guest_word(pc);
        let op = (w0 >> 26) as i32;
        steps += 1;
        if steps > 2000 || t > 1000.0 {
            truncated = (truncated & 4) | 1;
            break;
        }
        match op {
            0 => return Ok((events, t, truncated)),
            1 => {
                t += (w0 & 0x3FFFFFF) as f32;
                pc += 4;
                continue;
            }
            2 => {
                let at = (w0 & 0x3FFFFFF) as f32;
                if at > t {
                    t = at;
                }
                pc += 4;
                continue;
            }
            3 => {
                if nloop < 8 {
                    loop_body[nloop] = pc + 4;
                    loop_left[nloop] = (w0 & 0x3FFFFFF) as i32;
                    nloop += 1;
                }
                pc += 4;
                continue;
            }
            4 => {
                if nloop > 0 {
                    loop_left[nloop - 1] -= 1;
                }
                if nloop > 0 && loop_left[nloop - 1] > 0 {
                    pc = loop_body[nloop - 1];
                } else {
                    if nloop > 0 {
                        nloop -= 1;
                    }
                    pc += 4;
                }
                continue;
            }
            5 => {
                if nret < 8 && g.guest_ok(pc + 4, 4) {
                    ret[nret] = pc + 8;
                    nret += 1;
                    pc = g.guest_word(pc + 4);
                } else {
                    return Ok((events, t, truncated));
                }
                continue;
            }
            6 => {
                if nret == 0 {
                    return Ok((events, t, truncated));
                }
                nret -= 1;
                pc = ret[nret];
                continue;
            }
            7 => {
                if !g.guest_ok(pc + 4, 4) {
                    return Ok((events, t, truncated));
                }
                pc = g.guest_word(pc + 4);
                continue;
            }
            8 => {
                truncated = (truncated & 4) | 2; // waits for the animation to end
                return Ok((events, t, truncated));
            }
            9 => {
                pc += 4;
                continue;
            }
            _ => {}
        }
        let len: usize = if op == 59 {
            let l = ((w0 >> 16) & 0xF) as usize;
            if l == 0 { 1 } else { l }
        } else if (10..=58).contains(&op) {
            CMD_LEN[(op - 10) as usize] as usize
        } else {
            1
        };
        if !g.guest_ok(pc, 4 * len as u32) {
            break;
        }
        n += 1;
        events.raw_set(n, push_cmd(lua, g, pc, op, t, len)?)?;
        if op == GENO_FTCMD_OP {
            let sub = ubits(w0, 20, 6) as i32;
            if sub == GENO_SUB_ORIG {
                pc = g.geno_script_original(pc);
                continue;
            }
            if sub == GENO_SUB_SKIP && len >= 2 {
                let skip = g.guest_word(pc + 4);
                if skip > (u32::MAX - pc) / 4 - len as u32 {
                    truncated = (truncated & 4) | 1;
                    return Ok((events, t, truncated));
                }
                pc += 4 * (len as u32 + skip);
                continue;
            }
            if sub == GENO_SUB_IF || sub == GENO_SUB_IFV {
                // a static walk cannot know per-action variables: keep the fall-through path and disclose it
                truncated |= 4;
            }
        }
        pc += 4 * len as u32;
    }
    Ok((events, t, truncated))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "timeline", |lua, st, a| {
        let slot = slot_arg(&a, 1)? as usize;
        let s = st.borrow();
        let g = &*s.game;
        let slot = present(g, slot);
        if slot < 0 {
            return Ok(MultiValue::from_iter([Value::Nil]));
        }
        let name_kind = g.lab_i(slot, I_NAME_KIND);
        let (motion, anim);
        if a.none_or_nil(2) {
            motion = g.fighter_i(slot, SI_ACTION);
            anim = g.lab_i(slot, I_ANIM_ID);
        } else {
            motion = a.check_integer(2)? as i32;
            if !motion_ok(g, slot, motion) {
                return Ok(MultiValue::from_iter([
                    Value::Nil,
                    Value::String(lua.create_string("no such motion for this fighter")?),
                ]));
            }
            anim = g.lab_motion_anim(slot, motion);
        }
        let t = lua.create_table()?;
        t.int("motion", motion as i64)?;
        t.int("anim_id", anim as i64)?;
        let an = anim_name(if anim >= 0 { g.lab_anim_symbol_for(slot, anim) } else { None }.as_deref());
        t.string("anim_name", &an)?;
        t.string("motion_name", &motion_name_at(g, slot, name_kind, motion, Some(&an)))?;
        if a.none_or_nil(2) {
            t.num("end_frame", g.lab_anim_end(slot) as f64)?;
        }
        let script = if anim >= 0 { g.lab_script(slot, anim) } else { None };
        let Some(script) = script.filter(|&sc| g.guest_ok(sc, 4)) else {
            t.set("events", lua.create_table()?)?;
            t.num("length", 0.0)?;
            return Ok(MultiValue::from_iter([Value::Table(t)]));
        };
        t.int("script", script as i64)?;
        let (events, len, truncated) = walk_script(lua, g, script)?;
        t.set("events", events)?;
        t.num("length", (len + 1.0) as f64)?;
        t.string(
            "stop",
            match truncated & 3 {
                1 => "limit",
                2 => "anim_end",
                _ => "end",
            },
        )?;
        t.boolean("conditional", truncated & 4 != 0)?;
        Ok(MultiValue::from_iter([Value::Table(t)]))
    })?;
    reg(e, gd, "motion_list", |lua, st, a| {
        let slot = slot_arg(&a, 1)? as usize;
        let s = st.borrow();
        let g = &*s.game;
        let slot = present(g, slot);
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let common = g.lab_common_count(slot);
        let nspec = special_count(g, slot);
        let kind = g.lab_i(slot, I_NAME_KIND);
        let dec = if kind >= 0 && (kind as usize) < crate::motion_names::SPECIAL.len() {
            crate::motion_names::SPECIAL[kind as usize].len() as i32
        } else {
            0
        };
        let p = g.geno_profile_for_kind(g.lab_i(slot, I_KIND));
        let out = lua.create_table()?;
        let mut n = 0;
        let mut m = 0;
        while m < common + nspec + if p >= 0 { GENO_MOTION_BASE + g.geno_state_count(p) } else { 0 } {
            if m >= common + nspec && m < GENO_MOTION_BASE {
                if p < 0 {
                    break;
                }
                m = GENO_MOTION_BASE; // continue's ++m lands past GENO_MOTION_BASE - 1
                continue;
            }
            if !motion_ok(g, slot, m) {
                m += 1;
                continue;
            }
            let anim = g.lab_motion_anim(slot, m);
            let an = anim_name(if anim >= 0 { g.lab_anim_symbol_for(slot, anim) } else { None }.as_deref());
            let group = if m >= GENO_MOTION_BASE {
                "geno"
            } else if m < common {
                "common"
            } else if m - common < dec {
                "special"
            } else {
                "mex"
            };
            let r = lua.create_table()?;
            r.int("id", m as i64)?;
            r.string("name", &motion_name_at(g, slot, kind, m, Some(&an)))?;
            r.string("group", group)?;
            r.int("anim_id", anim as i64)?;
            r.string("anim_name", &an)?;
            n += 1;
            out.raw_set(n, r)?;
            m += 1;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "set_motion", |lua, st, a| {
        let motion = a.check_integer(2)? as i32;
        let frame = a.opt_integer(3, 1)? as i32;
        let rate = a.opt_number(4, 1.0)? as f32;
        let lift = a.opt_number(5, 0.0)? as f32;
        let mut s = st.borrow_mut();
        s.require_offline("set_motion")?;
        let mut slots = Vec::new();
        if a.is_table(1) {
            let t = a.check_table(1)?;
            let len = t.raw_len();
            for i in 1..=len {
                if slots.len() >= 6 {
                    break;
                }
                let v: Value = t.raw_get(i)?;
                let view = crate::args::Args::new("set_motion", MultiValue::from_iter([v]));
                slots.push(present(&*s.game, slot_arg(&view, 1)?));
            }
        } else {
            slots.push(present(&*s.game, slot_arg(&a, 1)?));
        }
        let fail = |why: &str| -> LuaResult<MultiValue> {
            Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
        };
        for &sl in &slots {
            if sl < 0 {
                return fail("no fighter on that port");
            }
            if !motion_ok(&*s.game, sl, motion) {
                return fail("no such motion for this fighter");
            }
        }
        let frame = frame.clamp(1, 600);
        s.step = 0; // a new request replaces one still stepping
        for &sl in &slots {
            s.set_motion_core(sl as usize, motion, frame, rate, lift);
        }
        s.paused = true;
        s.step = frame - 1; // entering is frame 1
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;
    reg(e, gd, "anim_rows", |lua, st, a| {
        let slot = slot_arg(&a, 1)? as usize;
        let s = st.borrow();
        let g = &*s.game;
        let slot = present(g, slot);
        if slot < 0 {
            return Ok(Value::Nil);
        }
        let n = g.lab_row_count(slot);
        let out = lua.create_table()?;
        for i in 0..n.max(0) {
            let sym = g.lab_anim_symbol_for(slot, i);
            let r = lua.create_table()?;
            r.int("row", i as i64)?;
            r.string("name", &anim_name(sym.as_deref()))?;
            r.string("symbol", sym.as_deref().unwrap_or(""))?;
            out.raw_set(i + 1, r)?;
        }
        Ok(Value::Table(out))
    })?;
    reg(e, gd, "anim_preview", |lua, st, a| {
        let slot = present(&*st.borrow().game, slot_arg(&a, 1)?);
        let row = if a.none_or_nil(2) { -1 } else { a.check_integer(2)? as i32 };
        let frame = a.opt_integer(3, 1)? as i32;
        let rate = a.opt_number(4, 1.0)? as f32;
        let facing = a.opt_integer(5, 0)? as i32;
        const WAIT: i32 = 14; // ftCo_MS_Wait
        let mut s = st.borrow_mut();
        s.require_offline("anim_preview")?;
        let fail = |why: &str| -> LuaResult<MultiValue> {
            Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
        };
        if slot < 0 {
            return fail("no fighter on that port");
        }
        if row >= s.game.lab_row_count(slot) {
            return fail("no such animation row");
        }
        s.game.lab_set_preview_row(slot, row, facing);
        let frame = frame.clamp(1, 600);
        s.step = 0;
        s.set_motion[slot as usize] = WAIT + 1;
        s.set_rate[slot as usize] = rate;
        s.set_lift[slot as usize] = 0.0;
        s.paused = true;
        s.step = frame - 1;
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;
    Ok(())
}
