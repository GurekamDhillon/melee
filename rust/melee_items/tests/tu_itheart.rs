//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/itheart.c.
//! src/tu/itheart.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py itheart`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::itheart as m;
use mtu_harness::*;
#[path = "mtu/itheart_gen.rs"]
mod g;





unsafe fn set_uv(op: u32) {
    g::UV_SpawnItem_x44_flag = 0; g::UV_SpawnItem_x45_flag = 0; g::UV_SpawnItem_x46_flag = 0; g::UV_SpawnItem_x47_flag = 0; g::UV_HSD_JObj_u = 0; g::UV_ECBSource_x4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_CommandInfo_anon_1020_x8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_x134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_xDAA = 0; g::UV_Item_xDAA_xDAA_flag = 0; g::UV_Item_xDBC = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0; g::UV_itHeart_ItemVars_xDD8 = 0; g::UV_HeartContainerAttr_x4 = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; g::UV_Item_xDD4_itemVar = 34; g::UV_HeartContainerAttr_x4 = 1; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
}

#[test]
fn itheart_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_itheart.txt"));
        let nc: [u32; 21] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..21u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(1) as *mut HSD_GObj; let a1: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a2: i32 = (fv()) as i32; ret = pi(m::it_80283AE4(a0, a1, a2)); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80283BD4(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_Spawned(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_Destroyed(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80283C7C(a0); }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion0_Anim(a0) as u32; }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_UnkMotion0_Phys(a0); }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion0_Coll(a0) as u32; }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80283DD4(a0); }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion3_Anim(a0) as u32; }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_UnkMotion3_Phys(a0); }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion3_Coll(a0) as u32; }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_PickedUp(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion2_Anim(a0) as u32; }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_UnkMotion2_Phys(a0); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_Dropped(a0); }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_EnteredAir(a0); }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion4_Anim(a0) as u32; }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_UnkMotion4_Phys(a0); }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHeart_UnkMotion4_Coll(a0) as u32; }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHeart_Logic8_EvtUnk(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8400);
    }
}
