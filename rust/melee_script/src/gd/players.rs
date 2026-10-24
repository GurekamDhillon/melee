//! `gd.scene`, `gd.match`, `gd.players`, `gd.player`, `gd.char_name` and the player table they share
//! (`gs_push_player`, `gs_push_lab_fields`).
use super::{reg, slot_arg, TableExt};
use crate::engine::State;
use crate::game::{lab::*, *};
use crate::motion_names;
use mlua::{Lua, Result as LuaResult, Table, Value};

pub const CHAR_NAMES: [&str; 26] = [
    "Captain Falcon", "Donkey Kong", "Fox", "Mr. Game & Watch", "Kirby", "Bowser", "Link", "Luigi", "Mario", "Marth",
    "Mewtwo", "Ness", "Peach", "Pikachu", "Ice Climbers", "Jigglypuff", "Samus", "Yoshi", "Zelda", "Sheik", "Falco",
    "Young Link", "Dr. Mario", "Roy", "Pichu", "Ganondorf",
];

/// `GS_GENO_MOTION_BASE`: Geno action states are motion 0x400 + n.
pub const GENO_MOTION_BASE: i32 = 0x400;

/// `gs_char_name`
pub fn char_name(g: &dyn ScriptGame, c: i32) -> String {
    if c >= 0 && (c as usize) < CHAR_NAMES.len() {
        return CHAR_NAMES[c as usize].to_string();
    }
    // a Geno-defined fighter (ckind = kind + 1): the define's own name, as written
    if c > 0 {
        if let Some(n) = g.geno_define_name(c) {
            return n;
        }
    }
    // an m-ex fighter: its own name from the disc's m-ex data, lower case like the vanilla table
    if c >= 0 {
        if let Some(m) = g.mex_fighter_name(c).filter(|m| !m.is_empty()) {
            return m.to_ascii_lowercase().chars().take(47).collect();
        }
    }
    format!("character {c}")
}

/// `gs_anim_name`: "PlyKirby5K_Share_ACTION_AttackAirF_figatree" -> "AttackAirF"; the whole symbol when it has
/// another shape; "" for none.
pub fn anim_name(sym: Option<&str>) -> String {
    let Some(sym) = sym else { return String::new() };
    let p = sym.find("_ACTION_").map(|i| i + 8).unwrap_or(0);
    let rest = &sym[p..];
    let n = rest.find("_figatree").unwrap_or(rest.len());
    rest[..n.min(127)].to_string()
}

/// `gs_motion_name`: common states, each vanilla fighter's specials, then the animation name, then `Special<n>`.
pub fn motion_name(name_kind: i32, motion: i32, fallback: Option<&str>) -> String {
    if motion < 0 {
        return "none".into();
    }
    let common = motion_names::COMMON_COUNT as i32;
    if motion < common && !motion_names::COMMON[motion as usize].is_empty() {
        return motion_names::COMMON[motion as usize].into();
    }
    if motion >= common && name_kind >= 0 && (name_kind as usize) < motion_names::SPECIAL.len() {
        let k = (motion - common) as usize;
        let tbl = motion_names::SPECIAL[name_kind as usize];
        if k < tbl.len() && !tbl[k].is_empty() {
            return tbl[k].into();
        }
    }
    if let Some(f) = fallback.filter(|f| !f.is_empty()) {
        return f.into();
    }
    format!("Special{}", motion - common)
}

/// `gs_motion_name_at`: plus Geno action states by their geno.json name.
pub fn motion_name_at(g: &dyn ScriptGame, slot: i32, name_kind: i32, motion: i32, fallback: Option<&str>) -> String {
    if motion >= GENO_MOTION_BASE && slot >= 0 {
        let profile = g.geno_profile_for_kind(g.lab_i(slot, I_KIND));
        if let Some(n) = g.geno_state_name(profile, motion - GENO_MOTION_BASE).filter(|n| !n.is_empty()) {
            return n;
        }
        return format!("Geno{}", motion - GENO_MOTION_BASE);
    }
    motion_name(name_kind, motion, fallback)
}

