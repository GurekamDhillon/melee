//! src/tu/ftcoll.rs (the world-level hit / hurt / shield / absorb / wind passes) vs the C (gen/cref/tu/ftcoll_ops.c:
//! the whole TU, shipping contraction, recording stubs, the echo layer as the identity, three random fighters in the
//! fighter plink). Regenerate: `bash rust/melee_fighter/gen/mtu/regen_ftcoll.sh`.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_variables, unused_parens, unused_unsafe, unused_mut, unused_imports, clippy::all)]
mod mtu_h;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftcoll as m;
use melee_fighter::types::*;
use mtu_h::*;

mod ftcoll_stubs;
mod ftcoll_state {
    use super::*;
    include!("ftcoll_state/gen.rs");
}
use ftcoll_state::*;

const NG: usize = 3;
const NJ: usize = 8;
static mut GO: *mut [HSD_GObj; NG] = null_mut();
static mut JO: *mut [HSD_JObj; NJ] = null_mut();
static mut FT: *mut [Fighter; NG] = null_mut();
static mut FDATA: *mut [ftData; NG] = null_mut();
static mut PARTS: *mut [[FighterBone; 64]; NG] = null_mut();
static mut CD: *mut ftCommonData = null_mut();
static mut ICD: *mut ItemCommonData = null_mut();
static mut D6524: *mut Fighter_804D6524_t = null_mut();
static mut PLINKV: [*mut HSD_GObj; 16] = [null_mut(); 16];
static mut PADB: *mut [[u8; 2048]; 72] = null_mut();

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
    use m::ext as k;
    // the echo layer: the identity (4 capsules = fp->x914)
    k::SCRIPTGAME_ECHO_CAPSULE_COUNT = Some(|_| 4);
    k::SCRIPTGAME_ECHO_CAPSULE = Some(|f, i| &raw mut (*f).x914[i as usize]);
    k::SCRIPTGAME_ECHO_IS_CAPSULE = Some(|_| 0);
    k::SCRIPTGAME_ECHO_BLOCKED = Some(|_, _| 0);
    k::SCRIPTGAME_ECHO_SAME_GROUP = Some(|a, b| ((*a).x4 == (*b).x4) as i32);
    k::SCRIPTGAME_ECHO_TARGET_ALLOWED = Some(|_, _| 0);
    k::SCRIPTGAME_ECHO_CONNECTED = Some(|_, _| t(2040));
    k::SCRIPTGAME_ECHO_PREPARE = Some(|| t(2041));
    k::SCRIPTGAME_ECHO_HIT_INDEX = Some(|f, hh| { t(2042); hh.offset_from((*f).x914.as_ptr()) as i32 });
    k::SCRIPTGAME_ECHO_REPORT_CONTEXT = Some(|_, _, _| { t(2043); 1 });
    k::SCRIPTGAME_ECHO_CREDIT = Some(|_, _, _, d| { t(2044); t(d.to_bits()); (fv() & 1) as i32 });
    k::SCRIPTGAME_ECHO_SOURCE_X = Some(|_, fb| fb);
    k::EFSYNC_SPAWN = Some(|id, _g, _pos, _arg| { t(2050); t(id as u32); });
    h::PLINK_HEAD = Some(|link| PLINKV[link as usize]);
}

