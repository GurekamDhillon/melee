//! Port of src/melee/lb/lbaudio_ax.c (+ lbaudio_ax.static.h, lbaudio_stage_slots.inc): the SFX /
//! BGM front end over the baselib synth: bank bookkeeping, volume mixing, positional sound
//! objects and the per-frame update. The file has no fused float sites
//! (`tools/rust-port/fma_sites.py src/melee/lb/lbaudio_ax.c`). The TARGET_PC build's m-ex tables
//! (160 banks, 256 stages, 256 BGM files) are kept as in the C.
//!
//! Callees of other TUs are `ext` hooks named like the C functions.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_variables, unused_mut, dead_code)]

use crate::cabi;
use core::ffi::c_void;
use core::ptr::{null, null_mut};
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn OSReport(msg: &str);
    fn lb_800195D0();
    fn lbArchive_LoadSymbols(file: *const i8, out: *mut *mut LbAudioAxHeapData, sym: *const i8, end: i32);
    fn lbLang_IsSettingUS() -> i32 = crate::tu::lblanguage::lbLang_IsSettingUS() as _;
    fn lbLang_IsSavedLanguageUS() -> i32 = crate::tu::lblanguage::lbLang_IsSavedLanguageUS() as _;
    fn AXDriverStop() = { let _ = crate::tu::axdriver::AXDriverStop(); };
    fn AXDriverCheck() -> i32 = crate::tu::axdriver::AXDriverCheck() as _;
    fn AXDriverPause() = { let _ = crate::tu::axdriver::AXDriverPause(); };
    fn AXDriverResume() = { let _ = crate::tu::axdriver::AXDriverResume(); };
    fn AXDriver_8038DCFC() = crate::tu::axdriver::AXDriver_8038DCFC();
    fn AXDriver_8038DA70(path: *const i8, cb: Option<unsafe extern "C" fn()>);
    fn AXDriver_8038E5D4() -> i32 = crate::tu::axdriver::AXDriver_8038E5D4();
    fn AXDriver_8038E5DC() -> i32 = crate::tu::axdriver::AXDriver_8038E5DC();
    fn ARInit(stack: *mut u32, len: u32);
    fn ARQInit();
    fn AIInit(a0: *mut c_void);
    fn HSD_AudioSFXKeyOffAll() = crate::tu::axdriver::HSD_AudioSFXKeyOffAll();
    fn HSD_AudioSFXKeyOff(voice: i32) = { let _ = crate::tu::axdriver::HSD_AudioSFXKeyOff(voice); };
    fn HSD_AudioSFXKeyOffTrack(track: i32) = crate::tu::axdriver::HSD_AudioSFXKeyOffTrack(track);
    fn HSD_AudioSFXCheck(voice: i32) -> i32;
    fn HSD_AudioSFXStartParam(id: i32, vol: i32, pan: i32, track: i32, channel: i32) -> i32 = crate::tu::axdriver::HSD_AudioSFXStartParam(id, vol as _, pan as _, track, channel);
    fn HSD_AudioSFXSetPan(voice: i32, pan: i32);
    fn HSD_AudioSFXSetVolumeEx(voice: i32, vol: i32);
    fn HSD_AudioSFXSetPitchFid(voice: i32, pitch: i32) -> i32;
    fn HSD_AudioSFXSetMixGroup(a0: i32, a1: i32, a2: u8) = { let _ = crate::tu::axdriver::HSD_AudioSFXSetMixGroup(a0, a1, a2 as _); };
    fn HSD_AudioSFXGetDefaultAuxParam(kind: u32, out: *mut c_void);
    fn HSD_AudioGetAuxHeapSize(kind: i32, param: *mut c_void) -> u32;
    fn HSD_AudioSFXSetupAux(slot: i32, kind: i32, param: *mut c_void, heap: *mut u8, size: u32) = { let _ = crate::tu::axdriver::HSD_AudioSFXSetupAux(slot, kind, param, heap, size as _); };
    fn HSD_AudioInitMultiPStream(voices: i32, a1: i32, a2: i32, size: i32) = crate::tu::axdriver::HSD_AudioInitMultiPStream(voices, a1, a2, size);
    fn HSD_AudioPStreamStartChParam(path: *const i8, vol: i32, ch: i32) -> i32 = crate::tu::axdriver::HSD_AudioPStreamStartChParam(path, vol as _, ch) as _;
    fn HSD_AudioPStreamPauseCh(ch: i32) = { let _ = crate::tu::axdriver::HSD_AudioPStreamPauseCh(ch); };
    fn HSD_AudioPStreamResumeCh(ch: i32) = { let _ = crate::tu::axdriver::HSD_AudioPStreamResumeCh(ch); };
    fn HSD_SynthStreamSetVolume(v: f32) = crate::tu::synth::HSD_SynthStreamSetVolume(v);
    fn HSD_SynthSFXUpdateAllVolume(vol: u8, a1: i32, a2: i32) = crate::tu::synth::HSD_SynthSFXUpdateAllVolume(vol as _, a1 as _, a2);
    fn HSD_SynthGetSoundMode() -> u32 = crate::tu::synth::HSD_SynthGetSoundMode();
    fn HSD_SynthSetSoundMode(mode: u32) = crate::tu::synth::HSD_SynthSetSoundMode(mode as _);
    fn HSD_SynthSFXUnloadBank(bank: i32) = crate::tu::synth::HSD_SynthSFXUnloadBank(bank);
    fn HSD_SynthSFXLoad(path: *const i8, bank: i32, cb: Option<unsafe extern "C" fn(i32, i32)>, arg: i32) -> i32 = crate::tu::synth::HSD_SynthSFXLoad(path, bank, cb, arg);
    fn HSD_SynthSFXWaitForLoadCompletion(cb: Option<unsafe extern "C" fn()>) = crate::tu::synth::HSD_SynthSFXWaitForLoadCompletion(cb);
    fn HSD_SynthSFXGroupDataRemove(group: i32) = crate::tu::synth::HSD_SynthSFXGroupDataRemove(group);
    fn HSD_SynthSFXBankDeflag(bank: i32) = crate::tu::synth::HSD_SynthSFXBankDeflag(bank);
    fn HSD_SynthSFXBankDeflagSync() = crate::tu::synth::HSD_SynthSFXBankDeflagSync();
    fn HSD_SynthSFXGetPendingLoadCount() -> i32 = crate::tu::synth::HSD_SynthSFXGetPendingLoadCount();
    fn HSD_SynthSFXCancelLoad(group: i32) -> i32 = crate::tu::synth::HSD_SynthSFXCancelLoad(group);
    fn HSD_SynthSFXAllocateBank(size: i32) = crate::tu::synth::HSD_SynthSFXAllocateBank(size);
    fn HSD_ObjAllocInit(data: *mut HSD_ObjAllocData, size: u32, align: u32) = crate::tu::objalloc::HSD_ObjAllocInit(data, size as _, align);
    fn HSD_ObjAlloc(data: *mut HSD_ObjAllocData) -> *mut c_void = crate::tu::objalloc::HSD_ObjAlloc(data);
    fn HSD_ObjFree(data: *mut HSD_ObjAllocData, obj: *mut c_void) = crate::tu::objalloc::HSD_ObjFree(data, obj);
    fn HSD_Randi(n: i32) -> i32 = crate::tu::random::HSD_Randi(crate::tu::globals::rand(), n);
    fn HSD_GObjFree(gobj: *mut HSD_GObj);
    fn GObj_Create(class: u16, plink: u8, prio: u8) -> *mut HSD_GObj;
    fn GObj_InitUserData(gobj: *mut HSD_GObj, kind: u8, free: Option<unsafe extern "C" fn(*mut c_void)>, data: *mut c_void);
    fn HSD_GObj_SetupProc(gobj: *mut HSD_GObj, cb: Option<unsafe extern "C" fn(*mut HSD_GObj)>, prio: u8) -> *mut HSD_GObjProc;
    fn ftLib_80086644(gobj: *mut HSD_GObj, pos: *mut Vec3);
    fn it_8026B294(gobj: *mut HSD_GObj, pos: *mut Vec3);
    fn Camera_800307D0(left: *mut f32, center: *mut f32, right: *mut f32);
    fn Stage_80225194() -> u32;
    fn Stage_8022519C(stkind: u32) -> i32;
    fn gm_801603B0();
    fn gm_8016B184() -> i32;
    fn gm_80169370(i: i32) -> i32;
    fn Player_GetPlayerCharacter(slot: i32) -> i32;
    fn Player_GetPlayerSlotType(slot: i32) -> u32;
    // TARGET_PC m-ex / Geno glue
    fn Mex_FighterBgmForPortCKind(ck: i32, which: i32) -> i32;
    fn Geno_DefineBaseCK(ck: i32) -> i32;
    fn Mex_SsmForPortCKind(ck: i32) -> i32;
    fn gm_MexVanillaKind(kind: i32, character_kind: i32) -> i32;
    fn Mex_SsmCount() -> i32;
    fn Mex_SsmFile(i: i32) -> *const i8;
    fn Mex_SsmSize(i: i32) -> u32;
    fn Mex_SsmLookup(i: i32, k: i32) -> i32;
    fn Mex_BgmCount() -> i32;
    fn Mex_BgmFile(i: i32) -> *const i8;
    fn Mex_GrAudioCount() -> i32;
    fn Mex_GrAudioByte(i: i32, k: i32) -> i32;
    fn Mex_GrAudioFirst() -> i32;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::gobj_native::HSD_GObjPLinkHead;
}

unsafe extern "C" fn lb_800195D0_cb() {
    ext::lb_800195D0();
}

const VOL_MAX: i32 = 0x7F;
const PAN_MID: i32 = 0x40;
const LBAX_CAP: usize = 160;
const LBAX_STAGE_CAP: usize = 256;
const LBAX_BGM_CAP: usize = 256;
const PLINK_AUDIO_AX: u8 = (HSD_GOBJ_PLINK_MAX - 1) as u8;
const AUDIO_MODE_UNK0: u32 = 0;
const AUDIO_MODE_UNK1: u32 = 1;
const LANG_JP: i32 = 0;
const LANG_US: i32 = 1;

macro_rules! LBAX_N {
    () => {
        lbAx_N
    };
}

// ---- static.h state ---------------------------------------------------------------------------

static mut lbl_804D63F0: f32 = 0.0;
static mut lbl_804D63F4: i32 = 0;
static mut lbl_804D63F8: i32 = 0;
static mut lbl_804D63FC: i32 = 0;
static mut lbl_804D6400: i32 = 0;
static mut lbl_804D6404: i32 = 0;

static mut lbl_804D3870: i32 = 0x700000;
static mut sound_mode: i32 = 1;
static mut lbl_804D3878: i32 = -1;
static mut lbl_804D387C: i32 = 0x7F;
static mut synth_volume: f32 = 1.0;
static mut lbl_804D3884: i32 = 0x7F;
static mut lbl_804D3888: i32 = 0x7F;
static mut lbl_804D388C: i32 = 0x7F;
static mut lbl_804D3890: i32 = 0x7F;
static mut lbl_804D3894: i32 = 0x7F;
static mut lbl_804D3898: i32 = 0x7F;
static mut lbl_804D389C: i32 = 0x7F;
static mut lbl_804D38A0: i32 = 0x7F;
static mut lbl_804D38A4: i32 = 0x7F;
static mut lbl_804D38A8: i32 = 0x7F;
static mut lbl_804D38AC: f32 = 1.0;
static mut lbl_804D38B0: f32 = 1.0;
static mut lbl_804D38B4: f32 = 1.0;
static mut lbl_804D38B8: f32 = 1.0;
static mut lbl_804D38BC: f32 = 1.0;
static mut lbl_804D38C0: f32 = 1.0;
static mut lbl_804D38C4: f32 = 1.0;
static mut lbl_804D38C8: f32 = 1.0;
static mut lbl_804D38CC: i32 = 0x7F;
static mut ssm_stem_pos: usize = 7;
static mut hps_stem_pos: usize = 7;
static mut lbl_804D38D8: i32 = 1;
static mut lbl_804D38DC: i32 = -1;
static mut lbl_804D38E0: i32 = 1;
static mut lbl_804D38E4: f32 = 1.0;
static mut lbl_804D38E8: f32 = 1.0;
static mut lbl_804D38EC: f32 = 1.0;
static mut lbl_804D38F0: i32 = -1;
static mut lbl_804D38F4: i32 = -1;

static mut debug_enabled: bool = false;
static mut lbl_804D640C: i32 = 0;
static mut paused: bool = false;
static mut lbl_804D6414: i32 = 0;
static mut lbl_804D6418: i32 = 0;
static mut lbl_804D641C: i32 = 0;
static mut lbl_804D6420: i32 = 0;
static mut lbl_804D6424: i32 = 0;
static mut lbl_804D6428: i32 = 0;
static mut lbl_804D642C: i32 = 0;
static mut lbl_804D6430: i32 = 0;
static mut lbl_804D6434: i32 = 0;

static mut lbl_80433710: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
static mut lbl_8043373C: [i32; 17] = [0; 17];
static mut lbl_80433780: [i32; 17] = [0; 17];

static mut lbAx_N: i32 = 55;
static mut lbAx_BgmN: i32 = 0x62;

static mut lbl_804337C4: [i32; LBAX_CAP] = [0; LBAX_CAP];
static mut lbl_804338A4: [i32; LBAX_CAP] = [0; LBAX_CAP];
static mut lbl_80433984: [i32; LBAX_CAP] = [0; LBAX_CAP];
static mut lbl_80433A64: [i32; LBAX_CAP] = [0; LBAX_CAP];
static mut lbl_80433B44: [i32; LBAX_CAP] = [0; LBAX_CAP];

