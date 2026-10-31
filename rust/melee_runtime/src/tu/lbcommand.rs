//! Port of src/melee/lb/lbcommand.c (the ten animation-command handlers shared by fighters, items
//! and effects, and `Command_Execute`'s dispatch table). Verified against the C (whole TU) by
//! tests/tu_lbcommand.rs (`python rust/melee_runtime/gen/mtu/run_tu.py lbcommand`).
//!
//! `NEXT_CMD` steps `cmd->u` by one `CmdUnion` (4 bytes natively, like the stream's words in the raw
//! archive image; the jump targets of Command_05/07 are 32-bit guest addresses stored big-endian). The loop counter that `Command_03` parks in an `event_return` slot is a 32-bit
//! value on the console (`(union CmdUnion*) value`), so `Command_04` decrements the low 32 bits of
//! that slot, which is what the C's `((u32*) info)[loop_count + 3] -= 1` does at 4-byte pointers.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn lbBgFlash_80021C48(a0: u32, a1: u32);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const F32_MAX: f32 = 3.4028235e38;

#[inline(always)]
unsafe fn next_cmd(info: *mut CommandInfo) {
    (*info).anon_8.u = (*info).anon_8.u.add(1);
}

/// Reset
pub unsafe extern "C" fn Command_00(info: *mut CommandInfo) {
    melee_core::ct!("Command_00");
    (*info).anon_8.u = core::ptr::null_mut();
}

/// SynchronousTimer
pub unsafe extern "C" fn Command_01(info: *mut CommandInfo) {
    melee_core::ct!("Command_01");
    (*info).timer += (*(*info).anon_8.u).Command_00.value() as f32;
    next_cmd(info);
}

/// AsynchronousTimer
pub unsafe extern "C" fn Command_02(info: *mut CommandInfo) {
    melee_core::ct!("Command_02");
    (*info).timer = (*(*info).anon_8.u).Command_02.value() as f32 - (*info).frame_count;
    next_cmd(info);
}

/// SetLoop
pub unsafe extern "C" fn Command_03(info: *mut CommandInfo) {
    melee_core::ct!("Command_03");
    let i = &mut *info;
    let lc = i.loop_count as usize;
    i.event_return[lc] = i.anon_8.u.add(1);
    i.event_return[lc + 1] = (*i.anon_8.u).Command_03.value() as usize as *mut CmdUnion;
    i.loop_count += 2;
    next_cmd(info);
}

/// Execute Loop
pub unsafe extern "C" fn Command_04(info: *mut CommandInfo) {
    melee_core::ct!("Command_04");
    let i = &mut *info;
    let lc = i.loop_count as usize;
    let slot = (&raw mut i.event_return[lc - 1]) as *mut u32;
    *slot = (*slot).wrapping_sub(1);
    if (i.event_return[lc - 1] as usize as u32 as i32) != 0 {
        i.anon_8.u = i.event_return[lc - 2];
        return;
    }
    next_cmd(info);
    (*info).loop_count = (*info).loop_count.wrapping_sub(2);
}

/// Subroutine
pub unsafe extern "C" fn Command_05(info: *mut CommandInfo) {
    melee_core::ct!("Command_05");
    next_cmd(info);
    let i = &mut *info;
    let lc = i.loop_count as usize;
    i.event_return[lc] = i.anon_8.u.add(1);
    i.loop_count += 1;
    i.anon_8.u = u32::from_be((*i.anon_8.u).Command_05.ptr) as usize as *mut CmdUnion; // a 32-bit guest address stored big-endian in the raw stream
}

/// Return
pub unsafe extern "C" fn Command_06(info: *mut CommandInfo) {
    melee_core::ct!("Command_06");
    let i = &mut *info;
    i.loop_count = i.loop_count.wrapping_sub(1);
    i.anon_8.u = i.event_return[i.loop_count as usize];
}

/// Goto
pub unsafe extern "C" fn Command_07(info: *mut CommandInfo) {
    melee_core::ct!("Command_07");
    next_cmd(info);
    (*info).anon_8.u = u32::from_be((*(*info).anon_8.u).Command_07.ptr) as usize as *mut CmdUnion;
}

/// SetTimerAnimation
pub unsafe extern "C" fn Command_08(info: *mut CommandInfo) {
    melee_core::ct!("Command_08");
    next_cmd(info);
    (*info).timer = F32_MAX;
}

pub unsafe extern "C" fn Command_09(info: *mut CommandInfo) {
    melee_core::ct!("Command_09");
    let w = &(*(*info).anon_8.u).Command_09;
    ext::lbBgFlash_80021C48(w.param_1(), w.param_2());
    next_cmd(info);
}

pub static lbCommand_803B9840: [Option<unsafe extern "C" fn(*mut CommandInfo)>; 16] = [
    Some(Command_00),
    Some(Command_01),
    Some(Command_02),
    Some(Command_03),
    Some(Command_04),
    Some(Command_05),
    Some(Command_06),
    Some(Command_07),
    Some(Command_08),
    Some(Command_09),
    None,
    None,
    None,
    None,
    None,
    None,
];

pub unsafe extern "C" fn Command_Execute(info: *mut CommandInfo, command: u32) -> bool {
    melee_core::ct!("Command_Execute");
    if command < 10 {
        (lbCommand_803B9840[command as usize].unwrap())(info);
        return true;
    }
    false
}
