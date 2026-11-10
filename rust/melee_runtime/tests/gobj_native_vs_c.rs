//! C functions this scenario covers beyond its direct calls: HSD_GObj_80391260 (inside HSD_GObjInit), HSD_GObjProc_QueueProc (inside
//! HSD_GObj_SetupProc), HSD_GObjProc_UnlinkProcFromGObj (inside HSD_GObjProc_RemoveProc / RemoveAllProcs).
//! `tu::gobj_native` vs the real gobj*.c (shipping contraction): the same random scenario of library
//! calls as `gen/pure/drv_gobj_native.c`, with the whole library state hashed after every operation
//! (pointers replaced by creation-order ids). Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py gobj_native`.
#![allow(static_mut_refs, clippy::missing_safety_doc)]
mod pure_common;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_runtime::tu::gobj_native::*;
use melee_runtime::tu::{memory, objalloc};
use melee_types::*;
use pure_common::*;

const MAXG: usize = 24;
const MAXP: usize = 48;
static mut GP: [*mut HSD_GObj; MAXG] = [null_mut(); MAXG];
static mut GA: [bool; MAXG] = [false; MAXG];
static mut NG: usize = 0;
static mut PP: [*mut HSD_GObjProc; MAXP] = [null_mut(); MAXP];
static mut PA: [bool; MAXP] = [false; MAXP];
static mut NP: usize = 0;
static mut H: u64 = 0;
static mut TRACE: u32 = 0;

static mut DUMPW: Option<std::fs::File> = None;
unsafe fn hw(w: u32) {
    if let Some(f) = DUMPW.as_mut() {
        use std::io::Write;
        let _ = writeln!(f, "w {w}");
    }
    for b in w.to_le_bytes() {
        H ^= b as u64;
        H = H.wrapping_mul(0x100000001B3);
    }
}
unsafe fn gid(g: *mut HSD_GObj) -> i32 {
    if g.is_null() {
        return -1;
    }
    for i in (0..NG).rev() {
        if GP[i] == g && GA[i] {
            return i as i32;
        }
    }
    -2
}
unsafe fn pid(p: *mut HSD_GObjProc) -> i32 {
    if p.is_null() {
        return -1;
    }
    for i in (0..NP).rev() {
        if PP[i] == p && PA[i] {
            return i as i32;
        }
    }
    -2
}
unsafe fn reg_g(g: *mut HSD_GObj) {
    if NG < MAXG {
        GP[NG] = g;
        GA[NG] = true;
        NG += 1;
    }
}
unsafe fn reg_p(p: *mut HSD_GObjProc) {
    if NP < MAXP {
        PP[NP] = p;
        PA[NP] = true;
        NP += 1;
    }
}
unsafe fn kill_g(g: *mut HSD_GObj) {
    let i = gid(g);
    if i >= 0 {
        GA[i as usize] = false;
    }
}
unsafe fn kill_p(p: *mut HSD_GObjProc) {
    let i = pid(p);
    if i >= 0 {
        PA[i as usize] = false;
    }
}
static mut DUMPT: Option<std::fs::File> = None;
unsafe fn tr(m: u32) {
    if let Some(f) = DUMPT.as_mut() {
        use std::io::Write;
        let _ = writeln!(f, "t {m}");
    }
    TRACE = TRACE.wrapping_mul(16777619) ^ m;
}

unsafe extern "C" fn cb_nop(g: *mut HSD_GObj) {
    tr(1000 + (gid(g) + 2) as u32);
}
unsafe extern "C" fn cb_free(g: *mut HSD_GObj) {
    tr(2000 + (gid(g) + 2) as u32);
    HSD_GObjFree(g);
}
unsafe extern "C" fn cb_rmproc(g: *mut HSD_GObj) {
    tr(3000 + (gid(g) + 2) as u32);
    if !(*g).proc.is_null() {
        HSD_GObjProc_RemoveProc((*g).proc);
    }
}
unsafe extern "C" fn cb_chpri(g: *mut HSD_GObj) {
    tr(4000 + (gid(g) + 2) as u32);
    HSD_GObjPLink_ChangeGObjPri_Unk(0, g, (((*g).p_link as u32 + 1) % 4) as u8, 3, null_mut());
}
unsafe extern "C" fn cb_addproc(g: *mut HSD_GObj) {
    tr(5000 + (gid(g) + 2) as u32);
    HSD_GObj_SetupProc(g, Some(cb_nop), 1);
}
unsafe extern "C" fn render_a(g: *mut HSD_GObj, pass: isize) {
    tr(6000 + (gid(g) + 2) as u32 * 8 + pass as u32);
}
unsafe extern "C" fn render_b(g: *mut HSD_GObj, pass: isize) {
    tr(7000 + (gid(g) + 2) as u32 * 8 + pass as u32);
}
unsafe extern "C" fn ud_remove(d: *mut c_void) {
    tr(8000 + *(d as *const i32) as u32);
}
unsafe extern "C" fn kind0_free(o: *mut HSD_Obj) {
    tr(9000 + *(o as *const i32) as u32);
}
unsafe extern "C" fn kind1_free(o: *mut HSD_Obj) {
    tr(9100 + *(o as *const i32) as u32);
}
static mut KFUNCS: [GObjFunc; 2] = [Some(kind0_free), Some(kind1_free)];
static mut KFS: GObjFuncs = GObjFuncs { next: null_mut(), size: 2, funcs: null_mut() };
static mut OBJS: [i32; 4] = [11, 22, 33, 44];

