//! Port of src/melee/gr/groldpupupu.c (Dream Land 1.0 "old": the Whispy wind cycle, the clouds and the
//! enemy spawner). Every function of the TU, native: pointers into the game's structs, callees of
//! other TUs through `ext` hooks. Checked against the C (whole TU, shipping contraction, stateful
//! oracle) by tests/tu_groldpupupu.rs; regenerate the C side with
//! `python rust/melee_world/gen/cref/run_gr.py groldpupupu`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_runtime::cabi::f2i32;
use melee_types::inline::*;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn Camera_LogicBounds(a0: *mut f32, a1: *mut f32, a2: *mut f32) -> i32;
    fn Camera_RequestQuake(a0: u32, a1: *mut Vec);
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn Ground_801C0498() -> f32 = || unsafe { crate::tu::ground::Ground_801C0498() as _ };
    fn Ground_801C39C0() = || { unsafe { let _ = crate::tu::ground::Ground_801C39C0(); } };
    fn Ground_801C3BB4() = || { unsafe { let _ = crate::tu::ground::Ground_801C3BB4(); } };
    fn Ground_801C3FA4(a0: *mut HSD_GObj, a1: i32) -> *mut HSD_JObj = |a0: *mut HSD_GObj, a1: i32| unsafe { crate::tu::ground::Ground_801C3FA4(a0 as _, a1 as _) as _ };
    fn Ground_801C4A08(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = crate::tu::ground::Ground_801C4A08(a0 as _); } };
    fn Ground_GetMapGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetMapGObj(a0 as _) as _ };
    fn Ground_GetStageGObj(a0: i32) -> *mut HSD_GObj = |a0: i32| unsafe { crate::tu::ground::Ground_GetStageGObj(a0 as _) as _ };
    fn Ground_GetYakumonoParam() -> *mut c_void = || unsafe { crate::tu::ground::Ground_GetYakumonoParam() as _ };
    fn Ground_InitMapColl(a0: *mut HSD_JObj, a1: i32) -> i32 = |a0: *mut HSD_JObj, a1: i32| unsafe { crate::tu::ground::Ground_InitMapColl(a0 as _, a1 as _) as _ };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8| unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(a0 as _, a1 as _, a2 as _) as _ };
    fn HSD_JObjClearFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjClearFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetFlagsAll(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetFlagsAll(a0 as _, a1 as _); } };
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) = |a0: *mut HSD_JObj| { unsafe { let _ = melee_runtime::tu::jobj_world::HSD_JObjSetMtxDirtySub(a0 as _); } };
    fn HSD_Randf() -> f32;
    fn HSD_Randi(a0: i32) -> i32;
    fn ftCo_800C06E8(a0: *mut HSD_GObj, a1: i32, a2: *mut c_void);
    fn ftLib_800864A8(a0: *mut Vec, a1: *mut HSD_GObj) -> f32;
    fn ftLib_80086644(a0: *mut HSD_GObj, a1: *mut Vec);
    fn grAnime_801C7FF8(a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32) = |a0: *mut HSD_GObj, a1: i32, a2: i32, a3: i32, a4: f32, a5: f32| { unsafe { let _ = crate::tu::granime::grAnime_801C7FF8(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _); } };
    fn grAnime_801C8138(a0: *mut HSD_GObj, a1: i32, a2: i32) = |a0: *mut HSD_GObj, a1: i32, a2: i32| { unsafe { let _ = crate::tu::granime::grAnime_801C8138(a0 as _, a1 as _, a2 as _); } };
    fn grAnime_801C83D0(a0: *mut HSD_GObj, a1: i32, a2: i32) -> i32 = |a0: *mut HSD_GObj, a1: i32, a2: i32| unsafe { crate::tu::granime::grAnime_801C83D0(a0 as _, a1 as _, a2 as _) as _ };
    fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = crate::tu::grdisplay::grDisplay_801C5DB0(a0 as _, a1 as _); } };
    fn grMaterial_801C8858(a0: *mut HSD_JObj, a1: u32) = |a0: *mut HSD_JObj, a1: u32| { unsafe { let _ = crate::tu::grmaterial::grMaterial_801C8858(a0 as _, a1 as _); } };
    fn grZakoGenerator_801CAE04(a0: *mut grZakoGenerator_SpawnDesc) -> *mut HSD_GObj = |a0: *mut grZakoGenerator_SpawnDesc| unsafe { crate::tu::grzakogenerator::grZakoGenerator_801CAE04(a0 as _) as _ };
    fn lbAudioAx_800237A8(a0: i32, a1: i32, a2: i32) -> i32 = |a0: i32, a1: i32, a2: i32| unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(a0 as _, a1 as _, a2 as _) as _ };
    fn lb_8000B1CC(a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec) = |a0: *mut HSD_JObj, a1: *mut Vec, a2: *mut Vec| { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000B1CC(a0 as _, a1 as _, a2 as _); } };
    fn lb_800115F4() = || { unsafe { let _ = melee_runtime::tu::lb_00F9::lb_800115F4(); } };
    fn lb_80011A50(a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32) -> *mut lb_80011A50_t = |a0: *mut Vec, a1: i32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32, a7: f32, a8: f32| unsafe { melee_runtime::tu::lb_00F9::lb_80011A50(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _, a5 as _, a6 as _, a7 as _, a8 as _) as _ };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
        pub unsafe extern "C" fn grDisplay_801C5DB0(a0: *mut HSD_GObj, a1: i32) { super::grDisplay_801C5DB0(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut stage_info: StageInfo = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, _j: &mut HSD_JObj) { unreachable!("HSD_JObjSetMtxDirty (out of line) is not called by this TU") }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { ext::HSD_JObjSetMtxDirtySub(j) }
}

