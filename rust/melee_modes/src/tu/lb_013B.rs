//! Port of src/melee/lb/lb_013B.c (ColorOverlay command interpreter: the color / light opcodes
//! 0xA..0x14 behind `lb_80014258`, the overlay reset / start, and the LbRb.dat rumble table).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_013B.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_runtime::cabi::f2u8;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn Command_Execute(a0: *mut CommandInfo, a1: u32) -> i32 = crate::tu::adapt::Command_Execute__d5bf79;
    fn HSD_PadRumbleAdd(a0: u8, a1: i32, a2: i32, a3: i32, a4: *mut c_void) -> i32 = crate::tu::adapt::HSD_PadRumbleAdd__f8a1ce;
    fn HSD_PadRumbleOn(a0: u8) = crate::tu::adapt::HSD_PadRumbleOn__3bcc58;
    fn HSD_PadRumbleRemove(a0: u8) = crate::tu::adapt::HSD_PadRumbleRemove__adbf79;
    fn lbArchive_80017040(a0: *mut *mut HSD_Archive, a1: *const i8, a2: *mut c_void, a3: *const i8, a4: i32) -> i32;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

pub use crate::lb_ptr::lb_80013BB0;

/// `lb_804D63C0` (static): the loaded `lbRumbleData` table.
pub static mut lb_804D63C0: *mut Fighter_804D653C_t = core::ptr::null_mut();

/// `lb_803BA248_fn`
pub type lb_803BA248_fn = unsafe fn(*mut ColorOverlay) -> bool;

