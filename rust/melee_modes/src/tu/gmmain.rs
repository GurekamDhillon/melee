//! src/melee/gm/gmmain.c: boot-time pad setup, the debug level switch and the `main` sequence.
//! `main` (the C process entry) is not ported here: on PC the entry is the Rust `melee` bin,
//! which owns the boot order. `init_spr_unk` and `__eabi` are empty on PC (MWERKS_GEKKO is not
//! defined; `__eabi` is the GNU EABI stub) and are ported as empty functions.
//! Verification: none yet (the debug-level switch reads button state and the develop.ini file).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, dead_code)]

use melee_types::*;

/// C `static HSD_PadData gmMain_8046B108[5]` and `static HSD_PadRumbleListData gmMain_8046B1F8[12]`:
/// the pad library's storage, handed to `HSD_PadInit`. Zeroed here; the pad layer owns them.
pub static mut gmMain_8046B108: [HSD_PadData; 5] = unsafe { core::mem::zeroed() };
pub static mut gmMain_8046B1F8: [HSD_PadRumbleListData; 12] = unsafe { core::mem::zeroed() };
/// C `static bool gmMain_804D6594;` (set false in `main`; not read on this path).
static mut gmMain_804D6594: bool = false;

/// C `gmMain_8015FD24`: the pad library clamp setup, run by `lb_80019AAC`.
pub unsafe fn gmMain_8015FD24() {
    ext::PADSetSpec(5);
    ext::HSD_PadInit(
        5,
        core::ptr::addr_of_mut!(gmMain_8046B108) as *mut HSD_PadData,
        12,
        core::ptr::addr_of_mut!(gmMain_8046B1F8) as *mut HSD_PadRumbleListData,
    );
    let l = ext::HSD_PadLibData();
    (*l).clamp_stickType = 0;
    (*l).clamp_stickShift = 1;
    (*l).clamp_stickMax = 80;
    (*l).clamp_stickMin = 0;
    (*l).scale_stick = 80;
    (*l).clamp_analogLRShift = 1;
    (*l).clamp_analogLRMax = 140;
    (*l).clamp_analogLRMin = 0;
    (*l).scale_analogLR = 140;
}

/// C `gmMain_8015FDA0`: the post-retrace callback (empty).
pub unsafe fn gmMain_8015FDA0(_arg: u32) {}

/// C `gmMain_8015FDA4`: set the debug level from the develop.ini file and the launch buttons.
pub unsafe fn gmMain_8015FDA4() {
    if ext::DVDConvertPathToEntrynum("/develop.ini") != -1 {
        ext::set_db_804D6B20(true);
        if ext::db_gameLaunchButtonState() & HSD_PAD_X as u32 != 0 {
            let new_level = match ext::DbLevel() {
                x if x == DbLKind_NoDebugRom as i32 => DbLKind_DebugRom as i32,
                x if x == DbLKind_DebugDevelop as i32 => DbLKind_Develop as i32,
                x if x == DbLKind_DebugRom as i32 => DbLKind_NoDebugRom as i32,
                x if x == DbLKind_Develop as i32 => DbLKind_DebugDevelop as i32,
                _ => DbLKind_DebugDevelop as i32,
            };
            ext::set_DbLevel(new_level);
        } else if ext::db_gameLaunchButtonState() & HSD_PAD_Y as u32 != 0 {
            let level = ext::DbLevel();
            let new_level = match level {
                x if x == DbLKind_NoDebugRom as i32 => DbLKind_DebugRom as i32,
                x if x == DbLKind_DebugDevelop as i32 => DbLKind_Develop as i32,
                _ => level,
            };
            ext::set_DbLevel(new_level);
            ext::set_db_804D6B20(false);
        }
    } else {
        ext::HSD_ASSERT(210, ext::DbLevel() == DbLKind_NoDebugRom as i32);
        ext::set_DbLevel(DbLKind_Master as i32);
    }
}

/// C `init_spr_unk`: the MWERKS_GEKKO SPR setup. Empty on PC.
pub unsafe fn init_spr_unk() {}

/// C `__eabi`: the GNU EABI environment stub. Empty.
pub unsafe fn __eabi() {}

/// Callees in TUs that are not ported yet and globals owned by other TUs. Each is a settable
/// hook; unset it panics.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => unsafe { f($($a),*) },
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    hook!(PADSETSPEC, PADSetSpec, (spec: i32));
    pub static mut HSD_PADINIT: Option<fn(i32, *mut HSD_PadData, i32, *mut HSD_PadRumbleListData) > = None;
            #[allow(non_snake_case)]
            pub fn HSD_PadInit(n: i32, data: *mut HSD_PadData, rumble_n: i32, rumble: *mut HSD_PadRumbleListData)  {
                match unsafe { HSD_PADINIT } {
                    Some(f) => unsafe { f(n, data, rumble_n, rumble) },
                    None => { unsafe { let _ = melee_runtime::tu::controller::HSD_PadInit(n as _, data as _, rumble_n as _, rumble as _); } return; },
                }
            }
    hook!(HSD_PADLIBDATA, HSD_PadLibData, () -> *mut PadLibData);
    hook!(DVDCONVERTPATHTOENTRYNUM, DVDConvertPathToEntrynum, (path: &str) -> i32);
    hook!(SET_DB_804D6B20, set_db_804D6B20, (v: bool));
    hook!(DB_GAMELAUNCHBUTTONSTATE, db_gameLaunchButtonState, () -> u32);
    hook!(DBLEVEL, DbLevel, () -> i32);
    hook!(SET_DBLEVEL, set_DbLevel, (v: i32));
    hook!(HSD_ASSERT_HOOK, HSD_ASSERT, (line: i32, cond: bool));
}
