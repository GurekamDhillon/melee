//! Native texture-expression construction, resource assignment and TEV compilation.
//! All 26 definitions from texp.c plus its two header predicates. Arithmetic is
//! integer except SetReg's separately rounded scalar multiplies (zero fmuladd
//! sites in the shipping-flags oracle IR). All records come from melee_types.
#![allow(
    non_snake_case,
    non_upper_case_globals,
    unused_mut,
    unused_assignments,
    unused_parens,
    static_mut_refs,
    dead_code
)]
use ext::*;
use melee_types::*;
pub const HSD_TEXP_TEX: *mut HSD_TExp = -1isize as *mut HSD_TExp;
pub const HSD_TEXP_RAS: *mut HSD_TExp = -2isize as *mut HSD_TExp;
unsafe fn memset(
    p: *mut core::ffi::c_void,
    v: i32,
    n: usize,
) -> *mut core::ffi::c_void {
    core::ptr::write_bytes(p.cast::<u8>(), v as u8, n);
    p
}
unsafe fn __assert(_: *const i8, line: u32, condition: *const i8) -> ! {
    panic!(
        "texp.c:{line}: {}",
        std::ffi::CStr::from_ptr(condition).to_string_lossy()
    )
}
unsafe fn HSD_Panic(_: *const i8, line: u32, condition: *const i8) -> ! {
    __assert(core::ptr::null(), line, condition)
}
pub mod ext {
    use super::*;
    // TODO allocator / DAG bindings. Renderer hooks bind the explicit HsdRender
    // context used by the existing state/tev ports; their logic is not duplicated.
    macro_rules! hooks {
        (@default) => { None };
        (@default $e:expr) => { Some($e) };
        ($(fn $n:ident($($a:ident:$t:ty),*) $(->$r:ty)? $(= $default:expr)?;)*) => {
            pub mod hook { use super::*; $(pub static mut $n: Option<unsafe fn($($t),*) $(->$r)?>=hooks!(@default $($default)?);)* }
            $(pub unsafe fn $n($($a:$t),*) $(->$r)? {
                match hook::$n { Some(f)=>f($($a),*), None=>panic!(concat!("TODO binding: ",stringify!($n))) }
            })*
        }
    }
    hooks! {
        fn hsdAllocMemPiece(size:i32)->*mut core::ffi::c_void;
        fn hsdFreeMemPiece(mem:*mut core::ffi::c_void,size:i32);
        fn GXPixModeSync();
        fn GXSetTevColor(id:GXTevRegID,color:GXColor);
        fn GXSetTevKColor(id:GXTevKColorID,color:GXColor);
        fn HSD_StateInvalidate(mask:i32);
        fn HSD_StateAssignTev()->i32;
        fn HSD_SetupTevStage(desc:*mut HSD_TevDesc);
        fn HSD_Index2TevStage(idx:i32)->i32 = crate::tu::tev::HSD_Index2TevStage;
        fn HSD_TExpMakeDag(root:*mut HSD_TExp,list:*mut HSD_TExpDag)->i32;
        fn HSD_TExpSchedule(num:i32,list:*mut HSD_TExpDag,result:*mut *mut HSD_TExp,res:*mut HSD_TExpRes);
        fn HSD_TExpSimplify(texp:*mut HSD_TExp)->i32;
        fn HSD_TExpSimplify2(texp:*mut HSD_TExp)->i32;
    }
}
#[inline]
pub unsafe fn IsThroughColor(texp: *mut HSD_TExp) -> i32 {
    melee_core::ct!("IsThroughColor");
    melee_types::inline::IsThroughColor(&*texp) as i32
}
#[inline]
pub unsafe fn IsThroughAlpha(texp: *mut HSD_TExp) -> i32 {
    melee_core::ct!("IsThroughAlpha");
    melee_types::inline::IsThroughAlpha(&*texp) as i32
}
pub unsafe fn HSD_TExpGetType(mut texp: *mut HSD_TExp) -> HSD_TExpType {
    melee_core::ct!("HSD_TExpGetType");
    if texp.is_null() {
        return HSD_TE_ZERO;
    }
    if texp == HSD_TEXP_TEX {
        return HSD_TE_TEX;
    }
    if texp == HSD_TEXP_RAS {
        return HSD_TE_RAS;
    }
    return (*texp).r#type;
}
pub unsafe fn TevAlloc() -> *mut HSD_TExp {
    melee_core::ct!("TevAlloc");
    let mut texp: *mut HSD_TExp = hsdAllocMemPiece(::core::mem::size_of::<
        HSD_TETev,
    >() as s32) as *mut HSD_TExp;
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            28 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return texp;
}
pub unsafe fn CnstAlloc() -> *mut HSD_TExp {
    melee_core::ct!("CnstAlloc");
    let mut texp: *mut HSD_TExp = hsdAllocMemPiece(::core::mem::size_of::<
        HSD_TECnst,
    >() as s32) as *mut HSD_TExp;
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            35 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return texp;
}
pub unsafe fn HSD_TExpFree(mut texp: *mut HSD_TExp) {
    melee_core::ct!("HSD_TExpFree");
    match HSD_TExpGetType(texp) as ::core::ffi::c_int {
        1 => {
            hsdFreeMemPiece(
                texp as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HSD_TETev>() as s32,
            );
        }
        4 => {
            hsdFreeMemPiece(
                texp as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HSD_TECnst>() as s32,
            );
        }
        _ => {}
    };
}
pub unsafe fn HSD_TExpRef(mut texp: *mut HSD_TExp, mut sel: u8) {
    melee_core::ct!("HSD_TExpRef");
    let mut r#type: HSD_TExpType = HSD_TExpGetType(texp);
    match r#type as ::core::ffi::c_int {
        1 => {
            if sel as ::core::ffi::c_int == 1 {
                (*texp).tev.c_ref = (*texp).tev.c_ref.wrapping_add(1);
                return;
            } else {
                (*texp).tev.a_ref = (*texp).tev.a_ref.wrapping_add(1);
                return;
            }
        }
        4 => {
            (*texp).cnst.r#ref = ((*texp).cnst.r#ref as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int)
                as u8;
        }
        _ => {}
    };
}
pub unsafe fn HSD_TExpUnref(mut texp: *mut HSD_TExp, mut sel: u8) {
    melee_core::ct!("HSD_TExpUnref");
    let mut i: s32 = 0;
    let mut r#type: HSD_TExpType = HSD_TExpGetType(texp);
    if r#type as ::core::ffi::c_int != HSD_TE_CNST as ::core::ffi::c_int {
        if r#type as ::core::ffi::c_int >= HSD_TE_CNST as ::core::ffi::c_int
            || r#type as ::core::ffi::c_int != HSD_TE_TEV as ::core::ffi::c_int
        {
            return;
        }
        if sel as ::core::ffi::c_int == 1 {
            if (*texp).tev.c_ref != 0 as ::core::ffi::c_int {
                (*texp).tev.c_ref = (*texp).tev.c_ref.wrapping_sub(1);
            }
        } else if (*texp).tev.a_ref != 0 as ::core::ffi::c_int {
            (*texp).tev.a_ref = (*texp).tev.a_ref.wrapping_sub(1);
        }
        if (*texp).tev.c_ref == 0 as ::core::ffi::c_int
            && (*texp).tev.a_ref == 0 as ::core::ffi::c_int
        {
            i = 0 as ::core::ffi::c_int as s32;
            while i < 4 as ::core::ffi::c_int {
                HSD_TExpUnref(
                    (*texp).tev.c_in[i as usize].exp,
                    (*texp).tev.c_in[i as usize].sel,
                );
                HSD_TExpUnref(
                    (*texp).tev.a_in[i as usize].exp,
                    (*texp).tev.a_in[i as usize].sel,
                );
                i += 1;
            }
        }
        return;
    }
    if (*texp).cnst.r#ref as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        (*texp).cnst.r#ref = ((*texp).cnst.r#ref as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int) as u8;
        return;
    }
}
pub unsafe fn HSD_TExpFreeList(
    mut texp_list: *mut HSD_TExp,
    mut r#type: HSD_TExpType,
    mut all: s32,
) -> *mut HSD_TExp {
    melee_core::ct!("HSD_TExpFreeList");
    let mut handle: *mut *mut HSD_TExp =
        ::core::ptr::null_mut::<*mut HSD_TExp>();
    let mut ptr: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    let mut next: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    handle = &raw mut texp_list;
    if all != 0 as ::core::ffi::c_int {
        while !(*handle).is_null() {
            if r#type as ::core::ffi::c_int == HSD_TE_ALL as ::core::ffi::c_int
                || r#type as ::core::ffi::c_int
                    == (**handle).r#type as ::core::ffi::c_int
            {
                match (**handle).r#type as ::core::ffi::c_int {
                    1 => {
                        next = (**handle).tev.next;
                        HSD_TExpFree(*handle);
                        *handle = next;
                        continue;
                    }
                    4 => {
                        next = (**handle).tev.next;
                        HSD_TExpFree(*handle);
                        *handle = next;
                        continue;
                    }
                    _ => {
                        if 0 as ::core::ffi::c_int != 0 {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                129 as u32,
                                b"0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                    }
                }
            }
            handle = &raw mut (**handle).tev.next;
        }
    } else {
        if r#type as ::core::ffi::c_int == HSD_TE_ALL as ::core::ffi::c_int
            || r#type as ::core::ffi::c_int == HSD_TE_TEV as ::core::ffi::c_int
        {
            ptr = texp_list;
            while !ptr.is_null() {
                match (*ptr).r#type as ::core::ffi::c_int {
                    1 => {
                        if (*ptr).tev.c_ref == 0 as ::core::ffi::c_int
                            && (*ptr).tev.c_ref == 0 as ::core::ffi::c_int
                        {
                            HSD_TExpUnref(ptr, 1 as u8);
                            HSD_TExpUnref(ptr, 5 as u8);
                        }
                    }
                    _ => {}
                }
                ptr = (*ptr).tev.next;
            }
        }
        while !(*handle).is_null() {
            if r#type as ::core::ffi::c_int == HSD_TE_ALL as ::core::ffi::c_int
                || r#type as ::core::ffi::c_int
                    == (**handle).r#type as ::core::ffi::c_int
            {
                match (**handle).r#type as ::core::ffi::c_int {
                    1 => {
                        if !((**handle).tev.c_ref != 0 as ::core::ffi::c_int
                            || (**handle).tev.a_ref != 0 as ::core::ffi::c_int)
                        {
                            next = (**handle).tev.next;
                            HSD_TExpFree(*handle);
                            *handle = next;
                            continue;
                        }
                    }
                    4 => {
                        if !((**handle).cnst.r#ref as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int)
                        {
                            next = (**handle).tev.next;
                            HSD_TExpFree(*handle);
                            *handle = next;
                            continue;
                        }
                    }
                    _ => {
                        if 0 as ::core::ffi::c_int != 0 {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                171 as u32,
                                b"0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                    }
                }
            }
            handle = &raw mut (**handle).tev.next;
        }
    }
    return texp_list;
}
pub unsafe fn HSD_TExpTev(mut texp_list: *mut *mut HSD_TExp) -> *mut HSD_TExp {
    melee_core::ct!("HSD_TExpTev");
    let mut i: ::core::ffi::c_int = 0;
    let mut texp: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    if !texp_list.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            186 as u32,
            b"texp_list\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    texp = TevAlloc();
    memset(
        texp as *mut ::core::ffi::c_void,
        0xff as ::core::ffi::c_int,
        ::core::mem::size_of::<HSD_TETev>() as usize,
    );
    (*texp).r#type = HSD_TE_TEV;
    (*texp).tev.next = *texp_list;
    *texp_list = texp;
    (*texp).tev.c_ref = 0 as ::core::ffi::c_int as s32;
    (*texp).tev.a_ref = 0 as ::core::ffi::c_int as s32;
    (*texp).tev.tex = ::core::ptr::null_mut::<HSD_TObj>();
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        (*texp).tev.c_in[i as usize].exp = ::core::ptr::null_mut::<HSD_TExp>();
        (*texp).tev.a_in[i as usize].exp = ::core::ptr::null_mut::<HSD_TExp>();
        i += 1;
    }
    return texp;
}
pub unsafe fn HSD_TExpCnst(
    mut val: *mut ::core::ffi::c_void,
    mut comp: HSD_TEInput,
    mut r#type: HSD_TEType,
    mut texp_list: *mut *mut HSD_TExp,
) -> *mut HSD_TExp {
    melee_core::ct!("HSD_TExpCnst");
    let mut texp: *mut HSD_TExp = ::core::ptr::null_mut::<HSD_TExp>();
    if !texp_list.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            207 as u32,
            b"texp_list\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    texp = *texp_list;
    while !texp.is_null() {
        if (*texp).r#type as ::core::ffi::c_int
            == HSD_TE_CNST as ::core::ffi::c_int
            && (*texp).cnst.val == val
            && (*texp).cnst.comp as ::core::ffi::c_int
                == comp as ::core::ffi::c_int
        {
            if (*texp).cnst.ctype as ::core::ffi::c_int
                == r#type as ::core::ffi::c_int
            {
            } else {
                __assert(
                    b"src/sysdolphin/baselib/texp.c\0" as *const u8
                        as *const ::core::ffi::c_char,
                    215 as u32,
                    b"texp->cnst.ctype == type\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            };
            return texp;
        }
        texp = (*texp).cnst.next;
    }
    if comp as ::core::ffi::c_int == HSD_TE_0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<HSD_TExp>();
    }
    texp = CnstAlloc();
    (*texp).cnst.r#type = HSD_TE_CNST;
    (*texp).cnst.next = *texp_list;
    *texp_list = texp;
    (*texp).cnst.r#ref = 0 as u8;
    (*texp).cnst.val = val;
    (*texp).cnst.comp = comp;
    (*texp).cnst.ctype = r#type;
    (*texp).cnst.reg = 0xff as u8;
    (*texp).cnst.idx = 0xff as u8;
    return texp;
}
pub unsafe fn HSD_TExpColorOp(
    mut texp: *mut HSD_TExp,
    mut op: GXTevOp,
    mut bias: GXTevBias,
    mut scale: GXTevScale,
    mut clamp: u8,
) {
    melee_core::ct!("HSD_TExpColorOp");
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            243 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            244 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*texp).tev.c_op = op as u8;
    (*texp).tev.c_clamp = (if clamp as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as u8;
    if op as u32 <= GX_TEV_SUB as ::core::ffi::c_int as u32 {
        (*texp).tev.c_bias = bias as u8;
        (*texp).tev.c_scale = scale as u8;
    } else {
        (*texp).tev.c_bias = 0 as u8;
        (*texp).tev.c_scale = 0 as u8;
    };
}
pub unsafe fn HSD_TExpAlphaOp(
    mut texp: *mut HSD_TExp,
    mut op: GXTevOp,
    mut bias: GXTevBias,
    mut scale: GXTevScale,
    mut clamp: u8,
) {
    melee_core::ct!("HSD_TExpAlphaOp");
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            260 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            261 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*texp).tev.a_op = op as u8;
    (*texp).tev.a_clamp = (if clamp as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as u8;
    if op as u32 <= GX_TEV_SUB as ::core::ffi::c_int as u32 {
        (*texp).tev.a_bias = bias as u8;
        (*texp).tev.a_scale = scale as u8;
    } else {
        (*texp).tev.a_bias = 0 as u8;
        (*texp).tev.a_scale = 0 as u8;
    };
}
pub unsafe fn HSD_TExpColorInSub(
    mut tev: *mut HSD_TETev,
    mut sel: HSD_TEInput,
    mut exp: *mut HSD_TExp,
    mut idx: s32,
) {
    melee_core::ct!("HSD_TExpColorInSub");
    let mut prev: HSD_TEArg = _HSD_TEArg {
        r#type: 0,
        sel: 0,
        arg: 0,
        exp: ::core::ptr::null_mut::<HSD_TExp>(),
    };
    let mut ksel: u8 = 0;
    prev = (*tev).c_in[idx as usize];
    (*tev).c_in[idx as usize].r#type = HSD_TExpGetType(exp) as u8;
    (*tev).c_in[idx as usize].sel = sel as u8;
    (*tev).c_in[idx as usize].exp = exp;
    (*tev).c_in[idx as usize].arg = 0xff as u8;
    match sel as ::core::ffi::c_int {
        7 => {
            (*tev).c_in[idx as usize].arg =
                GX_CC_ZERO as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_ZERO as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].exp =
                ::core::ptr::null_mut::<HSD_TExp>();
        }
        8 => {
            (*tev).c_in[idx as usize].arg =
                GX_CC_ONE as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_IMM as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].exp =
                ::core::ptr::null_mut::<HSD_TExp>();
        }
        12 => {
            (*tev).c_in[idx as usize].arg =
                GX_CC_HALF as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_IMM as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].exp =
                ::core::ptr::null_mut::<HSD_TExp>();
        }
        9 | 10 | 11 | 13 | 14 | 15 => {
            (*tev).c_in[idx as usize].arg =
                GX_CC_KONST as ::core::ffi::c_int as u8;
            match sel as ::core::ffi::c_int {
                9 => {
                    ksel = GX_TEV_KCSEL_1_8 as ::core::ffi::c_int as u8;
                }
                10 => {
                    ksel = GX_TEV_KCSEL_1_4 as ::core::ffi::c_int as u8;
                }
                11 => {
                    ksel = GX_TEV_KCSEL_3_8 as ::core::ffi::c_int as u8;
                }
                13 => {
                    ksel = GX_TEV_KCSEL_5_8 as ::core::ffi::c_int as u8;
                }
                14 => {
                    ksel = GX_TEV_KCSEL_3_4 as ::core::ffi::c_int as u8;
                }
                15 => {
                    ksel = GX_TEV_KCSEL_7_8 as ::core::ffi::c_int as u8;
                }
                _ => {
                    if 0 as ::core::ffi::c_int != 0 {
                    } else {
                        __assert(
                            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            329 as u32,
                            b"0\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    };
                }
            }
            if (*tev).kcsel as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                (*tev).kcsel = ksel;
            } else if (*tev).kcsel as ::core::ffi::c_int
                == ksel as ::core::ffi::c_int
            {
                HSD_Panic(
                    b"src/sysdolphin/baselib/texp.c\0" as *const u8
                        as *const ::core::ffi::c_char,
                    503 as u32,
                    b"tev can't select multiple konst input.\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_KONST as ::core::ffi::c_int as u8;
        }
        _ => match (*tev).c_in[idx as usize].r#type as ::core::ffi::c_int {
            0 => {
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_ZERO as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].sel =
                    HSD_TE_0 as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    GX_CC_ZERO as ::core::ffi::c_int as u8;
            }
            1 => {
                let mut swap: u8 = 0;
                if sel as ::core::ffi::c_int
                    == HSD_TE_RGB as ::core::ffi::c_int
                    || sel as ::core::ffi::c_int
                        == HSD_TE_A as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        352 as u32,
                        b"sel == HSD_TE_RGB || sel == HSD_TE_A\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                if idx == 3 as ::core::ffi::c_int
                    || sel as ::core::ffi::c_int
                        != HSD_TE_RGB as ::core::ffi::c_int
                    || (*exp).tev.c_clamp as ::core::ffi::c_int != 0
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        354 as u32,
                        b"idx == 3 || sel != HSD_TE_RGB || exp->tev.c_clamp\0"
                            as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                if idx == 3 as ::core::ffi::c_int
                    || sel as ::core::ffi::c_int
                        != HSD_TE_A as ::core::ffi::c_int
                    || (*exp).tev.a_clamp as ::core::ffi::c_int != 0
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        356 as u32,
                        b"idx == 3 || sel != HSD_TE_A || exp->tev.a_clamp\0"
                            as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                swap = (*tev).c_in[idx as usize].sel;
                match HSD_TExpGetType((*tev).c_in[idx as usize].exp)
                    as ::core::ffi::c_int
                {
                    1 => {
                        if swap as ::core::ffi::c_int
                            == 1 as ::core::ffi::c_int
                        {
                            (*(*tev).c_in[idx as usize].exp).tev.c_ref =
                                (*(*tev).c_in[idx as usize].exp)
                                    .tev
                                    .c_ref
                                    .wrapping_add(1);
                        } else {
                            (*(*tev).c_in[idx as usize].exp).tev.a_ref =
                                (*(*tev).c_in[idx as usize].exp)
                                    .tev
                                    .a_ref
                                    .wrapping_add(1);
                        }
                    }
                    4 => {
                        (*(*tev).c_in[idx as usize].exp).cnst.r#ref =
                            ((*(*tev).c_in[idx as usize].exp).cnst.r#ref
                                as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int)
                                as u8;
                    }
                    _ => {}
                }
            }
            4 => {
                let mut swap_0: u8 = 0;
                (*tev).c_in[idx as usize].sel = (*exp).cnst.comp as u8;
                swap_0 = (*tev).c_in[idx as usize].sel;
                match HSD_TExpGetType((*tev).c_in[idx as usize].exp)
                    as ::core::ffi::c_int
                {
                    1 => {
                        if swap_0 as ::core::ffi::c_int
                            == 1 as ::core::ffi::c_int
                        {
                            (*(*tev).c_in[idx as usize].exp).tev.c_ref =
                                (*(*tev).c_in[idx as usize].exp)
                                    .tev
                                    .c_ref
                                    .wrapping_add(1);
                        } else {
                            (*(*tev).c_in[idx as usize].exp).tev.a_ref =
                                (*(*tev).c_in[idx as usize].exp)
                                    .tev
                                    .a_ref
                                    .wrapping_add(1);
                        }
                    }
                    4 => {
                        (*(*tev).c_in[idx as usize].exp).cnst.r#ref =
                            ((*(*tev).c_in[idx as usize].exp).cnst.r#ref
                                as ::core::ffi::c_int
                                + 1 as ::core::ffi::c_int)
                                as u8;
                    }
                    _ => {}
                }
            }
            2 => {
                let mut swap_1: u8 = HSD_TE_UNDEF as ::core::ffi::c_int as u8;
                match sel as ::core::ffi::c_int {
                    1 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_TEXC as ::core::ffi::c_int as u8;
                        swap_1 = GX_TEV_SWAP0 as ::core::ffi::c_int as u8;
                    }
                    2 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_TEXC as ::core::ffi::c_int as u8;
                        swap_1 = GX_TEV_SWAP1 as ::core::ffi::c_int as u8;
                    }
                    3 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_TEXC as ::core::ffi::c_int as u8;
                        swap_1 = GX_TEV_SWAP2 as ::core::ffi::c_int as u8;
                    }
                    4 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_TEXC as ::core::ffi::c_int as u8;
                        swap_1 = GX_TEV_SWAP3 as ::core::ffi::c_int as u8;
                    }
                    5 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_TEXA as ::core::ffi::c_int as u8;
                    }
                    _ => {
                        if 0 as ::core::ffi::c_int != 0 {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                415 as u32,
                                b"0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                    }
                }
                if (*tev).tex_swap as ::core::ffi::c_int
                    == HSD_TE_UNDEF as ::core::ffi::c_int
                {
                    (*tev).tex_swap = swap_1;
                } else {
                    if swap_1 as ::core::ffi::c_int
                        == HSD_TE_UNDEF as ::core::ffi::c_int
                        || (*tev).tex_swap as ::core::ffi::c_int
                            == swap_1 as ::core::ffi::c_int
                    {
                    } else {
                        __assert(
                            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            422 as u32,
                            b"swap == HSD_TE_UNDEF || tev->tex_swap == swap\0"
                                as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    };
                }
            }
            3 => {
                let mut swap_2: u8 = HSD_TE_UNDEF as ::core::ffi::c_int as u8;
                match sel as ::core::ffi::c_int {
                    1 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_RASC as ::core::ffi::c_int as u8;
                        swap_2 = GX_TEV_SWAP0 as ::core::ffi::c_int as u8;
                    }
                    2 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_RASC as ::core::ffi::c_int as u8;
                        swap_2 = GX_TEV_SWAP1 as ::core::ffi::c_int as u8;
                    }
                    3 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_RASC as ::core::ffi::c_int as u8;
                        swap_2 = GX_TEV_SWAP2 as ::core::ffi::c_int as u8;
                    }
                    4 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_RASC as ::core::ffi::c_int as u8;
                        swap_2 = GX_TEV_SWAP3 as ::core::ffi::c_int as u8;
                    }
                    5 => {
                        (*tev).c_in[idx as usize].arg =
                            GX_CC_RASA as ::core::ffi::c_int as u8;
                    }
                    _ => {
                        if 0 as ::core::ffi::c_int != 0 {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                448 as u32,
                                b"0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                    }
                }
                if (*tev).ras_swap as ::core::ffi::c_int
                    == HSD_TE_UNDEF as ::core::ffi::c_int
                {
                    (*tev).ras_swap = swap_2;
                } else {
                    if swap_2 as ::core::ffi::c_int
                        == HSD_TE_UNDEF as ::core::ffi::c_int
                        || (*tev).ras_swap as ::core::ffi::c_int
                            == swap_2 as ::core::ffi::c_int
                    {
                    } else {
                        __assert(
                            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            455 as u32,
                            b"swap == HSD_TE_UNDEF || tev->ras_swap == swap\0"
                                as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    };
                }
            }
            _ => {
                if 0 as ::core::ffi::c_int != 0 {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        459 as u32,
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                };
            }
        },
    }
    HSD_TExpUnref(prev.exp, prev.sel);
}
pub unsafe fn HSD_TExpColorIn(
    mut texp: *mut HSD_TExp,
    mut sel_a: HSD_TEInput,
    mut exp_a: *mut HSD_TExp,
    mut sel_b: HSD_TEInput,
    mut exp_b: *mut HSD_TExp,
    mut sel_c: HSD_TEInput,
    mut exp_c: *mut HSD_TExp,
    mut sel_d: HSD_TEInput,
    mut exp_d: *mut HSD_TExp,
) {
    melee_core::ct!("HSD_TExpColorIn");
    let mut tev: *mut HSD_TETev = ::core::ptr::null_mut::<HSD_TETev>();
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            473 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            474 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    tev = &raw mut (*texp).tev as *mut HSD_TETev;
    HSD_TExpColorInSub(tev, sel_a, exp_a, 0 as s32);
    HSD_TExpColorInSub(tev, sel_b, exp_b, 1 as s32);
    HSD_TExpColorInSub(tev, sel_c, exp_c, 2 as s32);
    HSD_TExpColorInSub(tev, sel_d, exp_d, 3 as s32);
}
pub unsafe fn HSD_TExpAlphaInSub(
    mut tev: *mut HSD_TETev,
    mut sel: HSD_TEInput,
    mut exp: *mut HSD_TExp,
    mut idx: s32,
) {
    melee_core::ct!("HSD_TExpAlphaInSub");
    let mut prev: HSD_TEArg = _HSD_TEArg {
        r#type: 0,
        sel: 0,
        arg: 0,
        exp: ::core::ptr::null_mut::<HSD_TExp>(),
    };
    let mut ksel: u8 = 0;
    prev = (*tev).a_in[idx as usize];
    (*tev).a_in[idx as usize].sel = sel as u8;
    (*tev).a_in[idx as usize].exp = exp;
    (*tev).a_in[idx as usize].r#type = HSD_TExpGetType(exp) as u8;
    (*tev).a_in[idx as usize].arg = 0xff as u8;
    match sel as ::core::ffi::c_int {
        7 => {
            (*tev).a_in[idx as usize].r#type =
                HSD_TE_ZERO as ::core::ffi::c_int as u8;
            (*tev).a_in[idx as usize].arg =
                GX_CA_ZERO as ::core::ffi::c_int as u8;
            (*tev).a_in[idx as usize].exp =
                ::core::ptr::null_mut::<HSD_TExp>();
        }
        8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => {
            (*tev).a_in[idx as usize].exp =
                ::core::ptr::null_mut::<HSD_TExp>();
            (*tev).a_in[idx as usize].arg =
                GX_CA_KONST as ::core::ffi::c_int as u8;
            match sel as ::core::ffi::c_int {
                9 => {
                    ksel = GX_TEV_KASEL_1_8 as ::core::ffi::c_int as u8;
                }
                10 => {
                    ksel = GX_TEV_KASEL_1_4 as ::core::ffi::c_int as u8;
                }
                11 => {
                    ksel = GX_TEV_KASEL_3_8 as ::core::ffi::c_int as u8;
                }
                12 => {
                    ksel = GX_TEV_KASEL_1_2 as ::core::ffi::c_int as u8;
                }
                13 => {
                    ksel = GX_TEV_KASEL_5_8 as ::core::ffi::c_int as u8;
                }
                14 => {
                    ksel = GX_TEV_KASEL_3_4 as ::core::ffi::c_int as u8;
                }
                15 => {
                    ksel = GX_TEV_KASEL_7_8 as ::core::ffi::c_int as u8;
                }
                _ => {
                    ksel = GX_TEV_KASEL_1 as ::core::ffi::c_int as u8;
                }
            }
            if (*tev).kasel as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                (*tev).kasel = ksel;
            } else {
                if (*tev).kasel as ::core::ffi::c_int
                    == ksel as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        542 as u32,
                        b"tev->kasel == ksel\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
            }
            (*tev).a_in[idx as usize].r#type =
                HSD_TE_KONST as ::core::ffi::c_int as u8;
        }
        _ => match (*tev).a_in[idx as usize].r#type as ::core::ffi::c_int {
            0 => {
                (*tev).a_in[idx as usize].exp =
                    ::core::ptr::null_mut::<HSD_TExp>();
                (*tev).a_in[idx as usize].r#type =
                    HSD_TE_ZERO as ::core::ffi::c_int as u8;
                (*tev).a_in[idx as usize].sel =
                    HSD_TE_0 as ::core::ffi::c_int as u8;
                (*tev).a_in[idx as usize].arg =
                    GX_CA_ZERO as ::core::ffi::c_int as u8;
            }
            1 => {
                if sel as ::core::ffi::c_int == HSD_TE_A as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        556 as u32,
                        b"sel == HSD_TE_A\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                if idx == 3 as ::core::ffi::c_int
                    || (*exp).tev.a_clamp as ::core::ffi::c_int != 0
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        557 as u32,
                        b"idx == 3 || exp->tev.a_clamp\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                HSD_TExpRef(
                    (*tev).a_in[idx as usize].exp,
                    (*tev).a_in[idx as usize].sel,
                );
            }
            4 => {
                if sel as ::core::ffi::c_int == HSD_TE_A as ::core::ffi::c_int
                    || sel as ::core::ffi::c_int
                        == HSD_TE_X as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        562 as u32,
                        b"sel == HSD_TE_A || sel == HSD_TE_X\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                if (*exp).cnst.comp as ::core::ffi::c_int
                    == HSD_TE_X as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        563 as u32,
                        b"exp->cnst.comp == HSD_TE_X\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                (*tev).a_in[idx as usize].sel =
                    HSD_TE_X as ::core::ffi::c_int as u8;
                HSD_TExpRef(
                    (*tev).a_in[idx as usize].exp,
                    (*tev).a_in[idx as usize].sel,
                );
            }
            2 => {
                if sel as ::core::ffi::c_int == HSD_TE_A as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        568 as u32,
                        b"sel == HSD_TE_A\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                (*tev).a_in[idx as usize].arg =
                    GX_CA_TEXA as ::core::ffi::c_int as u8;
            }
            3 => {
                if sel as ::core::ffi::c_int == HSD_TE_A as ::core::ffi::c_int
                {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        572 as u32,
                        b"sel == HSD_TE_A\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                };
                (*tev).a_in[idx as usize].arg =
                    GX_CA_RASA as ::core::ffi::c_int as u8;
            }
            _ => {
                if 0 as ::core::ffi::c_int != 0 {
                } else {
                    __assert(
                        b"src/sysdolphin/baselib/texp.c\0" as *const u8
                            as *const ::core::ffi::c_char,
                        576 as u32,
                        b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                };
            }
        },
    }
    HSD_TExpUnref(prev.exp, prev.sel);
}
pub unsafe fn HSD_TExpAlphaIn(
    mut texp: *mut HSD_TExp,
    mut sel_a: HSD_TEInput,
    mut exp_a: *mut HSD_TExp,
    mut sel_b: HSD_TEInput,
    mut exp_b: *mut HSD_TExp,
    mut sel_c: HSD_TEInput,
    mut exp_c: *mut HSD_TExp,
    mut sel_d: HSD_TEInput,
    mut exp_d: *mut HSD_TExp,
) {
    melee_core::ct!("HSD_TExpAlphaIn");
    let mut tev: *mut HSD_TETev = ::core::ptr::null_mut::<HSD_TETev>();
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            590 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            591 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    tev = &raw mut (*texp).tev as *mut HSD_TETev;
    HSD_TExpAlphaInSub(tev, sel_a, exp_a, 0 as s32);
    HSD_TExpAlphaInSub(tev, sel_b, exp_b, 1 as s32);
    HSD_TExpAlphaInSub(tev, sel_c, exp_c, 2 as s32);
    HSD_TExpAlphaInSub(tev, sel_d, exp_d, 3 as s32);
}
pub unsafe fn HSD_TExpOrder(
    mut texp: *mut HSD_TExp,
    mut tex: *mut HSD_TObj,
    mut chan: GXChannelID,
) {
    melee_core::ct!("HSD_TExpOrder");
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            602 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            603 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*texp).tev.tex = tex;
    if chan as ::core::ffi::c_int == GX_COLOR_NULL as ::core::ffi::c_int {
        (*texp).tev.chan = GX_COLOR_NULL as ::core::ffi::c_int as u8;
        return;
    }
    (*texp).tev.chan = chan as u8;
}
pub unsafe fn AssignColorReg(
    mut tev: *mut HSD_TETev,
    mut idx: ::core::ffi::c_int,
    mut res: *mut HSD_TExpRes,
) -> ::core::ffi::c_int {
    melee_core::ct!("AssignColorReg");
    static mut a_in: [GXTevColorArg; 4] =
        [GX_CC_A0, GX_CC_A1, GX_CC_A2, GX_CC_APREV];
    static mut c_in: [GXTevColorArg; 4] =
        [GX_CC_C0, GX_CC_C1, GX_CC_C2, GX_CC_CPREV];
    let mut cnst: *mut HSD_TECnst = ::core::ptr::null_mut::<HSD_TECnst>();
    let mut j: ::core::ffi::c_int = 0;
    cnst = &raw mut (*(*(&raw mut (*tev).c_in as *mut HSD_TEArg)
        .offset(idx as isize))
    .exp)
        .cnst as *mut HSD_TECnst;
    if (*cnst).reg as ::core::ffi::c_int != HSD_TE_UNDEF as ::core::ffi::c_int
    {
        if ((*cnst).reg as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*tev).c_in[idx as usize].r#type =
            HSD_TE_IMM as ::core::ffi::c_int as u8;
        if (*cnst).comp as ::core::ffi::c_int == HSD_TE_X as ::core::ffi::c_int
        {
            (*tev).c_in[idx as usize].arg =
                a_in[((*cnst).reg as ::core::ffi::c_int
                    - 4 as ::core::ffi::c_int) as usize] as u8;
        } else {
            (*tev).c_in[idx as usize].arg =
                c_in[((*cnst).reg as ::core::ffi::c_int
                    - 4 as ::core::ffi::c_int) as usize] as u8;
        }
        return 0 as ::core::ffi::c_int;
    } else if (*cnst).comp as ::core::ffi::c_int
        == HSD_TE_X as ::core::ffi::c_int
    {
        j = 4 as ::core::ffi::c_int;
        while j < 8 as ::core::ffi::c_int {
            if (*res).reg[j as usize].alpha as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].alpha = 1 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 3 as u8;
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_IMM as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    a_in[(j - 4 as ::core::ffi::c_int) as usize] as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    } else {
        j = 4 as ::core::ffi::c_int;
        while j < 8 as ::core::ffi::c_int {
            if (*res).reg[j as usize].color as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].color = 3 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 0 as u8;
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_IMM as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    c_in[(j - 4 as ::core::ffi::c_int) as usize] as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn AssignAlphaReg(
    mut tev: *mut HSD_TETev,
    mut idx: ::core::ffi::c_int,
    mut res: *mut HSD_TExpRes,
) -> ::core::ffi::c_int {
    melee_core::ct!("AssignAlphaReg");
    static mut in_0: [GXTevAlphaArg; 4] =
        [GX_CA_A0, GX_CA_A1, GX_CA_A2, GX_CA_APREV];
    let mut cnst: *mut HSD_TECnst = ::core::ptr::null_mut::<HSD_TECnst>();
    let mut j: ::core::ffi::c_int = 0;
    cnst = &raw mut (*(*(&raw mut (*tev).a_in as *mut HSD_TEArg)
        .offset(idx as isize))
    .exp)
        .cnst as *mut HSD_TECnst;
    if (*cnst).reg as ::core::ffi::c_int != HSD_TE_UNDEF as ::core::ffi::c_int
    {
        if ((*cnst).reg as ::core::ffi::c_int) < 4 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*tev).a_in[idx as usize].r#type =
            HSD_TE_IMM as ::core::ffi::c_int as u8;
        (*tev).a_in[idx as usize].arg =
            in_0[((*cnst).reg as ::core::ffi::c_int - 4 as ::core::ffi::c_int)
                as usize] as u8;
        return 0 as ::core::ffi::c_int;
    } else {
        j = 4 as ::core::ffi::c_int;
        while j < 8 as ::core::ffi::c_int {
            if (*res).reg[j as usize].alpha as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].alpha = 1 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 3 as u8;
                (*tev).a_in[idx as usize].r#type =
                    HSD_TE_IMM as ::core::ffi::c_int as u8;
                (*tev).a_in[idx as usize].arg =
                    in_0[(j - 4 as ::core::ffi::c_int) as usize] as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn AssignColorKonst(
    mut tev: *mut HSD_TETev,
    mut idx: ::core::ffi::c_int,
    mut res: *mut HSD_TExpRes,
) -> ::core::ffi::c_int {
    melee_core::ct!("AssignColorKonst");
    static mut xsel: [[GXTevKColorSel; 4]; 4] = [
        [
            GX_TEV_KCSEL_K0_R,
            GX_TEV_KCSEL_K0_G,
            GX_TEV_KCSEL_K0_B,
            GX_TEV_KCSEL_K0_A,
        ],
        [
            GX_TEV_KCSEL_K1_R,
            GX_TEV_KCSEL_K1_G,
            GX_TEV_KCSEL_K1_B,
            GX_TEV_KCSEL_K1_A,
        ],
        [
            GX_TEV_KCSEL_K2_R,
            GX_TEV_KCSEL_K2_G,
            GX_TEV_KCSEL_K2_B,
            GX_TEV_KCSEL_K2_A,
        ],
        [
            GX_TEV_KCSEL_K3_R,
            GX_TEV_KCSEL_K3_G,
            GX_TEV_KCSEL_K3_B,
            GX_TEV_KCSEL_K3_A,
        ],
    ];
    static mut csel: [GXTevKColorSel; 4] = [
        GX_TEV_KCSEL_K0,
        GX_TEV_KCSEL_K1,
        GX_TEV_KCSEL_K2,
        GX_TEV_KCSEL_K3,
    ];
    let mut cnst: *mut HSD_TECnst = ::core::ptr::null_mut::<HSD_TECnst>();
    let mut j: ::core::ffi::c_int = 0;
    cnst = &raw mut (*(*(&raw mut (*tev).c_in as *mut HSD_TEArg)
        .offset(idx as isize))
    .exp)
        .cnst as *mut HSD_TECnst;
    if (*cnst).reg as ::core::ffi::c_int != HSD_TE_UNDEF as ::core::ffi::c_int
    {
        if (*cnst).reg as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        if (*cnst).comp as ::core::ffi::c_int == HSD_TE_X as ::core::ffi::c_int
        {
            (*tev).kcsel =
                xsel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_KONST as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].arg =
                GX_CC_KONST as ::core::ffi::c_int as u8;
        } else {
            (*tev).kcsel = csel[(*cnst).reg as usize] as u8;
            (*tev).c_in[idx as usize].r#type =
                HSD_TE_KONST as ::core::ffi::c_int as u8;
            (*tev).c_in[idx as usize].arg =
                GX_CC_KONST as ::core::ffi::c_int as u8;
        }
        return 0 as ::core::ffi::c_int;
    } else if (*cnst).comp as ::core::ffi::c_int
        == HSD_TE_X as ::core::ffi::c_int
    {
        j = 1 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if (*res).reg[j as usize].alpha as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].alpha = 1 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 3 as u8;
                (*tev).kcsel =
                    xsel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_KONST as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    GX_CC_KONST as ::core::ffi::c_int as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
        j = 0 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if ((*res).reg[j as usize].color as ::core::ffi::c_int)
                < 3 as ::core::ffi::c_int
            {
                (*cnst).reg = j as u8;
                let fresh1 = (*res).reg[j as usize].color;
                (*res).reg[j as usize].color =
                    (*res).reg[j as usize].color.wrapping_add(1);
                (*cnst).idx = fresh1;
                (*tev).kcsel =
                    xsel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_KONST as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    GX_CC_KONST as ::core::ffi::c_int as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    } else {
        j = 0 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if (*res).reg[j as usize].color as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].color = 3 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 0 as u8;
                (*tev).kcsel = csel[(*cnst).reg as usize] as u8;
                (*tev).c_in[idx as usize].r#type =
                    HSD_TE_KONST as ::core::ffi::c_int as u8;
                (*tev).c_in[idx as usize].arg =
                    GX_CC_KONST as ::core::ffi::c_int as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn AssignAlphaKonst(
    mut tev: *mut HSD_TETev,
    mut idx: ::core::ffi::c_int,
    mut res: *mut HSD_TExpRes,
) -> ::core::ffi::c_int {
    melee_core::ct!("AssignAlphaKonst");
    static mut sel: [[GXTevKAlphaSel; 4]; 4] = [
        [
            GX_TEV_KASEL_K0_R,
            GX_TEV_KASEL_K0_G,
            GX_TEV_KASEL_K0_B,
            GX_TEV_KASEL_K0_A,
        ],
        [
            GX_TEV_KASEL_K1_R,
            GX_TEV_KASEL_K1_G,
            GX_TEV_KASEL_K1_B,
            GX_TEV_KASEL_K1_A,
        ],
        [
            GX_TEV_KASEL_K2_R,
            GX_TEV_KASEL_K2_G,
            GX_TEV_KASEL_K2_B,
            GX_TEV_KASEL_K2_A,
        ],
        [
            GX_TEV_KASEL_K3_R,
            GX_TEV_KASEL_K3_G,
            GX_TEV_KASEL_K3_B,
            GX_TEV_KASEL_K3_A,
        ],
    ];
    let mut cnst: *mut HSD_TECnst = ::core::ptr::null_mut::<HSD_TECnst>();
    let mut j: ::core::ffi::c_int = 0;
    cnst = &raw mut (*(*(&raw mut (*tev).a_in as *mut HSD_TEArg)
        .offset(idx as isize))
    .exp)
        .cnst as *mut HSD_TECnst;
    if (*cnst).reg as ::core::ffi::c_int != HSD_TE_UNDEF as ::core::ffi::c_int
    {
        if (*cnst).reg as ::core::ffi::c_int >= 4 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int);
        }
        (*tev).kasel = sel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
        (*tev).a_in[idx as usize].r#type =
            HSD_TE_KONST as ::core::ffi::c_int as u8;
        (*tev).a_in[idx as usize].arg =
            GX_CA_KONST as ::core::ffi::c_int as u8;
        return 0 as ::core::ffi::c_int;
    } else {
        j = 1 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if (*res).reg[j as usize].alpha as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                (*res).reg[j as usize].alpha = 1 as u8;
                (*cnst).reg = j as u8;
                (*cnst).idx = 3 as u8;
                (*tev).kasel =
                    sel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
                (*tev).a_in[idx as usize].r#type =
                    HSD_TE_KONST as ::core::ffi::c_int as u8;
                (*tev).a_in[idx as usize].arg =
                    GX_CA_KONST as ::core::ffi::c_int as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
        j = 0 as ::core::ffi::c_int;
        while j < 4 as ::core::ffi::c_int {
            if ((*res).reg[j as usize].color as ::core::ffi::c_int)
                < 3 as ::core::ffi::c_int
            {
                (*cnst).reg = j as u8;
                let fresh0 = (*res).reg[j as usize].color;
                (*res).reg[j as usize].color =
                    (*res).reg[j as usize].color.wrapping_add(1);
                (*cnst).idx = fresh0;
                (*tev).kasel =
                    sel[(*cnst).reg as usize][(*cnst).idx as usize] as u8;
                (*tev).a_in[idx as usize].r#type =
                    HSD_TE_KONST as ::core::ffi::c_int as u8;
                (*tev).a_in[idx as usize].arg =
                    GX_CA_KONST as ::core::ffi::c_int as u8;
                return 0 as ::core::ffi::c_int;
            }
            j += 1;
        }
    }
    return -(1 as ::core::ffi::c_int);
}
pub unsafe fn TExpAssignReg(
    mut texp: *mut HSD_TExp,
    mut res: *mut HSD_TExpRes,
) -> ::core::ffi::c_int {
    melee_core::ct!("TExpAssignReg");
    let mut tev: *mut HSD_TETev = texp as *mut HSD_TETev;
    let mut i: ::core::ffi::c_int = 0;
    let mut val: ::core::ffi::c_int = 0;
    if (*tev).c_ref > 0 as ::core::ffi::c_int {
        if (*tev).kcsel as ::core::ffi::c_int
            != HSD_TE_UNDEF as ::core::ffi::c_int
        {
            i = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                if (*tev).c_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    val = AssignColorReg(tev, i, res);
                    if val < 0 as ::core::ffi::c_int {
                        if val >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                831 as u32,
                                b"val >= 0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                        return val;
                    }
                }
                i += 1;
            }
        } else if IsThroughColor(tev as *mut HSD_TExp) != 0
            && (*tev).c_in[3 as ::core::ffi::c_int as usize].r#type
                as ::core::ffi::c_int
                == HSD_TE_CNST as ::core::ffi::c_int
        {
            if AssignColorKonst(tev, 3 as ::core::ffi::c_int, res)
                < 0 as ::core::ffi::c_int
            {
                val = AssignColorReg(tev, 3 as ::core::ffi::c_int, res);
                if val < 0 as ::core::ffi::c_int {
                    if val >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert(
                            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            842 as u32,
                            b"val >= 0\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    };
                    return val;
                }
            }
        } else {
            i = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                if (*tev).c_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    if AssignColorKonst(tev, i, res) < 0 as ::core::ffi::c_int
                    {
                        val = AssignColorReg(tev, i, res);
                        if val < 0 as ::core::ffi::c_int {
                            if val >= 0 as ::core::ffi::c_int {
                            } else {
                                __assert(
                                    b"src/sysdolphin/baselib/texp.c\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                    852 as u32,
                                    b"val >= 0\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                            };
                            return val;
                        }
                    }
                }
                i += 1;
            }
            while i < 4 as ::core::ffi::c_int {
                if (*tev).c_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    val = AssignColorReg(tev, i, res);
                    if val < 0 as ::core::ffi::c_int {
                        if val >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                862 as u32,
                                b"val >= 0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                        return val;
                    }
                }
                i += 1;
            }
        }
    }
    if (*tev).a_ref > 0 as ::core::ffi::c_int {
        if (*tev).kasel as ::core::ffi::c_int
            != HSD_TE_UNDEF as ::core::ffi::c_int
        {
            i = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                if (*tev).a_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    val = AssignAlphaReg(tev, i, res);
                    if val < 0 as ::core::ffi::c_int {
                        if val >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                876 as u32,
                                b"val >= 0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                        return val;
                    }
                }
                i += 1;
            }
        } else if IsThroughAlpha(tev as *mut HSD_TExp) != 0
            && (*tev).a_in[3 as ::core::ffi::c_int as usize].r#type
                as ::core::ffi::c_int
                == HSD_TE_CNST as ::core::ffi::c_int
        {
            if AssignAlphaReg(tev, 3 as ::core::ffi::c_int, res)
                < 0 as ::core::ffi::c_int
            {
                val = AssignAlphaKonst(tev, 3 as ::core::ffi::c_int, res);
                if val < 0 as ::core::ffi::c_int {
                    if val >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert(
                            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                as *const ::core::ffi::c_char,
                            888 as u32,
                            b"val >= 0\0" as *const u8
                                as *const ::core::ffi::c_char,
                        );
                    };
                    return val;
                }
            }
        } else {
            i = 0 as ::core::ffi::c_int;
            while i < 4 as ::core::ffi::c_int {
                if (*tev).a_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    if AssignAlphaKonst(tev, i, res) < 0 as ::core::ffi::c_int
                    {
                        val = AssignAlphaReg(tev, i, res);
                        if val < 0 as ::core::ffi::c_int {
                            if val >= 0 as ::core::ffi::c_int {
                            } else {
                                __assert(
                                    b"src/sysdolphin/baselib/texp.c\0"
                                        as *const u8
                                        as *const ::core::ffi::c_char,
                                    898 as u32,
                                    b"val >= 0\0" as *const u8
                                        as *const ::core::ffi::c_char,
                                );
                            };
                            return val;
                        }
                    }
                }
                i += 1;
            }
            while i < 4 as ::core::ffi::c_int {
                if (*tev).a_in[i as usize].r#type as ::core::ffi::c_int
                    == HSD_TE_CNST as ::core::ffi::c_int
                {
                    val = AssignAlphaReg(tev, i, res);
                    if val < 0 as ::core::ffi::c_int {
                        if val >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert(
                                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                                    as *const ::core::ffi::c_char,
                                908 as u32,
                                b"val >= 0\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        };
                        return val;
                    }
                }
                i += 1;
            }
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe fn TExp2TevDesc(
    mut texp: *mut HSD_TExp,
    mut desc: *mut HSD_TExpTevDesc,
    mut init_cprev: *mut ::core::ffi::c_int,
    mut init_aprev: *mut ::core::ffi::c_int,
) {
    melee_core::ct!("TExp2TevDesc");
    static mut dst: [GXTevRegID; 4] =
        [GX_TEVREG0, GX_TEVREG1, GX_TEVREG2, GX_TEVPREV];
    let mut tev: *mut HSD_TETev = ::core::ptr::null_mut::<HSD_TETev>();
    let mut tevdesc: *mut HSD_TevDesc = ::core::ptr::null_mut::<HSD_TevDesc>();
    if !texp.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            928 as u32,
            b"texp\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !desc.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            929 as u32,
            b"desc\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if HSD_TExpGetType(texp) as ::core::ffi::c_int
        == HSD_TE_TEV as ::core::ffi::c_int
    {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            930 as u32,
            b"HSD_TExpGetType(texp) == HSD_TE_TEV\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    tev = &raw mut (*texp).tev as *mut HSD_TETev;
    tevdesc = &raw mut (*desc).desc as *mut HSD_TevDesc;
    (*tevdesc).next = ::core::ptr::null_mut::<_HSD_TevDesc>();
    (*tevdesc).flags = 1 as u32;
    (*desc).tobj = (*tev).tex;
    if (*tev).tex.is_null() {
        (*tevdesc).coord = HSD_TE_UNDEF as ::core::ffi::c_int as u32;
        (*tevdesc).map = HSD_TE_UNDEF as ::core::ffi::c_int as u32;
    }
    (*tevdesc).color = (if (*tev).chan as ::core::ffi::c_int
        == HSD_TE_UNDEF as ::core::ffi::c_int
    {
        HSD_TE_UNDEF as ::core::ffi::c_int
    } else {
        (*tev).chan as ::core::ffi::c_int
    }) as u32;
    (*tevdesc).u.tevconf.ras_swap = (if (*tev).ras_swap as ::core::ffi::c_int
        == HSD_TE_UNDEF as ::core::ffi::c_int
    {
        0 as ::core::ffi::c_int
    } else {
        (*tev).ras_swap as ::core::ffi::c_int
    }) as GXTevSwapSel;
    (*tevdesc).u.tevconf.tex_swap = (if (*tev).tex_swap as ::core::ffi::c_int
        == HSD_TE_UNDEF as ::core::ffi::c_int
    {
        0 as ::core::ffi::c_int
    } else {
        (*tev).tex_swap as ::core::ffi::c_int
    }) as GXTevSwapSel;
    (*tevdesc).u.tevconf.kcsel = (if (*tev).kcsel as ::core::ffi::c_int
        == HSD_TE_UNDEF as ::core::ffi::c_int
    {
        0 as ::core::ffi::c_int
    } else {
        (*tev).kcsel as ::core::ffi::c_int
    }) as GXTevKColorSel;
    (*tevdesc).u.tevconf.kasel = (if (*tev).kasel as ::core::ffi::c_int
        == HSD_TE_UNDEF as ::core::ffi::c_int
    {
        0 as ::core::ffi::c_int
    } else {
        (*tev).kasel as ::core::ffi::c_int
    }) as GXTevKAlphaSel;
    if (*tev).c_op as ::core::ffi::c_int == HSD_TE_UNDEF as ::core::ffi::c_int
        || (*tev).c_ref == 0 as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 8 as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 9 as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 0xa as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 0xb as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 0xc as ::core::ffi::c_int
            && (*tev).a_op as ::core::ffi::c_int != 0xd as ::core::ffi::c_int
    {
        (*tevdesc).u.tevconf.clr_op = GX_TEV_ADD;
        (*tevdesc).u.tevconf.clr_a = GX_CC_ZERO;
        (*tevdesc).u.tevconf.clr_b = GX_CC_ZERO;
        (*tevdesc).u.tevconf.clr_c = GX_CC_ZERO;
        if *init_cprev != 0 as ::core::ffi::c_int {
            *init_cprev = 0 as ::core::ffi::c_int;
            (*tevdesc).u.tevconf.clr_d = GX_CC_ZERO;
        } else {
            (*tevdesc).u.tevconf.clr_d = GX_CC_CPREV;
        }
        (*tevdesc).u.tevconf.clr_scale = GX_CS_SCALE_1;
        (*tevdesc).u.tevconf.clr_bias = GX_TB_ZERO;
        (*tevdesc).u.tevconf.clr_clamp = 0 as u8;
        (*tevdesc).u.tevconf.clr_out_reg = GX_TEVPREV;
    } else {
        (*tevdesc).u.tevconf.clr_op = (*tev).c_op as GXTevOp;
        (*tevdesc).u.tevconf.clr_a = (if (*tev).c_in
            [0 as ::core::ffi::c_int as usize]
            .arg as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            GX_CC_ZERO as ::core::ffi::c_int
        } else {
            (*tev).c_in[0 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
        }) as GXTevColorArg;
        (*tevdesc).u.tevconf.clr_b = (if (*tev).c_in
            [1 as ::core::ffi::c_int as usize]
            .arg as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            GX_CC_ZERO as ::core::ffi::c_int
        } else {
            (*tev).c_in[1 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
        }) as GXTevColorArg;
        (*tevdesc).u.tevconf.clr_c = (if (*tev).c_in
            [2 as ::core::ffi::c_int as usize]
            .arg as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            GX_CC_ZERO as ::core::ffi::c_int
        } else {
            (*tev).c_in[2 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
        }) as GXTevColorArg;
        (*tevdesc).u.tevconf.clr_d = (if (*tev).c_in
            [3 as ::core::ffi::c_int as usize]
            .arg as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            GX_CC_ZERO as ::core::ffi::c_int
        } else {
            (*tev).c_in[3 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
        }) as GXTevColorArg;
        (*tevdesc).u.tevconf.clr_scale = (if (*tev).c_scale
            as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            (*tev).c_scale as ::core::ffi::c_int
        }) as GXTevScale;
        (*tevdesc).u.tevconf.clr_bias = (if (*tev).c_bias as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            (*tev).c_bias as ::core::ffi::c_int
        }) as GXTevBias;
        (*tevdesc).u.tevconf.clr_clamp = (if (*tev).c_clamp
            as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as u8;
        if (*tev).c_dst as ::core::ffi::c_int
            != HSD_TE_UNDEF as ::core::ffi::c_int
        {
        } else {
            __assert(
                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                985 as u32,
                b"tev->c_dst != HSD_TE_UNDEF\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
        (*tevdesc).u.tevconf.clr_out_reg = dst[(*tev).c_dst as usize];
        if (*tevdesc).u.tevconf.clr_out_reg as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *init_cprev = 0 as ::core::ffi::c_int;
        }
    }
    if (*tev).a_op as ::core::ffi::c_int == HSD_TE_UNDEF as ::core::ffi::c_int
        || (*tev).a_ref == 0 as ::core::ffi::c_int
    {
        (*tevdesc).u.tevconf.alpha_op = GX_TEV_ADD;
        (*tevdesc).u.tevconf.alpha_a = GX_CA_ZERO;
        (*tevdesc).u.tevconf.alpha_b = GX_CA_ZERO;
        (*tevdesc).u.tevconf.alpha_c = GX_CA_ZERO;
        if *init_aprev != 0 as ::core::ffi::c_int {
            *init_aprev = 0 as ::core::ffi::c_int;
            (*tevdesc).u.tevconf.alpha_d = GX_CA_ZERO;
        } else {
            (*tevdesc).u.tevconf.alpha_d = GX_CA_APREV;
        }
        (*tevdesc).u.tevconf.alpha_scale = GX_CS_SCALE_1;
        (*tevdesc).u.tevconf.alpha_bias = GX_TB_ZERO;
        (*tevdesc).u.tevconf.alpha_clamp = 0 as u8;
        (*tevdesc).u.tevconf.alpha_out_reg = GX_TEVPREV;
    } else {
        (*tevdesc).u.tevconf.alpha_op = (*tev).a_op as GXTevOp;
        (*tevdesc).u.tevconf.alpha_a =
            (if (*tev).a_in[0 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                GX_CA_ZERO as ::core::ffi::c_int
            } else {
                (*tev).a_in[0 as ::core::ffi::c_int as usize].arg
                    as ::core::ffi::c_int
            }) as GXTevAlphaArg;
        (*tevdesc).u.tevconf.alpha_b =
            (if (*tev).a_in[1 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                GX_CA_ZERO as ::core::ffi::c_int
            } else {
                (*tev).a_in[1 as ::core::ffi::c_int as usize].arg
                    as ::core::ffi::c_int
            }) as GXTevAlphaArg;
        (*tevdesc).u.tevconf.alpha_c =
            (if (*tev).a_in[2 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                GX_CA_ZERO as ::core::ffi::c_int
            } else {
                (*tev).a_in[2 as ::core::ffi::c_int as usize].arg
                    as ::core::ffi::c_int
            }) as GXTevAlphaArg;
        (*tevdesc).u.tevconf.alpha_d =
            (if (*tev).a_in[3 as ::core::ffi::c_int as usize].arg
                as ::core::ffi::c_int
                == HSD_TE_UNDEF as ::core::ffi::c_int
            {
                GX_CA_ZERO as ::core::ffi::c_int
            } else {
                (*tev).a_in[3 as ::core::ffi::c_int as usize].arg
                    as ::core::ffi::c_int
            }) as GXTevAlphaArg;
        (*tevdesc).u.tevconf.alpha_scale = (if (*tev).a_scale
            as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            (*tev).a_scale as ::core::ffi::c_int
        }) as GXTevScale;
        (*tevdesc).u.tevconf.alpha_bias = (if (*tev).a_bias
            as ::core::ffi::c_int
            == HSD_TE_UNDEF as ::core::ffi::c_int
        {
            0 as ::core::ffi::c_int
        } else {
            (*tev).a_bias as ::core::ffi::c_int
        }) as GXTevBias;
        (*tevdesc).u.tevconf.alpha_clamp = (if (*tev).a_clamp
            as ::core::ffi::c_int
            != 0 as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as u8;
        if (*tev).a_dst as ::core::ffi::c_int
            != HSD_TE_UNDEF as ::core::ffi::c_int
        {
        } else {
            __assert(
                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                1024 as u32,
                b"tev->a_dst != HSD_TE_UNDEF\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
        (*tevdesc).u.tevconf.alpha_out_reg = dst[(*tev).a_dst as usize];
        if (*tevdesc).u.tevconf.alpha_out_reg as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        {
            *init_aprev = 0 as ::core::ffi::c_int;
        }
    }
    (*tevdesc).u.tevconf.mode = GX_TC_LINEAR;
}
static mut id1: [GXTevKColorID; 4] =
    [GX_KCOLOR0, GX_KCOLOR1, GX_KCOLOR2, GX_KCOLOR3];
static mut id2: [GXTevRegID; 3] = [GX_TEVREG0, GX_TEVREG1, GX_TEVREG2];
pub unsafe fn HSD_TExpSetReg(mut texp: *mut HSD_TExp) {
    melee_core::ct!("HSD_TExpSetReg");
    let mut i: ::core::ffi::c_int = 0;
    let mut reg: [GXColor; 8] = [_GXColor {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    }; 8];
    let mut changed: u32 = 0;
    let mut clist: *mut HSD_TECnst = ::core::ptr::null_mut::<HSD_TECnst>();
    clist = texp.cast::<HSD_TECnst>();
    changed = 0 as u32;
    while !clist.is_null() {
        if (*clist).r#type as ::core::ffi::c_int
            == HSD_TE_CNST as ::core::ffi::c_int
        {
        } else {
            __assert(
                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                1048 as u32,
                b"clist->type == HSD_TE_CNST\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
        if ((*clist).reg as ::core::ffi::c_int) < 8 as ::core::ffi::c_int {
            changed |= ((1 as ::core::ffi::c_int)
                << (*clist).reg as ::core::ffi::c_int)
                as u32;
            if (*clist).comp as ::core::ffi::c_int
                == HSD_TE_RGB as ::core::ffi::c_int
            {
                match (*clist).ctype as ::core::ffi::c_int {
                    0 => {
                        let mut col: GXColor = _GXColor {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 0,
                        };
                        col = *((*clist).val as *mut GXColor);
                        col.a = reg[(*clist).reg as usize].a;
                        reg[(*clist).reg as usize] = col;
                    }
                    2 => {
                        let mut ptr: *mut u32 = (*clist).val as *mut u32;
                        reg[(*clist).reg as usize].r =
                            (if *ptr.offset(0 as ::core::ffi::c_int as isize)
                                < 0x100 as u32
                            {
                                *ptr.offset(0 as ::core::ffi::c_int as isize)
                            } else {
                                0xff as u32
                            }) as u8;
                        reg[(*clist).reg as usize].g =
                            (if *ptr.offset(1 as ::core::ffi::c_int as isize)
                                < 0x100 as u32
                            {
                                *ptr.offset(1 as ::core::ffi::c_int as isize)
                            } else {
                                0xff as u32
                            }) as u8;
                        reg[(*clist).reg as usize].b =
                            (if *ptr.offset(2 as ::core::ffi::c_int as isize)
                                < 0x100 as u32
                            {
                                *ptr.offset(2 as ::core::ffi::c_int as isize)
                            } else {
                                0xff as u32
                            }) as u8;
                    }
                    _ => {
                        let mut r: ::core::ffi::c_int = 0;
                        let mut g: ::core::ffi::c_int = 0;
                        let mut b: ::core::ffi::c_int = 0;
                        match (*clist).ctype as ::core::ffi::c_int {
                            1 => {
                                let mut ptr_0: *mut u16 =
                                    (*clist).val as *mut u16;
                                r = *ptr_0
                                    .offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int;
                                g = *ptr_0
                                    .offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int;
                                b = *ptr_0
                                    .offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int;
                            }
                            3 => {
                                let mut ptr_1: *mut f32 =
                                    (*clist).val as *mut f32;
                                r = (0xff as ::core::ffi::c_int as f32
                                    * *ptr_1.offset(
                                        0 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                                g = (0xff as ::core::ffi::c_int as f32
                                    * *ptr_1.offset(
                                        1 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                                b = (0xff as ::core::ffi::c_int as f32
                                    * *ptr_1.offset(
                                        2 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                            }
                            _ => {
                                let mut ptr_2: *mut f64 =
                                    (*clist).val as *mut f64;
                                r = (0xff as ::core::ffi::c_int as f64
                                    * *ptr_2.offset(
                                        0 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                                g = (0xff as ::core::ffi::c_int as f64
                                    * *ptr_2.offset(
                                        1 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                                b = (0xff as ::core::ffi::c_int as f64
                                    * *ptr_2.offset(
                                        2 as ::core::ffi::c_int as isize,
                                    ))
                                    as ::core::ffi::c_int;
                            }
                        }
                        reg[(*clist).reg as usize].r =
                            (if r > 0xff as ::core::ffi::c_int {
                                0xff as ::core::ffi::c_int
                            } else if r < 0 as ::core::ffi::c_int {
                                0 as ::core::ffi::c_int
                            } else {
                                r
                            }) as u8;
                        reg[(*clist).reg as usize].g =
                            (if g > 0xff as ::core::ffi::c_int {
                                0xff as ::core::ffi::c_int
                            } else if g < 0 as ::core::ffi::c_int {
                                0 as ::core::ffi::c_int
                            } else {
                                g
                            }) as u8;
                        reg[(*clist).reg as usize].b =
                            (if b > 0xff as ::core::ffi::c_int {
                                0xff as ::core::ffi::c_int
                            } else if b < 0 as ::core::ffi::c_int {
                                0 as ::core::ffi::c_int
                            } else {
                                b
                            }) as u8;
                    }
                }
            } else {
                let mut x: ::core::ffi::c_int = 0;
                let mut val: u8 = 0;
                match (*clist).ctype as ::core::ffi::c_int {
                    0 => {
                        x = *((*clist).val as *mut u8) as ::core::ffi::c_int;
                        val = x as u8;
                    }
                    1 => {
                        x = *((*clist).val as *mut u16) as ::core::ffi::c_int;
                        val = (if x > 0xff as ::core::ffi::c_int {
                            0xff as ::core::ffi::c_int
                        } else if x < 0 as ::core::ffi::c_int {
                            0 as ::core::ffi::c_int
                        } else {
                            x
                        }) as u8;
                    }
                    2 => {
                        x = *((*clist).val as *mut u32) as ::core::ffi::c_int;
                        val = (if x > 0xff as ::core::ffi::c_int {
                            0xff as ::core::ffi::c_int
                        } else if x < 0 as ::core::ffi::c_int {
                            0 as ::core::ffi::c_int
                        } else {
                            x
                        }) as u8;
                    }
                    3 => {
                        x = (0xff as ::core::ffi::c_int as f32
                            * *((*clist).val as *mut f32))
                            as ::core::ffi::c_int;
                        val = (if x > 0xff as ::core::ffi::c_int {
                            0xff as ::core::ffi::c_int
                        } else if x < 0 as ::core::ffi::c_int {
                            0 as ::core::ffi::c_int
                        } else {
                            x
                        }) as u8;
                    }
                    _ => {
                        x = (0xff as ::core::ffi::c_int as f64
                            * *((*clist).val as *mut f64))
                            as ::core::ffi::c_int;
                        val = (if x > 0xff as ::core::ffi::c_int {
                            0xff as ::core::ffi::c_int
                        } else if x < 0 as ::core::ffi::c_int {
                            0 as ::core::ffi::c_int
                        } else {
                            x
                        }) as u8;
                    }
                }
                if ((*clist).reg as ::core::ffi::c_int)
                    < 4 as ::core::ffi::c_int
                {
                    match (*clist).idx as ::core::ffi::c_int {
                        0 => {
                            reg[(*clist).reg as usize].r = val;
                        }
                        1 => {
                            reg[(*clist).reg as usize].g = val;
                        }
                        2 => {
                            reg[(*clist).reg as usize].b = val;
                        }
                        _ => {
                            reg[(*clist).reg as usize].a = val;
                        }
                    }
                } else if (*clist).idx as ::core::ffi::c_int
                    == 3 as ::core::ffi::c_int
                {
                    reg[(*clist).reg as usize].a = val;
                } else {
                    reg[(*clist).reg as usize].r = val;
                    reg[(*clist).reg as usize].g = val;
                    reg[(*clist).reg as usize].b = val;
                }
            }
        }
        clist = (*clist).next.cast::<HSD_TECnst>();
    }
    if changed != 0 as u32 {
        GXPixModeSync();
        i = 0 as ::core::ffi::c_int;
        while i < 4 as ::core::ffi::c_int {
            if changed & ((1 as ::core::ffi::c_int) << i) as u32 != 0 {
                GXSetTevKColor(id1[i as usize], reg[i as usize]);
            }
            i += 1;
        }
        i = 4 as ::core::ffi::c_int;
        while i < 7 as ::core::ffi::c_int {
            if changed & ((1 as ::core::ffi::c_int) << i) as u32 != 0 {
                GXSetTevColor(
                    id2[(i - 4 as ::core::ffi::c_int) as usize],
                    reg[i as usize],
                );
            }
            i += 1;
        }
        HSD_StateInvalidate(0x10 as ::core::ffi::c_int);
        GXPixModeSync();
    }
}
pub unsafe fn HSD_TExpSetupTev(
    mut tevdesc: *mut HSD_TExpTevDesc,
    mut texp: *mut HSD_TExp,
) {
    melee_core::ct!("HSD_TExpSetupTev");
    HSD_TExpSetReg(texp);
    while !tevdesc.is_null() {
        if !(*tevdesc).tobj.is_null() {
            (*tevdesc).desc.map = (*(*tevdesc).tobj).id as u32;
            (*tevdesc).desc.coord = (*(*tevdesc).tobj).coord as u32;
        }
        HSD_StateAssignTev();
        HSD_SetupTevStage(&raw mut (*tevdesc).desc);
        tevdesc = (*tevdesc).desc.next as *mut HSD_TExpTevDesc;
    }
}
pub unsafe fn HSD_TExpCompile(
    mut texp: *mut HSD_TExp,
    mut tevdesc: *mut *mut HSD_TExpTevDesc,
    mut texp_list: *mut *mut HSD_TExp,
) -> ::core::ffi::c_int {
    melee_core::ct!("HSD_TExpCompile");
    let mut num: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut val: ::core::ffi::c_int = 0;
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
    let mut order: [*mut HSD_TExp; 32] =
        [::core::ptr::null_mut::<HSD_TExp>(); 32];
    let mut list: [HSD_TExpDag; 32] = [HSD_TExpDag {
        tev: ::core::ptr::null_mut::<_HSD_TETev>(),
        idx: 0,
        nb_dep: 0,
        nb_ref: 0,
        dist: 0,
        depend: [::core::ptr::null_mut::<HSD_TExpDag>(); 8],
    }; 32];
    let mut init_cprev: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut init_aprev: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    if !tevdesc.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            1191 as u32,
            b"tevdesc\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !texp_list.is_null() {
    } else {
        __assert(
            b"src/sysdolphin/baselib/texp.c\0" as *const u8
                as *const ::core::ffi::c_char,
            1192 as u32,
            b"texp_list\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    memset(
        &raw mut res as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<HSD_TExpRes>() as usize,
    );
    HSD_TExpRef(texp, HSD_TE_RGB as ::core::ffi::c_int as u8);
    HSD_TExpRef(texp, HSD_TE_A as ::core::ffi::c_int as u8);
    HSD_TExpSimplify(texp);
    num = HSD_TExpMakeDag(texp, &raw mut list as *mut HSD_TExpDag);
    HSD_TExpSchedule(
        num,
        &raw mut list as *mut HSD_TExpDag,
        &raw mut order as *mut *mut HSD_TExp,
        &raw mut res,
    );
    i = 0 as ::core::ffi::c_int;
    while i < num {
        val = TExpAssignReg(order[i as usize], &raw mut res);
        if val >= 0 as ::core::ffi::c_int {
        } else {
            __assert(
                b"src/sysdolphin/baselib/texp.c\0" as *const u8
                    as *const ::core::ffi::c_char,
                1204 as u32,
                b"val >= 0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        };
        i += 1;
    }
    i = num - 1 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        HSD_TExpSimplify2(order[i as usize]);
        i -= 1;
    }
    num = HSD_TExpMakeDag(texp, &raw mut list as *mut HSD_TExpDag);
    HSD_TExpSchedule(
        num,
        &raw mut list as *mut HSD_TExpDag,
        &raw mut order as *mut *mut HSD_TExp,
        &raw mut res,
    );
    *tevdesc = ::core::ptr::null_mut::<HSD_TExpTevDesc>();
    i = 0 as ::core::ffi::c_int;
    while i < num {
        let mut tdesc: *mut HSD_TExpTevDesc =
            hsdAllocMemPiece(::core::mem::size_of::<HSD_TExpTevDesc>() as s32)
                as *mut HSD_TExpTevDesc;
        (*tdesc).desc.stage = HSD_Index2TevStage(i) as u32;
        TExp2TevDesc(
            order[(num - i - 1 as ::core::ffi::c_int) as usize],
            tdesc,
            &raw mut init_cprev,
            &raw mut init_aprev,
        );
        (*tdesc).desc.next = (*tevdesc).cast::<HSD_TevDesc>();
        *tevdesc = tdesc;
        i += 1;
    }
    *texp_list = HSD_TExpFreeList(*texp_list, HSD_TE_TEV, 1 as s32);
    *texp_list = HSD_TExpFreeList(*texp_list, HSD_TE_CNST, 0 as s32);
    return num;
}
pub unsafe fn HSD_TExpFreeTevDesc(mut tdesc: *mut HSD_TExpTevDesc) {
    melee_core::ct!("HSD_TExpFreeTevDesc");
    let mut next: *mut HSD_TExpTevDesc = tdesc;
    while !next.is_null() {
        let mut temp: *mut HSD_TExpTevDesc =
            (*next).desc.next as *mut HSD_TExpTevDesc;
        hsdFreeMemPiece(
            next as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HSD_TExpTevDesc>() as s32,
        );
        next = temp;
    }
}
pub const NULL: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
