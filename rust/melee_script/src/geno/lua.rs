//! geno_lua_core.h: Geno's fighter-Lua domain (slice 5, docs/geno.md section 23). It knows nothing of the registry or
//! of the fighter: a profile owns one `GLua`, a call gets the game half's io block (GENO_LUA_IO_WORDS words, native
//! order in this engine; the C build reads its big-endian guest words with gw_r32) and returns a fault code.
//!
//! The guarantees are the C's, one for one:
//!   - a module is a chunk returning a table of functions only, loaded with a read-only proxy environment over a fixed
//!     allowlist (type, select, ipairs, assert, error, freeze, math.{abs,min,max,floor,ceil,sqrt,tointeger,huge,pi});
//!     no libraries are opened;
//!   - a function may not keep state: its bytecode is scanned for OP_SETUPVAL / OP_SETTABUP (`geno_lua_scan_closure`,
//!     the vendored 5.4.7's internals, compiled with the interpreter) and every captured upvalue must be a plain value,
//!     a function passing the same test, or a frozen table;
//!   - every call builds a fresh ctx, runs under the instruction budget (counted in GENO_LUA_INSN_STEP steps) and the
//!     heap budget, with the collector stopped during the call and a full collection after it;
//!   - the only state is ctx.state, a proxy over the declared slots, read and written straight in the io block.
//! The interpreter is the same 5.4.7 as the C build (rust/third_party/lua-src), so the instruction counts agree. The
//! heap budget counts the allocator's bytes: the C build is 32-bit, so a module that runs near GENO_LUA_HEAP_BUDGET
//! faults at a different point here (no shipped module comes near it).
#![allow(non_snake_case, clippy::missing_safety_doc)]
use super::consts::*;
use mlua::ffi::{self, lua_State};
use std::ffi::{c_char, c_int, c_void, CStr};

pub const GLUA_MAX_FNS: usize = 48;
pub const GLUA_NAME: usize = 32;
pub const GLUA_SLOT_NAME: usize = 24;
const GLUA_LOAD_HEAP: usize = 4 * 1024 * 1024;

pub const GLUA_INT: i32 = 0;
pub const GLUA_FLOAT: i32 = 1;
pub const GLUA_BOOL: i32 = 2;

unsafe extern "C" {
    /// rust/third_party/lua-src/extra/geno_lua_scan.c: 0 ok, 1 OP_SETUPVAL, 2 OP_SETTABUP.
    fn geno_lua_scan_closure(L: *mut lua_State, idx: c_int) -> c_int;
}

pub struct GLua {
    L: *mut lua_State,
    used: usize,
    limit: usize,
    insns: i32,
    insn_limit: i32,
    fault: i32,
    io: *mut u32,
    ncmd: i32,
    pub nslot: usize,
    pub slot_name: Vec<String>,
    pub slot_type: Vec<i32>,
    target_name: Vec<String>,
    fn_name: Vec<String>,
    fn_ref: Vec<c_int>,
    /// The last refusal or fault, for the log.
    pub err: String,
    pub calls: u32,
    pub faults: u32,
}

unsafe fn glua_of(L: *mut lua_State) -> *mut GLua {
    *(ffi::lua_getextraspace(L) as *mut *mut GLua)
}

unsafe extern "C" fn glua_alloc(ud: *mut c_void, ptr: *mut c_void, osize: usize, nsize: usize) -> *mut c_void {
    let g = &mut *(ud as *mut GLua);
    let old = if ptr.is_null() { 0 } else { osize };
    if nsize == 0 {
        if !ptr.is_null() {
            g.used -= osize;
            libc_free(ptr);
        }
        return core::ptr::null_mut();
    }
    if nsize > old && g.limit != 0 && g.used + (nsize - old) > g.limit {
        if g.fault == 0 {
            g.fault = GENO_LUA_FAULT_HEAP as i32;
        }
        return core::ptr::null_mut();
    }
    let p = libc_realloc(ptr, nsize);
    if p.is_null() {
        return p;
    }
    g.used = g.used - old + nsize;
    p
}

unsafe extern "C" {
    #[link_name = "realloc"]
    fn libc_realloc(p: *mut c_void, n: usize) -> *mut c_void;
    #[link_name = "free"]
    fn libc_free(p: *mut c_void);
}

/// Raise a Lua error with `msg` (longjmp out of the calling C function: the callers hold no value with a destructor).
unsafe fn raise(L: *mut lua_State, msg: &str) -> ! {
    ffi::lua_pushlstring(L, msg.as_ptr() as *const c_char, msg.len());
    ffi::lua_error(L)
}

