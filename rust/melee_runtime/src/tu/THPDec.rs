//! Native TARGET_PC THP decoder. All C definitions, including the bitstream and AAN helpers.
//! Asset words stay big endian; game state and generated records are native little endian.
//! Native-width scratch/predictor access replaces the original fixed GameCube offset views.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments,
         unused_variables, unused_unsafe, unused_parens, static_mut_refs, dead_code)]
use melee_types::*;
use melee_core::fp::fmadds;
use core::ffi::c_void;
use ext::*;

const fn native_work_prefix() -> usize { (core::mem::size_of::<THPFileInfo>() + 31) & !31 }
unsafe fn memset(dst: *mut c_void, v: i32, n: usize) -> *mut c_void {
    core::ptr::write_bytes(dst.cast::<u8>(), v as u8, n); dst
}

/// Platform cache and diagnostic hooks, retaining the C symbol names.
pub mod ext {
    use core::ffi::c_void;
    pub static mut DCZeroRange_HOOK: Option<unsafe fn(*mut c_void, u32)> = None;
    pub static mut LCStoreData_HOOK: Option<unsafe fn(*mut c_void, *mut c_void, u32) -> u32> = None;
    pub static mut LCQueueWait_HOOK: Option<unsafe fn(u32)> = None;
    pub static mut OSReport_HOOK: Option<unsafe fn(*mut i8)> = None;
    pub unsafe fn DCZeroRange(p: *mut c_void, n: u32) {
        (DCZeroRange_HOOK.expect("TODO: DCZeroRange"))(p,n)
    }
    pub unsafe fn LCStoreData(d: *mut c_void, s: *mut c_void, n: u32) -> u32 {
        (LCStoreData_HOOK.expect("TODO: LCStoreData"))(d,s,n)
    }
    pub unsafe fn LCQueueWait(n: u32) { (LCQueueWait_HOOK.expect("TODO: LCQueueWait"))(n) }
    pub unsafe fn OSReport(s: *mut i8) { (OSReport_HOOK.expect("TODO: OSReport"))(s) }
}
static mut __THP420Error: [i8; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [i8; 35]>(
        *b"ERROR: THP only supports 4:2:0!!!\n\0",
    )
};
static mut __THPJpegNaturalOrder: [u8; 80] = [
    0i32 as u8,
    1i32 as u8,
    8i32 as u8,
    16i32 as u8,
    9i32 as u8,
    2i32 as u8,
    3i32 as u8,
    10i32 as u8,
    17i32 as u8,
    24i32 as u8,
    32i32 as u8,
    25i32 as u8,
    18i32 as u8,
    11i32 as u8,
    4i32 as u8,
    5i32 as u8,
    12i32 as u8,
    19i32 as u8,
    26i32 as u8,
    33i32 as u8,
    40i32 as u8,
    48i32 as u8,
    41i32 as u8,
    34i32 as u8,
    27i32 as u8,
    20i32 as u8,
    13i32 as u8,
    6i32 as u8,
    7i32 as u8,
    14i32 as u8,
    21i32 as u8,
    28i32 as u8,
    35i32 as u8,
    42i32 as u8,
    49i32 as u8,
    56i32 as u8,
    57i32 as u8,
    50i32 as u8,
    43i32 as u8,
    36i32 as u8,
    29i32 as u8,
    22i32 as u8,
    15i32 as u8,
    23i32 as u8,
    30i32 as u8,
    37i32 as u8,
    44i32 as u8,
    51i32 as u8,
    58i32 as u8,
    59i32 as u8,
    52i32 as u8,
    45i32 as u8,
    38i32 as u8,
    31i32 as u8,
    39i32 as u8,
    46i32 as u8,
    53i32 as u8,
    60i32 as u8,
    61i32 as u8,
    54i32 as u8,
    47i32 as u8,
    55i32 as u8,
    62i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
    63i32 as u8,
];
static mut __THPAANScaleFactor: [f64; 8] = [
    1.0f32 as f64,
    1.387039845f32 as f64,
    1.306562965f32 as f64,
    1.175875602f32 as f64,
    1.0f32 as f64,
    0.785694958f32 as f64,
    0.541196100f32 as f64,
    0.275899379f32 as f64,
];
static mut Ydchuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Udchuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Vdchuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Yachuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Uachuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Vachuff: THPAlignedHuffmanTabPtr = THPAlignedHuffmanTabPtr {
    value: ::core::ptr::null::<THPHuffmanTab>() as *mut THPHuffmanTab,
    padding: [0; 28],
};
static mut Gbase: THPAlignedSamplePtr = THPAlignedSamplePtr {
    value: ::core::ptr::null::<THPSample>() as *mut THPSample,
    padding: [0; 28],
};
static mut Gwid: THPAlignedWidth = THPAlignedWidth {
    value: 0,
    padding: [0; 28],
};
static mut Gq: THPQuantizationPtr = THPQuantizationPtr {
    value: ::core::ptr::null::<f32>() as *mut f32,
    padding: 0,
};
pub unsafe extern "C" fn __THPSlw(mut v: u32, mut n: u32) -> u32 {
    return if n as u32 & 0x20 as u32 != 0 {
        0 as u32
    } else {
        v << (n as u32 & 0x1f as u32)
    };
}
pub unsafe extern "C" fn __THPSrw(mut v: u32, mut n: u32) -> u32 {
    return if n as u32 & 0x20 as u32 != 0 {
        0 as u32
    } else {
        v >> (n as u32 & 0x1f as u32)
    };
}
pub unsafe extern "C" fn __THPNextWord(mut info: *mut THPFileInfo) -> u32 {
    let mut p: *mut u32 = (*info).file as *mut u32;
    p = p.offset(1);
    (*info).file = p as *mut u8;
    (*info).currByte = core::ptr::read_unaligned(p).to_be();
    return core::ptr::read_unaligned(p).to_be();
}
pub unsafe extern "C" fn __THPGetBits(mut info: *mut THPFileInfo, mut t: u32) -> u32 {
    let mut cnt: u32 = (*info).cnt;
    let mut avail: u32 = (33 as u32).wrapping_sub(cnt);
    let mut hi: u32 = __THPSlw((*info).currByte, cnt.wrapping_sub(1 as u32));
    let mut v: u32 = 0;
    if t <= avail {
        v = hi >> (32 as u32).wrapping_sub(t);
        (*info).cnt = cnt.wrapping_add(t);
    } else {
        let mut lo: u32 = __THPNextWord(info);
        v = hi.wrapping_add(__THPSrw(lo, avail)) >> (32 as u32).wrapping_sub(t);
        (*info).cnt = (t as u32)
            .wrapping_sub(avail as u32)
            .wrapping_add(1 as u32) as u32;
    }
    return v;
}
pub unsafe extern "C" fn __THPPeek5(mut info: *const THPFileInfo) -> u32 {
    let mut cnt: u32 = (*info).cnt;
    let mut avail: u32 = (33 as u32).wrapping_sub(cnt);
    let mut hi: u32 = __THPSlw((*info).currByte, cnt.wrapping_sub(1 as u32));
    if avail >= 5 as u32 {
        return hi >> 27i32;
    }
    return hi.wrapping_add(__THPSrw(
        core::ptr::read_unaligned(((*info).file as *const u32).offset(1 as isize)).to_be(),
        avail,
    )) >> 27i32;
}
pub unsafe extern "C" fn __THPSkipBits(mut info: *mut THPFileInfo, mut t: u32) {
    let mut cnt: u32 = (*info).cnt;
    let mut avail: u32 = (33 as u32).wrapping_sub(cnt);
    if t <= avail {
        (*info).cnt = cnt.wrapping_add(t);
    } else {
        __THPNextWord(info);
        (*info).cnt = (t as u32)
            .wrapping_sub(avail as u32)
            .wrapping_add(1 as u32) as u32;
    };
}
pub unsafe extern "C" fn __THPHuffDecodePC(
    mut info: *mut THPFileInfo,
    mut h: *const THPHuffmanTab,
) -> s32 {
    let mut idx: u32 = __THPPeek5(info);
    let mut sym: u32 = (*h).quick[idx as usize] as u32;
    let mut code: s32 = 0;
    let mut l: s32 = 0;
    if sym != 0xff as u32 {
        __THPSkipBits(info, (*h).increment[idx as usize] as u32);
        return sym as s32;
    }
    __THPSkipBits(info, 5 as u32);
    code = idx as s32;
    l = 5i32 as s32;
    loop {
        code = code << 1i32 | __THPGetBits(info, 1 as u32) as s32;
        l += 1;
        if !(code > (*h).maxCode[l as usize]) {
            break;
        }
    }
    return *(*h).Vij.offset((code + (*h).valPtr[l as usize]) as isize) as s32;
}
pub unsafe extern "C" fn __THPExtend(mut v: u32, mut t: u32) -> s32 {
    if v < (1 as u32)
        << (t as u32).wrapping_sub(1 as u32)
    {
        return (v as u32).wrapping_add(
            ((0xffffffff as u32) << t).wrapping_add(1 as u32),
        ) as s32;
    }
    return v as s32;
}
pub unsafe extern "C" fn __THPHuffDecodeBlockPC(
    mut info: *mut THPFileInfo,
    mut block: *mut THPCoeff,
    mut dch: *const THPHuffmanTab,
    mut ach: *const THPHuffmanTab,
    mut predDC: *mut THPCoeff,
) {
    let mut diff: THPCoeff = 0 as THPCoeff;
    let mut t: s32 = 0;
    let mut k: s32 = 0;
    memset(
        block as *mut ::core::ffi::c_void,
        0i32,
        (64 as usize).wrapping_mul(::core::mem::size_of::<THPCoeff>() as usize) as usize,
    );
    t = __THPHuffDecodePC(info, dch);
    if t != 0i32 {
        diff = __THPExtend(__THPGetBits(info, t as u32), t as u32) as THPCoeff;
    }
    *predDC = (*predDC as i32 + diff as i32) as s16;
    *block.offset(0 as isize) = *predDC;
    k = 1i32 as s32;
    while k < 64i32 {
        let mut sym: s32 = __THPHuffDecodePC(info, ach);
        let mut size: s32 = sym & 15 as s32;
        let mut run: s32 = sym >> 4i32;
        if size != 0i32 {
            k += run;
            *block.offset(__THPJpegNaturalOrder[k as usize] as isize) =
                __THPExtend(__THPGetBits(info, size as u32), size as u32) as THPCoeff;
        } else {
            if run != 15i32 {
                break;
            }
            k += 15i32;
        }
        k += 1;
    }
}
pub unsafe extern "C" fn __THPStoreSample(mut dst: *mut THPSample, mut v: f32) {
    let mut i: s32 = (v as f32 * 0.125f32) as s32;
    if i < 0i32 {
        i = 0i32 as s32;
    } else if i > 255i32 {
        i = 255i32 as s32;
    }
    *dst = i as THPSample;
}
pub unsafe extern "C" fn __THPInverseDCTPC(
    mut in_0: *const THPCoeff,
    mut xPos: u32,
    mut rowOff: u32,
) {
    let mut q: *const f32 = Gq.value;
    let mut obase: *mut THPSample = Gbase.value;
    let mut wid: u32 = Gwid.value;
    let mut ws: [f32; 64] = [0.; 64];
    let mut out0: *mut THPSample = ::core::ptr::null_mut::<THPSample>();
    let mut out1: *mut THPSample = ::core::ptr::null_mut::<THPSample>();
    let mut i: i32 = 0;
    i = 0i32;
    while i < 8i32 {
        let mut ip: *const THPCoeff = in_0.offset((i * 8i32) as isize);
        let mut qp: *const f32 = q.offset((i * 8i32) as isize);
        let mut wp: *mut f32 =
            (&raw mut ws as *mut f32).offset((i * 8i32) as isize);
        let mut t0: f32 = 0.;
        let mut t1: f32 = 0.;
        let mut t2: f32 = 0.;
        let mut t3: f32 = 0.;
        let mut t4: f32 = 0.;
        let mut t5: f32 = 0.;
        let mut t6: f32 = 0.;
        let mut t7: f32 = 0.;
        let mut t10: f32 = 0.;
        let mut t11: f32 = 0.;
        let mut t12: f32 = 0.;
        let mut t13: f32 = 0.;
        let mut z5: f32 = 0.;
        let mut z10: f32 = 0.;
        let mut z11: f32 = 0.;
        let mut z12: f32 = 0.;
        let mut z13: f32 = 0.;
        if *ip.offset(1 as isize) as i32
            == 0i32
            && *ip.offset(2 as isize) as i32
                == 0i32
            && *ip.offset(3 as isize) as i32
                == 0i32
            && *ip.offset(4 as isize) as i32
                == 0i32
            && *ip.offset(5 as isize) as i32
                == 0i32
            && *ip.offset(6 as isize) as i32
                == 0i32
            && *ip.offset(7 as isize) as i32
                == 0i32
        {
            let mut dc: f32 = *ip.offset(0 as isize) as f32
                * *qp.offset(0 as isize);
            let ref mut fresh16 = *wp.offset(3 as isize);
            *fresh16 = dc;
            let ref mut fresh17 = *wp.offset(2 as isize);
            *fresh17 = *fresh16;
            let ref mut fresh18 = *wp.offset(1 as isize);
            *fresh18 = *fresh17;
            *wp.offset(0 as isize) = *fresh18;
            let ref mut fresh19 = *wp.offset(7 as isize);
            *fresh19 = dc;
            let ref mut fresh20 = *wp.offset(6 as isize);
            *fresh20 = *fresh19;
            let ref mut fresh21 = *wp.offset(5 as isize);
            *fresh21 = *fresh20;
            *wp.offset(4 as isize) = *fresh21;
        } else {
            t0 = *ip.offset(0 as isize) as f32
                * *qp.offset(0 as isize);
            t1 = *ip.offset(2 as isize) as f32
                * *qp.offset(2 as isize);
            t2 = *ip.offset(4 as isize) as f32
                * *qp.offset(4 as isize);
            t3 = *ip.offset(6 as isize) as f32
                * *qp.offset(6 as isize);
            t10 = t0 + t2;
            t11 = t0 - t2;
            t13 = t1 + t3;
            t12 = fmadds(t1 - t3, 1.414213562f32, -t13);
            t0 = t10 + t13;
            t3 = t10 - t13;
            t1 = t11 + t12;
            t2 = t11 - t12;
            t4 = *ip.offset(1 as isize) as f32
                * *qp.offset(1 as isize);
            t5 = *ip.offset(3 as isize) as f32
                * *qp.offset(3 as isize);
            t6 = *ip.offset(5 as isize) as f32
                * *qp.offset(5 as isize);
            t7 = *ip.offset(7 as isize) as f32
                * *qp.offset(7 as isize);
            z13 = t6 + t5;
            z10 = t6 - t5;
            z11 = t4 + t7;
            z12 = t4 - t7;
            t7 = z11 + z13;
            t11 = ((z11 as f32 - z13 as f32) * 1.414213562f32)
                as f32;
            z5 = ((z10 as f32 + z12 as f32) * 1.847759065f32)
                as f32;
            t10 = fmadds(1.082392200f32, z12, -z5);
            t12 = fmadds(-2.613125930f32, z10, z5);
            t6 = t12 - t7;
            t5 = t11 - t6;
            t4 = t10 + t5;
            *wp.offset(0 as isize) = t0 + t7;
            *wp.offset(7 as isize) = t0 - t7;
            *wp.offset(1 as isize) = t1 + t6;
            *wp.offset(6 as isize) = t1 - t6;
            *wp.offset(2 as isize) = t2 + t5;
            *wp.offset(5 as isize) = t2 - t5;
            *wp.offset(4 as isize) = t3 + t4;
            *wp.offset(3 as isize) = t3 - t4;
        }
        i += 1;
    }
    out0 = obase
        .offset(rowOff as isize)
        .offset(xPos.wrapping_mul(4 as u32) as isize);
    out1 = out0.offset(wid.wrapping_mul(4 as u32) as isize);
    i = 0i32;
    while i < 8i32 {
        let mut cp: *const f32 = (&raw mut ws as *mut f32).offset(i as isize);
        let mut t0_0: f32 = 0.;
        let mut t1_0: f32 = 0.;
        let mut t2_0: f32 = 0.;
        let mut t3_0: f32 = 0.;
        let mut t4_0: f32 = 0.;
        let mut t5_0: f32 = 0.;
        let mut t6_0: f32 = 0.;
        let mut t7_0: f32 = 0.;
        let mut t10_0: f32 = 0.;
        let mut t11_0: f32 = 0.;
        let mut t12_0: f32 = 0.;
        let mut t13_0: f32 = 0.;
        let mut z5_0: f32 = 0.;
        let mut z10_0: f32 = 0.;
        let mut z11_0: f32 = 0.;
        let mut z12_0: f32 = 0.;
        let mut z13_0: f32 = 0.;
        t10_0 = (*cp.offset(0 as isize) as f32
            + *cp.offset(32 as isize) as f32
            + 1024.0f32) as f32;
        t11_0 = (*cp.offset(0 as isize) as f32
            - *cp.offset(32 as isize) as f32
            + 1024.0f32) as f32;
        t13_0 = *cp.offset(16 as isize)
            + *cp.offset(48 as isize);
        t12_0 = fmadds((*cp.offset(16 as isize)
            - *cp.offset(48 as isize)), 1.414213562f32, -t13_0);
        t0_0 = t10_0 + t13_0;
        t3_0 = t10_0 - t13_0;
        t1_0 = t11_0 + t12_0;
        t2_0 = t11_0 - t12_0;
        z13_0 = *cp.offset(40 as isize)
            + *cp.offset(24 as isize);
        z10_0 = *cp.offset(40 as isize)
            - *cp.offset(24 as isize);
        z11_0 = *cp.offset(8 as isize)
            + *cp.offset(56 as isize);
        z12_0 = *cp.offset(8 as isize)
            - *cp.offset(56 as isize);
        t7_0 = z11_0 + z13_0;
        t11_0 = ((z11_0 as f32 - z13_0 as f32) * 1.414213562f32)
            as f32;
        z5_0 = ((z10_0 as f32 + z12_0 as f32) * 1.847759065f32)
            as f32;
        t10_0 = fmadds(1.082392200f32, z12_0, -z5_0);
        t12_0 = fmadds(-2.613125930f32, z10_0, z5_0);
        t6_0 = t12_0 - t7_0;
        t5_0 = t11_0 - t6_0;
        t4_0 = t10_0 + t5_0;
        __THPStoreSample(out0.offset(i as isize) as *mut THPSample, t0_0 + t7_0);
        __THPStoreSample(
            out0.offset((8i32 + i) as isize) as *mut THPSample,
            t1_0 + t6_0,
        );
        __THPStoreSample(
            out0.offset((16i32 + i) as isize) as *mut THPSample,
            t2_0 + t5_0,
        );
        __THPStoreSample(
            out0.offset((24i32 + i) as isize) as *mut THPSample,
            t3_0 - t4_0,
        );
        __THPStoreSample(out1.offset(i as isize) as *mut THPSample, t3_0 + t4_0);
        __THPStoreSample(
            out1.offset((8i32 + i) as isize) as *mut THPSample,
            t2_0 - t5_0,
        );
        __THPStoreSample(
            out1.offset((16i32 + i) as isize) as *mut THPSample,
            t1_0 - t6_0,
        );
        __THPStoreSample(
            out1.offset((24i32 + i) as isize) as *mut THPSample,
            t0_0 - t7_0,
        );
        i += 1;
    }
}
pub unsafe extern "C" fn __THPPrepBitStream(mut info: *mut THPFileInfo) {
    let mut ptr: *mut u32 = ::core::ptr::null_mut::<u32>();
    let mut offset: u32 = 0;
    let mut i: u32 = 0;
    let mut j: u32 = 0;
    let mut k: u32 = 0;
    ptr = ((*info).file as usize & !(3i32) as usize) as *mut u32;
    offset = ((*info).file as usize & 3 as usize) as u32;
    if (*info).cnt != 33 as u32 {
        (*info).cnt = (*info)
            .cnt
            .wrapping_sub((3 as u32).wrapping_sub(offset).wrapping_mul(8 as u32));
    } else {
        (*info).cnt = offset.wrapping_mul(8 as u32).wrapping_add(1 as u32);
    }
    (*info).file = ptr as *mut u8;
    (*info).currByte = core::ptr::read_unaligned(ptr).to_be();
    i = 0 as u32;
    while i < 4 as u32 {
        if (*info).validHuffmanTabs as i32 & (1i32) << i != 0 {
            j = 0 as u32;
            while j < 32 as u32 {
                (*info).huffmanTabs[i as usize].quick[j as usize] = 0xff as u8;
                k = 0 as u32;
                while k < 5 as u32 {
                    let mut code: s32 =
                        (j >> (5 as u32).wrapping_sub(k).wrapping_sub(1 as u32)) as s32;
                    if code
                        <= (*info).huffmanTabs[i as usize].maxCode
                            [k.wrapping_add(1 as u32) as usize]
                    {
                        (*info).huffmanTabs[i as usize].quick[j as usize] =
                            *(*info).huffmanTabs[i as usize].Vij.offset(
                                (code
                                    + (*info).huffmanTabs[i as usize].valPtr
                                        [k.wrapping_add(1 as u32) as usize])
                                    as isize,
                            );
                        (*info).huffmanTabs[i as usize].increment[j as usize] =
                            k.wrapping_add(1 as u32) as u8;
                        k = 99 as u32;
                    }
                    k = k.wrapping_add(1);
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
    let mut YdcTab: s32 = 0;
    let mut UdcTab: s32 = 0;
    let mut VdcTab: s32 = 0;
    let mut YacTab: s32 = 0;
    let mut UacTab: s32 = 0;
    let mut VacTab: s32 = 0;
    YdcTab = (((*info).components[0 as usize].DCTableSelector
        as i32)
        << 1i32) as s32;
    UdcTab = (((*info).components[1 as usize].DCTableSelector
        as i32)
        << 1i32) as s32;
    VdcTab = (((*info).components[2 as usize].DCTableSelector
        as i32)
        << 1i32) as s32;
    YacTab = ((((*info).components[0 as usize].ACTableSelector
        as i32)
        << 1i32)
        + 1i32) as s32;
    UacTab = ((((*info).components[1 as usize].ACTableSelector
        as i32)
        << 1i32)
        + 1i32) as s32;
    VacTab = ((((*info).components[2 as usize].ACTableSelector
        as i32)
        << 1i32)
        + 1i32) as s32;
    Ydchuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(YdcTab as isize)
        as *mut THPHuffmanTab;
    Udchuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(UdcTab as isize)
        as *mut THPHuffmanTab;
    Vdchuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(VdcTab as isize)
        as *mut THPHuffmanTab;
    Yachuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(YacTab as isize)
        as *mut THPHuffmanTab;
    Uachuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(UacTab as isize)
        as *mut THPHuffmanTab;
    Vachuff.value = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(VacTab as isize)
        as *mut THPHuffmanTab;
}
pub unsafe extern "C" fn THPDec_8032F8D4(
    mut data: *mut u8,
    mut out: *mut THPDec_8032FD40_Data,
) -> s32 {
    let mut hSample: [u8; 4] = [0; 4];
    let mut vSample: [u8; 4] = [0; 4];
    let mut componentId: [u8; 4] = [0; 4];
    let mut quantizationSelector: [u8; 4] = [0; 4];
    let mut marker: u8 = 0;
    let mut componentCount: u8 = 0;
    let mut tag: [u8; 5] = ::core::mem::transmute::<[u8; 5], [u8; 5]>(*b"JFIF\0");
    let mut i: u8 = 0;
    let mut valid: u8 = 0;
    let mut j: u32 = 0;
    let mut length: u16 = 0;
    valid = 0 as u8;
    memset(
        out as *mut ::core::ffi::c_void,
        0i32,
        0xc as usize,
    );
    let fresh28 = data;
    data = data.offset(1);
    let mut soi0: u8 = *fresh28;
    let fresh29 = data;
    data = data.offset(1);
    let mut soi1: u8 = *fresh29;
    if soi0 as i32 != 0xffi32
        || soi1 as i32 != 0xd8i32
    {
        return 0 as s32;
    }
    loop {
        let fresh30 = data;
        data = data.offset(1);
        if *fresh30 as i32 != 0xffi32 {
            return 0 as s32;
        }
        while *data as s8 as i32 == 0xffi32 {
            data = data.offset(1);
        }
        let fresh31 = data;
        data = data.offset(1);
        marker = *fresh31;
        if marker as i32 == 0xc0i32 {
            (*out)._pad = (*data.offset(4 as isize) as i32
                | (*data.offset(3 as isize) as i32)
                    << 8i32) as u16;
            (*out).val1 = (*data.offset(6 as isize) as i32
                | (*data.offset(5 as isize) as i32)
                    << 8i32) as u16;
            componentCount = *data.offset(7 as isize);
            data = data.offset(8 as isize);
            if componentCount as i32 != 3i32 {
                return 0 as s32;
            }
            i = 0 as u8;
            while (i as i32) < componentCount as i32 {
                let mut factors: u8 = 0;
                let fresh32 = data;
                data = data.offset(1);
                componentId[i as usize] = *fresh32;
                let fresh33 = data;
                data = data.offset(1);
                factors = *fresh33;
                hSample[i as usize] =
                    (factors as i32 >> 4i32) as u8;
                vSample[i as usize] =
                    (factors as i32 & 0xfi32) as u8;
                let fresh34 = data;
                data = data.offset(1);
                quantizationSelector[i as usize] = *fresh34;
                i = i.wrapping_add(1);
            }
            if hSample[0 as usize] as i32
                / hSample[1 as usize] as i32
                == 2i32
                && hSample[0 as usize] as i32
                    / hSample[2 as usize] as i32
                    == 2i32
            {
                if vSample[0 as usize] as i32
                    / vSample[1 as usize] as i32
                    == 2i32
                    && vSample[0 as usize] as i32
                        / vSample[2 as usize] as i32
                        == 2i32
                {
                    (*out).val2 = 4 as u8;
                } else if vSample[0 as usize] as i32
                    == vSample[1 as usize] as i32
                    && vSample[0 as usize] as i32
                        == vSample[2 as usize] as i32
                {
                    (*out).val2 = 2 as u8;
                }
            } else if hSample[0 as usize] as i32
                == hSample[1 as usize] as i32
                && hSample[0 as usize] as i32
                    == hSample[2 as usize] as i32
            {
                if vSample[0 as usize] as i32
                    == vSample[1 as usize] as i32
                    && vSample[0 as usize] as i32
                        == vSample[2 as usize] as i32
                {
                    (*out).val2 = 1 as u8;
                }
            } else {
                return 0 as s32;
            }
        } else if marker as i32 == 0xe0i32 {
            let fresh35 = data;
            data = data.offset(1);
            length = *fresh35 as u16;
            let fresh36 = data;
            data = data.offset(1);
            length = ((length as i32) << 8i32
                | *fresh36 as i32) as u16;
            i = 0 as u8;
            while (i as i32) < 5i32 {
                let fresh37 = data;
                data = data.offset(1);
                componentCount = *fresh37;
                if componentCount as i32 != tag[i as usize] as i32 {
                    return 0 as s32;
                }
                i = i.wrapping_add(1);
            }
            valid = 1 as u8;
            j = 0 as u32;
            while j < (length as i32 - 7i32) as u32 {
                data = data.offset(1);
                j = j.wrapping_add(1);
            }
        } else {
            if marker as i32 == 0xdai32 {
                break;
            }
            if 0xc0i32 <= marker as i32
                && marker as i32 <= 0xfei32
            {
                length = (*data.offset(1 as isize) as i32
                    | (*data.offset(0 as isize) as i32)
                        << 8i32) as u16;
                data = data.offset(2 as isize);
                j = 0 as u32;
                while j < (length as i32 - 2i32) as u32 {
                    data = data.offset(1);
                    j = j.wrapping_add(1);
                }
            }
        }
        if (*out).val2 as i32 != 0i32
            && valid as i32 != 0i32
        {
            break;
        }
    }
    return 1 as s32;
}
pub unsafe extern "C" fn THPDec_8032FD40(
    mut data: *mut THPDec_8032FD40_Data,
    mut num: u16,
) -> s32 {
    let mut base: s32 = (*data).val0 + 0x4028 as s32;
    let mut a: s32 = 0;
    if (*data).val2 as i32 != 4i32 {
        OSReport(&raw mut __THP420Error as *mut i8);
        return 0 as s32;
    }
    a = ((*data).val1 as i32 / 2i32
        * (num as i32 / 2i32)
        * 2i32
        + (*data).val1 as i32 * num as i32) as s32;
    base = base + a;
    return base;
}
pub unsafe extern "C" fn THPVideoDecode(
    mut hdr: *mut ::core::ffi::c_void,
    mut status_out: *mut ::core::ffi::c_void,
    mut work: *mut THPFileInfo,
    mut data: *mut ::core::ffi::c_void,
    mut desc: *mut THPDec_8032FD40_Data,
) -> *mut THPFileInfo {
    let mut current_block: u64;
    let mut done: u8 = 0;
    let mut info: *mut THPFileInfo = work;
    let mut header: *mut THPVideoDecodeHeader = hdr as *mut THPVideoDecodeHeader;
    let mut statusOut: *mut u8 = status_out as *mut u8;
    let mut status: u8 = 0;
    let mut length: s32 = 0;
    let mut i: u32 = 0;
    DCZeroRange(info as *mut ::core::ffi::c_void, native_work_prefix() as u32);
    (*info).scratch = info as *mut u8;
    (*info).scratch = (*info).scratch.offset(native_work_prefix() as isize);
    (*info).xSize = (*header).xSize;
    (*info).ySize = (*header).ySize;
    (*info).file = (*info).scanStart;
    (*info).cnt = 33 as u32;
    (*info).x8EC = 0 as u16;
    (*info).x8EE = 0 as u16;
    (*info).x8D2 = 0 as u16;
    (*info).x8E8 = 0 as u16;
    (*info).x8EA = 0 as u16;
    (*info).x7D = 0 as u8;
    (*info).dataStart = data as *mut u8;
    THPDec_803300E0(info);
    done = 0 as u8;
    (*info).file = (*info).dataStart;
    loop {
        let fresh22 = (*info).file;
        (*info).file = (*info).file.offset(1);
        if *fresh22 as i32 != 0xffi32 {
            current_block = 9214795318326493763;
            break;
        }
        while *(*info).file as i32 == 0xffi32 {
            (*info).file = (*info).file.offset(1);
        }
        let fresh23 = (*info).file;
        (*info).file = (*info).file.offset(1);
        status = *fresh23;
        if status as i32 <= 0xd7i32 {
            if status as i32 == 0xc4i32 {
                status = __THPReadHuffmanTableSpecification(info);
                if status as i32 != 0i32 {
                    current_block = 12001599333083166235;
                    break;
                }
            } else if status as i32 == 0xc0i32 {
                status = __THPReadFrameHeader(info);
                if status as i32 != 0i32 {
                    current_block = 12001599333083166235;
                    break;
                }
            } else {
                *statusOut = 11 as u8;
                return ::core::ptr::null_mut::<THPFileInfo>();
            }
        } else if 0xd8i32 <= status as i32
            && status as i32 <= 0xdfi32
        {
            if status as i32 == 0xddi32 {
                __THPRestartDefinition(info);
            } else if status as i32 == 0xdbi32 {
                status = __THPReadQuantizationTable(info);
                if status as i32 != 0i32 {
                    current_block = 12001599333083166235;
                    break;
                }
            } else if status as i32 == 0xdai32 {
                status = __THPReadScaneHeader(info);
                if status as i32 != 0i32 {
                    current_block = 12001599333083166235;
                    break;
                }
                done = 1 as u8;
                (*info).scanStart = (*info).file;
            } else if status as i32 != 0xd8i32 {
                *statusOut = 11 as u8;
                return ::core::ptr::null_mut::<THPFileInfo>();
            }
        } else if 0xe0i32 <= status as i32 {
            if status as i32 == 0xe0i32 {
                status = THPDec_80330158(info);
                if status as i32 != 0i32 {
                    current_block = 12001599333083166235;
                    break;
                }
            } else if 0xe1i32 <= status as i32
                && status as i32 <= 0xefi32
            {
                THPDec_803302EC(&raw mut (*info).file);
            } else if status as i32 == 0xfei32 {
                length = ((*(*info).file.offset(0 as isize)
                    as i32)
                    << 8i32
                    | *(*info).file.offset(1 as isize) as i32)
                    as s32;
                (*info).file = (*info).file.offset(2 as isize);
                i = 0 as u32;
                while i < (length as i32 - 2i32) as u32 {
                    (*info).file = (*info).file.offset(1);
                    i = i.wrapping_add(1);
                }
            } else {
                *statusOut = 11 as u8;
                return ::core::ptr::null_mut::<THPFileInfo>();
            }
        }
        if done != 0 {
            *statusOut = 0 as u8;
            return work;
        }
    }
    match current_block {
        9214795318326493763 => {
            *statusOut = 3 as u8;
        }
        _ => {
            *statusOut = status;
        }
    }
    return ::core::ptr::null_mut::<THPFileInfo>();
}
pub unsafe extern "C" fn THPDec_803300E0(mut info: *mut THPFileInfo) {
    let mut p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut i: u8 = 0;
    p = (((*info).scratch as usize).wrapping_add(31) & !31usize) as *mut u8;
    (*info).scratch = p.offset((16i32 * 128i32) as isize);
    i = 0 as u8;
    while (i as i32) < 16i32 {
        (*info).mcuBuffer[i as usize] = p as *mut THPCoeff;
        p = p.offset(128 as isize);
        i = i.wrapping_add(1);
    }
}
pub unsafe extern "C" fn THPDec_80330158(mut info: *mut THPFileInfo) -> u8 {
    let mut tag: [u8; 5] = ::core::mem::transmute::<[u8; 5], [u8; 5]>(*b"JFIF\0");
    let mut length: u16 = 0;
    let mut i: u32 = 0;
    let mut xThumb: u8 = 0;
    let mut yThumb: u8 = 0;
    let mut version: u16 = 0;
    let mut units: u8 = 0;
    let mut segmentLength: u32 = 0;
    length = ((*(*info).file.offset(0 as isize) as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    i = 0 as u32;
    while i < 5 as u32 {
        let fresh24 = (*info).file;
        (*info).file = (*info).file.offset(1);
        if *fresh24 as i32 != tag[i as usize] as i32 {
            return 3 as u8;
        }
        i = i.wrapping_add(1);
    }
    version = ((*(*info).file.offset(0 as isize) as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    let fresh25 = (*info).file;
    (*info).file = (*info).file.offset(1);
    units = *fresh25;
    (*info).file = (*info).file.offset(1 as isize);
    (*info).file = (*info).file.offset(1 as isize);
    (*info).file = (*info).file.offset(1 as isize);
    (*info).file = (*info).file.offset(1 as isize);
    let fresh26 = (*info).file;
    (*info).file = (*info).file.offset(1);
    xThumb = *fresh26;
    let fresh27 = (*info).file;
    (*info).file = (*info).file.offset(1);
    yThumb = *fresh27;
    if xThumb as i32 != 0i32
        || yThumb as i32 != 0i32
    {
        return 7 as u8;
    }
    segmentLength = (*info).file.offset_from((*info).dataStart) as i64 as u32;
    if (length as i32 + 4i32) as u32 != segmentLength {
        return 8 as u8;
    }
    return 0 as u8;
}
pub unsafe extern "C" fn THPDec_803302EC(mut data: *mut *mut u8) -> s32 {
    let mut i: u32 = 0;
    let mut ptr: *mut u8 = *data;
    let mut high: u16 = *ptr.offset(0 as isize) as u16;
    let mut low: u16 = *ptr.offset(1 as isize) as u16;
    let mut count: u16 = ((high as i32) << 8i32
        | low as i32) as u16;
    i = 0 as u32;
    while i < count as u32 {
        *data = (*data).offset(1);
        i = i.wrapping_add(1);
    }
    return 0 as s32;
}
pub unsafe extern "C" fn __THPReadFrameHeader(mut info: *mut THPFileInfo) -> u8 {
    let mut i: u8 = 0;
    let mut j: u8 = 0;
    let mut k: u8 = 0;
    let mut comp: *mut THPComponent = ::core::ptr::null_mut::<THPComponent>();
    let mut utmp8: u8 = 0;
    let mut ySize: u16 = 0;
    (*info).file = (*info).file.offset(2 as isize);
    let fresh0 = (*info).file;
    (*info).file = (*info).file.offset(1);
    utmp8 = *fresh0;
    if utmp8 as i32 != 8i32 {
        return 10 as u8;
    }
    (*info).yPixelSize = ((*(*info).file.offset(0 as isize)
        as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    (*info).xPixelSize = ((*(*info).file.offset(0 as isize)
        as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    let fresh1 = (*info).file;
    (*info).file = (*info).file.offset(1);
    (*info).nComponents = *fresh1;
    if (*info).nComponents as i32 != 3i32
        && (*info).nComponents as i32 != 1i32
    {
        return 12 as u8;
    }
    i = 0 as u8;
    while (i as i32) < (*info).nComponents as i32 {
        let fresh2 = (*info).file;
        (*info).file = (*info).file.offset(1);
        (*info).components[i as usize].componentID = *fresh2;
        let fresh3 = (*info).file;
        (*info).file = (*info).file.offset(1);
        utmp8 = *fresh3;
        (*info).components[i as usize].samplingH =
            (utmp8 as i32 >> 4i32) as u8;
        (*info).components[i as usize].samplingV =
            (utmp8 as i32 & 0xfi32) as u8;
        let fresh4 = (*info).file;
        (*info).file = (*info).file.offset(1);
        (*info).components[i as usize].quantizationTableSelector = *fresh4;
        i = i.wrapping_add(1);
    }
    (*info).samplingHMax = 1 as u8;
    (*info).samplingVMax = 1 as u8;
    j = 0 as u8;
    while (j as i32) < (*info).nComponents as i32 {
        comp = (&raw mut (*info).components as *mut THPComponent).offset(j as isize)
            as *mut THPComponent;
        (*info).samplingHMax = (if (*info).samplingHMax as i32
            > (*comp).samplingH as i32
        {
            (*info).samplingHMax as i32
        } else {
            (*comp).samplingH as i32
        }) as u8;
        (*info).samplingVMax = (if (*info).samplingVMax as i32
            > (*comp).samplingV as i32
        {
            (*info).samplingVMax as i32
        } else {
            (*comp).samplingV as i32
        }) as u8;
        j = j.wrapping_add(1);
    }
    ySize = (*info).yPixelSize;
    (*info).x8D4 = ((ySize as s32 as i32
        + (((ySize as s32 as i32
            + (((*info).samplingVMax as i32 * 8i32)
                as i32
                - 1i32))
            / ((*info).samplingVMax as i32 * 8i32)
                as i32) as s32 as i32
            - 1i32))
        / ((ySize as s32 as i32
            + (((*info).samplingVMax as i32 * 8i32)
                as i32
                - 1i32))
            / ((*info).samplingVMax as i32 * 8i32)
                as i32) as s32 as i32) as u16;
    k = 0 as u8;
    while (k as i32) < (*info).nComponents as i32 {
        (*info).components[k as usize].x08 = ((((*info).xPixelSize as i32
            * (*info).components[k as usize].samplingH as i32)
            as i32
            + ((*info).samplingHMax as s32 as i32 - 1i32))
            / (*info).samplingHMax as s32 as i32)
            as u32;
        (*info).components[k as usize].x0C = ((((*info).yPixelSize as i32
            * (*info).components[k as usize].samplingV as i32)
            as i32
            + ((*info).samplingVMax as s32 as i32 - 1i32))
            / (*info).samplingVMax as s32 as i32)
            as u32;
        k = k.wrapping_add(1);
    }
    return 0 as u8;
}
pub unsafe extern "C" fn __THPReadQuantizationTable(mut info: *mut THPFileInfo) -> u8 {
    let mut length: u16 = 0;
    let mut id: u16 = 0;
    let mut i: u16 = 0;
    let mut row: u16 = 0;
    let mut col: u16 = 0;
    let mut q_temp: [f32; 64] = [0.; 64];
    length = ((*(*info).file.offset(0 as isize) as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    length = (length as i32 - 2i32) as u16;
    loop {
        let fresh9 = (*info).file;
        (*info).file = (*info).file.offset(1);
        id = *fresh9 as u16;
        i = 0 as u16;
        while (i as i32) < 64i32 {
            let fresh10 = (*info).file;
            (*info).file = (*info).file.offset(1);
            q_temp[__THPJpegNaturalOrder[i as usize] as usize] = *fresh10 as f32;
            i = i.wrapping_add(1);
        }
        (*info).validQuantTabs = ((*info).validQuantTabs as i32
            | (1i32) << id as i32)
            as u8;
        i = 0 as u16;
        row = 0 as u16;
        while (row as i32) < 8i32 {
            col = 0 as u16;
            while (col as i32) < 8i32 {
                (*info).quantTabs[id as usize][i as usize] = (q_temp[i as usize] as f64
                    * __THPAANScaleFactor[row as usize]
                    * __THPAANScaleFactor[col as usize])
                    as f32;
                i = i.wrapping_add(1);
                col = col.wrapping_add(1);
            }
            row = row.wrapping_add(1);
        }
        length = (length as i32 - 65i32) as u16;
        if length == 0 {
            break;
        }
    }
    return 0 as u8;
}
pub unsafe extern "C" fn __THPReadHuffmanTableSpecification(mut info: *mut THPFileInfo) -> u8 {
    let mut t_class: u8 = 0;
    let mut id: u8 = 0;
    let mut i: u8 = 0;
    let mut tab_index: u8 = 0;
    let mut length: u16 = 0;
    let mut num_Vij: u16 = 0;
    let mut huffmanBits: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut result: u8 = 0;
    length = ((*(*info).file.offset(0 as isize) as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    length = (length as i32 - 2i32) as u16;
    loop {
        let fresh11 = (*info).file;
        (*info).file = (*info).file.offset(1);
        i = *fresh11;
        id = (i as i32 & 15i32) as u8;
        t_class = (i as i32 >> 4i32) as u8;
        huffmanBits = (*info).file;
        tab_index = (((id as i32) << 1i32)
            + t_class as i32) as u8;
        num_Vij = 0 as u16;
        i = 0 as u8;
        while (i as i32) < 16i32 {
            let fresh12 = (*info).file;
            (*info).file = (*info).file.offset(1);
            num_Vij = (num_Vij as i32 + *fresh12 as i32) as u16;
            i = i.wrapping_add(1);
        }
        (*info).huffmanTabs[tab_index as usize].bits = huffmanBits;
        (*info).huffmanTabs[tab_index as usize].Vij = (*info).file;
        (*info).huffmanTabs[tab_index as usize].numVij = num_Vij;
        (*info).file = (*info).file.offset(num_Vij as i32 as isize);
        result = __THPHuffGenerateSizeTable(info, tab_index, huffmanBits as usize);
        if result != 0 {
            return result;
        }
        result = __THPHuffGenerateCodeTable(info, tab_index);
        if result != 0 {
            return result;
        }
        __THPHuffGenerateDecoderTables(info, tab_index);
        (*info).validHuffmanTabs = ((*info).validHuffmanTabs as i32
            | (1i32) << tab_index as i32)
            as u8;
        length = (length as i32
            - (17i32 + num_Vij as i32)) as u16;
        if length as i32 == 0i32 {
            break;
        }
    }
    return 0 as u8;
}
pub unsafe extern "C" fn __THPReadScaneHeader(mut info: *mut THPFileInfo) -> u8 {
    let mut numComponents: u8 = 0;
    let mut i: u8 = 0;
    let mut ptr: *mut u8 = ::core::ptr::null_mut::<u8>();
    (*info).file = (*info).file.offset(2 as isize);
    ptr = (*info).file;
    (*info).file = (*info).file.offset(1);
    numComponents = *ptr;
    if numComponents as i32 != (*info).nComponents as i32 {
        return 12 as u8;
    }
    i = 0 as u8;
    while (i as i32) < numComponents as i32 {
        let mut selectors: u8 = 0;
        let mut blocksPerRow: u16 = 0;
        let mut rows: u16 = 0;
        let mut shift: s32 = 0;
        let fresh5 = (*info).file;
        (*info).file = (*info).file.offset(1);
        selectors = *fresh5;
        let fresh6 = (*info).file;
        (*info).file = (*info).file.offset(1);
        selectors = *fresh6;
        (*info).components[i as usize].DCTableSelector =
            (selectors as i32 >> 4i32) as u8;
        (*info).components[i as usize].ACTableSelector =
            (selectors as i32 & 0xfi32) as u8;
        if (*info).validHuffmanTabs as i32
            & (1i32)
                << (selectors as i32 >> 4i32)
            == 0
        {
            return 15 as u8;
        }
        if (*info).validHuffmanTabs as i32
            & (1i32)
                << (selectors as i32 & 0xfi32)
                    + 1i32
            == 0
        {
            return 15 as u8;
        }
        (*info).x74 = (*info).xSize;
        (*info).decompressedY = (*info).ySize;
        blocksPerRow = (*info).components[i as usize].x08 as u16;
        rows = (((*info).decompressedY as s32 as i32
            + ((*info).x8D4 as s32 as i32 - 1i32))
            / (*info).x8D4 as s32 as i32) as u16;
        rows = (rows as i32
            + if (*info).decompressedY as i32 % (*info).x8D4 as i32
                == 0i32
            {
                0i32
            } else {
                1i32
            }) as u16;
        rows = (rows as i32 * (*info).x8D4 as i32) as u16;
        shift = ((*info).samplingVMax as i32
            - (*info).components[i as usize].samplingV as i32)
            as s32;
        rows = (rows as i32 >> shift) as u16;
        (*info).components[i as usize].x10 = (*info).scratch as usize as u32;
        (*info).scratch = (*info)
            .scratch
            .offset((blocksPerRow as i32 * rows as i32) as isize);
        i = i.wrapping_add(1);
    }
    (*info).file = (*info).file.offset(3 as isize);
    return THPDec_803310CC(info);
}
pub unsafe extern "C" fn __THPHuffGenerateSizeTable(
    mut info: *mut THPFileInfo,
    mut tab_index: u8,
    mut huffmanBits: usize,
) -> u8 {
    let mut bits: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut p: s32 = 0;
    let mut l: s32 = 0;
    let mut i: s32 = 0;
    bits = huffmanBits as *mut u8;
    p = 0i32 as s32;
    l = 1i32 as s32;
    while l <= 16i32 {
        p += *bits.offset((l as i32 - 1i32) as isize)
            as i32;
        l += 1;
    }
    (*info).huffmanTabs[tab_index as usize].sizeTab = (*info).scratch as *mut s8;
    (*info).scratch = (*info)
        .scratch
        .offset((p as i32 + 1i32) as isize);
    p = 0i32 as s32;
    l = 1i32 as s32;
    while l <= 16i32 {
        i = *bits.offset((l as i32 - 1i32) as isize) as s32;
        loop {
            let fresh14 = i;
            i = i - 1;
            if !(fresh14 != 0) {
                break;
            }
            let fresh15 = p;
            p = p + 1;
            *(*info).huffmanTabs[tab_index as usize]
                .sizeTab
                .offset(fresh15 as isize) = l as s8;
        }
        l += 1;
    }
    *(*info).huffmanTabs[tab_index as usize]
        .sizeTab
        .offset(p as isize) = 0 as s8;
    (*info).huffmanTabs[tab_index as usize].numCodes = p as u32;
    return 0 as u8;
}
pub unsafe extern "C" fn __THPHuffGenerateCodeTable(
    mut info: *mut THPFileInfo,
    mut tab_index: u8,
) -> u8 {
    let mut si: s32 = 0;
    let mut code: u32 = 0;
    let mut p: s32 = 0;
    p = 0i32 as s32;
    si = *(*info).huffmanTabs[tab_index as usize]
        .sizeTab
        .offset(0 as isize) as s32;
    while *(*info).huffmanTabs[tab_index as usize]
        .sizeTab
        .offset(p as isize)
        != 0
    {
        while *(*info).huffmanTabs[tab_index as usize]
            .sizeTab
            .offset(p as isize) as i32
            == si
        {
            p += 1;
        }
        si += 1;
    }
    (*info).huffmanTabs[tab_index as usize].codeTab = (*info).scratch as *mut u32;
    (*info).scratch = (*info)
        .scratch
        .offset((p as usize).wrapping_mul(::core::mem::size_of::<u32>() as usize) as isize);
    p = 0i32 as s32;
    code = 0 as u32;
    si = *(*info).huffmanTabs[tab_index as usize]
        .sizeTab
        .offset(0 as isize) as s32;
    while *(*info).huffmanTabs[tab_index as usize]
        .sizeTab
        .offset(p as isize)
        != 0
    {
        while *(*info).huffmanTabs[tab_index as usize]
            .sizeTab
            .offset(p as isize) as i32
            == si
        {
            let fresh13 = p;
            p = p + 1;
            core::ptr::write_unaligned((*info).huffmanTabs[tab_index as usize]
                .codeTab
                .offset(fresh13 as isize), code);
            code = code.wrapping_add(1);
        }
        code <<= 1i32;
        si += 1;
    }
    return 0 as u8;
}
pub unsafe extern "C" fn __THPHuffGenerateDecoderTables(
    mut info: *mut THPFileInfo,
    mut tabIndex: u8,
) -> i32 {
    let mut p: s32 = 0;
    let mut l: s32 = 0;
    let mut h: *mut THPHuffmanTab = ::core::ptr::null_mut::<THPHuffmanTab>();
    p = 0i32 as s32;
    h = (&raw mut (*info).huffmanTabs as *mut THPHuffmanTab).offset(tabIndex as isize)
        as *mut THPHuffmanTab;
    l = 1i32 as s32;
    while l <= 16i32 {
        if *(*h)
            .bits
            .offset((l as i32 - 1i32) as isize)
            != 0
        {
            (*h).valPtr[l as usize] =
                (p as u32).wrapping_sub(core::ptr::read_unaligned((*h).codeTab.offset(p as isize))) as s32;
            p += *(*h)
                .bits
                .offset((l as i32 - 1i32) as isize)
                as i32;
            (*h).maxCode[l as usize] = core::ptr::read_unaligned((*h).codeTab.offset((p - 1) as isize))
                as s32;
        } else {
            (*h).maxCode[l as usize] = -(1i32) as s32;
            (*h).valPtr[l as usize] = -(1i32) as s32;
        }
        l += 1;
    }
    (*h).maxCode[17 as usize] = 0xfffffi32 as s32;
    return 0i32;
}
pub unsafe extern "C" fn THPDec_803310CC(mut info: *mut THPFileInfo) -> u8 {
    let mut i: u32 = 0;
    let mut j: s32 = 0;
    (*info).MCUsPerRow = (((*info).xPixelSize as s32 as i32
        + (((*info).samplingHMax as i32 * 8i32)
            as i32
            - 1i32))
        / ((*info).samplingHMax as i32 * 8i32)
            as i32) as u16;
    (*info).x8D0 = (((*info).yPixelSize as s32 as i32
        + (((*info).samplingVMax as i32 * 8i32)
            as i32
            - 1i32))
        / ((*info).samplingVMax as i32 * 8i32)
            as i32) as u16;
    (*info).x8CE = 0 as u16;
    i = 0 as u32;
    while i < (*info).nComponents as u32 {
        let mut c: *mut THPComponent = (&raw mut (*info).components as *mut THPComponent)
            .offset(i as isize) as *mut THPComponent;
        (*c).x28 = ((((*info).xPixelSize as i32
            * (*c).samplingH as i32) as i32
            + (((*info).samplingHMax as i32 * 8i32)
                as i32
                - 1i32))
            / ((*info).samplingHMax as i32 * 8i32)
                as i32) as s32;
        (*c).x24 = ((((*info).yPixelSize as i32
            * (*c).samplingV as i32) as i32
            + (((*info).samplingVMax as i32 * 8i32)
                as i32
                - 1i32))
            / ((*info).samplingVMax as i32 * 8i32)
                as i32) as s32;
        (*c).x14 = (*c).samplingH as s32;
        (*c).x18 = (*c).samplingV as s32;
        (*c).x1C = (*c).x14 * (*c).x18;
        (*c).x20 = ((*c).x14 as i32 * 8i32) as s32;
        j = (*c).x1C;
        if (*info).x8CE as s32 + (*c).x1C > 0x10i32 {
            return 0x11 as u8;
        }
        loop {
            let fresh7 = j;
            j = j - 1;
            if !(fresh7 > 0i32) {
                break;
            }
            let fresh8 = (*info).x8CE;
            (*info).x8CE = (*info).x8CE.wrapping_add(1);
            (*info).x8BC[fresh8 as usize] = i as u8;
        }
        if (*info).x8CE as i32 > 6i32 {
            OSReport(
                b"THP does not support anything other than 4:2:0!\n\0" as *const u8
                    as *const i8 as *mut i8,
            );
            return 0 as u8;
        }
        (*c).predDC = 0 as THPCoeff;
        i = i.wrapping_add(1);
    }
    return 0 as u8;
}
pub unsafe extern "C" fn __THPRestartDefinition(mut info: *mut THPFileInfo) -> u8 {
    (*info).RST = 1 as u8;
    (*info).file = (*info).file.offset(2 as isize);
    (*info).nMCU = ((*(*info).file.offset(0 as isize) as i32)
        << 8i32
        | *(*info).file.offset(1 as isize) as i32)
        as u16;
    (*info).file = (*info).file.offset(2 as isize);
    (*info).currMCU = (*info).nMCU;
    return 0 as u8;
}
pub unsafe extern "C" fn THPDec_80331340(
    mut info: *mut THPFileInfo,
    mut tileY: *mut ::core::ffi::c_void,
    mut tileU: *mut ::core::ffi::c_void,
    mut tileV: *mut ::core::ffi::c_void,
) {
    (*info).tileY = tileY as *mut u8;
    (*info).tileU = tileU as *mut u8;
    (*info).tileV = tileV as *mut u8;
    __THPPrepBitStream(info);
    while ((*info).x8EE as i32)
        < (*info).x8EA as i32 + (*info).decompressedY as i32
    {
        __THPDecompressiMCURow640x480(info);
        (*info).x8EE =
            ((*info).x8EE as i32 + (*info).x8D4 as i32) as u16;
    }
}
pub unsafe extern "C" fn THPDec_803313D0(
    mut info: *mut THPFileInfo,
    mut tileY: *mut ::core::ffi::c_void,
    mut tileU: *mut ::core::ffi::c_void,
    mut tileV: *mut ::core::ffi::c_void,
    mut x: u32,
) {
    let mut width: u32 = x;
    (*info).tileY = tileY as *mut u8;
    (*info).tileU = tileU as *mut u8;
    (*info).tileV = tileV as *mut u8;
    __THPPrepBitStream(info);
    while ((*info).x8EE as i32)
        < (*info).x8EA as i32 + (*info).decompressedY as i32
    {
        __THPDecompressiMCURowNxN(info, width);
        (*info).x8EE =
            ((*info).x8EE as i32 + (*info).x8D4 as i32) as u16;
    }
}
#[inline]
pub unsafe extern "C" fn __THPInverseDCTNoYPos(mut in_0: *mut THPCoeff, mut xPos: u32) {
    __THPInverseDCTPC(in_0, xPos, 0 as u32);
}
#[inline]
pub unsafe extern "C" fn __THPInverseDCTY8(mut in_0: *mut THPCoeff, mut xPos: u32) {
    __THPInverseDCTPC(in_0, xPos, Gwid.value.wrapping_mul(8 as u32));
}
#[inline]
pub unsafe extern "C" fn __THPHuffDecodeTab(
    mut info: *mut THPFileInfo,
    mut h: *mut THPHuffmanTab,
) -> s32 {
    return __THPHuffDecodePC(info, h);
}
pub unsafe extern "C" fn __THPDecompressiMCURow640x480(mut info: *mut THPFileInfo) {
    let mut cl_num: u8 = 0;
    let mut x_pos: u32 = 0;
    let mut comp: *mut THPComponent = ::core::ptr::null_mut::<THPComponent>();
    LCQueueWait(3 as u32);
    cl_num = 0 as u8;
    while (cl_num as i32) < (*info).MCUsPerRow as i32 {
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[0 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[1 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[2 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[3 as usize]);
        __THPHuffDecodeDCTCompU(info, (*info).mcuBuffer[4 as usize]);
        __THPHuffDecodeDCTCompV(info, (*info).mcuBuffer[5 as usize]);
        Gbase.value = __THPLCWork672[0 as usize] as *mut THPSample;
        Gwid.value = 640 as u32;
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab).offset(
            (*(&raw mut (*info).components as *mut THPComponent)
                .offset(0 as isize))
            .quantizationTableSelector as isize,
        ) as *mut f32;
        x_pos = (cl_num as i32 * 16i32) as u32;
        __THPInverseDCTNoYPos((*info).mcuBuffer[0 as usize], x_pos);
        __THPInverseDCTNoYPos(
            (*info).mcuBuffer[1 as usize],
            x_pos.wrapping_add(8 as u32),
        );
        __THPInverseDCTY8((*info).mcuBuffer[2 as usize], x_pos);
        __THPInverseDCTY8(
            (*info).mcuBuffer[3 as usize],
            x_pos.wrapping_add(8 as u32),
        );
        comp = (&raw mut (*info).components as *mut THPComponent)
            .offset(1 as isize) as *mut THPComponent;
        Gbase.value = __THPLCWork672[1 as usize] as *mut THPSample;
        Gwid.value = 320 as u32;
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab)
            .offset((*comp).quantizationTableSelector as isize) as *mut f32;
        x_pos = x_pos.wrapping_div(2 as u32);
        __THPInverseDCTNoYPos((*info).mcuBuffer[4 as usize], x_pos);
        comp = (&raw mut (*info).components as *mut THPComponent)
            .offset(2 as isize) as *mut THPComponent;
        Gbase.value = __THPLCWork672[2 as usize] as *mut THPSample;
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab)
            .offset((*comp).quantizationTableSelector as isize) as *mut f32;
        __THPInverseDCTNoYPos((*info).mcuBuffer[5 as usize], x_pos);
        if (*info).RST as i32 != 0i32 {
            (*info).currMCU = (*info).currMCU.wrapping_sub(1);
            if (*info).currMCU as i32 == 0i32 {
                (*info).currMCU = (*info).nMCU;
                (*info).cnt = (1 as u32).wrapping_add(
                    ((*info).cnt as u32).wrapping_add(6 as u32)
                        & 0xfffffff8 as u32,
                ) as u32;
                if (*info).cnt > 32 as u32 {
                    (*info).cnt = 33 as u32;
                }
                (*info).components[0 as usize].predDC = 0 as THPCoeff;
                (*info).components[1 as usize].predDC = 0 as THPCoeff;
                (*info).components[2 as usize].predDC = 0 as THPCoeff;
            }
        }
        cl_num = cl_num.wrapping_add(1);
    }
    LCStoreData(
        (*info).tileY as *mut ::core::ffi::c_void,
        __THPLCWork672[0 as usize] as *mut ::core::ffi::c_void,
        0x2800 as u32,
    );
    LCStoreData(
        (*info).tileU as *mut ::core::ffi::c_void,
        __THPLCWork672[1 as usize] as *mut ::core::ffi::c_void,
        0xa00 as u32,
    );
    LCStoreData(
        (*info).tileV as *mut ::core::ffi::c_void,
        __THPLCWork672[2 as usize] as *mut ::core::ffi::c_void,
        0xa00 as u32,
    );
    (*info).tileY = (*info).tileY.offset(0x2800 as isize);
    (*info).tileU = (*info).tileU.offset(0xa00 as isize);
    (*info).tileV = (*info).tileV.offset(0xa00 as isize);
}
pub unsafe extern "C" fn __THPDecompressiMCURowNxN(mut info: *mut THPFileInfo, mut x: u32) {
    let mut cl_num: u8 = 0;
    let mut x_pos: u32 = 0;
    let mut comp: *mut THPComponent = ::core::ptr::null_mut::<THPComponent>();
    LCQueueWait(3 as u32);
    cl_num = 0 as u8;
    while (cl_num as i32) < (*info).MCUsPerRow as i32 {
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[0 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[1 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[2 as usize]);
        __THPHuffDecodeDCTCompY(info, (*info).mcuBuffer[3 as usize]);
        __THPHuffDecodeDCTCompU(info, (*info).mcuBuffer[4 as usize]);
        __THPHuffDecodeDCTCompV(info, (*info).mcuBuffer[5 as usize]);
        comp = (&raw mut (*info).components as *mut THPComponent)
            .offset(0 as isize) as *mut THPComponent;
        Gbase.value = __THPLCWork672[0 as usize] as *mut THPSample;
        Gwid.value = x;
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab)
            .offset((*comp).quantizationTableSelector as isize) as *mut f32;
        x_pos = (cl_num as i32 * 16i32) as u32;
        __THPInverseDCTNoYPos((*info).mcuBuffer[0 as usize], x_pos);
        __THPInverseDCTNoYPos(
            (*info).mcuBuffer[1 as usize],
            x_pos.wrapping_add(8 as u32),
        );
        __THPInverseDCTY8((*info).mcuBuffer[2 as usize], x_pos);
        __THPInverseDCTY8(
            (*info).mcuBuffer[3 as usize],
            x_pos.wrapping_add(8 as u32),
        );
        comp = (&raw mut (*info).components as *mut THPComponent)
            .offset(1 as isize) as *mut THPComponent;
        Gbase.value = __THPLCWork672[1 as usize] as *mut THPSample;
        Gwid.value = x.wrapping_div(2 as u32);
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab)
            .offset((*comp).quantizationTableSelector as isize) as *mut f32;
        x_pos = x_pos.wrapping_div(2 as u32);
        __THPInverseDCTNoYPos((*info).mcuBuffer[4 as usize], x_pos);
        comp = (&raw mut (*info).components as *mut THPComponent)
            .offset(2 as isize) as *mut THPComponent;
        Gbase.value = __THPLCWork672[2 as usize] as *mut THPSample;
        Gq.value = &raw mut *(&raw mut (*info).quantTabs as *mut THPQuantTab)
            .offset((*comp).quantizationTableSelector as isize) as *mut f32;
        __THPInverseDCTNoYPos((*info).mcuBuffer[5 as usize], x_pos);
        if (*info).RST as i32 != 0i32 {
            (*info).currMCU = (*info).currMCU.wrapping_sub(1);
            if (*info).currMCU as i32 == 0i32 {
                (*info).currMCU = (*info).nMCU;
                (*info).cnt = (1 as u32).wrapping_add(
                    ((*info).cnt as u32).wrapping_add(6 as u32)
                        & 0xfffffff8 as u32,
                ) as u32;
                if (*info).cnt > 32 as u32 {
                    (*info).cnt = 33 as u32;
                }
                (*info).components[0 as usize].predDC = 0 as THPCoeff;
                (*info).components[1 as usize].predDC = 0 as THPCoeff;
                (*info).components[2 as usize].predDC = 0 as THPCoeff;
            }
        }
        cl_num = cl_num.wrapping_add(1);
    }
    LCStoreData(
        (*info).tileY as *mut ::core::ffi::c_void,
        __THPLCWork672[0 as usize] as *mut ::core::ffi::c_void,
        (4 as usize)
            .wrapping_mul(::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as u32,
    );
    LCStoreData(
        (*info).tileU as *mut ::core::ffi::c_void,
        __THPLCWork672[1 as usize] as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as u32,
    );
    LCStoreData(
        (*info).tileV as *mut ::core::ffi::c_void,
        __THPLCWork672[2 as usize] as *mut ::core::ffi::c_void,
        (::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as u32,
    );
    (*info).tileY = (*info).tileY.offset(
        (4 as usize)
            .wrapping_mul(::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as isize,
    );
    (*info).tileU = (*info).tileU.offset(
        (::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as isize,
    );
    (*info).tileV = (*info).tileV.offset(
        (::core::mem::size_of::<u8>() as usize)
            .wrapping_mul(64 as usize)
            .wrapping_mul(x.wrapping_div(16 as u32) as usize) as isize,
    );
}
pub unsafe extern "C" fn __THPHuffDecodeDCTCompY(mut info: *mut THPFileInfo, mut block: *mut THPCoeff) {
    __THPHuffDecodeBlockPC(
        info,
        block,
        Ydchuff.value,
        Yachuff.value,
        &raw mut (*info).components[0].predDC,
    );
}
pub unsafe extern "C" fn __THPHuffDecodeDCTCompU(mut info: *mut THPFileInfo, mut block: *mut THPCoeff) {
    __THPHuffDecodeBlockPC(
        info,
        block,
        Udchuff.value,
        Uachuff.value,
        &raw mut (*info).components[1].predDC,
    );
}
pub unsafe extern "C" fn __THPHuffDecodeDCTCompV(mut info: *mut THPFileInfo, mut block: *mut THPCoeff) {
    __THPHuffDecodeBlockPC(
        info,
        block,
        Vdchuff.value,
        Vachuff.value,
        &raw mut (*info).components[2].predDC,
    );
}
pub unsafe extern "C" fn THPInit() {
    static mut __THPLCWorkBuffer: [u8; 16128] = [0; 16128];
    __THPLCWork672[0 as usize] = (&raw mut __THPLCWorkBuffer as *mut u8)
        .offset(0 as isize)
        as *mut u8;
    __THPLCWork672[1 as usize] = (&raw mut __THPLCWorkBuffer as *mut u8)
        .offset(0x2a00 as isize)
        as *mut u8;
    __THPLCWork672[2 as usize] = (&raw mut __THPLCWorkBuffer as *mut u8)
        .offset((0x2a00i32 + 0xa80i32) as isize)
        as *mut u8;
}
pub static mut __THPLCWork672: [*mut u8; 3] = [::core::ptr::null::<u8>() as *mut u8; 3];
pub const NULL: i32 = 0i32;

/// The TARGET_PC body has no Gekko quantisation registers to initialise.
pub unsafe extern "C" fn OSInitFastCast() {}

#[cfg(test)]
#[path = "../../tests/thp_state/mod.rs"]
mod tests;
