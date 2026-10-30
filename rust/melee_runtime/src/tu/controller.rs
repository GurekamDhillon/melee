//! Port of src/sysdolphin/baselib/controller.c: the HSD pad layer. `PADRead` samples go into a
//! raw queue (`HSD_PadRenewRawStatus`), the master status is derived from the queue with the
//! stick clamp, digital conversion of the sticks, normalisation and cross-direction rules
//! (`HSD_PadRenewMasterStatus`), and the copy / game statuses follow it. The TARGET_PC additions
//! (rollback latch, offline UCF latch, rollback and Lab pad replay, m-ex "X+Y disables Start",
//! fighter-controls remap) are included.
//!
//! Float rule: clang fuses `HSD_PadClampCheck3`'s two `x*x + y*y` and `vec2DSqDist`'s one into
//! `fmadds(x, x, y*y)` (left multiply fused); `sqrtf` is the MSL `gekko::sqrtf`; `atan2f` is
//! lbtrigf's. `(s8) f` goes through gwtool's saturate-then-truncate ([`crate::cabi::f2i8`]).
//! Callees of unported TUs (OS, PAD, rumble, the PC platform hooks) are the hooks in [`ext`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use crate::cabi::f2i8;
use crate::gekko::sqrtf;
use core::ffi::c_void;
use melee_core::fp::fmadds;
use melee_types::{
    HSD_FlushType, HSD_PadData, HSD_PadRumbleListData, HSD_PadStatus, PADStatus, PadLibData,
    HSD_PAD_FLUSH_QUEUE_LEAVE1, HSD_PAD_FLUSH_QUEUE_MERGE, HSD_PAD_FLUSH_QUEUE_THROWAWAY,
};

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
    fn OSDisableInterrupts() -> i32;
    fn OSRestoreInterrupts(level: i32) -> i32;
    fn OSGetResetSwitchState() -> i32;
    fn PADRead(status: *mut PADStatus) -> u32;
    fn PADReset(mask: u32) -> i32;
    fn PADRecalibrate(mask: u32) -> i32;
    fn PADInit() -> i32;
    fn HSD_PadRumbleInterpret() = crate::tu::rumble::HSD_PadRumbleInterpret();
    fn HSD_PadRumbleRemoveAll() = crate::tu::rumble::HSD_PadRumbleRemoveAll();
    fn HSD_PadRumbleOffN(chan: u8) = crate::tu::rumble::HSD_PadRumbleOffN(chan as _);
    fn HSD_PadRumbleInit(nb_list: u16, listdatap: *mut HSD_PadRumbleListData) = crate::tu::rumble::HSD_PadRumbleInit(nb_list, listdatap as _);
    fn RB_PadLatch(port: i32, button: i32, sx: i32, sy: i32, cx: i32, cy: i32, l: i32, r: i32, a: i32, b: i32, err: i32);
    fn Ucf_PadLatch(port: i32, sx: i32, sy: i32, cx: i32, cy: i32);
    fn RB_PadGoverned() -> i32;
    fn RB_PadField(port: i32, which: i32) -> i32;
    fn LabPad_Replay(out: *mut c_void, size: i32) -> i32;
    fn LabPad_Record(st: *const c_void, size: i32);
    fn Mex_Enabled(name: *const i8) -> i32;
    fn FighterControls_Apply(port: i32, buttons: i32) -> i32;
    fn Replay_ControlsRaw() -> i32;
    fn atan2f(y: f32, x: f32) -> f32 = crate::lbtrigf::atan2f(y, x);
    }
}

const PAD_BUTTON_LEFT: u32 = 0x0001;
const PAD_BUTTON_RIGHT: u32 = 0x0002;
const PAD_BUTTON_DOWN: u32 = 0x0004;
const PAD_BUTTON_UP: u32 = 0x0008;
const PAD_BUTTON_X: u32 = 0x0400;
const PAD_BUTTON_Y: u32 = 0x0800;
const PAD_BUTTON_START: u32 = 0x1000;

