//! `src/sysdolphin/baselib/fobj.c`: the HSD keyframe ("FObj") interpreter that drives every
//! AObj track (joint, material, texture, light, camera animation).
//!
//! Native shape: the C `HSD_FObj` linked list (`next`) is a `Vec<HsdFObj>` / slice; `ad_head`
//! is a shared byte buffer (`Arc<[u8]>`) and `ad` an offset into it. A byte read past the end
//! of the buffer yields 0 (the C reads whatever follows in memory; a DAT loader that gives the
//! full anim-data region makes the two agree). The `HSD_ObjUpdateFunc` callback becomes
//! `FnMut(obj_type, value)`: the C passes a zero-initialised `HSD_ObjData` whose only written
//! member is `fv`, and discards any write the callee makes, so the f32 is the whole contract
//! (an int-typed track reads the same bits via `f32::to_bits`).
//!
//! Fused sites (`fma_sites.py`): one, `d0 * time + p0` in FObjUpdateAnim's linear case.
//! Not ported: alloc/free/remove and the alloc-data accessors (objalloc pool; native `Vec`
//! ownership replaces them). Divergence: the C interpreter loops forever if the state nibble
//! is 7..15 (only reachable through an external `HSD_FObjSetState`); this port returns.
use std::sync::Arc;

use melee_core::fp::fmadds;

use crate::cabi::f2i16;
use crate::spline::get_helmite;

pub const HSD_A_OP_NONE: u8 = 0;
pub const HSD_A_OP_CON: u8 = 1;
pub const HSD_A_OP_LIN: u8 = 2;
pub const HSD_A_OP_SPL0: u8 = 3;
pub const HSD_A_OP_SPL: u8 = 4;
pub const HSD_A_OP_SLP: u8 = 5;
pub const HSD_A_OP_KEY: u8 = 6;

pub const HSD_A_FRAC_FLOAT: u8 = 0 << 5;
pub const HSD_A_FRAC_S16: u8 = 1 << 5;
pub const HSD_A_FRAC_U16: u8 = 2 << 5;
pub const HSD_A_FRAC_S8: u8 = 3 << 5;
pub const HSD_A_FRAC_U8: u8 = 4 << 5;

/// Native `HSD_FObj` (GameCube record: `melee_types::HSD_FObj`, size 0x30; see `GcLayout` impl).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdFObj {
    /// `ad` as an offset from `ad_head`.
    pub ad: usize,
    pub ad_head: Arc<[u8]>,
    pub length: u32,
    pub flags: u8,
    pub op: u8,
    pub op_intrp: u8,
    pub obj_type: u8,
    pub frac_value: u8,
    pub frac_slope: u8,
    pub nb_pack: u16,
    pub startframe: i16,
    pub fterm: u16,
    pub time: f32,
    pub p0: f32,
    pub p1: f32,
    pub d0: f32,
    pub d1: f32,
}

impl melee_core::gc::GcLayout for HsdFObj {
    const GC_NAME: &'static str = <melee_types::HSD_FObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_FObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_FObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_FObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// Native `HSD_FObjDesc` (GameCube record `melee_types::HSD_FObjDesc`, size 0x14).
#[derive(Clone, Debug)]
pub struct HsdFObjDesc {
    pub length: u32,
    pub startframe: f32,
    pub type_: u8,
    pub frac_value: u8,
    pub frac_slope: u8,
    pub ad: Arc<[u8]>,
}

impl melee_core::gc::GcLayout for HsdFObjDesc {
    const GC_NAME: &'static str = <melee_types::HSD_FObjDesc as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_FObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_FObjDesc as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_FObjDesc as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// `HSD_ObjUpdateFunc` minus the `obj` pointer (captured by the closure): `(obj_type, fv)`.
pub type ObjUpdate<'a> = Option<&'a mut dyn FnMut(u8, f32)>;

impl HsdFObj {
    /// `HSD_FObjLoadDesc` for one desc (the `next` chain is [`load_desc_list`]).
    pub fn load_desc(desc: &HsdFObjDesc) -> Self {
        HsdFObj {
            ad: 0,
            ad_head: desc.ad.clone(),
            length: desc.length,
            flags: 0,
            op: 0,
            op_intrp: 0,
            obj_type: desc.type_,
            frac_value: desc.frac_value,
            frac_slope: desc.frac_slope,
            nb_pack: 0,
            startframe: f2i16(desc.startframe as f64),
            fterm: 0,
            time: 0.0,
            p0: 0.0,
            p1: 0.0,
            d0: 0.0,
            d1: 0.0,
        }
    }

    #[inline]
    fn rd(&mut self) -> u8 {
        let b = self.ad_head.get(self.ad).copied().unwrap_or(0);
        self.ad += 1;
        b
    }
    #[inline]
    fn peek(&self, i: usize) -> u8 {
        self.ad_head.get(self.ad + i).copied().unwrap_or(0)
    }

