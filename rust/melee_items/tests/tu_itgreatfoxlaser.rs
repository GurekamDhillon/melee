//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/itgreatfoxlaser.c.
//! src/tu/itgreatfoxlaser.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py itgreatfoxlaser`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itgreatfoxlaser as m;
use mtu_harness::*;
#[path = "mtu/itgreatfoxlaser_gen.rs"]
mod g;





unsafe fn set_uv(op: u32) {
    g::UV_SpawnItem_x44_flag = 0; g::UV_SpawnItem_x45_flag = 0; g::UV_SpawnItem_x46_flag = 0; g::UV_SpawnItem_x47_flag = 0; g::UV_HSD_JObj_u = 0; g::UV_ECBSource_anon_4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_anon_134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_anon_daa = 0; g::UV_Item_anon_477_xDAA_flag = 0; g::UV_Item_anon_dbc = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 55; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_anon_477_xDAA_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 29; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
    for n in 0..8usize { g::P_DynamicBoneTable[0].bones[n] = &raw mut g::P_HSD_JObj[(rnd() % 6) as usize]; }
}

#[test]
fn itgreatfoxlaser_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itgreatfoxlaser.txt"));
        let nc: [u32; 11] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..11u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj;
                        // the port reads the x86-32 word over the laser's x38/x3A (see it_802EAF28); the x86-64 oracle reads the
                        // native likelike.x38 slot: check the rule, compare the oracle's value
                        let ip = (*a0).user_data as *mut Item;
                        let v = &(*ip).xDD4_itemVar.greatfoxlaser;
                        assert_eq!(m::it_802EAF28(a0), ((v.x38 as u16 as u32) | ((v.x3A as u16 as u32) << 16)) as i32);
                        ret = (*ip).xDD4_itemVar.likelike.x38 as u32; }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a2: i32 = ((0 + (rnd() % 2) as i32)) as i32; ret = pi(m::it_802EAF34(a0, a1, a2)); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802EB1EC(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802EB268(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGreatfoxlaser_UnkMotion1_Anim(a0) as u32; }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itGreatfoxlaser_UnkMotion1_Phys(a0); }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGreatFoxLaser_Logic27_DmgDealt(a0) as u32; }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGreatFoxLaser_Logic27_Clanked(a0) as u32; }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itGreatFoxLaser_Logic27_Absorbed(a0) as u32; }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::it_2725_Logic27_Reflected(a0) as u32; }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_802EB5A8(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 4400);
    }
}