const PAD_CHAN0_BIT: u32 = 0x8000_0000;
const PAD_CHAN1_BIT: u32 = 0x4000_0000;
const PAD_CHAN2_BIT: u32 = 0x2000_0000;
const PAD_CHAN3_BIT: u32 = 0x1000_0000;

/// `default_status_data`: all zero but `cross_dir = 1`.
pub fn default_status_data() -> HSD_PadStatus {
    let mut s = HSD_PadStatus::zeroed();
    s.cross_dir = 1;
    s
}

/// `default_libinfo_data`.
pub fn default_libinfo_data() -> PadLibData {
    let mut p: PadLibData = unsafe { core::mem::zeroed() };
    p.repeat_start = 0x2D;
    p.repeat_interval = 8;
    p.adc_th = 0x1E;
    p.clamp_stickMax = 0x7F;
    p.clamp_analogLRMax = 0xFF;
    p.clamp_analogABMax = 0xFF;
    p.scale_stick = 0x7F;
    p.scale_analogLR = 0xFF;
    p.scale_analogAB = 0xFF;
    p
}

pub static mut HSD_PadLibData: PadLibData = unsafe { core::mem::zeroed() };
pub static mut HSD_PadMasterStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
pub static mut HSD_PadCopyStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
pub static mut HSD_PadGameStatus: [HSD_PadStatus; 4] = unsafe { core::mem::zeroed() };
pub const pad_bit: [u32; 4] = [PAD_CHAN0_BIT, PAD_CHAN1_BIT, PAD_CHAN2_BIT, PAD_CHAN3_BIT];

/// `HSD_PadGetRawQueueCount`.
pub fn HSD_PadGetRawQueueCount() -> u8 {
    unsafe {
        let intr = ext::OSDisableInterrupts();
        let queue_count = HSD_PadLibData.qcount;
        ext::OSRestoreInterrupts(intr);
        queue_count
    }
}

/// `HSD_PadGetResetSwitch`.
pub fn HSD_PadGetResetSwitch() -> i32 {
    unsafe { (HSD_PadLibData.reset_switch != 0) as i32 }
}

/// `HSD_PadRawQueueShift`.
fn HSD_PadRawQueueShift(qnum: u8, qptr: &mut u8) {
    *qptr = ((*qptr as i32 + 1) % qnum as i32) as u8;
}

/// `HSD_PadRawMerge`.
unsafe fn HSD_PadRawMerge(src1: *const PADStatus, src2: *const PADStatus, dst: *mut PADStatus) {
    for i in 0..4 {
        (*dst.add(i)).button = (*src1.add(i)).button | (*src2.add(i)).button;
    }
}

/// `&p->queue->stat[idx * 4]` (the C indexes the 4-entry `stat` array in steps of whole samples).
unsafe fn queue_stat(p: *mut PadLibData, idx: u8) -> *mut PADStatus {
    ((*(*p).queue).stat.as_mut_ptr()).add(idx as usize * 4)
}

