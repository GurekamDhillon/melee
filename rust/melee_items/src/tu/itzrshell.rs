//! Port of src/melee/it/kinds/itzrshell.c: the red Koopa shell of the Subspace Emissary (logic table 12). It shares
//! the green shell's state functions (`itzgshell.rs`) through its own state table `it_803F87F0` and forwards its
//! logic callbacks to them. The forwarded callees are `ext` hooks like every other unported callee so the
//! oracle can record them; `link()` wires them to the real `itzgshell` ports. Checked against the C (whole TU,
//! shipping contraction) by tests/kinds_itzrshell.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::itzgshell as zg;
use super::kinds_inl::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/kinds/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjAnimAll(a0: *mut HSD_JObj);
    fn Item_80268E5C(a0: *mut HSD_GObj, a1: i32, a2: u32) => unsafe { crate::tu::item::Item_80268E5C(a0, a1, a2) };
    fn grZakoGenerator_801CACB8(a0: *mut HSD_GObj) => unsafe { melee_world::tu::grzakogenerator::grZakoGenerator_801CACB8(a0) };
    fn itZGShell_Logic11_DmgDealt(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZGShell_Logic11_DmgDealt(a0 as _) as _ };
    fn itZGShell_Logic11_DmgReceived(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZGShell_Logic11_DmgReceived(a0 as _) as _ };
    fn itZGShell_Logic11_Dropped(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZGShell_Logic11_Dropped(a0 as _) };
    fn itZGShell_Logic11_Reflected(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZGShell_Logic11_Reflected(a0 as _) as _ };
    fn itZGShell_Logic11_ShieldBounced(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZGShell_Logic11_ShieldBounced(a0 as _) as _ };
    fn itZGShell_Logic11_Thrown(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZGShell_Logic11_Thrown(a0 as _) };
    fn itZrshell_UnkMotion0_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion0_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion0_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion0_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion0_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion0_Phys(a0 as _) };
    fn itZrshell_UnkMotion10_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion10_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion10_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion10_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion10_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion10_Phys(a0 as _) };
    fn itZrshell_UnkMotion11_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion11_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion11_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion11_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion11_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion11_Phys(a0 as _) };
    fn itZrshell_UnkMotion1_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion1_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion1_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion1_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion1_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion1_Phys(a0 as _) };
    fn itZrshell_UnkMotion2_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion2_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion2_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion2_Phys(a0 as _) };
    fn itZrshell_UnkMotion3_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion3_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion3_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion3_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion3_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion3_Phys(a0 as _) };
    fn itZrshell_UnkMotion4_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion4_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion4_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion4_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion4_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion4_Phys(a0 as _) };
    fn itZrshell_UnkMotion6_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion6_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion6_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion6_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion6_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion6_Phys(a0 as _) };
    fn itZrshell_UnkMotion8_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion8_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion8_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion8_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion8_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion8_Phys(a0 as _) };
    fn itZrshell_UnkMotion9_Anim(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion9_Anim(a0 as _) as _ };
    fn itZrshell_UnkMotion9_Coll(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion9_Coll(a0 as _) as _ };
    fn itZrshell_UnkMotion9_Phys(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::itZrshell_UnkMotion9_Phys(a0 as _) };
    fn it_2725_Logic11_Clanked(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::it_2725_Logic11_Clanked(a0 as _) as _ };
    fn it_2725_Logic11_EnteredAir(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::it_2725_Logic11_EnteredAir(a0 as _) };
    fn it_2725_Logic11_HitShield(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itzgshell::it_2725_Logic11_HitShield(a0 as _) as _ };
    fn it_2725_Logic11_PickedUp(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::it_2725_Logic11_PickedUp(a0 as _) };
    fn it_8026B894(a0: *mut HSD_GObj, a1: *mut HSD_GObj) -> i32 => unsafe { crate::itlib::it_8026b894(&mut *((*a0).user_data as *mut Item), a1) as _ };
    fn it_802762B0(a0: *mut Item) => unsafe { crate::item::it_802762b0(&mut *a0) };
    fn it_802762BC(a0: *mut Item) => unsafe { crate::item::it_802762bc(&mut *a0) };
    fn it_80277040(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itmaplib::it_80277040(a0) as _ };
    fn it_8027B5B0(a0: i32, a1: *mut Vec, a2: *mut HSD_JObj, a3: *mut Vec, a4: i32) -> *mut HSD_GObj => unsafe { crate::tu::itzako::it_8027B5B0(a0, a1 as _, a2, a3 as _, a4) };
    fn it_8027C56C(a0: *mut HSD_GObj, a1: f32) => unsafe { crate::tu::itzako::it_8027C56C(a0, a1) };
    fn it_802DDA84(a0: *mut HSD_GObj) -> i32 => unsafe { crate::tu::itnokonoko::it_802DDA84(a0 as _) as _ };
    fn it_802DE040(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::it_802DE040(a0 as _) };
    fn it_802DE0F0(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::it_802DE0F0(a0 as _) };
    fn it_802DE320(a0: *mut HSD_GObj) => unsafe { crate::tu::itzgshell::it_802DE320(a0 as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// Points the `ext` hooks of the functions that live in `itzgshell.rs` at their ports.
pub fn link() {
    melee_core::ct!("link");
    unsafe {
        use ext::hook::*;
        itZGShell_Logic11_DmgDealt = Some(|g| unsafe { zg::itZGShell_Logic11_DmgDealt(g) as i32 });
        itZGShell_Logic11_DmgReceived = Some(|g| unsafe { zg::itZGShell_Logic11_DmgReceived(g) as i32 });
        itZGShell_Logic11_Dropped = Some(|g| unsafe { zg::itZGShell_Logic11_Dropped(g) });
        itZGShell_Logic11_Reflected = Some(|g| unsafe { zg::itZGShell_Logic11_Reflected(g) as i32 });
        itZGShell_Logic11_ShieldBounced = Some(|g| unsafe { zg::itZGShell_Logic11_ShieldBounced(g) as i32 });
        itZGShell_Logic11_Thrown = Some(|g| unsafe { zg::itZGShell_Logic11_Thrown(g) });
        it_2725_Logic11_Clanked = Some(|g| unsafe { zg::it_2725_Logic11_Clanked(g) as i32 });
        it_2725_Logic11_EnteredAir = Some(|g| unsafe { zg::it_2725_Logic11_EnteredAir(g) });
        it_2725_Logic11_HitShield = Some(|g| unsafe { zg::it_2725_Logic11_HitShield(g) as i32 });
        it_2725_Logic11_PickedUp = Some(|g| unsafe { zg::it_2725_Logic11_PickedUp(g) });
        it_802DE040 = Some(|g| unsafe { zg::it_802DE040(g) });
        it_802DE0F0 = Some(|g| unsafe { zg::it_802DE0F0(g) });
        it_802DE320 = Some(|g| unsafe { zg::it_802DE320(g) });
    }
}

/// `ItemStateTable it_803F87F0[]` (the motion functions are `itzgshell.rs`'s)
pub static mut it_803F87F0: [ItemStateTable; 13] = [
    st_row(0, pred_b!(zg::itZrshell_UnkMotion0_Anim), Some(zg::itZrshell_UnkMotion0_Phys), pred_b!(zg::itZrshell_UnkMotion0_Coll)),
    st_row(0, pred_b!(zg::itZrshell_UnkMotion1_Anim), Some(zg::itZrshell_UnkMotion1_Phys), pred_b!(zg::itZrshell_UnkMotion1_Coll)),
    st_row(0, pred_b!(zg::itZrshell_UnkMotion2_Anim), Some(zg::itZrshell_UnkMotion2_Phys), None),
    st_row(1, pred_b!(zg::itZrshell_UnkMotion3_Anim), Some(zg::itZrshell_UnkMotion3_Phys), pred_b!(zg::itZrshell_UnkMotion3_Coll)),
    st_row(1, pred_b!(zg::itZrshell_UnkMotion4_Anim), Some(zg::itZrshell_UnkMotion4_Phys), pred_b!(zg::itZrshell_UnkMotion4_Coll)),
    st_row(1, pred_b!(zg::itZrshell_UnkMotion6_Anim), Some(zg::itZrshell_UnkMotion6_Phys), pred_b!(zg::itZrshell_UnkMotion6_Coll)),
    st_row(0, pred_b!(zg::itZrshell_UnkMotion6_Anim), Some(zg::itZrshell_UnkMotion6_Phys), pred_b!(zg::itZrshell_UnkMotion6_Coll)),
    st_row(1, pred_b!(zg::itZrshell_UnkMotion8_Anim), Some(zg::itZrshell_UnkMotion8_Phys), pred_b!(zg::itZrshell_UnkMotion8_Coll)),
    st_row(0, pred_b!(zg::itZrshell_UnkMotion8_Anim), Some(zg::itZrshell_UnkMotion8_Phys), pred_b!(zg::itZrshell_UnkMotion8_Coll)),
    st_row(0, pred_b!(zg::itZrshell_UnkMotion9_Anim), Some(zg::itZrshell_UnkMotion9_Phys), pred_b!(zg::itZrshell_UnkMotion9_Coll)),
    st_row(2, pred_b!(zg::itZrshell_UnkMotion10_Anim), Some(zg::itZrshell_UnkMotion10_Phys), pred_b!(zg::itZrshell_UnkMotion10_Coll)),
    st_row(3, pred_b!(zg::itZrshell_UnkMotion11_Anim), Some(zg::itZrshell_UnkMotion11_Phys), pred_b!(zg::itZrshell_UnkMotion11_Coll)),
    st_row(4, None, None, None),
];

pub unsafe extern "C" fn it_802E02E8(gobj: *mut Item_GObj) {
    melee_core::ct!("it_802E02E8");
    ext::it_802DE040(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_PickedUp(gobj: *mut Item_GObj) {
    melee_core::ct!("itZRShell_Logic12_PickedUp");
    ext::it_2725_Logic11_PickedUp(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_Thrown(gobj: *mut Item_GObj) {
    melee_core::ct!("itZRShell_Logic12_Thrown");
    ext::itZGShell_Logic11_Thrown(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_Dropped(gobj: *mut Item_GObj) {
    melee_core::ct!("itZRShell_Logic12_Dropped");
    ext::itZGShell_Logic11_Dropped(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_EnteredAir(gobj: *mut Item_GObj) {
    melee_core::ct!("itZRShell_Logic12_EnteredAir");
    ext::it_2725_Logic11_EnteredAir(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_Destroyed(gobj: *mut Item_GObj) {
    melee_core::ct!("itZRShell_Logic12_Destroyed");
    ext::grZakoGenerator_801CACB8(gobj);
}

pub unsafe extern "C" fn itZRShell_Logic12_DmgDealt(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_DmgDealt");
    ext::itZGShell_Logic11_DmgDealt(gobj) != 0
}

pub unsafe extern "C" fn itZRShell_Logic12_DmgReceived(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_DmgReceived");
    ext::itZGShell_Logic11_DmgReceived(gobj) != 0
}

pub unsafe extern "C" fn itZRShell_Logic12_Reflected(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_Reflected");
    ext::itZGShell_Logic11_Reflected(gobj) != 0
}

pub unsafe extern "C" fn itZRShell_Logic12_Clanked(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_Clanked");
    ext::it_2725_Logic11_Clanked(gobj) != 0
}

pub unsafe extern "C" fn itZRShell_Logic12_HitShield(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_HitShield");
    ext::it_2725_Logic11_HitShield(gobj) != 0
}

pub unsafe extern "C" fn itZRShell_Logic12_ShieldBounced(gobj: *mut Item_GObj) -> bool {
    melee_core::ct!("itZRShell_Logic12_ShieldBounced");
    ext::itZGShell_Logic11_ShieldBounced(gobj) != 0
}

pub unsafe extern "C" fn it_802E0468(gobj: *mut Item_GObj, ref_gobj: *mut HSD_GObj) {
    melee_core::ct!("it_802E0468");
    ext::it_8026B894(gobj, ref_gobj);
}

/// 2E0488: spawns a red shell.
pub unsafe extern "C" fn it_802E0488(arg0: i32, arg1: *mut Vec3, arg2: i32) -> *mut Item_GObj {
    melee_core::ct!("it_802E0488");
    let mut pos: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let it_gp = ext::it_8027B5B0(It_Kind_ZRShell, arg1, null_mut(), &mut pos, 1);
    if !it_gp.is_null() {
        let it = (*it_gp).user_data as *mut Item;

        (*it).facing_dir = arg2 as f32;
        ext::it_8027C56C(it_gp, (*it).facing_dir);
        (*it).xDD4_itemVar.zrshell.xE10 = arg0;
        ext::Item_80268E5C(it_gp, 0xC, ITEM_ANIM_UPDATE as u32);
        ext::HSD_JObjAnimAll(GET_JOBJ(it_gp));
        if ext::it_802DDA84(it_gp) != 0 {
            ext::it_802762B0(it);
            if ext::it_80277040(it_gp) == 0 {
                ext::it_802DE0F0(it_gp);
            } else {
                ext::it_2725_Logic11_EnteredAir(it_gp);
            }
        } else {
            ext::it_802762BC(it);
            ext::it_802DE320(it_gp);
        }
    }
    it_gp
}