const fn cstr_buf(s: &[u8]) -> [u8; 0x40] {
    let mut b = [0u8; 0x40];
    let mut i = 0;
    while i < s.len() {
        b[i] = s[i];
        i += 1;
    }
    b
}
static mut cur_hps_stem: [u8; 0x40] = cstr_buf(b"");
static mut cur_ssm_file: [u8; 0x40] = cstr_buf(b"/audio/");
static mut cur_hps_file: [u8; 0x40] = cstr_buf(b"/audio/");
static str_audio_us: &[u8] = b"/audio/us/\0";
static str_audio: &[u8] = b"/audio/\0";

static mut lbl_804D6438: i32 = 0;
static mut lbl_804D643C: i32 = 0;
static mut lbl_804D6440: i32 = 0;
static mut lbl_804D6444: i32 = 0;
static mut lbl_804D6448: i32 = 0;
static mut lbl_804D644C: i32 = 0;
static mut lbl_804D6450: u32 = 0;

static mut lbl_804D6454: *mut LbAudioAxHeapData = null_mut();
static mut lbAx_stage_pending: [u8; LBAX_CAP] = [0; LBAX_CAP];

// ---- C string helpers over the fixed buffers ---------------------------------------------------

/// `strcpy(&buf[pos], src)` with a NUL-terminated `src`.
unsafe fn strcpy_at(buf: *mut [u8; 0x40], pos: usize, src: *const i8) {
    let mut i = 0usize;
    loop {
        let c = *src.add(i) as u8;
        (*buf)[pos + i] = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}
unsafe fn strcpy_bytes(buf: *mut [u8; 0x40], src: &[u8]) {
    for (i, &c) in src.iter().enumerate() {
        (*buf)[i] = c;
    }
}
unsafe fn cstr_eq(a: *const u8, b: *const i8) -> bool {
    let mut i = 0usize;
    loop {
        let x = *a.add(i);
        let y = *b.add(i) as u8;
        if x != y {
            return false;
        }
        if x == 0 {
            return true;
        }
        i += 1;
    }
}

#[inline]
fn LBAX_MEX_ID(id: i32) -> bool {
    unsafe { lbAx_N > 55 && id >= 550000 && id < lbAx_N * 10000 }
}

// ---- tables (lbaudio_ax.static.h) --------------------------------------------------------------

pub static lbl_803BB3C0: [LbAudioAxBankMask; 33] = [
    LbAudioAxBankMask { x0: 0x06, x8: 0x000000040 },
    LbAudioAxBankMask { x0: 0x08, x8: 0x000000100 },
    LbAudioAxBankMask { x0: 0x0B, x8: 0x000000800 },
    LbAudioAxBankMask { x0: 0x1D, x8: 0x020000000 },
    LbAudioAxBankMask { x0: 0x0E, x8: 0x000004000 },
    LbAudioAxBankMask { x0: 0x0F, x8: 0x000008000 },
    LbAudioAxBankMask { x0: 0x10, x8: 0x000010000 },
    LbAudioAxBankMask { x0: 0x11, x8: 0x000020000 },
    LbAudioAxBankMask { x0: 0x12, x8: 0x000040000 },
    LbAudioAxBankMask { x0: 0x13, x8: 0x000080000 },
    LbAudioAxBankMask { x0: 0x14, x8: 0x000100000 },
    LbAudioAxBankMask { x0: 0x15, x8: 0x000200000 },
    LbAudioAxBankMask { x0: 0x16, x8: 0x000400000 },
    LbAudioAxBankMask { x0: 0x18, x8: 0x001000000 },
    LbAudioAxBankMask { x0: 0x0D, x8: 0x000002000 },
    LbAudioAxBankMask { x0: 0x19, x8: 0x002000000 },
    LbAudioAxBankMask { x0: 0x1A, x8: 0x004000000 },
    LbAudioAxBankMask { x0: 0x1C, x8: 0x010000000 },
    LbAudioAxBankMask { x0: 0x1B, x8: 0x008000000 },
    LbAudioAxBankMask { x0: 0x1B, x8: 0x008000000 },
    LbAudioAxBankMask { x0: 0x0A, x8: 0x000000400 },
    LbAudioAxBankMask { x0: 0x07, x8: 0x000000080 },
    LbAudioAxBankMask { x0: 0x09, x8: 0x000000200 },
    LbAudioAxBankMask { x0: 0x1F, x8: 0x080000000 },
    LbAudioAxBankMask { x0: 0x17, x8: 0x000800000 },
    LbAudioAxBankMask { x0: 0x1E, x8: 0x040000000 },
    LbAudioAxBankMask { x0: 0x20, x8: 0x100000000 },
    LbAudioAxBankMask { x0: 0x37, x8: 0x000000000 },
    LbAudioAxBankMask { x0: 0x37, x8: 0x000000000 },
    LbAudioAxBankMask { x0: 0x0C, x8: 0x000001000 },
    LbAudioAxBankMask { x0: 0x20, x8: 0x100000000 },
    LbAudioAxBankMask { x0: 0x37, x8: 0x000000000 },
    LbAudioAxBankMask { x0: 0x0D, x8: 0x000002000 },
];

static mut s32_arr_803BB5D0: [[i8; 4]; 160] = [
    [1, 5, 5, 0],
    [1, 5, 5, 0],
    [3, 0, 0, 0],
    [3, 0, 0, 0],
    [3, 4, 4, 0],
    [3, 4, 4, 0],
    [4, 2, 3, 4],
    [4, 2, 3, 7],
    [4, 2, 3, 7],
    [4, 2, 3, 6],
    [4, 2, 3, 6],
    [4, 2, 3, 4],
    [4, 2, 1, 0],
    [4, 2, 3, 9],
    [4, 2, 3, 1],
    [4, 2, 3, 8],
    [4, 2, 3, 7],
    [4, 2, 3, 2],
    [4, 2, 3, 1],
    [4, 2, 3, 7],
    [4, 2, 3, 1],
    [4, 2, 3, 6],
    [4, 2, 3, 1],
    [4, 2, 3, 1],
    [4, 2, 3, 7],
    [4, 2, 3, 1],
    [4, 2, 3, 6],
    [4, 2, 3, 2],
    [4, 2, 3, 1],
    [4, 2, 3, 6],
    [4, 2, 3, 7],
    [4, 2, 3, 6],
    [4, 2, 1, 0],
    [4, 2, 1, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 2, 0],
    [5, 2, 4, 0],
    [6, 1, 1, 0],
    [2, 5, 5, 0],
    [6, 1, 1, 0],
    [5, 2, 2, 0],
    [1, 5, 5, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

static mut s32_arr_803BB6B0: [[u8; 3]; 256] = [
    [55, 1, 1],
    [55, 1, 1],
    [34, 1, 1],
    [55, 1, 1],
    [37, 1, 1],
    [41, 1, 1],
    [36, 1, 1],
    [55, 24, 24],
    [40, 64, 64],
    [42, 128, 128],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [43, 1, 1],
    [35, 1, 1],
    [44, 1, 136],
    [48, 1, 1],
    [55, 1, 1],
    [38, 1, 64],
    [45, 1, 1],
    [39, 1, 1],
    [46, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [47, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [49, 1, 1],
    [49, 72, 72],
    [49, 136, 136],
    [49, 1, 1],
    [55, 1, 1],
    [55, 56, 56],
    [53, 56, 56],
    [55, 1, 1],
    [55, 1, 1],
    [49, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [55, 1, 1],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
];

static flags_arr_803BB800: [i8; 212] = [
    1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1,
    0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1,
    1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1,
    1, 1, 0, 0, 2, 16, 0, 65, 0, 6, 0, 23, 0, 11, 0, 7,
    0, 85, 0, 112, 0, 67, 0, 102, 0, 125, 0, -127, 0, 69, 0, -7,
    0, -102, 0, 80, 0, 109, 0, 104, 0, 94, 0, -123, 0, -108, 0, 102,
    0, 91, 0, 97, 0, 103, 0, 79, 0, 75, 0, -56, 0, 100, 0, 72,
    0, 91, 0, -124, 0, 30, 0, -34, 0, 8, 0, 26, 0, 11, 0, 2,
    0, 11, 0, 14, 0, 12, 0, 5, 0, 9, 0, 10, 0, 25, 0, 7,
    0, 5, 0, 9, 0, 2, 0, 7, 0, 16, 0, 54, 0, 2, 0, 5,
    0, 1, 0, 0,
];

static mut s32_arr_803BB8D4: [[i32; 2]; 160] = [
    [10000, 10064],
    [20000, 20005],
    [30000, 30022],
    [40000, 40010],
    [50000, 50006],
    [60000, 60084],
    [70000, 70111],
    [80000, 80066],
    [90000, 90101],
    [100000, 100124],
    [110000, 110128],
    [120000, 120068],
    [130000, 130248],
    [140000, 140153],
    [150000, 150079],
    [160000, 160108],
    [170000, 170103],
    [180000, 180093],
    [190000, 190132],
    [200000, 200147],
    [210000, 210101],
    [220000, 220090],
    [230000, 230096],
    [240000, 240102],
    [250000, 250078],
    [260000, 260074],
    [270000, 270199],
    [280000, 280099],
    [290000, 290071],
    [300000, 300090],
    [310000, 310131],
    [320000, 320029],
    [330000, 330221],
    [340000, 340007],
    [350000, 350025],
    [360000, 360010],
    [370000, 370001],
    [380000, 380010],
    [390000, 390013],
    [400000, 400011],
    [410000, 410004],
    [420000, 420008],
    [430000, 430009],
    [440000, 440024],
    [450000, 450006],
    [460000, 460004],
    [470000, 470008],
    [480000, 480001],
    [490000, 490006],
    [500000, 500015],
    [510000, 510053],
    [520000, 520001],
    [530000, 530004],
    [540000, 540000],
    [540000, 540000],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

static mut ssm_files: [*const i8; 160] = [
    b"main.ssm ".as_ptr() as *const i8,
    b"pokemon.ssm ".as_ptr() as *const i8,
    b"nr_title.ssm ".as_ptr() as *const i8,
    b"nr_select.ssm ".as_ptr() as *const i8,
    b"nr_1p.ssm ".as_ptr() as *const i8,
    b"nr_vs.ssm ".as_ptr() as *const i8,
    b"captain.ssm ".as_ptr() as *const i8,
    b"clink.ssm ".as_ptr() as *const i8,
    b"dk.ssm ".as_ptr() as *const i8,
    b"drmario.ssm ".as_ptr() as *const i8,
    b"falco.ssm ".as_ptr() as *const i8,
    b"fox.ssm ".as_ptr() as *const i8,
    b"gkoopa.ssm ".as_ptr() as *const i8,
    b"ice.ssm ".as_ptr() as *const i8,
    b"kirby.ssm ".as_ptr() as *const i8,
    b"koopa.ssm ".as_ptr() as *const i8,
    b"link.ssm ".as_ptr() as *const i8,
    b"luigi.ssm ".as_ptr() as *const i8,
    b"mario.ssm ".as_ptr() as *const i8,
    b"mars.ssm ".as_ptr() as *const i8,
    b"mewtwo.ssm ".as_ptr() as *const i8,
    b"ness.ssm ".as_ptr() as *const i8,
    b"peach.ssm ".as_ptr() as *const i8,
    b"pichu.ssm ".as_ptr() as *const i8,
    b"pikachu.ssm ".as_ptr() as *const i8,
    b"purin.ssm ".as_ptr() as *const i8,
    b"samus.ssm ".as_ptr() as *const i8,
    b"zs.ssm ".as_ptr() as *const i8,
    b"yoshi.ssm ".as_ptr() as *const i8,
    b"gw.ssm ".as_ptr() as *const i8,
    b"ganon.ssm ".as_ptr() as *const i8,
    b"emblem.ssm ".as_ptr() as *const i8,
    b"mhands.ssm ".as_ptr() as *const i8,
    b"kirbytm.ssm ".as_ptr() as *const i8,
    b"castle.ssm ".as_ptr() as *const i8,
    b"corneria.ssm ".as_ptr() as *const i8,
    b"greatbay.ssm ".as_ptr() as *const i8,
    b"kongo.ssm ".as_ptr() as *const i8,
    b"mutecity.ssm ".as_ptr() as *const i8,
    b"onett.ssm ".as_ptr() as *const i8,
    b"zebes.ssm ".as_ptr() as *const i8,
    b"garden.ssm ".as_ptr() as *const i8,
    b"klaid.ssm ".as_ptr() as *const i8,
    b"greens.ssm ".as_ptr() as *const i8,
    b"venom.ssm ".as_ptr() as *const i8,
    b"bigblue.ssm ".as_ptr() as *const i8,
    b"fourside.ssm ".as_ptr() as *const i8,
    b"pupupu.ssm ".as_ptr() as *const i8,
    b"pstadium.ssm ".as_ptr() as *const i8,
    b"1padv.ssm ".as_ptr() as *const i8,
    b"ending.ssm ".as_ptr() as *const i8,
    b"nr_name.ssm ".as_ptr() as *const i8,
    b"1pend.ssm ".as_ptr() as *const i8,
    b"last.ssm ".as_ptr() as *const i8,
    b"end.ssm ".as_ptr() as *const i8,
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
];

static mut hps_files: [*const i8; 256] = [
    b"1p_qk.hps ".as_ptr() as *const i8,
    b"akaneia.hps ".as_ptr() as *const i8,
    b"baloon.hps ".as_ptr() as *const i8,
    b"bigblue.hps ".as_ptr() as *const i8,
    b"castle.hps ".as_ptr() as *const i8,
    b"continue.hps ".as_ptr() as *const i8,
    b"corneria.hps ".as_ptr() as *const i8,
    b"docmari.hps ".as_ptr() as *const i8,
    b"ending.hps ".as_ptr() as *const i8,
    b"famidemo.hps ".as_ptr() as *const i8,
    b"ff_1p01.hps ".as_ptr() as *const i8,
    b"ff_1p02.hps ".as_ptr() as *const i8,
    b"ff_bad.hps ".as_ptr() as *const i8,
    b"ff_dk.hps ".as_ptr() as *const i8,
    b"ff_emb.hps ".as_ptr() as *const i8,
    b"ff_flat.hps ".as_ptr() as *const i8,
    b"ff_fox.hps ".as_ptr() as *const i8,
    b"ff_fzero.hps ".as_ptr() as *const i8,
    b"ff_good.hps ".as_ptr() as *const i8,
    b"ff_ice.hps ".as_ptr() as *const i8,
    b"ff_kirby.hps ".as_ptr() as *const i8,
    b"ff_link.hps ".as_ptr() as *const i8,
    b"ff_mario.hps ".as_ptr() as *const i8,
    b"ff_nes.hps ".as_ptr() as *const i8,
    b"ff_poke.hps ".as_ptr() as *const i8,
    b"ff_samus.hps ".as_ptr() as *const i8,
    b"ff_step1.hps ".as_ptr() as *const i8,
    b"ff_step2.hps ".as_ptr() as *const i8,
    b"ff_step3.hps ".as_ptr() as *const i8,
    b"ff_yoshi.hps ".as_ptr() as *const i8,
    b"flatzone.hps ".as_ptr() as *const i8,
    b"fourside.hps ".as_ptr() as *const i8,
    b"gameover.hps ".as_ptr() as *const i8,
    b"garden.hps ".as_ptr() as *const i8,
    b"greatbay.hps ".as_ptr() as *const i8,
    b"greens.hps ".as_ptr() as *const i8,
    b"howto.hps ".as_ptr() as *const i8,
    b"howto_s.hps ".as_ptr() as *const i8,
    b"hyaku.hps ".as_ptr() as *const i8,
    b"hyaku2.hps ".as_ptr() as *const i8,
    b"icemt.hps ".as_ptr() as *const i8,
    b"inis1_01.hps ".as_ptr() as *const i8,
    b"inis1_02.hps ".as_ptr() as *const i8,
    b"inis2_01.hps ".as_ptr() as *const i8,
    b"inis2_02.hps ".as_ptr() as *const i8,
    b"intro_es.hps ".as_ptr() as *const i8,
    b"intro_nm.hps ".as_ptr() as *const i8,
    b"item_h.hps ".as_ptr() as *const i8,
    b"item_s.hps ".as_ptr() as *const i8,
    b"izumi.hps ".as_ptr() as *const i8,
    b"kongo.hps ".as_ptr() as *const i8,
    b"kraid.hps ".as_ptr() as *const i8,
    b"menu01.hps ".as_ptr() as *const i8,
    b"menu02.hps ".as_ptr() as *const i8,
    b"menu3.hps ".as_ptr() as *const i8,
    b"mrider.hps ".as_ptr() as *const i8,
    b"mutecity.hps ".as_ptr() as *const i8,
    b"old_dk.hps ".as_ptr() as *const i8,
    b"old_kb.hps ".as_ptr() as *const i8,
    b"old_ys.hps ".as_ptr() as *const i8,
    b"onetto.hps ".as_ptr() as *const i8,
    b"onetto2.hps ".as_ptr() as *const i8,
    b"opening.hps ".as_ptr() as *const i8,
    b"pokesta.hps ".as_ptr() as *const i8,
    b"pstadium.hps ".as_ptr() as *const i8,
    b"pura.hps ".as_ptr() as *const i8,
    b"rcruise.hps ".as_ptr() as *const i8,
    b"s_info1.hps ".as_ptr() as *const i8,
    b"s_info2.hps ".as_ptr() as *const i8,
    b"s_info3.hps ".as_ptr() as *const i8,
    b"s_new1.hps ".as_ptr() as *const i8,
    b"s_new2.hps ".as_ptr() as *const i8,
    b"s_newcom.hps ".as_ptr() as *const i8,
    b"s_select.hps ".as_ptr() as *const i8,
    b"saria.hps ".as_ptr() as *const i8,
    b"shrine.hps ".as_ptr() as *const i8,
    b"siren.hps ".as_ptr() as *const i8,
    b"smari3.hps ".as_ptr() as *const i8,
    b"sp_end.hps ".as_ptr() as *const i8,
    b"sp_giga.hps ".as_ptr() as *const i8,
    b"sp_metal.hps ".as_ptr() as *const i8,
    b"sp_zako.hps ".as_ptr() as *const i8,
    b"swm_15min.hps ".as_ptr() as *const i8,
    b"target.hps ".as_ptr() as *const i8,
    b"venom.hps ".as_ptr() as *const i8,
    b"vl_battle.hps ".as_ptr() as *const i8,
    b"vl_castle.hps ".as_ptr() as *const i8,
    b"vl_corneria.hps ".as_ptr() as *const i8,
    b"vl_cosmos.hps ".as_ptr() as *const i8,
    b"vl_figure1.hps ".as_ptr() as *const i8,
    b"vl_figure2.hps ".as_ptr() as *const i8,
    b"vl_fzero.hps ".as_ptr() as *const i8,
    b"vl_last_v2.hps ".as_ptr() as *const i8,
    b"vs_hyou1.hps ".as_ptr() as *const i8,
    b"vs_hyou2.hps ".as_ptr() as *const i8,
    b"yorster.hps ".as_ptr() as *const i8,
    b"ystory.hps ".as_ptr() as *const i8,
    b"zebes.hps ".as_ptr() as *const i8,
    b"testnz.hps ".as_ptr() as *const i8,
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
    core::ptr::null(),
];

static unk_arr_803BC4A0: [[u8; 2]; 0x21] = [
    [56, 3],
    [50, 33],
    [6, 84],
    [30, 30],
    [35, 49],
    [4, 4],
    [34, 75],
    [43, 43],
    [77, 77],
    [1, 1],
    [65, 65],
    [60, 31],
    [66, 66],
    [64, 65],
    [40, 2],
    [64, 64],
    [97, 51],
    [96, 95],
    [34, 74],
    [34, 74],
    [84, 6],
    [74, 74],
    [7, 7],
    [1, 1],
    [63, 63],
    [34, 34],
    [98, 98],
    [98, 98],
    [98, 98],
    [98, 98],
    [98, 98],
    [98, 98],
    [0, 0],
];

static mut offsets_arr_803BC4E4: [[u32; 2]; 160] = [
    [2045824, 0],
    [561760, 1],
    [153024, 0],
    [239264, 1],
    [121888, 0],
    [54176, 0],
    [443328, 1],
    [298400, 0],
    [206368, 0],
    [430560, 1],
    [599712, 1],
    [573216, 1],
    [487936, 0],
    [477600, 1],
    [586208, 1],
    [526752, 1],
    [328672, 1],
    [372992, 1],
    [372736, 0],
    [513088, 1],
    [562912, 1],
    [509024, 0],
    [416736, 1],
    [580128, 1],
    [613088, 1],
    [334528, 1],
    [319040, 1],
    [590176, 1],
    [321472, 1],
    [165344, 1],
    [427872, 1],
    [495520, 1],
    [563712, 0],
    [572832, 0],
    [176224, 0],
    [441760, 1],
    [490240, 0],
    [360064, 0],
    [150144, 0],
    [54208, 0],
    [387872, 0],
    [428896, 0],
    [304096, 0],
    [144480, 0],
    [382272, 1],
    [34880, 0],
    [78592, 0],
    [123104, 0],
    [101504, 0],
    [160576, 0],
    [1168864, 0],
    [348160, 1],
    [363936, 0],
    [663136, 0],
    [1536, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
    [0, 0],
];

static offsets_arr_803BC6A4: [[i32; 2]; 75] = [
    [150050, 330000],
    [150053, 330003],
    [150056, 330006],
    [150059, 330009],
    [260006, 330012],
    [260009, 330015],
    [260012, 330018],
    [260015, 330021],
    [260018, 330024],
    [260021, 330027],
    [80007, 330030],
    [80010, 330033],
    [210069, 330036],
    [210072, 330039],
    [210084, 330042],
    [240076, 330045],
    [240079, 330048],
    [60079, 330051],
    [110100, 330054],
    [110103, 330057],
    [110106, 330060],
    [110109, 330063],
    [110085, 330066],
    [180025, 330069],
    [180007, 330072],
    [160082, 330075],
    [160061, 330078],
    [160064, 330081],
    [220061, 330084],
    [130021, 330087],
    [130024, 330090],
    [280073, 330093],
    [280076, 330096],
    [280088, 330099],
    [90030, 330102],
    [90012, 330105],
    [100096, 330108],
    [100099, 330111],
    [100102, 330114],
    [100105, 330117],
    [100081, 330120],
    [300085, 330123],
    [290063, 330126],
    [310102, 330129],
    [310105, 330132],
    [310108, 330135],
    [250064, 330138],
    [250067, 330141],
    [250070, 330144],
    [250073, 330147],
    [70082, 330150],
    [70061, 330153],
    [70064, 330156],
    [170026, 330159],
    [170008, 330162],
    [190103, 330165],
    [190106, 330168],
    [190109, 330171],
    [200106, 330174],
    [200109, 330177],
    [200112, 330180],
    [200115, 330183],
    [200118, 330186],
    [200121, 330189],
    [200124, 330192],
    [200127, 330195],
    [200130, 330198],
    [270134, 330201],
    [270137, 330204],
    [270140, 330207],
    [270191, 330210],
    [270197, 330213],
    [230067, 330216],
    [230070, 330219],
    [540000, 540000],
];

// ---- functions ----------------------------------------------------------------------------------

pub unsafe extern "C" fn lbAudioAx_8002305C(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("lbAudioAx_8002305C");
    let size = unk_arr_803BC4A0.len() as i32;
    let mex = ext::Mex_FighterBgmForPortCKind(arg0, arg1);
    if mex >= 0 {
        return mex;
    }
    if arg0 >= 0 && arg0 < size {
        return unk_arr_803BC4A0[arg0 as usize][arg1 as usize] as i32;
    }
    0x62
}

pub unsafe extern "C" fn lbAudioAx_80023090(idx: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023090");
    if idx < 0 {
        return 0;
    }
    if idx >= 0x62 {
        return 0;
    }
    flags_arr_803BB800[idx as usize] as i32
}

pub unsafe extern "C" fn lbAudioAx_800230C8(i: i32, lo: *mut i32, hi: *mut i32) -> i32 {
    melee_core::ct!("lbAudioAx_800230C8");
    if i < 0 {
        return 1;
    }
    if i >= LBAX_N!() {
        return 1;
    }
    if !lo.is_null() {
        *lo = s32_arr_803BB8D4[i as usize][0];
    }
    if !hi.is_null() {
        *hi = s32_arr_803BB8D4[i as usize][1];
    }
    0
}

pub unsafe extern "C" fn lbAudioAx_80023130(arg0: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023130");
    if LBAX_MEX_ID(arg0) {
        return arg0 / 10000;
    }
    if arg0 >= 0 && arg0 < 0x83D60 {
        for i in 0..LBAX_N!() as usize {
            if s32_arr_803BB8D4[i][0] <= arg0 && arg0 <= s32_arr_803BB8D4[i][1] {
                return i as i32;
            }
        }
    }
    55
}

pub unsafe extern "C" fn lbAudioAx_80023220(idx: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023220");
    if idx >= 0 && idx < LBAX_N!() {
        return s32_arr_803BB5D0[idx as usize][3] as i32;
    }
    0
}

/// Shift the priority list down one slot from `count` to make room.
unsafe fn fn_80023254_shift(count: i32) {
    let mut i = LBAX_N!();
    while i > count {
        lbl_80433B44[i as usize] = lbl_80433B44[(i - 1) as usize];
        i -= 1;
    }
}

unsafe fn fn_80023254(arg0: i32) {
    let mut used = [false; LBAX_CAP];
    let mut count: i32 = 0;
    let mut i = 0i32;
    while {
        count = 0;
        i < LBAX_N!() + 1
    } {
        lbl_80433B44[i as usize] = 55;
        used[i as usize] = false;
        i += 1;
    }

    while count <= LBAX_N!() {
        let mut index = 0i32;
        while index <= LBAX_N!() {
            if arg0 == s32_arr_803BB5D0[index as usize][0] as i32
                && !used[index as usize]
                && offsets_arr_803BC4E4[lbl_80433B44[count as usize] as usize][0] < offsets_arr_803BC4E4[index as usize][0]
            {
                fn_80023254_shift(count);
                lbl_80433B44[count as usize] = index;
                used[index as usize] = true;
            }
            index += 1;
        }
        count += 1;
    }
}

pub unsafe extern "C" fn lbAudioAx_800233EC(arg0: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800233EC");
    if fn_80026E58(0x21) {
        let slot = lbAudioAx_80023130(arg0);
        match slot {
            6..=11 | 13 | 15..=31 => {
                for i in 0..0x4Ausize {
                    if arg0 == offsets_arr_803BC6A4[i][0] {
                        return offsets_arr_803BC6A4[i][1];
                    }
                }
            }
            _ => {}
        }
    } else {
        let slot = lbAudioAx_80023130(arg0);
        if slot == 0x21 {
            for i in 0..0x4Ausize {
                if arg0 == offsets_arr_803BC6A4[i][1] {
                    return offsets_arr_803BC6A4[i][0];
                }
            }
        }
    }
    arg0
}

pub unsafe extern "C" fn lbAudioAx_80023694() -> i32 {
    melee_core::ct!("lbAudioAx_80023694");
    ext::HSD_AudioSFXKeyOffAll();
    -1
}

pub unsafe extern "C" fn lbAudioAx_800236B8(voice: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800236B8");
    ext::HSD_AudioSFXKeyOff(voice);
    -1
}

pub unsafe extern "C" fn lbAudioAx_800236DC() -> i32 {
    melee_core::ct!("lbAudioAx_800236DC");
    ext::AXDriverStop();
    lbl_804D6418 = 0;
    cur_hps_stem[0] = 0;
    -1
}

pub unsafe extern "C" fn lbAudioAx_80023710(arg0: i32) -> bool {
    melee_core::ct!("lbAudioAx_80023710");
    ext::HSD_AudioSFXCheck(arg0) != 0
}

pub unsafe extern "C" fn lbAudioAx_80023730() -> bool {
    melee_core::ct!("lbAudioAx_80023730");
    ext::AXDriverCheck() != 0
}

static mut fn_80023750_logged: i32 = 0;

unsafe fn fn_80023750(id: i32, mut vol: i32, mut pan: i32, track: i32, channel: i32) -> i32 {
    vol = vol.wrapping_mul(2);
    if vol < 0 {
        vol = 0;
    }
    if vol > 0xFF {
        vol = 0xFF;
    }
    pan = pan.wrapping_mul(2);
    if pan < 0 {
        pan = 0;
    }
    if pan > 0xFF {
        pan = 0xFF;
    }
    if id >= 550000 && id < 0x83D60 + 1000000 {
        let r = ext::HSD_AudioSFXStartParam(id, vol, pan, track, channel);
        if fn_80023750_logged < 12 {
            fn_80023750_logged += 1;
            ext::OSReport(&format!("lbAudioAx: m-ex sfx {} -> voice {}\n", id, r));
        }
        return r;
    }
    ext::HSD_AudioSFXStartParam(id, vol, pan, track, channel)
}

pub unsafe extern "C" fn lbAudioAx_800237A8(id: i32, vol: i32, pan: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800237A8");
    if LBAX_MEX_ID(id) {
        return fn_80023750(id, vol, pan, 0, 7);
    }
    if id >= 0x83D61 {
        fn_80023750(0x83D60, 0, PAN_MID, 0, 7)
    } else {
        fn_80023750(id, vol, pan, 0, 7)
    }
}

pub unsafe extern "C" fn lbAudioAx_80023870(id: i32, vol: i32, pan: i32, track: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023870");
    if track == 0 {
        return lbAudioAx_800237A8(id, vol, pan);
    }
    if id == 0x83D61 {
        ext::HSD_AudioSFXKeyOffTrack(track);
        return -1;
    }
    fn_80023750(id, vol, pan, track, 7)
}

pub unsafe extern "C" fn lbAudioAx_8002392C() {
    melee_core::ct!("lbAudioAx_8002392C");
    ext::lbArchive_LoadSymbols(c"LbAd.dat".as_ptr(), &raw mut lbl_804D6454, c"lbAudioLoadData".as_ptr(), 0);
}

unsafe fn getAudioLoadData(arg0: i32) -> *mut i32 {
    if ext::lbLang_IsSettingUS() != 0 {
        if ext::lbLang_IsSavedLanguageUS() != 0 {
            *(*lbl_804D6454).x4.offset(arg0 as isize)
        } else {
            *(*lbl_804D6454).xC.offset(arg0 as isize)
        }
    } else if ext::lbLang_IsSavedLanguageUS() != 0 {
        *(*lbl_804D6454).x8.offset(arg0 as isize)
    } else {
        *(*lbl_804D6454).x0.offset(arg0 as isize)
    }
}

pub unsafe extern "C" fn lbAudioAx_80023968(arg0: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023968");
    let mut i = 0;
    let mut var_r4 = getAudioLoadData(arg0);
    while *var_r4 != 0x83D60 {
        var_r4 = var_r4.add(1);
        i += 1;
    }
    i
}

pub unsafe extern "C" fn lbAudioAx_80023A44(arg0: i32, arg1: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023A44");
    let var_r3 = getAudioLoadData(arg0);
    *var_r3.offset(arg1 as isize)
}

/// A pure busy loop (the C's timing delay); the result is unused.
#[inline(never)]
fn fn_800269AC_delay() {
    let mut total: i32 = 0;
    for _j in 0..0x3E8 {
        for _k in 0..0x3E8 {
            let mut s = 0i32;
            for m in 0..9 {
                s += m;
            }
            total = total.wrapping_add(s);
        }
    }
    core::hint::black_box(total);
}

pub unsafe extern "C" fn lbAudioAx_80023B24(id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023B24");
    let slot = lbAudioAx_80023130(id) as usize;

    if s32_arr_803BB5D0[slot][1] != 5 {
        ext::HSD_AudioSFXKeyOffAll();
        if lbl_80433984[slot] != 2 {
            fn_800269AC_delay();
            ext::HSD_SynthSFXUnloadBank(2);

            for k in 0..LBAX_N!() as usize {
                if s32_arr_803BB5D0[k][2] != 5 {
                    lbl_804337C4[k] = -1;
                    lbl_804338A4[k] = -1;
                    lbl_80433984[k] = -1;
                    lbl_80433A64[k] = -1;
                }
            }

            strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, ssm_files[slot]);
            lbl_80433A64[slot] = ext::HSD_SynthSFXLoad(cur_ssm_file.as_ptr() as *const i8, 2, None, 0);
            ext::HSD_SynthSFXWaitForLoadCompletion(Some(lb_800195D0_cb));
            lbl_80433984[slot] = 2;
        }
    }

    fn_80023750(id, VOL_MAX, PAN_MID, 0, 7)
}

unsafe fn fn_80023ED4(path: *const i8, mut vol: i32, mut arg2: i32) -> bool {
    vol = vol.wrapping_mul(2);
    if vol < 0 {
        vol = 0;
    }
    if vol > 0xFF {
        vol = 0xFF;
    }
    if arg2 < 0 {
        arg2 = 0;
    }
    if arg2 >= 9 {
        arg2 = 8;
    }
    ext::HSD_AudioPStreamStartChParam(path, vol, arg2) != 0
}

unsafe fn getHPSFile(arg0: i32) -> *const i8 {
    if arg0 < 0 {
        null()
    } else if arg0 >= lbAx_BgmN {
        null()
    } else {
        strcpy_at(&raw mut cur_hps_file, hps_stem_pos, hps_files[arg0 as usize]);
        cur_hps_file.as_ptr() as *const i8
    }
}

unsafe fn lbAudioAx_80023F28_helper1(filename: *const i8) -> i32 {
    if filename.is_null() || *filename == 0 {
        1
    } else if !cstr_eq(cur_hps_stem.as_ptr(), filename) {
        lbAudioAx_800236DC();
        // strcpy(cur_hps_stem, filename)
        strcpy_at(&raw mut cur_hps_stem, 0, filename);
        fn_80023ED4(cur_hps_stem.as_ptr() as *const i8, VOL_MAX, 1);
        0
    } else {
        1
    }
}

pub unsafe extern "C" fn lbAudioAx_80023F28(arg0: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80023F28");
    if arg0 < 0 || arg0 >= lbAx_BgmN {
        return 1;
    }
    let filename = getHPSFile(arg0);
    if filename.is_null() {
        return 1;
    }
    lbAudioAx_80023F28_helper1(filename)
}

pub unsafe extern "C" fn lbAudioAx_80024030(arg0: i32) {
    melee_core::ct!("lbAudioAx_80024030");
    static lbl_803BC918: [lbl_803BC918_t; 11] = [
        lbl_803BC918_t { id: 0xAC, track: 0x13, channel: 3 },
        lbl_803BC918_t { id: 0xAD, track: 0x14, channel: 3 },
        lbl_803BC918_t { id: 0xAE, track: 0x00, channel: 3 },
        lbl_803BC918_t { id: 0xAF, track: 0x00, channel: 3 },
        lbl_803BC918_t { id: 0xB0, track: 0x15, channel: 3 },
        lbl_803BC918_t { id: 0xB1, track: 0x00, channel: 2 },
        lbl_803BC918_t { id: 0xC1, track: 0x00, channel: 2 },
        lbl_803BC918_t { id: 0xBC, track: 0x00, channel: 2 },
        lbl_803BC918_t { id: 0xDB, track: 0x00, channel: 3 },
        lbl_803BC918_t { id: 0xDC, track: 0x00, channel: 3 },
        lbl_803BC918_t { id: 0xDD, track: 0x00, channel: 3 },
    ];
    if arg0 < 0xB {
        let e = &lbl_803BC918[arg0 as usize];
        fn_80023750(e.id, VOL_MAX, PAN_MID, e.track, e.channel);
    }
}

pub unsafe extern "C" fn lbAudioAx_800240B4(id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800240B4");
    fn_80023750(id, VOL_MAX, PAN_MID, 0, 5)
}

pub unsafe extern "C" fn lbAudioAx_8002411C(id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_8002411C");
    fn_80023750(id, VOL_MAX, PAN_MID, 0, 6)
}

pub unsafe extern "C" fn lbAudioAx_80024184(id: i32, vol: i32, pan: i32, mut track: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80024184");
    track = match id {
        0x56 => 9,
        0x57 => 8,
        0x58 => 7,
        0x59 => 0xC,
        0x5A => 0xB,
        0x5B => 0xA,
        0x6F => 0xD,
        0x70 => 0xE,
        0x71 => 0xF,
        0x65 => 0x10,
        0x66 => 0x11,
        0x67 => 0x12,
        0x54 => 0x17,
        0x12E => 0x18,
        0x20D => {
            if track != -1 {
                track
            } else {
                0
            }
        }
        _ => 0,
    };
    fn_80023750(id, vol, pan, track, 7)
}

pub unsafe extern "C" fn lbAudioAx_80024304(mut id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80024304");
    let track;
    match id {
        0x8A | 0x8B | 0x8C => {
            track = 0x16;
            id = 0x8B;
        }
        _ => track = 0,
    }
    fn_80023750(id, VOL_MAX, PAN_MID, track, 7)
}

pub unsafe extern "C" fn lbAudioAx_8002438C(id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_8002438C");
    fn_80023750(id, VOL_MAX, PAN_MID, 0, 8)
}

pub unsafe extern "C" fn lbAudioAx_800243F4(id: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800243F4");
    // The C is a fallthrough switch: entering at the k-th label runs every `track += 1` from there
    // to the end; the first label (0x7C830) first sets track = 0xCB.
    const LABELS: [i32; 28] = [
        0x7C830, 0x7C831, 0x7C835, 0x7C83A, 0x7C83F, 0x7C840, 0x7C842, 0x7C844, 0x7C845, 0x7C846, 0x7C848, 0x7C84A, 0x7C84B, 0x7C84D,
        0x7C83B, 0x7C83D, 0x7C84E, 0x7C84F, 0x7C850, 0x7C851, 0x7C834, 0x7C843, 0x7C832, 0x7C83C, 0x7C84C, 0x7C836, 0x7C849, 0x7C838,
    ];
    let n = LABELS.len();
    let mut track = 0xCA;
    if let Some(k) = LABELS.iter().position(|&l| l == id) {
        if k == 0 {
            track = 0xCB + (n as i32 - 1);
        } else {
            track = 0xCA + (n - k) as i32;
        }
    }
    fn_80023750(id, VOL_MAX, PAN_MID, track, 7)
}

unsafe fn fn_800244F4() {
    lbl_804D387C = 0x7F;
    lbl_804D3884 = 0x7F;
    lbl_804D388C = 0x7F;
    lbl_804D3894 = 0x7F;
    lbl_804D389C = 0x7F;
    lbl_804D38A4 = 0x7F;
    lbl_804D38D8 = 1;
    lbl_804D38C0 = 1.0;
    lbl_804D38AC = 1.0;
    lbl_804D38B0 = 1.0;
    lbl_804D38B4 = 1.0;
    lbl_804D38B8 = 1.0;
    lbl_804D38BC = 1.0;
    lbl_804D640C = 0;
    paused = false;
    lbl_804D38E4 = 1.0;
    lbl_804D38E8 = 1.0;
    lbl_804D38EC = 1.0;
    lbl_804D6414 = 0;
    lbl_804D6418 = 0;
    lbl_804D6420 = 0;
    lbl_804D6424 = 0;
    lbl_804D6428 = 0;
    lbl_804D642C = 0;
    lbl_804D38F0 = -1;
    lbl_804D38F4 = -1;
    lbl_804D6430 = 0;
    lbl_804D6434 = 0;
    lbl_804D641C = 0;
    lbl_804D63F4 = 0;
    lbl_804D63F8 = 0;
    lbl_804D63FC = 0;
    lbl_804D6400 = 0;
    lbl_804D6404 = 0;
    synth_volume = 1.0;
    lbl_804D3888 = 0x7F;
    lbl_804D3890 = 0x7F;
    lbl_804D3898 = 0x7F;
    lbl_804D38A0 = 0x7F;
    lbl_804D38A8 = 0x7F;
    lbl_804D63F0 = 0.0;
    lbl_804D38CC = 0x7F;
    ext::gm_801603B0();
}

pub unsafe extern "C" fn lbAudioAx_800245D4(mut arg0: i32) {
    melee_core::ct!("lbAudioAx_800245D4");
    if arg0 < 0 {
        arg0 = 0;
    }
    if arg0 > 0x7F {
        arg0 = 0x7F;
    }
    lbl_804D387C = arg0;
}

pub unsafe extern "C" fn lbAudioAx_800245F4(mut arg0: i32) -> i32 {
    melee_core::ct!("lbAudioAx_800245F4");
    if arg0 < 0 {
        arg0 = 0;
    }
    if arg0 > 0x7F {
        arg0 = 0x7F;
    }
    lbl_804D388C = arg0;
    arg0
}

pub unsafe extern "C" fn lbAudioAx_80024614(mut arg0: i32) {
    melee_core::ct!("lbAudioAx_80024614");
    if arg0 < 0 {
        arg0 = 0;
    }
    if arg0 > 0x7F {
        arg0 = 0x7F;
    }
    lbl_804D3884 = arg0;
}

pub unsafe extern "C" fn lbAudioAx_80024634(mut arg0: i32) {
    melee_core::ct!("lbAudioAx_80024634");
    if arg0 < 0 {
        arg0 = 0;
    }
    if arg0 > 0xFF {
        arg0 = 0xFF;
    }
    lbl_804D38D8 = arg0;
}

#[inline]
fn u8c(f: f32) -> i32 {
    cabi::f2u8(f as f64) as i32
}

unsafe fn fn_80024654(arg0: i32) {
    synth_volume = (lbl_804D387C as f32 / 127.0f32) * lbl_804D38C0;
    if arg0 == 1 {
        ext::HSD_SynthStreamSetVolume(synth_volume);
        lbl_804D63F0 = synth_volume;
    } else if lbl_804D63F0 != synth_volume {
        ext::HSD_SynthStreamSetVolume(synth_volume);
        lbl_804D63F0 = synth_volume;
    }
    let temp_f31 = lbl_804D38CC as f32 / 127.0f32;
    lbl_804D38B0 = lbl_804D38C8;
    lbl_804D38B4 = lbl_804D38C8 * temp_f31;
    lbl_804D3898 = u8c(lbl_804D38B4 * (0.88f32 * (lbl_804D388C.wrapping_mul(2)) as f32));
    if arg0 == 1 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3898 as u8, 4, 2);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3898 as u8, 4, 3);
        lbl_804D63FC = lbl_804D3898;
    } else if lbl_804D63FC != lbl_804D3898 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3898 as u8, 4, 2);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3898 as u8, 4, 3);
        lbl_804D63FC = lbl_804D3898;
    }
    lbl_804D38B8 = temp_f31 * (lbl_804D38C8 * lbl_804D38E4);
    lbl_804D38A0 = u8c(lbl_804D38B8 * (0.88f32 * (lbl_804D388C.wrapping_mul(2)) as f32));
    if arg0 == 1 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 4);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 5);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 6);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 7);
        lbl_804D6400 = lbl_804D38A0;
    } else if lbl_804D6400 != lbl_804D38A0 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 4);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 5);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 6);
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A0 as u8, 4, 7);
        lbl_804D6400 = lbl_804D38A0;
    }
    lbl_804D38BC = lbl_804D38C8 * lbl_804D38E4;
    lbl_804D38A8 = u8c(lbl_804D38BC * (0.88f32 * (lbl_804D388C.wrapping_mul(2)) as f32));
    if arg0 == 1 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A8 as u8, 4, 8);
        lbl_804D6404 = lbl_804D38A8;
    } else if lbl_804D6404 != lbl_804D38A8 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D38A8 as u8, 4, 8);
        lbl_804D6404 = lbl_804D38A8;
    }
    lbl_804D38AC = temp_f31 * (lbl_804D38EC * (lbl_804D38C4 * lbl_804D38E8));
    lbl_804D3888 = u8c(lbl_804D38AC * (lbl_804D3884.wrapping_mul(2)) as f32);
    if arg0 == 1 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3888 as u8, 4, 1);
        lbl_804D63F4 = lbl_804D3888;
    } else if lbl_804D63F4 != lbl_804D3888 {
        ext::HSD_SynthSFXUpdateAllVolume(lbl_804D3888 as u8, 4, 1);
        lbl_804D63F4 = lbl_804D3888;
    }
    lbl_804D38E0 = lbl_804D38D8;
    if arg0 == 1 {
        ext::HSD_AudioSFXSetMixGroup(5, 1, 0x20);
        ext::HSD_AudioSFXSetMixGroup(6, 1, 0x20);
        ext::HSD_AudioSFXSetMixGroup(8, 1, lbl_804D38D8 as u8);
        ext::HSD_AudioSFXSetMixGroup(7, 1, lbl_804D38D8 as u8);
        lbl_804D38DC = lbl_804D38E0;
        return;
    }
    if lbl_804D38DC != lbl_804D38E0 {
        ext::HSD_AudioSFXSetMixGroup(5, 1, 0x20);
        ext::HSD_AudioSFXSetMixGroup(6, 1, 0x20);
        ext::HSD_AudioSFXSetMixGroup(8, 1, lbl_804D38D8 as u8);
        ext::HSD_AudioSFXSetMixGroup(7, 1, lbl_804D38D8 as u8);
        lbl_804D38DC = lbl_804D38E0;
    }
}