/// `HSD_PadRenewRawStatus`.
pub unsafe fn HSD_PadRenewRawStatus(err_check: bool) {
    let p: *mut PadLibData = &mut HSD_PadLibData;
    let mut now = HSD_PadData::zeroed();

    ext::HSD_PadRumbleInterpret();
    ext::PADRead(now.stat.as_mut_ptr());
    for i in 0..4 {
        let s = &now.stat[i];
        ext::RB_PadLatch(
            i as i32, s.button as i32, s.stickX as i32, s.stickY as i32, s.substickX as i32,
            s.substickY as i32, s.triggerLeft as i32, s.triggerRight as i32, s.analogA as i32,
            s.analogB as i32, s.err as i32,
        );
    }
    for i in 0..4 {
        let s = &now.stat[i];
        ext::Ucf_PadLatch(i as i32, s.stickX as i32, s.stickY as i32, s.substickX as i32, s.substickY as i32);
    }
    if err_check {
        let mut i = 0;
        while i < 4 {
            if now.stat[i].err == 0 {
                break;
            }
            i += 1;
        }
        if i == 4 {
            return;
        }
    }

    let qwrite: *mut HSD_PadData = (*p).queue.add((*p).qwrite as usize);
    let mut skip = false;
    if (*p).qcount == (*p).qnum {
        match (*p).qtype {
            0 => {
                HSD_PadRawQueueShift((*p).qnum, &mut (*p).qread);
                let qread = (*(*p).queue.add((*p).qread as usize)).stat.as_mut_ptr();
                if (*p).qnum != 1 {
                    HSD_PadRawMerge((*qwrite).stat.as_ptr(), qread, qread);
                } else {
                    let nowp = now.stat.as_mut_ptr();
                    HSD_PadRawMerge(nowp, qread, nowp);
                }
            }
            1 => HSD_PadRawQueueShift((*p).qnum, &mut (*p).qread),
            2 => skip = true,
            _ => {}
        }
    } else {
        (*p).qcount += 1;
    }

    if !skip {
        *qwrite = now;
        HSD_PadRawQueueShift((*p).qnum, &mut (*p).qwrite);
    }

    let mut mask = 0u32;
    for i in 0..4 {
        if now.stat[i].err == -1 {
            mask |= pad_bit[i];
        }
    }
    if mask != 0 {
        ext::PADReset(mask);
    }
    if ext::OSGetResetSwitchState() != 0 {
        (*p).reset_switch_status = 1;
    } else if (*p).reset_switch_status != 0 {
        (*p).reset_switch = 1;
        (*p).reset_switch_status = 0;
    }
}

/// `HSD_PadFlushQueue`.
pub unsafe fn HSD_PadFlushQueue(ftype: HSD_FlushType) {
    let p: *mut PadLibData = &mut HSD_PadLibData;
    let intr = ext::OSDisableInterrupts();
    match ftype {
        HSD_PAD_FLUSH_QUEUE_MERGE => {
            while (*p).qcount > 1 {
                let qread = queue_stat(p, (*p).qread);
                HSD_PadRawQueueShift((*p).qnum, &mut (*p).qread);
                let qdst = queue_stat(p, (*p).qread);
                HSD_PadRawMerge(qread, qdst, qdst);
                (*p).qcount -= 1;
            }
        }
        HSD_PAD_FLUSH_QUEUE_THROWAWAY => {
            (*p).qread = (*p).qwrite;
            (*p).qcount = 0;
        }
        HSD_PAD_FLUSH_QUEUE_LEAVE1 => {
            if (*p).qcount > 1 {
                (*p).qread = if (*p).qwrite != 0 { (*p).qwrite - 1 } else { (*p).qnum - 1 };
                (*p).qcount = 1;
            }
        }
        _ => {}
    }
    ext::OSRestoreInterrupts(intr);
}

/// `HSD_PadClampCheck1`.
fn HSD_PadClampCheck1(val: &mut u8, shift: u8, min: u8, max: u8) {
    if *val < min {
        *val = 0;
        return;
    }
    if *val > max {
        *val = max;
    }
    if shift != 1 {
        return;
    }
    *val = val.wrapping_sub(min);
}

/// `HSD_PadClampCheck3`.
fn HSD_PadClampCheck3(x: &mut i8, y: &mut i8, shift: u8, min: i8, max: i8) {
    let (fx, fy) = (*x as f32, *y as f32);
    let mut r = sqrtf(fmadds(fx, fx, fy * fy));

    if r < min as f32 {
        *y = 0;
        *x = 0;
        return;
    }
    if r > max as f32 {
        *x = f2i8((((*x as f32) * (max as f32)) / r) as f64);
        *y = f2i8((((*y as f32) * (max as f32)) / r) as f64);
        let (fx, fy) = (*x as f32, *y as f32);
        r = sqrtf(fmadds(fx, fx, fy * fy));
    }

    if shift == 1 && r > 1.000000013351432e-10f32 {
        *x = f2i8(((*x as f32) - (((*x as f32) * (min as f32)) / r)) as f64);
        *y = f2i8(((*y as f32) - (((*y as f32) * (min as f32)) / r)) as f64);
    }
}

