//! Port round 9: the IT_* hooks group 3 calls now default to group 5's ports
//! (melee_items::itlib, verified there against the C). This checks only the wiring: the hook
//! reads the Item behind `gobj->user_data` and returns what the itlib port returns.

use melee_fighter::hooks as h;
use melee_fighter::types::*;

#[test]
fn item_hooks_default_to_melee_items() {
    let mut attr: Box<ItemAttr> = Box::new(unsafe { core::mem::zeroed() });
    let mut it: Box<Item> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    it.xCC_item_attr = &mut *attr;
    g.user_data = (&mut *it as *mut Item).cast();
    let gp: *mut HSD_GObj = &mut *g;
    for (kind, msid, heavy) in [(It_Kind_Kuriboh, 3, 1u8), (7, 0, 0), (30, 11, 1)] {
        it.kind = kind;
        it.msid = msid;
        attr.set_x0_is_heavy(heavy);
        assert_eq!(h::it_get_kind(gp), kind);
        assert_eq!(h::it_get_motion_id(gp), msid);
        assert_eq!(h::it_is_heavy(gp), heavy != 0);
        assert_eq!(h::it_8026b6c8(gp), melee_items::itlib::it_8026b6c8(&it) as i32);
        h::it_8026bd0c(gp);
        assert_eq!(it.xDCD_flag.x0.b2(), 1);
        h::it_8026bcf4(gp);
        assert_eq!(it.xDCD_flag.x0.b2(), 0);
    }
    assert_eq!(h::it_8026b6c8(gp), 0);
}

/// Group 3 round 10: four more stubs now default to existing ports (it_80284FC4 / it_80285024
/// in melee_items::itkinds5, HSD_JObjGetTranslation / HSD_JObjMtxIsDirty in
/// melee_types::inline), plus the plbonuslib item / ftLib hooks' defaults.
#[test]
fn round10_hooks_default_to_ports() {
    let mut it: Box<Item> = Box::new(unsafe { core::mem::zeroed() });
    let mut g: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    g.user_data = (&mut *it as *mut Item).cast();
    let gp: *mut HSD_GObj = &mut *g;
    h::it_80284fc4(gp, 7, 2.5);
    let mut want: Box<Item> = Box::new(unsafe { core::mem::zeroed() });
    melee_items::itkinds5::it_80284fc4(&mut want, 7, 2.5);
    unsafe {
        assert_eq!(it.xDD4_itemVar.sword.x8, want.xDD4_itemVar.sword.x8);
        assert_eq!(it.xDD4_itemVar.sword.xC, 1);
    }
    h::it_80285024(gp, 3);
    unsafe {
        assert_eq!(it.xDD4_itemVar.sword.x4, 3);
        assert_eq!(it.xDD4_itemVar.sword.xC, -1);
    }
    it.x1C = 5;
    assert_eq!(h::it_8026b7cc(gp), 5);
    assert_eq!(h::it_get_owner(gp), it.owner);

    let mut j: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    j.translate = Vec3 { x: 1.0, y: -2.0, z: 3.5 };
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    h::hsd_jobj_get_translation(&mut *j, &mut v);
    assert_eq!((v.x, v.y, v.z), (1.0, -2.0, 3.5));
    for flags in [0u32, JOBJ_MTX_DIRTY as u32, (JOBJ_MTX_DIRTY | JOBJ_USER_DEF_MTX) as u32] {
        j.flags = flags;
        assert_eq!(h::hsd_jobj_mtx_is_dirty(&mut *j), melee_types::inline::HSD_JObjMtxIsDirty(&j));
    }
    j.flags = JOBJ_MTX_DIRTY as u32;
    assert!(h::hsd_jobj_mtx_is_dirty(&mut *j));
}
