//! src/melee/ft/ftmetal.c: metal box / always-metal fighters (switch the part JObjs to the metal
//! lighting flags and back) and loading the fighter's per-part DObjs (ft_800C85B8).
#![allow(non_snake_case)]

use crate::types::*;

/// Calls into TUs that are not ported yet (TODO: rewire when they are).
#[allow(non_snake_case, unused_variables)]
pub mod ext {
    use crate::types::*;
    use core::ffi::c_void;
    pub fn HSD_JObjGetDObj(jobj: *mut HSD_JObj) -> *mut HSD_DObj { panic!("TODO(group1 jobj) HSD_JObjGetDObj") }
    pub fn HSD_JObjSetFlags(jobj: *mut HSD_JObj, flags: u32) { panic!("TODO(group1 jobj) HSD_JObjSetFlags") }
    pub fn HSD_JObjClearFlags(jobj: *mut HSD_JObj, flags: u32) { panic!("TODO(group1 jobj) HSD_JObjClearFlags") }
    pub fn HSD_JObjAddDObj(jobj: *mut HSD_JObj, dobj: *mut HSD_DObj) { panic!("TODO(group1 jobj) HSD_JObjAddDObj") }
    pub fn HSD_DObjModifyFlags(dobj: *mut HSD_DObj, a: u32, b: u32) { panic!("TODO(group1 dobj) HSD_DObjModifyFlags") }
    pub fn HSD_DObjLoadDesc(desc: *mut HSD_DObjDesc) -> *mut HSD_DObj { panic!("TODO(group1 dobj) HSD_DObjLoadDesc") }
    pub fn HSD_DObjResolveRefsAll(dobj: *mut HSD_DObj, desc: *mut HSD_DObjDesc) { panic!("TODO(group1 dobj) HSD_DObjResolveRefsAll") }
    pub fn HSD_IDInsertToTable(table: *mut c_void, key: usize, data: *mut c_void) { { unsafe { let _ = melee_runtime::tu::id::HSD_IDInsertToTable(table as _, key as _, data as _); } return; } }
    pub fn ftAnim_GetNextJointInTree(joint: &mut *mut HSD_Joint, depth: &mut i32) { panic!("TODO(ftanim) ftAnim_GetNextJointInTree") }
    pub fn lb_8000CE30(a: *mut HSD_DObj, b: *mut HSD_DObj) { { unsafe { let _ = melee_runtime::tu::lb_00B0::lb_8000CE30(a as _, b as _); } return; } }
    pub fn hsdChangeClass(obj: *mut c_void, info: *mut c_void) { { unsafe { let _ = melee_runtime::tu::class::hsdChangeClass(obj as _, info as _); } return; } }
    /// `&ftMObj` (ftmaterial.c class info).
    pub fn ftMObj() -> *mut c_void { panic!("TODO(ftmaterial) ftMObj class info") }
}

use super::ftlipstickswing::fp;

fn parts_num(f: &Fighter) -> i32 {
    unsafe { (**crate::hooks::ft_parts_table().add(f.kind as usize)).parts_num as i32 }
}

#[inline]
fn bone<'a>(f: &Fighter, i: i32) -> &'a mut FighterBone {
    unsafe { &mut *f.parts.add(i as usize) }
}

/// `ft_800C8170(fp)`: restore each visible part's own JObj lighting flags and DObj blend
/// classes (the inverse of the metal switch).
pub fn ft_800C8170(f: &mut Fighter) {
    for i in 0..parts_num(f) {
        let b = bone(f, i);
        let bits = unsafe { b.anon_8.anon_0 };
        if bits.flags_b1() == 0 {
            continue;
        }
        let jobj = b.joint;
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        let mut flags: u32 = 0;
        if bits.flags_b7() != 0 { flags |= JOBJ_LIGHTING as u32; }
        if bits.flags2_b0() != 0 { flags |= JOBJ_TEXGEN as u32; }
        if bits.flags2_b1() != 0 { flags |= JOBJ_SPECULAR as u32; }
        if bits.flags2_b2() != 0 { flags |= JOBJ_UNK_B18 as u32; }
        if bits.flags2_b3() != 0 { flags |= JOBJ_UNK_B20 as u32; }
        if bits.flags2_b4() != 0 { flags |= JOBJ_UNK_B19 as u32; }
        ext::HSD_JObjClearFlags(jobj, 0x50180);
        ext::HSD_JObjSetFlags(jobj, flags);
        while !dobj.is_null() {
            let mobj = unsafe { (*dobj).mobj };
            let rm = unsafe { (*mobj).rendermode } as u32;
            let blend = (RENDER_XLU | RENDER_NO_ZUPDATE) as u32;
            match rm & blend {
                0 => ext::HSD_DObjModifyFlags(dobj, 2, 0xE),
                x if x == RENDER_XLU as u32 => ext::HSD_DObjModifyFlags(dobj, 8, 0xE),
                x if x == blend => ext::HSD_DObjModifyFlags(dobj, 4, 0xE),
                _ => panic!("mobj has unexpected blending flags (0x{rm:x})."),
            }
            dobj = unsafe { (*dobj).next };
        }
    }
}