unsafe extern "C-unwind" fn glua_hook(L: *mut lua_State, _ar: *mut ffi::lua_Debug) {
    let g = &mut *glua_of(L);
    g.insns += GENO_LUA_INSN_STEP as i32;
    if g.insns > g.insn_limit {
        if g.fault == 0 {
            g.fault = GENO_LUA_FAULT_INSNS as i32;
        }
        lerror(L, "instruction budget exceeded");
    }
}

/// luaL_error: the message with the position prefix (luaL_where(L, 1)).
unsafe fn lerror(L: *mut lua_State, msg: &'static str) -> ! {
    let mut buf = [0u8; 160];
    let n = msg.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&msg.as_bytes()[..n]);
    ffi::luaL_error(L, c"%s".as_ptr(), buf.as_ptr() as *const c_char);
    unreachable!()
}

unsafe fn glua_fault(L: *mut lua_State, code: u32, msg: &'static str) -> ! {
    let g = &mut *glua_of(L);
    if g.fault == 0 {
        g.fault = code as i32;
    }
    lerror(L, msg)
}

// ---- frozen tables ----

unsafe fn glua_is_frozen(L: *mut lua_State, idx: c_int) -> bool {
    let mut r = false;
    if ffi::lua_getmetatable(L, idx) != 0 {
        ffi::lua_getfield(L, -1, c"__gnfrozen".as_ptr());
        r = ffi::lua_toboolean(L, -1) != 0;
        ffi::lua_pop(L, 2);
    }
    r
}

unsafe extern "C-unwind" fn glua_frozen_len(L: *mut lua_State) -> c_int {
    ffi::lua_pushinteger(L, ffi::lua_rawlen(L, ffi::lua_upvalueindex(1)) as ffi::lua_Integer);
    1
}

unsafe extern "C-unwind" fn glua_frozen_newindex(L: *mut lua_State) -> c_int {
    glua_fault(L, GENO_LUA_FAULT_ERROR, "attempt to modify a frozen table")
}

/// Pushes the frozen form of the value at idx. false (with a message pushed) for a table/function key, or nesting
/// deeper than 8.
unsafe fn glua_freeze_val(L: *mut lua_State, idx: c_int, depth: i32) -> bool {
    let idx = ffi::lua_absindex(L, idx);
    if ffi::lua_type(L, idx) != ffi::LUA_TTABLE || glua_is_frozen(L, idx) {
        ffi::lua_pushvalue(L, idx);
        return true;
    }
    if depth > 8 {
        ffi::lua_pushliteral(L, c"freeze: tables nested deeper than 8");
        return false;
    }
    ffi::luaL_checkstack(L, 10, c"freeze".as_ptr());
    ffi::lua_newtable(L);
    let data = ffi::lua_gettop(L);
    ffi::lua_pushnil(L);
    while ffi::lua_next(L, idx) != 0 {
        let kt = ffi::lua_type(L, -2);
        if kt == ffi::LUA_TTABLE || kt == ffi::LUA_TFUNCTION {
            ffi::lua_pushliteral(L, c"freeze: a table or function used as a key");
            return false;
        }
        if !glua_freeze_val(L, -1, depth + 1) {
            return false;
        }
        ffi::lua_pushvalue(L, -3);
        ffi::lua_insert(L, -2);
        ffi::lua_rawset(L, data);
        ffi::lua_pop(L, 1);
    }
    ffi::lua_newtable(L);
    let proxy = ffi::lua_gettop(L);
    ffi::lua_newtable(L);
    ffi::lua_pushvalue(L, data);
    ffi::lua_setfield(L, -2, c"__index".as_ptr());
    ffi::lua_pushvalue(L, data);
    ffi::lua_pushcclosure(L, glua_frozen_len, 1);
    ffi::lua_setfield(L, -2, c"__len".as_ptr());
    ffi::lua_pushcfunction(L, glua_frozen_newindex);
    ffi::lua_setfield(L, -2, c"__newindex".as_ptr());
    ffi::lua_pushboolean(L, 1);
    ffi::lua_setfield(L, -2, c"__gnfrozen".as_ptr());
    ffi::lua_pushboolean(L, 0);
    ffi::lua_setfield(L, -2, c"__metatable".as_ptr());
    ffi::lua_setmetatable(L, proxy);
    ffi::lua_remove(L, data);
    true
}

