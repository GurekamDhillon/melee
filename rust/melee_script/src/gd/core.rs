//! Core `gd.*`: logging, clocks, drawing, keys, commands, tasks, per-script data files, pause control.
use super::{lua_err, reg, TableExt};
use crate::args::Args;
use crate::engine::{Command, Draw, DrawKind, Engine, Task, GS_MAX_COMMANDS, GS_MAX_DRAW, GS_MAX_TASKS};
use mlua::{Lua, MultiValue, Result as LuaResult, Table, Value};
use std::io::Write;

/// `gs_clampi`
pub fn clampi(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

/// `gs_field_int`: an integer field of a table, `def` when absent or not a number.
pub fn field_int(a: &Table, k: &str, def: i32) -> i32 {
    match a.get::<Value>(k) {
        Ok(Value::Integer(n)) => n as i32,
        Ok(Value::Number(n)) => n as i32,
        Ok(Value::String(s)) => match crate::args::str_to_number(&s.to_str().map(|s| s.to_string()).unwrap_or_default()) {
            Some(Value::Integer(n)) => n as i32,
            Some(Value::Number(n)) => n as i32,
            _ => def,
        },
        _ => def,
    }
}

/// `gs_color_arg`: 0xRRGGBBAA integer, or {r=,g=,b=,a=}; `def` when absent.
pub fn color_arg(a: &Args, i: usize, def: u32) -> LuaResult<u32> {
    if a.none_or_nil(i) {
        return Ok(def);
    }
    if a.is_integer(i) {
        return Ok(a.check_integer(i)? as u32);
    }
    if a.is_table(i) {
        let t = a.check_table(i)?;
        let c = |k| clampi(field_int(&t, k, 255), 0, 255) as u32;
        return Ok((c("r") << 24) | (c("g") << 16) | (c("b") << 8) | c("a"));
    }
    Err(lua_err("colours are 0xRRGGBBAA numbers (gd.rgb(r, g, b [, a])) or {r=,g=,b=,a=}"))
}

/// `gs_vk`: the Windows virtual-key code of a key name.
pub fn vk(name: &str) -> LuaResult<usize> {
    const NAMED: &[(&str, usize)] = &[
        ("SPACE", 0x20), ("ENTER", 0x0D), ("TAB", 0x09), ("ESCAPE", 0x1B), ("BACKSPACE", 0x08), ("SHIFT", 0x10),
        ("CTRL", 0x11), ("ALT", 0x12), ("LEFT", 0x25), ("RIGHT", 0x27), ("UP", 0x26), ("DOWN", 0x28),
        ("HOME", 0x24), ("END", 0x23), ("PAGEUP", 0x21), ("PAGEDOWN", 0x22), ("INSERT", 0x2D), ("DELETE", 0x2E),
        ("PERIOD", 0xBE), ("COMMA", 0xBC), ("MINUS", 0xBD), ("PLUS", 0xBB), ("SLASH", 0xBF), ("SEMICOLON", 0xBA),
        ("GRAVE", 0xC0), ("LBRACKET", 0xDB), ("RBRACKET", 0xDC), ("BACKSLASH", 0xDD), ("QUOTE", 0xDE),
    ];
    let n = name.len();
    if n == 0 || n >= 16 {
        return Err(lua_err(format!("unknown key \"{name}\"")));
    }
    let up = name.to_ascii_uppercase();
    let b = up.as_bytes();
    if n == 1 && (b[0].is_ascii_uppercase() || b[0].is_ascii_digit()) {
        return Ok(b[0] as usize);
    }
    if b[0] == b'F' && (2..=3).contains(&n) {
        let f = crate::manifest::atoi(&up[1..]);
        if (1..=12).contains(&f) {
            return Ok(0x70 + f as usize - 1);
        }
    }
    if b[0] == b'K' && n >= 2 && b[1] == b'P' && n == 3 && b[2].is_ascii_digit() {
        return Ok(0x60 + (b[2] - b'0') as usize);
    }
    if let Some((_, v)) = NAMED.iter().find(|(k, _)| *k == up) {
        return Ok(*v);
    }
    Err(lua_err(format!(
        "unknown key \"{name}\" (A-Z 0-9 F1-F12 KP0-KP9 SPACE ENTER SHIFT CTRL ALT arrows ...)"
    )))
}

/// `gs_data_path`: the validated absolute path of a script data file; creates the folders.
pub fn data_path(st: &crate::engine::Shared, name: &str) -> LuaResult<std::path::PathBuf> {
    let s = st.borrow();
    let Some(sc) = s.cur_script() else { return Err(lua_err("no current script")) };
    if name.is_empty() || name.len() > 96 {
        return Err(lua_err("data file names are 1-96 characters"));
    }
    let b = name.as_bytes();
    for (i, &c) in b.iter().enumerate() {
        let slash_ok = c == b'/' && i != 0 && i + 1 < b.len() && b[i + 1] != b'/';
        let ok = c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || slash_ok || (c == b'.' && i != 0 && b[i - 1] != b'/');
        if !ok {
            return Err(lua_err(format!("data file names use letters, digits, _ - . and / (got \"{name}\")")));
        }
    }
    let id: String = sc.id.chars().map(|c| if c == '/' || c == '\\' { '_' } else { c }).collect();
    let dir = s.data_dir.join(&id);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Ok(path)
}

fn fail(lua: &Lua, why: &str) -> LuaResult<MultiValue> {
    Ok(MultiValue::from_iter([Value::Boolean(false), Value::String(lua.create_string(why)?)]))
}

pub fn register(e: &Engine, gd: &Table) -> LuaResult<()> {
    gd.set("log", e.make_log("log")?)?;

    reg(e, gd, "frame", |_, st, _| Ok(st.borrow().frame as i64))?;
    reg(e, gd, "time", |_, st, _| Ok(st.borrow().host.borrow().now_ms() / 1000.0))?;
    reg(e, gd, "rng", |_, st, _| Ok(st.borrow().game.rng_seed() as i64))?;
    reg(e, gd, "paused", |_, st, _| Ok(st.borrow().paused))?;
    reg(e, gd, "pause", |_, st, _| {
        let mut s = st.borrow_mut();
        s.require_gameplay("pause")?;
        s.paused = true;
        Ok(())
    })?;
    reg(e, gd, "resume", |_, st, _| {
        let mut s = st.borrow_mut();
        s.require_gameplay("resume")?;
        s.paused = false;
        s.step = 0;
        Ok(())
    })?;
    reg(e, gd, "step", |_, st, a| {
        let n = a.opt_integer(1, 1)? as i32;
        let mut s = st.borrow_mut();
        s.require_gameplay("step")?;
        s.paused = true;
        s.step += if n < 1 { 1 } else { n };
        Ok(())
    })?;

    reg(e, gd, "rgb", |_, _, a| {
        let r = clampi(a.check_integer(1)? as i32, 0, 255) as i64;
        let g = clampi(a.check_integer(2)? as i32, 0, 255) as i64;
        let b = clampi(a.check_integer(3)? as i32, 0, 255) as i64;
        let al = clampi(a.opt_integer(4, 255)? as i32, 0, 255) as i64;
        Ok((r << 24) | (g << 16) | (b << 8) | al)
    })?;
    reg(e, gd, "label", |_, st, a| {
        let text = a.opt_string(1, "")?;
        st.borrow().host.borrow_mut().set_run_label(&text);
        Ok(())
    })?;
    reg(e, gd, "quit", |_, st, _| {
        let mut s = st.borrow_mut();
        s.require_gameplay("quit")?;
        let id = s.script_id(s.cur);
        s.log(&format!("script [{id}]: quit"));
        s.host.borrow_mut().request_quit();
        Ok(())
    })?;
    reg(e, gd, "safe_area", |lua, st, _| {
        let w = st.borrow().host.borrow().canvas_width() as f64;
        let t = lua.create_table()?;
        t.num("x", 0.0)?;
        t.num("y", 0.0)?;
        t.num("w", w)?;
        t.num("h", 480.0)?;
        t.num("right", w)?;
        t.num("bottom", 480.0)?;
        Ok(t)
    })?;

    // ---- drawing
    reg(e, gd, "text", |lua, st, a| {
        let x = a.check_number(1)? as f32;
        let y = a.check_number(2)? as f32;
        // the colour and size before the text is stringified (index 4 would be the text otherwise)
        let rgba = color_arg(&a, 4, 0xFFFFFFFF)?;
        let size = a.opt_number(5, 1.0)? as f32;
        let text = {
            let f: mlua::Function = lua.globals().get("tostring")?;
            f.call::<mlua::LuaString>(a.get(3).clone())?.to_string_lossy().to_string()
        };
        let mut s = st.borrow_mut();
        let mut t = text;
        if t.len() > 159 {
            let mut end = 159;
            while !t.is_char_boundary(end) {
                end -= 1;
            }
            t.truncate(end);
        }
        draw_new(&mut s, Draw { kind: DrawKind::Text, x, y, w: 0.0, h: 0.0, rgba, size, text: t });
        Ok(())
    })?;
    for (name, kind) in [("box", DrawKind::Box), ("fill", DrawKind::Fill), ("line", DrawKind::Line)] {
        reg(e, gd, name, move |_, st, a| {
            let x = a.check_number(1)? as f32;
            let y = a.check_number(2)? as f32;
            let w = a.check_number(3)? as f32;
            let h = a.check_number(4)? as f32;
            let def = if kind == DrawKind::Fill { 0x000000A0 } else { 0xFFFFFFFF };
            let rgba = color_arg(&a, 5, def)?;
            let mut s = st.borrow_mut();
            draw_new(&mut s, Draw { kind, x, y, w, h, rgba, size: 1.0, text: String::new() });
            Ok(())
        })?;
    }

    // ---- keyboard / mouse
    reg(e, gd, "key", |_, st, a| {
        let v = vk(&a.check_string(1)?)?;
        Ok(st.borrow().key_now[v])
    })?;
    reg(e, gd, "key_pressed", |_, st, a| {
        let v = vk(&a.check_string(1)?)?;
        let s = st.borrow();
        Ok(s.key_now[v] && !s.key_prev[v])
    })?;
    reg(e, gd, "mouse", |_, st, _| {
        let (x, y, b, w) = st.borrow().host.borrow().mouse();
        Ok((x as f64, y as f64, b as i64, w as f64))
    })?;

    // ---- console commands registered by scripts
    reg(e, gd, "command", |_, st, a| {
        let name = a.check_string(1)?;
        let help = a.opt_string(3, "")?;
        let func = a.check_function(2)?;
        let mut s = st.borrow_mut();
        let cur = s.cur.unwrap_or(0);
        let cmd = Command { name: name.clone(), help, func, script: cur };
        match s.commands.iter().position(|c| c.name.eq_ignore_ascii_case(&name)) {
            Some(i) => s.commands[i] = cmd,
            None => {
                if s.commands.len() >= GS_MAX_COMMANDS {
                    return Err(lua_err("too many console commands"));
                }
                s.commands.push(cmd);
            }
        }
        Ok(())
    })?;

    // ---- tasks
    reg(e, gd, "run", |lua, st, a| {
        let f = a.check_function(1)?;
        let mut s = st.borrow_mut();
        let Some(cur) = s.cur.filter(|&c| c < s.scripts.len()) else {
            return Err(lua_err("gd.run: no current script"));
        };
        let console = s.console;
        let client = s.client_owner;
        let Some(k) = s.scripts[cur].tasks.iter().position(|t| t.is_none()) else {
            return Err(lua_err(format!("gd.run: at most {GS_MAX_TASKS} tasks per script")));
        };
        let thread = lua.create_thread(f)?;
        s.scripts[cur].tasks[k] = Some(Task {
            thread,
            wait: 0,
            pad_owner: (crate::engine::GS_MAX_SCRIPTS + crate::engine::GS_CLIENTS + 1 + cur * GS_MAX_TASKS + k) as i32,
            client_owner: if cur == console { client } else { 0 },
        });
        Ok((k + 1) as i64)
    })?;

    // ---- per-script data
    reg(e, gd, "data_exists", |lua, st, a| {
        let p = data_path(st, &a.check_string(1)?)?;
        match std::fs::metadata(&p) {
            Ok(_) => Ok(MultiValue::from_iter([Value::Boolean(true)])),
            Err(err) if matches!(err.kind(), std::io::ErrorKind::NotFound) => Ok(MultiValue::from_iter([Value::Boolean(false)])),
            Err(err) => Ok(MultiValue::from_iter([Value::Nil, Value::String(lua.create_string(err.to_string())?)])),
        }
    })?;
    reg(e, gd, "data_read", |lua, st, a| {
        let p = data_path(st, &a.check_string(1)?)?;
        let nil = |why: &str| -> LuaResult<MultiValue> {
            Ok(MultiValue::from_iter([Value::Nil, Value::String(lua.create_string(why)?)]))
        };
        match std::fs::metadata(&p) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return nil("missing"),
            Err(err) => return nil(&err.to_string()),
            Ok(md) if md.len() > (1 << 20) => return nil("data file too large"),
            Ok(_) => {}
        }
        match std::fs::read(&p) {
            Ok(b) => Ok(MultiValue::from_iter([Value::String(lua.create_string(&b)?)])),
            Err(err) => nil(&err.to_string()),
        }
    })?;
    reg(e, gd, "data_write", |_, st, a| {
        let name = a.check_string(1)?;
        let p = data_path(st, &name)?;
        let text = a.check_bytes(2)?;
        if text.len() > (1 << 20) {
            return Err(lua_err("data files are at most 1 MB"));
        }
        std::fs::write(&p, &text).map_err(|_| lua_err(format!("cannot write {name}")))?;
        Ok(())
    })?;
    reg(e, gd, "data_write_atomic", |lua, st, a| {
        let name = a.check_string(1)?;
        let p = data_path(st, &name)?;
        let text = a.check_bytes(2)?;
        if text.len() > (1 << 20) {
            return fail(lua, "data files are at most 1 MB");
        }
        // the temporary lives in the target's directory as a component starting with '.', which the data-name
        // grammar cannot address, so it never collides with a user's file
        let base = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let tmp = p.with_file_name(format!(".atomic-{base}.tmp"));
        let open = || std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp);
        let f = match open() {
            Ok(f) => Ok(f),
            Err(_) => {
                let _ = std::fs::remove_file(&tmp);
                open()
            }
        };
        let Ok(mut f) = f else { return fail(lua, "cannot open the temporary file") };
        let wrote = f.write_all(&text).and_then(|_| f.flush()).and_then(|_| f.sync_all());
        drop(f);
        if wrote.is_err() {
            let _ = std::fs::remove_file(&tmp);
            return fail(lua, "temporary write failed");
        }
        if std::fs::rename(&tmp, &p).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return fail(lua, "could not replace the previous file");
        }
        Ok(MultiValue::from_iter([Value::Boolean(true)]))
    })?;

    reg(e, gd, "script", |lua, st, _| {
        let s = st.borrow();
        let t = lua.create_table()?;
        if let Some(sc) = s.cur_script() {
            t.string("id", &sc.id)?;
            t.string("name", &sc.name)?;
            t.string("version", &sc.version)?;
            t.string("author", &sc.author)?;
            t.boolean("gameplay", sc.gameplay)?;
            t.boolean("rollback_safe", sc.rollback_safe)?;
        }
        Ok(t)
    })?;
    Ok(())
}

/// `luaL_tolstring` through the global `tostring` (so `__tostring` and `__name` apply).
pub fn tostring_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.to_string_lossy().to_string(),
        Value::Nil => "nil".into(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Number(n) => crate::args::fmt_number(*n),
        other => format!("{}: {:p}", match other { Value::Table(_) => "table", Value::Function(_) => "function", Value::Thread(_) => "thread", _ => "userdata" }, other.to_pointer()),
    }
}

fn draw_new(s: &mut crate::engine::State, d: Draw) {
    let b = s.build;
    if s.draw[b].len() < GS_MAX_DRAW {
        s.draw[b].push(d);
    }
}
