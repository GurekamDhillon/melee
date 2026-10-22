//! State oracle for tu/gmprogressivemode.rs (docs: src/oracle.rs, gen/tu/run.py).
use melee_modes::oracle::*;
use melee_modes::tu::gmprogressivemode as tu;
use melee_types::GameModeState;

fn build_enter(c: &mut Case) {
    let st = c.arena.alloc::<GameModeState>();
    let data = c.arena.alloc_bytes(16);
    c.rng.fill(data, 16);
    c.ret_ptr("gm_GetGameModeStateEnterData", data);
    c.arg(st);
}
fn build_exit(c: &mut Case) {
    let st = c.arena.alloc::<GameModeState>();
    c.arg(st);
}

unsafe fn call_enter(a: &[u64]) -> u64 {
    tu::gm_801BF8F8(a[0] as usize as *mut GameModeState);
    0
}
unsafe fn call_exit(a: &[u64]) -> u64 {
    tu::gm_801BF920(a[0] as usize as *mut GameModeState);
    0
}

#[test]
fn gmprogressivemode_vs_c() {
    let funcs = [
        FuncDef { name: "gm_801BF8F8", argk: "p", retk: "v", ncases: 4, build: build_enter, call: call_enter },
        FuncDef { name: "gm_801BF920", argk: "p", retk: "v", ncases: 4, build: build_exit, call: call_exit },
    ];
    run_tu(&TuOracle {
        tu: "gmprogressivemode",
        funcs: &funcs,
        globals: tu::oracle_globals(),
        fn_table: vec![("gm_801BF8F8", tu::gm_801BF8F8 as usize), ("gm_801BF920", tu::gm_801BF920 as usize)],
    });
}