unsafe extern "C-unwind" fn glua_freeze_fn(L: *mut lua_State) -> c_int {
    ffi::luaL_checktype(L, 1, ffi::LUA_TTABLE);
    if !glua_freeze_val(L, 1, 0) {
        ffi::lua_error(L);
    }
    1
}

// ---- the ctx: typed state, commands ----

unsafe fn key_str<'a>(L: *mut lua_State, idx: c_int) -> &'a [u8] {
    let mut n = 0usize;
    let p = ffi::lua_tolstring(L, idx, &mut n);
    core::slice::from_raw_parts(p as *const u8, n)
}

fn slot_find(g: &GLua, k: &[u8]) -> Option<usize> {
    // strcmp on the C string: a key with an embedded NUL compares up to it
    let k = k.split(|&b| b == 0).next().unwrap_or(&[]);
    (0..g.nslot).find(|&i| g.slot_name[i].as_bytes() == k)
}

unsafe extern "C-unwind" fn glua_state_index(L: *mut lua_State) -> c_int {
    let g = &mut *glua_of(L);
    if ffi::lua_type(L, 2) != ffi::LUA_TSTRING {
        glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: the key is not a name");
    }
    let Some(slot) = slot_find(g, key_str(L, 2)) else {
        glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared")
    };
    let w = *g.io.add((GENO_LUA_IO_STATE as usize) + slot);
    match g.slot_type[slot] {
        GLUA_INT => ffi::lua_pushinteger(L, w as i32 as ffi::lua_Integer),
        GLUA_FLOAT => ffi::lua_pushnumber(L, f32::from_bits(w) as f64),
        _ => ffi::lua_pushboolean(L, (w != 0) as c_int),
    }
    1
}

unsafe extern "C-unwind" fn glua_state_newindex(L: *mut lua_State) -> c_int {
    let g = &mut *glua_of(L);
    if ffi::lua_type(L, 2) != ffi::LUA_TSTRING {
        glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: the key is not a name");
    }
    let Some(slot) = slot_find(g, key_str(L, 2)) else {
        glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: no such slot is declared")
    };
    let w: u32 = match g.slot_type[slot] {
        GLUA_INT => {
            let mut ok: c_int = 0;
            let mut v: ffi::lua_Integer = 0;
            if ffi::lua_type(L, 3) == ffi::LUA_TNUMBER {
                v = ffi::lua_tointegerx(L, 3, &mut ok);
            }
            if ok == 0 || v < i32::MIN as i64 || v > i32::MAX as i64 {
                glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: an int slot takes an integer that fits 32 bits");
            }
            v as i32 as u32
        }
        GLUA_FLOAT => {
            if ffi::lua_type(L, 3) != ffi::LUA_TNUMBER {
                glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a float slot takes a number");
            }
            let v = ffi::lua_tonumberx(L, 3, core::ptr::null_mut());
            if v.is_nan() || v.abs() > 3.0e38 {
                glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a float slot takes a finite value");
            }
            (v as f32).to_bits()
        }
        _ => {
            if ffi::lua_type(L, 3) != ffi::LUA_TBOOLEAN {
                glua_fault(L, GENO_LUA_FAULT_STATE, "ctx.state: a bool slot takes true or false");
            }
            (ffi::lua_toboolean(L, 3) != 0) as u32
        }
    };
    *g.io.add(GENO_LUA_IO_STATE as usize + slot) = w;
    0
}

unsafe fn glua_cmd(L: *mut lua_State, op: u32, a: u32, b: u32) -> c_int {
    let g = &mut *glua_of(L);
    if g.ncmd >= GENO_LUA_MAX_CMDS as i32 {
        glua_fault(L, GENO_LUA_FAULT_CMDS, "more commands than one call may queue");
    }
    let c = g.io.add(GENO_LUA_IO_CMDS as usize + 3 * g.ncmd as usize);
    *c = op;
    *c.add(1) = a;
    *c.add(2) = b;
    g.ncmd += 1;
    *g.io.add(GENO_LUA_IO_NCMDS as usize) = g.ncmd as u32;
    0
}

unsafe fn glua_checkfinite(L: *mut lua_State, arg: c_int, lim: f64, what: &'static str) -> f32 {
    if ffi::lua_type(L, arg) != ffi::LUA_TNUMBER {
        glua_fault(L, GENO_LUA_FAULT_COMMAND, what);
    }
    let v = ffi::lua_tonumberx(L, arg, core::ptr::null_mut());
    if v.is_nan() || v.abs() > lim {
        glua_fault(L, GENO_LUA_FAULT_COMMAND, what);
    }
    v as f32
}

