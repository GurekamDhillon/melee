//! src/tu/ftparts.rs (part setup, visibility tables, 80074E58, 80074194, 8007487C, JObjSetRotation)
//! vs the C (gen/cref/tu/ftparts.c: the whole TU, shipping contraction; regenerate with
//! `python rust/melee_fighter/gen/cref/tu/run_tu.py ftparts`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftdata;
use melee_fighter::tu::ftparts as m;
use melee_fighter::types::*;
use tu_ftco_harness::*;

const KIND: usize = 7;
const MASK: u32 = (JOBJ_LIGHTING | JOBJ_TEXGEN | JOBJ_SPECULAR | JOBJ_UNK_B18 | JOBJ_UNK_B19 | JOBJ_UNK_B20) as u32;

struct W {
    jt: *mut [HSD_JObj; 16],
    ja: *mut [HSD_JObj; 16],
    dp: *mut [HSD_DObj; 64],
    mo: *mut [HSD_MObj; 64],
    to: *mut [HSD_TObj; 128],
    dl: *mut [*mut HSD_DObj; 128],
    dl2: *mut [*mut HSD_DObj; 128],
    parts: *mut [FighterBone; 40],
    pt: *mut FighterPartsTable,
    j2p: *mut [u8; 64],
    p2j: *mut [u8; 64],
    t6540: *mut Fighter_804D6540_t,
    x6540: *mut [Fighter_804D6540_x0_t; 4],
    lka: *mut [[FtPartsVisLookup; 12]; 5],
    tsa: *mut [[[TempS; 3]; 12]; 5],
    ub: *mut [[[[u8; 4]; 3]; 12]; 5],
    vt: *mut [[*mut c_void; 4]; 3],
    fdesc: *mut FtPartsDesc,
    fx8: *mut ftData_x8,
    fdata: *mut ftData,
    ptbl: *mut [*mut FighterPartsTable; 127],
    t6540l: *mut [*mut Fighter_804D6540_t; 127],
    nj: usize,
    nnd: usize,
}
static mut WW: *mut W = null_mut();
fn w() -> &'static mut W { unsafe { &mut *WW } }
fn bz<T>() -> *mut T { Box::leak(Box::new(unsafe { core::mem::zeroed::<T>() })) }

static mut HH: u32 = 0;
fn hm(v: u32) { unsafe { HH = HH.wrapping_mul(0x0100_0193) ^ v } }

fn ep<T>(p: *const T) -> u32 {
    let a = p as usize;
    if a == 0 {
        return 0;
    }
    let w = w();
    let inr = |base: *const u8, n: usize, sz: usize| a >= base as usize && a < base as usize + n * sz;
    if inr(w.jt as *const u8, 16, core::mem::size_of::<HSD_JObj>()) { return 0x100 + ((a - w.jt as usize) / core::mem::size_of::<HSD_JObj>()) as u32; }
    if inr(w.ja as *const u8, 16, core::mem::size_of::<HSD_JObj>()) { return 0x200 + ((a - w.ja as usize) / core::mem::size_of::<HSD_JObj>()) as u32; }
    if inr(w.dp as *const u8, 64, core::mem::size_of::<HSD_DObj>()) { return 0x300 + ((a - w.dp as usize) / core::mem::size_of::<HSD_DObj>()) as u32; }
    if inr(w.to as *const u8, 128, core::mem::size_of::<HSD_TObj>()) { return 0x400 + ((a - w.to as usize) / core::mem::size_of::<HSD_TObj>()) as u32; }
    if a == w.dl as usize { return 0x501; }
    if a == w.dl2 as usize { return 0x502; }
    if a == w.parts as usize { return 0x503; }
    if a == unsafe { F } as usize { return 0x504; }
    0xFFFF
}
fn lkp(p: *const FtPartsVisLookup) -> u32 {
    if p.is_null() {
        return 0;
    }
    let w = w();
    for r in 0..5 {
        if p as usize == unsafe { (*w.lka)[r].as_ptr() } as usize {
            return r as u32 + 1;
        }
    }
    0xFF
}