/// `HSD_PadClamp`.
unsafe fn HSD_PadClamp(mp: &mut HSD_PadStatus) {
    let p = &HSD_PadLibData;
    if p.clamp_stickType == 0 {
        HSD_PadClampCheck3(&mut mp.stickX, &mut mp.stickY, p.clamp_stickShift, p.clamp_stickMin, p.clamp_stickMax);
        HSD_PadClampCheck3(&mut mp.subStickX, &mut mp.subStickY, p.clamp_stickShift, p.clamp_stickMin, p.clamp_stickMax);
    }
    HSD_PadClampCheck1(&mut mp.analogL, p.clamp_analogLRShift, p.clamp_analogLRMin, p.clamp_analogLRMax);
    HSD_PadClampCheck1(&mut mp.analogR, p.clamp_analogLRShift, p.clamp_analogLRMin, p.clamp_analogLRMax);
    HSD_PadClampCheck1(&mut mp.analogA, p.clamp_analogABShift, p.clamp_analogABMin, p.clamp_analogABMax);
    HSD_PadClampCheck1(&mut mp.analogB, p.clamp_analogABShift, p.clamp_analogABMin, p.clamp_analogABMax);
}

/// `sq` (static inline).
#[inline]
pub fn sq(x: f32) -> f32 {
    x * x
}

/// `vec2DSqDist` (static inline): one fused site.
#[inline]
fn vec2DSqDist(x: f32, y: f32) -> f32 {
    fmadds(x, x, y * y)
}

/// `vec2Dlen` (static inline).
#[inline]
fn vec2Dlen(x: i8, y: i8) -> f32 {
    sqrtf(vec2DSqDist(x as f32, y as f32))
}

/// `HSD_PadADConvertCheck1`.
unsafe fn HSD_PadADConvertCheck1(mp: &mut HSD_PadStatus, x: i8, y: i8, up: u32, down: u32, left: u32, right: u32) {
    let p = &HSD_PadLibData;
    let _ = sq(x as f32);
    let r = vec2Dlen(x, y);

    let a: f32 = if (x as f32).abs() == 0.0 {
        (if y >= 0 { 1.5707963267948966f64 } else { -1.5707963267948966f64 }) as f32
    } else {
        ext::atan2f(y as f32, x as f32)
    };

    let ha = 0.5f32 * p.adc_angle;
    if r < p.adc_th as f32 {
        return;
    }
    let (a, ha) = (a as f64, ha as f64);
    if a < -2.356194490192345 + ha {
        mp.button |= left;
    }
    if a >= -2.356194490192345 - ha && a <= -0.7853981633974483 + ha {
        mp.button |= down;
    }
    if a > -0.7853981633974483 - ha && a < 0.7853981633974483 + ha {
        mp.button |= right;
    }
    if a >= 0.7853981633974483 - ha && a <= 2.356194490192345 + ha {
        mp.button |= up;
    }
    if a > 2.356194490192345 - ha {
        mp.button |= left;
    }
}

/// `HSD_PadADConvert`.
unsafe fn HSD_PadADConvert(mp: &mut HSD_PadStatus) {
    if HSD_PadLibData.adc_type == 0 {
        let (sx, sy, cx, cy) = (mp.stickX, mp.stickY, mp.subStickX, mp.subStickY);
        HSD_PadADConvertCheck1(mp, sx, sy, 0x10000, 0x20000, 0x40000, 0x80000);
        HSD_PadADConvertCheck1(mp, cx, cy, 0x100000, 0x200000, 0x400000, 0x800000);
    }
}