/// gr/inlines.h `Ground_StartMapAnim`.
unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// gr/inlines.h `Ground_InitMapCollAndAnim`.
unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// `struct grOldpupupu_YakumonoParam` (file-local in the C).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOldpupupu_YakumonoParam {
    pub x0: i16,
    pub x2: i16,
    pub x4: i16,
    pub x6: i16,
    pub x8: i32,
    pub xC: i32,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1C: f32,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    pub x2C: f32,
    pub x30: f32,
}
melee_core::gc_layout!(grOldpupupu_YakumonoParam, "grOldpupupu_YakumonoParam", size 0x34, align 4, [
    x0 @ 0x0: 2 "s16", x2 @ 0x2: 2 "s16", x4 @ 0x4: 2 "s16", x6 @ 0x6: 2 "s16",
    x8 @ 0x8: 4 "int", xC @ 0xC: 4 "int", x10 @ 0x10: 4 "f32", x14 @ 0x14: 4 "f32", x18 @ 0x18: 4 "f32",
    x1C @ 0x1C: 4 "f32", x20 @ 0x20: 4 "f32", x24 @ 0x24: 4 "f32", x28 @ 0x28: 4 "f32", x2C @ 0x2C: 4 "f32",
    x30 @ 0x30: 4 "f32",
]);

/// `typedef struct grOldPupupuSpawnDesc`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct grOldPupupuSpawnDesc {
    pub x0: i8,
    pub x1: i8,
    pub x2: i16,
}
melee_core::gc_layout!(grOldPupupuSpawnDesc, "grOldPupupuSpawnDesc", size 4, align 2, [
    x0 @ 0x0: 1 "s8", x1 @ 0x1: 1 "s8", x2 @ 0x2: 2 "s16",
]);

/// `static struct grOldpupupu_YakumonoParam* yakumono_param`
pub static mut yakumono_param: *mut grOldpupupu_YakumonoParam = null_mut();

/// `static void* grOp_804D6A9C`
pub static mut grOp_804D6A9C: *mut c_void = null_mut();

pub static mut grOp_804D48A0: [[i32; 2]; 1] = [[4, 0]];
pub static mut grOp_804D48A8: [[i32; 2]; 1] = [[5, 1]];
pub static mut grOp_804D48B0: [[i32; 2]; 1] = [[2, 3]];
pub static mut grOp_804D48C8: [[i32; 2]; 1] = [[4, 0]];
pub static mut grOp_804D48D0: [[i32; 2]; 1] = [[5, 1]];
pub static mut grOp_804D48D8: [[i32; 2]; 1] = [[2, 3]];
pub static mut grOp_804D48E0: [[i32; 2]; 1] = [[1, 0]];

