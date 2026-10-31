//! src/melee/ft/kinds/ftCommon/ftCo_09F4.c: the fighter light overlay. `ftCo_8009F4A4` builds the
//! stage light pair (`lobj0` with the overlay setup, `lobj1` lit without it) and the GObj that animates
//! it; `ftCo_8009F578` gives a fighter its own light from the file's static light list; `ftCo_8009F5AC`
//! / `ftCo_8009F75C` / `ftCo_8009F7F8` switch the lights on for a fighter's draw.
//!
//! The file's static light data is GameCube-layout data in the C (`floats` read through a
//! `HSD_WObjDesc*`): here the `HSD_WObjDesc` is the native record with the same values (`class_name`
//! 0, `pos` 0.57 x3, `robjdesc` 0). No fused sites (fma_sites.py): `position.z * sinf(angle_x)` has no add.
//! Checked against the C by tests/tu_ftCo_09f4_vs_c.rs (everything but the GObj creation).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::types::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::tu::lobj as lo;

/// Callees outside this TU, each a settable hook (the C oracle in tests/tu_ftCo_09f4_vs_c.rs installs
/// recording stubs) that runs the runtime / world port when unset.
pub mod ext {
    use super::*;
    /// `HSD_GObj_LightKind` (baselib/gobjobject.c): set by the engine's init; the light GObj object kind.
    pub use melee_runtime::tu::gobj_native::HSD_GObj_LightKind;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $d:expr;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => $d,
                }
            } )*
        };
    }
    ext_fns! {
        fn GObj_Create(class: u16, p_link: u8, prio: u8) -> *mut HSD_GObj = unsafe { melee_runtime::tu::gobj_native::GObj_Create(class, p_link, prio) };
        fn Ground_801C49B4() -> *mut *mut LightList = unsafe { melee_world::tu::ground::Ground_801C49B4() };
        fn lb_80011AC4(list: *mut *mut LightList) -> *mut HSD_LObj = unsafe { melee_runtime::tu::lbspdisplay::lb_80011AC4(list) };
        fn lb_8000CDC0(l: *mut HSD_LObj) -> *mut HSD_LObj = unsafe { melee_runtime::tu::lb_00B0::lb_8000CDC0(l) };
        fn Ground_801C2374(l: *mut HSD_LObj) = unsafe { melee_world::tu::ground::Ground_801C2374(l) };
        fn HSD_GObjObject_80390A70(g: *mut HSD_GObj, kind: u8, o: *mut c_void) = unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(g, kind, o) };
        fn GObj_SetupGXLink(g: *mut HSD_GObj, cb: GObj_RenderFunc, gx_link: u8, prio: u32) = unsafe { melee_runtime::tu::gobj_native::GObj_SetupGXLink(g, cb, gx_link, prio) };
        fn HSD_GObj_SetupProc(g: *mut HSD_GObj, f: HSD_GObjEvent, prio: u8) = unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(g, f, prio); };
        fn HSD_LObjAnimAll(l: *mut HSD_LObj) = unsafe { lo::HSD_LObjAnimAll(l) };
        fn HSD_LObjReqAnimAll(l: *mut HSD_LObj, f: f32) = unsafe { lo::HSD_LObjReqAnimAll(l, f) };
        fn HSD_LObj_803668EC(l: *mut HSD_LObj) = unsafe { lo::HSD_LObj_803668EC(l) };
        fn HSD_LObjSetupInit(c: *mut HSD_CObj) = unsafe { lo::HSD_LObjSetupInit(c) };
        fn HSD_CObjGetCurrent() -> *mut HSD_CObj = unsafe { melee_runtime::tu::cobj::HSD_CObjGetCurrent() };
        fn HSD_LObjSetPosition(l: *mut HSD_LObj, v: *const Vec) = unsafe { lo::HSD_LObjSetPosition(l, v) };
        fn HSD_LObjSetColor(l: *mut HSD_LObj, c: _GXColor) = unsafe { lo::HSD_LObjSetColor(l, c) };
        fn HSD_LObjSetFlags(l: *mut HSD_LObj, f: u32) = unsafe { lo::HSD_LObjSetFlags(l, f) };
        fn HSD_LObjClearFlags(l: *mut HSD_LObj, f: u32) = unsafe { lo::HSD_LObjClearFlags(l, f) };
        fn HSD_LObjSetCurrentAll(l: *mut HSD_LObj) = unsafe { lo::HSD_LObjSetCurrentAll(l) };
        fn HSD_LObjAddCurrent(l: *mut HSD_LObj) = unsafe { lo::HSD_LObjAddCurrent(l) };
        fn ftCo_800C0658(fp: *mut Fighter) -> *mut ColorOverlay = unsafe { crate::ft0df::ftco_800c0658(&mut *fp) };
    }
}

/// `lobj0`: the stage light with the fighter-overlay setup.
pub static mut lobj0: *mut HSD_LObj = null_mut();
/// `lobj1`: the copy lit without the overlay.
pub static mut lobj1: *mut HSD_LObj = null_mut();

/// `floats[]`' meaning: `{ 0, 0.57, 0.57, 0.57, 0 }` read as a GC `HSD_WObjDesc`.
pub static mut node0_position: HSD_WObjDesc = HSD_WObjDesc { class_name: null_mut(), pos: Vec { x: 0.57, y: 0.57, z: 0.57 }, robjdesc: null_mut() };

