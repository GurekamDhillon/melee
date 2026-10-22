//! src/melee/gm/gmapproach.c: the Approach (ready-to-fight) scene: its model, camera and light setup,
//! the idle animation step and the start-button check. All five functions are checked bit for bit
//! against the C by a state oracle (tests/tu_gmapproach.rs over gen/turef/tu_gmapproach_ref.c).
//! Callbacks cross the hook boundary as their address (`usize`), as the C compares them.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::c_void;
use melee_types::*;

/// `static struct { ... } gm_80480D98;`: the scene's archive, animation counters and pad slot.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct gm_80480D98_t {
    pub x0: *mut HSD_Archive,
    pub x4: i32,
    pub x8: u16,
    pub xA: u16,
    pub xC: u16,
    pub xE: u8,
}

/// The scene's state (`gm_80480D98`).
pub static mut GM_80480D98: gm_80480D98_t = gm_80480D98_t {
    x0: core::ptr::null_mut(),
    x4: 0,
    x8: 0,
    xA: 0,
    xC: 0,
    xE: 0,
};

/// Calls into TUs that are not ported yet (engine, archive, audio, pad, the gm pool). Each is a settable
/// hook; an unset hook panics. Callbacks are passed as their address.
pub mod ext {
    use super::*;
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
    /// `lbArchive_80016DBC(filename, &symbols, name, flag)`: loads the scene archive; `symbols` gets the desc.
    hook!(LBARCHIVE_80016DBC, lbArchive_80016DBC, (filename: &str, symbols: *mut *mut SceneDesc, name: &str, flag: i32) -> *mut HSD_Archive);
    /// `lbArchive_80016EFC(archive)`.
    hook!(LBARCHIVE_80016EFC, lbArchive_80016EFC, (archive: *mut HSD_Archive));
    /// `GObj_Create(classifier, p_link, priority)`.
    hook!(GOBJ_CREATE, GObj_Create, (classifier: u16, p_link: u8, priority: u8) -> *mut HSD_GObj);
    /// `HSD_CObjLoadDesc(desc)`.
    hook!(HSD_COBJLOADDESC, HSD_CObjLoadDesc, (desc: *mut HSD_CObjDesc) -> *mut HSD_CObj);
    /// `HSD_GObjObject_80390A70(gobj, kind, obj)`.
    hook!(HSD_GOBJOBJECT_80390A70, HSD_GObjObject_80390A70, (gobj: *mut HSD_GObj, kind: u8, obj: *mut c_void));
    /// `GObj_SetupGXLinkMax(gobj, render, priority)` (the render callback's address).
    hook!(GOBJ_SETUPGXLINKMAX, GObj_SetupGXLinkMax, (gobj: *mut HSD_GObj, render: usize, priority: u32));
    /// `lb_80011AC4(lights)`.
    hook!(LB_80011AC4, lb_80011AC4, (lights: *mut *mut LightList) -> *mut HSD_LObj);
    /// `GObj_SetupGXLink(gobj, render, gx_link, priority)`.
    hook!(GOBJ_SETUPGXLINK, GObj_SetupGXLink, (gobj: *mut HSD_GObj, render: usize, gx_link: u8, priority: u32));
    /// `HSD_JObjLoadJoint(joint)`.
    hook!(HSD_JOBJLOADJOINT, HSD_JObjLoadJoint, (joint: *mut HSD_Joint) -> *mut HSD_JObj);
    /// `gm_8016895C(jobj, model, idx)` (gm/gm_1689.c).
    hook!(GM_8016895C, gm_8016895C, (jobj: *mut HSD_JObj, model: *mut DynamicModelDesc, idx: i32));
    /// `HSD_JObjReqAnimAll(jobj, frame)`.
    hook!(HSD_JOBJREQANIMALL, HSD_JObjReqAnimAll, (jobj: *mut HSD_JObj, frame: f32));
    /// `HSD_JObjAnimAll(jobj)`.
    hook!(HSD_JOBJANIMALL, HSD_JObjAnimAll, (jobj: *mut HSD_JObj));
    /// `HSD_GObj_SetupProc(gobj, proc, priority)` (the proc's address).
    hook!(HSD_GOBJ_SETUPPROC, HSD_GObj_SetupProc, (gobj: *mut HSD_GObj, proc: usize, priority: u8));
    /// `HSD_TObjReqAnimAll(tobj, start_frame)`.
    hook!(HSD_TOBJREQANIMALL, HSD_TObjReqAnimAll, (tobj: *mut HSD_TObj, start_frame: f32));
    /// `HSD_AObjSetRate(aobj, rate)`.
    hook!(HSD_AOBJSETRATE, HSD_AObjSetRate, (aobj: *mut HSD_AObj, rate: f32));
    /// `gm_801A4B60()` (gm/gmscene: the scene exit).
    hook!(GM_801A4B60, gm_801A4B60, ());
    /// `lbAudioAx_80023F28(id)` / `lbAudioAx_800236DC()` (lb/lbaudio_ax.c).
    hook!(LBAUDIOAX_80023F28, lbAudioAx_80023F28, (id: i32) -> i32);
    hook!(LBAUDIOAX_800236DC, lbAudioAx_800236DC, () -> i32);
    /// `HSD_PadCopyStatus[idx].trigger` (sysdolphin/baselib/controller.c).
    hook!(HSD_PADCOPYSTATUS_TRIGGER, hsd_pad_copy_status_trigger, (idx: usize) -> u32);
    /// `HSD_GObj_JObjKind` / `HSD_GObj_LightKind` / `HSD_GObj_CameraKind` (sysdolphin/baselib/gobj.c).
    hook!(HSD_GOBJ_JOBJKIND, hsd_gobj_jobj_kind, () -> u8);
    hook!(HSD_GOBJ_LIGHTKIND, hsd_gobj_light_kind, () -> u8);
    hook!(HSD_GOBJ_CAMERAKIND, hsd_gobj_camera_kind, () -> u8);