pub unsafe extern "C" fn lbAudioAx_80024B1C(voice: i32, mut pan: i32) {
    melee_core::ct!("lbAudioAx_80024B1C");
    if pan < 0 {
        pan = 0;
    }
    if pan > 0x7F {
        pan = 0x7F;
    }
    ext::HSD_AudioSFXSetPan(voice, pan.wrapping_mul(2));
}

pub unsafe extern "C" fn lbAudioAx_80024B58(voice: i32, mut vol: i32) {
    melee_core::ct!("lbAudioAx_80024B58");
    if vol < 0 {
        vol = 0;
    }
    if vol > VOL_MAX {
        vol = VOL_MAX;
    }
    ext::HSD_AudioSFXSetVolumeEx(voice, vol.wrapping_mul(2));
}

pub unsafe extern "C" fn lbAudioAx_80024B94(voice: i32, mut arg1: i32) -> i32 {
    melee_core::ct!("lbAudioAx_80024B94");
    let limit = 0x4B0;
    if arg1 < -limit {
        arg1 = -limit;
    }
    if arg1 > limit {
        arg1 = limit;
    }
    ext::HSD_AudioSFXSetPitchFid(voice, arg1)
}

pub unsafe extern "C" fn lbAudioAx_80024BD0() -> bool {
    melee_core::ct!("lbAudioAx_80024BD0");
    let mode = ext::HSD_SynthGetSoundMode();
    sound_mode = mode as i32;
    mode != 1
}

