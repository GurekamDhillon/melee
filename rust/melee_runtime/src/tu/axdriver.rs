//! Port of src/sysdolphin/baselib/axdriver.c (the HSD_SM virtual-voice driver: SEM script
//! interpreter, voice pool, start/key-off/pan/volume/pitch/mix, aux effect setup, pause channels,
//! PStream control, SEM file loader) on `melee_types::HSD_SM` and the AXFX records.
//!
//! Native-profile decisions (the C is a 32-bit big-endian program):
//! - `AXDriver_8038DA70` takes the SEM file bytes as delivered by `ext::DVDReadAsyncPrio`, in
//!   native byte order (the importer converts); its relocation tables are 32-bit words, as in the
//!   C (`*(u32*) += base & ~3`). A sample's command-stream pointer is rebuilt from that 32-bit word
//!   with the buffer's upper address bits ([`reloc_ptr`]), exact while the buffer sits inside one
//!   4 GiB window.
//! - `sqrtf` is `crate::gekko::sqrtf` (math_ppc.h); `powf` (MSL) is a TODO stub in `ext`.
//! - The TARGET_PC Geno-Lab replay wrappers (`LabAudio_*`, `Snap_*`, `LabSfx_*`) are ported as the
//!   public names around the `*_raw` bodies, their externs being TODO stubs.
//! - `HSD_ASSERT` panics (the C build does not return from `__assert`).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, unused_variables, unused_mut, dead_code)]

use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

pub mod ext {
    #![allow(non_snake_case, unused_variables, dead_code)]
    use super::*;
    macro_rules! todo_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            $( pub unsafe extern "C" fn $name($($a: $t),*) $(-> $r)? {
                panic!(concat!("TODO: ", stringify!($name), " is not ported"))
            } )*
        };
    }
    todo_fns! {
        // dolphin/os, dvd, ax, axfx
        fn powf(a0: f32, a1: f32) -> f32; // MSL powf (math.h): not ported
        fn OSDisableInterrupts() -> bool;
        fn OSRestoreInterrupts(a0: bool) -> bool;
        fn DVDConvertPathToEntrynum(a0: *const i8) -> i32;
        fn DVDFastOpen(a0: i32, a1: *mut DVDFileInfo) -> i32;
        fn DVDReadAsyncPrio(a0: *mut DVDFileInfo, a1: *mut c_void, a2: i32, a3: i32, a4: Option<unsafe extern "C" fn(i32, *mut DVDFileInfo)>, a5: i32) -> i32;
        fn DVDClose(a0: *mut DVDFileInfo) -> i32;
        fn AXRegisterAuxACallback(a0: *mut c_void, a1: *mut c_void);
        fn AXRegisterAuxBCallback(a0: *mut c_void, a1: *mut c_void);
        fn AXFXSetHooks(a0: Option<unsafe extern "C" fn(usize) -> *mut c_void>, a1: Option<unsafe extern "C" fn(*mut c_void)>);
        fn AXFXReverbHiInit(a0: *mut AXFX_REVERBHI) -> i32;
        fn AXFXReverbHiShutdown(a0: *mut AXFX_REVERBHI) -> i32;
        fn AXFXReverbHiCallback(a0: *mut c_void, a1: *mut c_void);
        fn AXFXReverbStdInit(a0: *mut AXFX_REVERBSTD) -> i32;
        fn AXFXReverbStdShutdown(a0: *mut AXFX_REVERBSTD) -> i32;
        fn AXFXReverbStdCallback(a0: *mut c_void, a1: *mut c_void);
        fn AXFXChorusInit(a0: *mut AXFX_CHORUS) -> i32;
        fn AXFXChorusShutdown(a0: *mut AXFX_CHORUS) -> i32;
        fn AXFXChorusCallback(a0: *mut c_void, a1: *mut c_void);
        fn AXFXDelayInit(a0: *mut AXFX_DELAY) -> i32;
        fn AXFXDelayShutdown(a0: *mut AXFX_DELAY) -> i32;
        fn AXFXDelayCallback(a0: *mut c_void, a1: *mut c_void);
        // baselib synth
        fn HSD_AudioMalloc(a0: usize) -> *mut c_void;
        fn HSD_AudioFree(a0: *mut c_void);
        fn HSD_SynthInit(a0: i32, a1: i32, a2: i32, a3: i32);
        fn HSD_SynthSFXKeyOff(a0: i32);
        fn HSD_SynthSFXPause(a0: i32);
        fn HSD_SynthSFXResume(a0: i32);
        fn HSD_SynthSFXCheck(a0: i32) -> i32;
        fn HSD_SynthSFXSetPan(a0: i32, a1: u8);
        fn HSD_SynthSFXSetMix(a0: i32, a1: f32, a2: f32, a3: f32);
        fn HSD_SynthSFXSetPitchRatio(a0: i32, a1: i32, a2: f32);
        fn HSD_SynthSFXSetPriority(a0: i32, a1: i32);
        fn HSD_SynthSFXSetVolumeFade(a0: i32, a1: u8, a2: i32);
        fn HSD_SynthSFXSetDriverInactivatedCallback(a0: Option<unsafe extern "C" fn(i32)>);
        fn HSD_SynthSFXSetDriverMasterClockCallback(a0: Option<unsafe extern "C" fn()>);
        fn HSD_SynthSFXSetDriverPauseCallback(a0: Option<unsafe extern "C" fn(i32)>);
        fn HSD_SynthPStreamStart(a0: i32, a1: u8, a2: u8, a3: i32) -> i32;
        fn HSD_SynthSFXPlayWithGroup(a0: i32, a1: u8, a2: u8, a3: u8, a4: i32, a5: i32, a6: i32, a7: f32, a8: f32, a9: f32, a10: f32, a11: f32) -> i32;
        // TARGET_PC Geno-Lab / rollback hooks
        fn Snap_SuppressSfx() -> i32;
        fn Snap_SfxTake(a0: i32) -> i32;
        fn Snap_SfxPut(a0: i32, a1: i32);
        fn Snap_SfxAdd(a0: i32, a1: i32);
        fn LabSfx_Replay(a0: i32) -> i32;
        fn LabSfx_Handle() -> i32;
        fn LabSfx_Record(a0: i32, a1: i32);
        fn LabSfx_Map(a0: i32, a1: i32);
        fn LabAudio_Replay(a0: i32, a1: i32) -> i32;
        fn LabAudio_Value() -> i32;
        fn LabAudio_Real() -> i32;
        fn LabAudio_Record(a0: i32, a1: i32, a2: i32);
    }
    /// `OSReport` (variadic; only plain messages are used here).
    pub unsafe fn OSReport(msg: &str) {
        panic!("TODO: OSReport is not ported: {msg}")
    }
}

