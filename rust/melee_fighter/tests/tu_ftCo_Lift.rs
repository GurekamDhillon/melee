//! src/tu/ftCo_Lift.rs vs the C (gen/cref/tu/ftCo_Lift.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Lift`).
#![allow(static_mut_refs, non_snake_case)]
#[path = "../../melee_core/test_support/fnsym.rs"]
mod fnsym;
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Lift as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut PARTS: *mut FighterBone = core::ptr::null_mut();

fn jo(i: usize) -> *mut HSD_JObj { unsafe { JO.add(i) } }
fn itid(p: *mut Item_GObj) -> u32 { if p.is_null() { 0 } else if p as *mut HSD_GObj == igo() { 1 } else { 99 } }
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == f() as *const Fighter) as u32 }
fn cbid(e: HSD_GObjEvent) -> u32 {
    let Some(p) = e else { return 0 };
    if fnsym::same(p as usize, m::ftCo_800974C4_cb as usize) { 1 } else { 99 }
}

macro_rules! v { ($($n:ident = $id:expr),*) => { unsafe { $( hk::$n = Some(|g| { t($id); t(is_go(g)) }); )* } } }

#[test]
fn ftCo_Lift_matches_c() {
    init();
    unsafe {
        JO = Box::leak((0..4).map(|_| HSD_JObj::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice()).as_mut_ptr();
        PARTS = Box::leak((0..4).map(|_| FighterBone::zeroed()).collect::<std::vec::Vec<_>>().into_boxed_slice()).as_mut_ptr();
    }
    v!(ft_80084F3C = 3, ft_80085088 = 4, ftCo_80090780 = 6, ftCo_Fall_Enter = 7, ftAnim_8006EBA4 = 9);
    unsafe {
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(1); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftCo_80094EA4 = Some(|g| { t(2); t(is_go(g)); sr(2) & sr(18) });
        hk::Item_8026ABD8 = Some(|i, p, fv| { t(5); t(itid(i)); t(b((*p).x)); t(b((*p).y)); t(b((*p).z)); t(b(fv)) });
        hk::ft_8008403C = Some(|g, cb| { t(8); t(is_go(g)); t(if fnsym::same(cb as usize, m::ftCo_80096E68 as usize) { 2 } else { 99 }) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(10); t(is_go(g)); sr(10) });
        hk::ftParts_GetBoneIndex = Some(|fr, p| { t(11); t(isf(fr)); t(p as u32); (rnd() % 4) as i32 });
        hk::ftPartGetRotZ = Some(|fr, p| { t(12); t(isf(fr)); t(p as u32); rw() });
        hk::ftPartSetRotY = Some(|fr, p, v| { t(13); t(isf(fr)); t(p as u32); t(b(v)) });
        hk::itIsHeavy = Some(|i| { t(14); t(itid(i)); sr(14) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Lift.txt"));
    for op in 0..23u32 {
        for k in 0..1000u32 {
            case(op, k);
            unsafe {
                for i in 0..4 {
                    *jo(i) = HSD_JObj::zeroed();
                    (*jo(i)).flags = JOBJ_MTX_INDEP_SRT as u32;
                    (*jo(i)).rotate.y = rg();
                    (*PARTS.add(i)).x4_jobj2 = jo(i);
                }
            }
            let fr = f();
            fr.parts = unsafe { PARTS };
            fr.input.lstick[0].x = rg();
            fr.facing_dir = if rnd() % 3 == 0 { rg() } else if rnd() & 1 != 0 { 1.0 } else { -1.0 };
            cd().x228 = rg(); cd().x22C = rg();
            cd().x230 = if rnd() & 1 != 0 { (rnd() % 12) as f32 } else { rw() };
            unsafe {
                fr.mv.co.lift.x0 = (rnd() & 1) as i32;
                fr.mv.co.lift.x4 = if rnd() & 1 != 0 { (rnd() % 12) as f32 } else { rw() };
                fr.mv.co.lift.x8 = (rnd() & 1) as i32;
            }
            fr.set_x2222_b6((rnd() & 1) as u8);
            fr.gr_vel = rw();
            fr.item_gobj = if rnd() & 1 != 0 { igo() as *mut Item_GObj } else { core::ptr::null_mut() };
            let g = go();
            let mut ret = 0u32;
            match op {
                0 => m::ftCo_80096D9C(g),
                1 => m::ftCo_LiftWait_Anim(g),
                2 => m::ftCo_LiftWait_IASA(g),
                3 => m::ftCo_LiftWait_Phys(g),
                4 => m::ftCo_80096E68(g),
                5 => m::ftCo_LiftWait_Coll(g),
                6 => ret = m::ftCo_80096EF8(g) as u32,
                7 => m::ftCo_80096F48(g),
                8 => m::ftCo_LiftWalk_Anim(g),
                9 => m::ftCo_LiftWalk_IASA(g),
                10 => m::ftCo_LiftWalk_Phys(g),
                11 => m::ftCo_LiftWalk_Coll(g),
                12 => ret = m::ftCo_800970E0(g) as u32,
                13 => m::ftCo_80097130(g),
                14 => m::ftCo_LiftTurn_Anim(g),
                15 => m::ftCo_LiftTurn_IASA(g),
                16 => m::ftCo_LiftTurn_Phys(g),
                17 => m::ftCo_LiftTurn_Coll(g),
                18 => m::ftCo_800974C4(g),
                19 => m::ftCo_8009750C(g),
                20 => { m::ftCo_LiftTurn_Anim(g); m::ftCo_LiftTurn_Anim(g); m::ftCo_LiftTurn_Anim(g) }
                21 => { m::ftCo_80097130(g); m::ftCo_LiftTurn_Anim(g) }
                _ => { m::ftCo_80096F48(g); m::ftCo_80096F48(g) }
            }
            let fr = f();
            let mut h = Hs::new();
            unsafe {
                h.m(fr.mv.co.lift.x0 as u32); h.m(b(fr.mv.co.lift.x4)); h.m(fr.mv.co.lift.x8 as u32);
            }
            h.m(cbid(fr.take_dmg_cb)); h.m(b(fr.facing_dir)); h.m(b(fr.gr_vel));
            for i in 0..4 { h.m(b(unsafe { (*jo(i)).rotate.y })); }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 23000);
}
