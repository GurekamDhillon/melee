//! Round 5: lbcommand.c script commands over nested pointers that round 3 skipped, verified
//! bit for bit against the C by `tests/oracle_r5.rs`. TODO(group1): lb leftovers; move into
//! melee_runtime if group 1 takes lbcommand.c.
use melee_types as mt;
use mt::{CmdUnion, CommandInfo};

/// lbcommand.c `Command_04`: end of a loop. The C decrements `((u32*) info)[loop_count + 3]`
/// (the GC layout's "hack to match"); on the native layout that word is whatever sits at byte
/// 4*(loop_count+3), so this works on the bytes the same way. `event_return[loop_count - 1]`
/// is indexed as u32 arithmetic (loop_count 1 makes the C fault natively); the pointer is
/// truncated to s32 for the test. `&info->ptr[loop_count][0]` reads the pointer-sized word at
/// byte 8 + 8*loop_count without dereferencing it.
pub fn Command_04(info: &mut CommandInfo) {
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

/// lbcommand.c `Command_05`: call a subroutine (push the return command, jump).
pub fn Command_05(info: &mut CommandInfo) {
    // SAFETY: `u` is the union's live member; the C reads the next command's pointer field.
    unsafe {
        let u = info.anon_8.u.wrapping_add(1);
        info.anon_8.u = u;
        info.event_return[info.loop_count as usize] = u.wrapping_add(1);
        info.loop_count = info.loop_count.wrapping_add(1);
        info.anon_8.u = (*u).Command_05.ptr;
    }
}

/// lbcommand.c `Command_07`: jump.
pub fn Command_07(info: &mut CommandInfo) {
    // SAFETY: as Command_05.
    unsafe {
        let u = info.anon_8.u.wrapping_add(1);
        info.anon_8.u = u;
        info.anon_8.u = (*u).Command_07.ptr;
    }
}