/// Set sound mode, returns true if successful
pub unsafe extern "C" fn lbAudioAx_80024C08(mode: u32) -> bool {
    melee_core::ct!("lbAudioAx_80024C08");
    static to_synth_mode: [u32; 2] = [1, 0];
    match mode {
        AUDIO_MODE_UNK0 | AUDIO_MODE_UNK1 => {
            if sound_mode as u32 != to_synth_mode[mode as usize] {
                sound_mode = to_synth_mode[mode as usize] as i32;
                if to_synth_mode[mode as usize] != 1 && to_synth_mode[mode as usize] != 0 {
                    return false;
                }
                ext::HSD_SynthSetSoundMode(to_synth_mode[mode as usize]);
                return true;
            }
            true
        }
        _ => false,
    }
}

pub unsafe extern "C" fn lbAudioAx_80024C84() {
    melee_core::ct!("lbAudioAx_80024C84");
    lbl_804D640C = 0;
    paused = false;
    lbl_804D38E4 = 1.0;
    lbl_804D38E8 = 1.0;
    lbl_804D38EC = 1.0;
    lbl_804D6414 = 0;
    lbl_804D6418 = 0;
    lbl_804D6420 = 0;
    lbl_804D6424 = 0;
    lbl_804D6428 = 0;
    lbl_804D642C = 0;
    lbl_804D38F0 = -1;
    lbl_804D38F4 = -1;
    lbl_804D6430 = 0;
    lbl_804D6434 = 0;
    ext::HSD_AudioSFXKeyOffTrack(5);
    ext::HSD_AudioSFXKeyOffTrack(6);
    lbl_804D641C = 0;
    lbl_804D38C0 = 1.0;
    lbl_804D38AC = 1.0;
    lbl_804D38B0 = 1.0;
    lbl_804D38B4 = 1.0;
    lbl_804D38B8 = 1.0;
    lbl_804D38BC = 1.0;
    ext::HSD_AudioPStreamResumeCh(5);
    ext::HSD_AudioPStreamResumeCh(6);
    ext::HSD_AudioPStreamResumeCh(8);
    ext::HSD_AudioPStreamResumeCh(7);
    lbl_804D38D8 = 1;
    lbl_804D38CC = 0x7F;
}

