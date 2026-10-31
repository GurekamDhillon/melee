//! Port of src/melee/if/textdraw.c: the DevText overlay's pool of 32 text boxes (a free list and
//! the id-sorted draw list), its orthographic camera and the draw pass. Callees of unported TUs go
//! through `ext`. Checked against the C (whole TU, shipping contraction) by tests/tu_textdraw.rs.
//!
//! The boxes' `unk` / `current_color` bitfield byte is read and written through the GameCube-order
//! accessors of melee_types, so the raw byte differs from a native C build while every field value
//! is the same (the oracle compares the values).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_world/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn DrawASCII(a0: i32, a1: f32, a2: f32, a3: *mut _GXColor) -> f32;
    fn DrawRectangle(a0: f32, a1: f32, a2: f32, a3: f32, a4: *mut _GXColor);
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj = |a0: u16, a1: u8, a2: u8| unsafe { melee_runtime::tu::gobj_native::GObj_Create(a0 as _, a1 as _, a2 as _) as _ };
    fn GObj_SetupGXLink(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u8, a3: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLink(a0 as _, a1 as _, a2 as _, a3 as _); } };
    fn GObj_SetupGXLinkMax(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32) = |a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj, i32)>, a2: u32| { unsafe { let _ = melee_runtime::tu::gobj_native::GObj_SetupGXLinkMax(a0 as _, a1 as _, a2 as _); } };
    fn HSD_CObjAlloc() -> *mut HSD_CObj = || unsafe { melee_runtime::tu::cobj::HSD_CObjAlloc() as _ };
    fn HSD_CObjLoadDesc(a0: *mut HSD_CObjDesc) -> *mut HSD_CObj = |a0: *mut HSD_CObjDesc| unsafe { melee_runtime::tu::cobj::HSD_CObjLoadDesc(a0 as _) as _ };
    fn HSD_CObjSetCurrent(a0: *mut HSD_CObj) -> i32 = |a0: *mut HSD_CObj| unsafe { melee_runtime::tu::cobj::HSD_CObjSetCurrent(a0 as _) as _ };
    fn HSD_CObjSetEyePosition(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetEyePosition(a0 as _, a1 as _); } };
    fn HSD_CObjSetFar(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetFar(a0 as _, a1 as _); } };
    fn HSD_CObjSetInterest(a0: *mut HSD_CObj, a1: *mut Vec) = |a0: *mut HSD_CObj, a1: *mut Vec| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetInterest(a0 as _, a1 as _); } };
    fn HSD_CObjSetNear(a0: *mut HSD_CObj, a1: f32) = |a0: *mut HSD_CObj, a1: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetNear(a0 as _, a1 as _); } };
    fn HSD_CObjSetOrtho(a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32) = |a0: *mut HSD_CObj, a1: f32, a2: f32, a3: f32, a4: f32| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetOrtho(a0 as _, a1 as _, a2 as _, a3 as _, a4 as _); } };
    fn HSD_CObjSetProjectionType(a0: *mut HSD_CObj, a1: u32) = |a0: *mut HSD_CObj, a1: u32| { unsafe { let _ = melee_runtime::tu::cobj_world::HSD_CObjSetProjectionType(a0 as _, a1 as _); } };
    fn HSD_CObjSetRoll(a0: *mut HSD_CObj, a1: f32);
    fn HSD_CObjSetScissor(a0: *mut HSD_CObj, a1: *mut _Scissor) = |a0: *mut HSD_CObj, a1: *mut _Scissor| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetScissor(a0 as _, a1 as _); } };
    fn HSD_CObjSetViewport(a0: *mut HSD_CObj, a1: *mut _HSD_RectS16) = |a0: *mut HSD_CObj, a1: *mut _HSD_RectS16| { unsafe { let _ = melee_runtime::tu::cobj::HSD_CObjSetViewport(a0 as _, a1 as _); } };
    fn HSD_FogSet(a0: *mut HSD_Fog);
    fn HSD_GObjFree(a0: *mut HSD_GObj) = |a0: *mut HSD_GObj| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjFree(a0 as _); } };
    fn HSD_GObjObject_80390A70(a0: *mut HSD_GObj, a1: u8, a2: *mut c_void) = |a0: *mut HSD_GObj, a1: u8, a2: *mut c_void| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(a0 as _, a1 as _, a2 as _); } };
    fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) = |a0: *mut HSD_GObj, a1: i32| { unsafe { let _ = melee_runtime::tu::gobj_native::HSD_GObj_803910D8(a0 as _, a1 as _); } };
    fn Wb_RenderActive() -> i32;
    fn hsd_80391A04(a0: f32, a1: f32, a2: i32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_GObj_803910D8(a0: *mut HSD_GObj, a1: i32) { super::HSD_GObj_803910D8(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub use melee_runtime::tu::gobj_native::HSD_GObj_CameraKind;
}
// END GENERATED EXT

/// `struct DevText_Pool` (4A1FD8): 32 boxes and 0x30 unallocated bytes.
#[repr(C)]
pub struct DevText_Pool {
    pub entries: [DevText; 32],
    pub pad: [u8; 0x6B0 - 0x680],
}

/// `static HSD_WObjDesc devtext_eyepos` (3FDC20)
pub static mut devtext_eyepos: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 0.0, y: 40.241424560546875, z: 300.2409973144531 },
    robjdesc: null_mut(),
};
/// `static HSD_WObjDesc devtext_interest` (3FDC34)
pub static mut devtext_interest: HSD_WObjDesc = HSD_WObjDesc {
    class_name: null_mut(),
    pos: Vec { x: 0.0, y: 10.0, z: 0.0 },
    robjdesc: null_mut(),
};
/// `static HSD_CameraDescPerspective devtext_CObjDesc` (3FDC48)
pub static mut devtext_CObjDesc: HSD_CameraDescPerspective = HSD_CameraDescPerspective {
    class_name: null_mut(),
    flags: 0,
    projection_type: 1,
    viewport: _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 },
    scissor: _Scissor { left: 0, right: 640, top: 0, bottom: 480 },
    eyepos: &raw mut devtext_eyepos,
    interest: &raw mut devtext_interest,
    roll: 0.0,
    up_vector: null_mut(),
    nnear: 0.1,
    ffar: 32768.0,
    fov: 30.0,
    aspect: 1.3636000156402588,
};

