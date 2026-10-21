//! Hand-written `GcDecode` for records whose union member is chosen by a tag field, so the bytes
//! plus the relocation table cannot pick it (gen/gen_types.py `HAND_DECODE`). Each impl decodes
//! the generated fields, then the union the way the C loader reads it.

use crate::gen::*;
use melee_core::gcdecode::{GcDecode, GcDecoder};

/// jobj.h `JOBJ_PTCL`, `JOBJ_SPLINE`.
const JOBJ_PTCL: u32 = 1 << 5;
const JOBJ_SPLINE: u32 = 1 << 14;
/// robj.h `ROBJ_TYPE_MASK` and `REFTYPE_*`.
const ROBJ_TYPE_MASK: u32 = 0x7000_0000;
const REFTYPE_EXP: u32 = 0x0000_0000;
const REFTYPE_JOBJ: u32 = 0x1000_0000;
const REFTYPE_LIMIT: u32 = 0x2000_0000;
const REFTYPE_BYTECODE: u32 = 0x3000_0000;
const REFTYPE_IKHINT: u32 = 0x4000_0000;

fn off<T: melee_core::gc::GcLayout>(f: &str) -> u32 {
    T::gc_field(f).expect("generated field").offset
}

/// `JObjLoadJointSub` (jobj.c): spline if JOBJ_SPLINE, else particle list if JOBJ_PTCL, else
/// the DObj descriptor.
impl GcDecode for HSD_Joint {
    const GC_BYTES: u32 = <HSD_Joint as melee_core::gc::GcLayout>::GC_SIZE;
    fn gc_zeroed() -> Self {
        Self::zeroed()
    }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let mut v = Self::gc_decode_fields(d, at);
        let u = at + off::<HSD_Joint>("u");
        v.u = HSD_Joint_u::zeroed();
        if v.flags & JOBJ_SPLINE != 0 {
            v.u.spline = d.ptr(u);
        } else if v.flags & JOBJ_PTCL != 0 {
            v.u.ptcl = d.ptr(u);
        } else {
            v.u.dobjdesc = d.ptr(u);
        }
        v
    }
}

/// `HSD_RObjDesc.u` by `flags & ROBJ_TYPE_MASK` (robj.c: exp, jobj, limit, bytecode, ik hint).
impl GcDecode for HSD_RObjDesc {
    const GC_BYTES: u32 = <HSD_RObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    fn gc_zeroed() -> Self {
        Self::zeroed()
    }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let mut v = Self::gc_decode_fields(d, at);
        let u = at + off::<HSD_RObjDesc>("u");
        v.u = HSD_RObjDesc_u::zeroed();
        match v.flags & ROBJ_TYPE_MASK {
            REFTYPE_EXP => v.u.exp = d.ptr(u),
            REFTYPE_JOBJ => v.u.joint = d.ptr(u),
            REFTYPE_LIMIT => v.u.limit = f32::gc_decode(d, u),
            REFTYPE_BYTECODE => v.u.bcexp = d.ptr(u),
            REFTYPE_IKHINT => v.u.ik_hint = d.ptr(u),
            _ => v.u.i = u32::gc_decode(d, u),
        }
        v
    }
}
