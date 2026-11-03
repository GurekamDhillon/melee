//! State oracle for tu/mndeflicker.rs (src/oracle.rs, gen/tu/run.py).
use core::ffi::c_void;
use core::ptr::{null_mut, addr_of};
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::mndeflicker as tu;
use melee_modes::tu::mndeflicker::ext;
use melee_types::*;
#[path = "mn_common/mod.rs"]
mod mn;
use mn::*;

unsafe fn flow(c: &mut Case) {
    tu::mn_804A04F0.cur_menu = pick(c, &[0x15, 0x15, 0x15, 3, 0x20]);
    tu::mn_804A04F0.prev_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.entering_menu = c.rng.range(0, 255) as u8;
    tu::mn_804A04F0.buttons = c.rng.next();
    tu::mn_804D6BC8.cooldown = pick(c, &[0, 0, 0, 1, 5]);
    tu::mn_804D6BC8.x2 = c.rng.range(0, 99) as u16;
    tu::mn_804D6BC8.x4 = c.rng.range(0, 99) as i32;
    tu::HSD_GObj_804D783C = c.rng.range(0, 7) as i32;
    tu::HSD_GObj_JObjKind = c.rng.range(0, 255) as u8;
}

/// the screen gobj (global) with a Menu user data and a jobj
unsafe fn screen(c: &mut Case) -> *mut HSD_GObj {
    let m = c.zobj::<Menu>();
    (*m).cursor = c.rng.range(0, 1) as u8;
    (*m).text = if c.rng.chance(1, 2) { text(c) } else { null_mut() };
    let j = jobj(c);
    let g = gobj(c, m.cast(), j.cast());
    tu::mnDeflicker_804D6C38 = g;
    g
}

fn rets(c: &mut Case) {
    c.ret_int("mn_80229624", 0, 0xFFF);
    c.ret_int("gmMainLib_8015F4E8", 0, 3);
    let ef = pick(c, &[19.0f32, 30.0, 0.0, 20.0, 5.0]);
    c.ret_flt("mn_8022EC18", ef, ef);
    c.ret_pool("HSD_SisLib_803A5ACC", core::mem::size_of::<HSD_Text>());
    lb_children(c, 3);
    let ps = procs(c, 3);
    unsafe { tu::HSD_GObj_CurrentInvokedProc = ps[0] };
}

fn b_input(c: &mut Case) {
    unsafe {
        flow(c);
        screen(c);
        tu::mnDeflicker_804D6C3C = c.rng.range(0, 1) as u8;
        let btn = pick(c, &[0u64, 0x20, 0x200, 0x220, 0x1]);
        c.ret_seq_u64("mn_80229624", &[btn]);
    }
    rets(c);
    let g = c.zobj::<HSD_GObj>();
    c.arg(g);
}

fn b_gobj(c: &mut Case) {
    unsafe {
        flow(c);
        tu::mnDeflicker_804D6C3C = c.rng.range(0, 1) as u8;
    }
    rets(c);
    let m = c.zobj::<Menu>();
    unsafe { (*m).text = if c.rng.chance(1, 2) { text(c) } else { null_mut() } };
    let j = jobj(c);
    let g = gobj(c, m.cast(), j.cast());
    c.arg(g);
}

fn b_create(c: &mut Case) {
    // what GObj_Create / GObj_InitUserData / HSD_GObjObject_80390A70 would have wired
    let m = c.obj::<Menu>();
    c.ret_ptr("HSD_MemAlloc", m);
    let j = jobj(c);
    c.ret_ptr("HSD_JObjLoadJoint", j);
    let g = gobj(c, m.cast(), j.cast());
    c.ret_ptr("GObj_Create", g);
    unsafe {
        flow(c);
        tu::mn_804D6BB8 = c.buf(16).cast();
        tu::mnDeflicker_804A08B8.joint = c.buf(16).cast();
        tu::mnDeflicker_804A08B8.animjoint = c.buf(16).cast();
        tu::mnDeflicker_804A08B8.matanim_joint = c.buf(16).cast();
        tu::mnDeflicker_804A08B8.shapeanim_joint = c.buf(16).cast();
    }
    rets(c);
    let g = c.zobj::<HSD_GObj>();
    c.arg(g);
}

#[test]
fn mndeflicker_vs_c() {
    let funcs = [
        fd!("mnDeflicker_8024A168", "p", "v", 200, b_input, |a| { tu::mnDeflicker_8024A168(p(a, 0)); 0 }),
        fd!("mnDeflicker_8024A2E8", "p", "v", 40, b_gobj, |a| { tu::mnDeflicker_8024A2E8(p(a, 0)); 0 }),
        fd!("mnDeflicker_8024A344", "p", "v", 60, b_gobj, |a| { tu::mnDeflicker_8024A344(p(a, 0)); 0 }),
        fd!("mnDeflicker_8024A3E8", "p", "v", 100, b_gobj, |a| { tu::mnDeflicker_8024A3E8(p(a, 0)); 0 }),
        fd!("mnDeflicker_8024A4BC", "p", "v", 40, b_create, |a| { tu::mnDeflicker_8024A4BC(p(a, 0)); 0 }),
        fd!("mnDeflicker_8024A6C4", "p", "v", 40, b_create, |a| { tu::mnDeflicker_8024A6C4(p(a, 0)); 0 }),
    ];
    let fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("mnDeflicker_8024A168", tu::mnDeflicker_8024A168 as usize),
        ("mnDeflicker_8024A2E8", tu::mnDeflicker_8024A2E8 as usize),
        ("mnDeflicker_8024A344", tu::mnDeflicker_8024A344 as usize),
        ("mnDeflicker_8024A3E8", tu::mnDeflicker_8024A3E8 as usize),
        ("mnDeflicker_8024A4BC", tu::mnDeflicker_8024A4BC as usize),
        ("mnDeflicker_8024A6C4", tu::mnDeflicker_8024A6C4 as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_Free", ext::HSD_Free as usize),
    ];
    let _ = (addr_of!(tu::mn_804D6BB8), null_mut::<c_void>());
    let mut globals = tu::oracle_globals();
    globals.extend(tu::oracle_globals_ext());
    // the C oracle defines these externs zero-initialised; the re-exported owners (mnmain, gobj) carry their own initialisers
    for g in tu::oracle_globals_ext() { unsafe { core::ptr::write_bytes(g.addr, 0, g.size) } }
    run_tu(&TuOracle { tu: "mndeflicker", funcs: &funcs, globals, fn_table });
}
