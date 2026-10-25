//! `luarun <script.lua> [args...]`: the stock `lua` standalone interpreter on the vendored Lua 5.4.7
//! (the C build's version), full standard library. It runs the pc/tests and pc/geno/tools scripts that need
//! no engine (stub `gd` / mock hosts) so the Rust host can be compared with the system `lua` on them.
use mlua::{Lua, MultiValue, Value};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(script) = args.first() else {
        eprintln!("usage: luarun <script.lua> [args...]");
        return ExitCode::from(2);
    };
    // Safety: the test scripts use io/os/package/debug exactly as the C-side `lua` binary lets them.
    let lua = unsafe { Lua::unsafe_new() };
    let argt = lua.create_table().unwrap();
    argt.set(-1, "luarun").unwrap();
    for (i, a) in args.iter().enumerate() {
        argt.set(i as i64, a.as_str()).unwrap();
    }
    lua.globals().set("arg", argt).unwrap();
    let mut src = match std::fs::read(script) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("luarun: cannot open {script}: {e}");
            return ExitCode::FAILURE;
        }
    };
    // luaL_loadfile skips a leading '#' line (keeping the newline so line numbers hold).
    if src.first() == Some(&b'#') {
        let end = src.iter().position(|&b| b == b'\n').unwrap_or(src.len());
        src.drain(..end);
    }
    let varargs: MultiValue = args[1..].iter().map(|a| Value::String(lua.create_string(a).unwrap())).collect();
    let chunk = lua.load(&src[..]).set_name(format!("@{script}"));
    match chunk.call::<MultiValue>(varargs) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("luarun: {e}");
            ExitCode::FAILURE
        }
    }
}
