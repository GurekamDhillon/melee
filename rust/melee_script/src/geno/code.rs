//! A define's fighter code: stateless callbacks over the game half's io block (GENO_LUA_IO_WORDS words, the
//! geno_lua_core.h contract: the only state is the declared slots in the block, a call returns 0 or a GENO_LUA_FAULT_*
//! code). The geno 12 `wasm` block is the one kind this build runs; the host instantiates it (Env::fighter_wasm,
//! melee_engine::geno_wasm). The geno 10 `lua` block is refused: there is no Lua in the Rust port, and
//! tools/rust-port/geno/lua_to_wasm_package.py converts a package once its callbacks are compiled.

/// GLUA_NAME: a state name's buffer (31 bytes and the terminator), the `go` target names' cap
pub const GLUA_NAME: usize = 32;
/// GLUA_SLOT_NAME: a slot name's buffer (at most 23 bytes)
pub const GLUA_SLOT_NAME: usize = 24;
/// Slot types
pub const GLUA_INT: i32 = 0;
pub const GLUA_FLOAT: i32 = 1;
pub const GLUA_BOOL: i32 = 2;

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