fn cbs() -> [HSD_GObjEvent; 5] {
    [Some(cb_nop), Some(cb_free), Some(cb_rmproc), Some(cb_chpri), Some(cb_addproc)]
}

unsafe fn state() {
    hw(HSD_GObj_804D783C as u32);
    hw(HSD_GObj_DelayedProcInfo.flags);
    hw(pid(HSD_GObj_CurrentInvokedProc) as u32);
    hw(gid(HSD_GObj_CurrentInvokedProcGObj) as u32);
    hw(pid(HSD_GObj_NextInvokedProc) as u32);
    hw(HSD_GObj_CurrentInvokedSLink as u32);
    let pm = HSD_GObjLibInitData.p_link_max as usize;
    let gm = HSD_GObjLibInitData.gx_link_max as usize;
    let sm = HSD_GObjLibInitData.gproc_pri_max as usize;
    for l in 0..=pm {
        hw(0xA0000000 + l as u32);
        let mut g = *HSD_GObjPLinkHead.add(l);
        while !g.is_null() {
            hw(gid(g) as u32);
            g = (*g).next;
        }
        hw(gid(*plinklow_gobjs.add(l)) as u32);
        for i in 0..=sm {
            hw(pid(*HSD_GObj_ProcList.add(l + i * (pm + 1))) as u32);
        }
    }
    for l in 0..=gm + 1 {
        hw(0xB0000000 + l as u32);
        let mut g = *HSD_GObjGXLinkHead.add(l);
        while !g.is_null() {
            hw(gid(g) as u32);
            g = (*g).next_gx;
        }
        hw(gid(*HSD_GObj_804D7820.add(l)) as u32);
    }
    for i in 0..=sm {
        hw(0xC0000000 + i as u32);
        let mut p = *HSD_GObj_GObjProcHead.add(i);
        while !p.is_null() {
            hw(pid(p) as u32);
            hw(pid((*p).prev) as u32);
            p = (*p).next;
        }
    }
    for i in 0..NG {
        if !GA[i] {
            continue;
        }
        let g = GP[i];
        hw((*g).classifier as u32);
        hw((*g).p_link as u32);
        hw((*g).gx_link as u32);
        hw((*g).p_priority as u32);
        hw((*g).render_priority as u32);
        hw((*g).obj_kind as u32);
        hw((*g).user_data_kind as u32);
        hw(gid((*g).next) as u32);
        hw(gid((*g).prev) as u32);
        hw(gid((*g).next_gx) as u32);
        hw(gid((*g).prev_gx) as u32);
        hw(pid((*g).proc) as u32);
        let rc = (*g).render_cb;
        hw(if rc == Some(render_a as unsafe extern "C" fn(*mut HSD_GObj, isize)) {
            1
        } else if rc == Some(render_b as unsafe extern "C" fn(*mut HSD_GObj, isize)) {
            2
        } else if rc.is_some() {
            3
        } else {
            0
        });
        hw((*g).gxlink_prios as u32);
        hw(((*g).gxlink_prios >> 32) as u32);
        hw(if (*g).hsd_obj.is_null() { 0 } else { *((*g).hsd_obj as *const i32) as u32 });
        hw(if (*g).user_data.is_null() { 0 } else { *((*g).user_data as *const i32) as u32 });
        hw((*g).user_data_remove_func.is_some() as u32);
    }
    let table = cbs();
    for i in 0..NP {
        if !PA[i] {
            continue;
        }
        let p = PP[i];
        hw(pid((*p).child) as u32);
        hw(pid((*p).next) as u32);
        hw(pid((*p).prev) as u32);
        hw((*p).s_link as u32);
        hw((*p).flags_1() as u32);
        hw((*p).flags_2() as u32);
        hw((*p).flags_3() as u32);
        hw(gid((*p).gobj) as u32);
        for (c, f) in table.iter().enumerate() {
            if (*p).on_invoke == *f {
                hw(100 + c as u32);
            }
        }
    }
}