#[inline]
fn hsd_assert(cond: bool, line: u32) {
    if !cond {
        panic!("HSD_ASSERT axdriver.c:{line:#x}");
    }
}

#[inline]
fn clamp_i(min: i32, val: i32, max: i32) -> i32 {
    // MAX2(MIN2(val, max), min)
    let m = if val < max { val } else { max };
    if m < min { min } else { m }
}

const SM_MASK: u32 = SMSTATE_MASK;
const SM_ACTIVE: u32 = SMSTATE_ACTIVE as u32;
const SM_SLEEP: u32 = SMSTATE_SLEEP;

// ---- axdriver.static.h
pub static mut AXDriver_804C45A0: [HSD_SM; 0x60] = unsafe { core::mem::zeroed() };
pub static mut AXDriver_804C5920: [*mut HSD_SM; 0x40] = [null_mut(); 0x40];
pub static mut AXDriver_804C5A20: [[u8; 0x10]; 2] = [[0; 0x10]; 2];
pub static mut AXDriver_804C5A40: [AXFX_REVERBHI; 2] = unsafe { core::mem::zeroed() };
pub static mut AXDriver_804C5E00: [AXFX_REVERBSTD; 2] = unsafe { core::mem::zeroed() };
pub static mut AXDriver_804C60A8: [AXFX_CHORUS; 2] = unsafe { core::mem::zeroed() };
pub static mut AXDriver_804C61E0: [AXFX_DELAY; 2] = unsafe { core::mem::zeroed() };

pub static mut AXDriver_804D6038: i32 = -1;
pub static mut AXDriver_804D603C: u32 = 2;

static mut vidhigh: i32 = 0;
static mut AXDriver_804D778C: i32 = 0;
static mut AXDriver_804D7790: *mut HSD_SM = null_mut();
static mut AXDriver_804D7794: *mut HSD_SM = null_mut();
static mut AXDriver_804D7798: *mut c_void = null_mut();
static mut AXDriver_804D779C: u32 = 0;
static mut AXDriver_804D77A0: i32 = 0;
static mut AXDriver_804D77A4: *mut c_void = null_mut();
static mut AXDriver_804D77A8: i32 = 0;
static mut AXDriver_804D77AC: *mut u32 = null_mut();
static mut AXDriver_804D77B0: i32 = 0;
static mut AXDriver_804D77B4: *mut u32 = null_mut();
static mut AXDriver_804D77B8: i32 = 0;
/// `u32** AXDriver_804D77BC`: a table of 32-bit relocated words (see [`reloc_ptr`]).
static mut AXDriver_804D77BC: *mut u32 = null_mut();
static mut AXDriver_804D77C0: i32 = 0;
static mut AXDriver_804D77C4: *mut u32 = null_mut();
static mut AXDriver_804D77C8: i32 = 0;
static mut AXDriver_804D77CC: u32 = 0;
static mut AXDriver_804D77D0: i32 = 0;
static mut AXDriver_804D77D4: *mut u8 = null_mut();
static mut axfxallocsize: u32 = 0;
static mut axfxmaxsize: u32 = 0;
static mut AXDriver_804D77E0: i32 = 0;
static mut AXDriver_804D77E4: i32 = 0;
static mut AXDriver_804D77E8: i32 = 0;
static mut AXDriver_804D77EC: i32 = 0;

/// A 32-bit relocated table word back to a native pointer: the word holds the low 32 bits of
/// `buffer + offset`; the high bits are the buffer's.
#[inline]
unsafe fn reloc_ptr(word: u32) -> *mut u32 {
    let hi = (AXDriver_804D7798 as usize) & !0xFFFF_FFFFusize;
    (hi | word as usize) as *mut u32
}

#[inline]
unsafe fn rd_u32(p: *const u8) -> u32 {
    (p as *const u32).read_unaligned()
}

pub unsafe extern "C" fn AXDriverAlloc(size: usize) -> *mut c_void {
    let ptr = AXDriver_804D77D4.add(axfxallocsize as usize);
    axfxallocsize = axfxallocsize.wrapping_add(size as u32);
    hsd_assert(axfxallocsize < axfxmaxsize, 78);
    ptr as *mut c_void
}

pub unsafe extern "C" fn AXDriverFree(ptr: *mut c_void) {}

pub unsafe extern "C" fn AXDriverUnlink(v: *mut HSD_SM, head: *mut *mut HSD_SM) {
    if !v.is_null() {
        let p = (*v).prev;
        let n = (*v).next;
        (*v).next = null_mut();
        (*v).prev = null_mut();
        if !p.is_null() {
            (*p).next = n;
        }
        if !n.is_null() {
            (*n).prev = p;
        }
        if *head == v {
            *head = n;
        }
        hsd_assert(*head != v, 113);
    }
}

unsafe fn AXDriverLink(v: *mut HSD_SM, head: *mut *mut HSD_SM) {
    if v.is_null() {
        return;
    }
    hsd_assert(*head != v, 0x7A);

    (*v).prev = null_mut();
    if !(*head).is_null() {
        (**head).prev = v;
    }
    (*v).next = *head;
    *head = v;
}

unsafe fn AXDriverKeyOff(v: *mut HSD_SM) -> bool {
    let state = (*v).flags & SM_MASK;
    hsd_assert(state == SM_ACTIVE || state == SM_SLEEP, 0x92);

    let idx = (*v).vID;
    if (*v).vID != -1 {
        AXDriver_804C5920[(idx & 0x3F) as usize] = null_mut();
        (*v).vID = -1;
        ext::HSD_SynthSFXKeyOff(idx);
        AXDriver_804D77C8 -= 1;
    }

    (*v).flags &= !0x20000000;
    (*v).flags &= !SM_MASK;
    true
}

/// `HSD_AudioSFXKeyOff` before the Lab wrapper (`HSD_AudioSFXKeyOff_raw`).
pub unsafe extern "C" fn HSD_AudioSFXKeyOff_raw(vid: i32) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }

    let enabled = ext::OSDisableInterrupts();
    let v = &raw mut AXDriver_804C45A0[idx as usize];

    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        ext::OSRestoreInterrupts(enabled);
        return false;
    }

    let result = if v.is_null() { false } else { AXDriverKeyOff(v) };

    ext::OSRestoreInterrupts(enabled);
    result
}

pub unsafe extern "C" fn HSD_AudioSFXKeyOffAll() {
    let enabled = ext::OSDisableInterrupts();
    let mut v = AXDriver_804D7794;

    while !v.is_null() {
        if (*v).flags & SM_MASK != 0 {
            AXDriverKeyOff(v);
        }
        v = (*v).next;
    }
    ext::OSRestoreInterrupts(enabled);
}

