//! Smoke test for src/tu/ftdynamics.rs: dynamic-bone toggling over a hand-built fighter.
#![allow(static_mut_refs)]
use melee_fighter::tu::ftdynamics as d;
use melee_fighter::types::*;

static mut CLEARS: u32 = 0;
fn clear(_j: *mut HSD_JObj, f: u32) { assert_eq!(f, 0x20000); unsafe { CLEARS += 1 } }
fn dirty(_j: *mut HSD_JObj) {}
fn not_mex(_: i32) -> i32 { 0 }

#[test]
fn toggle_bones() {
    unsafe {
        d::ext::HSD_JOBJ_CLEAR_FLAGS = Some(clear);
        d::ext::HSD_JOBJ_SET_MTX_DIRTY_SUB = Some(dirty);
        d::ext::GENO_DEFINE_TEMPLATE_SKELETON = Some(not_mex);
        d::ext::FTKB_MEX_KIRBY_LIKE = Some(not_mex);
    }
    let mut j: std::vec::Vec<HSD_JObj> = (0..3).map(|_| HSD_JObj::zeroed()).collect();
    let jp = j.as_mut_ptr();
    unsafe {
        (*jp).child = jp.add(1);
        (*jp.add(1)).child = jp.add(2);
        (*jp.add(1)).mtx[0][3] = 5.0;
        (*jp.add(1)).mtx[2][3] = -2.0;
    }
    let mut parts: std::vec::Vec<FighterBone> = (0..8).map(|_| FighterBone::zeroed()).collect();
    for i in 0..3 { parts[4 + i].joint = unsafe { jp.add(i) }; }
    let mut dd: std::vec::Vec<DynamicsData> = (0..3).map(|_| DynamicsData::zeroed()).collect();
    let ddp = dd.as_mut_ptr();
    for i in 0..3 {
        unsafe {
            (*ddp.add(i)).desc.lb_unk0.jobj = jp.add(i);
            (*ddp.add(i)).next = if i < 2 { ddp.add(i + 1) } else { core::ptr::null_mut() };
        }
    }
    let mut bones = [BoneDynamicsTemplate::zeroed(); Ft_Dynamics_NumMax as usize];
    bones[0].bone_id = 4;
    let mut dy = ftDynamics::zeroed();
    dy.x0.dynamicsNum = 1;
    dy.x0.ftDynamicBones = bones.as_mut_ptr();
    let mut data = ftData::zeroed();
    data.x2C = &mut dy;
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.ft_data = &mut data;
    f.parts = parts.as_mut_ptr();
    f.dynamics_num = 1;
    f.kind = Ft_Kind_Mario;
    f.dynamic_bone_sets[0].dyn_desc.data = ddp;

    // arg2=1 with a count of 2: first two bones inverse (0), third flag (1).
    d::ftCo_8009CB40(&mut f, 0, 1, 2);
    assert_eq!(f.dynamic_bone_sets[0].bone_id, 2);
    let fl = |p: &FighterBone| unsafe { p.x8.x0.flags_b0() };
    assert_eq!((fl(&parts[4]), fl(&parts[5]), fl(&parts[6])), (0, 0, 1));
    assert_eq!(unsafe { CLEARS }, 1);
    // arg2=0: all bones flag 0 -> inverse 1 for k < 0? limit 0 so all take flag 0.
    d::ftCo_8009E140(&mut f, 0);
    assert_eq!(f.dynamic_bone_sets[0].bone_id, 0x100);
    assert_eq!((fl(&parts[4]), fl(&parts[5]), fl(&parts[6])), (0, 0, 0));
    // arg2=1, NULL: all set to 1; bone 5's snapshot copies its mtx column 3.
    d::ftCo_8009E140(&mut f, 1);
    assert_eq!((fl(&parts[4]), fl(&parts[5]), fl(&parts[6])), (1, 1, 1));
    assert_eq!(unsafe { dd[1].desc.lb_unk0.unk_2C.x }, 5.0);
    assert_eq!(unsafe { dd[1].desc.lb_unk0.unk_2C.z }, -2.0);

    // Tree mirror lookup.
    let mut k: std::vec::Vec<HSD_JObj> = (0..3).map(|_| HSD_JObj::zeroed()).collect();
    let kp = k.as_mut_ptr();
    unsafe { (*kp).child = kp.add(1); (*kp.add(1)).child = kp.add(2); }
    assert_eq!(d::ftCo_8009E1D4(jp, kp, unsafe { jp.add(2) }), unsafe { kp.add(2) });
    assert!(d::ftCo_8009E1D4(jp, kp, 1 as *mut HSD_JObj).is_null());

    // Unload.
    unsafe { d::ext::LB_8000FD18 = Some(|_| {}) };
    d::ftCo_UnloadDynamicBones(&mut f);
    assert_eq!(f.dynamics_num, 0);
}