fn tgt(kind: u32, id: u32) -> u32 {
    ((kind & 15) << 28) | (id & 0xFFFF)
}

unsafe extern "C-unwind" fn glua_go(L: *mut lua_State) -> c_int {
    let g = &*glua_of(L);
    if ffi::lua_type(L, 1) != ffi::LUA_TSTRING {
        glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.go takes a state name");
    }
    let name = key_str(L, 1);
    let name = name.split(|&b| b == 0).next().unwrap_or(&[]);
    match name {
        b"auto" => return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TGT_AUTO, 0),
        b"roll" => return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TGT_ROLL, 0),
        b"helpless" => return glua_cmd(L, GENO_LUA_CMD_GO, GENO_TGT_HELPLESS, 0),
        _ => {}
    }
    if let Some(i) = g.target_name.iter().position(|t| t.as_bytes() == name) {
        return glua_cmd(L, GENO_LUA_CMD_GO, tgt(GENO_TGT_GENO, i as u32), 0);
    }
    glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.go: no such state")
}

unsafe extern "C-unwind" fn glua_velocity(L: *mut lua_State) -> c_int {
    let f = glua_checkfinite(L, 1, 1000.0, "ctx.velocity takes finite numbers (forward, up)");
    let u = glua_checkfinite(L, 2, 1000.0, "ctx.velocity takes finite numbers (forward, up)");
    glua_cmd(L, GENO_LUA_CMD_VELOCITY, f.to_bits(), u.to_bits())
}

unsafe fn int_arg(L: *mut lua_State, i: c_int) -> Option<i64> {
    if ffi::lua_type(L, i) != ffi::LUA_TNUMBER {
        return None;
    }
    let mut ok: c_int = 0;
    let m = ffi::lua_tointegerx(L, i, &mut ok);
    (ok != 0).then_some(m)
}

unsafe extern "C-unwind" fn glua_hitbox_damage(L: *mut lua_State) -> c_int {
    let m = int_arg(L, 1);
    let Some(m) = m.filter(|m| (1..=15).contains(m)) else {
        glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: the mask is 1..15 (bit n = hitbox slot n)")
    };
    let d = glua_checkfinite(L, 2, 1000.0, "ctx.hitbox_damage takes a finite damage");
    if d < 0.0 {
        glua_fault(L, GENO_LUA_FAULT_COMMAND, "ctx.hitbox_damage: damage is not negative");
    }
    glua_cmd(L, GENO_LUA_CMD_HITBOX_DAMAGE, m as u32, d.to_bits())
}

unsafe extern "C-unwind" fn glua_loop(L: *mut lua_State) -> c_int {
    glua_cmd(L, GENO_LUA_CMD_LOOP, 0, 0)
}

unsafe extern "C-unwind" fn glua_turn(L: *mut lua_State) -> c_int {
    glua_cmd(L, GENO_LUA_CMD_TURN, 0, 0)
}

/// geno 11: ctx.ki(delta) adds to the combo meter (meter units, two decimals; negative spends).
unsafe extern "C-unwind" fn glua_ki(L: *mut lua_State) -> c_int {
    let d = glua_checkfinite(L, 1, 1000.0, "ctx.ki takes a finite number (meter units)");
    let h = (d * 100.0 + if d < 0.0 { -0.5 } else { 0.5 }) as i32;
    glua_cmd(L, GENO_LUA_CMD_KI, h as u32, 0)
}

unsafe extern "C-unwind" fn glua_ledge(L: *mut lua_State) -> c_int {
    let Some(m) = int_arg(L, 1).filter(|m| (0..=3).contains(m)) else {
        glua_fault(
            L,
            GENO_LUA_FAULT_COMMAND,
            "ctx.ledge takes 0 (none), 1 (front), 2 (front and back) or 3 (2, and the special fall after the move)",
        )
    };
    glua_cmd(L, GENO_LUA_CMD_LEDGE, m as u32, 0)
}

// ---- closure scan ----

struct Seen {
    seen: [*const c_void; 16],
    n: usize,
}

