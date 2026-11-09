//! tu::lbheap + tu::lbmemory vs the C (state oracle): gen/turef/tu_lbheap_ref.c is the game's lbheap.c and lbmemory.c
//! compiled together natively (dolphin ar/os, the baselib heap and DevCom stubbed and logged; DevCom requests and
//! OSAlarms are queued and completed by `pump`). Every step prints the result, the callee log and a hash of both
//! modules' whole state; a hash of main RAM follows every 25 steps. The expected text is tests/data/tu_lbheap.txt
//! (python rust/melee_runtime/gen/turef/tu_shard833.py tu_lbheap). This file is the Rust twin of the driver's `main`.
//!
//! Functions compared (every step compares both modules' whole state, the callee log and the results):
//!   called by the driver: lbMemory_8001564C, lbHeap_80015F3C, lbHeap_800158D0, lbHeap_800158E8, lbHeap_80015900,
//!   lbHeap_80015BB8, lbHeap_80015BD0, lbHeap_80015CA8, lbHeap_80015D6C, lbHeap_80015DF8, lbHeap_Capacity,
//!   lbHeap_Free, lbHeap_IdOfHandle, lbHeap_StageSlotTryAlloc, lbHeap_StageSlotDump;
//!   reached through them: lbMemory_80014E24 (heap create), lbMemory_80014EEC (destroy), lbMemory_80014F7C
//!   (free size), lbMemory_80014FC8 (alloc), lbMemFreeToHeap, lbMemory_8001529C + lbMemory_80015320 + fn_80015184
//!   (compaction by DevCom / timer copy), lbMemory_800154BC (ARAM bounds), lbMemory_800154D4, lbMemory_800155A4.
#![allow(static_mut_refs, non_snake_case)]
#[macro_use]
#[path = "lbtest/mod.rs"]
mod common_lb;
use common_lb::*;
use core::ffi::{c_char, c_void};
use melee_runtime::tu::lbheap as lh;
use melee_runtime::tu::lbmemory as lm;
use melee_types::*;

struct Dc {
    on: bool,
    cb: HSD_DevComCallback,
    args: usize,
}
struct Al {
    on: bool,
    a: *mut OSAlarm,
    h: OSAlarmHandler,
}
static mut DC: Dc = Dc { on: false, cb: None, args: 0 };
static mut AL: Al = Al { on: false, a: core::ptr::null_mut(), h: None };
static mut G_DONE: i32 = 0;
static mut G_ASSERTED: bool = false;
static mut G_FREE_HEAP: i32 = -1;

unsafe extern "C" fn done_cb(v: u32) {
    lg(&format!("done({});", v));
    G_DONE += 1;
}

mod gen_stubs {
    use super::*;
    include!("lbtest/lbheap_stubs.rs");
}

fn h_HSD_DevComRequest(file: i32, src: usize, dest: usize, size: usize, ty: i32, pri: i32, cb: HSD_DevComCallback, args: usize) -> i32 {
    lg(&format!(
        "HSD_DevComRequest({},{},{},{},{},{},{},{});",
        file,
        pidu(src as u64),
        pidu(dest as u64),
        size as u32,
        ty,
        pri,
        pidf(cb.map_or(0usize, |f| f as usize)),
        pidu(args as u64)
    ));
    unsafe {
        DC.on = true;
        DC.cb = cb;
        DC.args = args;
    }
    0
}
fn h_OSSetAlarm(alarm: *mut OSAlarm, tick: i64, handler: OSAlarmHandler) {
    lg(&format!("OSSetAlarm({},{},{});", pid(alarm as *const c_void), tick as i32, pidf(handler.map_or(0usize, |f| f as usize))));
    unsafe {
        AL.on = true;
        AL.a = alarm;
        AL.h = handler;
    }
}
fn h_HSD_MemAlloc(size: i32) -> *mut c_void {
    lg(&format!("HSD_MemAlloc({});", size));
    if rnd() % 6 == 0 {
        core::ptr::null_mut()
    } else {
        arena_at((rnd() % 64) as usize * 16) as *mut c_void
    }
}
fn h_HSD_GetNextArena(lo: *mut *mut c_void, hi: *mut *mut c_void) {
    lg(&format!("HSD_GetNextArena({},{});", pid(lo as *const c_void), pid(hi as *const c_void)));
    let l = 0x8010_0000u64 + (rnd() % 4) as u64 * 0x1000;
    let h = l + 0x164B400u64 + 0x40000u64 + (rnd() % 4) as u64 * 0x1000;
    unsafe {
        *lo = l as usize as *mut c_void;
        *hi = h as usize as *mut c_void;
    }
}
fn h_OSAllocFromHeap(heap: i32, size: u32) -> *mut c_void {
    lg(&format!("OSAllocFromHeap({},{});", heap, size));
    if rnd() % 4 == 0 {
        core::ptr::null_mut()
    } else {
        arena_at((rnd() % 64) as usize * 16) as *mut c_void
    }
}

