//! src/melee/ft/kinds/ftGameWatch/ftgamewatch.c: Mr. Game & Watch's init callbacks.
//! No fused sites (tools/rust-port/fma_sites.py). Smoke-tested by tests/tu_ftgamewatch.rs.
//!
//! Data: file-name strings below; `ftGw_Init_MotionStateTable` (40 rows of callbacks into the
//! ftGameWatch / ftCommon TUs) belongs to the motion-state dispatcher and is TODO until those
//! callbacks are all native. `ftGw_CostumeList` is zero-initialised BSS.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::tu::{
    ftgamewatchattack11 as a11, ftgamewatchattackair as aair, ftgamewatchattacklw3 as alw3,
    ftgamewatchattacks4 as as4, ftgamewatchspecialhi as shi, ftgamewatchspeciallw as slw,
    ftgamewatchspecials as ss,
};
use melee_types::*;

/// Already ported (rounds 5/6): re-exported under this TU.
pub use crate::chars::r5::ftGameWatch::ftgamewatch::ftGw_Init_OnDeath;
pub use crate::chars::r6::ftGameWatch::ftgamewatch::{ftGw_Init_OnItemInvisible, ftGw_Init_OnItemVisible};
use crate::tu::ftclink::{Fighter_OnItemDrop, Fighter_OnItemPickup};

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
pub const ftGw_Init_CostumeStrings: [[&str; 2]; 4] = [
    [ftGw_Init_803D2904, ftGw_Init_803D2910],
    [ftGw_Init_803D2904, ftGw_Init_803D2910],
    [ftGw_Init_803D2904, ftGw_Init_803D2910],
    [ftGw_Init_803D2904, ftGw_Init_803D2910],
];

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use core::ffi::c_void;
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
    macro_rules! hook_or {
        ($hook:ident, $name:ident, $port:path, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
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
    // Ported elsewhere; hooks so the TU tests can install recording stand-ins.
    hook_or!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, melee_fighter::ftdmgfall::ftparts_get_bone_index,
        (fp: &Fighter, part: i32) -> i32);
    hook_or!(FTCOMMON_GETMODELSCALE, ftCommon_GetModelScale, melee_fighter::ftcommon2::get_model_scale,
        (fp: &Fighter) -> f32);
    hook_or!(FTPARTSETROTY, ftPartSetRotY, melee_fighter::tu::ftparts::ftPartSetRotY,
        (fp: &Fighter, part: i32, rot: f32));
    hook_or!(FTCOMMON_UPDATEFACING, ftCommon_UpdateFacing, crate::stubs::ftCommon_UpdateFacing,
        (fp: &mut Fighter));
    hook!(LB_8000B1CC, lb_8000B1CC, (jobj: *mut HSD_JObj, off: *const Vec3, out: *mut Vec3));
    hook!(FTMATERIAL_800BFB4C, ftMaterial_800BFB4C, (gobj: *mut HSD_GObj, color: *const _GXColor));
    hook!(IT_8026B3F8, it_8026B3F8, (article: *mut c_void, kind: i32));

    /// `Mex_CostumeVisIdx(fk, costume)` (TARGET_PC, ft/inlines.h): the raw costume id when there
    /// is no mexData (vanilla disc); the m-ex row lookup is installed here.
    pub static mut MEX_COSTUMEVISIDX: Option<fn(fk: i32, costume: i32) -> i32> = None;
    #[allow(non_snake_case)]
    pub fn Mex_CostumeVisIdx(fk: i32, costume: i32) -> i32 {
        match unsafe { MEX_COSTUMEVISIDX } {
            Some(f) => f(fk, costume),
            None => costume,
        }
    }
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

/// `ftGw_Init_OnDamage` as a Fighter callback entry (death2_cb / take_dmg_cb).
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

        let da = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*fp).x34_scale.z = (*da).x0_GAMEWATCH_WIDTH;
        (*fp).x610_color_rgba[1] = (*da).x14_GAMEWATCH_OUTLINE;
        ext::ftMaterial_800BFB4C(gobj, &(*da).x4_GAMEWATCH_COLOR[costume_vis_idx(fp)]);
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