impl GLua {
    unsafe fn check_fn(&mut self, idx: c_int, fname: &str, depth: i32, sn: &mut Seen) -> bool {
        let L = self.L;
        let idx = ffi::lua_absindex(L, idx);
        if ffi::lua_iscfunction(L, idx) != 0 {
            return true;
        }
        let cl = ffi::lua_topointer(L, idx);
        if sn.seen[..sn.n].contains(&cl) {
            return true;
        }
        if depth > 6 || sn.n >= 16 {
            self.err = format!("function {fname} captures functions nested too deep");
            return false;
        }
        sn.seen[sn.n] = cl;
        sn.n += 1;
        match geno_lua_scan_closure(L, idx) {
            1 => {
                self.err = format!("function {fname} assigns a captured variable (keep state in ctx.state)");
                return false;
            }
            2 => {
                self.err = format!("function {fname} assigns a global or a field of a captured table (keep state in ctx.state)");
                return false;
            }
            _ => {}
        }
        let mut i = 1;
        loop {
            let nm = ffi::lua_getupvalue(L, idx, i);
            if nm.is_null() {
                break;
            }
            let nm = CStr::from_ptr(nm).to_string_lossy().into_owned();
            let t = ffi::lua_type(L, -1);
            let mut ok = true;
            if nm == "_ENV" {
            } else if t == ffi::LUA_TNIL || t == ffi::LUA_TBOOLEAN || t == ffi::LUA_TNUMBER || t == ffi::LUA_TSTRING {
            } else if t == ffi::LUA_TFUNCTION {
                ok = self.check_fn(-1, fname, depth + 1, sn);
            } else if t == ffi::LUA_TTABLE {
                if !glua_is_frozen(L, -1) {
                    self.err = format!(
                        "function {fname} captures the mutable table '{nm}' (wrap constants in freeze(), or pass data through ctx)"
                    );
                    ok = false;
                }
            } else {
                self.err = format!("function {fname} captures '{nm}', which is not a plain value");
                ok = false;
            }
            ffi::lua_pop(L, 1);
            if !ok {
                return false;
            }
            i += 1;
        }
        true
    }

    fn budget(&mut self, heap: usize) {
        self.insns = 0;
        self.insn_limit = GENO_LUA_INSN_BUDGET as i32;
        self.limit = self.used + heap;
        self.fault = 0;
    }

    /// A new, empty domain (no module). None when Lua cannot allocate.
    pub fn new() -> Option<Box<GLua>> {
        let mut g = Box::new(GLua {
            L: core::ptr::null_mut(),
            used: 0,
            limit: 0,
            insns: 0,
            insn_limit: 0,
            fault: 0,
            io: core::ptr::null_mut(),
            ncmd: 0,
            nslot: 0,
            slot_name: Vec::new(),
            slot_type: Vec::new(),
            target_name: Vec::new(),
            fn_name: Vec::new(),
            fn_ref: Vec::new(),
            err: String::new(),
            calls: 0,
            faults: 0,
        });
        unsafe {
            let ud = &mut *g as *mut GLua as *mut c_void;
            let L = ffi::lua_newstate(glua_alloc, ud);
            if L.is_null() {
                return None;
            }
            g.L = L;
            *(ffi::lua_getextraspace(L) as *mut *mut GLua) = &mut *g;
            ffi::lua_gc(L, ffi::LUA_GCSTOP);
            ffi::lua_sethook(L, Some(glua_hook), ffi::LUA_MASKCOUNT, GENO_LUA_INSN_STEP as c_int);
        }
        Some(g)
    }

    pub fn set_layout(&mut self, names: &[String], types: &[i32]) {
        let n = names.len().min(GENO_LUA_STATE_SLOTS as usize);
        self.nslot = n;
        self.slot_name = names[..n].iter().map(|s| cut(s, GLUA_SLOT_NAME)).collect();
        self.slot_type = types[..n].to_vec();
    }

    pub fn set_targets(&mut self, names: &[String]) {
        let n = names.len().min(GENO_MAX_STATES as usize);
        self.target_name = names[..n].iter().map(|s| cut(s, GLUA_NAME)).collect();
    }

    /// Builds the allowlist environment (a frozen proxy) and leaves it on the stack.
    unsafe fn push_env(L: *mut lua_State) {
        const BASE: [&CStr; 5] = [c"type", c"select", c"ipairs", c"assert", c"error"];
        const MATHF: [&CStr; 9] = [c"abs", c"min", c"max", c"floor", c"ceil", c"sqrt", c"tointeger", c"huge", c"pi"];
        ffi::luaL_requiref(L, c"_G".as_ptr(), ffi::luaopen_base, 1);
        let env = ffi::lua_gettop(L) + 1;
        ffi::lua_newtable(L);
        for b in BASE {
            ffi::lua_getglobal(L, b.as_ptr());
            ffi::lua_setfield(L, env, b.as_ptr());
        }
        ffi::luaL_requiref(L, c"math".as_ptr(), ffi::luaopen_math, 0);
        let m = ffi::lua_gettop(L);
        ffi::lua_newtable(L);
        for f in MATHF {
            ffi::lua_getfield(L, m, f.as_ptr());
            ffi::lua_setfield(L, -2, f.as_ptr());
        }
        ffi::lua_setfield(L, env, c"math".as_ptr());
        ffi::lua_pushcfunction(L, glua_freeze_fn);
        ffi::lua_setfield(L, env, c"freeze".as_ptr());
        ffi::lua_pop(L, 1);
        glua_freeze_val(L, env, 0);
        ffi::lua_remove(L, env);
        ffi::lua_remove(L, env - 1);
        ffi::lua_pushnil(L);
        ffi::lua_setglobal(L, c"dofile".as_ptr());
    }