    /// `HSD_GObj_803910D8` (render callback, gm's GX link): address only; not ported yet.
    pub unsafe extern "C" fn HSD_GObj_803910D8(_gobj: *mut HSD_GObj, _code: i32) {
        panic!("TODO(sysdolphin): HSD_GObj_803910D8 is not ported");
    }
    /// `HSD_GObj_LObjCallback` / `HSD_GObj_JObjCallback` (sysdolphin/baselib/gobj.c): addresses only.
    pub unsafe extern "C" fn HSD_GObj_LObjCallback(_gobj: *mut HSD_GObj, _code: i32) {
        panic!("TODO(sysdolphin): HSD_GObj_LObjCallback is not ported");
    }
    pub unsafe extern "C" fn HSD_GObj_JObjCallback(_gobj: *mut HSD_GObj, _code: i32) {
        panic!("TODO(sysdolphin): HSD_GObj_JObjCallback is not ported");
    }
}

/// `static void fn_801AD920(HSD_GObj* gobj)`: the idle step of the approach models (the GX proc).
pub unsafe extern "C" fn fn_801AD920(gobj: *mut HSD_GObj) {
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    let var_r3: *mut HSD_JObj = if (*gobj).hsd_obj.is_null() { core::ptr::null_mut() } else { (*jobj).child };
    let var_r4: *mut HSD_JObj = if var_r3.is_null() { core::ptr::null_mut() } else { (*var_r3).next };
    let var_r0: *mut HSD_JObj = if var_r4.is_null() { core::ptr::null_mut() } else { (*var_r4).next };

    if GM_80480D98.x8 <= 0x12C {
        GM_80480D98.x8 += 1;
    } else {
        GM_80480D98.x8 = 0;
    }
    if GM_80480D98.xA < 0x12C {
        GM_80480D98.xA += 1;
    }
    ext::HSD_JObjReqAnimAll(var_r3, GM_80480D98.x8 as f32);
    ext::HSD_JObjReqAnimAll(var_r4, GM_80480D98.xA as f32);
    ext::HSD_JObjReqAnimAll(var_r0, GM_80480D98.xA as f32);
    let var_r0_2: i32 = match GM_80480D98.x4 {
        3 => 1,
        7 => 2,
        9 => 3,
        10 => 4,
        15 => 5,
        20 => 6,
        21 => 7,
        22 => 8,
        23 => 9,
        24 => 10,
        25 => 11,
        _ => 0,
    };
    let tobj = (*(*(*var_r0).u.dobj).mobj).tobj;
    ext::HSD_TObjReqAnimAll(tobj, var_r0_2 as f32);
    ext::HSD_AObjSetRate((*tobj).aobj, 0.0);
    ext::HSD_JObjAnimAll(jobj);
}