/// `struct DevText_Pool devtext_pool` (4A1FD8)
pub static mut devtext_pool: DevText_Pool = unsafe { core::mem::zeroed() };
/// `DevText* devtext_poolhead[2]` (4D6E38); the C uses `[0]` through a macro.
pub static mut devtext_poolhead: [*mut DevText; 2] = [null_mut(); 2];
pub static mut devtext_setup_render_priority: i32 = 0;
pub static mut devtext_setup_gx_link: i32 = 0;
pub static mut devtext_setup_priority: i32 = 0;
pub static mut devtext_setup_p_link: i32 = 0;
pub static mut devtext_setup_classifier: i32 = 0;
pub static mut devtext_cobj: *mut HSD_CObj = null_mut();
pub static mut devtext_gobj: *mut HSD_GObj = null_mut();
pub static mut devtext_drawlist: *mut DevText = null_mut();

/// `int DevText_StrLen(char*)`: NULL gives 0.
pub unsafe extern "C" fn DevText_StrLen(mut str: *mut i8) -> i32 {
    melee_core::ct!("DevText_StrLen");
    if !str.is_null() {
        let mut length = 0i32;
        loop {
            let c = *str;
            str = str.add(1);
            if c == 0 {
                break;
            }
            length = length.wrapping_add(1);
        }
        return length;
    }
    0
}