unsafe fn world() {
    *GO = core::mem::zeroed();
    *JO = core::mem::zeroed();
    *FT = core::mem::zeroed();
    *FDATA = core::mem::zeroed();
    *CD = core::mem::zeroed();
    *ICD = core::mem::zeroed();
    for i in 0..NG { fill_Fighter(&mut (*FT)[i]); }
    for i in 0..NJ { fill_HSD_JObj(&mut (*JO)[i]); }
    for i in 0..NG { fill_HSD_GObj(&mut (*GO)[i]); }
    for i in 0..NG {
        (*GO)[i].user_data = &raw mut (*FT)[i] as *mut c_void;
        (*GO)[i].hsd_obj = &raw mut (*JO)[i] as *mut c_void;
        (*GO)[i].classifier = HSD_GOBJ_CLASS_FIGHTER as _;
        (*GO)[i].next = if i + 1 < NG { &raw mut (*GO)[i + 1] } else { null_mut() };
        (*FT)[i].gobj = &raw mut (*GO)[i];
        (*FT)[i].ft_data = &raw mut (*FDATA)[i];
        (*FT)[i].grab_cb = Some(cbev1); (*FT)[i].grabbed_cb = Some(cbev3);
        (*FT)[i].kind = ((*FT)[i].kind as u32 % 27) as _; (*FT)[i].player_idx %= 4; (*FT)[i].pad_port %= 4; (*FT)[i].x166C %= 2;
        for b in 0..64 { (*PARTS)[i][b] = core::mem::zeroed(); (*PARTS)[i][b].joint = &raw mut (*JO)[(i * 5 + b * 3) % NJ]; }
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
    fill_ftCommonData(&mut *CD);
    h::P_FT_COMMON_DATA = CD;
    m::ext::it_804D6D28 = ICD;
    *D6524 = core::mem::zeroed(); (*D6524).x0 = rg(); (*D6524).x4 = rg();
    melee_fighter::tu::ftchangeparam::ext::Fighter_804D6524 = D6524;
    for i in 0..16 { PLINKV[i] = null_mut(); }
    PLINKV[HSD_GOBJ_PLINK_FIGHTER as usize] = &raw mut (*GO)[0];
    m::ext::ScriptStageHostHidden = (fv() & 1) as i32;
}
unsafe fn hworld() {
    for i in 0..NG { hash_Fighter(&(*FT)[i]); }
    for i in 0..NJ { hash_HSD_JObj(&(*JO)[i]); }
    for i in 0..NG { hash_HSD_GObj(&(*GO)[i]); }
}

#[test]
fn ftcoll_matches_c() {
    unsafe {
        GO = bz(); JO = bz(); FT = bz(); FDATA = bz(); PARTS = bz(); CD = bz(); ICD = bz(); D6524 = bz(); PADB = bz();
        preg(GO, core::mem::size_of_val(&*GO));
        preg(JO, core::mem::size_of_val(&*JO));
        preg(FT, core::mem::size_of_val(&*FT));
        preg(FDATA, core::mem::size_of_val(&*FDATA));
        freg(cbev1 as usize); freg(cbev2 as usize); freg(cbp1 as usize); freg(cbp2 as usize); freg(cbev3 as usize);
        ftcoll_stubs::install();
        install_hooks();
    }
    let mut c = Check::new(include_str!("data/tu_ftcoll_ops.txt"));
    const NOPS: u32 = 15;
    for op in 0..NOPS {
        for k in 0..500u32 {
            case(op, k);
            if std::env::var("TUV").is_ok() { eprintln!("op {op} k {k}"); }
            let mut ret = 0u32;
            let res = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| unsafe {
                world();
                let gi = (rnd() as usize) % NG;
                let g = &raw mut (*GO)[gi];
                let fp = &raw mut (*FT)[gi];
                match op {
                    0 => m::ftcoll_800764dc(g),
                    1 => m::ftcoll_80078a2c(g),
                    2 => m::ftcoll_80078c70(g),
                    3 => m::ftcoll_8007925c(g),
                    4 => m::ftcoll_8007ab48(g),
                    5 => m::ftcoll_8007ab80(g),
                    6 => { let i = (rnd() % 4) as usize; m::ftcoll_8007ad18(&*fp, &mut (*fp).x914[i]); }
                    7 => m::ftcoll_8007ae80(g),
                    8 => m::ftcoll_8007af60(g),
                    9 => m::ftcoll_8007b320(g),
                    10 => { let mut w: Vec3 = core::mem::zeroed(); m::ftcoll_get_wind_offset_vec(g, &mut w); hm(w.x.to_bits()); hm(w.y.to_bits()); hm(w.z.to_bits()); }
                    11 => m::ftcoll_8007ba0c(g),
                    12 => m::ftcoll_8007bac0(g),
                    13 => ret = m::ftcoll_8007bbcc(g).to_bits(),
                    14 => m::ftcoll_8007bc90(g),
                    _ => unreachable!(),
                }
                hworld();
            }));
            if res.is_err() { c.assert_line(op, k); continue; }
            unsafe { c.line(op, k, ret); }
        }
    }
    assert_eq!(c.done(), NOPS * 500);
}
