//! Field-by-field differential oracle, 14 scenario families x 256 seeds.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_variables,
    unused_assignments,
    unused_parens,
    static_mut_refs,
    dead_code
)]
#[path = "../src/tu/texp.rs"]
mod m;
use m::*;
use melee_types::*;
pub mod tu {
    pub use melee_runtime::tu::tev;
}
static mut WORDS: std::vec::Vec<u32> = std::vec::Vec::new();
unsafe fn emit(v: u32) {
    WORDS.push(v);
}
unsafe fn memset(
    p: *mut core::ffi::c_void,
    v: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    core::ptr::write_bytes(p.cast::<u8>(), v as u8, n);
    p
}
static mut pool: [HSD_TExp; 64] = unsafe { core::mem::zeroed() };
static mut nodes: [HSD_TExp; 12] = unsafe { core::mem::zeroed() };
static mut descs: [HSD_TExpTevDesc; 4] = unsafe { core::mem::zeroed() };
static mut textures: [HSD_TObj; 2] = unsafe { core::mem::zeroed() };
static mut values: [[::core::ffi::c_double; 4]; 8] = [[0.; 4]; 8];
static mut used: ::core::ffi::c_int = 0;
static mut rs: u32 = 0;
unsafe fn rnd() -> u32 {
    rs ^= rs << 13 as ::core::ffi::c_int;
    rs ^= rs >> 17 as ::core::ffi::c_int;
    rs ^= rs << 5 as ::core::ffi::c_int;
    return rs;
}
unsafe fn PI(mut p: *const ::core::ffi::c_void) -> u32 {
    let mut a: ::core::ffi::c_ulonglong = p as ::core::ffi::c_ulonglong;
    let mut bases: [*const ::core::ffi::c_void; 5] = [
        &raw mut nodes as *mut HSD_TExp as *const ::core::ffi::c_void,
        &raw mut pool as *mut HSD_TExp as *const ::core::ffi::c_void,
        &raw mut descs as *mut HSD_TExpTevDesc as *const ::core::ffi::c_void,
        &raw mut textures as *mut HSD_TObj as *const ::core::ffi::c_void,
        &raw mut values as *mut [::core::ffi::c_double; 4]
            as *const ::core::ffi::c_void,
    ];
    let mut sizes: [::core::ffi::c_uint; 5] = [
        ::core::mem::size_of::<[HSD_TExp; 12]>() as ::core::ffi::c_uint,
        ::core::mem::size_of::<[HSD_TExp; 64]>() as ::core::ffi::c_uint,
        ::core::mem::size_of::<[HSD_TExpTevDesc; 4]>() as ::core::ffi::c_uint,
        ::core::mem::size_of::<[HSD_TObj; 2]>() as ::core::ffi::c_uint,
        ::core::mem::size_of::<[[::core::ffi::c_double; 4]; 8]>()
            as ::core::ffi::c_uint,
    ];
    let mut i: ::core::ffi::c_int = 0;
    if p.is_null() {
        return 0 as u32;
    }
    if p == HSD_TEXP_TEX as *const ::core::ffi::c_void {
        return 0xffffffff as u32;
    }
    if p == HSD_TEXP_RAS as *const ::core::ffi::c_void {
        return 0xfffffffe as u32;
    }
    i = 0 as ::core::ffi::c_int;
    while i < 5 as ::core::ffi::c_int {
        if a >= bases[i as usize] as ::core::ffi::c_ulonglong
            && a < (bases[i as usize] as ::core::ffi::c_ulonglong)
                .wrapping_add(sizes[i as usize] as ::core::ffi::c_ulonglong)
        {
            return (((i + 1 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as u32)
                .wrapping_add(
                    a.wrapping_sub(bases[i as usize] as ::core::ffi::c_ulonglong)
                        as u32,
                );
        }
        i += 1;
    }
    return 0xffff0000 as u32;
}
pub const HDBG: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe fn dump_HSD_TETev(mut p: *const HSD_TETev) {
    let mut i0: ::core::ffi::c_int = 0;
    let mut i1: ::core::ffi::c_int = 0;
    let mut i2: ::core::ffi::c_int = 0;
    let mut j0: ::core::ffi::c_int = 0;
    let mut j1: ::core::ffi::c_int = 0;
    let mut j2: ::core::ffi::c_int = 0;
    let mut j3: ::core::ffi::c_int = 0;
    emit((*p).r#type as u32);
    emit(PI((*p).next as *const ::core::ffi::c_void));
    emit((*p).c_ref as u32);
    emit((*p).c_dst as u32);
    emit((*p).c_op as u32);
    emit((*p).c_clamp as u32);
    emit((*p).c_bias as u32);
    emit((*p).c_scale as u32);
    emit((*p).c_range as u32);
    emit((*p).a_ref as u32);
    emit((*p).a_dst as u32);
    emit((*p).a_op as u32);
    emit((*p).a_clamp as u32);
    emit((*p).a_bias as u32);
    emit((*p).a_scale as u32);
    emit((*p).tex_swap as u32);
    emit((*p).ras_swap as u32);
    emit((*p).kcsel as u32);
    emit((*p).kasel as u32);
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).c_in[i0 as usize].r#type as u32);
        emit((*p).c_in[i0 as usize].sel as u32);
        emit((*p).c_in[i0 as usize].arg as u32);
        emit(PI((*p).c_in[i0 as usize].exp as *const ::core::ffi::c_void));
        i0 += 1;
    }
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).a_in[i0 as usize].r#type as u32);
        emit((*p).a_in[i0 as usize].sel as u32);
        emit((*p).a_in[i0 as usize].arg as u32);
        emit(PI((*p).a_in[i0 as usize].exp as *const ::core::ffi::c_void));
        i0 += 1;
    }
    emit(PI((*p).tex as *const ::core::ffi::c_void));
    emit((*p).chan as u32);
}
unsafe fn dump_HSD_TECnst(mut p: *const HSD_TECnst) {
    let mut i0: ::core::ffi::c_int = 0;
    let mut i1: ::core::ffi::c_int = 0;
    let mut i2: ::core::ffi::c_int = 0;
    let mut j0: ::core::ffi::c_int = 0;
    let mut j1: ::core::ffi::c_int = 0;
    let mut j2: ::core::ffi::c_int = 0;
    let mut j3: ::core::ffi::c_int = 0;
    emit((*p).r#type as u32);
    emit(PI((*p).next as *const ::core::ffi::c_void));
    emit(PI((*p).val as *const ::core::ffi::c_void));
    emit((*p).comp as u32);
    emit((*p).ctype as u32);
    emit((*p).reg as u32);
    emit((*p).idx as u32);
    emit((*p).r#ref as u32);
    emit((*p).range as u32);
}
unsafe fn dump_HSD_TExpRes(mut p: *const HSD_TExpRes) {
    let mut i0: ::core::ffi::c_int = 0;
    let mut i1: ::core::ffi::c_int = 0;
    let mut i2: ::core::ffi::c_int = 0;
    let mut j0: ::core::ffi::c_int = 0;
    let mut j1: ::core::ffi::c_int = 0;
    let mut j2: ::core::ffi::c_int = 0;
    let mut j3: ::core::ffi::c_int = 0;
    emit((*p).failed as u32);
    emit((*p).texmap as u32);
    emit((*p).cnst_remain as u32);
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 8 as ::core::ffi::c_int {
        emit((*p).reg[i0 as usize].color as u32);
        emit((*p).reg[i0 as usize].alpha as u32);
        i0 += 1;
    }
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).c_ref[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).a_ref[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).c_use[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as ::core::ffi::c_int;
    while i0 < 4 as ::core::ffi::c_int {
        emit((*p).a_use[i0 as usize] as u32);
        i0 += 1;
    }
}
unsafe fn dump_HSD_TevConf(mut p: *const HSD_TevConf) {
    let mut i0: ::core::ffi::c_int = 0;
    let mut i1: ::core::ffi::c_int = 0;
    let mut i2: ::core::ffi::c_int = 0;
    let mut j0: ::core::ffi::c_int = 0;
    let mut j1: ::core::ffi::c_int = 0;
    let mut j2: ::core::ffi::c_int = 0;
    let mut j3: ::core::ffi::c_int = 0;
    emit((*p).clr_op as u32);
    emit((*p).clr_a as u32);
    emit((*p).clr_b as u32);
    emit((*p).clr_c as u32);
    emit((*p).clr_d as u32);
    emit((*p).clr_scale as u32);
    emit((*p).clr_bias as u32);
    emit((*p).clr_clamp as u32);
    emit((*p).clr_out_reg as u32);
    emit((*p).alpha_op as u32);
    emit((*p).alpha_a as u32);
    emit((*p).alpha_b as u32);
    emit((*p).alpha_c as u32);
    emit((*p).alpha_d as u32);
    emit((*p).alpha_scale as u32);
    emit((*p).alpha_bias as u32);
    emit((*p).alpha_clamp as u32);
    emit((*p).alpha_out_reg as u32);
    emit((*p).mode as u32);
    emit((*p).ras_swap as u32);
    emit((*p).tex_swap as u32);
    emit((*p).kcsel as u32);
    emit((*p).kasel as u32);
}
unsafe fn dump_desc(mut p: *mut HSD_TExpTevDesc) {
    emit(PI((*p).desc.next as *const ::core::ffi::c_void));
    emit((*p).desc.flags);
    emit((*p).desc.stage);
    emit((*p).desc.coord);
    emit((*p).desc.map);
    emit((*p).desc.color);
    dump_HSD_TevConf(&raw mut (*p).desc.u.tevconf);
    emit(PI((*p).tobj as *const ::core::ffi::c_void));
}

pub unsafe fn stub_hsdAllocMemPiece(mut n: s32) -> *mut ::core::ffi::c_void {
    emit(100 as u32);
    emit(n as u32);
    let fresh0 = used;
    used = used + 1;
    return (&raw mut pool as *mut HSD_TExp).offset(fresh0 as isize)
        as *mut HSD_TExp as *mut ::core::ffi::c_void;
}
pub unsafe fn stub_hsdFreeMemPiece(
    mut p: *mut ::core::ffi::c_void,
    mut n: s32,
) {
    emit(101 as u32);
    emit(PI(p));
    emit(n as u32);
}
pub unsafe fn stub_GXPixModeSync() {
    emit(102 as u32);
}
pub unsafe fn stub_GXSetTevColor(mut id: GXTevRegID, mut c: GXColor) {
    emit(103 as u32);
    emit(id as u32);
    emit(c.r as u32);
    emit(c.g as u32);
    emit(c.b as u32);
    emit(c.a as u32);
}
pub unsafe fn stub_GXSetTevKColor(mut id: GXTevKColorID, mut c: GXColor) {
    emit(104 as u32);
    emit(id as u32);
    emit(c.r as u32);
    emit(c.g as u32);
    emit(c.b as u32);
    emit(c.a as u32);
}
pub unsafe fn stub_HSD_StateInvalidate(mut mask: s32) {
    emit(105 as u32);
    emit(mask as u32);
}
pub unsafe fn stub_HSD_StateAssignTev() -> ::core::ffi::c_int {
    emit(106 as u32);
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn stub_HSD_SetupTevStage(mut p: *mut HSD_TevDesc) {
    emit(107 as u32);
    dump_desc(p as *mut HSD_TExpTevDesc);
}
pub unsafe fn stub_HSD_Index2TevStage(
    mut i: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    emit(108 as u32);
    emit(i as u32);
    return i;
}
pub unsafe fn stub_HSD_TExpSimplify(
    mut p: *mut HSD_TExp,
) -> ::core::ffi::c_int {
    emit(109 as u32);
    emit(PI(p as *const ::core::ffi::c_void));
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn stub_HSD_TExpSimplify2(
    mut p: *mut HSD_TExp,
) -> ::core::ffi::c_int {
    emit(110 as u32);
    emit(PI(p as *const ::core::ffi::c_void));
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn stub_HSD_TExpMakeDag(
    mut p: *mut HSD_TExp,
    mut d: *mut HSD_TExpDag,
) -> ::core::ffi::c_int {
    emit(111 as u32);
    emit(PI(p as *const ::core::ffi::c_void));
    let ref mut fresh2 = (*d.offset(0 as ::core::ffi::c_int as isize)).tev;
    *fresh2 = &raw mut (*p).tev;
    return 1 as ::core::ffi::c_int;
}
pub unsafe fn stub_HSD_TExpSchedule(
    mut n: ::core::ffi::c_int,
    mut d: *mut HSD_TExpDag,
    mut order: *mut *mut HSD_TExp,
    mut r: *mut HSD_TExpRes,
) {
    emit(112 as u32);
    emit(n as u32);
    dump_HSD_TExpRes(r);
    let ref mut fresh1 = *order.offset(0 as ::core::ffi::c_int as isize);
    *fresh1 =
        (*d.offset(0 as ::core::ffi::c_int as isize)).tev as *mut HSD_TExp;
    (**order.offset(0 as ::core::ffi::c_int as isize)).tev.c_dst = 3 as u8;
    (**order.offset(0 as ::core::ffi::c_int as isize)).tev.a_dst = 3 as u8;
}
unsafe fn reset(mut seed: u32) {
    rs = seed;
    used = 0 as ::core::ffi::c_int;
    memset(
        &raw mut nodes as *mut HSD_TExp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[HSD_TExp; 12]>() as usize,
    );
    memset(
        &raw mut pool as *mut HSD_TExp as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[HSD_TExp; 64]>() as usize,
    );
    memset(
        &raw mut descs as *mut HSD_TExpTevDesc as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[HSD_TExpTevDesc; 4]>() as usize,
    );
    memset(
        &raw mut textures as *mut HSD_TObj as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[HSD_TObj; 2]>() as usize,
    );
    memset(
        &raw mut values as *mut [::core::ffi::c_double; 4]
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[[::core::ffi::c_double; 4]; 8]>() as usize,
    );
}
unsafe fn init_tev(mut p: *mut HSD_TExp) {
    let mut i: ::core::ffi::c_int = 0;
    memset(
        p as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HSD_TExp>() as usize,
    );
    (*p).r#type = HSD_TE_TEV;
    (*p).tev.c_op = rnd().wrapping_rem(16 as u32) as u8;
    (*p).tev.a_op = rnd().wrapping_rem(16 as u32) as u8;
    (*p).tev.c_ref = rnd().wrapping_rem(3 as u32) as s32;
    (*p).tev.a_ref = rnd().wrapping_rem(3 as u32) as s32;
    (*p).tev.c_clamp = 1 as u8;
    (*p).tev.a_clamp = 1 as u8;
    (*p).tev.c_dst = rnd().wrapping_rem(4 as u32) as u8;
    (*p).tev.a_dst = rnd().wrapping_rem(4 as u32) as u8;
    (*p).tev.c_bias = rnd().wrapping_rem(3 as u32) as u8;
    (*p).tev.a_bias = rnd().wrapping_rem(3 as u32) as u8;
    (*p).tev.c_scale = rnd().wrapping_rem(4 as u32) as u8;
    (*p).tev.a_scale = rnd().wrapping_rem(4 as u32) as u8;
    (*p).tev.kasel = 255 as u8;
    (*p).tev.kcsel = (*p).tev.kasel;
    (*p).tev.ras_swap = (*p).tev.kcsel;
    (*p).tev.tex_swap = (*p).tev.ras_swap;
    (*p).tev.chan = 255 as u8;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*p).tev.c_in[i as usize].sel = 7 as u8;
        (*p).tev.a_in[i as usize].sel = 7 as u8;
        (*p).tev.c_in[i as usize].arg = 15 as u8;
        (*p).tev.a_in[i as usize].arg = 7 as u8;
        i += 1;
    }
}
unsafe fn init_cnst(mut p: *mut HSD_TExp, mut scalar: ::core::ffi::c_int) {
    memset(
        p as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HSD_TExp>() as usize,
    );
    (*p).r#type = HSD_TE_CNST;
    (*p).cnst.comp = (if scalar != 0 {
        HSD_TE_X as ::core::ffi::c_int
    } else {
        HSD_TE_RGB as ::core::ffi::c_int
    }) as HSD_TEInput;
    (*p).cnst.ctype = rnd().wrapping_rem(5 as u32) as HSD_TEType;
    (*p).cnst.reg = 255 as u8;
    (*p).cnst.idx = 255 as u8;
    (*p).cnst.r#ref = rnd().wrapping_rem(5 as u32) as u8;
    (*p).cnst.val = &raw mut values as *mut [::core::ffi::c_double; 4]
        as *mut ::core::ffi::c_void;
}
unsafe fn snapshot() {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 12 as ::core::ffi::c_int {
        if nodes[i as usize].r#type as ::core::ffi::c_int
            == HSD_TE_TEV as ::core::ffi::c_int
        {
            dump_HSD_TETev(
                &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                    .offset(i as isize))
                .tev,
            );
        } else if nodes[i as usize].r#type as ::core::ffi::c_int
            == HSD_TE_CNST as ::core::ffi::c_int
        {
            dump_HSD_TECnst(
                &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                    .offset(i as isize))
                .cnst,
            );
        } else {
            emit(nodes[i as usize].r#type as u32);
        }
        i += 1;
    }
    emit(used as u32);
    i = 0 as ::core::ffi::c_int;
    while i < used {
        if pool[i as usize].r#type as ::core::ffi::c_int
            == HSD_TE_TEV as ::core::ffi::c_int
        {
            dump_HSD_TETev(
                &raw mut (*(&raw mut pool as *mut HSD_TExp)
                    .offset(i as isize))
                .tev,
            );
        } else if pool[i as usize].r#type as ::core::ffi::c_int
            == HSD_TE_CNST as ::core::ffi::c_int
        {
            dump_HSD_TECnst(
                &raw mut (*(&raw mut pool as *mut HSD_TExp)
                    .offset(i as isize))
                .cnst,
            );
        } else {
            emit(pool[i as usize].r#type as u32);
        }
        i += 1;
    }
}
pub unsafe fn run_case(
    mut op: ::core::ffi::c_int,
    mut seed: ::core::ffi::c_int,
) {
    let mut list: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    let mut p: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    let mut q: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    let mut res: HSD_TExpRes = HSD_TExpRes {
        failed: 0,
        texmap: 0,
        cnst_remain: 0,
        reg: [HSD_TExpRes_reg { color: 0, alpha: 0 }; 8],
        c_ref: [0; 4],
        a_ref: [0; 4],
        c_use: [0; 4],
        a_use: [0; 4],
    };
    let mut out: *mut HSD_TExpTevDesc =
        ::core::ptr::null_mut::<HSD_TExpTevDesc>();
    let mut i: ::core::ffi::c_int = 0;
    let mut j: ::core::ffi::c_int = 0;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cp: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut ap: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut sel: u32 = 0;
    reset(seed as u32);
    memset(
        &raw mut res as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HSD_TExpRes>() as usize,
    );
    emit(op as u32);
    emit(seed as u32);
    init_tev(
        (&raw mut nodes as *mut HSD_TExp)
            .offset(0 as ::core::ffi::c_int as isize) as *mut HSD_TExp,
    );
    init_tev(
        (&raw mut nodes as *mut HSD_TExp)
            .offset(1 as ::core::ffi::c_int as isize) as *mut HSD_TExp,
    );
    init_cnst(
        (&raw mut nodes as *mut HSD_TExp)
            .offset(2 as ::core::ffi::c_int as isize) as *mut HSD_TExp,
        seed & 1 as ::core::ffi::c_int,
    );
    match op {
        0 => {
            if seed % 5 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.a_ref =
                    0x7fffffff as ::core::ffi::c_int as s32;
                nodes[0 as ::core::ffi::c_int as usize].tev.c_ref =
                    nodes[0 as ::core::ffi::c_int as usize].tev.a_ref;
            }
            if seed % 7 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[1 as ::core::ffi::c_int as usize].tev.a_ref =
                    (-(2147483647 as ::core::ffi::c_int)
                        - 1 as ::core::ffi::c_int) as s32;
                nodes[1 as ::core::ffi::c_int as usize].tev.c_ref =
                    nodes[1 as ::core::ffi::c_int as usize].tev.a_ref;
            }
            nodes[2 as ::core::ffi::c_int as usize].cnst.r#ref = 255 as u8;
            emit(HSD_TExpGetType(::core::ptr::null_mut::<HSD_TExp>()) as u32);
            emit(HSD_TExpGetType(HSD_TEXP_TEX) as u32);
            emit(HSD_TExpGetType(HSD_TEXP_RAS) as u32);
            i = 0 as ::core::ffi::c_int;
            while i < 3 as ::core::ffi::c_int {
                HSD_TExpRef(
                    (&raw mut nodes as *mut HSD_TExp).offset(i as isize)
                        as *mut HSD_TExp,
                    (seed % 7 as ::core::ffi::c_int) as u8,
                );
                emit(HSD_TExpGetType(
                    (&raw mut nodes as *mut HSD_TExp).offset(i as isize)
                        as *mut HSD_TExp,
                ) as u32);
                HSD_TExpUnref(
                    (&raw mut nodes as *mut HSD_TExp).offset(i as isize)
                        as *mut HSD_TExp,
                    (seed % 7 as ::core::ffi::c_int) as u8,
                );
                i += 1;
            }
            nodes[0 as ::core::ffi::c_int as usize].tev.c_ref =
                1 as ::core::ffi::c_int as s32;
            nodes[0 as ::core::ffi::c_int as usize].tev.a_ref =
                0 as ::core::ffi::c_int as s32;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                [0 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(1 as ::core::ffi::c_int as isize)
                as *mut HSD_TExp;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                [0 as ::core::ffi::c_int as usize]
                .sel = 1 as u8;
            nodes[0 as ::core::ffi::c_int as usize].tev.a_in
                [0 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(2 as ::core::ffi::c_int as isize)
                as *mut HSD_TExp;
            HSD_TExpUnref(&raw mut nodes as *mut HSD_TExp, 1 as u8);
            HSD_TExpUnref(::core::ptr::null_mut::<HSD_TExp>(), 0 as u8);
        }
        1 => {
            p = HSD_TExpTev(&raw mut list);
            q = HSD_TExpCnst(
                &raw mut values as *mut [::core::ffi::c_double; 4]
                    as *mut ::core::ffi::c_void,
                HSD_TE_RGB,
                HSD_TE_F64,
                &raw mut list,
            );
            emit(PI(HSD_TExpCnst(
                &raw mut values as *mut [::core::ffi::c_double; 4]
                    as *mut ::core::ffi::c_void,
                HSD_TE_RGB,
                HSD_TE_F64,
                &raw mut list,
            ) as *const ::core::ffi::c_void));
            emit(PI(HSD_TExpCnst(
                &raw mut values as *mut [::core::ffi::c_double; 4]
                    as *mut ::core::ffi::c_void,
                HSD_TE_0,
                HSD_TE_U8,
                &raw mut list,
            ) as *const ::core::ffi::c_void));
            (*p).tev.c_ref = (seed % 2 as ::core::ffi::c_int) as s32;
            (*q).cnst.r#ref = ((seed >> 1 as ::core::ffi::c_int)
                % 2 as ::core::ffi::c_int) as u8;
            list = HSD_TExpFreeList(
                list,
                (if seed % 2 as ::core::ffi::c_int != 0 {
                    HSD_TE_ALL as ::core::ffi::c_int
                } else {
                    HSD_TE_CNST as ::core::ffi::c_int
                }) as HSD_TExpType,
                (seed as s32 >> 2 as ::core::ffi::c_int) % 2 as s32,
            );
            emit(PI(list as *const ::core::ffi::c_void));
            HSD_TExpFree(HSD_TEXP_TEX);
        }
        2 => {
            HSD_TExpColorOp(
                &raw mut nodes as *mut HSD_TExp,
                (seed % 16 as ::core::ffi::c_int) as GXTevOp,
                (seed / 2 as ::core::ffi::c_int % 3 as ::core::ffi::c_int)
                    as GXTevBias,
                (seed / 3 as ::core::ffi::c_int % 4 as ::core::ffi::c_int)
                    as GXTevScale,
                (seed % 3 as ::core::ffi::c_int) as u8,
            );
            HSD_TExpAlphaOp(
                &raw mut nodes as *mut HSD_TExp,
                (seed / 3 as ::core::ffi::c_int % 16 as ::core::ffi::c_int)
                    as GXTevOp,
                (seed / 5 as ::core::ffi::c_int % 3 as ::core::ffi::c_int)
                    as GXTevBias,
                (seed / 7 as ::core::ffi::c_int % 4 as ::core::ffi::c_int)
                    as GXTevScale,
                (seed / 2 as ::core::ffi::c_int % 3 as ::core::ffi::c_int)
                    as u8,
            );
        }
        3 => {
            sel = (1 as ::core::ffi::c_int + seed % 15 as ::core::ffi::c_int)
                as u32;
            i = seed % 4 as ::core::ffi::c_int;
            p = if sel <= 5 as u32 {
                if seed % 2 as ::core::ffi::c_int != 0 {
                    HSD_TEXP_TEX
                } else {
                    HSD_TEXP_RAS
                }
            } else {
                ::core::ptr::null_mut::<HSD_TExp>()
            };
            HSD_TExpColorInSub(
                &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                    .offset(0 as ::core::ffi::c_int as isize))
                .tev,
                sel as HSD_TEInput,
                p,
                i as s32,
            );
            HSD_TExpAlphaInSub(
                &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                    .offset(0 as ::core::ffi::c_int as isize))
                .tev,
                (if sel >= 7 as u32 {
                    sel
                } else {
                    HSD_TE_A as ::core::ffi::c_int as u32
                }) as HSD_TEInput,
                if sel >= 7 as u32 {
                    ::core::ptr::null_mut::<HSD_TExp>()
                } else {
                    p
                },
                i as s32,
            );
        }
        4 => {
            nodes[2 as ::core::ffi::c_int as usize].cnst.comp = HSD_TE_X;
            HSD_TExpColorIn(
                &raw mut nodes as *mut HSD_TExp,
                HSD_TE_RGB,
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut HSD_TExp,
                HSD_TE_A,
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut HSD_TExp,
                HSD_TE_X,
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(2 as ::core::ffi::c_int as isize)
                    as *mut HSD_TExp,
                HSD_TE_0,
                ::core::ptr::null_mut::<HSD_TExp>(),
            );
            HSD_TExpAlphaIn(
                &raw mut nodes as *mut HSD_TExp,
                HSD_TE_A,
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *mut HSD_TExp,
                HSD_TE_X,
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(2 as ::core::ffi::c_int as isize)
                    as *mut HSD_TExp,
                HSD_TE_A,
                HSD_TEXP_TEX,
                HSD_TE_A,
                HSD_TEXP_RAS,
            );
        }
        5 => {
            HSD_TExpOrder(
                &raw mut nodes as *mut HSD_TExp,
                if seed % 2 as ::core::ffi::c_int != 0 {
                    &raw mut textures as *mut HSD_TObj
                } else {
                    ::core::ptr::null_mut::<HSD_TObj>()
                },
                (seed % 9 as ::core::ffi::c_int) as GXChannelID,
            );
        }
        6 => {
            i = 0 as ::core::ffi::c_int;
            while i < 8 as ::core::ffi::c_int {
                res.reg[i as usize].color = rnd().wrapping_rem(4 as u32) as u8;
                res.reg[i as usize].alpha = rnd().wrapping_rem(2 as u32) as u8;
                i += 1;
            }
            nodes[2 as ::core::ffi::c_int as usize].cnst.reg =
                (if seed % 3 as ::core::ffi::c_int != 0 {
                    255 as ::core::ffi::c_int
                } else {
                    seed / 3 as ::core::ffi::c_int % 8 as ::core::ffi::c_int
                }) as u8;
            nodes[2 as ::core::ffi::c_int as usize].cnst.idx =
                (seed % 4 as ::core::ffi::c_int) as u8;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                [0 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(2 as ::core::ffi::c_int as isize);
            nodes[0 as ::core::ffi::c_int as usize].tev.a_in
                [0 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(2 as ::core::ffi::c_int as isize);
            match seed % 4 as ::core::ffi::c_int {
                0 => {
                    r = AssignColorReg(
                        &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                            .offset(0 as ::core::ffi::c_int as isize))
                        .tev,
                        0 as ::core::ffi::c_int,
                        &raw mut res,
                    );
                }
                1 => {
                    r = AssignAlphaReg(
                        &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                            .offset(0 as ::core::ffi::c_int as isize))
                        .tev,
                        0 as ::core::ffi::c_int,
                        &raw mut res,
                    );
                }
                2 => {
                    r = AssignColorKonst(
                        &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                            .offset(0 as ::core::ffi::c_int as isize))
                        .tev,
                        0 as ::core::ffi::c_int,
                        &raw mut res,
                    );
                }
                3 => {
                    r = AssignAlphaKonst(
                        &raw mut (*(&raw mut nodes as *mut HSD_TExp)
                            .offset(0 as ::core::ffi::c_int as isize))
                        .tev,
                        0 as ::core::ffi::c_int,
                        &raw mut res,
                    );
                }
                _ => {}
            }
            dump_HSD_TExpRes(&raw mut res);
        }
        7 => {
            nodes[0 as ::core::ffi::c_int as usize].tev.a_op = 0 as u8;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_op =
                nodes[0 as ::core::ffi::c_int as usize].tev.a_op;
            nodes[0 as ::core::ffi::c_int as usize].tev.a_scale = 0 as u8;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_scale =
                nodes[0 as ::core::ffi::c_int as usize].tev.a_scale;
            nodes[0 as ::core::ffi::c_int as usize].tev.a_bias =
                nodes[0 as ::core::ffi::c_int as usize].tev.c_scale;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_bias =
                nodes[0 as ::core::ffi::c_int as usize].tev.a_bias;
            nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                [3 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(2 as ::core::ffi::c_int as isize);
            nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                [3 as ::core::ffi::c_int as usize]
                .r#type = 4 as u8;
            init_cnst(
                (&raw mut nodes as *mut HSD_TExp)
                    .offset(3 as ::core::ffi::c_int as isize),
                1 as ::core::ffi::c_int,
            );
            nodes[0 as ::core::ffi::c_int as usize].tev.a_in
                [3 as ::core::ffi::c_int as usize]
                .exp = (&raw mut nodes as *mut HSD_TExp)
                .offset(3 as ::core::ffi::c_int as isize);
            nodes[0 as ::core::ffi::c_int as usize].tev.a_in
                [3 as ::core::ffi::c_int as usize]
                .r#type = 4 as u8;
            if seed % 2 as ::core::ffi::c_int != 0 {
                nodes[0 as ::core::ffi::c_int as usize].tev.kasel = 0 as u8;
                nodes[0 as ::core::ffi::c_int as usize].tev.kcsel =
                    nodes[0 as ::core::ffi::c_int as usize].tev.kasel;
            }
            if seed % 3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.c_in
                    [0 as ::core::ffi::c_int as usize]
                    .sel = 0 as u8;
                nodes[0 as ::core::ffi::c_int as usize].tev.a_in
                    [0 as ::core::ffi::c_int as usize]
                    .sel = 0 as u8;
            }
            r = TExpAssignReg(&raw mut nodes as *mut HSD_TExp, &raw mut res);
            dump_HSD_TExpRes(&raw mut res);
            emit(IsThroughColor(&raw mut nodes as *mut HSD_TExp) as u32);
            emit(IsThroughAlpha(&raw mut nodes as *mut HSD_TExp) as u32);
        }
        8 => {
            cp = seed % 2 as ::core::ffi::c_int;
            ap = (seed >> 1 as ::core::ffi::c_int) % 2 as ::core::ffi::c_int;
            if seed % 4 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.c_op = 255 as u8;
            }
            if seed % 5 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.a_op = 255 as u8;
            }
            if seed % 3 as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.tex =
                    &raw mut textures as *mut HSD_TObj;
            }
            i = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                nodes[0 as ::core::ffi::c_int as usize].tev.c_in[i as usize]
                    .arg = (if (seed + i) % 3 as ::core::ffi::c_int != 0 {
                    rnd().wrapping_rem(16 as u32)
                } else {
                    255 as u32
                }) as u8;
                nodes[0 as ::core::ffi::c_int as usize].tev.a_in[i as usize]
                    .arg = (if (seed + i) % 4 as ::core::ffi::c_int != 0 {
                    rnd().wrapping_rem(8 as u32)
                } else {
                    255 as u32
                }) as u8;
                i += 1;
            }
            TExp2TevDesc(
                &raw mut nodes as *mut HSD_TExp,
                &raw mut descs as *mut HSD_TExpTevDesc,
                &raw mut cp,
                &raw mut ap,
            );
            dump_desc(&raw mut descs as *mut HSD_TExpTevDesc);
            emit(cp as u32);
            emit(ap as u32);
        }
        9 => {
            i = 0 as ::core::ffi::c_int;
            while i < 8 as ::core::ffi::c_int {
                init_cnst(
                    (&raw mut nodes as *mut HSD_TExp).offset(i as isize),
                    ((seed + i) % 3 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int,
                );
                nodes[i as usize].cnst.reg = i as u8;
                nodes[i as usize].cnst.idx =
                    ((seed + i) % 4 as ::core::ffi::c_int) as u8;
                nodes[i as usize].cnst.ctype =
                    ((seed + i) % 5 as ::core::ffi::c_int) as HSD_TEType;
                nodes[i as usize].cnst.next = if i == 7 as ::core::ffi::c_int {
                    ::core::ptr::null_mut::<HSD_TExp>()
                } else {
                    (&raw mut nodes as *mut HSD_TExp)
                        .offset(i as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                };
                nodes[i as usize].cnst.val = &raw mut *(&raw mut values
                    as *mut [::core::ffi::c_double; 4])
                    .offset(i as isize)
                    as *mut ::core::ffi::c_double
                    as *mut ::core::ffi::c_void;
                j = 0 as ::core::ffi::c_int;
                while j < 4 as ::core::ffi::c_int {
                    let mut v: u32 = rnd();
                    match nodes[i as usize].cnst.ctype as ::core::ffi::c_int {
                        0 => {
                            *(&raw mut *(&raw mut values
                                as *mut [::core::ffi::c_double; 4])
                                .offset(i as isize)
                                as *mut ::core::ffi::c_double
                                as *mut u8)
                                .offset(j as isize) = v as u8;
                        }
                        1 => {
                            *(&raw mut *(&raw mut values
                                as *mut [::core::ffi::c_double; 4])
                                .offset(i as isize)
                                as *mut ::core::ffi::c_double
                                as *mut u16)
                                .offset(j as isize) = v as u16;
                        }
                        2 => {
                            *(&raw mut *(&raw mut values
                                as *mut [::core::ffi::c_double; 4])
                                .offset(i as isize)
                                as *mut ::core::ffi::c_double
                                as *mut u32)
                                .offset(j as isize) = v;
                        }
                        3 => {
                            let mut bits: [u32; 8] = [
                                0x7fc00000 as ::core::ffi::c_int as u32,
                                0x7f800000 as ::core::ffi::c_int as u32,
                                0xff800000 as ::core::ffi::c_uint,
                                0x80000000 as ::core::ffi::c_uint,
                                0 as ::core::ffi::c_int as u32,
                                0x3f800000 as ::core::ffi::c_int as u32,
                                0x3b808081 as ::core::ffi::c_int as u32,
                                v,
                            ];
                            *(&raw mut *(&raw mut values
                                as *mut [::core::ffi::c_double; 4])
                                .offset(i as isize)
                                as *mut ::core::ffi::c_double
                                as *mut f32)
                                .offset(j as isize) =
                                *((&raw mut bits as *mut u32).offset(
                                    ((seed + j) % 8 as ::core::ffi::c_int)
                                        as isize,
                                ) as *mut u32
                                    as *mut f32);
                        }
                        _ => {
                            values[i as usize][j as usize] = (v
                                .wrapping_rem(1024 as u32)
                                as ::core::ffi::c_int
                                - 512 as ::core::ffi::c_int)
                                as ::core::ffi::c_double
                                / 255.0f64;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
            HSD_TExpSetReg(&raw mut nodes as *mut HSD_TExp);
            HSD_TExpSetReg(::core::ptr::null_mut::<HSD_TExp>());
        }
        10 => {
            textures[0 as ::core::ffi::c_int as usize].id =
                (seed % 8 as ::core::ffi::c_int) as GXTexMapID;
            textures[0 as ::core::ffi::c_int as usize].coord =
                (seed % 9 as ::core::ffi::c_int) as GXTexCoordID;
            descs[0 as ::core::ffi::c_int as usize].desc.next =
                &raw mut (*(&raw mut descs as *mut HSD_TExpTevDesc)
                    .offset(1 as ::core::ffi::c_int as isize))
                .desc;
            descs[0 as ::core::ffi::c_int as usize].tobj =
                &raw mut textures as *mut HSD_TObj;
            HSD_TExpSetupTev(
                &raw mut descs as *mut HSD_TExpTevDesc,
                ::core::ptr::null_mut::<HSD_TExp>(),
            );
            dump_desc(&raw mut descs as *mut HSD_TExpTevDesc);
            dump_desc(
                (&raw mut descs as *mut HSD_TExpTevDesc)
                    .offset(1 as ::core::ffi::c_int as isize),
            );
        }
        11 => {
            list = HSD_TExpTev(&raw mut list);
            HSD_TExpColorOp(
                list,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                1 as u8,
            );
            HSD_TExpAlphaOp(
                list,
                GX_TEV_ADD,
                GX_TB_ZERO,
                GX_CS_SCALE_1,
                1 as u8,
            );
            r = HSD_TExpCompile(list, &raw mut out, &raw mut list);
            emit(PI(list as *const ::core::ffi::c_void));
            emit(PI(out as *const ::core::ffi::c_void));
            dump_desc(out);
            HSD_TExpFreeTevDesc(out);
        }
        12 => {
            descs[0 as ::core::ffi::c_int as usize].desc.next =
                &raw mut (*(&raw mut descs as *mut HSD_TExpTevDesc)
                    .offset(1 as ::core::ffi::c_int as isize))
                .desc;
            HSD_TExpFreeTevDesc(&raw mut descs as *mut HSD_TExpTevDesc);
            HSD_TExpFreeTevDesc(::core::ptr::null_mut::<HSD_TExpTevDesc>());
        }
        13 => {
            p = TevAlloc();
            (*p).r#type = HSD_TE_TEV;
            q = CnstAlloc();
            (*q).r#type = HSD_TE_CNST;
            HSD_TExpFree(p);
            HSD_TExpFree(q);
        }
        _ => {}
    }
    emit(r as u32);
    snapshot();
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;

#[test]
fn every_texp_function_matches_native_c_fields_and_transcript() {
    unsafe {
        m::ext::hook::hsdAllocMemPiece = Some(stub_hsdAllocMemPiece);
        m::ext::hook::hsdFreeMemPiece = Some(stub_hsdFreeMemPiece);
        m::ext::hook::GXPixModeSync = Some(stub_GXPixModeSync);
        m::ext::hook::GXSetTevColor = Some(stub_GXSetTevColor);
        m::ext::hook::GXSetTevKColor = Some(stub_GXSetTevKColor);
        m::ext::hook::HSD_StateInvalidate = Some(stub_HSD_StateInvalidate);
        m::ext::hook::HSD_StateAssignTev = Some(stub_HSD_StateAssignTev);
        m::ext::hook::HSD_SetupTevStage = Some(stub_HSD_SetupTevStage);
        m::ext::hook::HSD_Index2TevStage = Some(stub_HSD_Index2TevStage);
        m::ext::hook::HSD_TExpMakeDag = Some(stub_HSD_TExpMakeDag);
        m::ext::hook::HSD_TExpSchedule = Some(stub_HSD_TExpSchedule);
        m::ext::hook::HSD_TExpSimplify = Some(stub_HSD_TExpSimplify);
        m::ext::hook::HSD_TExpSimplify2 = Some(stub_HSD_TExpSimplify2);

        let oracle = include_str!("data/oracle_texp.txt");
        let mut lines = oracle.lines().filter(|s| !s.trim().is_empty());
        for op in 0..14 {
            for seed in 1..=256 {
                WORDS.clear();
                run_case(op, seed);
                let expected: std::vec::Vec<u32> = lines
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .map(|s| u32::from_str_radix(s, 16).unwrap())
                    .collect();
                assert_eq!(
                    WORDS.len(),
                    expected.len(),
                    "op={op}, seed={seed}: word count"
                );
                for (field, (got, want)) in
                    WORDS.iter().zip(expected.iter()).enumerate()
                {
                    assert_eq!(
                        got, want,
                        "op={op}, seed={seed}, field/call word={field}"
                    );
                }
            }
        }
        assert!(lines.next().is_none());
    }
}