/// `HSD_PadScale`.
unsafe fn HSD_PadScale(mp: &mut HSD_PadStatus) {
    let p = &HSD_PadLibData;
    mp.nml_stickX = mp.stickX as f32 / p.scale_stick as f32;
    mp.nml_stickY = mp.stickY as f32 / p.scale_stick as f32;
    mp.nml_subStickX = mp.subStickX as f32 / p.scale_stick as f32;
    mp.nml_subStickY = mp.subStickY as f32 / p.scale_stick as f32;
    mp.nml_analogL = mp.analogL as f32 / p.scale_analogLR as f32;
    mp.nml_analogR = mp.analogR as f32 / p.scale_analogLR as f32;
    mp.nml_analogA = mp.analogA as f32 / p.scale_analogAB as f32;
    mp.nml_analogB = mp.analogB as f32 / p.scale_analogAB as f32;
}

/// `HSD_PadCrossDir`.
unsafe fn HSD_PadCrossDir(mp: &mut HSD_PadStatus) {
    let ud = PAD_BUTTON_DOWN | PAD_BUTTON_UP;
    let lr = PAD_BUTTON_LEFT | PAD_BUTTON_RIGHT;
    match HSD_PadLibData.cross_dir {
        1 => {
            if mp.button & ud == 0 {
                return;
            }
            mp.button &= !lr;
        }
        2 => {
            if mp.button & lr == 0 {
                return;
            }
            mp.button &= !ud;
        }
        3 => {
            if mp.button & ud != 0 {
                if mp.button & lr != 0 {
                    if mp.cross_dir == 1 {
                        mp.button &= !lr;
                        return;
                    }
                    mp.button &= !ud;
                } else {
                    mp.cross_dir = 1;
                }
                return;
            }
            if mp.button & lr != 0 {
                mp.cross_dir = 2;
            }
        }
        _ => {}
    }
}