/// Removes every item G&W owns (damage / death).
pub fn ftGw_Init_OnDamage(gobj: *mut HSD_GObj) {
    a11::ftGw_Attack11_ItemGreenhouseRemove(gobj);
    alw3::ftGw_AttackLw3_ItemManholeOnDamage(gobj);
    as4::ftGw_AttackS4_ItemTorchOnDamage(gobj);
    aair::ftGw_AttackAirN_ItemParachuteRemove(gobj);
    aair::ftGw_AttackAirN_ItemTurtleRemove(gobj);
    aair::ftGw_AttackAirN_ItemSparkyRemove(gobj);
    ss::ftGw_SpecialS_ItemJudgementRemove(gobj);
    slw::ftGw_SpecialLw_ItemPanicRemove(gobj);
    shi::ftGw_SpecialHi_ItemRescueRemove(gobj);
}

pub fn ftGw_Init_8014A538(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).ground_or_air == GA_Air {
            a11::ftGw_Attack11_ItemGreenhouseRemove(gobj);
            alw3::ftGw_AttackLw3_ItemManholeOnDamage(gobj);
            as4::ftGw_AttackS4_ItemTorchOnDamage(gobj);
            aair::ftGw_AttackAirN_ItemParachuteRemove(gobj);
            aair::ftGw_AttackAirN_ItemTurtleRemove(gobj);
            aair::ftGw_AttackAirN_ItemSparkyRemove(gobj);
            ss::ftGw_SpecialS_ItemJudgementRemove(gobj);
            slw::ftGw_SpecialLw_ItemPanicRemove(gobj);
            shi::ftGw_SpecialHi_ItemRescueRemove(gobj);
        }
    }
}

pub fn ftGw_Init_OnItemPickup(gobj: *mut HSD_GObj, flag: i32) {
    Fighter_OnItemPickup(gobj, flag, 1, 1);
}

pub fn ftGw_Init_OnItemDrop(gobj: *mut HSD_GObj, drop_item_flag: i32) {
    Fighter_OnItemDrop(gobj, drop_item_flag, 1, 1);
}

/// Set the Oil Panic color overlay.
pub fn ftGw_Init_UnkMotionStates4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        if (*fp).u.gw.x2238_panicCharge >= ftGw_Panic_Full as i32 {
            melee_fighter::tu::ftcolanim::ftco_800bffd0(&mut *fp, 5, 0);
        }
    }
}

/// `COPY_ATTRS(gobj, ftGameWatchAttributes)`.
pub fn ftGw_Init_LoadSpecialAttrs(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        *((*fp).dat_attrs as *mut ftGameWatchAttributes) =
            *((*(*fp).ft_data).ext_attr as *const ftGameWatchAttributes);
    }
}

/// Apply color to G&W's items: `item_mod->x0_unk = attrs.COLOR[FT_COSTUME_VIS_IDX(fp)]`.
pub fn ftGw_Init_8014A7F4(gobj: *mut HSD_GObj, item_mod: *mut ItemModStruct) {
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*item_mod).x0_unk = (*attrs).x4_GAMEWATCH_COLOR[costume_vis_idx(fp)];
    }
}

/// Apply the outline color to G&W's items.
pub fn ftGw_Init_8014A814(gobj: *mut HSD_GObj, item_mod: *mut ItemModStruct) {
    unsafe {
        let fp = fpp(gobj);
        let attrs = (*fp).dat_attrs as *const ftGameWatchAttributes;
        (*item_mod).x0_unk = (*attrs).x14_GAMEWATCH_OUTLINE;
    }
}

/// OnAbsorb callback.
pub fn ftGw_Init_OnAbsorb(gobj: *mut HSD_GObj) {
    slw::ftGw_SpecialLw_AbsorbThink_DecideAction(gobj);
}