    /// Loads `src` as the module, checks it, and registers every function it returns. false with `err` set.
    pub fn load(&mut self, src: &[u8], chunk: &str) -> bool {
        unsafe {
            let L = self.L;
            let top = ffi::lua_gettop(L);
            self.err.clear();
            self.budget(0);
            self.limit = self.used + GLUA_LOAD_HEAP;
            Self::push_env(L);
            let chunk_c = std::ffi::CString::new(chunk).unwrap_or_default();
            if ffi::luaL_loadbufferx(L, src.as_ptr() as *const c_char, src.len(), chunk_c.as_ptr(), c"t".as_ptr()) != ffi::LUA_OK {
                self.err = tostr(L, -1);
                ffi::lua_settop(L, top);
                return false;
            }
            ffi::lua_pushvalue(L, -2);
            if ffi::lua_setupvalue(L, -2, 1).is_null() {
                self.err = "the module has no environment".into();
                ffi::lua_settop(L, top);
                return false;
            }
            let st = ffi::lua_pcall(L, 0, 1, 0);
            if st != ffi::LUA_OK {
                let m = if ffi::lua_tostring(L, -1).is_null() { "error".to_string() } else { tostr(L, -1) };
                self.err = format!("{m}{}", if self.fault == GENO_LUA_FAULT_INSNS as i32 { " (loading the module)" } else { "" });
                ffi::lua_settop(L, top);
                return false;
            }
            if ffi::lua_type(L, -1) != ffi::LUA_TTABLE {
                self.err = "the module must return a table of functions".into();
                ffi::lua_settop(L, top);
                return false;
            }
            let tbl = ffi::lua_gettop(L);
            ffi::lua_pushnil(L);
            while ffi::lua_next(L, tbl) != 0 {
                if ffi::lua_type(L, -2) != ffi::LUA_TSTRING || ffi::lua_type(L, -1) != ffi::LUA_TFUNCTION || ffi::lua_iscfunction(L, -1) != 0 {
                    self.err = "the module table may hold only functions with name keys (constants belong in locals)".into();
                    ffi::lua_settop(L, top);
                    return false;
                }
                let k = CStr::from_ptr(ffi::lua_tostring(L, -2)).to_string_lossy().into_owned();
                if k.len() >= GLUA_NAME || self.fn_name.len() >= GLUA_MAX_FNS {
                    self.err = format!("function name '{k}' is too long or the module has more than {GLUA_MAX_FNS} functions");
                    ffi::lua_settop(L, top);
                    return false;
                }
                let mut sn = Seen { seen: [core::ptr::null(); 16], n: 0 };
                if !self.check_fn(-1, &k, 0, &mut sn) {
                    ffi::lua_settop(L, top);
                    return false;
                }
                self.fn_name.push(k);
                ffi::lua_pushvalue(L, -1);
                self.fn_ref.push(ffi::luaL_ref(L, ffi::LUA_REGISTRYINDEX));
                ffi::lua_pop(L, 1);
            }
            ffi::lua_settop(L, top);
            ffi::lua_newtable(L);
            ffi::lua_pushcfunction(L, glua_state_index);
            ffi::lua_setfield(L, -2, c"__index".as_ptr());
            ffi::lua_pushcfunction(L, glua_state_newindex);
            ffi::lua_setfield(L, -2, c"__newindex".as_ptr());
            ffi::lua_pushboolean(L, 0);
            ffi::lua_setfield(L, -2, c"__metatable".as_ptr());
            ffi::lua_setfield(L, ffi::LUA_REGISTRYINDEX, c"geno.lua.state".as_ptr());
            ffi::lua_gc(L, ffi::LUA_GCCOLLECT);
            self.limit = 0;
            true
        }
    }