pub unsafe extern "C" fn HSD_AudioSFXKeyOffTrack(track: i32) {
    let enabled = ext::OSDisableInterrupts();
    let mut v = AXDriver_804D7794;

    while !v.is_null() {
        if (*v).flags & SM_MASK != 0 && (*v).track as i32 == track {
            AXDriverKeyOff(v);
        }
        v = (*v).next;
    }
    ext::OSRestoreInterrupts(enabled);
}

/// `(v->x26 * v->x24[n]) / 65535.0F`
#[inline]
unsafe fn mix_vol(v: *mut HSD_SM, a: u8, b: u8) -> f32 {
    ((a as i32 * b as i32) as f32) / 65535.0
}

pub unsafe extern "C" fn AXDriverExec(v: *mut HSD_SM) {
    let mut i: i32 = 0;
    while i <= 9 {
        let flag: u32 = 1u32 << i;

        if (*v).flags & flag != 0 {
            match flag {
                0x1 => {
                    let left_vol = mix_vol(v, (*v).x26, (*v).x24[0]);
                    let right_vol = mix_vol(v, (*v).x27, (*v).x24[1]);
                    let left_sqrt = crate::gekko::sqrtf(left_vol);
                    let left_inv_sqrt = crate::gekko::sqrtf(1.0f32 - left_vol);
                    let right_sqrt = crate::gekko::sqrtf(right_vol);
                    let right_inv_sqrt = crate::gekko::sqrtf(1.0f32 - right_vol);
                    let tmp2 = left_inv_sqrt * right_inv_sqrt;
                    let pitch1 = ext::powf(2.0, (*v).x20 as f32 / 1200.0);
                    let pitch2 = ext::powf(2.0, (*v).fadetime as f32 / 1200.0);

                    (*v).vID = ext::HSD_SynthSFXPlayWithGroup(
                        (*v).fid as i32,
                        (*v).x1A,
                        (*v).volume,
                        if (*v).flags & 0x20000 != 0 { (*v).pan } else { (*v).x1C },
                        (*v).pri as i32,
                        (*v).itdflag as i32,
                        (*v).track as i32,
                        pitch1,
                        pitch2,
                        left_inv_sqrt * tmp2,
                        left_sqrt,
                        right_sqrt * left_inv_sqrt,
                    );

                    if (*v).vID != -1 {
                        AXDriver_804C5920[((*v).vID & 0x3F) as usize] = v;
                        (*v).flags &= 0xFFF4FF99;
                        if (*v).flags & 0x40000 != 0 {
                            (*v).flags &= !0x40000;
                        }
                        AXDriver_804D77C8 += 1;
                    } else {
                        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x13B);
                        (*v).flags &= !SM_MASK;
                        return;
                    }
                }
                0x2 => ext::HSD_SynthSFXSetPriority((*v).vID, (*v).pri as i32),
                0x4 => ext::HSD_SynthSFXSetVolumeFade((*v).vID, (*v).x1A, 0),
                0x8 => ext::HSD_SynthSFXSetPan((*v).vID, (*v).x1C),
                0x10 => {}
                0x20 => ext::HSD_SynthSFXSetPitchRatio((*v).vID, 0, ext::powf(2.0, (*v).x20 as f32 / 1200.0)),
                0x40 => ext::HSD_SynthSFXSetPitchRatio((*v).vID, 1, ext::powf(2.0, (*v).fadetime as f32 / 1200.0)),
                0x80 => {
                    let left_vol = mix_vol(v, (*v).x26, (*v).x24[0]);
                    let right_vol = mix_vol(v, (*v).x27, (*v).x24[1]);
                    let left_sqrt = crate::gekko::sqrtf(left_vol);
                    let left_inv_sqrt = crate::gekko::sqrtf(1.0f32 - left_vol);
                    let right_sqrt = crate::gekko::sqrtf(right_vol);
                    let right_inv_sqrt = crate::gekko::sqrtf(1.0f32 - right_vol);

                    ext::HSD_SynthSFXSetMix((*v).vID, left_inv_sqrt * (left_inv_sqrt * right_inv_sqrt), left_sqrt, right_sqrt * left_inv_sqrt);
                }
                0x100 => {
                    (*v).flags = ((*v).flags & !SM_MASK) | SM_SLEEP;
                    return;
                }
                0x200 => {
                    AXDriverKeyOff(v);
                    return;
                }
                _ => {}
            }
        }

        (*v).flags &= !flag;
        i += 1;
    }
}

pub unsafe extern "C" fn parseWait(param_type: u32, param_value: u32) -> u32 {
    match param_type {
        0 => param_value & 0xFFFFFF,
        1 | 2 | 3 | 4 | 5 | 14 | 15 | 20 | 21 => 0,
        6 | 7 | 8 | 9 | 10 | 11 | 16 | 17 | 18 | 19 => (param_value >> 8) & 0xFFFF,
        12 | 13 => (param_value >> 16) & 0xFF,
        _ => 0,
    }
}