/// `void DevText_NumToStr(int, char*)`: decimal digits, '-' first for a negative (`-INT_MIN` wraps).
pub unsafe extern "C" fn DevText_NumToStr(mut num: i32, mut str: *mut i8) {
    melee_core::ct!("DevText_NumToStr");
    let mut length = 0i32;
    if num < 0 {
        *str = b'-' as i8;
        num = num.wrapping_neg();
        str = str.add(1);
    }
    let mut str_end = str;
    loop {
        *str_end = (num.wrapping_rem(10)).wrapping_add(b'0' as i32) as i8;
        num = num.wrapping_div(10);
        str_end = str_end.add(1);
        length += 1;
        if num == 0 {
            break;
        }
    }
    *str_end = 0;
    str_end = str_end.sub(1);
    length >>= 1;
    let mut i = 0;
    while i != length {
        let temp = *str;
        *str = *str_end;
        *str_end = temp;
        str_end = str_end.sub(1);
        str = str.add(1);
        i += 1;
    }
}

/// `HSD_GObj* DevText_GetGObj(void)`
pub unsafe extern "C" fn DevText_GetGObj() -> *mut HSD_GObj {
    melee_core::ct!("DevText_GetGObj");
    devtext_gobj
}

/// `void DevText_InitPool(void)`: chains the 32 boxes into the free list; clears the draw list.
pub unsafe extern "C" fn DevText_InitPool() {
    melee_core::ct!("DevText_InitPool");
    let e = &raw mut devtext_pool.entries as *mut DevText;
    (*e).prev = null_mut();
    for i in 0..31usize {
        (*e.add(i + 1)).prev = e.add(i);
        (*e.add(i)).next = e.add(i + 1);
    }
    (*e.add(31)).next = null_mut();
    devtext_poolhead[0] = e;
    devtext_drawlist = null_mut();
}

/// `void DevText_Remove(DevText** ptext)`: unlinks `*ptext` (a list head) onto the free list.
pub unsafe extern "C" fn DevText_Remove(ptext: *mut *mut DevText) {
    melee_core::ct!("DevText_Remove");
    let text = *ptext;
    let next = (*text).next;
    if !next.is_null() {
        (*next).prev = (*text).prev;
    }
    let cur = *ptext;
    if !(**ptext).prev.is_null() {
        (*(**ptext).prev).next = (**ptext).next;
    } else if !(*cur).next.is_null() {
        *ptext = (**ptext).next;
    } else {
        *ptext = null_mut();
    }
    (*text).next = devtext_poolhead[0];
    (*text).prev = null_mut();
    devtext_poolhead[0] = text;
}

/// TARGET_PC `void DevText_Unlink(DevText*)`: unlinks by the box (not by a list-head address).
pub unsafe extern "C" fn DevText_Unlink(text: *mut DevText) {
    melee_core::ct!("DevText_Unlink");
    if text.is_null() {
        return;
    }
    if !(*text).next.is_null() {
        (*(*text).next).prev = (*text).prev;
    }
    if !(*text).prev.is_null() {
        (*(*text).prev).next = (*text).next;
    } else if devtext_drawlist == text {
        devtext_drawlist = (*text).next;
    }
    (*text).next = devtext_poolhead[0];
    (*text).prev = null_mut();
    devtext_poolhead[0] = text;
}

/// TARGET_PC `bool DevText_IsListed(DevText*)`
pub unsafe extern "C" fn DevText_IsListed(text: *mut DevText) -> i32 {
    melee_core::ct!("DevText_IsListed");
    let mut cur = devtext_drawlist;
    while !cur.is_null() {
        if cur == text {
            return 1;
        }
        cur = (*cur).next;
    }
    0
}