    /// `HSD_FObjSetState`.
    pub fn set_state(&mut self, state: u32) -> u32 {
        self.flags = ((state & 0xF) as u8) | (self.flags & 0xF0);
        state
    }
    /// `HSD_FObjGetState`.
    pub fn get_state(&self) -> u32 {
        (self.flags & 0xF) as u32
    }

    /// `HSD_FObjReqAnim` (static inline).
    pub fn req_anim(&mut self, startframe: f32) {
        self.ad = 0;
        self.time = self.startframe as f32 + startframe;
        self.op = 0;
        self.op_intrp = 0;
        self.flags &= !0x40;
        self.nb_pack = 0;
        self.fterm = 0;
        self.p0 = 0.0;
        self.p1 = 0.0;
        self.d0 = 0.0;
        self.d1 = 0.0;
        self.set_state(1);
    }

    /// `parseFloat` (static).
    fn parse_float(&mut self, frac: u8) -> f32 {
        if frac == HSD_A_FRAC_FLOAT {
            let mut d = self.rd() as u32;
            d |= (self.rd() as u32) << 8;
            d |= (self.rd() as u32) << 16;
            d |= (self.rd() as u32) << 24;
            return f32::from_bits(d);
        }
        let denom: i32 = <i32 as melee_core::ppc::Ppc>::pshl(1i32, ((frac & 0x1F)) as i64);
        let numer: f32 = match frac & 0xE0 {
            HSD_A_FRAC_S8 => {
                let n = self.peek(0) as i8 as f32;
                self.ad += 1;
                n
            }
            HSD_A_FRAC_U8 => {
                let n = self.peek(0) as f32;
                self.ad += 1;
                n
            }
            HSD_A_FRAC_S16 => {
                let n = (((self.peek(1) as i8 as i32) << 8) | self.peek(0) as i32) as f32;
                self.ad += 2;
                n
            }
            HSD_A_FRAC_U16 => {
                let n = (((self.peek(1) as i32) << 8) | self.peek(0) as i32) as f32;
                self.ad += 2;
                n
            }
            _ => return 0.0,
        };
        numer / denom as f32
    }

    /// `parseOpCode` (static).
    fn parse_op_code(&self) -> u8 {
        self.peek(0) & 0xF
    }

    /// `parsePackInfo` (static).
    fn parse_pack_info(&mut self) -> u32 {
        let mut d = self.rd();
        let mut nb_pack = (((d >> 4) & 7) + 1) as u32;
        let mut shift: u32 = 3;
        if d & 0x80 == 0 {
            return nb_pack;
        }
        loop {
            d = self.rd();
            nb_pack = nb_pack.wrapping_add(crate::cabi::shl((d & 0x7F) as u32, shift));
            shift += 7;
            if d & 0x80 == 0 {
                return nb_pack;
            }
        }
    }

    /// `FObjLaunchKeyData` (static).
    fn launch_key_data(&mut self) {
        if self.flags & 0x40 != 0 {
            self.op_intrp = self.op;
            self.flags &= !0x40;
            self.flags |= 0x80;
            self.p0 = self.p1;
        }
    }

    /// `parseWait` (static).
    fn parse_wait(&mut self) -> i32 {
        let mut wait: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let d = self.rd();
            wait |= crate::cabi::shl((d & 0x7F) as u32, shift);
            shift += 7;
            if d & 0x80 == 0 {
                return wait as i32;
            }
        }
    }

    /// `FObjLoadWait` (static).
    fn load_wait(&mut self) -> u32 {
        if self.ad as u32 >= self.length {
            6
        } else {
            self.fterm = self.parse_wait() as u16;
            self.flags |= 0x20;
            self.set_state(2)
        }
    }

    fn next_state(&mut self, st: u32) -> u32 {
        self.set_state(if st == 1 { 3 } else { 4 })
    }

    /// `FObjAnimCON` and `FObjAnimLinear` (identical bodies in the C).
    fn anim_con_lin(&mut self) -> u32 {
        let st = self.get_state();
        self.p0 = self.p1;
        self.p1 = self.parse_float(self.frac_value);
        if self.op_intrp != 5 {
            self.d0 = self.d1;
            self.d1 = 0.0;
        }
        self.next_state(st)
    }
    /// `FObjAnimSPL0`.
    fn anim_spl0(&mut self) -> u32 {
        let st = self.get_state();
        self.p0 = self.p1;
        self.d0 = self.d1;
        self.p1 = self.parse_float(self.frac_value);
        self.d1 = 0.0;
        self.next_state(st)
    }
    /// `FObjAnimSPL`.
    fn anim_spl(&mut self) -> u32 {
        let st = self.get_state();
        self.p0 = self.p1;
        self.p1 = self.parse_float(self.frac_value);
        self.d0 = self.d1;
        self.d1 = self.parse_float(self.frac_slope);
        self.next_state(st)
    }
    /// `FObjAnimSLP`.
    fn anim_slp(&mut self) -> u32 {
        self.d0 = self.d1;
        self.d1 = self.parse_float(self.frac_slope);
        self.get_state()
    }
    /// `FObjAnimKey`.
    fn anim_key(&mut self) -> u32 {
        let st = self.get_state();
        self.launch_key_data();
        self.p1 = self.parse_float(self.frac_value);
        self.flags |= 0x40;
        self.next_state(st)
    }