pub unsafe extern "C" fn lbAudioAx_80024D50() {
    melee_core::ct!("lbAudioAx_80024D50");
    lbAudioAx_80024C84();
    lbl_804D641C = 1;
}

/// The stage audio row byte, guarded against a stage id outside the table.
unsafe fn lbAudioAx_StageAudioByte(grkind: i32, which: usize) -> i32 {
    static mut warned: i32 = -1;
    if grkind >= 0 && (grkind as usize) < s32_arr_803BB6B0.len() {
        return s32_arr_803BB6B0[grkind as usize][which] as i32;
    }
    if warned != grkind {
        warned = grkind;
        ext::OSReport(&format!("lbAudioAx: stage grkind={} is outside s32_arr_803BB6B0[{}]\n", grkind, s32_arr_803BB6B0.len()));
    }
    if which == 0 {
        55
    } else {
        1
    }
}

pub unsafe extern "C" fn lbAudioAx_80024D78(arg0: i32) {
    melee_core::ct!("lbAudioAx_80024D78");
    lbl_804D38D8 = lbAudioAx_StageAudioByte(ext::Stage_8022519C(ext::Stage_80225194()), arg0 as usize);
}

pub unsafe extern "C" fn lbAudioAx_80024DC4(arg0: i32) {
    melee_core::ct!("lbAudioAx_80024DC4");
    for i in 0..0x10usize {
        if lbl_8043373C[i] == arg0 {
            lbl_80433780[i] = 0xa;
            return;
        }
    }
    for i in 0..0x10usize {
        if lbl_8043373C[i] == 0x83D60 {
            lbl_8043373C[i] = arg0;
            lbl_80433780[i] = 0xa;
            return;
        }
    }
}

pub unsafe extern "C" fn lbAudioAx_80024E50(pause: bool) {
    melee_core::ct!("lbAudioAx_80024E50");
    paused = pause;
    if pause {
        ext::AXDriverPause();
    } else {
        ext::AXDriverResume();
    }
}

pub unsafe extern "C" fn lbAudioAx_80024E84(arg0: bool) {
    melee_core::ct!("lbAudioAx_80024E84");
    lbl_804D640C = arg0 as i32;
    if arg0 {
        lbl_804D38E4 = 0.2;
        lbl_804D38E8 = 0.2;
        ext::HSD_AudioPStreamPauseCh(5);
        ext::HSD_AudioPStreamPauseCh(6);
        ext::HSD_AudioPStreamPauseCh(8);
        ext::HSD_AudioPStreamPauseCh(7);
    } else {
        lbl_804D38E4 = 1.0;
        lbl_804D38E8 = 1.0;
        ext::HSD_AudioPStreamResumeCh(5);
        ext::HSD_AudioPStreamResumeCh(6);
        ext::HSD_AudioPStreamResumeCh(8);
        ext::HSD_AudioPStreamResumeCh(7);
    }
}

pub unsafe extern "C" fn lbAudioAx_80024F08() {
    melee_core::ct!("lbAudioAx_80024F08");
    ext::HSD_SynthStreamSetVolume(0.0);
    for ch in 2..=9 {
        ext::HSD_AudioPStreamPauseCh(ch);
    }
}

pub unsafe extern "C" fn lbAudioAx_80024F6C() {
    melee_core::ct!("lbAudioAx_80024F6C");
    ext::HSD_SynthStreamSetVolume(synth_volume);
    ext::HSD_AudioPStreamResumeCh(2);
    ext::HSD_AudioPStreamResumeCh(3);
    ext::HSD_AudioPStreamResumeCh(4);
    ext::HSD_AudioPStreamResumeCh(9);
    if lbl_804D640C == 0 {
        ext::HSD_AudioPStreamResumeCh(5);
        ext::HSD_AudioPStreamResumeCh(6);
        ext::HSD_AudioPStreamResumeCh(8);
        ext::HSD_AudioPStreamResumeCh(7);
    }
}

pub unsafe extern "C" fn lbAudioAx_80024FDC() {
    melee_core::ct!("lbAudioAx_80024FDC");
    lbl_804D6430 = 0x1FE;
    lbl_804D6420 += 1;
}

pub unsafe extern "C" fn lbAudioAx_80024FF4() {
    melee_core::ct!("lbAudioAx_80024FF4");
    lbl_804D6434 = 0x1E0;
    lbl_804D6424 += 1;
}

pub unsafe extern "C" fn lbAudioAx_8002500C(arg0: i32) {
    melee_core::ct!("lbAudioAx_8002500C");
    if arg0 > 0 {
        lbl_804D6420 -= arg0;
        if lbl_804D6420 <= 0 {
            lbl_804D6420 = 0;
        }
    }
}

pub unsafe extern "C" fn lbAudioAx_80025038(arg0: i32) {
    melee_core::ct!("lbAudioAx_80025038");
    if arg0 > 0 {
        lbl_804D6424 -= arg0;
        if lbl_804D6424 < 0 {
            lbl_804D6424 = 0;
        }
    }
}

pub unsafe extern "C" fn lbAudioAx_80025064(arg0: bool, arg1: bool) {
    melee_core::ct!("lbAudioAx_80025064");
    lbl_804D38C4 = if arg0 { 1.0 } else { 0.0 };
    lbl_804D38C8 = if arg1 { 1.0 } else { 0.0 };
}

pub unsafe extern "C" fn lbAudioAx_80025098(debug: bool) {
    melee_core::ct!("lbAudioAx_80025098");
    debug_enabled = debug;
}

pub fn calcPan(mut current: i32, end: i32, left: i32, right: i32) -> i32 {
    melee_core::ct!("calcPan");
    if current > end {
        current = end;
    }
    if left != right {
        let mut result: f32;
        if left <= right {
            result = left as f32 + (current as f32 * (right as f32 - left as f32)) / end as f32;
        } else {
            result = right as f32 + (current as f32 * (left as f32 - right as f32)) / end as f32;
        }
        if result < 0.0f32 {
            result = 0.0;
        }
        if result > 127.0f32 {
            result = 127.0;
        }
        return cabi::f2i32(result as f64);
    }
    64
}