/// `void DevText_SetupCObj(void)`: the orthographic camera, built once.
pub unsafe extern "C" fn DevText_SetupCObj() {
    melee_core::ct!("DevText_SetupCObj");
    if devtext_cobj.is_null() {
        let mut viewport = _HSD_RectS16 { xmin: 0, xmax: 640, ymin: 0, ymax: 480 };
        let mut scissor = _Scissor { left: 0, right: 640, top: 0, bottom: 480 };
        let mut eyepos = Vec { x: 0.0, y: 0.0, z: 1.0 };
        let mut interest = Vec { x: 0.0, y: 0.0, z: 0.0 };
        let roll = 0.0f32;
        let near = 0.0f32;
        let far = 2.0f32;
        let top = -20.0f32;
        let bottom = 500.0f32;
        let left = -20.0f32;
        let right = 660.0f32;
        devtext_cobj = ext::HSD_CObjAlloc();
        ext::HSD_CObjSetProjectionType(devtext_cobj, 3);
        ext::HSD_CObjSetViewport(devtext_cobj, &mut viewport);
        ext::HSD_CObjSetScissor(devtext_cobj, &mut scissor);
        ext::HSD_CObjSetEyePosition(devtext_cobj, &mut eyepos);
        ext::HSD_CObjSetInterest(devtext_cobj, &mut interest);
        ext::HSD_CObjSetRoll(devtext_cobj, roll);
        ext::HSD_CObjSetNear(devtext_cobj, near);
        ext::HSD_CObjSetFar(devtext_cobj, far);
        ext::HSD_CObjSetOrtho(devtext_cobj, top, bottom, left, right);
    }
    ext::HSD_CObjSetCurrent(devtext_cobj);
}

/// `void DevText_Draw(DevText*)`: background boxes, the character grid, the blinking cursor.
/// The `a * b + c` sites are f32 fused multiply-adds (shipping contraction).
pub unsafe extern "C" fn DevText_Draw(text: *mut DevText) {
    melee_core::ct!("DevText_Draw");
    let t = &mut *text;
    if ext::Wb_RenderActive() != 0 {
        return;
    }
    ext::hsd_80391A04(t.scale_x, t.scale_y, t.line_width as i32);
    if (t.flags as i32 & DEVTEXT_FLAG_HIDEBACKGROUND) == 0 {
        let mut color = t.bg_color;
        ext::DrawRectangle(
            (t.x as i32 - 8) as f32,
            (t.y as i32 - 8) as f32,
            fmadds(t.scale_x, t.w as f32, 8.0),
            fmadds(t.scale_y, t.h as f32, 8.0),
            &mut color,
        );
        let mut color = t.bg_color;
        ext::DrawRectangle(
            (t.x as i32 - 4) as f32,
            (t.y as i32 - 4) as f32,
            fmadds(t.scale_x, t.w as f32, 4.0),
            fmadds(t.scale_y, t.h as f32, 4.0),
            &mut color,
        );
    }
    if (t.flags as i32 & DEVTEXT_FLAG_HIDETEXT) == 0 {
        let mut y = t.y as i32;
        let mut row = 0i32;
        while row < t.h as i32 {
            let mut x = t.x as i32;
            let mut col = 0i32;
            while col < t.w as i32 {
                let index = (col + t.w as i32 * row) * 2;
                let chr = *t.buf.offset(index as isize);
                let color_idx = ((*t.buf.offset(index as isize + 1) as u8) & 0xC0) >> 6;
                if chr != 0 {
                    let mut color = t.text_colors[color_idx as usize];
                    ext::DrawASCII(chr as i32, x as f32, y as f32, &mut color);
                }
                x = (x as f32 + t.scale_x) as i32;
                col += 1;
            }
            y = (y as f32 + t.scale_y) as i32;
            row += 1;
        }
        if (t.flags as i32 & DEVTEXT_FLAG_SHOWCURSOR) != 0 {
            let u = t.unk().wrapping_add(1) & 0x3F;
            t.set_unk(u);
            if u > 16 {
                t.set_unk(0);
            } else if 8 < u {
                let mut color = _GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xC0 };
                ext::DrawRectangle(
                    fmadds(t.scale_x, t.cursor_x as f32, t.x as f32),
                    fmadds(t.scale_y, t.cursor_y as f32, t.y as f32) - 4.0,
                    t.scale_x,
                    t.scale_y,
                    &mut color,
                );
            }
        }
    }
}