pub const ELEMENT_NAMES: [&str; 17] = [
    "normal", "fire", "electric", "slash", "coin", "ice", "nap", "sleep", "catch", "ground", "cape", "inert", "disable",
    "dark", "scball", "lipstick", "leadead",
];
pub fn element_name(e: i32) -> &'static str {
    if e >= 0 && (e as usize) < ELEMENT_NAMES.len() { ELEMENT_NAMES[e as usize] } else { "?" }
}
pub fn body_state(v: i32) -> &'static str {
    match v {
        0 => "normal",
        1 => "invincible",
        2 => "intangible",
        _ => "?",
    }
}

/// `gs_push_hit_fields`
pub fn push_hit_fields(t: &Table, hi: &[i32], hf: &[f32]) -> LuaResult<()> {
    t.int("group", hi[HI_GROUP as usize] as i64)?;
    t.int("bone", hi[HI_BONE as usize] as i64)?;
    t.num("damage", hf[HF_DAMAGE as usize] as f64)?;
    t.int("angle", hi[HI_ANGLE as usize] as i64)?;
    t.int("kbg", hi[HI_KBG as usize] as i64)?;
    t.int("bkb", hi[HI_BKB as usize] as i64)?;
    t.int("wbk", hi[HI_WBK as usize] as i64)?;
    t.int("element", hi[HI_ELEMENT as usize] as i64)?;
    t.string("element_name", element_name(hi[HI_ELEMENT as usize]))?;
    t.int("shield_damage", hi[HI_SHIELD_DMG as usize] as i64)?;
    t.num("radius", hf[HF_SIZE as usize] as f64)?;
    for (k, f) in [("x", HF_X), ("y", HF_Y), ("z", HF_Z), ("px", HF_PX), ("py", HF_PY), ("pz", HF_PZ)] {
        t.num(k, hf[f as usize] as f64)?;
    }
    t.boolean("hit_air", hi[HI_HIT_AIR as usize] != 0)?;
    t.boolean("hit_ground", hi[HI_HIT_GROUND as usize] != 0)?;
    Ok(())
}

/// `gs_push_hitbox`
pub fn push_hitbox(lua: &Lua, g: &dyn ScriptGame, slot: i32, i: i32) -> LuaResult<Table> {
    let hi: Vec<i32> = (0..HI_COUNT).map(|k| g.hit_i(slot, i, k)).collect();
    let hf: Vec<f32> = (0..HF_COUNT).map(|k| g.hit_f(slot, i, k)).collect();
    let t = lua.create_table()?;
    t.int("id", i as i64)?;
    t.int("state", hi[HI_STATE as usize] as i64)?;
    t.boolean("active", hi[HI_STATE as usize] != 0)?;
    t.boolean("thrown", i == 4)?;
    push_hit_fields(&t, &hi, &hf)?;
    t.num("ox", hf[HF_OX as usize] as f64)?;
    t.num("oy", hf[HF_OY as usize] as f64)?;
    t.num("oz", hf[HF_OZ as usize] as f64)?;
    t.int("sfx_severity", hi[HI_SFX_SEVERITY as usize] as i64)?;
    t.int("sfx_kind", hi[HI_SFX_KIND as usize] as i64)?;
    t.boolean("clank", hi[HI_CLANK as usize] != 0)?;
    t.boolean("rebound", hi[HI_REBOUND as usize] != 0)?;
    Ok(t)
}

/// `gs_push_hitbox_list`: the fighter's hitboxes that are on (0-3, and 4 = the thrown hitbox)
pub fn push_hitbox_list(lua: &Lua, g: &dyn ScriptGame, slot: i32) -> LuaResult<Table> {
    let out = lua.create_table()?;
    let mut n = 0;
    for i in 0..5 {
        if g.hit_i(slot, i, HI_STATE) > 0 {
            n += 1;
            out.raw_set(n, push_hitbox(lua, g, slot, i)?)?;
        }
    }
    Ok(out)
}

fn push_xy(lua: &Lua, t: &Table, k: &str, x: f32, y: f32) -> LuaResult<()> {
    let p = lua.create_table()?;
    p.num("x", x as f64)?;
    p.num("y", y as f64)?;
    t.set(k, p)
}