/// `grOp_803E67B0` (3E67B0)
pub static mut grOp_803E67B0: [grOldPupupuSpawnDesc; 10] = [
    grOldPupupuSpawnDesc { x0: -1, x1: 1, x2: 1 },
    grOldPupupuSpawnDesc { x0: 1, x1: 1, x2: 3 },
    grOldPupupuSpawnDesc { x0: 1, x1: 1, x2: 5 },
    grOldPupupuSpawnDesc { x0: -1, x1: 1, x2: 7 },
    grOldPupupuSpawnDesc { x0: -1, x1: 0, x2: 9 },
    grOldPupupuSpawnDesc { x0: 1, x1: 0, x2: 11 },
    grOldPupupuSpawnDesc { x0: -1, x1: 0, x2: 13 },
    grOldPupupuSpawnDesc { x0: 1, x1: 0, x2: 15 },
    grOldPupupuSpawnDesc { x0: -1, x1: 0, x2: 17 },
    grOldPupupuSpawnDesc { x0: 1, x1: 0, x2: 19 },
];

/// `grOp_803E67D8` (3E67D8)
pub static mut grOp_803E67D8: [i32; 3] = [0, 1, 2];

/// `grOp_803E67E4` (3E67E4)
pub static mut grOp_803E67E4: [Vec3; 2] = [Vec3 { x: -1.0, y: 0.0, z: 0.0 }, Vec3 { x: 1.0, y: 0.0, z: 0.0 }];

const fn cb(on_init: HSD_GObjEvent, callback1: HSD_GObjPredicate, gobj_proc: HSD_GObjEvent, callback3: Option<unsafe extern "C" fn(*mut Ground_GObj)>, flags: u32) -> StageCallbacks {
    StageCallbacks { on_init, callback1, gobj_proc, callback3, anon_10: StageCallbacks_anon_135 { flags } }
}

/// `grOp_StageCallbacks` (3E66A8)
pub static mut grOp_StageCallbacks: [StageCallbacks; 9] = [
    cb(Some(stageGObj0_OnInit), Some(grOldPupupu_802109C8), Some(grOldPupupu_802109D0), Some(grOldPupupu_802109D4), 0),
    cb(Some(grOldPupupu_802109D8), Some(grOldPupupu_80210A1C), Some(grOldPupupu_80210A24), Some(grOldPupupu_80210B00), 0),
    cb(Some(grOldPupupu_80210BE4), Some(grOldPupupu_80210C2C), Some(grOldPupupu_80210C34), Some(grOldPupupu_80210C78), 0),
    cb(Some(grOldPupupu_80211110), Some(grOldPupupu_8021118C), Some(grOldPupupu_80211194), Some(grOldPupupu_80211198), 0),
    cb(Some(grOldPupupu_80210B04), Some(grOldPupupu_80210B48), Some(grOldPupupu_80210B50), Some(grOldPupupu_80210B54), 0),
    cb(Some(grOldPupupu_80210B58), Some(grOldPupupu_80210BB8), Some(grOldPupupu_80210BC0), Some(grOldPupupu_80210BE0), (1 << 30) | (1 << 31)),
    cb(Some(grOldPupupu_80211C10), Some(grOldPupupu_80211C14), Some(grOldPupupu_80211C1C), Some(grOldPupupu_80211C9C), 0),
    cb(Some(grOldPupupu_8021119C), Some(grOldPupupu_80211284), Some(grOldPupupu_802113E0), Some(grOldPupupu_80211C0C), 0),
    cb(Some(grOldPupupu_80210C7C), Some(grOldPupupu_80210D08), Some(grOldPupupu_80210D10), Some(grOldPupupu_8021110C), 0),
];

/// `grOp_StageData` (3E6748)
pub static mut grOp_StageData: StageData = StageData {
    grkind: Gr_Kind_OldPupupu as _,
    callbacks: unsafe { &raw mut grOp_StageCallbacks as *mut StageCallbacks },
    data1: b"/GrOp.dat\0".as_ptr() as *mut i8,
    on_init: Some(grOldPupupu_802107E0),
    on_demo_init: Some(grOldPupupu_802107DC),
    on_load: Some(grOldPupupu_80210884),
    on_start: Some(grOldPupupu_80210888),
    callback4: Some(grOldPupupu_802108AC),
    on_touch_line: Some(grOldPupupu_80211CA0),
    on_check_shadow_render: Some(grOldPupupu_80211CA8),
    flags2: 1 << 0,
    joints: null_mut(),
    joint_count: 0,
};

