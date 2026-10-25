//! Scripted CPU controller (`gw_script_cpu_ctl.inc`): cpu_pad, cpu_script, cpu_dest, cpu_target, cpu_script_done,
//! cpu_script_status, cpu_goto, cpu_goto_status, cpu_cancel, cpu_commands, cpu_attrs, cpu_macro. A script writes a
//! CPU's virtual controller (fp->cpu.*), the same controller the retail AI writes. The game half answers
//! `ScriptGame::cpu_ctl(slot, sub, op, a)`; command names and numbers are the retail CpuCmd_* enum.
use super::core::{clampi, field_int};
use super::pad::parse_buttons;
use super::{lua_err, reg, slot_arg, TableExt};
use crate::args::{typename, Args};
use crate::engine::{Engine, Shared};
use crate::game::{SF_FACING, SI_PRESENT};
use mlua::{Table, Value};

type LuaResult<T> = mlua::Result<T>;

const GCC_PAD_COMMIT: i32 = 1;
const GCC_SCRIPT_BEGIN: i32 = 2;
const GCC_SCRIPT_BYTE: i32 = 3;
const GCC_SCRIPT_RUN: i32 = 4;
const GCC_DEST_X: i32 = 5;
const GCC_DEST_Y: i32 = 6;
const GCC_TARGET: i32 = 7;
const GCC_CANCEL: i32 = 8;
const GCC_STAGE: i32 = 10;
const GCC_GOTO_X: i32 = 20;
const GCC_GOTO_Y: i32 = 21;
const GCC_GOTO_RADIUS: i32 = 22;
const GCC_GOTO_START: i32 = 23;
const GCC_G_ACTIVE: i32 = 100;
const GCC_G_RUNNING: i32 = 101;
const GCC_G_DONE_COUNT: i32 = 102;
const GCC_G_PAD_FRAMES: i32 = 103;
const GCC_G_DEST_X: i32 = 104;
const GCC_G_DEST_Y: i32 = 105;
const GCC_G_BUTTONS: i32 = 106;
const GCC_G_LX: i32 = 107;
const GCC_G_LY: i32 = 108;
const GCC_G_CX: i32 = 109;
const GCC_G_CY: i32 = 110;
const GCC_G_LTRIG: i32 = 111;
const GCC_G_RTRIG: i32 = 112;
const GCC_G_TARGET: i32 = 113;
const GCC_G_CSP: i32 = 114;
const GCC_G_DURATION: i32 = 115;
const GCC_G_GOTO_STATE: i32 = 117;
const GCC_G_GOTO_TICKS: i32 = 118;

/// name, code, argument count, argument kind ('s' signed stick/magnitude byte, 'u' 0..255)
const CMDS: &[(&str, i32, usize, char)] = &[
    ("press_a", 1, 0, ' '), ("release_a", 2, 0, ' '), ("press_b", 3, 0, ' '), ("release_b", 4, 0, ' '),
    ("press_x", 5, 0, ' '), ("release_x", 6, 0, ' '), ("press_y", 7, 0, ' '), ("release_y", 8, 0, ' '),
    ("press_r", 9, 0, ' '), ("release_r", 10, 0, ' '), ("press_l", 11, 0, ' '), ("release_l", 12, 0, ' '),
    ("press_z", 13, 0, ' '), ("release_z", 14, 0, ' '), ("press_up", 15, 0, ' '), ("release_up", 16, 0, ' '),
    ("press_down", 17, 0, ' '), ("release_down", 18, 0, ' '), ("press_right", 19, 0, ' '),
    ("release_right", 20, 0, ' '), ("press_left", 21, 0, ' '), ("release_left", 22, 0, ' '),
    ("press_start", 23, 0, ' '), ("release_start", 24, 0, ' '), ("release_all", 25, 0, ' '),
    ("set_lstick_x", 0x80, 1, 's'), ("set_lstick_y", 0x81, 1, 's'),
    ("set_cstick_x", 0x82, 1, 's'), ("set_cstick_y", 0x83, 1, 's'),
    ("set_rtrigger", 0x84, 1, 'u'), ("set_ltrigger", 0x85, 1, 'u'),
    ("press_a_for", 0x86, 1, 'u'), ("release_a_for", 0x87, 1, 'u'),
    ("press_b_for", 0x88, 1, 'u'), ("release_b_for", 0x89, 1, 'u'),
    ("press_x_for", 0x8A, 1, 'u'), ("release_x_for", 0x8B, 1, 'u'),
    ("press_y_for", 0x8C, 1, 'u'), ("release_y_for", 0x8D, 1, 'u'),
    ("wait", 0x8E, 1, 'u'), ("wait_for", 0x8E, 1, 'u'),
    ("lstick_toward_destination", 0x8F, 1, 's'), ("lstick_x_toward_destination", 0x90, 1, 's'),
    ("lstick_x_forward", 0x91, 1, 's'), ("wait_if_motion", 0x92, 1, 'u'),
    ("unknown_0x93", 0x93, 1, 'u'), // retail stores it in fp->cpu.x18; its meaning is unknown
    ("lstick_toward_fighter", 0x94, 1, 's'), ("lstick_x_toward_fighter", 0x95, 1, 's'),
    ("lstick_toward_destination_clamped", 0xC0, 2, 's'),
    ("lstick_x_toward_destination_clamped", 0xC1, 2, 's'),
    ("lstick_forward_clamped", 0xC2, 2, 's'),
];
const MAXBYTES: usize = 255; // the retail buffer is 0x100 bytes and the Done terminator takes one