/// `gs_push_lab_fields`
pub fn push_lab_fields(lua: &Lua, s: &State, t: &Table, slot: i32) -> LuaResult<()> {
    let g = &*s.game;
    let sym = g.lab_anim_symbol(slot);
    let name_kind = g.lab_i(slot, I_NAME_KIND);
    let action = g.fighter_i(slot, SI_ACTION);
    let jumps_used = g.lab_i(slot, I_JUMPS_USED);
    let jumps_max = g.lab_i(slot, I_MAX_JUMPS);
    let anim = anim_name(sym.as_deref());
    let li = |f| g.lab_i(slot, f);
    let lf = |f| g.lab_f(slot, f) as f64;
    t.string("motion_name", &motion_name_at(g, slot, name_kind, action, Some(&anim)))?;
    t.int("anim_id", li(I_ANIM_ID) as i64)?;
    t.string("anim_name", &anim)?;
    t.string("anim_symbol", sym.as_deref().unwrap_or(""))?;
    t.num("anim_frame_f", g.fighter_f(slot, SF_ANIM_FRAME) as f64)?;
    t.num("anim_rate", lf(F_ANIM_RATE))?;
    t.num("anim_end", g.lab_anim_end(slot) as f64)?; // the playing clip's last frame (LAB CLIPS)
    t.num("hitstun", lf(F_HITSTUN))?;
    t.boolean("in_hitlag", li(I_IN_HITLAG) == 1)?;
    t.boolean("in_hitstun", li(I_IN_HITSTUN) == 1)?;
    t.int("intangible", li(I_INTANG_TIMER) as i64)?;
    t.int("invincible", li(I_INVINC_TIMER) as i64)?;
    t.string("body_state", body_state(li(I_BODY_STATE)))?;
    t.string("timed_state", body_state(li(I_TIMED_STATE)))?;
    t.num("kb_vx", lf(F_KB_VX))?;
    t.num("kb_vy", lf(F_KB_VY))?;
    t.num("ground_vel", lf(F_GR_VEL))?;
    t.num("kb_applied", lf(F_KB_APPLIED))?;
    t.num("z", lf(F_Z))?;
    t.num("scale", lf(F_SCALE))?;
    t.num("cmd_timer", lf(F_CMD_TIMER))?;
    t.num("kb_last", lf(F_KB_LAST))?;
    let ecb = lua.create_table()?;
    push_xy(lua, &ecb, "top", g.lab_f(slot, F_ECB_TOP_X), g.lab_f(slot, F_ECB_TOP_Y))?;
    push_xy(lua, &ecb, "bottom", g.lab_f(slot, F_ECB_BOTTOM_X), g.lab_f(slot, F_ECB_BOTTOM_Y))?;
    push_xy(lua, &ecb, "left", g.lab_f(slot, F_ECB_LEFT_X), g.lab_f(slot, F_ECB_LEFT_Y))?;
    push_xy(lua, &ecb, "right", g.lab_f(slot, F_ECB_RIGHT_X), g.lab_f(slot, F_ECB_RIGHT_Y))?;
    t.set("ecb", ecb)?;
    t.int("ecb_lock", li(I_ECB_LOCK) as i64)?;
    t.int("jumps_used", jumps_used as i64)?;
    t.int("jumps_max", jumps_max as i64)?;
    t.int("jumps_left", if jumps_max > jumps_used { jumps_max - jumps_used } else { 0 } as i64)?;
    t.boolean("on_floor", li(I_ON_FLOOR) == 1)?;
    if li(I_FLOOR_VALID) == 1 {
        t.num("floor_y", lf(F_FLOOR_Y))?;
    } else {
        t.set("floor_y", Value::Nil)?;
    }
    t.boolean("floor_passthrough", li(I_FLOOR_PASSTHROUGH) == 1)?;
    t.int("wall", li(I_WALL) as i64)?;
    t.boolean("ceiling", li(I_CEILING) == 1)?;
    t.boolean("ledge", li(I_LEDGE) == 1)?;
    t.int("walljumps_used", li(I_WALLJUMPS_USED) as i64)?;
    t.num("shield", lf(F_SHIELD))?;
    t.boolean("iasa", li(I_IASA) == 1)?;
    t.int("ledge_cooldown", li(I_LEDGE_COOLDOWN) as i64)?;
    t.int("lr_age", li(I_LR_AGE) as i64)?;
    t.int("jump_age", li(I_JUMP_AGE) as i64)?;
    t.boolean("shield_on", li(I_SHIELD_ON) == 1)?;
    t.num("shield_x", lf(F_SHIELD_X))?;
    t.num("shield_y", lf(F_SHIELD_Y))?;
    t.num("shield_r", lf(F_SHIELD_R))?;
    t.int("draw_flags", li(I_DRAW_FLAGS) as i64)?;
    t.int("joint_count", li(I_JOINTS) as i64)?;
    t.boolean("hidden", li(I_HIDDEN) == 1)?;
    t.int("hurtbox_count", li(I_HURTBOXES) as i64)?;
    t.set("hitboxes", push_hitbox_list(lua, g, slot)?)?;
    Ok(())
}

