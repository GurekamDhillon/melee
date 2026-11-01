//! lbcommand.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use mt::CmdUnion;
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::CommandInfo;
#[allow(unused_imports)]
use crate::lb_ptr::{end};

// ---- folded from lb_ptr.rs (consolidation steps 21/22) ----
/// lbcommand.c `Command_00_fp`: end of script.
pub fn Command_00_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_00_fp");
    info.anon_8.u = core::ptr::null_mut();
}

/// lbcommand.c `Command_06_fp`: return from a subroutine.
pub fn Command_06_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_06_fp");
    info.loop_count = info.loop_count.wrapping_sub(1);
    info.anon_8.u = info.event_return[info.loop_count as usize];
}

/// lbcommand.c `Command_08_fp`: next command, then wait forever (F32_MAX).
pub fn Command_08_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_08_fp");
    // SAFETY: reading the union's pointer member; the C reads the same bytes.
    let u = unsafe { info.anon_8.u };
    info.anon_8.u = u.wrapping_add(1);
    info.timer = 3.4028235e38f32;
}

// ---- folded from lb_r5.rs (consolidation steps 21/22) ----
/// lbcommand.c `Command_04_fp`: end of a loop. The C decrements `((u32*) info)[loop_count + 3]`
/// (the GC layout's "hack to match"); on the native layout that word is whatever sits at byte
/// 4*(loop_count+3), so this works on the bytes the same way. `event_return[loop_count - 1]`
/// is indexed as u32 arithmetic (loop_count 1 makes the C fault natively); the pointer is
/// truncated to s32 for the test. `&info->ptr[loop_count][0]` reads the pointer-sized word at
/// byte 8 + 8*loop_count without dereferencing it.
pub fn Command_04_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_04_fp");
    let base = info as *mut CommandInfo as *mut u8;
    // SAFETY: the C works on the record's own bytes; every offset is inside the record for the
    // loop counts the C runs with (2..=4: bytes <= 56 = size_of::<CommandInfo>()).
    unsafe {
        let lc = info.loop_count;
        let w = base.add(4 * (lc.wrapping_add(3) as usize)) as *mut u32;
        w.write_unaligned(w.read_unaligned().wrapping_sub(1));
        let lc = (base.add(core::mem::offset_of!(CommandInfo, loop_count)) as *const u32).read();
        let er = base.add(core::mem::offset_of!(CommandInfo, event_return)) as *const *mut CmdUnion;
        let e = er.add(lc.wrapping_sub(1) as usize).read();
        if e as usize as u32 as i32 != 0 {
            let p = (base.add(core::mem::offset_of!(CommandInfo, anon_8)) as *const *mut u32).add(lc as usize).read();
            info.anon_8.ptr[0] = p;
            return;
        }
        info.anon_8.u = info.anon_8.u.wrapping_add(1);
    }
    info.loop_count = info.loop_count.wrapping_sub(2);
}

/// lbcommand.c `Command_05_fp`: call a subroutine (push the return command, jump).
pub fn Command_05_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_05_fp");
    // SAFETY: `u` is the union's live member; the C reads the next command's pointer field.
    unsafe {
        let u = info.anon_8.u.wrapping_add(1);
        info.anon_8.u = u;
        info.event_return[info.loop_count as usize] = u.wrapping_add(1);
        info.loop_count = info.loop_count.wrapping_add(1);
        info.anon_8.u = u32::from_be((*u).Command_05.ptr) as usize as *mut CmdUnion;
    }
}

/// lbcommand.c `Command_07_fp`: jump.
pub fn Command_07_fp(info: &mut CommandInfo) {
    melee_core::ct!("Command_07_fp");
    // SAFETY: as Command_05_fp.
    unsafe {
        let u = info.anon_8.u.wrapping_add(1);
        info.anon_8.u = u;
        info.anon_8.u = u32::from_be((*u).Command_07.ptr) as usize as *mut CmdUnion;
    }
}
