//! src/tu/ftaction.rs (the fighter subaction command handlers and runners) vs the C (gen/cref/tu/ftaction_ops.c: the
//! whole TU, shipping contraction, recording stubs, a random fighter world, command words filled through the union
//! view each handler reads). Regenerate: `bash rust/melee_fighter/gen/mtu/regen_ftaction.sh`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_variables, unused_parens, unused_unsafe, unused_mut, unused_imports, clippy::all)]
mod mtu_h;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::tu::ftaction as m;
use melee_fighter::types::*;
use mtu_h::*;

mod ftaction_stubs;
mod ftaction_state {
    use super::*;
    include!("ftaction_state/gen.rs");
}
use ftaction_state::*;

const NG: usize = 3;
const NJ: usize = 8;
const NW: usize = 512;
static mut GO: *mut [HSD_GObj; NG] = null_mut();
static mut JO: *mut [HSD_JObj; NJ] = null_mut();
static mut FT: *mut [Fighter; NG] = null_mut();
static mut FDATA: *mut [ftData; NG] = null_mut();
static mut PARTS: *mut [[FighterBone; 256]; NG] = null_mut();
static mut W: *mut [CmdUnion; NW] = null_mut();
static mut CI: *mut CommandInfo = null_mut();
static mut PROC: *mut HSD_GObjProc = null_mut();
static mut PADB: *mut [[u8; 2048]; 72] = null_mut();
static mut CUR_PROC: *mut HSD_GObjProc = null_mut();