pub unsafe extern "C" fn grOldPupupu_802107DC(_arg: i32) {}

pub unsafe extern "C" fn grOldPupupu_802107E0() {
    yakumono_param = ext::Ground_GetYakumonoParam() as *mut grOldpupupu_YakumonoParam;
    ext::stage_info.unk8C.set_b4(0);
    ext::stage_info.unk8C.set_b5(1);
    grOldPupupu_802108B4(0);
    grOldPupupu_802108B4(3);
    grOldPupupu_802108B4(7);
    grOldPupupu_802108B4(5);
    grOldPupupu_802108B4(4);
    grOldPupupu_802108B4(6);
    grOldPupupu_802108B4(1);
    grOldPupupu_802108B4(8);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    grOp_804D6A9C = null_mut();
}

pub unsafe extern "C" fn grOldPupupu_80210884() {}

pub unsafe extern "C" fn grOldPupupu_80210888() {
    ext::grZakoGenerator_801CAE04(null_mut());
}

pub unsafe extern "C" fn grOldPupupu_802108AC() -> i32 {
    0
}

/// Spells out `Ground_SetupStageCallbacks` (no `Mex_GrBind`) so that it stays out-of-line.
pub unsafe extern "C" fn grOldPupupu_802108B4(arg0: i32) -> *mut HSD_GObj {
    let callbacks = (&raw mut grOp_StageCallbacks as *mut StageCallbacks).add(arg0 as usize);
    let gobj = ext::Ground_GetStageGObj(arg0);
    if !gobj.is_null() {
        let gp = (*gobj).user_data as *mut Ground;
        (*gp).x8_callback = None;
        (*gp).xC_callback = None;
        ext::GObj_SetupGXLink(gobj, Some(ext::cb::grDisplay_801C5DB0), 3, 0);
        if (*callbacks).callback3.is_some() {
            (*gp).x1C_callback = (*callbacks).callback3;
        }
        if let Some(f) = (*callbacks).on_init {
            f(gobj);
        }
        if (*callbacks).gobj_proc.is_some() {
            ext::HSD_GObj_SetupProc(gobj, (*callbacks).gobj_proc, 4);
        }
    }
    gobj
}

pub unsafe extern "C" fn stageGObj0_OnInit(gobj: *mut HSD_GObj) {
    Ground_StartMapAnim(gobj);
}