/// `void DevText_DrawAll(HSD_GObj*, int pass)`: the bottom-half pass draws the whole list.
pub unsafe extern "C" fn DevText_DrawAll(_gobj: *mut HSD_GObj, pass: i32) {
    melee_core::ct!("DevText_DrawAll");
    if pass as u32 == HSD_RP_BOTTOMHALF as u32 {
        let mut text = devtext_drawlist;
        ext::HSD_FogSet(null_mut());
        DevText_SetupCObj();
        while !text.is_null() {
            DevText_Draw(text);
            text = (*text).next;
        }
    }
}

/// `void DevText_CreateCObj(...)`: the overlay camera gobj.
pub unsafe extern "C" fn DevText_CreateCObj(classifier: i32, p_link: i32, gobj_priority: i32, gx_link: i32, gx_priority: u8) {
    melee_core::ct!("DevText_CreateCObj");
    let gobj = ext::GObj_Create(classifier as u16, p_link as u8, gobj_priority as u8);
    if !gobj.is_null() {
        let cobj = ext::HSD_CObjLoadDesc(&raw mut devtext_CObjDesc as *mut HSD_CObjDesc);
        if !cobj.is_null() {
            ext::HSD_GObjObject_80390A70(gobj, ext::HSD_GObj_CameraKind, cobj as *mut core::ffi::c_void);
            ext::GObj_SetupGXLinkMax(gobj, Some(ext::cb::HSD_GObj_803910D8), gx_priority as u32);
            (*gobj).gxlink_prios = 1u64 << (gx_link as u32 & 63);
        } else {
            ext::HSD_GObjFree(gobj);
        }
    }
}

/// `HSD_GObj* DevText_Setup(...)`: remembers the arguments, builds camera, pool and the draw gobj.
pub unsafe extern "C" fn DevText_Setup(
    classifier: i32,
    p_link: i32,
    priority: i32,
    gx_link: i32,
    render_priority: i32,
    camera_priority: u8,
) -> *mut HSD_GObj {
    melee_core::ct!("DevText_Setup");
    devtext_setup_classifier = classifier;
    devtext_setup_p_link = p_link;
    devtext_setup_priority = priority;
    devtext_setup_gx_link = gx_link;
    devtext_setup_render_priority = render_priority;
    devtext_cobj = null_mut();

    DevText_CreateCObj(classifier, p_link, priority, gx_link, camera_priority);
    DevText_InitPool();
    let gobj = ext::GObj_Create(
        devtext_setup_classifier as u16,
        devtext_setup_p_link as u8,
        devtext_setup_priority as u8,
    );
    if !gobj.is_null() {
        ext::GObj_SetupGXLink(
            gobj,
            Some(DevText_DrawAll),
            devtext_setup_gx_link as u8,
            (devtext_setup_render_priority & 0xFF) as u32,
        );
    }
    devtext_gobj = gobj;
    devtext_gobj
}

/// `void DevText_AddToList(DevText** list, DevText* text)`: inserts into the id-sorted list after the
/// last entry with id <= the new one (the C never sets `next->prev`).
pub unsafe extern "C" fn DevText_AddToList(list: *mut *mut DevText, text: *mut DevText) {
    melee_core::ct!("DevText_AddToList");
    if !(*list).is_null() {
        let mut next = *list;
        let mut prev: *mut DevText = null_mut();
        while !next.is_null() {
            if (*text).id >= (*next).id {
                prev = next;
                next = (*next).next;
            } else {
                break;
            }
        }
        (*text).next = next;
        if !prev.is_null() {
            (*prev).next = text;
            (*text).prev = prev;
        } else {
            (*text).prev = null_mut();
            *list = text;
        }
    } else {
        *list = text;
    }
}

/// `void DevText_Show(HSD_GObj*, DevText*)`
pub unsafe extern "C" fn DevText_Show(_gobj: *mut HSD_GObj, text: *mut DevText) {
    melee_core::ct!("DevText_Show");
    DevText_AddToList(&raw mut devtext_drawlist, text);
}