/// `HSD_PadRenewMasterStatus` (TARGET_PC build).
pub unsafe fn HSD_PadRenewMasterStatus() {
    let p: *mut PadLibData = &mut HSD_PadLibData;
    let mut rbpad: [PADStatus; 4] = core::mem::zeroed();
    let mut labpad: [PADStatus; 4] = core::mem::zeroed();
    let mut lab_have = false;

    // Rollback session with raw-controller sources: statuses come from the session's ring.
    let rb_pads = ext::RB_PadGoverned() != 0;
    if rb_pads {
        for i in 0..4 {
            let f = |w| ext::RB_PadField(i as i32, w);
            rbpad[i].button = f(0) as u16;
            rbpad[i].stickX = f(1) as i8;
            rbpad[i].stickY = f(2) as i8;
            rbpad[i].substickX = f(3) as i8;
            rbpad[i].substickY = f(4) as i8;
            rbpad[i].triggerLeft = f(5) as u8;
            rbpad[i].triggerRight = f(6) as u8;
            rbpad[i].analogA = f(7) as u8;
            rbpad[i].analogB = f(8) as u8;
            rbpad[i].err = f(9) as i8;
        }
    }
    // The Geno Lab's rewind: re-simulated frames take the statuses logged when they first ran.
    let mut lab_pads = 0;
    if !rb_pads {
        lab_pads = ext::LabPad_Replay(labpad.as_mut_ptr() as *mut c_void, core::mem::size_of_val(&labpad) as i32);
    }

    let mut mp: *mut HSD_PadStatus = HSD_PadMasterStatus.as_mut_ptr();
    let intr = ext::OSDisableInterrupts();
    let mut qread: *mut PADStatus = core::ptr::null_mut();
    if (*p).qcount != 0 {
        qread = queue_stat(p, (*p).qread);
        HSD_PadRawQueueShift((*p).qnum, &mut (*p).qread);
        (*p).qcount -= 1;
        lab_have = true;
    }
    if rb_pads {
        qread = rbpad.as_mut_ptr();
        lab_have = true;
    }
    if lab_pads == 1 {
        qread = labpad.as_mut_ptr();
        lab_have = true;
    } else if lab_pads == 2 {
        lab_have = false;
    }
    ext::LabPad_Record(
        if lab_have { qread as *const c_void } else { core::ptr::null() },
        core::mem::size_of_val(&labpad) as i32,
    );
    if lab_have {
        for i in 0..4 {
            let m = &mut *mp;
            let q = &*qread;
            m.last_button = m.button;
            m.err = q.err;
            if m.err == 0 {
                // m-ex xy_disables_start: X+Y+Start clears the Start bit before it is latched.
                let b = q.button as u32;
                if ext::Mex_Enabled(c"xy_disables_start".as_ptr()) != 0
                    && (b & (PAD_BUTTON_START | PAD_BUTTON_X | PAD_BUTTON_Y)) == (PAD_BUTTON_START | PAD_BUTTON_X | PAD_BUTTON_Y)
                {
                    m.button = b & !PAD_BUTTON_START;
                } else {
                    m.button = b;
                }
                // Raw rollback/LAB samples stay raw in their logs: map once here.
                if ext::Replay_ControlsRaw() != 0 {
                    m.button = ext::FighterControls_Apply(i as i32, m.button as i32) as u32;
                }
                m.stickX = q.stickX;
                m.stickY = q.stickY;
                m.subStickX = q.substickX;
                m.subStickY = q.substickY;
                m.analogL = q.triggerLeft;
                m.analogR = q.triggerRight;
                m.analogA = q.analogA;
                m.analogB = q.analogB;
                HSD_PadClamp(m);
                HSD_PadADConvert(m);
                HSD_PadScale(m);
                HSD_PadCrossDir(m);
            } else if m.err == -3 {
                m.err = 0;
            } else {
                m.button = 0;
                m.subStickY = 0;
                m.subStickX = 0;
                m.stickY = 0;
                m.stickX = 0;
                m.analogB = 0;
                m.analogA = 0;
                m.analogR = 0;
                m.analogL = 0;
                m.nml_subStickY = 0.0;
                m.nml_subStickX = 0.0;
                m.nml_stickY = 0.0;
                m.nml_stickX = 0.0;
                m.nml_analogB = 0.0;
                m.nml_analogA = 0.0;
                m.nml_analogR = 0.0;
                m.nml_analogL = 0.0;
            }
            m.trigger = m.button & (m.last_button ^ m.button);
            m.release = m.last_button & (m.last_button ^ m.button);
            update_repeat(m, &*p);
            mp = mp.add(1);
            qread = qread.add(1);
        }
    }
    ext::OSRestoreInterrupts(intr);
}

/// The repeat bookkeeping shared by the master / copy / game status updates.
#[inline]
fn update_repeat(s: &mut HSD_PadStatus, p: &PadLibData) {
    if s.last_button ^ s.button != 0 {
        s.repeat = s.trigger;
        s.repeat_count = p.repeat_start;
    } else {
        let v = s.repeat_count.wrapping_sub(1);
        s.repeat_count = v;
        if v != 0 {
            s.repeat = 0;
        } else {
            s.repeat = s.button;
            s.repeat_count = p.repeat_interval;
        }
    }
}