    /// glua_find: the index of a module function, -1 none.
    pub fn find(&self, name: &str) -> i32 {
        self.fn_name.iter().position(|n| n == name).map_or(-1, |i| i as i32)
    }

    unsafe fn push_ctx(&mut self) {
        let L = self.L;
        let io = self.io;
        let r = |i: u32| *io.add(i as usize);
        let fl = r(GENO_LUA_IO_FLAGS);
        let held = r(GENO_LUA_IO_HELD);
        let pressed = r(GENO_LUA_IO_PRESSED);
        let sx = f32::from_bits(r(GENO_LUA_IO_STICK_X));
        let sy = f32::from_bits(r(GENO_LUA_IO_STICK_Y));
        const BTN: [(u32, &CStr, &CStr); 6] = [
            (GENO_BTN_ATTACK, c"attack_held", c"attack_pressed"),
            (GENO_BTN_SPECIAL, c"special_held", c"special_pressed"),
            (GENO_BTN_JUMP, c"jump_held", c"jump_pressed"),
            (GENO_BTN_SHIELD, c"shield_held", c"shield_pressed"),
            (GENO_BTN_GRAB, c"grab_held", c"grab_pressed"),
            (GENO_BTN_BLAST, c"blast_held", c"blast_pressed"),
        ];
        let setnum = |k: &CStr, v: f64| {
            ffi::lua_pushnumber(L, v);
            ffi::lua_setfield(L, -2, k.as_ptr());
        };
        let setbool = |k: &CStr, v: bool| {
            ffi::lua_pushboolean(L, v as c_int);
            ffi::lua_setfield(L, -2, k.as_ptr());
        };
        let setint = |k: &CStr, v: i64| {
            ffi::lua_pushinteger(L, v);
            ffi::lua_setfield(L, -2, k.as_ptr());
        };
        ffi::lua_createtable(L, 0, 8);
        ffi::lua_createtable(L, 0, 14);
        for (bit, h, p) in BTN {
            setbool(h, held & bit != 0);
            setbool(p, pressed & bit != 0);
        }
        setnum(c"stick_x", sx as f64);
        setnum(c"stick_y", sy as f64);
        setnum(c"stick_fwd", if fl & 2 != 0 { -sx } else { sx } as f64);
        ffi::lua_setfield(L, -2, c"input".as_ptr());
        ffi::lua_createtable(L, 0, 16);
        setbool(c"air", fl & 1 != 0);
        setbool(c"anim_ended", fl & 4 != 0);
        setnum(c"facing", if fl & 2 != 0 { -1.0 } else { 1.0 });
        setnum(c"percent", f32::from_bits(r(GENO_LUA_IO_PERCENT)) as f64);
        setnum(c"x", f32::from_bits(r(GENO_LUA_IO_POS_X)) as f64);
        setnum(c"y", f32::from_bits(r(GENO_LUA_IO_POS_Y)) as f64);
        setnum(c"vel_x", f32::from_bits(r(GENO_LUA_IO_VEL_X)) as f64);
        setnum(c"vel_y", f32::from_bits(r(GENO_LUA_IO_VEL_Y)) as f64);
        setint(c"action_frame", r(GENO_LUA_IO_ACTION_FRAME) as i32 as i64);
        setint(c"motion", r(GENO_LUA_IO_MOTION) as i32 as i64);
        setnum(c"hit_damage", f32::from_bits(r(GENO_LUA_IO_HIT_DAMAGE)) as f64);
        let hf = r(GENO_LUA_IO_HIT_FROM);
        setint(c"hit_from", if hf == 1 { 1 } else if hf == 2 { -1 } else { 0 });
        setint(c"countered", r(GENO_LUA_IO_COUNTERED) as i32 as i64);
        setnum(c"ki", (r(GENO_LUA_IO_KI) as i32) as f64 / 100.0);
        setint(c"combo_links", r(GENO_LUA_IO_LINKS) as i32 as i64);
        setint(c"connected", r(GENO_LUA_IO_CONNECTED) as i32 as i64);
        ffi::lua_setfield(L, -2, c"self".as_ptr());
        ffi::lua_newtable(L);
        ffi::lua_getfield(L, ffi::LUA_REGISTRYINDEX, c"geno.lua.state".as_ptr());
        ffi::lua_setmetatable(L, -2);
        ffi::lua_setfield(L, -2, c"state".as_ptr());
        let fns: [(&CStr, ffi::lua_CFunction); 7] = [
            (c"go", glua_go),
            (c"velocity", glua_velocity),
            (c"hitbox_damage", glua_hitbox_damage),
            (c"loop", glua_loop),
            (c"turn", glua_turn),
            (c"ki", glua_ki),
            (c"ledge", glua_ledge),
        ];
        for (k, f) in fns {
            ffi::lua_pushcfunction(L, f);
            ffi::lua_setfield(L, -2, k.as_ptr());
        }
    }

