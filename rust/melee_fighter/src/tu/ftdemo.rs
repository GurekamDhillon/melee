//! src/melee/ft/ftdemo.c: demo / results-screen fighters (no gameplay physics).
//! CreateFighter is the C call sequence over callees that are not ported yet (TODO stubs);
//! the logic of its own (demo anim pair choice, per-`unk8` create hook) is ported natively.
//! No fused float sites (fma_sites.py); SetFacingDirection is `(f32)(M_PI_2 * (double)dir)`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::types::*;
use core::ffi::c_char;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees / tables in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    use core::ffi::{c_char, c_void};
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        FIGHTER_UNK_INIT_LOAD Fighter_UnkInitLoad_80068914(g: G, info: &plAllocInfo);
        GOBJ_CREATE GObj_Create(class: i32, link: i32, prio: i32) -> G;
        GOBJ_SETUP_GX_LINK_DRAW GObj_SetupGXLink_ftDrawCommon_80080E18(g: G, link: i32, prio: i32);
        HSD_OBJ_ALLOC_FIGHTER HSD_ObjAlloc_fighter() -> *mut Fighter;
        HSD_OBJ_ALLOC_ATTRS HSD_ObjAlloc_fighter_dat_attrs() -> *mut c_void;
        GOBJ_INIT_USER_DATA GObj_InitUserData(g: G, kind: i32, fp: *mut Fighter);
        FTDATA_8008572C ftData_8008572C(kind: i32);
        FTDATA_803C52A0 ftData_803C52A0() -> *mut MotionState;
        FTDATA_UNK_MOTION_STATES0 ftData_UnkMotionStates0(kind: i32) -> *mut MotionState;
        FTDATA_UNK_BYTE ftData_UnkBytePerCharacter(kind: i32) -> i32;
        EFASYNC_LOAD_SYNC efAsync_LoadSync(id: i32);
        FTDATA_80085820 ftData_80085820(kind: i32, costume: u8);
        FTDATA_800858E4 ftData_800858E4(kind: i32, costume: u8);
        FIGHTER_UNK_UPDATE_COSTUME_JOINT Fighter_UnkUpdateCostumeJoint_800686E4(g: G);
        // `ftData_UnkDemoCallbacks0[kind]` called as (unk8, &a, &b); None = NULL entry (PC).
        FTDATA_UNK_DEMO_CALLBACKS0 ftData_UnkDemoCallbacks0(kind: i32) -> Option<fn(i32, &mut i32, &mut i32)>;
        FTDATA_80085B98 ftData_80085B98(fp: &mut Fighter, a: i32, b: i32);
        // The fixed init chain after ftData_80085B98, in C order: ftParts_80074E58,
        // ftParts_SetupParts, ftAnim_80070308, ftCo_800C884C, Fighter_80068E64, ftParts_800749CC,
        // ftAnim_8007077C, ftCo_8009CF84, ftAnim_8006FE48, Fighter_UnkUpdateVecFromBones_8006876C,
        // ftCo_8009F578, Mex_OnLoadDispatch(kind, gobj, ftData_OnLoad[kind]), ftColl_8007B320.
        FTDEMO_INIT_CHAIN ftDemo_init_chain(g: G);
        CAMERA_80029020 Camera_80029020() -> *mut CmSubject;
        LBSHADOW_8000ED54 lbShadow_8000ED54(sh: *mut LbShadow, obj: *mut c_void);
        // The five HSD_GObj_SetupProc calls (Fighter_8006A360/procUpdate/procMap/8006C80C/8006D9AC).
        FTDEMO_SETUP_PROCS ftDemo_setup_procs(g: G);
        FIGHTER_UNK_PROCESS_DEATH Fighter_UnkProcessDeath_80068354(g: G);
        // `on_create_fighter[i]` (ftCo_800BECB0 ...); C entry 3 is NULL.
        ON_CREATE_FIGHTER on_create_fighter(i: i32, g: G);
        FIGHTER_800679B0 Fighter_800679B0();
        HSD_OBJ_ALLOC_INIT_X59C HSD_ObjAllocInit_fighter_x59C(size: u32, align: u32);
        FTDATA_UNK_INT_PAIRS ftData_UnkIntPairs(i: i32) -> *mut ftData_UnkCountStruct;
        HSD_ARCHIVE_GET_PUBLIC_ADDRESS HSD_ArchiveGetPublicAddress(ar: *mut HSD_Archive, name: *const c_char) -> *mut c_void;
        FTDATA_803C2468 ftData_803C2468(pairs_idx: i32, arr_idx: i32) -> *const c_char;
        // `ftData_803C24EC[cb_idx]`; None = NULL.
        FTDATA_803C24EC ftData_803C24EC(cb_idx: i32) -> Option<fn(i32) -> *mut c_char>;
        FTPART_SET_ROT_Y ftPartSetRotY(fp: &mut Fighter, part: i32, rot: f32);
    }
}

