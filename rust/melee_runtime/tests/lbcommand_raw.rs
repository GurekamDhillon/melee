//! lbcommand.c over a raw command stream: 4-byte big-endian words in memory below 4 GiB (as the archive host lays
//! fighter / item / effect scripts out), jump targets as 32-bit guest addresses. `tu_lbcommand.rs` (the C oracle) walks
//! native 8-byte `CmdUnion` arrays and is ignored: the C build's pointer-bearing union cannot have the raw stream's layout.
#![cfg(windows)]
use core::ffi::c_void;
use melee_runtime::tu::lbcommand::Command_Execute;
use melee_types::*;

extern "system" {
    fn VirtualAlloc(addr: *mut c_void, size: usize, ty: u32, prot: u32) -> *mut c_void;
}

/// A stream of `words.len()` words below 4 GiB; `words` may name positions of the stream itself through `Addr`.
enum W {
    Op(u32, u32),
    Addr(usize),
}

fn stream(words: &[W]) -> *mut CmdUnion {
    let mut hint = 0x8100_0000usize;
    let base = loop {
        let p = unsafe { VirtualAlloc(hint as *mut c_void, 4096, 0x3000, 0x04) };
        if !p.is_null() {
            break p as *mut u32;
        }
        hint += 0x0100_0000;
        assert!(hint < 0xF000_0000, "no low memory");
    };
    for (i, w) in words.iter().enumerate() {
        let v = match *w {
            W::Op(op, val) => (op << 26) | (val & 0x03FF_FFFF),
            W::Addr(k) => base as usize as u32 + 4 * k as u32,
        };
        unsafe { base.add(i).write(v.to_be()) };
    }
    base as *mut CmdUnion
}

fn info(u: *mut CmdUnion) -> CommandInfo {
    let mut i = CommandInfo::zeroed();
    i.anon_8.u = u;
    i
}

/// The opcode of the word `u` points at (the top six bits of the big-endian word).
unsafe fn op(u: *mut CmdUnion) -> u32 {
    u32::from_be(*(u as *const u32)) >> 26
}

/// Run until the stream resets (`u == NULL`); returns the number of commands executed.
unsafe fn run(i: &mut CommandInfo) -> u32 {
    let mut n = 0;
    while !i.anon_8.u.is_null() {
        let o = op(i.anon_8.u);
        assert!(Command_Execute(i, o));
        n += 1;
        assert!(n < 100, "runaway stream");
    }
    n
}

#[test]
fn words_are_four_bytes() {
    assert_eq!(core::mem::size_of::<CmdUnion>(), 4);
    assert_eq!(core::mem::align_of::<CmdUnion>(), 4);
}

#[test]
fn timers_and_reset() {
    unsafe {
        // SynchronousTimer 5, AsynchronousTimer 9 (timer = value - frame_count), Reset
        let s = stream(&[W::Op(1, 5), W::Op(2, 9), W::Op(0, 0)]);
        let mut i = info(s);
        i.frame_count = 4.0;
        assert_eq!(run(&mut i), 3);
        assert_eq!(i.timer, 5.0);
    }
}

#[test]
fn loop_runs_its_body_count_times() {
    unsafe {
        // SetLoop(3) { SynchronousTimer 1 } ExecuteLoop; Reset
        let s = stream(&[W::Op(3, 3), W::Op(1, 1), W::Op(4, 0), W::Op(0, 0)]);
        let mut i = info(s);
        run(&mut i);
        assert_eq!(i.timer, 3.0);
        assert_eq!(i.loop_count, 0);
    }
}

#[test]
fn subroutine_returns_to_the_word_after_its_address() {
    unsafe {
        // 0: Subroutine -> 5; 1: address; 2: SynchronousTimer 2; 3: Reset; 4: pad; 5: SynchronousTimer 7; 6: Return
        let s = stream(&[W::Op(5, 0), W::Addr(5), W::Op(1, 2), W::Op(0, 0), W::Op(0, 0), W::Op(1, 7), W::Op(6, 0)]);
        let mut i = info(s);
        run(&mut i);
        assert_eq!(i.timer, 9.0);
        assert_eq!(i.loop_count, 0);
    }
}

#[test]
fn goto_jumps_over_words() {
    unsafe {
        // 0: Goto -> 4; 1: address; 2: SynchronousTimer 100 (skipped); 3: Reset; 4: SynchronousTimer 3; 5: Reset
        let s = stream(&[W::Op(7, 0), W::Addr(4), W::Op(1, 100), W::Op(0, 0), W::Op(1, 3), W::Op(0, 0)]);
        let mut i = info(s);
        run(&mut i);
        assert_eq!(i.timer, 3.0);
    }
}

#[test]
fn set_timer_animation_and_dispatch_limit() {
    unsafe {
        let s = stream(&[W::Op(8, 0), W::Op(0, 0)]);
        let mut i = info(s);
        assert!(Command_Execute(&mut i, 8));
        assert_eq!(i.timer, f32::MAX);
        assert!(!Command_Execute(&mut i, 10));
    }
}
