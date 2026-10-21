//! src/tu/ftCo_HammerTurn.rs vs the C (gen/cref/tu/ftCo_HammerTurn.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_HammerTurn`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_HammerTurn as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

static mut FR: f32 = 0.0;
static mut FL: u32 = 0;
static mut PART: u32 = 0;
static mut PARTS: *mut FighterBone = core::ptr::null_mut();
static mut JO: *mut HSD_JObj = core::ptr::null_mut();
fn isf(p: &mut Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }

#[test]
fn ftCo_HammerTurn_matches_c() {
    init();
    unsafe {
        PARTS = Box::leak(Box::new([FighterBone::zeroed(), FighterBone::zeroed(), FighterBone::zeroed(), FighterBone::zeroed()])).as_mut_ptr();
        JO = Box::leak(Box::new([HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed(), HSD_JObj::zeroed()])).as_mut_ptr();
        hk::ftCo_800C54C4 = Some(|p| { t(2); t(isf(p)); FL });
        hk::ftCo_800C548C = Some(|p| { t(3); t(isf(p)); FR });
        hk::ftCo_800C4E94 = Some(|p| { t(4); t(isf(p)) });
        hk::ftCo_800C4F64 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_800C97A8 = Some(|g| { t(6); t(is_go(g)); sr(1) });
        hk::ftCo_Turn_Enter = Some(|g, ms, fl, a, bb, c| { t(7); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)) });
        hk::ftCo_800C4ED8 = Some(|g| { t(8); t(is_go(g)) });
        hk::ftParts_GetBoneIndex = Some(|p, part| { t(9); t(isf(p)); t(part); PART });
        hk::ftPartSetRotY = Some(|p, part, r| { t(10); t(isf(p)); t(part as u32); t(b(r)) });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(11); t(j.offset_from(JO) as u32) });
        hk::ftCo_800C5A50 = Some(|g| { t(12); t(is_go(g)); sr(2) });
        hk::ft_80084F3C = Some(|g| { t(13); t(is_go(g)) });
        hk::ftCo_HammerWait_Coll = Some(|g| { t(14); t(is_go(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_HammerTurn.txt"));
    let mask = (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY) as u32;
    for op in 0..6u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                FR = rw(); FL = rnd(); PART = rnd() % 4;
                for i in 0..4 {
                    let j = &mut *JO.add(i);
                    *j = HSD_JObj::zeroed();
                    (*PARTS.add(i)).x4_jobj2 = j;
                    j.flags = rnd() & mask;
                    j.rotate.y = rg();
                }
            }
            let fr = f();
            fr.parts = unsafe { PARTS };
            let fa = if rnd() & 1 != 0 { (rnd() % 12) as i32 as f32 } else { rg() * 4.0 };
            unsafe { fr.mv.co.walk.fast_anim_frame = fa; }
            fr.facing_dir = rg();
            fr.set_x2222_b6((rnd() & 1) as u8);
            cd().x6B0 = (rnd() % 20) as i32 + 1;
            let g = go();
            let ret = match op {
                0 => m::ftCo_800C576C(g) as u32,
                1 => { m::ftCo_800C57B4(g); 0 }
                2 => { m::ftCo_HammerTurn_Anim(g); 0 }
                3 => { m::ftCo_HammerTurn_IASA(g); 0 }
                4 => { m::ftCo_HammerTurn_Phys(g); 0 }
                _ => { m::ftCo_HammerTurn_Coll(g); 0 }
            };
            let mut h = Hs::new();
            unsafe {
                h.m(b(f().mv.co.walk.fast_anim_frame)); h.m(b(f().facing_dir));
                for i in 0..4 { h.m(b((*JO.add(i)).rotate.y)); }
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 12000);
}
