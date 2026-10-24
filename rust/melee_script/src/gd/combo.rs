//! `gd.combo(port [, all])` and `gd.combo_ki(port, n)`: the Geno combo graph (`pc/platform/gw_script_combo.inc`,
//! docs/geno.md section 25). `gd.combo` is a read; `gd.combo_ki` writes the meter (offline, a rewind branch).
use super::{reg, slot_arg, TableExt};
use crate::engine::Engine;
use mlua::{Result as LuaResult, Table, Value};

// geno.h: GENO_CR_*, the target word
const CR_FROM: i32 = 0;
const CR_TO: i32 = 1;
const CR_INPUT: i32 = 2;
const CR_DIR: i32 = 3;
const CR_ON: i32 = 4;
const CR_WHERE: i32 = 5;
const CR_WIN_LO: i32 = 6;
const CR_WIN_HI: i32 = 7;
const CR_BUFFER: i32 = 8;
const CR_PRIORITY: i32 = 10;
const CR_KI_COST: i32 = 11;
const TGT_GENO: u32 = 2;
const TGT_AUTO: u32 = 0xFFFF_FFFE;
const TGT_HELPLESS: u32 = 0xFFFF_FFFD;
const COMBO_TGT_ACT: u32 = 12;
const COMBO_FROM_ANY: u32 = 0xFFFF;

const ACTS: [&str; 15] = [
    "special_n", "special_s", "special_hi", "special_lw", "?", "?", "?", "?", "jump", "nair", "fair", "bair", "uair", "dair",
    "jump_short",
];
const INPUT: [&str; 6] = ["light", "heavy", "special", "jump", "grab", "shield"];
const DIR: [&str; 7] = ["any", "neutral", "forward", "back", "up", "down", "side"];
const ON: [&str; 5] = ["any", "hit", "whiff", "block", "contact"];
const WHERE: [&str; 3] = ["any", "ground", "air"];

fn pick(list: &[&'static str], i: i32) -> &'static str {
    if i >= 0 && (i as usize) < list.len() {
        list[i as usize]
    } else {
        "?"
    }
}

/// `gs_combo_target`
fn target_name(g: &dyn crate::ScriptGame, p: i32, t: u32) -> String {
    let (kind, id) = (t >> 28, t & 0xFFFF);
    if t == TGT_AUTO {
        "auto".into()
    } else if t == TGT_HELPLESS {
        "helpless".into()
    } else if kind == TGT_GENO {
        format!("geno:{}", g.combo_state_name(p, id as i32))
    } else if kind == COMBO_TGT_ACT {
        (if id < 15 { ACTS[id as usize] } else { "?" }).into()
    } else if id == COMBO_FROM_ANY {
        "*".into()
    } else {
        format!("motion:{id}")
    }
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    // gd.combo(port [, all]) -> nil without a combo graph, else the table documented in gw_script_combo.inc
    reg(e, gd, "combo", |lua, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let all = a.truthy(2);
        let s = st.borrow();
        let g = &*s.game;
        let n = g.combo_info(slot, 3);
        if n < 0 {
            return Ok(Value::Nil);
        }
        let p = g.combo_info(slot, 11);
        let t = lua.create_table()?;
        t.num("links", g.combo_info(slot, 0) as f64)?;
        t.num("max_links", g.combo_info(slot, 7) as f64)?;
        t.num("ki", g.combo_info(slot, 1) as f64 / 100.0)?;
        t.num("ki_max", g.combo_info(slot, 8) as f64 / 100.0)?;
        t.num("last_route", g.combo_info(slot, 2) as f64)?;
        t.num("taken", g.combo_info(slot, 6) as f64)?;
        t.num("hit", g.combo_info(slot, 4) as f64)?;
        t.num("blocked", g.combo_info(slot, 5) as f64)?;
        t.num("action_frame", g.combo_info(slot, 9) as f64)?;
        t.num("route_count", n as f64)?;
        let last = g.combo_info(slot, 2);
        t.string("last_name", &if last >= 0 { g.combo_name(p, last) } else { String::new() })?;
        let routes = lua.create_table()?;
        let mut k = 0;
        for r in 0..n {
            let stt = g.combo_info(slot, 16 + r);
            if !all && stt <= 0 {
                continue;
            }
            let row = lua.create_table()?;
            row.num("index", r as f64)?;
            row.string("name", &g.combo_name(p, r))?;
            row.string("from", &target_name(g, p, g.combo_word(p, r, CR_FROM) as u32))?;
            row.string("to", &target_name(g, p, g.combo_word(p, r, CR_TO) as u32))?;
            let i = g.combo_word(p, r, CR_INPUT);
            row.string("input", if (0..6).contains(&i) { INPUT[i as usize] } else { "none" })?;
            row.string("dir", pick(&DIR, g.combo_word(p, r, CR_DIR)))?;
            row.string("on", pick(&ON, g.combo_word(p, r, CR_ON)))?;
            row.string("where", pick(&WHERE, g.combo_word(p, r, CR_WHERE)))?;
            row.num("lo", g.combo_word(p, r, CR_WIN_LO) as f64)?;
            row.num("hi", g.combo_word(p, r, CR_WIN_HI) as f64)?;
            row.num("buffer", g.combo_word(p, r, CR_BUFFER) as f64)?;
            row.num("priority", g.combo_word(p, r, CR_PRIORITY) as f64)?;
            row.num("ki_cost", g.combo_word(p, r, CR_KI_COST) as f64 / 100.0)?;
            row.num("state", stt as f64)?;
            k += 1;
            routes.raw_set(k, row)?;
        }
        t.set("routes", routes)?;
        Ok(Value::Table(t))
    })?;
    // gd.combo_ki(port, n) -> the new meter (meter units) | nil
    reg(e, gd, "combo_ki", |_, st, a| {
        let slot = slot_arg(&a, 1)? as i32;
        let n = a.check_number(2)?;
        let mut s = st.borrow_mut();
        s.require_offline("combo_ki")?;
        s.rw_branch();
        let v = s.game.combo_set_ki(slot, (n * 100.0 + 0.5) as i32);
        Ok(if v < 0 { Value::Nil } else { Value::Number(v as f64 / 100.0) })
    })?;
    Ok(())
}