fn pump() {
    unsafe {
        let mut n = 0;
        while (DC.on || AL.on) && n < 4000 {
            n += 1;
            if DC.on {
                DC.on = false;
                let f: unsafe extern "C-unwind" fn(i32, usize, *mut c_void, i32) = core::mem::transmute(DC.cb.unwrap());
                f(0, DC.args, core::ptr::null_mut(), 0);
            }
            if AL.on {
                AL.on = false;
                let f: unsafe extern "C-unwind" fn(*mut OSAlarm, *mut OSContext) = core::mem::transmute(AL.h.unwrap());
                f(AL.a, core::ptr::null_mut());
            }
        }
    }
}

static mut H: u32 = 0;
fn hx(v: u32) {
    unsafe { H = (H ^ v).wrapping_mul(16777619) }
}
fn hv<T>(p: *const T) -> u32 {
    let a = p as usize as u64;
    let b = unsafe { &lm::lbMemory_804318B0 as *const _ as usize as u64 };
    if a >= b && a < b + core::mem::size_of::<Allocator>() as u64 {
        return 0x4000_0000u32.wrapping_add((a - b) as u32);
    }
    a as u32
}
fn hfn(f: usize) -> u32 {
    if f == 0 {
        0
    } else if f == done_cb as usize {
        1
    } else if f == lm::lbMemory_80015320 as usize {
        2
    } else {
        3
    }
}
unsafe fn state_hash() -> u32 {
    H = 2166136261;
    let a = &lm::lbMemory_804318B0;
    hx(hv(a.a_arenaLo as *const c_void));
    hx(hv(a.a_arenaHi as *const c_void));
    for i in 0..0x83 {
        hx(hv(a.mem[i].next));
        hx(hv(a.mem[i].addr));
        hx(hv(a.mem[i].size as *const c_void));
    }
    hx(hv(a.free_mem));
    hx(a.num_allocs as u32);
    hx(a.max_num_allocs as u32);
    for i in 0..6 {
        hx(hv(a.heap[i].next));
        hx(hv(a.heap[i].lo));
        hx(hv(a.heap[i].hi));
        hx(hv(a.heap[i].blocks));
    }
    hx(hv(a.free_heap));
    hx(hv(a.aram_heap));
    hx(a.mgr.src as usize as u32);
    hx(a.mgr.dst as usize as u32);
    hx(a.mgr.size);
    hx(a.mgr.offset);
    hx(hv(a.mgr.cb_arg as *const c_void));
    hx(hfn(a.mgr.cb.map_or(0usize, |f| f as usize)));
    hx(a.compact_arg);
    hx(hv(a.compact_cursor as *const c_void));
    hx(hfn(a.compact_cb.map_or(0usize, |f| f as usize)));
    let s = &lh::lbHeap_80431FA0;
    hx(hv(s.arena_lo));
    hx(hv(s.arena_hi));
    hx(s.aram_lo as u32);
    hx(s.aram_hi as u32);
    for i in 0..lh::LBHEAP_HEAP_COUNT {
        let p = &s.heap_array[i];
        hx(p.id as u32);
        hx(hv(p.handle));
        hx(p.start as u32);
        hx(((p.start as u64) >> 32) as u32);
        hx(p.size);
        hx(p.r#type as u32);
        hx(p.transient as u32);
        hx(p.status as u32);
    }
    hx(G_DONE as u32);
    hx(DC.on as u32);
    hx(AL.on as u32);
    H
}
unsafe fn mem_hash() -> u32 {
    let mut h: u32 = 2166136261;
    let m = mem2_at(0x100000);
    for i in 0..0x100000usize {
        h = (h ^ *m.add(i) as u32).wrapping_mul(16777619);
    }
    h
}

unsafe fn heap_ok(a: i32) -> bool {
    let p = &lh::lbHeap_80431FA0.heap_array[a as usize];
    p.status != LbHeapStatus_Create || p.r#type == 0 || (!p.handle.is_null() && p.handle as usize != usize::MAX)
}
#[derive(Clone, Copy)]
struct Slot {
    heap: i32,
    val: *mut c_void,
}
static mut SL: [Slot; 64] = [Slot { heap: 0, val: core::ptr::null_mut() }; 64];
static mut NSL: usize = 0;

fn in_alloc(v: *const c_void) -> bool {
    let a = v as usize as u64;
    let b = unsafe { &lm::lbMemory_804318B0 as *const _ as usize as u64 };
    a >= b && a < b + core::mem::size_of::<Allocator>() as u64
}

/// The C `TRY`: the flag describes the last TRY that ran (each one clears it first).
unsafe fn trying<F: FnOnce()>(f: F) {
    G_ASSERTED = false;
    if try_run(f) {
        G_ASSERTED = true;
    }
}

unsafe fn do_free(pa: &mut i32, pb: &mut u32) {
    let mut a = *pa;
    let mut b = *pb;
    let mut mode = (rnd() % 8) as i32;
    mode = if mode == 7 { 2 } else if mode < 4 { 0 } else { 1 };
    if NSL > 0 {
        let mut s = (rnd() % NSL as u32) as usize;
        if G_FREE_HEAP >= 0 {
            let mut m = 0;
            for q in 0..NSL {
                m += (SL[q].heap == G_FREE_HEAP) as u32;
            }
            if m == 0 {
                return;
            }
            let mut pick = (rnd() % m) as i32;
            for q in 0..NSL {
                if SL[q].heap == G_FREE_HEAP {
                    let hit = pick == 0;
                    pick -= 1;
                    if hit {
                        s = q;
                        break;
                    }
                }
            }
        }
        let mut v = SL[s].val;
        a = SL[s].heap;
        if in_alloc(v) && mode == 0 {
            mode = 1;
        }
        if mode == 1 && in_alloc(v) {
            v = (*(v as *mut Handle)).lo;
        }
        if mode == 2 {
            v = (rnd() % 4096) as usize as *mut c_void;
            a = (rnd() % 7) as i32;
        }
        b = hv(v);
        if heap_ok(a) {
            trying(|| lh::lbHeap_80015CA8(a, v));
        }
        if !G_ASSERTED && mode != 2 {
            NSL -= 1;
            SL[s] = SL[NSL];
        }
    }
    *pa = a;
    *pb = b;
}

#[test]
fn lbheap() {
    arena_init();
    mem2_init();
    install_common();
    unsafe {
        preg("buf", arena_at(0), 1 << 20);
        preg("alloc", &lm::lbMemory_804318B0 as *const _, core::mem::size_of::<Allocator>());
        preg("hs", &lh::lbHeap_80431FA0 as *const _, core::mem::size_of::<lbHeap_HeapState>());
        pfreg("done", done_cb as usize);
        pfreg("cb", lm::lbMemory_80015320 as usize);
        pfreg("alarm", lm::fn_80015184 as usize);
        gen_stubs::install_generated();
        lm::ext::H_HSD_DevComRequest = Some(h_HSD_DevComRequest);
        lm::ext::H_OSSetAlarm = Some(h_OSSetAlarm);
        lh::ext::H_HSD_MemAlloc = Some(h_HSD_MemAlloc);
        lh::ext::H_HSD_GetNextArena = Some(h_HSD_GetNextArena);
        lh::ext::H_OSAllocFromHeap = Some(h_OSAllocFromHeap);
        for i in 0..0x180000usize {
            *mem2_at(0x100000 + i) = rnd() as u8;
        }
        let n = 6000;
        for i in 0..n {
            let op = (rnd() % 100) as i32;
            let mut a: i32 = 0;
            let mut b: u32 = 0;
            let mut ret: u64 = 0;
            let mut rk = "-";
            G_ASSERTED = false;
            log_take();
            if i == 0 || op < 4 {
                let mut t = [0u32; 5];
                for k in 0..5 {
                    t[k] = if rnd() % 4 == 0 { 1 } else { 0 };
                }
                t[4] = 0;
                core::ptr::write_bytes(&raw mut lm::lbMemory_804318B0, 0, 1);
                core::ptr::write_bytes(&raw mut lh::lbHeap_80431FA0, 0, 1);
                DC.on = false;
                AL.on = false;
                NSL = 0;
                trying(|| {
                    lm::lbMemory_8001564C();
                    lh::lbHeap_80015F3C();
                    for k in 0..5 {
                        lh::lbHeap_800158D0(2 + k as i32, t[k] as i32);
                    }
                    lh::lbHeap_80015900();
                });
                a = -1;
            } else if op < 7 {
                a = (2 + rnd() % 5) as i32;
                b = rnd() % 2;
                trying(|| {
                    lh::lbHeap_800158D0(a, b as i32);
                    lh::lbHeap_80015900();
                });
                NSL = 0;
            } else if op < 35 || op >= 89 {
                let r = (rnd() % 25) as i32;
                a = (rnd() % 7) as i32;
                if rnd() % 2 != 0 {
                    a = 3 + (rnd() % 3) as i32;
                }
                let sz: usize = if r == 0 { 0 } else if r == 1 { 0x1000_0000 } else { (rnd() % 0x3000) as usize };
                b = sz as u32;
                let mut res: *mut c_void = core::ptr::null_mut();
                if heap_ok(a) {
                    trying(|| res = lh::lbHeap_80015BD0(a, sz));
                }
                ret = res as usize as u64;
                rk = "p";
                if !G_ASSERTED && !res.is_null() {
                    let s = if NSL < 64 {
                        NSL += 1;
                        NSL - 1
                    } else {
                        (rnd() % 64) as usize
                    };
                    SL[s].heap = a;
                    SL[s].val = res;
                }
            } else if op < 65 {
                if NSL > 0 {
                    do_free(&mut a, &mut b);
                }
            } else if op < 80 {
                let mut ok = true;
                a = (rnd() % 7) as i32;
                if rnd() % 2 != 0 {
                    a = 3 + (rnd() % 3) as i32;
                }
                b = rnd() % 100;
                if a >= 2 {
                    let p = &lh::lbHeap_80431FA0.heap_array[a as usize];
                    ok = p.status == LbHeapStatus_Create && p.r#type != 0 && !p.handle.is_null() && p.handle as usize != usize::MAX;
                }
                if ok {
                    let mut r = 0i32;
                    G_DONE = 0;
                    G_FREE_HEAP = if a >= 1 { a } else { -1 };
                    for _ in 0..3 {
                        if NSL > 0 && rnd() % 3 != 0 {
                            let (mut fa, mut fb) = (0i32, 0u32);
                            do_free(&mut fa, &mut fb);
                        }
                    }
                    trying(|| {
                        r = lh::lbHeap_80015D6C(a as u32, Some(done_cb), b);
                        pump();
                    });
                    G_FREE_HEAP = -1;
                    ret = r as u64;
                    rk = "i";
                } else {
                    rk = "skip";
                }
            } else if op < 83 {
                a = (rnd() % 9) as i32 - 1;
                let q1 = if a >= 0 && a < 7 { lh::lbHeap_800158E8(a) as u32 } else { 99 };
                let q2 = if a >= 0 && a < 7 { lh::lbHeap_80015BB8(a) as u32 } else { 99 };
                let q3 = lh::lbHeap_Capacity(a);
                let mut q4 = 0u32;
                trying(|| q4 = lh::lbHeap_Free(a));
                let mut name = [0 as c_char; 16];
                let hp: *mut c_void = if a >= 0 && a < 7 { lh::lbHeap_80431FA0.heap_array[a as usize].handle as *mut c_void } else { core::ptr::null_mut() };
                let id = lh::lbHeap_IdOfHandle(hp, name.as_mut_ptr(), 16);
                let nm = std::ffi::CStr::from_ptr(name.as_ptr()).to_string_lossy().into_owned();
                outf!("q {} {} {} {} id={} name={}\n", q1, q2, q3, q4, id, nm);
            } else if op < 86 {
                a = (rnd() % 3) as i32;
                b = rnd() % 3000;
                let mut res: *mut c_void = core::ptr::null_mut();
                trying(|| {
                    res = lh::lbHeap_StageSlotTryAlloc(b);
                    lh::lbHeap_StageSlotDump(a);
                });
                ret = res as usize as u64;
                rk = "p";
            } else {
                trying(|| lh::lbHeap_80015DF8());
            }
            let rs = if rk == "p" {
                pidu(ret)
            } else if rk == "i" {
                (if ret != 0 { "1" } else { "0" }).to_string()
            } else {
                "-".to_string()
            };
            let lgs = log_take();
            outf!("{} op={} a={} b={} asserted={} ret={}:{} h={:08x} log={}\n", i, op, a, b, G_ASSERTED as i32, rk, rs, state_hash(), lgs);
            if i % 25 == 24 {
                outf!("mem {:08x}\n", mem_hash());
            }
        }
    }
    check("tu_lbheap");
}