pub unsafe extern "C" fn AXDriverInterp(v: *mut HSD_SM) {
    while (*v).x30 == AXDriver_804D778C {
        let cmd_word: u32 = *(*v).cmd_stream;
        let cmd_type: u32 = cmd_word >> 0x18;

        let cmd_size: i32 = parseWait(cmd_type, cmd_word) as i32;
        if cmd_size != 0 {
            AXDriverExec(v);
        }
        (*v).x30 = (*v).x30.wrapping_add(cmd_size);
        let w = *(*v).cmd_stream;
        match cmd_type {
            2 => {
                (*v).x2A = w as u16;
                if (*v).x2A == 0 {
                    (*v).flags |= 0x100000;
                }
            }
            3 => {
                if (*v).flags & 0x100000 != 0 || (*v).x2A != 0 {
                    (*v).cmd_stream = (*v).cmd_stream.offset(-((w & 0xFFFFFF) as isize));
                    (*v).x2A = (*v).x2A.wrapping_sub(1);
                }
            }
            1 => {
                (*v).flags |= 1;
                (*v).fid = w as u16;
            }
            4 => {
                (*v).flags |= 2;
                (*v).pri = w as u8;
            }
            5 => {
                (*v).flags |= 2;
                let cmd_val = (*v).pri as i32 + (w as u8 as i8) as i32;
                (*v).pri = clamp_i(5, cmd_val, 0x1C) as u8;
            }
            6 => {
                (*v).flags |= 4;
                (*v).x1A = w as u8;
            }
            7 => {
                (*v).flags |= 4;
                let cmd_val = (*v).x1A as i32 + (w as u8 as i8) as i32;
                (*v).x1A = clamp_i(0, cmd_val, 0xFF) as u8;
            }
            8 => {
                (*v).flags |= 8;
                (*v).x1C = w as u8;
            }
            9 => {
                (*v).flags |= 8;
                let cmd_val = (*v).x1C as i32 + (w as u8 as i8) as i32;
                (*v).x1C = clamp_i(0, cmd_val, 0xFF) as u8;
            }
            10 => {
                (*v).flags |= 0x10;
                (*v).x1E = w as u8;
            }
            11 => {
                (*v).flags |= 0x10;
                let cmd_val = (*v).x1E as i32 + (w as u8 as i8) as i32;
                (*v).x1E = clamp_i(0, cmd_val, 0xFF) as u8;
            }
            12 => {
                (*v).flags |= 0x20;
                (*v).x20 = (w as u16) as i16;
            }
            13 => {
                (*v).flags |= 0x20;
                let cmd_val = (*v).x20 as i32 + ((w as u16) as i16) as i32;
                (*v).x20 = clamp_i(-0x2A30, cmd_val, 0x960) as i16;
            }
            16 => {
                if AXDriver_804D603C & 1 == 0 {
                    (*v).flags |= 0x80;
                    (*v).x24[0] = w as u8;
                }
            }
            20 => {
                if AXDriver_804D603C & 1 == 0 {
                    (*v).x26 = w as u8;
                }
            }
            21 => {
                if (AXDriver_804D603C >> 1) & 1 == 0 {
                    (*v).x27 = w as u8;
                }
            }
            17 => {
                if AXDriver_804D603C & 1 == 0 {
                    (*v).flags |= 0x80;
                    let cmd_val = (*v).x24[0] as i32 + (w as u8 as i8) as i32;
                    (*v).x24[0] = clamp_i(0, cmd_val, 0xFF) as u8;
                }
            }
            18 => {
                if (AXDriver_804D603C >> 1) & 1 == 0 {
                    (*v).flags |= 0x80;
                    (*v).x24[1] = w as u8;
                }
            }
            19 => {
                if (AXDriver_804D603C >> 1) & 1 == 0 {
                    (*v).flags |= 0x80;
                    let cmd_val = (*v).x24[1] as i32 + (w as u8 as i8) as i32;
                    (*v).x24[1] = clamp_i(0, cmd_val, 0xFF) as u8;
                }
            }
            15 => {
                (*v).flags |= 0x200;
                AXDriverExec(v);
                return;
            }
            14 => {
                (*v).flags |= 0x100;
                AXDriverExec(v);
                return;
            }
            _ => {}
        }
        (*v).cmd_stream = (*v).cmd_stream.add(1);
    }
}

unsafe extern "C" fn AXDriverCallback() {
    if AXDriver_804D77E0 != 0 {
        AXDriver_804D77E0 = 0;
        if AXDriver_804D6038 != -1 {
            let x = ext::powf(2.0, clamp_i(-0x2A30, AXDriver_804D77E4, 0x960) as f32 / 1200.0);
            ext::HSD_SynthSFXSetPitchRatio(AXDriver_804D6038, 1, x);
        }
    }
    let mut v = AXDriver_804D7794;
    AXDriver_804D778C = AXDriver_804D778C.wrapping_add(1);
    while !v.is_null() {
        let next = (*v).next;
        if (*v).x30 == -1 {
            (*v).x30 = AXDriver_804D778C;
        } else if (*v).flags & 0x20000000 != 0 {
            (*v).x30 = (*v).x30.wrapping_add(1);
        }
        match (*v).flags & SM_MASK {
            SM_ACTIVE => {
                if (*v).x30 == AXDriver_804D778C {
                    AXDriverInterp(v);
                }
            }
            0 => {
                AXDriverUnlink(v, &raw mut AXDriver_804D7794);
                AXDriverLink(v, &raw mut AXDriver_804D7790);
                AXDriver_804D77D0 -= 1;
            }
            SM_SLEEP => {
                if (*v).flags & 0x40 != 0 {
                    let x = ext::powf(2.0, (*v).fadetime as f32 / 1200.0);
                    ext::HSD_SynthSFXSetPitchRatio((*v).vID, 1, x);
                    (*v).flags &= 0xFFFFFFBF;
                }
            }
            _ => hsd_assert(false, 0x25F),
        }
        v = next;
    }
}

unsafe extern "C" fn AXDriverKillCallback(vID: i32) {
    let idx = (vID & 0x3F) as usize;

    hsd_assert(vID > 0, 0x26D);

    let v = AXDriver_804C5920[idx];
    if v.is_null() || (*v).vID != vID {
        return;
    }

    AXDriver_804C5920[idx] = null_mut();
    (*v).flags &= !0x20000000;
    (*v).flags &= !SM_MASK;
    AXDriver_804D77C8 -= 1;
}

unsafe extern "C" fn AXDriverPauseCallback(vID: i32) {
    let idx = (vID & 0x3F) as usize;

    hsd_assert(vID > 0, 0x288);

    let v = AXDriver_804C5920[idx];
    hsd_assert(!v.is_null() && (*v).vID == vID, 0x28A);

    (*v).flags |= 0x20000000;
}

unsafe fn AXDriverAssignVVoice() -> *mut HSD_SM {
    if AXDriver_804D7790.is_null() {
        null_mut()
    } else {
        let enabled = ext::OSDisableInterrupts();
        let v = AXDriver_804D7790;
        AXDriverUnlink(v, &raw mut AXDriver_804D7790);
        ext::OSRestoreInterrupts(enabled);
        (*v).vID = -1;
        (*v).x30 = -1;
        (*v).flags = 0;
        (*v).flags &= !SM_MASK;
        v
    }
}