unsafe fn gen_tree() {
    let w = w();
    let mut par = [0i32; 16];
    w.nj = 1 + (rnd() % 10) as usize;
    *w.jt = core::mem::zeroed();
    *w.ja = core::mem::zeroed();
    *w.dp = core::mem::zeroed();
    *w.mo = core::mem::zeroed();
    *w.to = core::mem::zeroed();
    let (jt, ja, dp) = (&mut *w.jt, &mut *w.ja, &mut *w.dp);
    for i in 0..w.nj {
        let r = rnd();
        par[i] = if i != 0 { ((r >> 8) % i as u32) as i32 } else { -1 };
        jt[i].flags = if r % 6 == 0 { JOBJ_INSTANCE as u32 } else { 0 };
        jt[i].flags |= rnd() & MASK;
    }
    for i in (1..w.nj).rev() {
        let p = par[i] as usize;
        jt[i].parent = &mut jt[p];
        jt[i].next = jt[p].child;
        jt[p].child = &mut jt[i];
    }
    let idx_of = |p: *mut HSD_JObj, base: *mut HSD_JObj| -> usize { (p as usize - base as usize) / core::mem::size_of::<HSD_JObj>() };
    for i in 0..w.nj {
        ja[i].flags = jt[i].flags;
        ja[i].parent = if jt[i].parent.is_null() { null_mut() } else { &mut ja[idx_of(jt[i].parent, jt.as_mut_ptr())] };
        ja[i].child = if jt[i].child.is_null() { null_mut() } else { &mut ja[idx_of(jt[i].child, jt.as_mut_ptr())] };
        ja[i].next = if jt[i].next.is_null() { null_mut() } else { &mut ja[idx_of(jt[i].next, jt.as_mut_ptr())] };
    }
    let mut nd = 0usize;
    for i in 0..w.nj {
        let c = (rnd() % 4) as usize;
        let mut first: *mut HSD_DObj = null_mut();
        for q in (0..c).rev() {
            dp[nd + q].next = first;
            first = &mut dp[nd + q];
        }
        jt[i].u.dobj = first;
        nd += c;
    }
    w.nnd = nd;
}

unsafe fn dry(root: *mut HSD_JObj) -> u32 {
    let mut jobj = root;
    let mut part: u32 = 0;
    while !jobj.is_null() {
        if m::ftparts_8007506c(KIND as i32, part as i32) != 0 {
            part += 1;
            continue;
        }
        part += 1;
        if (*jobj).flags & JOBJ_INSTANCE as u32 == 0 && !(*jobj).child.is_null() {
            jobj = (*jobj).child;
            continue;
        }
        if !(*jobj).next.is_null() {
            jobj = (*jobj).next;
            continue;
        }
        loop {
            if (*jobj).parent.is_null() {
                jobj = null_mut();
                break;
            }
            if !(*(*jobj).parent).next.is_null() {
                jobj = (*(*jobj).parent).next;
                break;
            }
            jobj = (*jobj).parent;
        }
    }
    part
}

unsafe fn tables() {
    let w = w();
    *w.t6540 = core::mem::zeroed();
    *w.x6540 = core::mem::zeroed();
    (*w.t6540).x4 = (rnd() % 4) as i32;
    (*w.t6540).x0 = (*w.x6540).as_mut_ptr();
    for i in 0..4 {
        (*w.x6540)[i].x0 = (rnd() % (w.nj as u32 + 3)) as u8;
    }
    h::FIGHTER_804D6540 = (*w.t6540l).as_mut_ptr();
    h::FT_PARTS_TABLE = (*w.ptbl).as_mut_ptr();
    (*w.t6540l)[KIND] = if rnd() % 8 == 0 { null_mut() } else { w.t6540 };
    (*w.ptbl)[KIND] = w.pt;
    (*w.pt).joint_to_part = (*w.j2p).as_mut_ptr();
    (*w.pt).part_to_joint = (*w.p2j).as_mut_ptr();
}

unsafe fn hbone(bn: &FighterBone) {
    hm(ep(bn.joint));
    hm(ep(bn.x4_jobj2));
    let a = &bn.x8.x0;
    for v in [a.flags_b0(), a.flags_b1(), a.flags_b2(), a.flags_b3(), a.flags_b4(), a.flags_b5(), a.flags_b6(), a.flags_b7(),
              a.flags2_b0(), a.flags2_b1(), a.flags2_b2(), a.flags2_b3(), a.flags2_b4(), a.flags2_b5(), a.flags2_b6(), a.flags2_b7()] {
        hm(v as u32);
    }
    hm(bn.xC_u.x0.xC as u32);
    hm(bn.xC_u.x0.xD() as u32);
}