#[inline]
fn abs_f(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

unsafe fn ud_of(gobj: *mut HSD_GObj) -> *mut lbAudioAx_UserData {
    (*gobj).user_data as *mut lbAudioAx_UserData
}

unsafe fn soundGetPosition(gobj: *mut HSD_GObj, pos: *mut Vec3) -> i32 {
    if !gobj.is_null() && !(*gobj).user_data.is_null() && !(*ud_of(gobj)).owner.is_null() {
        match (*(*ud_of(gobj)).owner).classifier as i32 {
            HSD_GOBJ_CLASS_FIGHTER => {
                ext::ftLib_80086644((*ud_of(gobj)).owner, pos);
                return 0;
            }
            HSD_GOBJ_CLASS_ITEM => {
                ext::it_8026B294((*ud_of(gobj)).owner, pos);
                return 0;
            }
            _ => {}
        }
        (*pos).x = 0.0;
        (*pos).y = 0.0;
        (*pos).z = 0.0;
    }
    1
}

unsafe extern "C" fn fn_800251EC(gobj: *mut HSD_GObj) -> i32 {
    let mut cam_left = 0.0f32;
    let mut cam_center = 0.0f32;
    let mut cam_right = 0.0f32;
    let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            ext::Camera_800307D0(&mut cam_left, &mut cam_center, &mut cam_right);

            if soundGetPosition(gobj, &mut pos) == 1 {
                return 1;
            }

            let mut pan = 64.0f32;

            if cam_left < cam_center && cam_center < cam_right && cam_left < pos.x && pos.x < cam_right {
                if cam_center < pos.x {
                    if (*ud).pan_right > PAN_MID {
                        let ratio = (pos.x - cam_center) / (cam_right - cam_center);
                        let mut temp = ((*ud).pan_right - PAN_MID) as f32;
                        temp = temp * ratio;
                        pan = 64.0f32 + abs_f(temp);
                    }
                } else if cam_center > pos.x {
                    if (*ud).pan_left < PAN_MID {
                        let ratio = (cam_center - pos.x) / (cam_center - cam_left);
                        let mut temp = (PAN_MID - (*ud).pan_left) as f32;
                        temp = temp * ratio;
                        pan = 64.0f32 - abs_f(temp);
                    }
                }
            }

            (*ud).pan = cabi::f2i32(pan as f64);
            return 0;
        }
    }
    1
}

unsafe extern "C" fn fn_800253D8(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            if (*ud).x3C == 1.0 {
                (*ud).pan = calcPan((*ud).current_frame, (*ud).end_frame, (*ud).pan_left, (*ud).pan_right);
            } else {
                (*ud).pan = 0x7F - calcPan((*ud).current_frame, (*ud).end_frame, (*ud).pan_left, (*ud).pan_right);
            }
        }
    }
    0
}

unsafe extern "C" fn fn_800256BC(gobj: *mut HSD_GObj) -> i32 {
    fn_800253D8(gobj)
}

unsafe extern "C" fn fn_800259A0(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            if (*ud).x44 == 0 {
                (*ud).x44 = 1;
                fn_800251EC(gobj);
            }
        }
    }
    0
}

unsafe extern "C" fn fn_800259EC(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            if (*ud).x44 == 0 {
                (*ud).x44 = 1;
                fn_800253D8(gobj);
            }
        }
    }
    0
}

unsafe extern "C" fn fn_80025A98(gobj: *mut HSD_GObj) -> i32 {
    fn_800259EC(gobj)
}

unsafe extern "C" fn fn_80025B44(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            (*ud).pan = calcPan((*ud).current_frame, (*ud).end_frame, (*ud).pan_left, (*ud).pan_right);
        }
    }
    0
}

unsafe extern "C" fn fn_80025CBC(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            (*ud).pan = 0x7F - calcPan((*ud).current_frame, (*ud).end_frame, (*ud).pan_left, (*ud).pan_right);
        }
    }
    0
}

unsafe extern "C" fn fn_80025E38(gobj: *mut HSD_GObj) -> i32 {
    if !gobj.is_null() {
        let ud = ud_of(gobj);
        if !ud.is_null() {
            if (*ud).current_frame <= (*ud).end_frame {
                let span = abs_f((*ud).end_vol as f32 - (*ud).start_vol as f32);
                let frac = ((*ud).current_frame as f32 / (*ud).end_frame as f32) * span;
                if (*ud).start_vol < (*ud).end_vol {
                    (*ud).vol = (*ud).start_vol.wrapping_add(cabi::f2i32(frac as f64));
                } else {
                    (*ud).vol = (*ud).end_vol.wrapping_sub(cabi::f2i32(frac as f64));
                }
            } else {
                (*ud).vol = VOL_MAX;
            }
        }
    }
    0
}

unsafe fn fn_80025FAC(gobj: *mut HSD_GObj, ud: *mut lbAudioAx_UserData, sp: *mut SoundParams) {
    static lbl_803BCA24: [unsafe extern "C" fn(*mut HSD_GObj) -> i32; 10] = [
        fn_800251EC,
        fn_800253D8,
        fn_800256BC,
        fn_800259A0,
        fn_800259EC,
        fn_80025A98,
        fn_80025B44,
        fn_80025CBC,
        fn_80025E38,
        fn_80025E38,
    ];

    if !gobj.is_null() && !ud.is_null() && !sp.is_null() {
        (*ud).gobj = gobj;
        (*ud).owner = (*sp).owner;
        (*ud).xC = (*sp).x4;
        (*ud).id = (*sp).sfx_id;
        (*ud).current_frame = 0;
        (*ud).end_frame = (*sp).end_frame;
        if (*sp).x20 == 0.0 {
            (*ud).x3C = if ext::HSD_Randi(2) == 0 { 1.0 } else { -1.0 };
        } else {
            (*ud).x3C = (*sp).x20;
        }
        (*ud).track = (*sp).x24;
        (*ud).start_vol = (*sp).start_vol;
        (*ud).end_vol = (*sp).end_vol;
        (*ud).pan_left = (*sp).pan_left;
        (*ud).pan_right = (*sp).pan_right;
        (*ud).x44 = 0;
        (*ud).vol = VOL_MAX;
        (*ud).pan = PAN_MID;
        (*ud).x10 = Some(lbl_803BCA24[(*ud).xC as usize]);
        (*ud).voice_id = -1;

        match (*ud).xC {
            0..=9 => {
                if (*ud).start_vol == (*ud).end_vol {
                    (*ud).vol = (*ud).start_vol;
                }
            }
            _ => {}
        }

        match (*ud).xC {
            0..=8 => {
                if (*ud).current_frame < 0 {
                    (*ud).current_frame = 0;
                }
                if (*ud).end_frame < 0 {
                    (*ud).end_frame = 0;
                }
                if (*ud).track == 0 {
                    (*ud).voice_id = lbAudioAx_800237A8((*ud).id, (*ud).vol, (*ud).pan);
                } else {
                    (*ud).voice_id = lbAudioAx_80023870((*ud).id, (*ud).vol, (*ud).pan, (*ud).track);
                }
            }
            9 => {
                if (*ud).current_frame < 0 {
                    (*ud).current_frame = 0;
                }
                if (*ud).end_frame < 0 {
                    (*ud).end_frame = 0;
                }
                (*ud).voice_id = (*sp).voice;
            }
            _ => {}
        }
    }
}

unsafe extern "C" fn fn_800262A0(gobj: *mut HSD_GObj) {
    if gobj.is_null() {
        return;
    }
    let ud = ud_of(gobj);
    if ud.is_null() {
        return;
    }
    let f = match (*ud).x10 {
        Some(f) => f,
        None => return,
    };
    if f(gobj) == 1 {
        if !gobj.is_null() {
            ext::HSD_GObjFree(gobj);
        }
        return;
    }

    if (*ud).voice_id != -1 {
        lbAudioAx_80024B1C((*ud).voice_id, (*ud).pan);
        lbAudioAx_80024B58((*ud).voice_id, (*ud).vol);
    }

    if (*ud).current_frame != -1 && (*ud).end_frame != -1 && ((*ud).current_frame >= (*ud).end_frame || (*ud).end_frame == 0) {
        if !gobj.is_null() {
            ext::HSD_GObjFree(gobj);
        }
    } else {
        (*ud).current_frame += 1;
    }
}

/// Free an object from the audio allocator pool.
unsafe extern "C" fn lbAudioAx_ObjFree(obj: *mut c_void) {
    if !obj.is_null() {
        ext::HSD_ObjFree(&raw mut lbl_80433710, obj);
    }
}

pub unsafe extern "C" fn lbAudioAx_800263E8(
    f1: f32,
    owner: *mut HSD_GObj,
    arg2: i32,
    sfx_id: i32,
    start_vol: i32,
    end_vol: i32,
    pan_left: i32,
    pan_right: i32,
    end_frame: i32,
    arg9: i32,
    voice: i32,
) -> *mut HSD_GObj {
    melee_core::ct!("lbAudioAx_800263E8");
    if sfx_id < 0x83D60 || LBAX_MEX_ID(sfx_id) {
        let mut params = SoundParams {
            owner,
            x4: arg2,
            sfx_id,
            start_vol,
            end_vol,
            pan_left,
            pan_right,
            end_frame,
            x20: f1,
            x24: arg9,
            voice,
        };

        let mut gobj = ext::GObj_Create(HSD_GOBJ_CLASS_SOUND as u16, PLINK_AUDIO_AX, 0);
        if !gobj.is_null() {
            let userdata = ext::HSD_ObjAlloc(&raw mut lbl_80433710);
            if userdata.is_null() {
                ext::HSD_GObjFree(gobj);
                gobj = null_mut();
            } else {
                ext::GObj_InitUserData(gobj, HSD_GOBJ_CLASS_SOUND as u8, Some(lbAudioAx_ObjFree), userdata);

                if !gobj.is_null() {
                    ext::HSD_GObj_SetupProc(gobj, Some(fn_800262A0), HSD_GOBJ_CLASS_SOUND as u8);
                }

                fn_80025FAC(gobj, userdata as *mut lbAudioAx_UserData, &mut params);
            }
        }
        return gobj;
    }
    null_mut()
}

pub unsafe extern "C" fn lbAudioAx_800264E4(data: *mut HSD_GObj) -> i32 {
    melee_core::ct!("lbAudioAx_800264E4");
    if !data.is_null() {
        let ud = ud_of(data);
        if ud.is_null() {
            return -1;
        }
        return (*ud).voice_id;
    }
    -1
}

pub unsafe extern "C" fn lbAudioAx_80026510(target: *mut HSD_GObj) -> bool {
    melee_core::ct!("lbAudioAx_80026510");
    let mut count = 0;
    if !target.is_null() {
        let mut cur = *ext::HSD_GObjPLinkHead.add(PLINK_AUDIO_AX as usize);
        while !cur.is_null() {
            let ud = ud_of(cur);
            let next = (*cur).next;
            if !ud.is_null() && (*ud).owner == target {
                if (*ud).voice_id != -1 {
                    ext::HSD_AudioSFXKeyOff((*ud).voice_id);
                }
                if !cur.is_null() {
                    ext::HSD_GObjFree(cur);
                }
                count += 1;
            }
            cur = next;
        }
    }
    count != 0
}

pub unsafe extern "C" fn lbAudioAx_800265C4(target_obj: *mut HSD_GObj, voice: i32) -> bool {
    melee_core::ct!("lbAudioAx_800265C4");
    let mut cur = *ext::HSD_GObjPLinkHead.add(PLINK_AUDIO_AX as usize);
    while !cur.is_null() {
        let ud = ud_of(cur);
        if !ud.is_null() && (*ud).owner == target_obj && (*ud).voice_id != -1 && (*ud).voice_id == voice {
            ext::HSD_AudioSFXKeyOff((*ud).voice_id);
            if !cur.is_null() {
                ext::HSD_GObjFree(cur);
            }
            return true;
        }
        cur = (*cur).next;
    }
    false
}

unsafe fn fn_80026650() -> i32 {
    let mut priority = 4i32;
    while priority >= 0 {
        for i in 0..LBAX_N!() as usize {
            if priority == s32_arr_803BB5D0[i][1] as i32 && lbl_804338A4[i] == 1 && lbl_80433984[i] == -1 {
                return i as i32;
            }
        }
        priority -= 1;
    }
    -1
}

unsafe fn fn_800267B0() {
    if lbl_804D6450 == 0 {
        return;
    }

    for i in 0..5i32 {
        let mut j = 0usize;
        while (lbl_804D6438 as u32) < (lbl_804D6448 as u32).wrapping_add(lbl_804D6450) && (j as i32) < LBAX_N!() {
            if lbl_80433984[j] != -1 && i == s32_arr_803BB5D0[j][2] as i32 && lbl_804338A4[j] == -1 {
                let x = lbl_80433A64[j];
                ext::HSD_SynthSFXGroupDataRemove(x);
                lbl_80433A64[j] = -1;
                lbl_80433984[j] = -1;
                lbl_804D6448 = lbl_804D6448.wrapping_sub(offsets_arr_803BC4E4[j][0] as i32);
            }
            j += 1;
        }
    }

    ext::HSD_SynthSFXBankDeflag(2);
    ext::HSD_SynthSFXBankDeflagSync();
}

unsafe fn fn_800268B4() {
    lbl_804D6450 = 0;
    lbl_804D644C = 0;
    lbl_804D6448 = 0;

    for i in 0..LBAX_N!() as usize {
        let flag1 = if lbl_804338A4[i] == -1 { 0 } else { 1 };
        let flag2 = if lbl_80433984[i] == -1 { 0 } else { 2 };
        let sz = offsets_arr_803BC4E4[i][0];
        match flag1 | flag2 {
            0 => {}
            1 => {
                lbl_804D644C = lbl_804D644C.wrapping_add(sz as i32);
                lbl_804D6450 = lbl_804D6450.wrapping_add(sz);
            }
            2 => {
                lbl_804D6448 = lbl_804D6448.wrapping_add(sz as i32);
            }
            3 => {
                lbl_804D644C = lbl_804D644C.wrapping_add(sz as i32);
                lbl_804D6448 = lbl_804D6448.wrapping_add(sz as i32);
            }
            _ => {}
        }
    }
}