unsafe fn HSD_AudioSFXStartParam_impl(sound_id: i32, volume: u8, pan: u8, track: i32, channel: i32) -> i32 {
    let bank_idx = sound_id / 10000;
    let bank_mem = sound_id % 10000;

    if AXDriver_804D77B0 <= bank_idx {
        return -1;
    }

    // `bank_mem + AXDriver_804D77B4[bank_idx]` is unsigned in C
    let sample_idx: u32 = (bank_mem as u32).wrapping_add(*AXDriver_804D77B4.offset(bank_idx as isize));

    if (AXDriver_804D77B8 as u32) <= sample_idx {
        return -1;
    }

    if bank_idx < AXDriver_804D77B0 - 1 && *AXDriver_804D77B4.offset(bank_idx as isize + 1) <= sample_idx {
        return -1;
    }

    if track < 0 || track > 0xFF {
        return -1;
    }

    if channel < 0 || channel >= 0x10 {
        return -1;
    }

    if AXDriver_804D77CC & (1u32 << channel) != 0 {
        return -1;
    }

    let v = AXDriverAssignVVoice();

    if v.is_null() {
        return -1;
    }

    (*v).x16 = sound_id as u16;
    (*v).cmd_stream = reloc_ptr(*AXDriver_804D77BC.add(sample_idx as usize));
    (*v).x1A = 0xFF;
    (*v).volume = volume;
    (*v).x1C = 0x80;
    (*v).pan = pan;
    (*v).x20 = 0;
    (*v).fadetime = 0;
    (*v).x24[0] = AXDriver_804C5A20[0][channel as usize];
    (*v).x24[1] = AXDriver_804C5A20[1][channel as usize];
    (*v).x26 = 0xFF;
    (*v).x27 = 0xFF;
    (*v).pri = 5;
    (*v).track = track as u8;
    (*v).itdflag = channel as u8;
    (*v).flags |= 0x30000;

    if (vidhigh as u32) & 0xFE000000 != 0 {
        ext::OSReport("vidhigh exceeds the max value\n");
        hsd_assert(false, 0x2EA);
    }

    let slot = v.offset_from(&raw mut AXDriver_804C45A0 as *mut HSD_SM) as i32;
    (*v).unk = (vidhigh << 7) | slot;
    vidhigh = vidhigh.wrapping_add(1);

    let enabled = ext::OSDisableInterrupts();
    (*v).flags = ((*v).flags & !SM_MASK) | SM_ACTIVE;
    AXDriverLink(v, &raw mut AXDriver_804D7794);
    AXDriver_804D77D0 += 1;
    ext::OSRestoreInterrupts(enabled);

    (*v).unk
}

/// `HSD_AudioSFXStartParam` (TARGET_PC): rollback / Geno-Lab handle replay around the real start.
pub unsafe extern "C" fn HSD_AudioSFXStartParam(sound_id: i32, volume: u8, pan: u8, track: i32, channel: i32) -> i32 {
    let lab_m = ext::LabSfx_Replay(sound_id);
    let lab_h = ext::LabSfx_Handle();
    if lab_m == 1 {
        return lab_h;
    }
    if lab_m == 2 {
        ext::LabSfx_Map(lab_h, HSD_AudioSFXStartParam_impl(sound_id, volume, pan, track, channel));
        return lab_h;
    }
    if lab_m == 3 {
        return HSD_AudioSFXStartParam_impl(sound_id, volume, pan, track, channel);
    }
    if ext::Snap_SuppressSfx() != 0 {
        let r = ext::Snap_SfxTake(sound_id);
        if r != -2 {
            return r;
        }
        let r = HSD_AudioSFXStartParam_impl(sound_id, volume, pan, track, channel);
        ext::Snap_SfxAdd(sound_id, r);
        return r;
    }
    let r = HSD_AudioSFXStartParam_impl(sound_id, volume, pan, track, channel);
    ext::Snap_SfxPut(sound_id, r);
    ext::LabSfx_Record(sound_id, r);
    r
}

pub unsafe extern "C" fn HSD_AudioSFXSetPan_raw(vid: i32, pan: u8) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    if (*v).vID != -1 {
        ext::HSD_SynthSFXSetPan((*v).vID, pan);
    } else {
        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x30B);
        (*v).pan = pan;
        (*v).flags |= 0x20000;
    }
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioSFXSetVolumeEx_raw(vid: i32, volume: u8) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    let voice_id = (*v).vID;
    if voice_id != -1 {
        ext::HSD_SynthSFXSetVolumeFade(voice_id, clamp_i(0, volume as i32, 0xFF) as u8, 1);
    } else {
        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x34D);
        (*v).volume = volume;
        (*v).flags |= 0x10000;
    }
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioSFXSetPitchFid_raw(vid: i32, pitch: i16) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let clamped = clamp_i(-0x2A30, pitch as i32, 0x960);
    let enabled = ext::OSDisableInterrupts();
    (*v).fadetime = clamped as i16;
    (*v).flags |= 0x40;
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioSFXSetMix_raw(vid: i32, aux_bus: i32, send_level: u8) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    if aux_bus < 0 || aux_bus > 1 {
        return false;
    }
    let lock_flag = if aux_bus == 0 { AXDriver_804D603C & 1 } else { (AXDriver_804D603C >> 1) & 1 };
    if lock_flag != 1 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let clamped = clamp_i(0, send_level as i32, 0xFF);
    let enabled = ext::OSDisableInterrupts();
    if (*v).vID != -1 {
        (*v).x24[aux_bus as usize] = clamped as u8;
        let left_vol = mix_vol(v, (*v).x26, (*v).x24[0]);
        let right_vol = mix_vol(v, (*v).x27, (*v).x24[1]);
        let left_sqrt = crate::gekko::sqrtf(left_vol);
        let left_inv_sqrt = crate::gekko::sqrtf(1.0f32 - left_vol);
        let right_sqrt = crate::gekko::sqrtf(right_vol);
        let right_inv_sqrt = crate::gekko::sqrtf(1.0f32 - right_vol);
        ext::HSD_SynthSFXSetMix((*v).vID, left_inv_sqrt * (left_inv_sqrt * right_inv_sqrt), left_sqrt, right_sqrt * left_inv_sqrt);
    } else {
        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x3AB);
        (*v).x24[aux_bus as usize] = clamped as u8;
        (*v).flags |= 0x80000;
    }
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioSFXSetMixGroup(channel: i32, aux_bus: i32, send_level: i8) -> bool {
    if channel < 0 || channel >= 0x10 {
        return false;
    }
    if aux_bus < 0 || aux_bus > 1 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    let mut v = AXDriver_804D7794;
    while !v.is_null() {
        if (*v).flags & SM_MASK != 0 && (*v).itdflag as i32 == channel {
            HSD_AudioSFXSetMix((*v).unk, aux_bus, send_level as u8);
        }
        v = (*v).next;
    }
    AXDriver_804C5A20[aux_bus as usize][channel as usize] = send_level as u8;
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioSFXCheck_raw(vid: i32) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    if ext::HSD_SynthSFXCheck((*v).vID) == -1 {
        return false;
    }
    true
}

unsafe extern "C" fn fn_8038DA5C(result: i32, file_info: *mut DVDFileInfo) {
    if result != -1 {
        AXDriver_804D77EC = 1;
    }
}