fn pick_HSD_GObj() -> *mut HSD_GObj { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*GO)[((r >> 8) as usize) % NG] } } }
fn pick_HSD_JObj() -> *mut HSD_JObj { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*JO)[((r >> 8) as usize) % NJ] } } }
fn pick_Fighter() -> *mut Fighter { let r = rnd(); unsafe { if r % 6 == 0 { null_mut() } else { &raw mut (*FT)[((r >> 8) as usize) % NG] } } }
fn pick_Item() -> *mut Item { null_mut() }
fn pick_HSD_DObj() -> *mut HSD_DObj { null_mut() }
pub unsafe extern "C" fn cbev1(g: *mut HSD_GObj) { t(7001); t(pi(g)); }
pub unsafe extern "C" fn cbev2(g: *mut HSD_GObj) { t(7002); t(pi(g)); }
pub unsafe extern "C" fn cbp1(g: *mut HSD_GObj) -> i32 { t(7003); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbp2(g: *mut HSD_GObj) -> i32 { t(7004); t(pi(g)); (fv() & 1) as i32 }
pub unsafe extern "C" fn cbev3(g: *mut HSD_GObj, hh: *mut HSD_GObj) { t(7005); t(pi(g)); t(pi(hh)); }
fn pick_fn(kind: i32) -> usize {
    let r = rnd();
    if r & 3 == 0 { return 0; }
    match kind {
        1 => if r & 4 != 0 { cbev1 as usize } else { cbev2 as usize },
        2 => if r & 4 != 0 { cbp1 as usize } else { cbp2 as usize },
        3 => cbev3 as usize,
        _ => 0,
    }
}

fn bz<T>() -> *mut T { Box::leak(Box::new(unsafe { core::mem::zeroed::<T>() })) }

unsafe fn install_hooks() {
    use m::ext::hook as k;
    k::HSD_GObj_CurrentInvokedProc = Some(|| CUR_PROC);
    k::Command_Execute = Some(|cmd, code| {
        let r = fv();
        t(1900); t(pi(cmd)); t(code); t((*cmd).anon_8.u.offset_from((*W).as_ptr()) as u32); t((*cmd).timer.to_bits()); t(r);
        if r & 4 != 0 { (*cmd).timer = (r % 7) as f32 + 1.0; }
        if r & 2 != 0 { (*cmd).anon_8.u = (*cmd).anon_8.u.add(1); }
        if code < 10 { if r & 6 == 0 { (*cmd).anon_8.u = (*cmd).anon_8.u.add(1); } return true; }
        r & 1 != 0
    });
    k::Geno_FtCmd = Some(|g, cmd, mode| { t(1901); t(pi(g)); t(pi(cmd)); t(mode as u32); (*cmd).anon_8.u = (*cmd).anon_8.u.add(1); });
}

unsafe fn world() {
    *GO = core::mem::zeroed();
    *JO = core::mem::zeroed();
    *FT = core::mem::zeroed();
    *FDATA = core::mem::zeroed();
    for i in 0..NG { fill_Fighter(&mut (*FT)[i]); }
    for i in 0..NJ { fill_HSD_JObj(&mut (*JO)[i]); }
    for i in 0..NG { fill_HSD_GObj(&mut (*GO)[i]); }
    for i in 0..NG {
        (*GO)[i].user_data = &raw mut (*FT)[i] as *mut c_void;
        (*GO)[i].hsd_obj = &raw mut (*JO)[i] as *mut c_void;
        (*GO)[i].classifier = HSD_GOBJ_CLASS_FIGHTER as _;
        (*FT)[i].gobj = &raw mut (*GO)[i];
        (*FT)[i].ft_data = &raw mut (*FDATA)[i];
        (*FT)[i].grab_cb = Some(cbev1); (*FT)[i].grabbed_cb = Some(cbev3);
        (*FT)[i].kind = ((*FT)[i].kind as u32 % 27) as _; (*FT)[i].player_idx %= 4; (*FT)[i].pad_port %= 4;
        for b in 0..256 { (*PARTS)[i][b] = core::mem::zeroed(); (*PARTS)[i][b].joint = &raw mut (*JO)[(i * 5 + b * 3) % NJ]; }
        (*FT)[i].parts = (*PARTS)[i].as_mut_ptr();
        // the ftData pointer words: one pad block each (the C driver's order)
        { let mut pk = i * 24; macro_rules! nx { () => {{ pk += 1; (*PADB)[pk - 1].as_mut_ptr() as *mut _ }}; }
          let d = &mut (*FDATA)[i];
          d.x0 = nx!(); d.ext_attr = nx!(); d.x8 = nx!(); d.xC = nx!(); d.x10 = nx!(); d.x14 = nx!(); d.x18 = nx!(); d.x1C = nx!();
          d.x20 = nx!(); d.x24 = nx!(); d.x28 = nx!(); d.x2C = nx!(); d.x30 = nx!(); d.x34 = nx!(); d.x38 = nx!(); d.x3C = nx!();
          d.x40 = nx!(); d.x44 = nx!(); d.x48_items = nx!(); d.x4C_sfx = nx!(); d.x50 = nx!(); d.x54 = nx!(); d.x58 = nx!(); d.x5C = nx!(); }
    }
    for i in 0..NJ {
        (*JO)[i].parent = if i != 0 { &raw mut (*JO)[(i - 1) / 2] } else { null_mut() };
        (*JO)[i].child = if 2 * i + 1 < NJ { &raw mut (*JO)[2 * i + 1] } else { null_mut() };
        (*JO)[i].next = if (i & 1) != 0 && i + 1 < NJ { &raw mut (*JO)[i + 1] } else { null_mut() };
    }
    (*PROC).s_link = (fv() % 20) as _;
    CUR_PROC = if fv() & 1 != 0 { PROC } else { null_mut() };
    *W = core::mem::zeroed();
    *CI = core::mem::zeroed();
    (*CI).anon_8.u = (*W).as_mut_ptr();
    (*CI).timer = rw();
    (*CI).frame_count = rg();
}
unsafe fn hworld() {
    let w0 = (*W).as_mut_ptr();
    hm((*CI).anon_8.u.offset_from(w0) as u32); hm((*CI).timer.to_bits()); hm((*CI).frame_count.to_bits());
    for i in 0..NG {
        let u = (*FT)[i].x3E4_fighterCmdScript.anon_8.u;
        let inw = u >= w0 && u < w0.add(NW);
        hm(if inw { u.offset_from(w0) as u32 } else { 0xFFFF });
        if inw { (*FT)[i].x3E4_fighterCmdScript.anon_8.u = null_mut(); }
    }
    for i in 0..NG { hash_Fighter(&(*FT)[i]); }
    for i in 0..NJ { hash_HSD_JObj(&(*JO)[i]); }
    for i in 0..NG { hash_HSD_GObj(&(*GO)[i]); }
}
unsafe fn stream() {
    let mut i = 0usize;
    while i < NW { (*W)[i].Command_09.set_id((10 + rnd() % 50) as _); i += 8; }
}

#[test]
fn ftaction_matches_c() {
    unsafe {
        GO = bz(); JO = bz(); FT = bz(); FDATA = bz(); PARTS = bz(); W = bz(); CI = bz(); PROC = bz(); PADB = bz();
        preg(GO, core::mem::size_of_val(&*GO));
        preg(JO, core::mem::size_of_val(&*JO));
        preg(FT, core::mem::size_of_val(&*FT));
        preg(FDATA, core::mem::size_of_val(&*FDATA));
        preg(CI, core::mem::size_of_val(&*CI));
        preg(PROC, core::mem::size_of_val(&*PROC));
        freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize);
        ftaction_stubs::install();
        install_hooks();
    }
    let mut c = Check::new(include_str!("data/tu_ftaction_ops.txt"));
    const NOPS: u32 = 18;
    for op in 0..NOPS {
        for k in 0..600u32 {
            case(op, k);
            if std::env::var("TUV").is_ok() { eprintln!("op {op} k {k}"); }
            let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| unsafe {
                world();
                let gi = (rnd() as usize) % NG;
                let g = &raw mut (*GO)[gi];
                let fp = &raw mut (*FT)[gi];
                let ci = CI;
                match op {
                    0 => {
                        fill_spawn_hitbox_0(&mut (*W)[0].create_hitbox_0); fill_spawn_hitbox_1(&mut (*W)[1].create_hitbox_1);
                        fill_spawn_hitbox_3(&mut (*W)[3].create_hitbox_3);
                        // the skip test reads word 3's create_hitbox_3.ignore_thrown_fighters on both sides since the 363720e12 sync
                        fill_spawn_hitbox_4(&mut (*W)[4].create_hitbox_4); fill_spawn_hitbox_5(&mut (*W)[5].create_hitbox_5);
                        if (*W)[0].create_hitbox_0.id() >= 4 { (*fp).x914[3].state = HitCapsule_Disabled; }
                        m::ftAction_8007121C(g, &mut *ci);
                    }
                    1 => { fill_set_hitbox_x42_b57(&mut (*W)[0].set_hitbox_x42_b57); let i = (*W)[0].set_hitbox_x42_b57.idx(); (*W)[0].set_hitbox_x42_b57.set_idx(i % 8); if (*W)[0].set_hitbox_x42_b57.idx() >= 4 { (*fp).x914[0].set_x43_b7(1); (*fp).x914[0].state = HitCapsule_Enabled; } m::ftAction_80071708(g, &mut *ci); }
                    2 => { fill_set_throw_flags(&mut (*W)[0].set_throw_flags); m::ftAction_800718A4(g, &mut *ci); }
                    3 => { fill_set_jab_combo(&mut (*W)[0].set_jab_combo); m::ftAction_80071AE8(g, &mut *ci); }
                    4 => {
                        fill_set_throw_hitbox_0(&mut (*W)[0].set_throw_hitbox_0); fill_set_throw_hitbox_1(&mut (*W)[1].set_throw_hitbox_1);
                        fill_set_throw_hitbox_2(&mut (*W)[2].set_throw_hitbox_2);
                        let i = (*W)[0].set_throw_hitbox_0.idx(); (*W)[0].set_throw_hitbox_0.set_idx(i % 2);
                        m::ftAction_80071E04(g, &mut *ci);
                    }
                    5 => { fill_unk27(&mut (*W)[0].unk27); m::ftAction_80071F34(g, &mut *ci); }
                    6 => { fill_set_tex_anim(&mut (*W)[0].set_tex_anim); m::ftAction_800726F4(g, &mut *ci); }
                    7 => { fill_part_anim(&mut (*W)[0].part_anim); m::ftAction_800727C8(g, &mut *ci); }
                    8 => { fill_unk10(&mut (*W)[0].unk10); m::ftAction_800728F8(g, &mut *ci); }
                    9 => { fill_unk19(&mut (*W)[0].unk19); m::ftAction_80072C6C(g, &mut *ci); }
                    10 => m::ftAction_8007296C(g, &mut *ci),
                    11 => m::ftAction_80072E24(g, &mut *ci),
                    12 => m::ftAction_800717D8(g, &mut *ci),
                    13 | 14 | 15 => {
                        stream();
                        (*fp).x3E4_fighterCmdScript.anon_8.u = (*W).as_mut_ptr();
                        (*fp).x3E4_fighterCmdScript.timer = (*ci).timer;
                        match op { 13 => m::ftAction_80073240(g), 14 => m::ftAction_80073354(g), _ => m::ftAction_8007349C(g) }
                    }
                    16 => { stream(); m::ftAction_80071820(g, &mut *ci); }
                    17 => { stream(); m::ftAction_8007320C(g, &mut *ci); }
                    _ => unreachable!(),
                }
                hworld();
            }));
            if res.is_err() { c.assert_line(op, k); continue; }
            unsafe { c.line(op, k, 0); }
        }
    }
    assert_eq!(c.done(), NOPS * 600);
}
