//! Native port of src/melee/vi/vi.c: the character-animation file table and the helpers the
//! vi*.c "visual" scenes (opening movie vignettes) share. Every function of the TU is here.
//! Callees in other TUs go through `ext` (an unset hook panics "TODO: <name> is not ported").
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]
use core::ffi::c_void;
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
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
    fn Camera_800310A0(a0: u8);
    fn Camera_800313E0(a0: *mut HSD_GObj, a1: u64);
    fn HSD_GObj_80390ED0(a0: *mut HSD_GObj, a1: u32);
    fn gm_801678F8(a0: i32, a1: i32, a2: i32);
    fn gm_8017E430() -> u8;
    fn gm_8017E440() -> u8;
    fn gm_801A4B60();
    fn gm_GetButtonsTriggered(a0: u8) -> u64;
    fn lbAudioAx_80023694() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023694() };
    fn lbAudioAx_800236DC() -> i32 => unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236DC() };
    fn lb_800145F4();
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// `PAD_BUTTON_START` (dolphin/pad.h).
const PAD_BUTTON_START: u64 = 0x1000;

macro_rules! cs {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *mut i8
    };
}

/// `char* vi_animation_table[26]`
pub static mut vi_animation_table: [*mut i8; 26] = [
    cs!("PlCaDViWaitAJ.dat"), cs!("PlDkDViWaitAJ.dat"), cs!("PlFxDViWaitAJ.dat"),
    cs!("PlGwDViWaitAJ.dat"), cs!("PlKbDViWaitAJ.dat"), cs!("PlKpDViWaitAJ.dat"),
    cs!("PlLkDViWaitAJ.dat"), cs!("PlLgDViWaitAJ.dat"), cs!("PlMrDViWaitAJ.dat"),
    cs!("PlMsDViWaitAj.dat"), cs!("PlMtDViWaitAJ.dat"), cs!("PlNsDViWaitAJ.dat"),
    cs!("PlPeDViWaitAJ.dat"), cs!("PlPkDViWaitAJ.dat"), cs!("PlPpDViWaitAJ.dat"),
    cs!("PlPrDViWaitAJ.dat"), cs!("PlSsDViWaitAJ.dat"), cs!("PlYsDViWaitAJ.dat"),
    cs!("PlZdDViWaitAJ.dat"), cs!("PlSkDViWaitAJ.dat"), cs!("PlFcDViWaitAJ.dat"),
    cs!("PlClDViWaitAJ.dat"), cs!("PlDrDViWaitAJ.dat"), cs!("PlFeDViWaitAJ.dat"),
    cs!("PlPcDViWaitAJ.dat"), cs!("PlGnDViWaitAJ.dat"),
];

/// `char* viGetCharAnimByIndex(s32)`
pub unsafe fn viGetCharAnimByIndex(index: i32) -> *mut i8 {
    melee_core::ct!("viGetCharAnimByIndex");
    vi_animation_table[index as usize]
}

/// `void vi_8031C9B4(s32, s32)`
pub unsafe fn vi_8031C9B4(arg0: i32, arg1: i32) {
    melee_core::ct!("vi_8031C9B4");
    let r3 = ext::gm_8017E440();
    if r3 != 4 {
        ext::gm_801678F8(r3 as i32, arg0, arg1);
    }
}

/// `void vi_8031CA04(HSD_GObj*)`
pub unsafe fn vi_8031CA04(gobj: *mut HSD_GObj) {
    melee_core::ct!("vi_8031CA04");
    ext::Camera_800310A0(2);
    (*gobj).gxlink_prios = 9;
    ext::HSD_GObj_80390ED0(gobj, 7);
    ext::Camera_800310A0(1);
    (*gobj).gxlink_prios = 8;
    ext::HSD_GObj_80390ED0(gobj, 7);
    ext::Camera_800310A0(0);
    (*gobj).gxlink_prios = 8;
    ext::HSD_GObj_80390ED0(gobj, 7);
    ext::Camera_800313E0(gobj, 1);
}

/// `void vi_8031CAAC(void)`
pub unsafe fn vi_8031CAAC() {
    melee_core::ct!("vi_8031CAAC");
    let temp_ret: u64 = ext::gm_GetButtonsTriggered(ext::gm_8017E430());
    if (temp_ret & PAD_BUTTON_START) != 0 {
        ext::lbAudioAx_800236DC();
        ext::lbAudioAx_80023694();
        ext::lb_800145F4();
        ext::gm_801A4B60();
    }
}