/// `static void gm_801ADB04(void)`: loads the approach scene (camera, light, model) and starts its proc.
pub fn gm_801ADB04() {
    unsafe {
        let mut spc: *mut SceneDesc = core::ptr::null_mut();
        GM_80480D98.x0 = ext::lbArchive_80016DBC("NtAppro", &mut spc, "ScNtcApproach_scene_data", 0);

        let mut gobj = ext::GObj_Create(0x13, 0x14, 0);
        let cobj = ext::HSD_CObjLoadDesc((*(*spc).cameras.add(0)).desc);
        ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_camera_kind(), cobj as *mut c_void);
        ext::GObj_SetupGXLinkMax(gobj, ext::HSD_GObj_803910D8 as usize, 0);
        (*gobj).gxlink_prios = 0x4801;

        gobj = ext::GObj_Create(0xB, 0xF, 0);
        let lobj = ext::lb_80011AC4((*spc).lights);
        ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_light_kind(), lobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_LObjCallback as usize, 0, 0);

        gobj = ext::GObj_Create(0xE, 0xF, 0);
        let model0 = *(*spc).models.add(0);
        let jobj = ext::HSD_JObjLoadJoint((*model0).joint);
        ext::HSD_GObjObject_80390A70(gobj, ext::hsd_gobj_jobj_kind(), jobj as *mut c_void);
        ext::GObj_SetupGXLink(gobj, ext::HSD_GObj_JObjCallback as usize, 0xB, 0);
        ext::gm_8016895C(jobj, model0, 0);
        ext::HSD_JObjReqAnimAll(jobj, 0.0);
        ext::HSD_JObjAnimAll(jobj);
        ext::HSD_GObj_SetupProc(gobj, fn_801AD920 as usize, 1);
    }
}

/// `gm_Scene_Approach_OnFrame`: after 180 frames, the start button (or A) leaves the scene.
pub fn gm_Scene_Approach_OnFrame() {
    unsafe {
        let old = GM_80480D98.xC;
        GM_80480D98.xC = GM_80480D98.xC.wrapping_add(1);
        if old as i32 >= 0xB4 && (ext::hsd_pad_copy_status_trigger(GM_80480D98.xE as usize) & (HSD_PAD_A | HSD_PAD_START) as u32) != 0 {
            ext::gm_801A4B60();
        }
    }
}

/// `gm_Scene_Approach_OnEnter(arg0)`: arg0[0] is the scene kind (clamped to the approach set), arg0[1] the pad.
pub fn gm_Scene_Approach_OnEnter(arg0_: *mut c_void) {
    unsafe {
        let arg0 = arg0_ as *const i8;
        gm_801ADB04();
        let mut var_r0 = *arg0 as i32;
        if var_r0 != 3
            && var_r0 != 7
            && var_r0 != 9
            && var_r0 != 10
            && var_r0 != 15
            && var_r0 != 20
            && var_r0 != 21
            && var_r0 != 22
            && var_r0 != 23
            && var_r0 != 24
            && var_r0 != 25
        {
            var_r0 = 25;
        }
        GM_80480D98.x4 = var_r0;
        GM_80480D98.xE = *arg0.add(1) as u8;
        GM_80480D98.x8 = 0;
        GM_80480D98.xA = 0;
        GM_80480D98.xC = 0;
        ext::lbAudioAx_80023F28(0x48);
    }
}

/// `gm_Scene_Approach_OnExit(exit_data)`: unload the scene archive and stop its audio.
pub fn gm_Scene_Approach_OnExit(_exit_data: *mut c_void) {
    unsafe {
        ext::lbArchive_80016EFC(GM_80480D98.x0);
        ext::lbAudioAx_800236DC();
    }
}