pub static mut node0: HSD_LightDesc = {
    let mut d: HSD_LightDesc = unsafe { core::mem::zeroed() };
    d.flags = 0x0005;
    d.attnflags = 0x0000;
    d.color = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
    d.position = &raw mut node0_position;
    d
};
pub static mut node1: LightList = LightList { desc: &raw mut node0, anims: null_mut() };
pub static mut node2: [*mut LightList; 2] = [&raw mut node1, null_mut()];

/// `ftCo_8009F480`: the GObj proc, animates the light.
pub unsafe extern "C" fn ftCo_8009F480(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftCo_8009F480");
    ext::HSD_LObjAnimAll((*gobj).hsd_obj as *mut HSD_LObj);
}

/// `ftCo_8009F4A4`: create the light GObj (render hook `ftCo_8009F54C`, proc `ftCo_8009F480`).
pub unsafe fn ftCo_8009F4A4() {
    melee_core::ct!("ftCo_8009F4A4");
    let gobj = ext::GObj_Create(12, 3, 0);
    let lobj = ext::lb_80011AC4(ext::Ground_801C49B4());
    lobj0 = lobj;
    lobj1 = ext::lb_8000CDC0(lobj);
    ext::Ground_801C2374(lobj);
    ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_LightKind as u8, lobj as *mut c_void);
    ext::GObj_SetupGXLink(gobj, Some(ftCo_8009F54C), 4, 0);
    ext::HSD_LObjReqAnimAll(lobj, 0.0);
    ext::HSD_GObj_SetupProc(gobj, Some(ftCo_8009F480), 1);
}

/// `ftCo_8009F54C`: the GObj render hook.
pub unsafe extern "C" fn ftCo_8009F54C(gobj: *mut HSD_GObj, _code: i32) {
    melee_core::ct!("ftCo_8009F54C");
    ext::HSD_LObj_803668EC((*gobj).hsd_obj as *mut HSD_LObj);
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
}

/// `ftCo_8009F578`: the fighter's own light, from the static list.
pub unsafe fn ftCo_8009F578(fp: *mut Fighter) {
    melee_core::ct!("ftCo_8009F578");
    (*fp).x588 = ext::lb_80011AC4((&raw mut node2) as *mut *mut LightList);
}

/// `MTXDegToRad` (dolphin/mtx.h): `a * 0.01745329252f`.
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `ftCo_8009F5AC`: refresh the overlay light from the colour overlay data and make it current.
pub unsafe fn ftCo_8009F5AC(fp: *mut Fighter) {
    melee_core::ct!("ftCo_8009F5AC");
    let data = ext::ftCo_800C0658(fp);
    if (*data).x7C_flag2() == 0 {
        return;
    }
    if (*fp).x2221_b3() != 0 {
        {
            let angle_yz = deg_to_rad((*data).x78_light_rot_yz);
            let rot_y = (*((*(*fp).gobj).hsd_obj as *mut HSD_JObj)).rotate.y;
            // `float * float / M_PI_2`: the division is in double
            let angle_x = ((deg_to_rad((*data).x74_light_rot_x) * rot_y) as f64 / core::f64::consts::FRAC_PI_2) as f32;
            let mut position = Vec { x: 0.0, y: 0.0, z: 0.0 };
            position.y = -melee_runtime::gekko::sinf(-angle_yz);
            position.z = melee_runtime::gekko::cosf(-angle_yz);
            position.x = position.z * melee_runtime::gekko::sinf(angle_x);
            position.z *= melee_runtime::gekko::cosf(angle_x);
            ext::HSD_LObjSetPosition((*fp).x588, &position);
        }
        if (*data).x7C_light_enable() != 0 {
            let mut color = node0.color;
            color.a = (*data).x50_light_color.a;
            ext::HSD_LObjSetColor((*fp).x588, color);
            ext::HSD_LObjSetFlags((*fp).x588, LOBJ_ALPHA as u32);
        } else {
            ext::HSD_LObjSetColor((*fp).x588, (*data).x50_light_color);
            ext::HSD_LObjClearFlags((*fp).x588, LOBJ_ALPHA as u32);
        }
        (*fp).set_x2221_b3(0);
    }
    if (*data).x7C_light_enable() != 0 {
        ext::HSD_LObj_803668EC(lobj0);
    } else {
        ext::HSD_LObjSetCurrentAll(lobj1);
    }
    ext::HSD_LObjAddCurrent((*fp).x588);
    ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
}

/// `ftCo_8009F75C(fp, set_alpha)`.
pub unsafe fn ftCo_8009F75C(fp: *mut Fighter, set_alpha: bool) {
    melee_core::ct!("ftCo_8009F75C");
    let data = ext::ftCo_800C0658(fp);
    if (*data).x7C_flag2() != 0 && (*data).x7C_light_enable() != 0 {
        if set_alpha {
            ext::HSD_LObjSetFlags((*fp).x588, LOBJ_ALPHA as u32);
        } else {
            ext::HSD_LObjClearFlags((*fp).x588, LOBJ_ALPHA as u32);
        }
        if set_alpha {
            ext::HSD_LObj_803668EC(lobj0);
        } else {
            ext::HSD_LObjSetCurrentAll(lobj1);
        }
        ext::HSD_LObjAddCurrent((*fp).x588);
        ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
    }
}

/// `ftCo_8009F7F8`: the plain overlay light, when the overlay is on.
pub unsafe fn ftCo_8009F7F8(fp: *mut Fighter) {
    melee_core::ct!("ftCo_8009F7F8");
    if (*ext::ftCo_800C0658(fp)).x7C_flag2() != 0 {
        ext::HSD_LObj_803668EC(lobj0);
        ext::HSD_LObjSetupInit(ext::HSD_CObjGetCurrent());
    }
}