unsafe fn fn_800269AC() {
    if ext::HSD_SynthSFXGetPendingLoadCount() != 0 {
        for i in 0..LBAX_N!() as usize {
            if s32_arr_803BB5D0[i][1] != 5 && lbl_80433A64[i] != -1 && lbl_80433984[i] == -1 {
                for _retry in 0..0x40 {
                    if ext::HSD_SynthSFXCancelLoad(lbl_80433A64[i]) == 1 {
                        break;
                    }
                    fn_800269AC_delay();
                }
                lbl_80433A64[i] = -1;
                lbl_80433984[i] = -1;
            }
        }

        if ext::HSD_SynthSFXGetPendingLoadCount() != 0 {
            ext::HSD_SynthSFXWaitForLoadCompletion(Some(lb_800195D0_cb));
            ext::HSD_SynthSFXUnloadBank(2);
            for i in 0..LBAX_N!() as usize {
                if s32_arr_803BB5D0[i][1] != 5 {
                    lbl_80433984[i] = -1;
                    lbl_80433A64[i] = -1;
                }
            }
        }
    }
}

unsafe extern "C" fn fn_80026C04(arg0: i32, _unused: i32) {
    if arg0 != -1 {
        for i in 0..LBAX_N!() as usize {
            if arg0 == lbl_80433A64[i] {
                lbl_80433984[i] = 1;
                let sz = offsets_arr_803BC4E4[i][0];
                lbl_804D644C = lbl_804D644C.wrapping_sub(sz as i32);
                lbl_804D6450 = lbl_804D6450.wrapping_sub(sz);
                lbl_804D6448 = lbl_804D6448.wrapping_add(sz as i32);
                break;
            }
        }
    }

    let slot = fn_80026650();
    if slot != -1 {
        strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, ssm_files[slot as usize]);
        lbl_80433A64[slot as usize] = ext::HSD_SynthSFXLoad(cur_ssm_file.as_ptr() as *const i8, 2, Some(fn_80026C04), 0);
    }
}

unsafe fn fn_80026E58(arg0: i32) -> bool {
    lbl_80433984[arg0 as usize] == 2
}

pub unsafe extern "C" fn lbAudioAx_80026E84(mut ckind: i32) -> u64 {
    melee_core::ct!("lbAudioAx_80026E84");
    let native_base = ext::Geno_DefineBaseCK(ckind);
    if native_base >= 0 {
        ckind = native_base;
    }
    if LBAX_N!() > 55 && ckind >= 0 {
        let ssm = ext::Mex_SsmForPortCKind(ckind);
        if ssm >= 0 {
            return if ssm < 64 { 1u64 << ssm } else { 0 };
        }
        ckind = ext::gm_MexVanillaKind(ckind, 1);
    }
    if ckind < 0 || ckind >= ChKind_Max as i32 {
        return 0;
    }
    lbl_803BB3C0[ckind as usize].x8
}

pub unsafe extern "C" fn lbAudioAx_80026EBC(stkind: u32) -> u64 {
    melee_core::ct!("lbAudioAx_80026EBC");
    let imax = s32_arr_803BB6B0.len() as i32;
    let grkind = ext::Stage_8022519C(stkind);

    if grkind < 0 || grkind >= imax {
        return 0;
    }
    let shift = lbAudioAx_StageAudioByte(grkind, 0);
    if shift == 55 {
        return 0;
    }
    if shift < 0 || shift >= LBAX_N!() {
        return 0;
    }
    if shift >= 64 {
        lbAx_stage_pending[shift as usize] = 1;
        return 0;
    }
    1u64 << shift
}

pub unsafe extern "C" fn lbAudioAx_80026F2C(flags: u32) {
    melee_core::ct!("lbAudioAx_80026F2C");
    let mut mask: u64 = 0;

    if flags & 1 != 0 {
        mask = mask.wrapping_add(0x0048000000000003);
    }
    if flags & 2 != 0 {
        mask = mask.wrapping_add(0x000000000000003C);
    }
    if flags & 4 != 0 {
        mask = mask.wrapping_add(0x00800003FFFFFFC0);
    }
    if flags & 8 != 0 {
        mask = mask.wrapping_add(0x0023FFFC00000000);
    }
    if flags & 16 != 0 {
        mask = mask.wrapping_add(0x0014000000000000);
    }

    for i in 0..(LBAX_CAP - 1) {
        if mask & 1 != 0 {
            lbl_804337C4[i] = -1;
        }
        mask >>= 1;
    }
    // m-ex banks (56..) are past the retail masks: reset them by LookupTable group.
    let mut i = 56i32;
    while i < LBAX_N!() {
        let g = s32_arr_803BB5D0[i as usize][0] as i32;
        if ((flags & 1) != 0 && (g == 1 || g == 2))
            || ((flags & 2) != 0 && g == 3)
            || ((flags & 4) != 0 && g == 4)
            || ((flags & 8) != 0 && g == 5)
            || ((flags & 16) != 0 && g == 6)
        {
            lbl_804337C4[i as usize] = -1;
        }
        i += 1;
    }
}

pub unsafe extern "C" fn lbAudioAx_8002702C(flags: u32, mut mask: u64) {
    melee_core::ct!("lbAudioAx_8002702C");
    let mut result: u64 = 0;

    mask = mask.wrapping_add(0x48000000000003);

    if flags & 1 != 0 {
        result = result.wrapping_add(0x48000000000003);
    }
    if flags & 2 != 0 {
        result = result.wrapping_add(0x3C);
    }
    if flags & 4 != 0 {
        result = result.wrapping_add(0x800003FFFFFFC0);
    }
    if flags & 8 != 0 {
        result = result.wrapping_add(0x23FFFC00000000);
    }
    if flags & 16 != 0 {
        result = result.wrapping_add(0x14000000000000);
    }

    for i in 0..(LBAX_CAP - 1) {
        if (result & 1) != 0 && (mask & 1) != 0 {
            lbl_804337C4[i] = 1;
        }
        result >>= 1;
        mask >>= 1;
    }

    // The by-index banks past the u64 mask (see lbAudioAx_80026EBC): requests are consumed.
    let mut i = 64i32;
    while i < LBAX_N!() && (i as usize) < lbAx_stage_pending.len() {
        if lbAx_stage_pending[i as usize] != 0 {
            lbAx_stage_pending[i as usize] = 0;
            lbl_804337C4[i as usize] = 1;
        }
        i += 1;
    }
}

unsafe fn lbAudioAx_80027168_inline() {
    for i in 0..LBAX_N!() as usize {
        if s32_arr_803BB5D0[i][1] != 5 && lbl_80433984[i] == 2 {
            lbl_80433984[i] = 1;
        }
    }
}

unsafe fn lbAudioAx_80027168_inline_2() {
    let slot = fn_80026650();
    if slot != -1 {
        strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, ssm_files[slot as usize]);
        lbl_80433A64[slot as usize] = ext::HSD_SynthSFXLoad(cur_ssm_file.as_ptr() as *const i8, 2, Some(fn_80026C04), 0);
    }
}

pub unsafe extern "C" fn lbAudioAx_80027168() {
    melee_core::ct!("lbAudioAx_80027168");
    lbAudioAx_80027168_inline();

    let mut count = 0;
    for i in 0..LBAX_N!() as usize {
        if s32_arr_803BB5D0[i][1] != 5 && lbl_804337C4[i] != -1 {
            count += 1;
        }
    }

    if count == 0 {
        return;
    }

    fn_800269AC();

    for i in 0..LBAX_N!() as usize {
        lbl_804338A4[i] = lbl_804337C4[i];
    }

    fn_800268B4();
    fn_800267B0();

    if (lbl_804D6438 as u32) < (lbl_804D6448 as u32).wrapping_add(lbl_804D6450) {
        ext::OSReport("******** CAUTION ********\nFGM load size is over\n");
        panic!("lbaudio_ax.c:0xDB3 0");
    }

    lbAudioAx_80027168_inline_2();
}

unsafe fn fn_80027488() -> i32 {
    for i in 0..LBAX_N!() as usize {
        if lbl_804338A4[i] == 1 && lbl_80433984[i] == -1 {
            return 1;
        }
    }

    for i in 0..LBAX_N!() as usize {
        if lbl_804338A4[i] == 1 && lbl_80433984[i] == 1 {
            lbl_80433984[i] = 2;
        }
    }

    0
}

pub unsafe extern "C" fn lbAudioAx_80027648() {
    melee_core::ct!("lbAudioAx_80027648");
    while fn_80027488() == 1 {
        ext::HSD_SynthSFXWaitForLoadCompletion(Some(lb_800195D0_cb));
    }
}

pub unsafe extern "C" fn lbAudioAx_8002785C() {
    melee_core::ct!("lbAudioAx_8002785C");
    let mut result: u64 = 0;

    if ext::gm_8016B184() != 0 {
        result = lbAudioAx_80026E84(ext::Player_GetPlayerCharacter(0));
        for i in 0..3 {
            if ext::gm_80169370(i) != ChKind_Max as i32 {
                let opp = ext::gm_80169370(i);
                result |= lbAudioAx_80026E84(opp);
                if opp == 4 {
                    result |= 0x200004000;
                }
            }
        }
    } else {
        for i in 0..4 {
            if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA as u32 {
                result |= lbAudioAx_80026E84(ext::Player_GetPlayerCharacter(i));
            }
        }
    }

    let stkind = ext::Stage_80225194();
    if stkind == 0xD9 || stkind == 0xE5 {
        result |= 0x200004000;
    }
    if stkind == 0x46 || stkind == 0x47 {
        result |= 0xC00;
    }

    lbl_804D38D8 = lbAudioAx_StageAudioByte(ext::Stage_8022519C(stkind), 1);
    result |= lbAudioAx_80026EBC(stkind);

    // Each player's bank is requested BY INDEX, which reaches the banks past the u64 mask.
    let mut extra = [0i32; 4];
    let mut n_extra = 0usize;
    if LBAX_N!() > 55 {
        for i in 0..4i32 {
            let ssm;
            if ext::gm_8016B184() != 0 {
                let ck = if i == 0 { ext::Player_GetPlayerCharacter(0) } else { ext::gm_80169370(i - 1) };
                ssm = if i == 0 || ck != ChKind_Max as i32 { ext::Mex_SsmForPortCKind(ck) } else { -1 };
            } else {
                ssm = if ext::Player_GetPlayerSlotType(i) != Gm_PKind_NA as u32 {
                    ext::Mex_SsmForPortCKind(ext::Player_GetPlayerCharacter(i))
                } else {
                    -1
                };
            }
            if ssm >= 64 && ssm < LBAX_N!() {
                extra[n_extra] = ssm;
                n_extra += 1;
            }
        }
    }
    if result != 0 || n_extra != 0 {
        lbAudioAx_80026F2C(0xC);
        lbAudioAx_8002702C(0xC, result);
        for i in 0..n_extra {
            lbl_804337C4[extra[i] as usize] = 1;
        }
        lbAudioAx_80027168();
        lbAudioAx_80027648();
    }
}

unsafe fn setup_audio_lang() -> i32 {
    if LBAX_N!() > 55 || ext::lbLang_IsSavedLanguageUS() != 0 {
        strcpy_bytes(&raw mut cur_ssm_file, str_audio_us);
        ssm_stem_pos = 10;
        LANG_US
    } else {
        strcpy_bytes(&raw mut cur_ssm_file, str_audio);
        ssm_stem_pos = 7;
        LANG_JP
    }
}

/// The "load a one-bank file" sequence the language switch repeats.
unsafe fn load_bank(slot: usize, bank: i32) {
    if lbl_80433984[slot] < 1 {
        strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, ssm_files[slot]);
        lbl_80433A64[slot] = ext::HSD_SynthSFXLoad(cur_ssm_file.as_ptr() as *const i8, bank, None, 0);
        ext::HSD_SynthSFXWaitForLoadCompletion(Some(lb_800195D0_cb));
        lbl_80433984[slot] = 2;
    }
}

pub unsafe extern "C" fn lbAudioAx_80027AB0(id: i32) {
    melee_core::ct!("lbAudioAx_80027AB0");
    let lang = setup_audio_lang();

    ext::HSD_AudioSFXKeyOffAll();

    if lbl_804D3878 != lang {
        lbl_804D3878 = lang;
        ext::AXDriver_8038DCFC();
        strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, c"smash2.sem".as_ptr());
        ext::AXDriver_8038DA70(cur_ssm_file.as_ptr() as *const i8, Some(lb_800195D0_cb));

        if id != 0x83D61 {
            lbAudioAx_800237A8(id, VOL_MAX, PAN_MID);
        }

        for i in 0..LBAX_N!() as usize {
            if i != 0 {
                lbl_80433A64[i] = -1;
                lbl_804337C4[i] = -1;
                lbl_804338A4[i] = -1;
                lbl_80433984[i] = -1;
            }
        }

        ext::HSD_SynthSFXUnloadBank(1);
        ext::HSD_SynthSFXUnloadBank(2);

        load_bank(0x33, 1);
        load_bank(1, 1);
        load_bank(0x36, 1);
    }
}

pub unsafe extern "C" fn lbAudioAx_80027DBC() {
    melee_core::ct!("lbAudioAx_80027DBC");
    ext::HSD_AudioSFXKeyOffAll();
    lbAudioAx_800236DC();
    lbAudioAx_80024C84();
    fn_800269AC();
}

unsafe fn lbAudioAx_80027DF8_inline() {
    if lbl_804D640C == 0 {
        for i in 0..16usize {
            if lbl_80433780[i] > 0 {
                lbl_80433780[i] -= 1;
            }
            if lbl_80433780[i] <= 0 {
                lbl_8043373C[i] = 0x83D60;
                lbl_80433780[i] = 0;
            }
        }
    }
}