#[derive(Default)]
struct Buf {
    b: Vec<u8>,
    over: bool,
}
impl Buf {
    fn put(&mut self, v: i32) {
        if self.b.len() >= MAXBYTES {
            self.over = true;
            return;
        }
        self.b.push(v as u8);
    }
    fn b0(&mut self, code: i32) {
        self.put(code);
    }
    fn b1(&mut self, code: i32, a: i32) {
        self.put(code);
        self.put(a);
    }
    fn wait(&mut self, mut frames: i32) {
        while frames > 255 {
            self.b1(0x8E, 255);
            frames -= 255;
        }
        if frames > 0 {
            self.b1(0x8E, frames);
        }
    }
}

fn is_numberish(v: &Value) -> bool {
    match v {
        Value::Integer(_) | Value::Number(_) => true,
        Value::String(s) => s.to_str().is_ok_and(|s| crate::args::str_to_number(&s).is_some()),
        _ => false,
    }
}
fn to_int(v: &Value) -> i32 {
    match v {
        Value::Integer(n) => *n as i32,
        Value::Number(f) => *f as i32,
        Value::String(s) => match crate::args::str_to_number(&s.to_str().map(|s| s.to_string()).unwrap_or_default()) {
            Some(Value::Integer(n)) => n as i32,
            Some(Value::Number(f)) => f as i32,
            _ => 0,
        },
        _ => 0,
    }
}

/// `gs_cpu_compile`: the steps table -> bytes. A step is a command name, or {name, a [, b]}.
fn compile(steps: &Table, out: &mut Buf) -> LuaResult<()> {
    let n = steps.raw_len();
    if n < 1 {
        return Err(lua_err("cpu_script: no steps"));
    }
    for i in 1..=n {
        let step: Value = steps.raw_get(i)?;
        let (name, args): (String, Vec<i32>) = match &step {
            Value::String(s) => (s.to_string_lossy().to_string(), vec![]),
            Value::Table(t) => {
                let first: Value = t.raw_get(1)?;
                let Value::String(nm) = first else {
                    return Err(lua_err(format!("cpu_script: step {i}: a command name first")));
                };
                let mut args = Vec::new();
                for na in 0..2 {
                    let v: Value = t.raw_get(na + 2)?;
                    if v.is_nil() {
                        break;
                    }
                    if !is_numberish(&v) {
                        return Err(lua_err(format!("cpu_script: step {i}: arguments are numbers")));
                    }
                    args.push(to_int(&v));
                }
                let extra: Value = t.raw_get(5)?;
                if !extra.is_nil() {
                    return Err(lua_err(format!("cpu_script: step {i}: at most two arguments")));
                }
                (nm.to_string_lossy().to_string(), args)
            }
            _ => return Err(lua_err(format!("cpu_script: step {i}: a command name or {{name, args}}"))),
        };
        let Some(c) = CMDS.iter().find(|c| c.0 == name) else {
            return Err(lua_err(format!("cpu_script: step {i}: unknown command \"{name}\" (gd.cpu_commands() lists them)")));
        };
        if args.len() != c.2 {
            return Err(lua_err(format!("cpu_script: step {i}: {name} takes {} argument(s)", c.2)));
        }
        for a in &args {
            let (lo, hi) = if c.3 == 's' { (-127, 127) } else { (0, 255) };
            if *a < lo || *a > hi {
                return Err(lua_err(format!("cpu_script: step {i}: {name} argument {lo}..{hi}, got {a}")));
            }
        }
        out.put(c.1);
        for a in &args {
            out.put(*a);
        }
        if out.over {
            return Err(lua_err(format!("cpu_script: more than {MAXBYTES} bytes (the retail buffer); split it in two")));
        }
    }
    Ok(())
}

