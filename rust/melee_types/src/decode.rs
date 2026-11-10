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
        // `mtx` is the envelope matrix: jobj.c copies a whole `Mtx` (3 rows) from it, not the one row `MtxPtr` points at.
        v.mtx = d.ptr_array::<[f32; 4]>(at + off::<HSD_Joint>("mtx"), 3);
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

/// `HSD_PObjDesc.u` by `flags & POBJ_TYPE_MASK` (pobj.c `PObjLoad`): the shape set (0x1000), the envelope table
/// (0x2000) or the joint (0).
impl GcDecode for HSD_PObjDesc {
    const GC_BYTES: u32 = <HSD_PObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    fn gc_zeroed() -> Self {
        Self::zeroed()
    }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let mut v = Self::gc_decode_fields(d, at);
        let u = at + off::<HSD_PObjDesc>("u");
        v.u = HSD_PObjDesc_u::zeroed();
        match v.flags & 0x3000 {
            0x1000 => v.u.shape_set = d.ptr(u),
            0x2000 => v.u.envelope_p = d.ptr_table_null_arrays::<HSD_EnvelopeDesc>(u),
            _ => v.u.joint = d.ptr(u),
        }
        v
    }
}

/// `FtPartsDesc` (ft/types.h): `vis_table` is `FtPartsVisLookupList (*)[4]`, one row of four pointers per costume visibility
/// row; every pointer addresses `model_num` `FtPartsVisLookup`s (ftparts.c `ftParts_8007487C` / `ftParts_80074B6C`).
/// The row count is not in the data: the table runs to the next object.
impl GcDecode for FtPartsDesc {
    const GC_BYTES: u32 = <FtPartsDesc as melee_core::gc::GcLayout>::GC_SIZE;
    fn gc_zeroed() -> Self {
        Self::zeroed()
    }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let mut v = Self::zeroed();
        v.model_num = u32::gc_decode(d, at + off::<FtPartsDesc>("model_num"));
        let tab = at + off::<FtPartsDesc>("vis_table");
        if let Some(t) = d.target(tab) {
            let rows = (d.extent(t) / 16).max(1) as usize;
            let mut table: std::vec::Vec<[*mut FtPartsVisLookup; 4]> = std::vec![[core::ptr::null_mut(); 4]; rows];
            for (r, row) in table.iter_mut().enumerate() {
                for (c, slot) in row.iter_mut().enumerate() {
                    let w = t + (r * 16 + c * 4) as u32;
                    if d.is_reloc(w) {
                        *slot = d.ptr_array::<FtPartsVisLookup>(w, v.model_num as usize);
                    }
                }
            }
            v.vis_table = Box::leak(table.into_boxed_slice()).as_mut_ptr();
        }
        v
    }
}

/// `HSD_CObjDesc` is the union of the camera descriptors (cobj.h); `projection_type` (u16 at +6, common to all of
/// them) names the live one: 1 perspective, 2 frustum, 3 ortho.
impl GcDecode for HSD_CObjDesc {
    const GC_BYTES: u32 = <HSD_CObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    fn gc_zeroed() -> Self {
        Self::zeroed()
    }
    fn gc_decode(d: &mut GcDecoder<'_>, at: u32) -> Self {
        let mut v = Self::zeroed();
        match d.u16_at(at + 6) {
            1 => v.perspective = HSD_CameraDescPerspective::gc_decode(d, at),
            2 => v.frustum = HSD_CameraDescFrustum::gc_decode(d, at),
            3 => v.ortho = HSD_CameraDescFrustum::gc_decode(d, at),
            _ => v.common = HSD_CameraDescCommon::gc_decode(d, at),
        }
        v
    }
}

// MnSelectStageDataTable: generated from mn/mnstagesel.h since upstream #3583 named it (it was hand-written here).
