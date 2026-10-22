//! src/melee/ft/kinds/ftGameWatch/ftgamewatch.c: Mr. Game & Watch's init callbacks (death,
//! load, damage, item pickup/drop/visibility, costume colours for his items).
//! No float arithmetic (tools/rust-port/fma_sites.py: 0 fused sites). Smoke-tested by
//! tests/tu_ftgamewatch.rs.
//!
//! Data: file-name strings below. `ftGw_Init_MotionStateTable` (36 rows of Anim/IASA/Phys/Coll
//! pointers into every ftGameWatch TU and ftCommon) belongs to the motion-state dispatcher and is
//! TODO until those callbacks are all native; `ftGw_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftGameWatch::ftgamewatch::ftGw_Init_OnDeath;
pub use crate::chars::r6::ftGameWatch::ftgamewatch::{ftGw_Init_OnItemInvisible, ftGw_Init_OnItemVisible};

pub const ftGw_Init_DatFilename: &str = "PlGw.dat";
pub const ftGw_Init_DataName: &str = "ftDataGamewatch";
pub const ftGw_Init_803D2904: &str = "PlGwNr.dat";
pub const ftGw_Init_803D2910: &str = "PlyGamewatch5K_Share_joint";
pub const ftGw_Init_AnimDatFilename: &str = "PlGwAJ.dat";
/// `ftGw_Init_DemoMotionFilenames` (result, intro, ending, vi-wait).
pub const ftGw_Init_DemoMotionFilenames: [&str; 4] = [
    "ftDemoResultMotionFileGamewatch",
    "ftDemoIntroMotionFileGamewatch",
    "ftDemoEndingMotionFileGamewatch",
    "ftDemoViWaitMotionFileGamewatch",
];
/// `ftGw_Init_CostumeStrings`: (dat, joint) per costume; no matanim joint.
pub const ftGw_Init_CostumeStrings: [[&str; 2]; 4] = [[ftGw_Init_803D2904, ftGw_Init_803D2910]; 4];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// Like `hook!`, but an unset hook runs the native port `$port` (a ported TU elsewhere).
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook!(FTGW_ATTACKLW3_ITEMMANHOLEONDAMAGE, ftGw_AttackLw3_ItemManholeOnDamage, (gobj: *mut HSD_GObj));
    hook!(FTGW_ATTACKS4_ITEMTORCHONDAMAGE, ftGw_AttackS4_ItemTorchOnDamage, (gobj: *mut HSD_GObj));
    hook!(FTGW_SPECIALS_ITEMJUDGEMENTREMOVE, ftGw_SpecialS_ItemJudgementRemove, (gobj: *mut HSD_GObj));
    hook!(FTGW_SPECIALLW_ITEMPANICREMOVE, ftGw_SpecialLw_ItemPanicRemove, (gobj: *mut HSD_GObj));
    hook!(FTGW_SPECIALHI_ITEMRESCUEREMOVE, ftGw_SpecialHi_ItemRescueRemove, (gobj: *mut HSD_GObj));
    hook!(FTGW_SPECIALLW_ABSORBTHINK_DECIDEACTION, ftGw_SpecialLw_AbsorbThink_DecideAction, (gobj: *mut HSD_GObj));
    hook!(FTMATERIAL_800BFB4C, ftMaterial_800BFB4C, (gobj: *mut HSD_GObj, diffuse: *mut _GXColor));
    hook!(IT_8026B3F8, it_8026B3F8, (article: *mut core::ffi::c_void, kind: i32));
    hook_or!(FTCO_800BFFD0, ftCo_800BFFD0, (fp: *mut Fighter, a: i32, b: i32) -> bool,
        |fp: *mut Fighter, a: i32, b: i32| unsafe { melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, a, b) });
    // TARGET_PC `Mex_CostumeVisIdx(kind, costume)`: the raw costume id unless an m-ex disc maps it.
    hook_or!(MEX_COSTUMEVISIDX, Mex_CostumeVisIdx, (kind: i32, costume: i32) -> i32, |_kind: i32, costume: i32| costume);
}

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

/// `FT_COSTUME_VIS_IDX(fp)`.
#[inline(always)]
unsafe fn costume_vis_idx(fp: *mut Fighter) -> usize {
    ext::Mex_CostumeVisIdx((*fp).kind as i32, (*fp).x619_costume_id as i32) as usize
}

/// `ftGw_Init_OnDamage` as a Fighter callback entry (death2 / take-damage).
pub unsafe extern "C" fn cb_ftGw_Init_OnDamage(g: *mut HSD_GObj) {
    ftGw_Init_OnDamage(g);
}