pub unsafe extern "C" fn grOldPupupu_802109C8(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_802109D0(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_802109D4(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_802109D8(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.oldpupupu.xC4 = 0;
    (*gp).u.oldpupupu.xC8 = 0;
    (*gp).u.oldpupupu.xCC = 0;
    (*gp).u.oldpupupu.xD0 = 1;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

pub unsafe extern "C" fn grOldPupupu_80210A1C(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80210A24(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let v = &mut (*gp).u.oldpupupu;
    if (grOp_804D6A9C as usize as i32) == 0 && v.xD0 == 1 {
        v.xD0 = 0;
        match v.xC8 {
            0 => ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48A0[v.xCC as usize][v.xD4 as usize]),
            1 => ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48B0[v.xCC as usize][v.xD4 as usize]),
            2 => ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48A8[v.xCC as usize][v.xD4 as usize]),
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grOldPupupu_80210B00(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80210B04(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x10_flags.set_b5(1);
}

pub unsafe extern "C" fn grOldPupupu_80210B48(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80210B50(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80210B54(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80210B58(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    Ground_InitMapCollAndAnim(gobj);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grOldPupupu_80210BB8(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80210BC0(_gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
}

pub unsafe extern "C" fn grOldPupupu_80210BE0(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80210BE4(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    ext::HSD_JObjSetFlagsAll((*gobj).hsd_obj as *mut HSD_JObj, JOBJ_HIDDEN as u32);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grOldPupupu_80210C2C(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80210C34(gobj: *mut HSD_GObj) {
    if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
        ext::Ground_801C4A08(gobj);
    }
}

pub unsafe extern "C" fn grOldPupupu_80210C78(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80210C7C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    let x2 = (*yp).x2 as i32;
    let x0 = (*yp).x0 as i32;
    (*gp).u.oldpupupu2.xC4 = if x2 > x0 {
        x0.wrapping_add(if x2.wrapping_sub(x0) != 0 { ext::HSD_Randi(x2.wrapping_sub(x0)) } else { 0 })
    } else if x2 < x0 {
        x2.wrapping_add(if x0.wrapping_sub(x2) != 0 { ext::HSD_Randi(x0.wrapping_sub(x2)) } else { 0 })
    } else {
        x2
    } as i16;
}

pub unsafe extern "C" fn grOldPupupu_80210D08(_gobj: *mut HSD_GObj) -> i32 {
    0
}

/// `grOldPupupu_GetSpawnJObj` (file-local inline): `HSD_ASSERT(0x216, jobj)`
pub unsafe fn grOldPupupu_GetSpawnJObj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    assert!(!jobj.is_null(), "HSD_ASSERT(0x216, jobj) groldpupupu.c");
    jobj
}

pub unsafe extern "C" fn grOldPupupu_80210D10(gobj: *mut HSD_GObj) {
    let mut cam_left: f32 = 0.0;
    let mut cam_right: f32 = 0.0;
    let mut cam_center: f32 = 0.0;
    let yp = yakumono_param;
    let gp = (*gobj).user_data as *mut Ground;
    let timer = (*gp).u.oldpupupu2.xC4;
    (*gp).u.oldpupupu2.xC4 = timer.wrapping_sub(1);
    if timer < 0 {
        let direction = ext::Ground_801C0498();
        let index = ext::HSD_Randi(10);
        let desc = grOp_803E67B0[index as usize];
        let count: i32 = if desc.x1 != 0 && ext::HSD_Randi(5) == 0 { 3 } else { 1 };
        ext::Camera_LogicBounds(&mut cam_left, &mut cam_center, &mut cam_right);
        let step: f32;
        let mut x: f32;
        if desc.x0 == 1 {
            step = -10.0;
            if cam_right < 200.0 {
                cam_right = 200.0;
            }
            // -((count * step) - (50 + cam_right)): the inner product-minus is fused
            x = -fmadds(count as f32, step, -(50.0 + cam_right));
        } else {
            step = 10.0;
            if cam_left > -200.0 {
                cam_left = -200.0;
            }
            x = -fmadds(count as f32, step, -(cam_left - 50.0));
        }
        let y;
        {
            let rand = ext::HSD_Randf();
            y = direction * fmadds((*yp).x4 as f32, fmadds(2.0, rand, -1.0), 20.0);
        }
        let z = -150.0 * direction;
        let mut i = 0;
        while i < count {
            let spawn = grOldPupupu_802108B4(2);
            if !spawn.is_null() {
                let jobj = grOldPupupu_GetSpawnJObj(spawn);
                HSD_JObjSetTranslateX(&mut *jobj, direction * x, &mut Dirty);
                HSD_JObjSetTranslateY(&mut *jobj, y, &mut Dirty);
                HSD_JObjSetTranslateZ(&mut *jobj, z, &mut Dirty);
                ext::grAnime_801C7FF8(spawn, 0, 7, 0, 40.0 * (i as f32), 1.0);
                ext::HSD_JObjClearFlagsAll(ext::Ground_801C3FA4(spawn, desc.x2 as i32), JOBJ_HIDDEN as u32);
            }
            x += step;
            i += 1;
        }

        let respawn = (*yp).x2 as i32;
        let min_respawn = (*yp).x0 as i32;
        let respawn = if respawn > min_respawn {
            min_respawn.wrapping_add(if respawn.wrapping_sub(min_respawn) != 0 { ext::HSD_Randi(respawn.wrapping_sub(min_respawn)) } else { 0 })
        } else if respawn < min_respawn {
            respawn.wrapping_add(if min_respawn.wrapping_sub(respawn) != 0 { ext::HSD_Randi(min_respawn.wrapping_sub(respawn)) } else { 0 })
        } else {
            respawn
        };
        (*gp).u.oldpupupu2.xC4 = respawn as i16;
    }
}

pub unsafe extern "C" fn grOldPupupu_8021110C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80211110(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*((*gobj).user_data as *mut Ground)).x10_flags.set_b2(0);
    ext::grMaterial_801C8858(jobj, 0x28000000);
    (*gp).x11_flags.set_b012(2);
}

pub unsafe extern "C" fn grOldPupupu_8021118C(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80211194(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80211198(_arg: *mut HSD_GObj) {}

/// `randrange` (file-local inline)
pub unsafe fn randrange(min: i32, max: i32) -> i32 {
    if min > max {
        let diff = min.wrapping_sub(max);
        let rng = if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
        max.wrapping_add(rng)
    } else if min < max {
        let diff = max.wrapping_sub(min);
        let rng = if diff != 0 { ext::HSD_Randi(diff) } else { 0 };
        min.wrapping_add(rng)
    } else {
        min
    }
}

pub unsafe extern "C" fn grOldPupupu_8021119C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;
    ext::ftCo_800C06E8(gobj, 0xA, fn_802112F4 as usize as *mut c_void);
    (*gp).u.oldpupupu.xC4 = 0;
    (*gp).u.oldpupupu.xC8 = 0;
    (*gp).u.oldpupupu.xCC = 0;
    (*gp).u.oldpupupu.xD0 = randrange((*yp).xC, (*yp).x8);
    (*gp).u.oldpupupu.xD4 = 1;
    (*gp).u.oldpupupu.xE0 = 0;
    (*gp).u.oldpupupu.xD8 = 1;
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
    (*gp).x11_flags.set_b012(1);
}

pub unsafe extern "C" fn grOldPupupu_80211284(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_8021128C(vec: *mut Vec, max_x: f32, mut min_x: f32, max_y: f32, mut min_y: f32) -> i32 {
    let max_x_actual: f32;
    let max_y_actual: f32;
    if max_x > min_x {
        max_x_actual = max_x;
    } else {
        max_x_actual = min_x;
        min_x = max_x;
    }
    if max_y > min_y {
        max_y_actual = max_y;
    } else {
        max_y_actual = min_y;
        min_y = max_y;
    }
    if min_x < (*vec).x && (*vec).x < max_x_actual {
        if min_y < (*vec).y && (*vec).y < max_y_actual {
            return 1;
        }
    }
    0
}

pub unsafe extern "C" fn fn_802112F4(gobj: *mut HSD_GObj, fighter_gobj: *mut HSD_GObj, vel: *mut Vec3) -> i32 {
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let yp = yakumono_param;
    ext::ftLib_80086644(fighter_gobj, &mut pos);
    (*vel).y = 0.0;
    (*vel).z = 0.0;

    let gp = (*gobj).user_data as *mut Ground;
    match (*gp).u.oldpupupu.xDC {
        1 => {
            if grOldPupupu_8021128C(&mut pos, (*yp).x1C, (*yp).x20, (*yp).x24, (*yp).x28) == 1 {
                (*vel).x = -(*yp).x10;
                return 1;
            }
        }
        2 => {
            if grOldPupupu_8021128C(&mut pos, (*yp).x18, (*yp).x14, (*yp).x24, (*yp).x28) == 1 {
                (*vel).x = (*yp).x10;
                return 1;
            }
        }
        _ => {}
    }

    (*vel).x = 0.0;
    0
}

/// `grOldPupupu_SyncSecondary` (file-local inline)
pub unsafe fn grOldPupupu_SyncSecondary(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let other_gp = (*ext::Ground_GetMapGObj(1)).user_data as *mut Ground;
    (*other_gp).u.oldpupupu.xC4 = (*gp).u.oldpupupu.xC4;
    (*other_gp).u.oldpupupu.xC8 = (*gp).u.oldpupupu.xC8;
    (*other_gp).u.oldpupupu.xCC = 0;
    (*other_gp).u.oldpupupu.xD4 = (*gp).u.oldpupupu.xD8;
    (*other_gp).u.oldpupupu.xD0 = 1;
}

/// `grOldPupupu_Advance` (file-local inline)
pub unsafe fn grOldPupupu_Advance(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    (*gp).u.oldpupupu.xC4 = (*gp).u.oldpupupu.xC4.wrapping_add(1);
    if (*gp).u.oldpupupu.xC4 >= 3 {
        (*gp).u.oldpupupu.xC4 = 0;
    }
    (*gp).u.oldpupupu.xC8 = grOp_803E67D8[(*gp).u.oldpupupu.xC4 as usize];
    (*gp).u.oldpupupu.xD4 = 1;
}

pub unsafe extern "C" fn grOldPupupu_802113E0(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let yp = yakumono_param;

    if (grOp_804D6A9C as usize as i32) == 0 {
        (*gp).u.oldpupupu.xE0 = (*gp).u.oldpupupu.xE0.wrapping_add(1);
        (*gp).u.oldpupupu.xDC = 0;

        match (*gp).u.oldpupupu.xC8 {
            0 => {
                if (*gp).u.oldpupupu.xD4 != 0 {
                    (*gp).u.oldpupupu.xD4 = 0;
                    (*gp).u.oldpupupu.xCC = 0;
                    (*gp).u.oldpupupu.xD0 = randrange((*yp).xC, (*yp).x8);
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48C8[(*gp).u.oldpupupu.xCC as usize][(*gp).u.oldpupupu.xD8 as usize]);
                    grOldPupupu_SyncSecondary(gobj);
                    (*gp).u.oldpupupu.xCC = randrange(f2i32((*yp).x30 as f64), f2i32((*yp).x2C as f64));
                }

                (*gp).u.oldpupupu.xD0 = (*gp).u.oldpupupu.xD0.wrapping_sub(1);
                if ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                    if (*gp).u.oldpupupu.xD0 > 0 {
                        (*gp).u.oldpupupu.xCC = (*gp).u.oldpupupu.xCC.wrapping_sub(1);
                        if (*gp).u.oldpupupu.xCC <= 0 {
                            (*gp).u.oldpupupu.xCC = 0;
                            ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48C8[(*gp).u.oldpupupu.xCC as usize][(*gp).u.oldpupupu.xD8 as usize]);
                            (*gp).u.oldpupupu.xCC = randrange(f2i32((*yp).x30 as f64), f2i32((*yp).x2C as f64));
                        }
                        grOldPupupu_SyncSecondary(gobj);
                        return;
                    }

                    {
                        grOldPupupu_Advance(gobj);
                        // grOldPupupu_SyncSecondary would recolor gp here (the C spells it out)
                        let gp = (*gobj).user_data as *mut Ground;
                        let other_gp = (*ext::Ground_GetMapGObj(1)).user_data as *mut Ground;
                        (*other_gp).u.oldpupupu.xC4 = (*gp).u.oldpupupu.xC4;
                        (*other_gp).u.oldpupupu.xC8 = (*gp).u.oldpupupu.xC8;
                        (*other_gp).u.oldpupupu.xCC = 0;
                        (*other_gp).u.oldpupupu.xD4 = (*gp).u.oldpupupu.xD8;
                        (*other_gp).u.oldpupupu.xD0 = 1;
                        return;
                    }
                }
            }

            1 => {
                if (*gp).u.oldpupupu.xD4 != 0 {
                    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

                    (*gp).u.oldpupupu.xCC = 0;
                    HSD_JObjGetTranslation(&*((*gobj).hsd_obj as *mut HSD_JObj), &mut pos);
                    let side: i32 = if ext::ftLib_800864A8(&mut pos, null_mut()) == 1.0 { 1 } else { 0 };
                    let side_f = side as f32;
                    if ((*gp).u.oldpupupu.xD8 as f32) == side_f {
                        (*gp).u.oldpupupu.xC8 = 2;
                        (*gp).u.oldpupupu.xD4 = 0;
                        (*gp).u.oldpupupu.xCC = 0;
                        (*gp).u.oldpupupu.xD0 = 0;
                        ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48D0[(*gp).u.oldpupupu.xCC as usize][(*gp).u.oldpupupu.xD8 as usize]);
                        grOldPupupu_Advance(gobj);
                        grOldPupupu_SyncSecondary(gobj);
                        grOldPupupu_SyncSecondary(gobj);
                    } else {
                        (*gp).u.oldpupupu.xD8 = f2i32(side_f as f64);
                        (*gp).u.oldpupupu.xD4 = 0;
                        (*gp).u.oldpupupu.xCC = 0;
                        (*gp).u.oldpupupu.xD0 = 0;
                        ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48D8[(*gp).u.oldpupupu.xCC as usize][(*gp).u.oldpupupu.xD8 as usize]);
                        grOldPupupu_SyncSecondary(gobj);
                    }
                }

                if (*gp).u.oldpupupu.xC8 == 1 && ext::grAnime_801C83D0(gobj, 0, 7) != 0 {
                    grOldPupupu_Advance(gobj);
                    grOldPupupu_SyncSecondary(gobj);
                    return;
                }
            }

            2 => {
                if (*gp).u.oldpupupu.xD4 != 0 {
                    (*gp).u.oldpupupu.xD4 = 0;
                    (*gp).u.oldpupupu.xCC = 0;
                    (*gp).u.oldpupupu.xD0 = 0;
                    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48D0[(*gp).u.oldpupupu.xCC as usize][(*gp).u.oldpupupu.xD8 as usize]);
                    grOldPupupu_SyncSecondary(gobj);
                }

                if ext::grAnime_801C83D0(gobj, 0, 7) == 0 {
                    (*gp).u.oldpupupu.xD0 = (*gp).u.oldpupupu.xD0.wrapping_add(1);
                    if (*gp).u.oldpupupu.xD0 == 0x2D {
                        ext::lbAudioAx_800237A8(if (*gp).u.oldpupupu.xD8 == 0 { 0x72BF0 } else { 0x72BF1 }, 0x7F, 0x40);
                    }
                    if (*gp).u.oldpupupu.xD0 > 0x2D && (*gp).u.oldpupupu.xD0 < 0x140 {
                        ext::Camera_RequestQuake(QuakeKind_Loop as _, null_mut());
                        (*gp).u.oldpupupu.xDC = (*gp).u.oldpupupu.xD8.wrapping_add(1);
                        if ((*gp).u.oldpupupu.xD0 % 10) == 0 {
                            let gp = (*gobj).user_data as *mut Ground;
                            if (*gp).u.oldpupupu.xD8 == 0 {
                                ext::lb_80011A50(
                                    (&raw mut grOp_803E67E4 as *mut Vec3).add((*gp).u.oldpupupu.xD8 as usize), 0xF, 0.5, 0.0, 0.0,
                                    (*yp).x20, (*yp).x24, (*yp).x1C, (*yp).x28,
                                );
                            } else {
                                ext::lb_80011A50(
                                    (&raw mut grOp_803E67E4 as *mut Vec3).add((*gp).u.oldpupupu.xD8 as usize), 0xF, 0.5, 0.0, 0.0,
                                    (*yp).x14, (*yp).x24, (*yp).x18, (*yp).x28,
                                );
                            }
                            return;
                        }
                    }
                } else {
                    grOldPupupu_Advance(gobj);
                    grOldPupupu_SyncSecondary(gobj);
                }
            }
            _ => {}
        }
    }
}

pub unsafe extern "C" fn grOldPupupu_80211C0C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80211C10(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80211C14(_gobj: *mut HSD_GObj) -> i32 {
    0
}

pub unsafe extern "C" fn grOldPupupu_80211C1C(gobj: *mut HSD_GObj) {
    let gp = (*gobj).user_data as *mut Ground;
    let other_gp = (*ext::Ground_GetMapGObj(7)).user_data as *mut Ground;

    if (*other_gp).u.oldpupupu.xC8 == 2 && (*other_gp).u.oldpupupu.xD4 != 0 {
        ext::grAnime_801C8138(gobj, (*gp).map_id as i32, grOp_804D48E0[(*other_gp).u.oldpupupu.xCC as usize][(*other_gp).u.oldpupupu.xD8 as usize]);
    }
}

pub unsafe extern "C" fn grOldPupupu_80211C9C(_arg: *mut HSD_GObj) {}

pub unsafe extern "C" fn grOldPupupu_80211CA0(_gobj: i32) -> *mut DynamicsDesc {
    null_mut()
}

pub unsafe extern "C" fn grOldPupupu_80211CA8(a: *mut Vec3, _arg: i32, joint: *mut HSD_JObj) -> i32 {
    let mut b = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    ext::lb_8000B1CC(joint, null_mut(), &mut b);
    if (*a).y > b.y {
        1
    } else {
        0
    }
}
