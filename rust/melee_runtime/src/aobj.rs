//! Port of `src/sysdolphin/baselib/aobj.c`: the animation object (frame counter, loop/rewind
//! logic and the FObj keyframe tracks it drives).
//!
//! Native shape: `HsdAObj<O>` owns its FObj tracks as a `Vec<HsdFObj>` (the C `fobj->next`
//! chain) and its target object as `Option<O>` (`hsd_obj`; the C only ever stores a JObj there).
//! The two file statics `HSD_AObj_804D762C` / `HSD_AObj_804D7630` and `endcallback_list` are
//! [`AObjGlobals`]. Float rule: aobj.c has no contracted site of its own; the one fused site is
//! inside MSL's inline `fmodf` (`a - b*q` -> `fmuladd(-b, q, a)`), ported in [`fmodf`].
//!
//! Not ported: `HSD_ForeachAnim` and its per-type walkers (need JObj/DObj/MObj/PObj/TObj/
//! LObj/CObj/WObj/RObj/Fog, not ported yet); the alloc-data pair (native allocation).
use melee_core::fp::fmadds;

use crate::fobj::{self, HsdFObj, HsdFObjDesc, ObjUpdate};

pub const AOBJ_REWINDED: u32 = 1 << 26;
pub const AOBJ_FIRST_PLAY: u32 = 1 << 27;
pub const AOBJ_NO_UPDATE: u32 = 1 << 28;
pub const AOBJ_LOOP: u32 = 1 << 29;
pub const AOBJ_NO_ANIM: u32 = 1 << 30;

/// Native `HSD_AObj` (GameCube record `melee_types::HSD_AObj`, size 0x1C).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdAObj<O> {
    pub flags: u32,
    pub curr_frame: f32,
    pub rewind_frame: f32,
    pub end_frame: f32,
    pub framerate: f32,
    pub fobj: Vec<HsdFObj>,
    pub hsd_obj: Option<O>,
}

impl<O> melee_core::gc::GcLayout for HsdAObj<O> {
    const GC_NAME: &'static str = <melee_types::HSD_AObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_AObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_AObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_AObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// Native `HSD_AObjDesc` (GameCube record `melee_types::HSD_AObjDesc`, size 0x10).
#[derive(Clone, Debug)]
pub struct HsdAObjDesc {
    pub flags: u32,
    pub end_frame: f32,
    /// The `fobjdesc->next` chain; empty for a NULL `fobjdesc`.
    pub fobjdesc: Vec<HsdFObjDesc>,
    pub obj_id: u32,
}

impl melee_core::gc::GcLayout for HsdAObjDesc {
    const GC_NAME: &'static str = <melee_types::HSD_AObjDesc as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_AObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_AObjDesc as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_AObjDesc as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// aobj.c's file statics.
#[derive(Default)]
pub struct AObjGlobals {
    /// `HSD_AObj_804D762C`: InterpretAnim calls that ended with the AObj stopped.
    pub n_stopped: i32,
    /// `HSD_AObj_804D7630`: InterpretAnim calls that left it running.
    pub n_running: i32,
    /// `endcallback_list` (nothing in the game adds to it; kept for the port's shape).
    pub endcallback_list: Vec<fn()>,
}

impl AObjGlobals {
    /// `HSD_AObjInitEndCallBack`.
    pub fn init_end_callback(&mut self) {
        self.n_stopped = 0;
        self.n_running = 0;
    }
    /// `HSD_AObjInvokeCallBacks`.
    pub fn invoke_callbacks(&self) {
        if self.n_stopped != 0 && self.n_running == 0 {
            for f in &self.endcallback_list {
                f();
            }
        }
    }
    /// `_HSD_AObjForgetMemory`.
    pub fn forget_memory(&mut self) {
        self.endcallback_list.clear();
    }
}

/// MSL `fmodf` (src/MSL/math.h, static inline) as the shipping build compiles it: the
/// quotient goes through `s64`, and `a - b*q` is contracted to `fmuladd(-b, q, a)`.
/// TODO(core): `fptosi float -> i64` is not rewritten by gwtool; Rust's saturating `as i64`
/// equals x86 `cvttss2si` only inside the i64 range and differs for NaN (0 vs i64::MIN).
/// aobj.c only calls it with `b > 0` and finite frames.
pub fn fmodf(a: f32, b: f32) -> f32 {
    if b.abs() > a.abs() {
        return a;
    }
    let q = (a / b) as i64;
    fmadds(-b, q as f32, a)
}

impl<O> HsdAObj<O> {
    /// `HSD_AObjAlloc`.
    pub fn alloc() -> Self {
        HsdAObj {
            flags: AOBJ_NO_ANIM,
            curr_frame: 0.0,
            rewind_frame: 0.0,
            end_frame: 0.0,
            framerate: 1.0,
            fobj: Vec::new(),
            hsd_obj: None,
        }
    }