/// static inline `enableMetal(fp)`.
fn enable_metal(f: &mut Fighter) {
    for i in 0..parts_num(f) {
        let b = bone(f, i);
        if unsafe { b.anon_8.anon_0.flags_b1() } == 0 {
            continue;
        }
        let jobj = b.joint;
        let mut dobj = ext::HSD_JObjGetDObj(jobj);
        ext::HSD_JObjSetFlags(jobj, 0x50180);
        ext::HSD_JObjClearFlags(jobj, 0x180000);
        while !dobj.is_null() {
            ext::HSD_DObjModifyFlags(dobj, 2, 0xE);
            dobj = unsafe { (*dobj).next };
        }
    }
    crate::ftcommon2::ftcommon_80080460(f);
}

/// `ftCo_800C8348(gobj, timer, health)`: start (or refresh) metal.
pub fn ftCo_800C8348(gobj: *mut HSD_GObj, timer: i32, health: i32) {
    let f = fp(gobj);
    f.metal_timer = timer;
    f.metal_health = health;
    if f.is_metal() != 0 {
        return;
    }
    f.set_is_metal(1);
    enable_metal(f);
}

/// `ftCo_800C8438(gobj)`: reset metal to the fighter's always-metal default.
pub fn ftCo_800C8438(gobj: *mut HSD_GObj) {
    let f = fp(gobj);
    let am = f.is_always_metal();
    f.set_is_metal(am);
    f.metal_timer = 0;
    f.metal_health = 0;
    if f.is_metal() == 0 {
        return;
    }
    enable_metal(f);
    crate::hooks::ftco_800d105c(gobj);
    crate::hooks::ft_80081c88(gobj, f.x34_scale.y);
}

/// `ftCo_800C8540(gobj)`: end metal (unless always metal).
pub fn ftCo_800C8540(gobj: *mut HSD_GObj) {
    let f = fp(gobj);
    f.metal_timer = 0;
    f.metal_health = 0;
    if f.is_always_metal() == 0 {
        f.set_is_metal(0);
        crate::hooks::ftco_800d105c(gobj);
        crate::hooks::ft_80081c88(gobj, f.x34_scale.y);
        ft_800C8170(f);
        crate::ftcommon2::ftcommon_80080474(f);
    }
}

/// `ft_800C85B8(gobj)`: register every part JObj by its joint desc, then load each joint's
/// DObj desc onto its part and record the DObjs in `fp->x203C`.
pub fn ft_800C85B8(gobj: *mut HSD_GObj) {
    let f = fp(gobj);
    let joint = unsafe { (*f.ft_data).x5C } as *mut HSD_Joint;
    let mut sp20 = joint;
    let mut sp1c: i32 = 0;
    let mut joint_idx: i32 = 0;
    let mut dobj_count: i32 = 0;
    while !sp20.is_null() {
        if crate::ftdmgfall::ftparts_8007506c(f.kind as i32, joint_idx) != 0 {
            joint_idx += 1;
        } else {
            ext::HSD_IDInsertToTable(core::ptr::null_mut(), sp20 as usize, bone(f, joint_idx).joint as *mut _);
            joint_idx += 1;
            ext::ftAnim_GetNextJointInTree(&mut sp20, &mut sp1c);
        }
    }
    sp20 = joint;
    sp1c = 0;
    let mut part_idx: i32 = 0;
    while !sp20.is_null() {
        if crate::ftdmgfall::ftparts_8007506c(f.kind as i32, part_idx) != 0 {
            part_idx += 1;
            continue;
        }
        let mut i = 0;
        let part_jobj = bone(f, part_idx).joint;
        let desc = unsafe { (*sp20).u.dobjdesc };
        let mut dobj = ext::HSD_DObjLoadDesc(desc);
        if !dobj.is_null() {
            let mut it = ext::HSD_JObjGetDObj(part_jobj);
            unsafe { bone(f, part_idx).anon_8.anon_0.set_flags2_b5(1) };
            ext::HSD_DObjResolveRefsAll(dobj, desc);
            if it.is_null() {
                ext::HSD_JObjAddDObj(part_jobj, dobj);
            } else {
                while unsafe { !(*it).next.is_null() } {
                    it = unsafe { (*it).next };
                }
                ext::lb_8000CE30(it, dobj);
            }
            while !dobj.is_null() {
                assert!(dobj_count < 0x20, "fighter parts model dobj num over!");
                unsafe { *f.x203C.data.add(dobj_count as usize) = dobj };
                let mobj = unsafe { (*dobj).mobj };
                if !mobj.is_null() {
                    ext::hsdChangeClass(mobj as *mut _, ext::ftMObj());
                }
                dobj = unsafe { (*dobj).next };
                dobj_count += 1;
                i += 1;
            }
            assert!(i < 0x80, "fighter dobj num over!");
            unsafe {
                bone(f, part_idx).anon_c.anon_0.set_xD((dobj_count - 1) as u8);
                bone(f, part_idx).anon_8.anon_0.set_flags2_b6(1);
            }
        }
        part_idx += 1;
        ext::ftAnim_GetNextJointInTree(&mut sp20, &mut sp1c);
    }
    f.x203C.count = dobj_count as u32;
}
