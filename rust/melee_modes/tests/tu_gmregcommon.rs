//! State oracle for tu/gmregcommon.rs (src/oracle.rs, gen/tu/run.py).
use melee_modes::oracle::*;
use melee_modes::tu::gmregcommon as tu;

fn build_be84(c: &mut Case) {
    let v = if c.idx < 4 { [0u32, 0xFFFF_FFFF, 8, 0x1F8][c.idx as usize] } else { c.rng.u32() };
    c.arg(v);
}
fn build_be8c(c: &mut Case) {
    let p = c.arena.alloc_bytes(8) as *mut i8;
    for i in 0..3 {
        let v = if c.rng.chance(1, 2) { 33 } else { c.rng.range(-128, 127) as i8 };
        unsafe { *p.add(i) = v };
    }
    c.arg(p);
}
fn build_setup(c: &mut Case) {
    let ck = c.arena.alloc_bytes(8) as *mut i8;
    let cl = c.arena.alloc_bytes(8);
    let ckind = if c.rng.chance(1, 8) { 33 } else { c.rng.range(0, 25) as i8 };
    let color = c.rng.range(0, 4) as u8;
    for i in 0..3 {
        let v = match c.rng.range(0, 3) {
            0 => 33,
            1 => ckind, // same character as the player: forces a recolour
            _ => c.rng.range(0, 25) as i8,
        };
        unsafe {
            *ck.add(i) = v;
            *cl.add(i) = match c.rng.range(0, 3) { 0 => 0xFF, 1 => color, _ => c.rng.range(0, 5) as u8 };
        }
    }
    c.ret_int("gm_GetNumCostumesForCKind", 5, 6);
    c.arg(ckind);
    c.arg(color);
    c.arg(ck);
    c.arg(cl);
}

unsafe fn call_be84(a: &[u64]) -> u64 {
    tu::gm_8017BE84(a[0] as u32) as u32 as u64
}
unsafe fn call_be8c(a: &[u64]) -> u64 {
    tu::gm_8017BE8C(a[0] as usize as *const i8) as u32 as u64
}
unsafe fn call_setup(a: &[u64]) -> u64 {
    tu::gmRegSetupEnemyColorTable(a[0] as u8 as i8, a[1] as u8, a[2] as usize as *const i8, a[3] as usize as *mut u8);
    0
}

#[test]
fn gmregcommon_vs_c() {
    let funcs = [
        FuncDef { name: "gm_8017BE84", argk: "i", retk: "i32", ncases: 24, build: build_be84, call: call_be84 },
        FuncDef { name: "gm_8017BE8C", argk: "p", retk: "i32", ncases: 24, build: build_be8c, call: call_be8c },
        FuncDef { name: "gmRegSetupEnemyColorTable", argk: "iipp", retk: "v", ncases: 200, build: build_setup, call: call_setup },
    ];
    run_tu(&TuOracle { tu: "gmregcommon", funcs: &funcs, globals: tu::oracle_globals(), fn_table: vec![] });
}
