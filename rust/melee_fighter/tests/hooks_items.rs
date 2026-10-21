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
        assert_eq!(it.xDCD_flag.anon_0.b2(), 1);
        h::it_8026bcf4(gp);
        assert_eq!(it.xDCD_flag.anon_0.b2(), 0);
    }
    assert_eq!(h::it_8026b6c8(gp), 0);
}