pub fn ftGw_Init_OnLoad(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        let items = (*(*fp).ft_data).x48_items;

        (*fp).set_x2222_b6(0);
        (*fp).set_x2223_b1(1);
        (*fp).set_x2224_b0(1);

        // PUSH_ATTRS(fp, ftGameWatchAttributes)
        let backup = (*fp).dat_attrs_backup;
        let src = (*(*fp).ft_data).ext_attr as *const ftGameWatchAttributes;
        *((*fp).dat_attrs_backup as *mut ftGameWatchAttributes) = *src;
        (*fp).dat_attrs = backup;
        (*fp).u.gw.x2238_panicCharge = ftGw_Panic_Empty as i32;

        let da = (*fp).dat_attrs as *mut ftGameWatchAttributes;
        (*fp).x34_scale.z = (*da).x0_GAMEWATCH_WIDTH;
        (*fp).x610_color_rgba[1] = (*da).x14_GAMEWATCH_OUTLINE;
        let idx = costume_vis_idx(fp);
        ext::ftMaterial_800BFB4C(gobj, &mut (*da).x4_GAMEWATCH_COLOR[idx]);
        (*fp).x5AC.xC[4] = *items.add(10) as *mut FtPartsVisLookup;

        ext::it_8026B3F8(*items.add(0), It_Kind_GameWatch_Greenhouse as i32);
        ext::it_8026B3F8(*items.add(1), It_Kind_GameWatch_Manhole as i32);
        ext::it_8026B3F8(*items.add(2), It_Kind_GameWatch_Fire as i32);
        ext::it_8026B3F8(*items.add(3), It_Kind_GameWatch_Parachute as i32);
        ext::it_8026B3F8(*items.add(4), It_Kind_GameWatch_Turtle as i32);
        ext::it_8026B3F8(*items.add(5), It_Kind_GameWatch_Breath as i32);
        ext::it_8026B3F8(*items.add(6), It_Kind_GameWatch_Judge as i32);
        ext::it_8026B3F8(*items.add(7), It_Kind_GameWatch_Panic as i32);
        ext::it_8026B3F8(*items.add(8), It_Kind_GameWatch_Chef as i32);
        ext::it_8026B3F8(*items.add(9), It_Kind_GameWatch_Rescue as i32);
    }
}

/// Removes every item G&W can have out (damage / death callback).
pub fn ftGw_Init_OnDamage(gobj: *mut HSD_GObj) {
    crate::tu::ftgamewatchattack11::ftGw_Attack11_ItemGreenhouseRemove(gobj);
    ext::ftGw_AttackLw3_ItemManholeOnDamage(gobj);
    ext::ftGw_AttackS4_ItemTorchOnDamage(gobj);
    crate::tu::ftgamewatchattackair::ftGw_AttackAirN_ItemParachuteRemove(gobj);
    crate::tu::ftgamewatchattackair::ftGw_AttackAirN_ItemTurtleRemove(gobj);
    crate::tu::ftgamewatchattackair::ftGw_AttackAirN_ItemSparkyRemove(gobj);
    ext::ftGw_SpecialS_ItemJudgementRemove(gobj);
    ext::ftGw_SpecialLw_ItemPanicRemove(gobj);
    ext::ftGw_SpecialHi_ItemRescueRemove(gobj);
}

/// Same as [`ftGw_Init_OnDamage`], only while airborne (collision callbacks).
pub fn ftGw_Init_8014A538(gobj: *mut HSD_GObj) {
    unsafe {
        if (*fpp(gobj)).ground_or_air == GA_Air {
            ftGw_Init_OnDamage(gobj);
        }
    }
}

pub fn ftGw_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    crate::tu::ftclink::Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftGw_Init_OnItemDrop(gobj: *mut HSD_GObj, drop_item_flag: i32) {
    crate::tu::ftclink::Fighter_OnItemDrop(gobj, drop_item_flag, 1, 1);
}

/// Oil Panic colour overlay.
pub fn ftGw_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            ext::ftCo_800BFFD0(fp, 5, 0);
        }
    }
}

/// `COPY_ATTRS(gobj, ftGameWatchAttributes)`.
pub fn ftGw_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftGameWatchAttributes) = *((*(*fp).ft_data).ext_attr as *const ftGameWatchAttributes);
    }
}

/// Item body colour from the costume's `GAMEWATCH_COLOR` entry.
pub fn ftGw_Init_8014A7F4(gobj: *mut HSD_GObj, item_mod: *mut ItemModStruct) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*item_mod).x0_unk = (*da).x4_GAMEWATCH_COLOR[costume_vis_idx(fp)];
    }
}

/// Item outline colour.
pub fn ftGw_Init_8014A814(gobj: *mut HSD_GObj, item_mod: *mut ItemModStruct) {
    unsafe {
        let fp = fpp(gobj);
        let da = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*item_mod).x0_unk = (*da).x14_GAMEWATCH_OUTLINE;
    }
}

pub fn ftGw_Init_OnAbsorb(gobj: *mut HSD_GObj) {
    ext::ftGw_SpecialLw_AbsorbThink_DecideAction(gobj);
}
