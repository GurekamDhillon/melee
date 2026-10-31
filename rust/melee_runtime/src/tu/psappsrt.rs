//! Native-pointer port of `src/sysdolphin/baselib/psappsrt.c`: the particle "app SRT" records a generator and the
//! particles it spawns share, with a use count. The records live in the C's `HSD_ObjAlloc` pool
//! `HSD_PSAppSrt_804D10B0` (host memory, native `HSD_psAppSRT` layout); the counters `hsd_804D78DE` (live) and
//! `hsd_804D78D8` (peak) are particle.c globals (`crate::tu::particle`).
//!
//! The owned-model port (`crate::psappsrt`, arena + ids) stays as a test oracle only.
//!
//! GameCube vs native: `psInitAppSRT(0, 0xA4)` passes the GameCube record size (0xA4). The native record is larger
//! (8-byte pointers), so the pool is initialised with `sizeof(HSD_psAppSRT)` and that is the memset size too.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use crate::tu::particle::{hsd_804D78D8, hsd_804D78DE};
use melee_types::{HSD_Generator, HSD_ObjAllocData, HSD_Particle, HSD_psAppSRT};

/// `HSD_PSAppSrt_804D10B0`.
pub static mut HSD_PSAppSrt_804D10B0: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
/// `HSD_PSAppSrt_804D7958[4]` (only element 0 is used: the record size).
pub static mut HSD_PSAppSrt_804D7958: [u16; 4] = [0; 4];

pub unsafe extern "C" fn psInitAppSRT(_num: i32, _size: i32) -> bool {
    melee_core::ct!("psInitAppSRT");
    hsd_804D78DE = 0;
    hsd_804D78D8 = 0;
    let native = core::mem::size_of::<HSD_psAppSRT>();
    HSD_PSAppSrt_804D7958[0] = native as u16;
    crate::tu::objalloc::HSD_ObjAllocInit(&raw mut HSD_PSAppSrt_804D10B0, native, 4);
    false
}

pub unsafe extern "C" fn psAddGeneratorAppSRT(status: i32, idnum: u16) -> *mut HSD_psAppSRT {
    melee_core::ct!("psAddGeneratorAppSRT");
    let srt = crate::tu::objalloc::HSD_ObjAlloc(&raw mut HSD_PSAppSrt_804D10B0) as *mut HSD_psAppSRT;
    if !srt.is_null() {
        core::ptr::write_bytes(srt as *mut u8, 0, HSD_PSAppSrt_804D7958[0] as usize);
    }
    if !srt.is_null() {
        (*srt).next = core::ptr::null_mut();
        (*srt).usedCount = 1;
        (*srt).frameNum = 0;
        (*srt).status = status as u8;
        (*srt).translate.z = 0.0;
        (*srt).translate.y = 0.0;
        (*srt).translate.x = 0.0;
        (*srt).rot.z = 0.0;
        (*srt).rot.y = 0.0;
        (*srt).rot.x = 0.0;
        (*srt).scale.z = 1.0;
        (*srt).scale.y = 1.0;
        (*srt).scale.x = 1.0;
        (*srt).freefunc = None;
        (*srt).xA2 = 0;
        (*srt).gp = core::ptr::null_mut();
        (*srt).idnum = idnum;
        hsd_804D78DE = hsd_804D78DE.wrapping_add(1);
        if hsd_804D78DE > hsd_804D78D8 {
            hsd_804D78D8 = hsd_804D78DE;
        }
    }
    srt
}

pub unsafe extern "C" fn psAddParticleAppSRT_begin(pp: *mut HSD_Particle, status: i32) -> *mut HSD_psAppSRT {
    melee_core::ct!("psAddParticleAppSRT_begin");
    (*pp).appsrt = psAddGeneratorAppSRT(status, (*pp).idnum);
    (*pp).appsrt
}

pub unsafe extern "C" fn psAddGeneratorAppSRT_begin(gp: *mut HSD_Generator, status: i32) -> *mut HSD_psAppSRT {
    melee_core::ct!("psAddGeneratorAppSRT_begin");
    (*gp).appsrt = psAddGeneratorAppSRT(status, (*gp).idnum);
    (*gp).appsrt
}

pub unsafe extern "C" fn psAttachParticleAppSRT(pp: *mut HSD_Particle, srt: *mut HSD_psAppSRT) -> i32 {
    melee_core::ct!("psAttachParticleAppSRT");
    if srt.is_null() || pp.is_null() || !(*pp).appsrt.is_null() {
        return -1;
    }
    (*pp).appsrt = srt;
    (*srt).usedCount = (*srt).usedCount.wrapping_add(1);
    (*srt).usedCount as i32
}

pub unsafe extern "C" fn psAttachGeneratorAppSRT(gp: *mut HSD_Generator, srt: *mut HSD_psAppSRT) -> i32 {
    melee_core::ct!("psAttachGeneratorAppSRT");
    if srt.is_null() || gp.is_null() || !(*gp).appsrt.is_null() {
        return -1;
    }
    (*gp).appsrt = srt;
    (*srt).usedCount = (*srt).usedCount.wrapping_add(1);
    (*srt).usedCount as i32
}

pub unsafe extern "C" fn psRemoveParticleAppSRT(pp: *mut HSD_Particle) -> i32 {
    melee_core::ct!("psRemoveParticleAppSRT");
    let srt = (*pp).appsrt;
    if srt.is_null() {
        return -1;
    }
    (*srt).usedCount = (*srt).usedCount.wrapping_sub(1);
    let count = (*srt).usedCount as i32;
    if count == 0 {
        let srt = (*pp).appsrt;
        if let Some(f) = (*srt).freefunc {
            f(srt);
        }
        crate::tu::objalloc::HSD_ObjFree(&raw mut HSD_PSAppSrt_804D10B0, srt as *mut core::ffi::c_void);
        hsd_804D78DE = hsd_804D78DE.wrapping_sub(1);
    }
    (*pp).appsrt = core::ptr::null_mut();
    count
}

pub unsafe extern "C" fn psRemoveGeneratorSRT(gp: *mut HSD_Generator) -> i32 {
    melee_core::ct!("psRemoveGeneratorSRT");
    let srt = (*gp).appsrt;
    if srt.is_null() {
        return -1;
    }
    if (*srt).gp == gp {
        (*srt).gp = core::ptr::null_mut();
    }
    let srt = (*gp).appsrt;
    (*srt).usedCount = (*srt).usedCount.wrapping_sub(1);
    let count = (*srt).usedCount as i32;
    if count == 0 {
        let srt = (*gp).appsrt;
        if let Some(f) = (*srt).freefunc {
            f(srt);
        }
        crate::tu::objalloc::HSD_ObjFree(&raw mut HSD_PSAppSrt_804D10B0, srt as *mut core::ffi::c_void);
        hsd_804D78DE = hsd_804D78DE.wrapping_sub(1);
    }
    (*gp).appsrt = core::ptr::null_mut();
    count
}