/// `HSD_PadCopyStatusFields` (static inline).
fn HSD_PadCopyStatusFields(dst: &mut HSD_PadStatus, src: &HSD_PadStatus) {
    dst.button = src.button;
    dst.stickX = src.stickX;
    dst.stickY = src.stickY;
    dst.subStickX = src.subStickX;
    dst.subStickY = src.subStickY;
    dst.analogL = src.analogL;
    dst.analogR = src.analogR;
    dst.analogA = src.analogA;
    dst.analogB = src.analogB;
    dst.nml_stickX = src.nml_stickX;
    dst.nml_stickY = src.nml_stickY;
    dst.nml_subStickX = src.nml_subStickX;
    dst.nml_subStickY = src.nml_subStickY;
    dst.nml_analogL = src.nml_analogL;
    dst.nml_analogR = src.nml_analogR;
    dst.nml_analogA = src.nml_analogA;
    dst.nml_analogB = src.nml_analogB;
}

/// `HSD_PadClearStatusFields` (static inline).
fn HSD_PadClearStatusFields(dst: &mut HSD_PadStatus) {
    dst.button = 0;
    dst.subStickY = 0;
    dst.subStickX = 0;
    dst.stickY = 0;
    dst.stickX = 0;
    dst.analogB = 0;
    dst.analogA = 0;
    dst.analogR = 0;
    dst.analogL = 0;
    dst.nml_subStickY = 0.0;
    dst.nml_subStickX = 0.0;
    dst.nml_stickY = 0.0;
    dst.nml_stickX = 0.0;
    dst.nml_analogB = 0.0;
    dst.nml_analogA = 0.0;
    dst.nml_analogR = 0.0;
    dst.nml_analogL = 0.0;
}

unsafe fn renew_from_master(dst: &mut [HSD_PadStatus; 4]) {
    let p = &HSD_PadLibData;
    for i in 0..4 {
        let mp = &HSD_PadMasterStatus[i];
        let d = &mut dst[i];
        d.last_button = d.button;
        d.err = mp.err;
        if d.err == 0 {
            HSD_PadCopyStatusFields(d, mp);
        } else {
            HSD_PadClearStatusFields(d);
        }
        d.trigger = d.button & (d.last_button ^ d.button);
        d.release = d.last_button & (d.last_button ^ d.button);
        update_repeat(d, p);
    }
}

/// `HSD_PadRenewCopyStatus`.
pub unsafe fn HSD_PadRenewCopyStatus() {
    renew_from_master(&mut HSD_PadCopyStatus);
}

/// `HSD_PadRenewGameStatus`.
pub unsafe fn HSD_PadRenewGameStatus() {
    renew_from_master(&mut HSD_PadGameStatus);
}

/// `HSD_PadRenewStatus`.
pub unsafe fn HSD_PadRenewStatus() {
    HSD_PadRenewRawStatus(false);
    HSD_PadRenewMasterStatus();
    HSD_PadRenewCopyStatus();
    HSD_PadRenewGameStatus();
}

/// `HSD_PadReset`.
pub unsafe fn HSD_PadReset() {
    let p: *mut PadLibData = &mut HSD_PadLibData;
    let intr = ext::OSDisableInterrupts();

    ext::HSD_PadRumbleRemoveAll();
    for i in 0..4 {
        ext::HSD_PadRumbleOffN(i);
    }
    HSD_PadFlushQueue(HSD_PAD_FLUSH_QUEUE_THROWAWAY);
    ext::PADRecalibrate(0xF000_0000);
    (*p).reset_switch = 0;

    ext::OSRestoreInterrupts(intr);
}

/// `HSD_PadInit`.
pub unsafe fn HSD_PadInit(qnum: u8, queue: *mut HSD_PadData, nb_list: u16, listdatap: *mut HSD_PadRumbleListData) {
    let p: *mut PadLibData = &mut HSD_PadLibData;
    *p = default_libinfo_data();
    (*p).qnum = qnum;
    (*p).queue = queue;
    ext::HSD_PadRumbleInit(nb_list, listdatap);
    for i in 0..4 {
        HSD_PadMasterStatus[i] = default_status_data();
        HSD_PadCopyStatus[i] = default_status_data();
        HSD_PadGameStatus[i] = default_status_data();
    }
    ext::PADInit();
}