unsafe fn vis_world() {
    let w = w();
    let f = f();
    *w.lka = core::mem::zeroed();
    *w.tsa = core::mem::zeroed();
    *w.ub = core::mem::zeroed();
    *w.vt = core::mem::zeroed();
    w.nnd = 1 + (rnd() % 8) as usize;
    for i in 0..128 {
        (*w.dl)[i] = null_mut();
        (*w.dl2)[i] = null_mut();
    }
    for i in 0..64 {
        (*w.dp)[i].flags = rnd();
    }
    for i in 0..w.nnd {
        (*w.dl)[i] = if rnd() % 5 == 0 { null_mut() } else { &mut (*w.dp)[i] };
        (*w.dl2)[i] = if rnd() % 5 == 0 { null_mut() } else { &mut (*w.dp)[8 + i] };
    }
    f.dobj_list.count = w.nnd as u32;
    f.dobj_list.data = (*w.dl).as_mut_ptr();
    f.x203C.count = w.nnd as u32;
    f.x203C.data = (*w.dl2).as_mut_ptr();
    for r in 0..5 {
        for mm in 0..12 {
            (*w.lka)[r][mm].x0 = (rnd() % 3) as i32;
            (*w.lka)[r][mm].x4 = (*w.tsa)[r][mm].as_mut_ptr();
            for s in 0..3 {
                (*w.tsa)[r][mm][s].x0 = (rnd() % 3) as i32;
                (*w.tsa)[r][mm][s].x4 = (*w.ub)[r][mm][s].as_mut_ptr();
                for q in 0..4 {
                    (*w.ub)[r][mm][s][q] = (rnd() % (w.nnd as u32 + 2)) as u8;
                }
            }
        }
    }
    for r in 0..5 {
        f.x5AC.xC[r] = if rnd() % 4 == 0 { null_mut() } else { (*w.lka)[r].as_mut_ptr() };
        f.x5AC.cleared[r] = (rnd() & 1) as u8;
    }
    f.x5AC.model_num = rnd() % 5;
    for i in 0..12 {
        f.x5F4_arr[i].idx = ((rnd() % 5) as i32 - 1) as i8;
        f.x5F4_arr[i].prev = ((rnd() % 5) as i32 - 1) as i8;
    }
    for r in 0..3 {
        for s in 0..4 {
            (*w.vt)[r][s] = if rnd() % 3 == 0 { null_mut() } else { (*w.lka)[(rnd() % 5) as usize].as_mut_ptr() as *mut c_void };
        }
    }
    (*w.fdesc).model_num = rnd() % 12;
    (*w.fdesc).vis_table = ((*w.vt).as_mut_ptr()) as *mut _;
    (*w.fx8).x0 = *w.fdesc;
    (*w.fdata).x8 = w.fx8;
    f.ft_data = w.fdata;
    f.set_x619_costume_id_hack((rnd() % 3) as u8);
    f.kind = KIND as _;
}
trait CostumeHack { fn set_x619_costume_id_hack(&mut self, v: u8); }
impl CostumeHack for Fighter { fn set_x619_costume_id_hack(&mut self, v: u8) { self.costume_id = v; } }

unsafe fn hvis() {
    let w = w();
    let f = &*F;
    hm(f.x5AC.model_num);
    for i in 0..5 {
        hm(lkp(f.x5AC.xC[i]));
        hm(f.x5AC.cleared[i] as u32);
    }
    for i in 0..12 {
        hm(f.x5F4_arr[i].idx as i32 as u32);
        hm(f.x5F4_arr[i].prev as i32 as u32);
    }
    for i in 0..64 {
        hm((*w.dp)[i].flags);
    }
    hm(f.x221D_b2() as u32);
}

unsafe extern "C" fn model_cb(fp: *mut Fighter, mdl: i32, b: i32) {
    t(5);
    t((fp as usize == F as usize) as u32);
    t(mdl as u32);
    t(b as u32);
}