/// 013BB8
pub unsafe fn lb_80013BB8(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013BB8");
    (*arg0).x0_timer = (*arg0).x0_timer.wrapping_add((*(*arg0).x8_ptr1).unk.timer() as i32);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 013BE4
pub unsafe fn lb_80013BE4(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013BE4");
    (*arg0).set_x7C_flag2(0);
    (*arg0).set_x7C_color_enable(0);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// Copies the GXColor at `x8_ptr1` into the light color and its float shadows, and clears the
/// light blend rates (the shared middle of lb_80013C18 / lb_80013D68).
#[inline(always)]
unsafe fn load_light_color(arg0: *mut ColorOverlay) {
    let c = (*(*arg0).x8_ptr1).light_color;
    (*arg0).x50_light_color.r = c.r;
    (*arg0).x50_light_color.g = c.g;
    (*arg0).x50_light_color.b = c.b;
    (*arg0).x50_light_color.a = c.a;
    (*arg0).x54_light_red = (*arg0).x50_light_color.r as f32;
    (*arg0).x58_light_green = (*arg0).x50_light_color.g as f32;
    (*arg0).x5C_light_blue = (*arg0).x50_light_color.b as f32;
    (*arg0).x60_light_alpha = (*arg0).x50_light_color.a as f32;
    (*arg0).x70_lightblend_alpha = 0.0;
    (*arg0).x6C_lightblend_blue = 0.0;
    (*arg0).x68_lightblend_green = 0.0;
    (*arg0).x64_lightblend_red = 0.0;
}

/// 013C18
pub unsafe fn lb_80013C18(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013C18");
    let rot2 = (*(*arg0).x8_ptr1).light_rot2;
    (*arg0).set_x7C_light_enable(rot2.light_enable() as u8);
    (*arg0).x74_light_rot_x = rot2.x() as f32;
    (*arg0).x78_light_rot_yz = rot2.yz() as f32;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    load_light_color(arg0);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    (*arg0).set_x7C_flag2(1);
    false
}

/// 013D68
pub unsafe fn lb_80013D68(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013D68");
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    load_light_color(arg0);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 013E3C
pub unsafe fn lb_80013E3C(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013E3C");
    let f = (*(*arg0).x8_ptr1).unk.timer() as i32 as f32;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    let c = (*(*arg0).x8_ptr1).light_color;
    (*arg0).x64_lightblend_red = ((0.5f32 + c.r as f32) - (*arg0).x50_light_color.r as f32) / f;
    (*arg0).x68_lightblend_green = ((0.5f32 + c.g as f32) - (*arg0).x50_light_color.g as f32) / f;
    (*arg0).x6C_lightblend_blue = ((0.5f32 + c.b as f32) - (*arg0).x50_light_color.b as f32) / f;
    (*arg0).x70_lightblend_alpha = ((0.5f32 + c.a as f32) - (*arg0).x50_light_color.a as f32) / f;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 013F78
pub unsafe fn lb_80013F78(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013F78");
    let rot1 = (*(*arg0).x8_ptr1).light_rot1;
    (*arg0).x74_light_rot_x = rot1.x() as f32;
    (*arg0).x78_light_rot_yz = rot1.yz() as f32;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 013FF0
pub unsafe fn lb_80013FF0(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80013FF0");
    (*arg0).set_x7C_flag2(0);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 014014
pub unsafe fn lb_80014014(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80014014");
    (*arg0).set_x7C_color_enable(1);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    let c = (*(*arg0).x8_ptr1).light_color;
    (*arg0).x2C_hex.r = c.r;
    (*arg0).x2C_hex.g = c.g;
    (*arg0).x2C_hex.b = c.b;
    (*arg0).x2C_hex.a = c.a;
    (*arg0).x30_color_red = (*arg0).x2C_hex.r as f32;
    (*arg0).x34_color_green = (*arg0).x2C_hex.g as f32;
    (*arg0).x38_color_blue = (*arg0).x2C_hex.b as f32;
    (*arg0).x3C_color_alpha = (*arg0).x2C_hex.a as f32;
    (*arg0).x4C_colorblend_alpha = 0.0;
    (*arg0).x48_colorblend_blue = 0.0;
    (*arg0).x44_colorblend_green = 0.0;
    (*arg0).x40_colorblend_red = 0.0;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 0140F8
pub unsafe fn lb_800140F8(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_800140F8");
    let f = (*(*arg0).x8_ptr1).unk.timer() as i32 as f32;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    let c = (*(*arg0).x8_ptr1).light_color;
    (*arg0).x40_colorblend_red = ((0.5f32 + c.r as f32) - (*arg0).x2C_hex.r as f32) / f;
    (*arg0).x44_colorblend_green = ((0.5f32 + c.g as f32) - (*arg0).x2C_hex.g as f32) / f;
    (*arg0).x48_colorblend_blue = ((0.5f32 + c.b as f32) - (*arg0).x2C_hex.b as f32) / f;
    (*arg0).x4C_colorblend_alpha = ((0.5f32 + c.a as f32) - (*arg0).x2C_hex.a as f32) / f;
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 014234
pub unsafe fn lb_80014234(arg0: *mut ColorOverlay) -> bool {
    melee_core::ct!("lb_80014234");
    (*arg0).set_x7C_color_enable(0);
    (*arg0).x8_ptr1 = (*arg0).x8_ptr1.add(1);
    false
}

/// 3BA248: the handlers of opcodes 0xA..0x14 (entries 11..21 are NULL).
pub static lb_803BA248: [Option<lb_803BA248_fn>; 22] = [
    Some(lb_80013BB0), Some(lb_80013BB8), Some(lb_80013BE4), Some(lb_80013C18), Some(lb_80013D68),
    Some(lb_80013E3C), Some(lb_80013F78), Some(lb_80013FF0), Some(lb_80014014), Some(lb_800140F8),
    Some(lb_80014234), None, None, None, None, None, None, None, None, None, None, None,
];

/// 014258: runs the overlay's command stream until a command ends the frame, then advances the
/// color / light blends and the priority timer. `true` = the animation is over.
pub unsafe fn lb_80014258(gobj: *mut Fighter_GObj, arg1: *mut c_void, cmd: FtCmd2) -> bool {
    melee_core::ct!("lb_80014258");
    let co = arg1 as *mut ColorOverlay;

    if !(*co).x8_ptr1.is_null() {
        let timer = (*co).x0_timer;
        if timer != 0 {
            (*co).x0_timer = timer.wrapping_sub(1);
        }
    }

    while !(*co).x8_ptr1.is_null() && (*co).x0_timer == 0 {
        let opcode = (*(*co).x8_ptr1).unk.unk();
        if ext::Command_Execute(co as *mut CommandInfo, opcode) == 0 {
            if opcode < 0x15 {
                let idx = opcode.wrapping_sub(0xA);
                if (lb_803BA248[idx as usize].unwrap())(co) {
                    return true;
                }
            } else {
                (cmd.unwrap())(gobj, co as *mut CommandInfo, opcode as i32);
            }
        }
    }

    if (*co).x7C_color_enable() != 0 {
        (*co).x30_color_red += (*co).x40_colorblend_red;
        (*co).x34_color_green += (*co).x44_colorblend_green;
        (*co).x38_color_blue += (*co).x48_colorblend_blue;
        (*co).x3C_color_alpha += (*co).x4C_colorblend_alpha;
        (*co).x2C_hex.r = f2u8((*co).x30_color_red as f64);
        (*co).x2C_hex.g = f2u8((*co).x34_color_green as f64);
        (*co).x2C_hex.b = f2u8((*co).x38_color_blue as f64);
        (*co).x2C_hex.a = f2u8((*co).x3C_color_alpha as f64);
    }
    if (*co).x7C_flag2() != 0 {
        (*co).x54_light_red += (*co).x64_lightblend_red;
        (*co).x58_light_green += (*co).x68_lightblend_green;
        (*co).x5C_light_blue += (*co).x6C_lightblend_blue;
        (*co).x60_light_alpha += (*co).x70_lightblend_alpha;
        (*co).x50_light_color.r = f2u8((*co).x54_light_red as f64);
        (*co).x50_light_color.g = f2u8((*co).x58_light_green as f64);
        (*co).x50_light_color.b = f2u8((*co).x5C_light_blue as f64);
        (*co).x50_light_color.a = f2u8((*co).x60_light_alpha as f64);
    }
    let fc = (*co).x4_pri;
    if fc != 0 {
        (*co).x4_pri = fc.wrapping_sub(1);
        if (*co).x4_pri == 0 {
            return true;
        }
    }
    false
}

/// 014498
pub unsafe fn lb_80014498(arg0: *mut ColorOverlay) {
    melee_core::ct!("lb_80014498");
    (*arg0).x8_ptr1 = core::ptr::null_mut();
    (*arg0).x4_pri = 0;
    (*arg0).x28_colanim.ptr = core::ptr::null_mut();
    (*arg0).set_x7C_flag2(0);
    (*arg0).set_x7C_color_enable(0);
}

/// 0144C8
pub unsafe fn lb_800144C8(arg0: *mut ColorOverlay, arg1: *mut Fighter_804D653C_t, arg2: i32, arg3: i32) -> bool {
    melee_core::ct!("lb_800144C8");
    if (*arg1.offset((*arg0).x28_colanim.i as isize)).unk4 <= (*arg1.offset(arg2 as isize)).unk4 {
        (*arg0).x28_colanim.i = arg2;
        (*arg0).x4_pri = arg3;
        (*arg0).x8_ptr1 = (*arg1.offset(arg2 as isize)).unk as *mut ColorOverlay_x8_t;
        (*arg0).x0_timer = 0;
        (*arg0).xC_loop = 0;
        (*arg0).set_x7C_flag2(0);
        (*arg0).set_x7C_color_enable(0);
        return true;
    }
    false
}

/// 014534
pub unsafe fn lb_80014534() {
    melee_core::ct!("lb_80014534");
    ext::lbArchive_80017040(core::ptr::null_mut(), c"LbRb.dat".as_ptr(), (&raw mut lb_804D63C0) as *mut c_void,
                            c"lbRumbleData".as_ptr(), 0);
}

/// 014574
pub unsafe fn lb_80014574(arg0: u8, arg1: i32, arg2: i32, arg3: i32) {
    melee_core::ct!("lb_80014574");
    let e = lb_804D63C0.offset(arg2 as isize);
    ext::HSD_PadRumbleAdd(arg0, arg1, if arg3 != 0 { arg3 } else { -2 }, (*e).unk4 as i32, (*e).unk);
}

/// 0145C0
pub unsafe fn lb_800145C0(slot: u8) {
    melee_core::ct!("lb_800145C0");
    ext::HSD_PadRumbleRemove(slot);
    ext::HSD_PadRumbleOn(slot);
}

/// 0145F4
pub unsafe fn lb_800145F4() {
    melee_core::ct!("lb_800145F4");
    for i in 0..4 {
        lb_800145C0(i as u8);
    }
}