pub unsafe extern "C" fn lbAudioAx_80027DF8() {
    melee_core::ct!("lbAudioAx_80027DF8");
    if lbl_804D640C == 0 {
        lbl_804D6430 -= 1;
        if lbl_804D6430 <= 0 {
            lbl_804D6420 = 0;
            lbl_804D6430 = 0;
        }
        lbl_804D6434 -= 1;
        if lbl_804D6434 <= 0 {
            lbl_804D6424 = 0;
            lbl_804D6434 = 0;
        }
    }

    if lbl_804D6420 != 0 {
        if lbl_804D38F0 != -1 {
            if ext::HSD_AudioSFXCheck(lbl_804D38F0) == 0 {
                lbl_804D38F0 = fn_80023750(0x84, lbl_804D6428, PAN_MID, 5, 4);
            } else {
                lbAudioAx_80024B58(lbl_804D38F0, lbl_804D6428);
            }
        } else {
            lbl_804D6428 = VOL_MAX;
            lbl_804D38F0 = fn_80023750(0x84, VOL_MAX, PAN_MID, 5, 4);
            lbl_804D642C = 0;
        }
    } else if lbl_804D38F0 != -1 {
        fn_80023750(0x83D60, lbl_804D6428, PAN_MID, 5, 4);
        let mut carry = 0;
        lbl_804D6428 = carry;
        lbl_804D38F0 = -1;
        carry = if lbl_804D38F4 == -1 { carry } else { 0x7F };
        lbl_804D642C = carry;
    }

    if lbl_804D6424 != 0 {
        if lbl_804D38F4 != -1 {
            if ext::HSD_AudioSFXCheck(lbl_804D38F4) == 0 {
                lbl_804D38F4 = fn_80023750(0x85, lbl_804D642C, PAN_MID, 6, 4);
            } else {
                lbAudioAx_80024B58(lbl_804D38F4, lbl_804D642C);
            }
        } else {
            lbl_804D642C = 0x7F;
            lbl_804D38F4 = fn_80023750(0x85, VOL_MAX, PAN_MID, 6, 4);
            lbl_804D6428 = 0;
        }
    } else if lbl_804D38F4 != -1 {
        fn_80023750(0x83D60, lbl_804D642C, PAN_MID, 6, 4);
        let mut carry = 0;
        lbl_804D642C = carry;
        lbl_804D38F4 = -1;
        carry = if lbl_804D38F0 == -1 { carry } else { 0x7F };
        lbl_804D6428 = carry;
    }

    if lbl_804D38F0 == -1 && lbl_804D38F4 == -1 {
        lbl_804D38EC = 1.0;
    } else {
        lbl_804D38EC = 0.2;
    }

    fn_80024654(0);

    if lbl_804D641C == 1 {
        for i in 0..0x10usize {
            if lbl_8043373C[i] != 0x83D60 {
                lbAudioAx_800237A8(lbl_8043373C[i], VOL_MAX, PAN_MID);
                lbl_8043373C[i] = 0x83D60;
                lbl_80433780[i] = 0;
            }
        }
        lbl_804D641C = 0;
        return;
    }

    lbAudioAx_80027DF8_inline();
}

pub unsafe extern "C" fn lbAudioAx_8002835C() {
    melee_core::ct!("lbAudioAx_8002835C");
    ext::HSD_ObjAllocInit(&raw mut lbl_80433710, 0x48, 4);
}

/// The bank tables come from mexData when the disc has it; every table is rewritten. No mexData:
/// the retail tables stay.
unsafe fn lbAudioAx_MexTables() {
    let mut n = ext::Mex_SsmCount();
    if n <= 55 {
        return;
    }
    if n > LBAX_CAP as i32 - 1 {
        ext::OSReport(&format!("lbAudioAx: m-ex has {} sound banks, the port holds {}\n", n, LBAX_CAP - 1));
        n = LBAX_CAP as i32 - 1;
    }
    for i in 0..n {
        let name = ext::Mex_SsmFile(i);
        if !name.is_null() {
            ssm_files[i as usize] = name;
        }
        offsets_arr_803BC4E4[i as usize][0] = ext::Mex_SsmSize(i);
        for k in 0..4 {
            s32_arr_803BB5D0[i as usize][k as usize] = ext::Mex_SsmLookup(i, k) as i8;
        }
        if i >= 55 {
            s32_arr_803BB8D4[i as usize][0] = i * 10000;
            s32_arr_803BB8D4[i as usize][1] = i * 10000 + 9999;
        }
    }
    // the entry past the last bank is the sort's "none" slot, like retail's 55
    offsets_arr_803BC4E4[n as usize][0] = 0;
    for k in 0..4 {
        s32_arr_803BB5D0[n as usize][k] = 0;
    }
    lbAx_N = n;
    ext::OSReport(&format!("lbAudioAx: {} sound banks from mexData\n", n));

    // BGM: m-ex appends its music after retail's 98
    let mut nb = ext::Mex_BgmCount();
    if nb > LBAX_BGM_CAP as i32 {
        nb = LBAX_BGM_CAP as i32;
    }
    let mut i = 0x62;
    while i < nb {
        hps_files[i as usize] = ext::Mex_BgmFile(i);
        i += 1;
    }
    if nb > 0x62 {
        lbAx_BgmN = nb;
    }
}

/// The stage-audio rows come from mexData too (rows 0..70 equal the compiled-in ones).
unsafe fn lbAudioAx_MexStageAudio() {
    let cap = s32_arr_803BB6B0.len() as i32;
    let mut n = ext::Mex_GrAudioCount();

    if n < 0 {
        n = 0;
    }
    if n > cap {
        ext::OSReport(&format!("lbAudioAx: m-ex has {} internal stages, the port's audio table holds {}\n", n, cap));
        n = cap;
    }
    let mut filled = ext::Mex_GrAudioFirst();
    let mut i = filled;
    while i < n {
        let mut ok = 1;
        for k in 0..3 {
            let b = ext::Mex_GrAudioByte(i, k);
            if b < 0 {
                ok = 0;
                break;
            }
            s32_arr_803BB6B0[i as usize][k as usize] = b as u8;
        }
        filled += ok;
        if ok == 0 {
            s32_arr_803BB6B0[i as usize][0] = 55;
            s32_arr_803BB6B0[i as usize][1] = 1;
            s32_arr_803BB6B0[i as usize][2] = 1;
        }
        i += 1;
    }
    let tail = if n > 0x6F { n } else { 0x6F };
    let mut i = tail;
    while i < cap {
        s32_arr_803BB6B0[i as usize][0] = 55;
        s32_arr_803BB6B0[i as usize][1] = 1;
        s32_arr_803BB6B0[i as usize][2] = 1;
        i += 1;
    }
    if n != 0 {
        ext::OSReport(&format!("lbAudioAx: {} stage audio rows from mexData ({} read cleanly)\n", n, filled));
    }
}

static mut ar_stack: [u32; 0x10] = [0; 0x10];
static mut lbl_80433C64: [u8; 53 * 1024] = [0; 53 * 1024];
static mut lbl_80441064: [u8; 71 * 1024] = [0; 71 * 1024];

pub unsafe extern "C" fn lbAudioAx_8002838C() {
    melee_core::ct!("lbAudioAx_8002838C");
    let mut rvb_std: AXFX_REVERBSTD = core::mem::zeroed();
    let mut delay: AXFX_DELAY = core::mem::zeroed();

    ext::ARInit(ar_stack.as_mut_ptr(), ar_stack.len() as u32);
    ext::ARQInit();
    ext::AIInit(null_mut());

    lbAudioAx_MexTables();
    lbAudioAx_MexStageAudio();

    lbl_804D643C = offsets_arr_803BC4E4[0][0] as i32;
    lbl_804D6440 = offsets_arr_803BC4E4[0x33][0] as i32;
    lbl_804D6440 = lbl_804D6440.wrapping_add(offsets_arr_803BC4E4[1][0] as i32);
    lbl_804D6440 = lbl_804D6440.wrapping_add(offsets_arr_803BC4E4[0x36][0] as i32);

    fn_80023254(3);
    lbl_804D6444 = offsets_arr_803BC4E4[lbl_80433B44[0] as usize][0] as i32;

    fn_80023254(4);
    for k in 0..4 {
        lbl_804D6444 = lbl_804D6444.wrapping_add(offsets_arr_803BC4E4[lbl_80433B44[k] as usize][0] as i32);
    }

    fn_80023254(5);
    lbl_804D6444 = lbl_804D6444.wrapping_add(offsets_arr_803BC4E4[lbl_80433B44[0] as usize][0] as i32);

    lbl_804D6438 = lbl_804D643C.wrapping_add(lbl_804D6440).wrapping_add(lbl_804D6444);
    lbl_804D3870 = lbl_804D6438;

    ext::HSD_AudioInitMultiPStream(64, 0, 0x40, lbl_804D3870);

    ext::HSD_AudioSFXGetDefaultAuxParam(AXDRIVER_AUX_REVERB_STD, &raw mut rvb_std as *mut c_void);
    rvb_std.time = 1.88;
    assert!(ext::HSD_AudioGetAuxHeapSize(2, &raw mut rvb_std as *mut c_void) < 53 * 1024, "lbaudio_ax.c:0xF6E");
    ext::HSD_AudioSFXSetupAux(0, 2, &raw mut rvb_std as *mut c_void, lbl_80433C64.as_mut_ptr(), lbl_80433C64.len() as u32);

    ext::HSD_AudioSFXGetDefaultAuxParam(AXDRIVER_AUX_DELAY, &raw mut delay as *mut c_void);
    assert!(ext::HSD_AudioGetAuxHeapSize(2, &raw mut delay as *mut c_void) < 71 * 1024, "lbaudio_ax.c:0xF72");
    ext::HSD_AudioSFXSetupAux(1, 4, &raw mut delay as *mut c_void, lbl_80441064.as_mut_ptr(), lbl_80441064.len() as u32);

    ext::HSD_SynthSFXAllocateBank(lbl_804D643C);
    ext::HSD_SynthSFXAllocateBank(lbl_804D6440);
    ext::HSD_SynthSFXAllocateBank(lbl_804D6444);

    for i in 0..(LBAX_N!() + 1) as usize {
        lbl_804337C4[i] = -1;
        lbl_804338A4[i] = -1;
        lbl_80433984[i] = -1;
        lbl_80433A64[i] = -1;
    }

    lbl_804D3878 = -1;
    lbl_804D6448 = 0;
    lbl_804D644C = 0;
    lbl_804D6450 = 0;
}

pub unsafe extern "C" fn lbAudioAx_80028690() {
    melee_core::ct!("lbAudioAx_80028690");
    lbAudioAx_80024BD0();
    fn_800244F4();
    fn_80024654(1);

    let lang = setup_audio_lang();

    strcpy_bytes(&raw mut cur_hps_file, str_audio);
    hps_stem_pos = 7;

    if lbl_804D3878 == -1 {
        ext::HSD_AudioSFXKeyOffAll();
        ext::HSD_SynthSFXUnloadBank(0);
        load_bank(0, 0);
    }

    if lbl_804D3878 != lang {
        lbl_804D3878 = lang;
        ext::AXDriver_8038DCFC();
        strcpy_at(&raw mut cur_ssm_file, ssm_stem_pos, c"smash2.sem".as_ptr());
        ext::AXDriver_8038DA70(cur_ssm_file.as_ptr() as *const i8, Some(lb_800195D0_cb));

        for i in 0..LBAX_N!() as usize {
            if i != 0 {
                lbl_80433A64[i] = -1;
                lbl_804337C4[i] = -1;
                lbl_804338A4[i] = -1;
                lbl_80433984[i] = -1;
            }
        }

        ext::HSD_SynthSFXUnloadBank(1);
        ext::HSD_SynthSFXUnloadBank(2);

        load_bank(0x33, 1);
        load_bank(1, 1);
        load_bank(0x36, 1);
        load_bank(2, 2);
    }

    for i in 0..=16usize {
        lbl_8043373C[i] = 0x83D60;
        lbl_80433780[i] = 0;
    }

    lbl_804D6414 = 0;
    lbl_804D38E4 = 1.0;
    lbl_804D38E8 = 1.0;
    lbl_804D38EC = 1.0;
    lbl_804D6418 = 0;
    lbl_804D640C = 0;
    lbl_804D6420 = 0;
    lbl_804D6424 = 0;
    lbl_804D6428 = 0;
    lbl_804D642C = 0;
    lbl_804D38F0 = -1;
    lbl_804D38F4 = -1;
    lbl_804D6430 = 0;
    lbl_804D6434 = 0;
    lbl_804D38C4 = 1.0;
    lbl_804D38C8 = 1.0;
}

pub unsafe extern "C" fn lbAudioAx_80028B2C() -> i32 {
    melee_core::ct!("lbAudioAx_80028B2C");
    ext::AXDriver_8038E5D4()
}

pub unsafe extern "C" fn lbAudioAx_80028B4C() -> i32 {
    melee_core::ct!("lbAudioAx_80028B4C");
    ext::AXDriver_8038E5DC()
}

pub unsafe extern "C" fn lbAudioAx_80028B6C() {
    melee_core::ct!("lbAudioAx_80028B6C");
    lbl_804D38CC -= 1;
    if lbl_804D38CC < 0 {
        lbl_804D38CC = 0;
    }
}

pub unsafe extern "C" fn lbAudioAx_80028B90() {
    melee_core::ct!("lbAudioAx_80028B90");
    lbl_804D38CC = 0x7F;
}

/// lbaudio_stage_slots.inc: the index of the current primary BGM in `hps_files`, or -1.
pub unsafe extern "C" fn lbAudioAx_StageSlotMusic() -> i32 {
    melee_core::ct!("lbAudioAx_StageSlotMusic");
    for i in 0..lbAx_BgmN {
        let f = hps_files[i as usize];
        if !f.is_null() && cstr_eq(cur_hps_stem.as_ptr().add(7), f) && cur_hps_stem[..7] == *b"/audio/" {
            return i;
        }
    }
    -1
}