unsafe fn pick(r: &mut Rng, n: usize, ok: impl Fn(usize) -> bool, len: usize) -> i32 {
    let mut ids = [0usize; MAXP];
    let mut k = 0;
    for i in 0..len {
        if ok(i) {
            ids[k] = i;
            k += 1;
        }
    }
    let _ = n;
    if k == 0 {
        -1
    } else {
        ids[(r.u() % k as u64) as usize] as i32
    }
}
unsafe fn pick_g(r: &mut Rng) -> i32 {
    pick(r, 0, |i| GA[i], NG)
}
unsafe fn pick_gx(r: &mut Rng) -> i32 {
    pick(r, 0, |i| GA[i] && (*GP[i]).gx_link != 0xFF, NG)
}
unsafe fn pick_gnx(r: &mut Rng) -> i32 {
    pick(r, 0, |i| GA[i] && (*GP[i]).gx_link == 0xFF, NG)
}
unsafe fn pick_same_link(r: &mut Rng, link: u32) -> i32 {
    pick(r, 0, |i| GA[i] && (*GP[i]).p_link as u32 == link, NG)
}
unsafe fn pick_p(r: &mut Rng) -> i32 {
    pick(r, 0, |i| PA[i] && gid((*PP[i]).gobj) >= 0, NP)
}

unsafe fn install_hooks() {
    memory::ext::hook::HSD_GetHeap = Some(|| 0);
    memory::ext::hook::OSAllocFromHeap = Some(|_h, size| {
        let l = std::alloc::Layout::from_size_align(size.max(8) as usize, 16).unwrap();
        std::alloc::alloc_zeroed(l).cast()
    });
    memory::ext::hook::OSFreeToHeap = Some(|_h, _p| ());
    memory::ext::hook::PcTraceHeapEnabled = Some(|| 0);
    memory::ext::hook::Snap_NoteMem = Some(|_s, _p, _c, _f| ());
    memory::return_address_hook = Some(|_d| null_mut());
    melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());
    objalloc::ext::hook::OSCheckHeap = Some(|_h| 0x7FFFFFFF);
    objalloc::ext::hook::Snap_InRender = Some(|| 0);
    objalloc::ext::hook::Snap_NoteObj = Some(|_d, _o, _f| ());
    objalloc::ext::hook::Snap_NoteArena = Some(|_d, _s, _z| ());
    objalloc::ext::hook::Snap_RenderPoolCount = Some(|| 0);
    objalloc::ext::hook::Snap_RenderPoolAt = Some(|_i| null_mut());
    objalloc::ext::hook::Snap_Resimulating = Some(|| 0);
}

