//! Written by gen/mtu/if_drv.py from the same description as gen/mtu/tu/ithammerhead.c.
//! src/tu/ithammerhead.rs vs the C (the whole TU, shipping contraction; regenerate the expected file
//! with `python rust/melee_world/gen/mtu/run_if.py ithammerhead`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_items::tu::ithammerhead as m;
use mtu_harness::*;
#[path = "mtu/ithammerhead_gen.rs"]
mod g;





unsafe fn set_uv(op: u32) {
    g::UV_SpawnItem_x44_flag = 0; g::UV_SpawnItem_x45_flag = 0; g::UV_SpawnItem_x46_flag = 0; g::UV_SpawnItem_x47_flag = 0; g::UV_HSD_JObj_u = 0; g::UV_ECBSource_x4 = 0; g::UV_CommandInfo_anon_8 = 0; g::UV_CommandInfo_anon_1020_x8 = 0; g::UV_ColorOverlay_x28_colanim = 0; g::UV_HitCapsule_x134 = 0; g::UV_Item_xD90 = 0; g::UV_Item_xDAA = 0; g::UV_Item_xDAA_xDAA_flag = 0; g::UV_Item_xDBC = 0; g::UV_Item_xDCE_flag = 0; g::UV_Item_xDCF_flag = 0; g::UV_Item_xDD0_flag = 0; g::UV_Item_xDD1_flag = 0; g::UV_Item_xDD2_flag = 0; g::UV_Item_xDD3_flag = 0; g::UV_Item_xDD4_itemVar = 0;
    match op {
        0 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        1 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        2 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        3 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        4 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        5 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        6 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        7 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        8 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        9 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        10 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        11 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        12 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        13 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        14 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        15 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        16 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        17 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        18 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        19 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        20 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        21 => { g::UV_Item_xDCE_flag = 1; g::UV_Item_xDCF_flag = 1; g::UV_Item_xDD0_flag = 1; g::UV_Item_xDD1_flag = 1; g::UV_Item_xDD2_flag = 1; g::UV_Item_xDD3_flag = 1; g::UV_SpawnItem_x44_flag = 1; g::UV_SpawnItem_x45_flag = 1; g::UV_SpawnItem_x46_flag = 1; g::UV_SpawnItem_x47_flag = 1; g::UV_Item_xD90 = 0; }
        _ => {}
    }
}

unsafe fn setup_extra() {
    let mut n: u32;
    let mut i: usize;
    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }
}

#[test]
fn ithammerhead_matches_c() {
    unsafe {
        g::install();
        g::preg_state();
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_ithammerhead.txt"));
        let nc: [u32; 22] = [400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400, 400];

        for op in 0..22u32 {
            for k in 0..nc[op as usize] {
                case(op, k);
                set_uv(op);
                g::reset_state();
                setup_extra();
                let mut ret = 0u32;
                match op {
                    0 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(1) as *mut HSD_GObj; let a1: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a2: *mut Vec = g::pick_Vec3(0) as *mut Vec; let a3: f32 = (rg()) as f32; m::it_80299C48(a0, a1, a2, a3); }
                    1 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerHead_Logic40_Spawned(a0); }
                    2 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80299D7C(a0); }
                    3 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerHead_Logic40_PickedUp(a0); }
                    4 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerhead_UnkMotion1_Anim(a0) as u32; }
                    5 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerhead_UnkMotion1_Phys(a0); }
                    6 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerHead_Logic40_Dropped(a0); }
                    7 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerHead_Logic40_Thrown(a0); }
                    8 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerhead_UnkMotion2_Anim(a0) as u32; }
                    9 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerhead_UnkMotion2_Phys(a0); }
                    10 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerhead_UnkMotion2_Coll(a0) as u32; }
                    11 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80299F94(a0); }
                    12 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::it_80299FB4(a0); }
                    13 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerhead_UnkMotion3_Anim(a0) as u32; }
                    14 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerhead_UnkMotion3_Phys(a0); }
                    15 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerhead_UnkMotion3_Coll(a0) as u32; }
                    16 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerHead_Logic40_DmgDealt(a0) as u32; }
                    17 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerHead_Logic40_Clanked(a0) as u32; }
                    18 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerHead_Logic40_HitShield(a0) as u32; }
                    19 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerHead_Logic40_Reflected(a0) as u32; }
                    20 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; ret = m::itHammerHead_Logic40_DmgReceived(a0) as u32; }
                    21 => {  let a0: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; let a1: *mut HSD_GObj = g::pick_HSD_GObj(0) as *mut HSD_GObj; m::itHammerHead_Logic40_EvtUnk(a0, a1); }
                    _ => unreachable!(),
                }
                g::hash_state();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 8800);
    }
}