pub unsafe extern "C" fn AXDriver_8038DA70(path: *const i8, callback: unsafe extern "C" fn()) {
    let mut file_info: DVDFileInfo = core::mem::zeroed();

    let entrynum = ext::DVDConvertPathToEntrynum(path);
    if entrynum == -1 || ext::DVDFastOpen(entrynum, &mut file_info) == 0 {
        ext::OSReport(&format!("can not open {}\n", core::ffi::CStr::from_ptr(path).to_string_lossy()));
        return;
    }

    AXDriver_804D779C = file_info.length;
    if AXDriver_804D779C == 0 {
        ext::OSReport(&format!("file size of \"{}\" is 0\n", core::ffi::CStr::from_ptr(path).to_string_lossy()));
        return;
    }

    let aligned_size = ((AXDriver_804D779C as i32).wrapping_add(0x1F)) & !0x1F;
    AXDriver_804D7798 = ext::HSD_AudioMalloc(aligned_size as usize);
    AXDriver_804D77EC = 0;
    ext::DVDReadAsyncPrio(&mut file_info, AXDriver_804D7798, aligned_size, 0, Some(fn_8038DA5C), 2);

    while AXDriver_804D77EC == 0 {
        callback();
    }

    ext::DVDClose(&mut file_info);

    let base = AXDriver_804D7798 as *mut u8;
    // `(u32) AXDriver_804D7798 & ~3u`: the low 32 bits of the buffer address
    let base32 = (base as usize as u32) & !3u32;

    AXDriver_804D77A0 = rd_u32(base) as i32;
    let mut count = AXDriver_804D77A0;
    let mut ptr: *mut c_void = if count != 0 { base.add(4) as *mut c_void } else { null_mut() };
    let mut offset: isize = count as isize * 4 + 4;
    AXDriver_804D77A4 = ptr;

    AXDriver_804D77A8 = rd_u32(base.offset(offset)) as i32;
    offset += 4;
    AXDriver_804D77AC = if AXDriver_804D77A8 != 0 { base.offset(offset) as *mut u32 } else { null_mut() };

    let mut i: i32 = 0;
    let mut j: isize = 0;
    while i < AXDriver_804D77A8 {
        i += 1;
        let p = (AXDriver_804D77AC as *mut u8).offset(j) as *mut u32;
        p.write_unaligned(p.read_unaligned().wrapping_add(base32));
        j += 4;
    }

    offset += AXDriver_804D77A8 as isize * 4;
    AXDriver_804D77B0 = rd_u32(base.offset(offset)) as i32;
    offset += 4;
    count = AXDriver_804D77B0;
    AXDriver_804D77B4 = if count != 0 { base.offset(offset) as *mut u32 } else { null_mut() };
    offset += count as isize * 4;

    AXDriver_804D77B8 = rd_u32(base.offset(offset)) as i32;
    offset += 4;
    AXDriver_804D77BC = if AXDriver_804D77B8 != 0 { base.offset(offset) as *mut u32 } else { null_mut() };
    j = 0;
    i = 0;
    while j < AXDriver_804D77B8 as isize {
        j += 1;
        let p = (AXDriver_804D77BC as *mut u8).offset(i as isize) as *mut u32;
        p.write_unaligned(p.read_unaligned().wrapping_add(base32));
        i += 4;
    }

    offset += AXDriver_804D77B8 as isize * 4;
    AXDriver_804D77C0 = rd_u32(base.offset(offset)) as i32;
    offset += 4;
    AXDriver_804D77C4 = if AXDriver_804D77C0 != 0 { base.offset(offset) as *mut u32 } else { null_mut() };
    j = 0;
    i = 0;
    while j < AXDriver_804D77C0 as isize {
        j += 1;
        let p = (AXDriver_804D77C4 as *mut u8).offset(i as isize) as *mut u32;
        p.write_unaligned(p.read_unaligned().wrapping_add(base32));
        i += 4;
    }
}

pub unsafe extern "C" fn AXDriver_8038DCFC() {
    if !AXDriver_804D7798.is_null() {
        ext::HSD_AudioFree(AXDriver_804D7798);
    }
    AXDriver_804D7798 = null_mut();
    // TARGET_PC: the tables point into the buffer just freed
    AXDriver_804D77B0 = 0;
    AXDriver_804D77B4 = null_mut();
    AXDriver_804D77B8 = 0;
    AXDriver_804D77BC = null_mut();
}

pub unsafe extern "C" fn AXDriverSetupAux(channel: i32, type_: AXDriverAuxType, param: *mut c_void) -> i32 {
    let aux_data_hi = &raw mut AXDriver_804C5A40[channel as usize];
    let aux_data_std = &raw mut AXDriver_804C5E00[channel as usize];
    let aux_data_chorus = &raw mut AXDriver_804C60A8[channel as usize];
    let aux_data_delay = &raw mut AXDriver_804C61E0[channel as usize];

    let mut result: i32 = 0;
    let mut callback: *mut c_void = null_mut();
    let mut callback_data: *mut c_void = null_mut();

    hsd_assert(channel == 0 || channel == 1, 0x49C);
    hsd_assert(type_ <= 4, 0x49D);
    hsd_assert(type_ == 0 || !param.is_null(), 0x49E);

    let old_type: i32;
    if channel == 0 {
        old_type = ((AXDriver_804D603C >> 8) & 0xF) as i32;
        AXDriver_804D603C &= !((old_type << 8) as u32);
        AXDriver_804D603C |= (type_ << 8) as u32;
        ext::AXRegisterAuxACallback(null_mut(), null_mut());
    } else {
        old_type = ((AXDriver_804D603C >> 12) & 0xF) as i32;
        AXDriver_804D603C &= !((old_type << 12) as u32);
        AXDriver_804D603C |= (type_ << 12) as u32;
        ext::AXRegisterAuxBCallback(null_mut(), null_mut());
    }

    match old_type as AXDriverAuxType {
        AXDRIVER_AUX_REVERB_HI => {
            ext::AXFXReverbHiShutdown(aux_data_hi);
        }
        AXDRIVER_AUX_REVERB_STD => {
            ext::AXFXReverbStdShutdown(aux_data_std);
        }
        AXDRIVER_AUX_CHORUS => {
            ext::AXFXChorusShutdown(aux_data_chorus);
        }
        AXDRIVER_AUX_DELAY => {
            ext::AXFXDelayShutdown(aux_data_delay);
        }
        _ => {}
    }

    match type_ {
        AXDRIVER_AUX_OFF => result = 1,
        AXDRIVER_AUX_REVERB_HI => {
            core::ptr::copy_nonoverlapping(param as *const AXFX_REVERBHI, aux_data_hi, 1);
            if ext::AXFXReverbHiInit(aux_data_hi) == 1 {
                callback = ext::AXFXReverbHiCallback as usize as *mut c_void;
                callback_data = aux_data_hi as *mut c_void;
                result = 1;
            }
        }
        AXDRIVER_AUX_REVERB_STD => {
            core::ptr::copy_nonoverlapping(param as *const AXFX_REVERBSTD, aux_data_std, 1);
            if ext::AXFXReverbStdInit(aux_data_std) == 1 {
                callback = ext::AXFXReverbStdCallback as usize as *mut c_void;
                callback_data = aux_data_std as *mut c_void;
                result = 1;
            }
        }
        AXDRIVER_AUX_CHORUS => {
            core::ptr::copy_nonoverlapping(param as *const AXFX_CHORUS, aux_data_chorus, 1);
            if ext::AXFXChorusInit(aux_data_chorus) == 1 {
                callback = ext::AXFXChorusCallback as usize as *mut c_void;
                callback_data = aux_data_chorus as *mut c_void;
                result = 1;
            }
        }
        AXDRIVER_AUX_DELAY => {
            core::ptr::copy_nonoverlapping(param as *const AXFX_DELAY, aux_data_delay, 1);
            if ext::AXFXDelayInit(aux_data_delay) == 1 {
                callback = ext::AXFXDelayCallback as usize as *mut c_void;
                callback_data = aux_data_delay as *mut c_void;
                result = 1;
            }
        }
        _ => {}
    }

    if channel == 0 {
        ext::AXRegisterAuxACallback(callback, callback_data);
    } else {
        ext::AXRegisterAuxBCallback(callback, callback_data);
    }

    result
}