    /// Runs function `fn_` (an index from `find`) over `io`. 0 = ok; GENO_LUA_FAULT_* otherwise (the io's state words
    /// and commands are then to be discarded by the caller).
    pub unsafe fn call(&mut self, fn_: i32, io: *mut u32) -> i32 {
        let L = self.L;
        let top = ffi::lua_gettop(L);
        self.calls += 1;
        self.err.clear();
        self.io = io;
        self.ncmd = 0;
        *io.add(GENO_LUA_IO_NCMDS as usize) = 0;
        *io.add(GENO_LUA_IO_FAULT as usize) = 0;
        self.budget(GENO_LUA_HEAP_BUDGET as usize);
        ffi::lua_rawgeti(L, ffi::LUA_REGISTRYINDEX, self.fn_ref[fn_ as usize] as ffi::lua_Integer);
        self.push_ctx();
        let st = ffi::lua_pcall(L, 1, 0, 0);
        // the ctx functions record their fault through the state's extraspace pointer while this `&mut self` is held:
        // read it back through that pointer (a plain `self.fault` may be assumed unchanged across the FFI call, which
        // reported every fault as GENO_LUA_FAULT_ERROR in an optimized build)
        self.fault = core::ptr::read_volatile(core::ptr::addr_of!((*glua_of(L)).fault));
        if st != ffi::LUA_OK {
            if self.fault == 0 {
                self.fault = if st == ffi::LUA_ERRMEM { GENO_LUA_FAULT_HEAP } else { GENO_LUA_FAULT_ERROR } as i32;
            }
            self.err = if ffi::lua_type(L, -1) == ffi::LUA_TSTRING { tostr(L, -1) } else { "error".into() };
            self.faults += 1;
            *io.add(GENO_LUA_IO_NCMDS as usize) = 0;
        }
        ffi::lua_settop(L, top);
        ffi::lua_gc(L, ffi::LUA_GCCOLLECT);
        self.limit = 0;
        let st = self.fault;
        self.fault = 0;
        self.io = core::ptr::null_mut();
        *io.add(GENO_LUA_IO_FAULT as usize) = st as u32;
        st
    }
}

/// A define's fighter code (the geno 10 `lua` block or the geno 12 `wasm` block): stateless callbacks over the game
/// half's io block. The registry and the game half see only this.
pub trait FighterCode {
    fn nslot(&self) -> usize;
    fn slot_name(&self, i: usize) -> &str;
    fn slot_type(&self, i: usize) -> i32;
    /// The index of a callback by name, -1 none.
    fn find(&self, name: &str) -> i32;
    /// Runs callback `f` over `io`: 0 ok, GENO_LUA_FAULT_* otherwise (the io's fault word set, its commands cleared).
    unsafe fn call(&mut self, f: i32, io: *mut u32) -> i32;
    /// The last refusal or fault, for the log.
    fn err(&self) -> &str;
    fn faults(&self) -> u32;
}

impl FighterCode for GLua {
    fn nslot(&self) -> usize {
        self.nslot
    }
    fn slot_name(&self, i: usize) -> &str {
        &self.slot_name[i]
    }
    fn slot_type(&self, i: usize) -> i32 {
        self.slot_type[i]
    }
    fn find(&self, name: &str) -> i32 {
        GLua::find(self, name)
    }
    unsafe fn call(&mut self, f: i32, io: *mut u32) -> i32 {
        GLua::call(self, f, io)
    }
    fn err(&self) -> &str {
        &self.err
    }
    fn faults(&self) -> u32 {
        self.faults
    }
}

impl Drop for GLua {
    fn drop(&mut self) {
        if !self.L.is_null() {
            unsafe { ffi::lua_close(self.L) };
        }
    }
}

unsafe fn tostr(L: *mut lua_State, i: c_int) -> String {
    let p = ffi::lua_tostring(L, i);
    if p.is_null() {
        return String::new();
    }
    CStr::from_ptr(p).to_string_lossy().into_owned()
}

fn cut(s: &str, cap: usize) -> String {
    if s.len() < cap { s.to_string() } else { s[..cap - 1].to_string() }
}
