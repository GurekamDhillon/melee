//! ftparts.c (partial) smoke tests: DObj bounds check, rotation getters on native JObjs.
use melee_fighter::tu::ftparts as p;
use melee_fighter::types::*;

#[test]
fn dobj_at_bounds() {
    let mut d = [HSD_DObj::zeroed(), HSD_DObj::zeroed()];
    let mut ptrs: [*mut HSD_DObj; 2] = [&mut d[0], &mut d[1]];
    let list = DObjList { count: 2, data: ptrs.as_mut_ptr() };
    assert_eq!(p::ftParts_DObjAt(&list, 1, "t"), ptrs[1]);
    assert!(p::ftParts_DObjAt(&list, 2, "t").is_null());
    assert!(p::ftParts_DObjAt(&list, -1, "t").is_null());
    assert!(p::ftParts_DObjAt(core::ptr::null(), 0, "t").is_null());
}

#[test]
fn rot_getters_follow_quaternion_flag() {
    let mut j = HSD_JObj::zeroed();
    let mut j2 = HSD_JObj::zeroed();
    j.rotate.x = 1.0; j.rotate.y = 2.0;
    j2.rotate.x = 3.0; j2.rotate.y = 4.0;
    let mut b = FighterBone::zeroed();
    b.joint = &mut j; b.x4_jobj2 = &mut j2;
    let mut fp = Fighter::zeroed();
    fp.parts = &mut b;
    assert_eq!((p::ftPartGetRotX(&fp, 0), p::ftPartGetRotZ(&fp, 0)), (1.0, 2.0));
    unsafe { (*b.joint).flags = JOBJ_USE_QUATERNION as u32 };
    assert_eq!((p::ftPartGetRotX(&fp, 0), p::ftPartGetRotZ(&fp, 0)), (3.0, 4.0));
}

#[test]
fn vis_rows_skip_without_lookup() {
    let mut vis = FtPartsVis::zeroed();
    vis.cleared[0] = 1;
    p::ftParts_80074D7C(&mut vis, 1, core::ptr::null()); // no lookup: untouched
    p::ftParts_80074CA0(&mut vis, 0, core::ptr::null());
    assert_eq!(vis.cleared[0], 1);
}