pub unsafe extern "C" fn HSD_AudioGetAuxHeapSize(type_: AXDriverAuxType, param: *mut c_void) -> i32 {
    let mut result: i32 = 0;

    if type_ > 4 || (type_ != 0 && param.is_null()) {
        return 0;
    }

    match type_ {
        AXDRIVER_AUX_REVERB_HI => {
            let dims: [i32; 8] = [0x6FD, 0x7CF, 0x91D, 0x1B1, 0x95, 0x2F, 0x49, 0x43];
            for k in 0..3usize {
                for i in 0..3usize {
                    result += (dims[i] + 2) * 4;
                    result += (dims[i + 3] + 2) * 4;
                }
                result += (dims[k + 5] + 2) * 4;
                result += ((32000.0f32 * (*(param as *mut AXFX_REVERBHI)).preDelay) as i32) * 4;
            }
        }
        AXDRIVER_AUX_REVERB_STD => {
            let dims: [i32; 4] = [0x6FD, 0x7CF, 0x1B1, 0x95];
            for _k in 0..3 {
                for i in 0..2usize {
                    result += (dims[i] + 2) * 4;
                    result += (dims[i + 2] + 2) * 4;
                }
                result += ((32000.0f32 * (*(param as *mut AXFX_REVERBSTD)).preDelay) as i32) * 4;
            }
        }
        AXDRIVER_AUX_CHORUS => result = 0x1680,
        AXDRIVER_AUX_DELAY => {
            let delay = param as *mut AXFX_DELAY;
            for i in 0..3usize {
                result += (((*delay).delay[i] as i32 - 5) * 32 + 159) / 160 * 640;
            }
        }
        _ => {}
    }

    result
}

pub unsafe extern "C" fn HSD_AudioSFXSetupAux(channel: i32, type_: i32, param: *mut c_void, heap: *mut u8, heap_size: usize) -> bool {
    if channel < 0 || channel > 1 {
        return false;
    }
    if type_ < 0 || type_ > 4 || (type_ != AXDRIVER_AUX_OFF as i32 && param.is_null()) {
        return false;
    }
    AXDriver_804D77D4 = heap;
    axfxallocsize = 0;
    axfxmaxsize = heap_size as u32;
    AXDriverSetupAux(channel, type_ as AXDriverAuxType, param) != 0
}

pub unsafe extern "C" fn HSD_AudioSFXGetDefaultAuxParam(type_: AXDriverAuxType, param: *mut c_void) -> bool {
    if type_ > AXDRIVER_AUX_DELAY || (type_ != AXDRIVER_AUX_OFF && param.is_null()) {
        return false;
    }
    match type_ {
        AXDRIVER_AUX_REVERB_HI => {
            let p = param as *mut AXFX_REVERBHI;
            (*p).tempDisableFX = 0;
            (*p).time = 3.0;
            (*p).preDelay = 0.1;
            (*p).damping = 0.5;
            (*p).coloration = 0.5;
            (*p).crosstalk = 0.3;
            (*p).mix = 0.5;
        }
        AXDRIVER_AUX_REVERB_STD => {
            let p = param as *mut AXFX_REVERBSTD;
            (*p).tempDisableFX = 0;
            (*p).time = 1.48;
            (*p).preDelay = 0.002;
            (*p).damping = 0.64;
            (*p).coloration = 0.5;
            (*p).mix = 1.0;
        }
        AXDRIVER_AUX_CHORUS => {
            let p = param as *mut AXFX_CHORUS;
            (*p).baseDelay = 0xF;
            (*p).variation = 0;
            (*p).period = 0x1F4;
        }
        AXDRIVER_AUX_DELAY => {
            let p = param as *mut AXFX_DELAY;
            (*p).delay[0] = 0x104;
            (*p).delay[1] = 0x136;
            (*p).delay[2] = 6;
            (*p).feedback[0] = 0x18;
            (*p).feedback[1] = 0x18;
            (*p).feedback[2] = 0;
            (*p).output[0] = 0x23;
            (*p).output[1] = 0x23;
            (*p).output[2] = 0;
        }
        _ => {}
    }
    true
}

pub unsafe extern "C" fn HSD_AudioInitMultiPStream(voices: i32, priority: i32, sample_rate: i32, aram_size: i32) {
    let mut i: usize = 0;
    while i < 0x60 {
        AXDriver_804C45A0[i].flags &= !SM_MASK;
        AXDriverLink(&raw mut AXDriver_804C45A0[i], &raw mut AXDriver_804D7790);
        i += 1;
    }

    ext::HSD_SynthInit(voices, priority, sample_rate, aram_size);
    ext::HSD_SynthSFXSetDriverMasterClockCallback(Some(AXDriverCallback));
    ext::HSD_SynthSFXSetDriverInactivatedCallback(Some(AXDriverKillCallback));
    ext::HSD_SynthSFXSetDriverPauseCallback(Some(AXDriverPauseCallback));

    axfxallocsize = 0;
    AXDriver_804D77D4 = null_mut();
    axfxmaxsize = 0;
    AXDriverSetupAux(0, AXDRIVER_AUX_OFF, null_mut());

    axfxallocsize = 0;
    AXDriver_804D77D4 = null_mut();
    axfxmaxsize = 0;
    AXDriverSetupAux(1, AXDRIVER_AUX_OFF, null_mut());
    ext::AXFXSetHooks(Some(AXDriverAlloc), Some(AXDriverFree));
}

pub unsafe extern "C" fn AXDriver_8038E5D4_raw() -> i32 {
    AXDriver_804D77C8
}

pub unsafe extern "C" fn AXDriver_8038E5DC_raw() -> i32 {
    AXDriver_804D77D0
}