/// `gs_push_player`
pub fn push_player(lua: &Lua, s: &State, slot: usize) -> LuaResult<Table> {
    let g = &*s.game;
    let sl = slot as i32;
    let t = lua.create_table()?;
    t.int("port", slot as i64 + 1)?;
    // `entity_ref` (gs_entity_push_current) needs the entity epoch registry; nil until that is ported
    let ch = g.fighter_i(sl, SI_CHAR);
    t.int("char", ch as i64)?;
    t.string("char_name", &char_name(g, ch))?;
    t.int("kind", g.fighter_i(sl, SI_KIND) as i64)?;
    t.int("costume", g.fighter_i(sl, SI_COSTUME) as i64)?;
    t.boolean("cpu", g.fighter_i(sl, SI_SLOT_TYPE) == 1)?;
    t.int("team", g.fighter_i(sl, SI_TEAM) as i64)?;
    t.num("x", g.fighter_f(sl, SF_X) as f64)?;
    t.num("y", g.fighter_f(sl, SF_Y) as f64)?;
    t.num("vx", g.fighter_f(sl, SF_VX) as f64)?;
    t.num("vy", g.fighter_f(sl, SF_VY) as f64)?;
    t.num("percent", g.fighter_f(sl, SF_PERCENT) as f64)?;
    t.int("stocks", g.fighter_i(sl, SI_STOCKS) as i64)?;
    t.int("falls", g.fighter_i(sl, SI_FALLS) as i64)?;
    t.int("facing", if g.fighter_f(sl, SF_FACING) < 0.0 { -1 } else { 1 })?;
    t.int("action", g.fighter_i(sl, SI_ACTION) as i64)?;
    t.int("action_frame", s.state_frame[slot] as i64)?;
    t.num("anim_frame", g.fighter_f(sl, SF_ANIM_FRAME) as f64)?;
    t.boolean("airborne", g.fighter_i(sl, SI_AIRBORNE) == 1)?;
    t.num("hitlag", g.fighter_f(sl, SF_HITLAG) as f64)?;
    push_lab_fields(lua, s, &t, sl)?;
    Ok(t)
}

pub fn register(e: &crate::engine::Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "scene", |lua, st, _| {
        let s = st.borrow();
        let mode = s.game.game_mode();
        let t = lua.create_table()?;
        t.int("kind", s.scene_kind as i64)?;
        t.string("name", &s.host.borrow().scene_name(s.scene_kind))?;
        t.int("mode", mode as i64)?;
        t.string("mode_name", &s.host.borrow().mode_name(mode))?;
        t.int("epoch", s.scene_epoch as i64)?;
        Ok(t)
    })?;
    reg(e, gd, "match", |lua, st, _| {
        let s = st.borrow();
        let t = lua.create_table()?;
        t.boolean("active", s.match_active)?;
        t.int("frame", s.match_frame as i64)?;
        t.int("stage", if s.match_active { s.game.stage_kind() } else { -1 } as i64)?;
        t.int("stage_external", if s.match_active { s.game.stage_external() } else { -1 } as i64)?;
        t.boolean("netplay", s.host.borrow().session())?;
        Ok(t)
    })?;
    reg(e, gd, "players", |lua, st, _| {
        let s = st.borrow();
        let out = lua.create_table()?;
        let mut n = 0;
        for slot in 0..6 {
            if players_present(&*s.game, slot) {
                n += 1;
                out.raw_set(n, push_player(lua, &s, slot)?)?;
            }
        }
        Ok(out)
    })?;
    reg(e, gd, "player", |lua, st, a| {
        let slot = slot_arg(&a, 1)?;
        let s = st.borrow();
        if !players_present(&*s.game, slot) {
            return Ok(Value::Nil);
        }
        Ok(Value::Table(push_player(lua, &s, slot)?))
    })?;
    reg(e, gd, "char_name", |_, st, a| {
        let c = a.check_integer(1)? as i32;
        Ok(char_name(&*st.borrow().game, c))
    })?;
    Ok(())
}
