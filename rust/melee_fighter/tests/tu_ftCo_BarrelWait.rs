//! src/tu/ftCo_BarrelWait.rs vs the C (gen/cref/tu/ftCo_BarrelWait.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_BarrelWait`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_BarrelWait as m;
use melee_fighter::tu::ftCo_Barrel::ext::hook as bk;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut CAM: *mut CmSubject = core::ptr::null_mut();
static mut ARG2: [u8; 64] = [0; 64];
unsafe extern "C" fn eaf8(_: *mut HSD_GObj) {}
fn cbid(e: HSD_GObjEvent) -> u32 {
    let Some(p) = e else { return 0 };
    let p = p as usize;
    if p == eaf8 as usize { 1 } else if p == m::ftCo_8009EC44 as usize { 2 } else { 99 }
}
fn isf(p: *const Fighter) -> u32 { (p == f() as *const Fighter) as u32 }

#[test]
fn ftCo_BarrelWait_matches_c() {
    init();
    unsafe {
        CAM = Box::leak(Box::new(CmSubject::zeroed()));
        bk::ftCommon_8007DB58 = Some(|g| { t(1); t(is_go(g)) });
        bk::ftCo_8009750C = Some(|g| { t(2); t(is_go(g)) });
        bk::ftCo_800DD168 = Some(|g| { t(3); t(is_go(g)) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftCo_8009EAF8_fn = Some(|| Some(eaf8));
        hk::ftCommon_8007E2FC = Some(|g| { t(6); t(is_go(g)) });
        hk::ftCommon_8007E2F4 = Some(|fr, v| { t(7); t(isf(fr)); t(v as i32 as u32) });
        hk::ftCommon_8007EFC0 = Some(|fr, v| { t(8); t(isf(fr)); t(v) });
        hk::Ground_801C4DD0 = Some(|| { t(9); sr(9) });
        hk::Ground_801C4DA0 = Some(|p, o| { t(10); t((p == &mut f().cur_pos as *mut Vec3) as u32); *o = rg(); sr(10) });
        hk::mpColl_80043680 = Some(|c, v| { t(11); t((c == &mut f().coll_data as *mut CollData) as u32); t(b((*v).x)); t(b((*v).y)); t(b((*v).z)) });
        hk::ftCamera_80076064 = Some(|fr| { t(12); t(isf(fr)) });
        hk::lbColl_80008D30 = Some(|h, a| {
            t(13); t((a as *mut u8 == ARG2.as_mut_ptr()) as u32);
            (*h).unk_count = rnd() % 50; (*h).element = rnd() % 12; (*h).damage = rw();
        });
        hk::ftColl_800788D4 = Some(|g| { t(14); t(is_go(g)) });
        hk::Fighter_UnkTakeDamage_8006CC30 = Some(|fr, d| { t(15); t(isf(fr)); t(b(d)) });
        hk::ftCo_8008DCE0 = Some(|g, a, d| { t(16); t(is_go(g)); t(a as u32); t(b(d)) });
        bk::ftColl_80079EA8 = Some(|fr, _h, n| { t(17); t(isf(fr)); t(n); rw() });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_BarrelWait.txt"));
    for op in 0..7u32 {
        for k in 0..1000u32 {
            case(op, k);
            let g = go();
            unsafe { *CAM = CmSubject::zeroed(); }
            let fr = f();
            fr.x890_cameraBox = unsafe { CAM };
            let cdm = cd();
            cdm.x4DC.x = rg(); cdm.x4DC.y = rw(); cdm.x4E4.x = rg(); cdm.x4E4.y = rw(); cdm.x4E4.z = rg();
            cdm.bury_timer_unk2 = rnd();
            fr.facing_dir = rg();
            fr.cur_pos.x = rw(); fr.cur_pos.y = rg(); fr.cur_pos.z = rg();
            fr.input.pressed_buttons = rnd() & 0x3FF;
            fr.dmg.x184c_damaged_hurtbox = rnd() as i32;
            let mut pos = Vec3 { x: rw(), y: rg(), z: rw() };
            let ang = rg() * 400.0;
            unsafe {
                match op {
                    0 => m::ftCo_8009EB18(g),
                    1 => m::ftCo_BarrelWait_Anim(g),
                    2 => m::ftCo_BarrelWait_IASA(g),
                    3 => m::ftCo_BarrelWait_Phys(g),
                    4 => m::ftCo_BarrelWait_Coll(g),
                    5 => m::ftCo_8009EC44(g),
                    _ => m::ftCo_8009EC70(g, &mut pos, ARG2.as_mut_ptr() as *mut lbColl_80008D30_arg1, ang),
                }
            }
            let fr = f();
            let cam = unsafe { &*CAM };
            let mut h = Hs::new();
            h.m(cbid(fr.death2_cb)); h.m(cbid(fr.accessory1_cb));
            h.m(fr.invisible() as u32); h.m(fr.x1988 as u32); h.m(fr.bury_timer_2);
            h.m(b(cam.target_ext.h.x)); h.m(b(cam.target_ext.h.y));
            h.m(b(cam.target_ext.v.x)); h.m(b(cam.target_ext.v.y)); h.m(b(cam.target_ext.v.z));
            h.m(b(fr.cur_pos.x)); h.m(b(fr.cur_pos.y)); h.m(b(fr.cur_pos.z));
            h.m(b(fr.dmg.kb_applied)); h.m(fr.dmg.x1848_kb_angle as u32); h.m(b(fr.dmg.facing_dir_1));
            h.m(fr.dmg.x184c_damaged_hurtbox as u32); h.m(b(fr.dmg.x1854_collpos.x)); h.m(b(fr.dmg.x1854_collpos.y));
            h.m(b(fr.dmg.x1854_collpos.z)); h.m(fr.dmg.x1860_element);
            c.line(op, k, 0, h.0);
        }
    }
    assert_eq!(c.done(), 7000);
}