unsafe fn PStreamPauseCh(vid: i32) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    if (*v).vID != -1 {
        ext::HSD_SynthSFXPause((*v).vID);
    } else {
        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x5D6);
        (*v).flags |= 0x20000000;
    }
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioPStreamPauseCh(channel: i32) -> bool {
    if channel < 0 || channel >= 0x10 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    let mut v = AXDriver_804D7794;
    while !v.is_null() {
        if (*v).flags & SM_MASK != 0 && (*v).itdflag as i32 == channel {
            PStreamPauseCh((*v).unk);
        }
        v = (*v).next;
    }
    AXDriver_804D77CC |= 1u32 << channel;
    ext::OSRestoreInterrupts(enabled);
    true
}

unsafe fn PStreamResumeCh(vid: i32) -> bool {
    let idx = vid & 0x7F;
    if vid < 0 || idx >= 0x60 {
        return false;
    }
    let v = &raw mut AXDriver_804C45A0[idx as usize];
    if (*v).unk != vid || (*v).flags & SM_MASK == 0 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    if (*v).vID != -1 {
        ext::HSD_SynthSFXResume((*v).vID);
    } else {
        hsd_assert((*v).flags & SM_MASK == SM_ACTIVE, 0x619);
    }
    (*v).flags &= 0xDFFFFFFF;
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioPStreamResumeCh(channel: i32) -> bool {
    if channel < 0 || channel >= 0x10 {
        return false;
    }
    let enabled = ext::OSDisableInterrupts();
    let mut v = AXDriver_804D7794;
    while !v.is_null() {
        if (*v).flags & SM_MASK != 0 && (*v).itdflag as i32 == channel {
            PStreamResumeCh((*v).unk);
        }
        v = (*v).next;
    }
    AXDriver_804D77CC &= !(1u32 << channel);
    ext::OSRestoreInterrupts(enabled);
    true
}

pub unsafe extern "C" fn HSD_AudioPStreamStartChParam(path: *const i8, volume: u8, track: i32) -> bool {
    let entrynum = ext::DVDConvertPathToEntrynum(path);
    if AXDriver_804D6038 != -1 {
        ext::HSD_SynthSFXKeyOff(AXDriver_804D6038);
    }
    AXDriver_804D6038 = ext::HSD_SynthPStreamStart(entrynum, 0xFF, volume, track);
    AXDriver_804D77E8 = AXDriver_804D778C;
    true
}

pub unsafe extern "C" fn AXDriverStop() -> bool {
    if AXDriver_804D6038 == -1 {
        return false;
    }
    ext::HSD_SynthSFXKeyOff(AXDriver_804D6038);
    AXDriver_804D6038 = -1;
    true
}

pub unsafe extern "C" fn AXDriverPause() -> bool {
    if AXDriver_804D6038 == -1 {
        return false;
    }
    ext::HSD_SynthSFXPause(AXDriver_804D6038);
    true
}

pub unsafe extern "C" fn AXDriverResume() -> bool {
    if AXDriver_804D6038 == -1 {
        return false;
    }
    ext::HSD_SynthSFXResume(AXDriver_804D6038);
    true
}

pub unsafe extern "C" fn AXDriverCheck_raw() -> bool {
    if ext::HSD_SynthSFXCheck(AXDriver_804D6038) == -1 {
        return false;
    }
    true
}

// ---- TARGET_PC Geno-Lab wrappers (LAB_AUDIO): kind 0 = live (call, then record), 1 = replayed
// (logged answer), 2 = replayed at speed (logged answer, side effect on the real voice).
unsafe fn lab_audio(kind: i32, arg: i32, call_real: impl FnOnce(i32) -> i32) -> i32 {
    let lab_m = ext::LabAudio_Replay(kind, arg);
    let mut lab_v = ext::LabAudio_Value();
    let lab_real = ext::LabAudio_Real();
    if lab_m == 0 {
        lab_v = call_real(arg);
        ext::LabAudio_Record(kind, arg, lab_v);
    } else if lab_m == 2 {
        call_real(lab_real);
    }
    lab_v
}

pub unsafe extern "C" fn HSD_AudioSFXKeyOff(vid: i32) -> bool {
    let lab_m = ext::LabAudio_Replay(1, vid);
    let mut lab_v = ext::LabAudio_Value();
    let lab_real = ext::LabAudio_Real();
    if lab_m == 0 {
        lab_v = HSD_AudioSFXKeyOff_raw(lab_real) as i32;
        ext::LabAudio_Record(1, vid, lab_v);
    } else if lab_m == 2 {
        HSD_AudioSFXKeyOff_raw(lab_real);
    }
    lab_v != 0
}

macro_rules! lab_wrapper {
    ($kind:expr, $name:ident ( $vid:ident $(, $p:ident : $t:ty)* ) = $raw:ident) => {
        pub unsafe extern "C" fn $name($vid: i32 $(, $p: $t)*) -> bool {
            let lab_m = ext::LabAudio_Replay($kind, $vid);
            let mut lab_v = ext::LabAudio_Value();
            let lab_real = ext::LabAudio_Real();
            if lab_m == 0 {
                lab_v = $raw(lab_real $(, $p)*) as i32;
                ext::LabAudio_Record($kind, $vid, lab_v);
            } else if lab_m == 2 {
                $raw(lab_real $(, $p)*);
            }
            lab_v != 0
        }
    };
}
lab_wrapper!(2, HSD_AudioSFXSetPan(vid, pan: u8) = HSD_AudioSFXSetPan_raw);
lab_wrapper!(3, HSD_AudioSFXSetVolumeEx(vid, volume: u8) = HSD_AudioSFXSetVolumeEx_raw);
lab_wrapper!(4, HSD_AudioSFXSetPitchFid(vid, pitch: i16) = HSD_AudioSFXSetPitchFid_raw);
lab_wrapper!(5, HSD_AudioSFXSetMix(vid, aux_bus: i32, send_level: u8) = HSD_AudioSFXSetMix_raw);
lab_wrapper!(6, HSD_AudioSFXCheck(vid) = HSD_AudioSFXCheck_raw);

pub unsafe extern "C" fn AXDriverCheck() -> bool {
    let lab_m = ext::LabAudio_Replay(7, 0);
    let mut lab_v = ext::LabAudio_Value();
    let lab_real = ext::LabAudio_Real();
    if lab_m == 0 {
        lab_v = AXDriverCheck_raw() as i32;
        ext::LabAudio_Record(7, 0, lab_v);
    } else if lab_m == 2 {
        AXDriverCheck_raw();
    }
    lab_v != 0
}

pub unsafe extern "C" fn AXDriver_8038E5D4() -> i32 {
    lab_audio(8, 0, |_| AXDriver_8038E5D4_raw())
}

pub unsafe extern "C" fn AXDriver_8038E5DC() -> i32 {
    lab_audio(9, 0, |_| AXDriver_8038E5DC_raw())
}