#[test]
fn ftparts_matches_c() {
    init();
    unsafe {
        let world = Box::leak(Box::new(W {
            jt: bz(), ja: bz(), dp: bz(), mo: bz(), to: bz(), dl: bz(), dl2: bz(), parts: bz(), pt: bz(), j2p: bz(), p2j: bz(),
            t6540: bz(), x6540: bz(), lka: bz(), tsa: bz(), ub: bz(), vt: bz(), fdesc: bz(), fx8: bz(), fdata: bz(),
            ptbl: bz(), t6540l: bz(), nj: 0, nnd: 0,
        }));
        WW = world;
        let pa: *mut c_void = Box::leak(Box::new(0u64)) as *mut u64 as *mut c_void;
        let _ = pa;
        use m::ext::hook as hk;
        hk::HSD_ObjAlloc = Some(|d| {
            t(1);
            let parts_d = (&raw mut melee_fighter::tu::fighter::fighter_parts_alloc_data) as *mut c_void;
            t((d == parts_d) as u32);
            if d == parts_d { w().parts as *mut c_void } else { w().dl as *mut c_void }
        });
        hk::HSD_DObjSetFlags = Some(|d, f| { t(2); t(ep(d)); t(f); if !d.is_null() { (*d).flags |= f; } });
        hk::HSD_DObjClearFlags = Some(|d, f| { t(3); t(ep(d)); t(f); if !d.is_null() { (*d).flags &= !f; } });
        hk::HSD_JObjClearFlags = Some(|j, f| { t(4); t(ep(j)); t(f); });
        hk::Mex_CostumeVisIdx = Some(|fk, c| { t(6); t(fk as u32); t(c as u32); (rnd() % 3) as i32 });
    }
    let mut c = Check::new(include_str!("data/tu_ftparts.txt"));
    for op in 0..12u32 {
        for k in 0..1500u32 {
            case(op, k);
            unsafe {
                HH = 0x811C_9DC5;
                let w = w();
                *w.parts = core::mem::zeroed();
                *w.p2j = [0; 64];
                *w.j2p = [0; 64];
                (*GO).user_data = F as *mut c_void;
                (*F).gobj = GO;
                (*F).player_idx = (rnd() % 4) as u8;
                (*F).kind = KIND as _;
                gen_tree();
                tables();
                ftdata::ftData_UnkIntBoolFunc0.model_events[KIND] = if rnd() & 1 != 0 { Some(model_cb) } else { None };
                let f = f();
                let mut ret = 0u32;
                match op {
                    0 => {
                        (*w.pt).parts_num = dry(&mut (*w.jt)[0]);
                        (*GO).hsd_obj = &mut (*w.jt)[0] as *mut HSD_JObj as *mut c_void;
                        f.parts = (*w.parts).as_mut_ptr();
                        f.dobj_list.data = (*w.dl).as_mut_ptr();
                        f.dobj_list.count = 99;
                        m::ftParts_SetupParts(go());
                        for i in 0..20 { hbone(&(*w.parts)[i]); }
                        for i in 0..w.nnd { hm(ep((*w.dl)[i])); }
                        hm(f.dobj_list.count);
                    }
                    1 => {
                        (*w.pt).parts_num = dry(&mut (*w.ja)[0]);
                        f.x8AC_animSkeleton = &mut (*w.ja)[0];
                        f.parts = (*w.parts).as_mut_ptr();
                        m::ftParts_8007462C(go());
                        for i in 0..20 { hbone(&(*w.parts)[i]); }
                    }
                    2 => {
                        vis_world();
                        m::ftParts_800749CC(go());
                        hvis();
                    }
                    3 => {
                        vis_world();
                        let a = rnd() % 5;
                        m::ftParts_800750C8(F, a, rnd() & 1 != 0);
                        hvis();
                    }
                    4 => {
                        vis_world();
                        let mut total = 0usize;
                        for i in 0..w.nnd {
                            (*w.dp)[i].mobj = null_mut();
                            if rnd() % 3 != 0 {
                                let mut prev: *mut HSD_TObj = null_mut();
                                let cnt = (rnd() % 3) as usize;
                                (*w.dp)[i].mobj = &mut (*w.mo)[i];
                                (*w.mo)[i].tobj = null_mut();
                                for j in (0..cnt).rev() {
                                    (*w.to)[i * 3 + j].next = prev;
                                    prev = &mut (*w.to)[i * 3 + j];
                                }
                                (*w.mo)[i].tobj = prev;
                            }
                        }
                        for i in 0..w.nnd {
                            (*w.dl)[i] = if rnd() % 5 == 0 { null_mut() } else { &mut (*w.dp)[i] };
                        }
                        for i in 0..w.nnd {
                            let d = (*w.dl)[i];
                            if !d.is_null() && !(*d).mobj.is_null() {
                                let mut tt = (*(*d).mobj).tobj;
                                while !tt.is_null() { total += 1; tt = (*tt).next; }
                            }
                        }
                        if total != 0 {
                            let n = rnd() % total as u32;
                            ret = ep(m::ftParts_80075240(&mut f.dobj_list, n as i32));
                        }
                    }
                    5 => {
                        vis_world();
                        let idx = (rnd() % 5) as i32;
                        let l: *const DObjList = if rnd() & 1 != 0 { &f.dobj_list } else { &f.x203C };
                        let vis: *mut FtPartsVis = &mut f.x5AC;
                        m::ftParts_80074B6C(&*F, &mut *vis, idx, l);
                        hvis();
                    }
                    6 => {
                        vis_world();
                        let idx = (rnd() % 5) as i32;
                        let l: *const DObjList = if rnd() & 1 != 0 { &f.dobj_list } else { &f.x203C };
                        m::ftParts_80074CA0(&mut f.x5AC, idx, l);
                        hvis();
                    }
                    7 => {
                        vis_world();
                        let idx = (rnd() % 5) as i32;
                        let l: *const DObjList = if rnd() & 1 != 0 { &f.dobj_list } else { &f.x203C };
                        m::ftParts_80074D7C(&mut f.x5AC, idx, l);
                        hvis();
                    }
                    8 => {
                        (*w.pt).parts_num = 1 + rnd() % 13;
                        for i in 0..64 { (*w.p2j)[i] = (rnd() % (*w.pt).parts_num) as u8; }
                        f.parts = null_mut();
                        f.dobj_list.data = null_mut();
                        m::ftParts_80074E58(f);
                        for i in 0..(*w.pt).parts_num as usize { hbone(&(*w.parts)[i]); }
                        hm(ep(f.parts));
                        hm(ep(f.dobj_list.data));
                    }
                    9 => {
                        let q = Vec4 { x: rw(), y: rw(), z: rw(), w: rw() };
                        let jo = &mut (*w.jt)[0];
                        jo.flags = if rnd() & 1 != 0 { (JOBJ_MTX_INDEP_SRT | JOBJ_USE_QUATERNION) as u32 } else { (JOBJ_MTX_DIRTY | JOBJ_USE_QUATERNION) as u32 };
                        jo.rotate.x = rw();
                        jo.rotate.y = rw();
                        m::ftParts_JObjSetRotation(jo, &q);
                        hm(jo.flags);
                        hm(b(jo.rotate.x)); hm(b(jo.rotate.y)); hm(b(jo.rotate.z)); hm(b(jo.rotate.w));
                    }
                    10 => {
                        let mut di = (rnd() % 6) as i32;
                        let depth = rnd() % 5;
                        let jj = (rnd() % w.nj as u32) as usize;
                        f.parts = (*w.parts).as_mut_ptr();
                        f.dobj_list.data = (*w.dl).as_mut_ptr();
                        melee_fighter::tu::ftparts_render::ftParts_80074194(F, &mut (*w.parts)[3], &mut (*w.jt)[jj], &mut di, depth);
                        hbone(&(*w.parts)[3]);
                        for i in 0..(di.min(128) as usize) { hm(ep((*w.dl)[i])); }
                        hm(di as u32);
                    }
                    11 => {
                        vis_world();
                        let costume = rnd() % 3;
                        melee_fighter::tu::ftparts_render::ftParts_8007487C(w.fdesc, &mut f.x5AC, costume, &mut f.dobj_list, &mut f.x203C);
                        hvis();
                    }
                    _ => unreachable!(),
                }
                c.line(op, k, ret, HH);
            }
        }
    }
    assert_eq!(c.done(), 12 * 1500);
}