    /// `FObjLoadData` (static inline).
    fn load_data(&mut self) -> u32 {
        if self.ad as u32 >= self.length {
            return 6;
        }
        self.op_intrp = self.op;
        if self.nb_pack == 0 {
            self.op = self.parse_op_code();
            self.nb_pack = self.parse_pack_info() as u16;
        }
        self.nb_pack = self.nb_pack.wrapping_sub(1);
        match self.op {
            HSD_A_OP_CON | HSD_A_OP_LIN => self.anim_con_lin(),
            HSD_A_OP_SPL0 => self.anim_spl0(),
            HSD_A_OP_SPL => self.anim_spl(),
            HSD_A_OP_SLP => self.anim_slp(),
            HSD_A_OP_KEY => self.anim_key(),
            _ => 0,
        }
    }

    /// `FObjUpdateAnim`.
    pub fn update_anim(&mut self, obj_update: &mut ObjUpdate<'_>) {
        let Some(cb) = obj_update.as_mut() else { return };
        let mut fv: f32 = 0.0;
        match self.op_intrp {
            HSD_A_OP_KEY => {
                if self.flags & 0x80 != 0 {
                    fv = self.p0;
                    self.flags &= 0x7F;
                } else {
                    return;
                }
            }
            HSD_A_OP_CON => fv = if self.time >= self.fterm as f32 { self.p1 } else { self.p0 },
            HSD_A_OP_LIN => {
                if self.flags & 0x20 != 0 {
                    self.flags &= 0xDF;
                    if self.fterm != 0 {
                        self.d0 = (self.p1 - self.p0) / self.fterm as f32;
                    } else {
                        self.d0 = 0.0;
                        self.p0 = self.p1;
                    }
                }
                fv = fmadds(self.d0, self.time, self.p0);
            }
            HSD_A_OP_SPL0 | HSD_A_OP_SPL | HSD_A_OP_SLP => {
                fv = if self.fterm != 0 {
                    get_helmite((1.0f64 / self.fterm as f64) as f32, self.time, self.p0, self.p1, self.d0, self.d1)
                } else {
                    self.p1
                };
            }
            _ => {}
        }
        cb(self.obj_type, fv);
    }

    /// `HSD_FObjInterpretAnim`.
    pub fn interpret_anim(&mut self, obj_update: &mut ObjUpdate<'_>, rate: f32) {
        let mut fterm: f32 = 0.0;
        let mut state = self.get_state();
        if state == 0 {
            return;
        }
        self.time += rate;
        if self.time < 0.0 {
            return;
        }
        loop {
            match state {
                6 => {
                    self.time += fterm;
                    self.launch_key_data();
                    self.update_anim(obj_update);
                    return;
                }
                1 | 2 => state = self.load_data(),
                3 => {
                    if self.flags & 0x80 != 0 {
                        self.update_anim(obj_update);
                    }
                    state = self.load_wait();
                }
                4 => {
                    if self.fterm as f32 <= self.time {
                        state = 3;
                        fterm = self.fterm as f32;
                        self.time -= self.fterm as f32;
                        self.set_state(state);
                        continue;
                    }
                    self.update_anim(obj_update);
                    self.set_state(5);
                    return;
                }
                5 => {
                    state = 4;
                    self.set_state(state);
                }
                // 0, and 7..15 (C: infinite loop; see module docs).
                _ => return,
            }
        }
    }

    /// `HSD_FObjStopAnim`.
    pub fn stop_anim(&mut self, obj_update: &mut ObjUpdate<'_>, rate: f32) {
        if self.op_intrp == HSD_A_OP_KEY {
            self.interpret_anim(obj_update, rate);
        }
        self.set_state(0);
    }
}

/// `HSD_FObjLoadDesc` over a `next` chain.
pub fn load_desc_list(descs: &[HsdFObjDesc]) -> Vec<HsdFObj> {
    descs.iter().map(HsdFObj::load_desc).collect()
}
/// `HSD_FObjReqAnimAll`.
pub fn req_anim_all(list: &mut [HsdFObj], startframe: f32) {
    list.iter_mut().for_each(|f| f.req_anim(startframe));
}
/// `HSD_FObjStopAnimAll`.
pub fn stop_anim_all(list: &mut [HsdFObj], obj_update: &mut ObjUpdate<'_>, rate: f32) {
    list.iter_mut().for_each(|f| f.stop_anim(obj_update, rate));
}
/// `HSD_FObjInterpretAnimAll`.
pub fn interpret_anim_all(list: &mut [HsdFObj], obj_update: &mut ObjUpdate<'_>, rate: f32) {
    list.iter_mut().for_each(|f| f.interpret_anim(obj_update, rate));
}