#[test]
fn gobj_native_matches_c() {
    unsafe {
        install_hooks();
        let mut r = Rng::new();
        let mut out = String::new();
        let dbg = std::env::var("GOBJ_DBG").is_ok();
        let mut dbgf = std::env::var("GOBJ_DBG").ok().map(|p| std::fs::File::create(p).unwrap());
        for k in 0..300 {
            let mut chain = 0xCBF29CE484222325u64;
            NG = 0;
            NP = 0;
            TRACE = 2166136261;
            let mut init = _HSD_GObjLibInitDataType::zeroed();
            HSD_GObjSetInitDefaults(&mut init);
            init.p_link_max = 3;
            init.gx_link_max = 3;
            init.gproc_pri_max = 2;
            KFS = GObjFuncs { next: null_mut(), size: 2, funcs: KFUNCS.as_mut_ptr() };
            init.funcs = &raw mut KFS;
            KFS.next = null_mut();
            HSD_GObjInit(&mut init);
            let table = cbs();
            for step in 0..60 {
                let op = (r.u() % 26) as i32;
                let rr = r.u() as u32;
                H = 0xCBF29CE484222325;
                DUMPT = match std::env::var("GOBJ_DUMP") {
                    Ok(v) if v.parse::<i32>().ok() == Some(k * 100 + step) => Some(std::fs::File::create(std::env::var("GOBJ_DUMPT").unwrap()).unwrap()),
                    _ => None,
                };
                match op {
                    0..=2 => {
                        if NG >= MAXG {
                        } else {
                            let g = GObj_Create((rr % 5) as u16, ((rr >> 4) % 4) as u8, ((rr >> 8) % 6) as u8);
                            reg_g(g);
                        }
                    }
                    3 => {
                        let wh = (rr % 4) as i32;
                        let pl = ((rr >> 4) % 4) as u8;
                        let mut pos = -1;
                        let mut skip = NG >= MAXG;
                        if !skip && wh >= 2 {
                            pos = pick_same_link(&mut r, pl as u32);
                            if pos < 0 {
                                skip = true;
                            }
                        }
                        if !skip {
                            let g = CreateGObj(
                                wh,
                                (((rr >> 8) as u16) % 5) as u16,
                                pl,
                                ((rr >> 12) % 6) as u8,
                                if pos < 0 { null_mut() } else { GP[pos as usize] },
                            );
                            reg_g(g);
                        }
                    }
                    4 | 5 | 6 => {
                        let a = pick_gnx(&mut r);
                        if a >= 0 {
                            let cb: GObj_RenderFunc = if rr & 1 != 0 { Some(render_a) } else { Some(render_b) };
                            let g = GP[a as usize];
                            match op {
                                4 => GObj_SetupGXLink(g, cb, ((rr >> 4) % 4) as u8, (rr >> 8) % 6),
                                5 => GObj_SetupGXLinkMax(g, cb, (rr >> 8) % 6),
                                _ => GObj_SetupGXLinkMaxSorted(g, cb, (rr >> 8) % 6),
                            }
                        }
                    }
                    7 => {
                        let a = pick_gx(&mut r);
                        if a >= 0 {
                            HSD_GObjGXLink_8039084C(GP[a as usize]);
                        }
                    }
                    8 => {
                        let a = pick_gx(&mut r);
                        if a >= 0 {
                            HSD_GObjGXLink_80390908(GP[a as usize], (rr % 4) as u8, ((rr >> 4) % 6) as u8);
                        }
                    }
                    9 => {
                        let a = pick_gx(&mut r);
                        let b = pick_gx(&mut r);
                        if !(a < 0 || b < 0 || a == b) {
                            HSD_GObjGXLink_803909D8(GP[a as usize], GP[b as usize]);
                        }
                    }
                    10..=12 => {
                        let a = pick_g(&mut r);
                        if !(a < 0 || NP >= MAXP) {
                            let p = HSD_GObj_SetupProc(GP[a as usize], table[(rr % 5) as usize], ((rr >> 4) % 3) as u8);
                            reg_p(p);
                        }
                    }
                    13 => {
                        let b = pick_p(&mut r);
                        if b >= 0 {
                            let p = PP[b as usize];
                            HSD_GObjProc_RemoveProc(p);
                            if HSD_GObj_DelayedProcInfo.flags & 4 == 0 {
                                kill_p(p);
                            }
                        }
                    }
                    14 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            let g = GP[a as usize];
                            let mut p = (*g).proc;
                            let mut ids = [0i32; MAXP];
                            let mut c = 0;
                            while !p.is_null() {
                                ids[c] = pid(p);
                                c += 1;
                                p = (*p).child;
                                if c >= MAXP {
                                    break;
                                }
                            }
                            HSD_GObjProc_RemoveAllProcs(g);
                            for i in 0..c {
                                if ids[i] >= 0 && HSD_GObj_DelayedProcInfo.flags & 4 == 0 {
                                    PA[ids[i] as usize] = false;
                                }
                            }
                        }
                    }
                    15 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            let mut pos: *mut HSD_GObj = null_mut();
                            let mut skip = false;
                            if rr % 4 >= 2 {
                                let b = pick_same_link(&mut r, (rr >> 4) % 4);
                                if b < 0 || b == a {
                                    skip = true;
                                } else {
                                    pos = GP[b as usize];
                                }
                            }
                            if !skip {
                                HSD_GObjPLink_ChangeGObjPri_Unk(
                                    rr % 4,
                                    GP[a as usize],
                                    ((rr >> 4) % 4) as u8,
                                    ((rr >> 8) % 6) as u8,
                                    pos,
                                );
                            }
                        }
                    }
                    16 => {
                        let a = pick_g(&mut r);
                        if !(a < 0 || (*GP[a as usize]).user_data_kind != 0xFF) {
                            GObj_InitUserData(
                                GP[a as usize],
                                (rr % 3) as u8,
                                Some(ud_remove),
                                (&raw mut OBJS as *mut i32).add((rr % 4) as usize).cast(),
                            );
                        }
                    }
                    17 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            GObj_RemoveUserData(GP[a as usize]);
                        }
                    }
                    18 => {
                        let a = pick_g(&mut r);
                        if !(a < 0 || (*GP[a as usize]).obj_kind != 0xFF) {
                            HSD_GObjObject_80390A70(
                                GP[a as usize],
                                (rr % 2) as u8,
                                (&raw mut OBJS as *mut i32).add(((rr >> 4) % 4) as usize).cast(),
                            );
                        }
                    }
                    19 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            let o = HSD_GObjObject_80390ADC(GP[a as usize]);
                            hw(if o.is_null() { 0xFFFF } else { *(o as *const i32) as u32 });
                        }
                    }
                    20 => {
                        let a = pick_g(&mut r);
                        if a >= 0 && HSD_GObj_DelayedProcInfo.flags == 0 {
                            let mut p = (*GP[a as usize]).proc;
                            let mut ids = [0i32; MAXP];
                            let mut c = 0;
                            while !p.is_null() && c < MAXP {
                                ids[c] = pid(p);
                                c += 1;
                                p = (*p).child;
                            }
                            HSD_GObjFree(GP[a as usize]);
                            kill_g(GP[a as usize]);
                            for i in 0..c {
                                if ids[i] >= 0 {
                                    PA[ids[i] as usize] = false;
                                }
                            }
                        }
                    }
                    21 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            let f = HSD_GObjObject_80390A3C((rr % 5) as u16, ((rr >> 4) % 4) as u8);
                            hw(gid(f) as u32);
                        }
                    }
                    22 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            let g = GP[a as usize];
                            match rr % 4 {
                                0 => HSD_GObj_80390C5C(g),
                                1 => HSD_GObj_80390C84(g),
                                2 => HSD_GObj_80390CAC(g),
                                _ => HSD_GObj_80390CD4(g),
                            }
                        }
                    }
                    24 => {
                        let a = pick_g(&mut r);
                        if a >= 0 {
                            (*GP[a as usize]).gxlink_prios = r.u() & 0x1F;
                            HSD_GObj_80390ED0(GP[a as usize], rr & 7);
                            hw(gid(HSD_GObj_804D7814) as u32);
                        }
                    }
                    25 => {
                        HSD_GObj_80390FC0();
                        hw(gid(HSD_GObj_804D7818) as u32);
                    }
                    _ => {
                        // 23: RunProcs, then drop what the callbacks freed from the registry
                        HSD_GObj_RunProcs();
                        for i in 0..NG {
                            if GA[i] {
                                let g = GP[i];
                                let mut found = false;
                                let mut l = 0;
                                while l <= HSD_GObjLibInitData.p_link_max as usize && !found {
                                    let mut q = *HSD_GObjPLinkHead.add(l);
                                    while !q.is_null() {
                                        if q == g {
                                            found = true;
                                            break;
                                        }
                                        q = (*q).next;
                                    }
                                    l += 1;
                                }
                                if !found {
                                    GA[i] = false;
                                }
                            }
                        }
                        for i in 0..NP {
                            if PA[i] {
                                let mut alive = false;
                                let mut q = 0;
                                while q <= HSD_GObjLibInitData.gproc_pri_max as usize && !alive {
                                    let mut t = *HSD_GObj_GObjProcHead.add(q);
                                    while !t.is_null() {
                                        if t == PP[i] {
                                            alive = true;
                                            break;
                                        }
                                        t = (*t).next;
                                    }
                                    q += 1;
                                }
                                if !alive {
                                    PA[i] = false;
                                }
                            }
                        }
                        for q in 0..=HSD_GObjLibInitData.gproc_pri_max as usize {
                            let mut t = *HSD_GObj_GObjProcHead.add(q);
                            while !t.is_null() {
                                if pid(t) == -2 && NP < MAXP {
                                    reg_p(t);
                                }
                                t = (*t).next;
                            }
                        }
                    }
                }
                DUMPW = match std::env::var("GOBJ_DUMP") {
                    Ok(v) if v.parse::<i32>().ok() == Some(k * 100 + step) => Some(std::fs::File::create(std::env::var("GOBJ_DUMPF").unwrap()).unwrap()),
                    _ => None,
                };
                hw(TRACE);
                state();
                DUMPW = None;
                if dbg {
                    use std::io::Write;
                    let _ = writeln!(dbgf.as_mut().unwrap(), "dbg {k} {step} op{op} {H:016x}");
                    let _ = dbgf.as_mut().unwrap().flush();
                }
                let hb = H.to_le_bytes();
                for b in hb {
                    chain ^= b as u64;
                    chain = chain.wrapping_mul(0x100000001B3);
                }
            }
            out += &format!("gobj {k} {chain:016x}\n");
        }
        compare("gobj_native", &out);
    }
}