fn fbits(f: f32) -> i32 {
    f.to_bits() as i32
}
fn flt(i: i32) -> f32 {
    f32::from_bits(i as u32)
}
/// `gs_cpu_target_check`
fn target_check(s: &crate::engine::State, f: &str, slot: usize, sub: bool) -> LuaResult<()> {
    let act = s.game.cpu_ctl_read(slot as i32, sub as i32, GCC_G_ACTIVE, 0);
    if act < 0 {
        return Err(lua_err(format!("{f}: P{} has no {} CPU fighter", slot + 1, if sub { "sub-fighter" } else { "live" })));
    }
    if act == 0 {
        return Err(lua_err(format!("{f}: P{} is not in script mode (gd.cpu_mode({}, \"script\"))", slot + 1, slot + 1)));
    }
    Ok(())
}
fn sub_opt(a: &Args, i: usize) -> LuaResult<bool> {
    if let Value::Table(t) = a.get(i) {
        let v: Value = t.get("sub")?;
        return Ok(!matches!(v, Value::Nil | Value::Boolean(false)));
    }
    Ok(false)
}
fn bool_arg(a: &Args, i: usize) -> bool {
    a.truthy(i)
}

fn cpu_pad(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let t = a.check_table(2)?;
    let frames = a.opt_integer(3, 1)? as i32;
    if frames < 1 {
        return Err(lua_err("cpu_pad: frames must be at least 1"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("cpu_pad")?;
    let sub = sub_opt(&a, 2)?;
    let bv: Value = t.get("buttons")?;
    let buttons = parse_buttons(&Args::new("cpu_pad", mlua::MultiValue::from_iter([bv])), 1)? as i32;
    let v = [
        buttons,
        clampi(field_int(&t, "x", 0), -127, 127),
        clampi(field_int(&t, "y", 0), -127, 127),
        clampi(field_int(&t, "cx", 0), -127, 127),
        clampi(field_int(&t, "cy", 0), -127, 127),
        clampi(field_int(&t, "l", 0), 0, 255),
        clampi(field_int(&t, "r", 0), 0, 255),
    ];
    target_check(&s, "cpu_pad", slot, sub)?;
    s.rw_branch();
    for (k, x) in v.iter().enumerate() {
        s.game.cpu_ctl(slot as i32, sub as i32, GCC_STAGE + k as i32, *x);
    }
    Ok(s.game.cpu_ctl(slot as i32, sub as i32, GCC_PAD_COMMIT, frames) != 0)
}

fn load(s: &mut crate::engine::State, f: &str, slot: usize, sub: bool, buf: &Buf, hold: bool) -> LuaResult<bool> {
    s.require_offline(f)?;
    target_check(s, f, slot, sub)?;
    s.rw_branch();
    let (sl, sb) = (slot as i32, sub as i32);
    s.game.cpu_ctl(sl, sb, GCC_SCRIPT_BEGIN, 0);
    let mut ok = true;
    for byte in &buf.b {
        if !ok {
            break;
        }
        ok = s.game.cpu_ctl(sl, sb, GCC_SCRIPT_BYTE, *byte as i32) != 0;
    }
    if ok {
        ok = s.game.cpu_ctl(sl, sb, GCC_SCRIPT_RUN, if hold { 0 } else { 1 }) != 0;
    }
    Ok(ok)
}

fn cpu_script(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let steps = a.check_table(2)?;
    let mut buf = Buf::default();
    compile(&steps, &mut buf)?;
    let mut hold = false;
    if let Value::Table(t) = a.get(3) {
        let v: Value = t.get("hold")?;
        hold = !matches!(v, Value::Nil | Value::Boolean(false));
    }
    let sub = sub_opt(&a, 3)?;
    load(&mut st.borrow_mut(), "cpu_script", slot, sub, &buf, hold)
}

fn cpu_dest(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<mlua::MultiValue> {
    let slot = slot_arg(&a, 1)?;
    let sub = sub_opt(&a, 4)?;
    if a.none_or_nil(2) {
        let s = st.borrow();
        if s.game.cpu_ctl_read(slot as i32, sub as i32, GCC_G_ACTIVE, 0) < 0 {
            return Err(lua_err(format!("cpu_dest: P{} has no such CPU fighter", slot + 1)));
        }
        let (x, y) = (s.game.cpu_ctl_read(slot as i32, sub as i32, GCC_G_DEST_X, 0), s.game.cpu_ctl_read(slot as i32, sub as i32, GCC_G_DEST_Y, 0));
        return Ok(mlua::MultiValue::from_iter([Value::Number(flt(x) as f64), Value::Number(flt(y) as f64)]));
    }
    let (x, y) = (a.check_number(2)?, a.check_number(3)?);
    if !(x.abs() <= 49000.0 && y.abs() <= 49000.0) {
        return Err(lua_err("cpu_dest: finite x, y within +-49000"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("cpu_dest")?;
    target_check(&s, "cpu_dest", slot, sub)?;
    s.rw_branch();
    s.game.cpu_ctl(slot as i32, sub as i32, GCC_DEST_X, fbits(x as f32));
    s.game.cpu_ctl(slot as i32, sub as i32, GCC_DEST_Y, fbits(y as f32));
    Ok(mlua::MultiValue::from_iter([Value::Boolean(true)]))
}

fn cpu_target(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let other = if a.none_or_nil(2) { -1 } else { slot_arg(&a, 2)? as i32 };
    let mut s = st.borrow_mut();
    s.require_offline("cpu_target")?;
    target_check(&s, "cpu_target", slot, false)?;
    s.rw_branch();
    s.game.cpu_ctl(slot as i32, 1, GCC_TARGET, other); // Nana, when there is one
    Ok(s.game.cpu_ctl(slot as i32, 0, GCC_TARGET, other) != 0)
}

fn cpu_script_done(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)? as i32;
    let sub = bool_arg(&a, 2) as i32;
    let s = st.borrow();
    let g = &*s.game;
    let act = g.cpu_ctl_read(slot, sub, GCC_G_ACTIVE, 0);
    if act < 0 {
        return Err(lua_err(format!("cpu_script_done: P{} has no such CPU fighter", slot + 1)));
    }
    Ok(act == 1
        && g.cpu_ctl_read(slot, sub, GCC_G_RUNNING, 0) == 0
        && g.cpu_ctl_read(slot, sub, GCC_G_PAD_FRAMES, 0) == 0
        && g.cpu_ctl_read(slot, sub, GCC_G_GOTO_STATE, 0) != 1)
}

fn cpu_script_status(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let slot = slot_arg(&a, 1)? as i32;
    let sub = bool_arg(&a, 2) as i32;
    let s = st.borrow();
    let g = &*s.game;
    let act = g.cpu_ctl_read(slot, sub, GCC_G_ACTIVE, 0);
    if act < 0 {
        return Err(lua_err(format!("cpu_script_status: P{} has no such CPU fighter", slot + 1)));
    }
    let r = |op| g.cpu_ctl_read(slot, sub, op, 0);
    let t = lua.create_table()?;
    t.boolean("script_mode", act == 1)?;
    t.boolean("running", r(GCC_G_RUNNING) != 0)?;
    t.int("completed", r(GCC_G_DONE_COUNT) as i64)?;
    t.int("pad_frames", r(GCC_G_PAD_FRAMES) as i64)?;
    t.int("buttons", r(GCC_G_BUTTONS) as i64)?;
    t.int("x", r(GCC_G_LX) as i64)?;
    t.int("y", r(GCC_G_LY) as i64)?;
    t.int("cx", r(GCC_G_CX) as i64)?;
    t.int("cy", r(GCC_G_CY) as i64)?;
    t.int("l", r(GCC_G_LTRIG) as i64)?;
    t.int("r", r(GCC_G_RTRIG) as i64)?;
    t.int("target", (r(GCC_G_TARGET) + 1) as i64)?; // 0 = first other
    t.int("script_pos", r(GCC_G_CSP) as i64)?; // -1 = none
    t.int("wait_left", r(GCC_G_DURATION) as i64)?;
    Ok(t)
}

fn cpu_goto(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let sub = sub_opt(&a, 4)?;
    let (x, y) = (a.check_number(2)?, a.check_number(3)?);
    let (mut radius, mut timeout) = (2.0f64, 900i32);
    if !(x.abs() <= 49000.0 && y.abs() <= 49000.0) {
        return Err(lua_err("cpu_goto: finite x, y within +-49000"));
    }
    if let Value::Table(t) = a.get(4) {
        let r: Value = t.get("radius")?;
        if is_numberish(&r) {
            radius = match r {
                Value::Integer(n) => n as f64,
                Value::Number(f) => f,
                other => crate::args::str_to_number(&other.to_string().unwrap_or_default()).map_or(radius, |v| match v {
                    Value::Integer(n) => n as f64,
                    Value::Number(f) => f,
                    _ => radius,
                }),
            };
        }
        timeout = field_int(t, "timeout", 900);
    }
    if !(0.5..=200.0).contains(&radius) {
        return Err(lua_err("cpu_goto: radius 0.5..200"));
    }
    if !(1..=36000).contains(&timeout) {
        return Err(lua_err("cpu_goto: timeout 1..36000 frames"));
    }
    let mut s = st.borrow_mut();
    s.require_offline("cpu_goto")?;
    target_check(&s, "cpu_goto", slot, sub)?;
    s.rw_branch();
    let (sl, sb) = (slot as i32, sub as i32);
    s.game.cpu_ctl(sl, sb, GCC_GOTO_X, fbits(x as f32));
    s.game.cpu_ctl(sl, sb, GCC_GOTO_Y, fbits(y as f32));
    s.game.cpu_ctl(sl, sb, GCC_GOTO_RADIUS, fbits(radius as f32));
    Ok(s.game.cpu_ctl(sl, sb, GCC_GOTO_START, timeout) != 0)
}

fn cpu_goto_status(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<mlua::MultiValue> {
    const NAMES: [&str; 5] = ["idle", "running", "arrived", "blocked", "fell"];
    let slot = slot_arg(&a, 1)? as i32;
    let sub = bool_arg(&a, 2) as i32;
    let s = st.borrow();
    let state = s.game.cpu_ctl_read(slot, sub, GCC_G_GOTO_STATE, 0);
    if state < 0 {
        return Err(lua_err(format!("cpu_goto_status: P{} has no such CPU fighter", slot + 1)));
    }
    let ticks = s.game.cpu_ctl_read(slot, sub, GCC_G_GOTO_TICKS, 0);
    Ok(mlua::MultiValue::from_iter([
        Value::String(lua.create_string(NAMES[if state > 4 { 0 } else { state as usize }])?),
        Value::Integer(ticks as i64),
    ]))
}

fn cpu_cancel(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<bool> {
    let slot = slot_arg(&a, 1)?;
    let sub = bool_arg(&a, 2);
    let mut s = st.borrow_mut();
    s.require_offline("cpu_cancel")?;
    target_check(&s, "cpu_cancel", slot, sub)?;
    s.rw_branch();
    Ok(s.game.cpu_ctl(slot as i32, sub as i32, GCC_CANCEL, 0) != 0)
}

fn cpu_commands(lua: &mlua::Lua, _: &Shared, _a: Args) -> LuaResult<Table> {
    let t = lua.create_table()?;
    for c in CMDS.iter() {
        let e = lua.create_table()?;
        e.int("code", c.1 as i64)?;
        e.int("args", c.2 as i64)?;
        t.set(c.0, e)?;
    }
    Ok(t)
}

// ---- technique macros, as command sequences timed from the fighter's own attributes
const CA_JUMP_SQUAT: i32 = 0;
const CA_HOP_V: i32 = 1;
const CA_GRAVITY: i32 = 3;
const CA_TERMINAL: i32 = 4;
const CA_TECH_WINDOW: i32 = 23;
const CA_COUNT: i32 = 24;
const ATTR_NAMES: [&str; 24] = [
    "jump_squat", "hop_v", "jump_v", "gravity", "terminal_velocity", "landing_nair", "landing_fair", "landing_bair",
    "landing_uair", "landing_dair", "landing_normal", "dash_initial", "dash_max", "walk_max", "max_jumps", "fast_fall",
    "air_max_horizontal", "ground_friction", "ground_to_air_momentum", "jump_h_initial", "airdodge_landing",
    "air_drift_max", "shield_size", "tech_window",
];

fn cpu_attrs(lua: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<Table> {
    let slot = slot_arg(&a, 1)? as i32;
    let s = st.borrow();
    let t = lua.create_table()?;
    if s.game.fighter_i(slot, SI_PRESENT) != 1 {
        return Ok(t);
    }
    for i in 0..CA_COUNT {
        t.num(ATTR_NAMES[i as usize], flt(s.game.cpu_attr(slot, i)) as f64)?;
    }
    Ok(t)
}

/// frames a short hop stays in the air: the fighter's own hop velocity, gravity and terminal velocity, stepped as the engine steps them
fn hop_air_frames(v0: f32, g: f32, term: f32) -> i32 {
    let (mut y, mut v, mut n) = (0.0f32, v0, 0);
    loop {
        y += v;
        v -= g;
        if v < -term {
            v = -term;
        }
        n += 1;
        if !(y > 0.0 && n < 240) {
            break;
        }
    }
    n
}
const WD_OFFSET: i32 = 0; // airdodge input frame, relative to the end of the jump squat
const LAND_DELAY: i32 = 2; // ticks from the height reaching zero to the landing action
const LC_LEAD: i32 = 3; // L-cancel press this many frames before the predicted landing
const AERIAL_AT: i32 = 2; // aerial input this many frames after take-off

#[derive(Clone, Copy)]
struct Dir {
    sign: i32,
    rel: bool,
}
/// `gs_cpu_dir`: "forward"/"back" follow the facing when the command runs; "left"/"right"/-1/1 are absolute
fn parse_dir(v: &Value, f: &str) -> LuaResult<Dir> {
    let d = Dir { sign: 1, rel: true };
    match v {
        Value::Nil => return Ok(d),
        Value::String(s) => match &*s.to_string_lossy() {
            "in" | "forward" => return Ok(d),
            "back" => return Ok(Dir { sign: -1, rel: true }),
            "left" => return Ok(Dir { sign: -1, rel: false }),
            "right" => return Ok(Dir { sign: 1, rel: false }),
            _ => {}
        },
        Value::Integer(n) if *n == 1 || *n == -1 => return Ok(Dir { sign: *n as i32, rel: false }),
        _ => {}
    }
    Err(lua_err(format!("{f}: dir is \"forward\", \"back\", \"left\", \"right\", -1 or 1")))
}
fn stick_x(p: &mut Buf, d: Dir, mag: i32) {
    if d.rel {
        p.b1(0x91, d.sign * mag);
    } else {
        p.b1(0x80, d.sign * mag);
    }
}
fn airdodge(p: &mut Buf, d: Dir, deg: i32) {
    let rad = deg as f64 * 3.14159265358979 / 180.0;
    let x = (127.0 * rad.cos() + 0.5) as i32;
    let y = (-127.0 * rad.sin() - 0.5) as i32;
    stick_x(p, d, x);
    p.b1(0x81, y);
    p.b0(11); // press L
    p.b1(0x8E, 2);
    p.b0(12);
}

struct Ctx<'a> {
    opts: Option<&'a Table>,
}
impl Ctx<'_> {
    fn opt(&self, k: &str, def: i32) -> i32 {
        self.opts.map_or(def, |t| field_int(t, k, def))
    }
}

/// macros: short_hop, full_hop, dash, dash_dance, wavedash, waveland, lcancel_aerial, shield, perfect_shield, tech,
/// jump_cancel_grab. Ok(false) for a name it does not know.
fn macro_build(s: &crate::engine::State, slot: usize, name: &str, opts: Option<&Table>, p: &mut Buf) -> LuaResult<bool> {
    let g = &*s.game;
    let sl = slot as i32;
    let attr = |id: i32| flt(g.cpu_attr(sl, id));
    let mut j = (attr(CA_JUMP_SQUAT) + 0.5) as i32;
    let mut dir = Dir { sign: 1, rel: true };
    if let Some(t) = opts {
        let v: Value = t.get("dir")?;
        if !v.is_nil() {
            dir = parse_dir(&v, "cpu_macro")?;
        }
    }
    if g.fighter_i(sl, SI_PRESENT) != 1 {
        return Err(lua_err(format!("cpu_macro: P{} has no fighter", slot + 1)));
    }
    if j < 1 {
        j = 1;
    }
    let c = Ctx { opts };
    match name {
        "short_hop" => {
            p.b0(5);
            p.b1(0x8E, 1);
            p.b0(6);
            p.wait(j);
        }
        "full_hop" => {
            p.b0(5);
            p.wait(j + 2);
            p.b0(6);
        }
        "dash" => {
            stick_x(p, dir, 127);
            p.wait(c.opt("frames", 12));
            p.b1(0x80, 0);
        }
        "dash_dance" => {
            let cycles = clampi(c.opt("cycles", 3), 1, 12);
            let hold = clampi(c.opt("hold", 6), 1, 40);
            for _ in 0..cycles {
                stick_x(p, dir, 127);
                p.wait(hold);
                stick_x(p, dir, -127);
                p.wait(hold);
            }
            p.b1(0x80, 0);
        }
        "wavedash" => {
            p.b0(5);
            p.b1(0x8E, 1);
            p.b0(6);
            p.wait(j - 1 + c.opt("offset", WD_OFFSET));
            airdodge(p, dir, clampi(c.opt("angle", 20), 5, 80));
            p.b1(0x81, 0);
            p.b1(0x80, 0);
        }
        "waveland" => {
            airdodge(p, dir, clampi(c.opt("angle", 20), 5, 80));
            p.b1(0x81, 0);
            p.b1(0x80, 0);
        }
        "lcancel_aerial" => {
            // ticks are the macro's own: the first command runs at tick 1 (the jump starts on it), the fighter is first
            // airborne on tick 1+J, and the engine lands it LAND_DELAY ticks after the integrated height first reaches zero
            let mut aerial = "nair".to_string();
            let sim = hop_air_frames(attr(CA_HOP_V), attr(CA_GRAVITY), attr(CA_TERMINAL));
            let landing = j + sim + LAND_DELAY;
            let lcancel = c.opt("lcancel", 1);
            let at = clampi(c.opt("at", AERIAL_AT), 0, 60);
            let lead = clampi(c.opt("lead", LC_LEAD), 1, 20);
            let face = if g.fighter_f(sl, SF_FACING) < 0.0 { -1 } else { 1 };
            if let Some(t) = opts {
                if let Value::String(a) = t.get::<Value>("aerial")? {
                    aerial = a.to_string_lossy().to_string();
                }
            }
            p.b0(5);
            p.b1(0x8E, 1);
            p.b0(6);
            let t0 = 2;
            p.wait(1 + j + at - t0);
            let mut t = 1 + j + at;
            if aerial == "nair" {
                p.b0(1);
                p.b1(0x8E, 1);
                p.b0(2);
            } else {
                let (mut cx, mut cy) = (0, 0);
                match aerial.as_str() {
                    "fair" => cx = face * dir.sign * 127,
                    "bair" => cx = -face * dir.sign * 127,
                    "uair" => cy = 127,
                    "dair" => cy = -127,
                    _ => return Err(lua_err("cpu_macro lcancel_aerial: aerial is nair, fair, bair, uair or dair")),
                }
                p.b1(0x82, cx);
                p.b1(0x83, cy);
                p.b1(0x8E, 1);
                p.b1(0x82, 0);
                p.b1(0x83, 0);
            }
            t += 1;
            if landing - lead > t {
                p.wait(landing - lead - t);
            }
            if lcancel != 0 {
                p.b0(11);
                p.b1(0x8E, 2);
                p.b0(12);
            }
        }
        "shield" => {
            p.b0(11);
            p.wait(clampi(c.opt("frames", 30), 1, 600));
            p.b0(12);
        }
        "perfect_shield" => {
            // press shield `at` frames from now (the caller times it to the incoming hit), hold briefly
            p.wait(clampi(c.opt("at", 0), 0, 600));
            p.b0(11);
            p.wait(clampi(c.opt("hold", 12), 1, 600));
            p.b0(12);
        }
        "tech" => {
            // shield press toward dir; `at` frames from now. dir "in" = no stick (tech in place). The game reads the stick when it
            // LANDS the fighter, up to its tech window after the shield press, so the stick is held for the whole window.
            let tech_in = match opts {
                Some(t) => matches!(t.get::<Value>("dir")?, Value::String(s) if s.to_string_lossy() == "in"),
                None => false,
            };
            let win = clampi(c.opt("window", (attr(CA_TECH_WINDOW) + 0.5) as i32), 3, 120);
            p.wait(clampi(c.opt("at", 0), 0, 600));
            if !tech_in {
                stick_x(p, dir, 127);
            }
            p.b0(11);
            p.b1(0x8E, 3);
            p.b0(12);
            if !tech_in {
                p.wait(win - 3);
                p.b1(0x80, 0);
            }
        }
        "jump_cancel_grab" => {
            p.b0(5);
            p.wait(clampi(c.opt("z_at", 1), 1, if j > 1 { j - 1 } else { 1 }));
            p.b0(13);
            p.b1(0x8E, 2);
            p.b0(14);
            p.b0(6);
        }
        _ => return Ok(false),
    }
    if p.over {
        return Err(lua_err("cpu_macro: macro too long"));
    }
    Ok(true)
}

fn cpu_macro(_: &mlua::Lua, st: &Shared, a: Args) -> LuaResult<mlua::MultiValue> {
    let slot = slot_arg(&a, 1)?;
    let name = a.check_string(2)?;
    if !a.none_or_nil(3) {
        a.check_table(3)?;
    }
    let opts = if let Value::Table(t) = a.get(3) { Some(t.clone()) } else { None };
    let mut buf = Buf::default();
    let mut s = st.borrow_mut();
    if !macro_build(&s, slot, &name, opts.as_ref(), &mut buf)? {
        return Err(lua_err(format!(
            "cpu_macro: unknown macro \"{name}\" (short_hop full_hop dash dash_dance wavedash waveland lcancel_aerial shield perfect_shield tech jump_cancel_grab)"
        )));
    }
    let mut hold = false;
    if let Some(t) = &opts {
        let v: Value = t.get("hold")?;
        hold = !matches!(v, Value::Nil | Value::Boolean(false));
    }
    let sub = sub_opt(&a, 3)?;
    let ok = load(&mut s, "cpu_macro", slot, sub, &buf, hold)?;
    Ok(mlua::MultiValue::from_iter([Value::Boolean(ok), Value::Integer(buf.b.len() as i64)]))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    reg(e, gd, "cpu_pad", cpu_pad)?;
    reg(e, gd, "cpu_script", cpu_script)?;
    reg(e, gd, "cpu_dest", cpu_dest)?;
    reg(e, gd, "cpu_target", cpu_target)?;
    reg(e, gd, "cpu_script_done", cpu_script_done)?;
    reg(e, gd, "cpu_script_status", cpu_script_status)?;
    reg(e, gd, "cpu_goto", cpu_goto)?;
    reg(e, gd, "cpu_goto_status", cpu_goto_status)?;
    reg(e, gd, "cpu_cancel", cpu_cancel)?;
    reg(e, gd, "cpu_commands", cpu_commands)?;
    reg(e, gd, "cpu_attrs", cpu_attrs)?;
    reg(e, gd, "cpu_macro", cpu_macro)?;
    Ok(())
}

#[allow(dead_code)]
fn _u(a: &Args) -> &'static str {
    typename(Some(a.get(1)))
}