/// `initFighter`: plAllocInfo2 -> plAllocInfo, then Fighter_UnkInitLoad_80068914.
pub fn init_fighter(g: G, a: &plAllocInfo2) {
    melee_core::ct!("init_fighter");
    let mut t = plAllocInfo::zeroed();
    t.internal_id = a.internal_id;
    t.slot = a.slot;
    t.anon_6.set_b0(a.anon_c.has_transformation());
    ext::Fighter_UnkInitLoad_80068914(g, &t);
}

/// The demo anim pair `(a, b)` chosen from `alloc_info->unk8` (TARGET_PC null-callback guard).
pub fn demo_anim_pair(kind: i32, unk8: i32) -> (i32, i32) {
    melee_core::ct!("demo_anim_pair");
    if unk8 >= 9 {
        match ext::ftData_UnkDemoCallbacks0(kind) {
            Some(cb) => {
                let (mut a, mut b) = (0, 0);
                cb(unk8, &mut a, &mut b);
                (a, b)
            }
            None => (0, 9),
        }
    } else if unk8 >= 8 {
        (13, 13)
    } else if unk8 >= 7 {
        (12, 12)
    } else if unk8 >= 5 {
        (10, 11)
    } else {
        (0, 9)
    }
}

/// `ftDemo_CreateFighter`.
pub fn ftdemo_create_fighter(a: &plAllocInfo2) -> G {
    melee_core::ct!("ftdemo_create_fighter");
    let g = ext::GObj_Create(HSD_GOBJ_CLASS_FIGHTER as i32, 8, 0);
    ext::GObj_SetupGXLink_ftDrawCommon_80080E18(g, 5, 0);
    let p = ext::HSD_ObjAlloc_fighter();
    unsafe { (*p).dat_attrs_backup = ext::HSD_ObjAlloc_fighter_dat_attrs() };
    ext::GObj_InitUserData(g, 4, p);
    ext::ftData_8008572C(a.internal_id as i32);
    init_fighter(g, a);
    let f = fp(g);
    f.x18 = 14;
    f.x1C_actionStateList = ext::ftData_803C52A0();
    f.x20_actionStateList = ext::ftData_UnkMotionStates0(f.kind as i32);
    unsafe {
        f.x24 = (*f.ft_data).x14 as _;
        f.x28 = (*f.ft_data).x18 as _;
    }
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(f.kind as i32));
    if a.anon_c.b0() == 0 {
        ext::ftData_80085820(f.kind as i32, f.x619_costume_id);
    } else {
        ext::ftData_800858E4(f.kind as i32, f.x619_costume_id);
    }
    ext::Fighter_UnkUpdateCostumeJoint_800686E4(g);
    let (pa, pb) = demo_anim_pair(fp(g).kind as i32, a.unk8 as i32);
    ext::ftData_80085B98(fp(g), pa, pb);
    ext::ftDemo_init_chain(g);
    fp(g).x890_cameraBox = ext::Camera_80029020();
    ext::lbShadow_8000ED54(&mut fp(g).x20A4, unsafe { (*g).hsd_obj } as *mut _);
    ext::ftDemo_setup_procs(g);
    ext::Fighter_UnkProcessDeath_80068354(g);
    if a.unk8 as i32 != 3 {
        ext::on_create_fighter(a.unk8 as i32, g);
    }
    crate::ftlib::ftlib_800867e8(g);
    g
}

/// `ftDemo_ObjAllocInit`.
pub fn ftdemo_obj_alloc_init() {
    melee_core::ct!("ftdemo_obj_alloc_init");
    ext::Fighter_800679B0();
    ext::HSD_ObjAllocInit_fighter_x59C(0xB000, 0x20);
}

/// `ftDemo_SetArchiveData`: resolve a demo anim symbol once per pair.
pub fn ftdemo_set_archive_data(pairs_idx: i32, archive: *mut HSD_Archive, arr_idx: i32) {
    melee_core::ct!("ftdemo_set_archive_data");
    const INTS: [i32; 5] = [9, 10, 11, 14, 15];
    let pair = unsafe { &mut *ext::ftData_UnkIntPairs(pairs_idx) };
    if pair.data.is_null() {
        let name = if arr_idx >= 4 {
            ftdemo_get_motion_file_string(pairs_idx, INTS[(arr_idx - 4) as usize]) as *const c_char
        } else {
            ext::ftData_803C2468(pairs_idx, arr_idx)
        };
        pair.data = ext::HSD_ArchiveGetPublicAddress(archive, name);
    }
}

/// `ftDemo_GetMotionFileString`. NULL callback: HSD_ASSERTREPORT (fatal).
pub fn ftdemo_get_motion_file_string(cb_idx: i32, cb_arg: i32) -> *mut c_char {
    melee_core::ct!("ftdemo_get_motion_file_string");
    match ext::ftData_803C24EC(cb_idx) {
        Some(cb) => cb(cb_arg),
        None => panic!("no demo vi anim! {cb_arg}"),
    }
}

/// `ftDemo_SetFacingDirection`.
pub fn ftdemo_set_facing_direction(g: G, facing_dir: f32) {
    melee_core::ct!("ftdemo_set_facing_direction");
    let f = fp(g);
    f.facing_dir = facing_dir;
    let rot = (core::f64::consts::FRAC_PI_2 * f.facing_dir as f64) as f32;
    ext::ftPartSetRotY(f, 0, rot);
}