    /// `HSD_AObjGetFlags` (on a live AObj; the NULL case returns 0).
    pub fn get_flags(&self) -> u32 {
        self.flags
    }
    /// `HSD_AObjSetFlags`: only LOOP and NO_UPDATE can be set from outside.
    pub fn set_flags(&mut self, flags: u32) {
        self.flags |= flags & (AOBJ_LOOP | AOBJ_NO_UPDATE);
    }
    /// `HSD_AObjClearFlags`.
    pub fn clear_flags(&mut self, flags: u32) {
        self.flags &= !(flags & (AOBJ_LOOP | AOBJ_NO_UPDATE));
    }
    /// `HSD_AObjSetFObj` (the old chain is `HSD_FObjRemoveAll`ed, i.e. dropped).
    pub fn set_fobj(&mut self, fobj: Vec<HsdFObj>) {
        self.fobj = fobj;
    }
    /// `HSD_AObjSetRate`.
    pub fn set_rate(&mut self, rate: f32) {
        self.framerate = rate;
    }
    /// `HSD_AObjSetRewindFrame`.
    pub fn set_rewind_frame(&mut self, frame: f32) {
        self.rewind_frame = frame;
    }
    /// `HSD_AObjSetEndFrame`.
    pub fn set_end_frame(&mut self, frame: f32) {
        self.end_frame = frame;
    }

    /// `HSD_AObjReqAnim`.
    pub fn req_anim(&mut self, frame: f32) {
        self.curr_frame = frame;
        self.flags = (self.flags & !AOBJ_NO_ANIM) | AOBJ_FIRST_PLAY;
        fobj::req_anim_all(&mut self.fobj, frame);
    }

    /// `HSD_AObjStopAnim`.
    pub fn stop_anim(&mut self, update: &mut ObjUpdate<'_>) {
        fobj::stop_anim_all(&mut self.fobj, update, self.framerate);
        self.flags |= AOBJ_NO_ANIM;
    }

    /// `HSD_AObjSetCurrentFrame`: ignored while the AObj is stopped.
    pub fn set_current_frame(&mut self, frame: f32) {
        if self.flags & AOBJ_NO_ANIM == 0 {
            self.curr_frame = frame;
            self.flags = (self.flags & 0xBFFF_FFFF) | AOBJ_FIRST_PLAY;
            fobj::req_anim_all(&mut self.fobj, frame);
        }
    }

    /// `HSD_AObjInterpretAnim`: advance one frame, loop or stop at `end_frame`, and run the
    /// tracks (`update` is withheld from the tracks while NO_UPDATE is set).
    pub fn interpret_anim(&mut self, g: &mut AObjGlobals, update: &mut ObjUpdate<'_>) {
        if self.flags & AOBJ_NO_ANIM != 0 {
            return;
        }
        let mut rate;
        if self.flags & AOBJ_FIRST_PLAY != 0 {
            self.flags &= 0xF7FF_FFFF;
            rate = 0.0;
        } else {
            rate = self.framerate;
            self.curr_frame += self.framerate;
        }

        if self.flags & AOBJ_LOOP != 0 && self.end_frame <= self.curr_frame {
            if self.rewind_frame < self.end_frame {
                fobj::stop_anim_all(&mut self.fobj, update, rate);
                let y = self.end_frame - self.rewind_frame;
                let x = self.curr_frame - self.rewind_frame;
                self.curr_frame = fmodf(x, y) + self.rewind_frame;
                fobj::req_anim_all(&mut self.fobj, self.curr_frame);
            } else {
                self.curr_frame = self.end_frame;
            }
            rate = 0.0;
            self.flags |= AOBJ_REWINDED;
        } else {
            self.flags &= 0xFBFF_FFFF;
        }

        if self.flags & AOBJ_NO_UPDATE != 0 {
            fobj::interpret_anim_all(&mut self.fobj, &mut None, rate);
        } else {
            fobj::interpret_anim_all(&mut self.fobj, update, rate);
        }

        if self.flags & AOBJ_LOOP == 0 && self.end_frame <= self.curr_frame {
            fobj::stop_anim_all(&mut self.fobj, update, self.framerate);
            self.flags |= AOBJ_NO_ANIM;
        }

        if self.flags & AOBJ_NO_ANIM != 0 {
            g.n_stopped = g.n_stopped.wrapping_add(1);
        } else {
            g.n_running = g.n_running.wrapping_add(1);
        }
    }

    /// `HSD_AObjLoadDesc`. `resolve_obj` stands in for the `obj_id != 0` path
    /// (`HSD_IDGetDataFromTable` + `ref_INC`, else `HSD_JObjLoadJoint`).
    /// TODO(group1 jobj): JObj is not ported; callers pass the lookup.
    pub fn load_desc(desc: &HsdAObjDesc, resolve_obj: impl FnOnce(u32) -> Option<O>) -> Self {
        let mut a = Self::alloc();
        a.set_flags(desc.flags);
        a.set_rewind_frame(0.0);
        a.set_end_frame(desc.end_frame);
        a.set_fobj(fobj::load_desc_list(&desc.fobjdesc));
        if desc.obj_id != 0 {
            // The old hsd_obj (always None here) would be HSD_JObjUnref'd: dropping O is the unref.
            a.hsd_obj = resolve_obj(desc.obj_id);
        }
        a
    }

    /// `HSD_AObjRemove`: drops the tracks and the object reference (`HSD_JObjUnref`, which is
    /// `O`'s drop; TODO(group1 jobj)).
    pub fn remove(self) {}
}

/// `HSD_AObjGetFlags` with the C's NULL check.
pub fn get_flags<O>(a: Option<&HsdAObj<O>>) -> u32 {
    a.map_or(0, |a| a.flags)
}
