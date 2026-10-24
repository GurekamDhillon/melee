//! Port of src/melee/mp/mpisland.c: the "island" segment lists the map collision keeps for floors
//! and ceilings (one `mp_UnkStruct0` per connected run of lines, used by the stage / item code to
//! find the platform under a point), and the TARGET_PC script-joint batch rebuild. Generated from
//! the C by `gen/mtu/c2rs.py` (see `mplib.rs`); `mpIsland_8005AE1C` is a hand-structured goto
//! state machine (`gen/mtu/hand/`). `mpIsland_TerrainPalette` is `{ terrain kind, GXColor }[20]`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_none {
        ($name:ident, ($($a:ident),*)) => { panic!(concat!("TODO: ", stringify!($name), " is not ported")) };
        ($name:ident, ($($a:ident),*), $def:expr) => { ($def)($($a),*) };
    }
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => ext_none!($name, ($($a),*) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_Free(a0: *mut c_void) = |a0: *mut c_void| { unsafe { let _ = melee_runtime::tu::memory::HSD_Free(a0 as _); } };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void = |a0: i32| unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0 as _) as _ };
    }
}

pub static mpIsland_TerrainPalette: mpIsland_Palette = mpIsland_Palette { x0: [mpIsland_PaletteEntry { kind: 1_u32, color: GXColor { r: 128_u8, g: 96_u8, b: 96_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 2_u32, color: GXColor { r: 64_u8, g: 255_u8, b: 64_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 3_u32, color: GXColor { r: 192_u8, g: 96_u8, b: 96_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 4_u32, color: GXColor { r: 192_u8, g: 128_u8, b: 64_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 5_u32, color: GXColor { r: 64_u8, g: 64_u8, b: 64_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 6_u32, color: GXColor { r: 96_u8, g: 64_u8, b: 64_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 7_u32, color: GXColor { r: 192_u8, g: 192_u8, b: 96_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 8_u32, color: GXColor { r: 255_u8, g: 255_u8, b: 96_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 9_u32, color: GXColor { r: 192_u8, g: 255_u8, b: 192_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 10_u32, color: GXColor { r: 64_u8, g: 64_u8, b: 255_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 11_u32, color: GXColor { r: 64_u8, g: 255_u8, b: 255_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 12_u32, color: GXColor { r: 192_u8, g: 192_u8, b: 255_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 13_u32, color: GXColor { r: 192_u8, g: 255_u8, b: 64_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 14_u32, color: GXColor { r: 255_u8, g: 255_u8, b: 255_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 15_u32, color: GXColor { r: 192_u8, g: 192_u8, b: 255_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 16_u32, color: GXColor { r: 192_u8, g: 192_u8, b: 192_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 17_u32, color: GXColor { r: 12_u8, g: 12_u8, b: 12_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 18_u32, color: GXColor { r: 255_u8, g: 255_u8, b: 192_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 19_u32, color: GXColor { r: 255_u8, g: 5_u8, b: 5_u8, a: 255_u8 } }, mpIsland_PaletteEntry { kind: 4294967295_u32, color: GXColor { r: 0_u8, g: 0_u8, b: 0_u8, a: 0_u8 } }] };
pub static mut mpIsland_80458E88: mpIsland_80458E88_t = unsafe { core::mem::zeroed() };

/// `mpIsland_8005A6F8`
pub unsafe fn mpIsland_8005A6F8() {
    mpIsland_80458E88.next = null_mut();
    mpIsland_80458E88.x4 = null_mut();
    mpIsland_80458E88.x8 = null_mut();
    mpIsland_80458E88.xC = null_mut();
    mpIsland_80458E88.x18 = null_mut();
    mpIsland_80458E88.x1C = null_mut();
    mpIsland_80458E88.x10 = null_mut();
    mpIsland_80458E88.x14 = null_mut();
    mpIsland_80458E88.x20 = null_mut();
}

/// `mpIsland_AssertSeg`
unsafe fn mpIsland_AssertSeg(mut mpisp: *mut mp_UnkStruct0) {
    if !mpisp.is_null() { (); } else { panic!("mpIsland_AssertSeg: assertion failed"); }
}

/// `mpIsland_8005A728`
pub unsafe fn mpIsland_8005A728() {
    let mut map: *mut MapCollData = null_mut();
    let mut lines: *mut CollLine = null_mut();
    let mut vtx: *mut CollVtx = null_mut();
    #[derive(Clone, Copy)] #[repr(C)] struct Local1 { p: *mut mp_UnkStruct0 }
    let mut seg: Local1 = unsafe { core::mem::zeroed() };
    let mut z_val: f32 = 0.0;
    let mut count: i32 = 0;
    let mut line_idx: i32 = 0;
    let mut end_idx: i32 = 0;
    let mut next: i32 = 0;
    let mut hidden: i32 = 0;
    let mut prev: *mut mp_UnkStruct0 = null_mut();
    let mut visited: [u8; 1536] = [0; 1536];
    map = super::mplib::mpLib_8004D164();
    lines = super::mplib::mpGetGroundCollLine();
    vtx = super::mplib::mpGetGroundCollVtx();
    mpIsland_8005A6F8();
    core::ptr::write_bytes(((visited.as_mut_ptr()) as *mut c_void) as *mut u8, 0, (1536_u64 as i32) as usize);
    prev = null_mut();
    if { count = (*map).ranges[(MapLineGroup_Floor as i32) as usize].count as i32; count } != 0 {
        line_idx = (*map).ranges[(MapLineGroup_Floor as i32) as usize].start as i32;
        z_val = 0.0f32;
        'l25: while count != 0 {
            seg.p = ext::HSD_MemAlloc(44) as *mut mp_UnkStruct0;
            mpIsland_AssertSeg(seg.p);
            if !prev.is_null() {
                (*prev).next = seg.p;
            } else {
                mpIsland_80458E88.next = seg.p;
            }
            prev = seg.p;
            end_idx = line_idx;
            next = (!(line_idx != 0)) as i32;
            hidden = 0;
            'l26: while line_idx != next {
                visited[end_idx as usize] = 1 as u8;
                if (!(((*lines.offset(end_idx as isize)).flags & ((1 << 16) as u32)) != 0)) || (((*lines.offset(end_idx as isize)).flags & ((1 << 18) as u32)) != 0) {
                    hidden = 1;
                }
                next = (*(*lines.offset(end_idx as isize)).x0).next_id0 as i32;
                if (next == (-1_i32)) || (!((((*(*lines.offset(next as isize)).x0).hi_flags as i32) & (CollLine_Floor as i32)) != 0)) {
                    break 'l26;
                }
                end_idx = next;
            }
            (*seg.p).x20 = if hidden != 0 { 2 } else { 0 };
            (*seg.p).next = null_mut();
            (*seg.p).x24 = line_idx as i16;
            (*seg.p).x26 = end_idx as i16;
            (*seg.p).x4 = (*(*lines.offset(line_idx as isize)).x0).v0_idx;
            (*seg.p).x6 = (*(*lines.offset(end_idx as isize)).x0).v1_idx;
            (*seg.p).x8.x = (*vtx.offset((*seg.p).x4 as isize)).pos.x;
            (*seg.p).x8.y = (*vtx.offset((*seg.p).x4 as isize)).pos.y;
            (*seg.p).x8.z = z_val;
            (*seg.p).x14.x = (*vtx.offset((*seg.p).x6 as isize)).pos.x;
            (*seg.p).x14.y = (*vtx.offset((*seg.p).x6 as isize)).pos.y;
            (*seg.p).x14.z = z_val;
            (*seg.p).x28 = super::mplib::mpJointFromLine(line_idx) as i16;
            count = count - 1;
            line_idx = line_idx + 1;
            {
                let mut p: *mut u8 = core::ptr::addr_of_mut!(visited[line_idx as usize]) as *mut u8;
                let mut ctr: i32 = 0;
                {
                    ctr = count;
                    'l27: while ctr != 0 {
                        if !((*p) != 0) {
                            break 'l27;
                        }
                        p = p.offset(1);
                        line_idx = line_idx + 1;
                        count = count - 1;
                        ctr = ctr - 1;
                    }
                }
            }
        }
    }
    mpIsland_80458E88.x8 = prev;
    prev = null_mut();
    ();
    count = (*map).ranges[(MapLineGroup_Ceiling as i32) as usize].count as i32;
    if count != 0 {
        line_idx = (*map).ranges[(MapLineGroup_Ceiling as i32) as usize].start as i32;
        z_val = 0.0f32;
        'l28: while count != 0 {
            seg.p = ext::HSD_MemAlloc(44) as *mut mp_UnkStruct0;
            mpIsland_AssertSeg(seg.p);
            if !prev.is_null() {
                (*prev).next = seg.p;
            } else {
                mpIsland_80458E88.x4 = seg.p;
            }
            prev = seg.p;
            end_idx = line_idx;
            next = (!(line_idx != 0)) as i32;
            hidden = 0;
            'l29: while line_idx != next {
                visited[end_idx as usize] = 1 as u8;
                if (!(((*lines.offset(end_idx as isize)).flags & ((1 << 16) as u32)) != 0)) || (((*lines.offset(end_idx as isize)).flags & ((1 << 18) as u32)) != 0) {
                    hidden = 1;
                }
                next = (*(*lines.offset(end_idx as isize)).x0).prev_id0 as i32;
                if (next == (-1_i32)) || (!((((*(*lines.offset(next as isize)).x0).hi_flags as i32) & (CollLine_Ceiling as i32)) != 0)) {
                    break 'l29;
                }
                end_idx = next;
            }
            (*seg.p).x20 = if hidden != 0 { 2 } else { 0 };
            (*seg.p).next = null_mut();
            (*seg.p).x24 = line_idx as i16;
            (*seg.p).x26 = end_idx as i16;
            (*seg.p).x4 = (*(*lines.offset(line_idx as isize)).x0).v1_idx;
            (*seg.p).x6 = (*(*lines.offset(end_idx as isize)).x0).v0_idx;
            (*seg.p).x8.x = (*vtx.offset((*seg.p).x4 as isize)).pos.x;
            (*seg.p).x8.y = (*vtx.offset((*seg.p).x4 as isize)).pos.y;
            (*seg.p).x8.z = z_val;
            (*seg.p).x14.x = (*vtx.offset((*seg.p).x6 as isize)).pos.x;
            (*seg.p).x14.y = (*vtx.offset((*seg.p).x6 as isize)).pos.y;
            (*seg.p).x14.z = z_val;
            (*seg.p).x28 = super::mplib::mpJointFromLine(line_idx) as i16;
            count = count - 1;
            line_idx = line_idx + 1;
            {
                let mut p: *mut u8 = core::ptr::addr_of_mut!(visited[line_idx as usize]) as *mut u8;
                let mut ctr: i32 = 0;
                {
                    ctr = count;
                    'l30: while ctr != 0 {
                        if !((*p) != 0) {
                            break 'l30;
                        }
                        p = p.offset(1);
                        line_idx = line_idx + 1;
                        count = count - 1;
                        ctr = ctr - 1;
                    }
                }
            }
        }
    }
    mpIsland_80458E88.xC = prev;
}

/// `mpIsland_8005AB54`
pub unsafe fn mpIsland_8005AB54(mut surface_idx: i32) -> *mut mp_UnkStruct0 {
    if super::mplib::mpLib_80054ED8(surface_idx) != 0 {
        let mut v2: *mut CollLine = super::mplib::mpGetGroundCollLine();
        let mut done: i32 = 0;
        let mut cur: *mut mp_UnkStruct0 = null_mut();
        {
            cur = mpIsland_80458E88.next;
            'l31: while !cur.is_null() {
                let mut j: i32 = 0;
                let mut j_next: i32 = 0;
                {
                    j = (*cur).x24 as i32;
                    'l32: while j != (-1_i32) {
                        if j == surface_idx {
                            return cur;
                        }
                        if j == ((*cur).x26 as i32) {
                            break 'l32;
                        }
                        done = 1;
                        j_next = (*(*v2.offset(j as isize)).x0).next_id0 as i32;
                        if (j_next != (-1_i32)) && (((*v2.offset(j_next as isize)).flags & ((CollLine_Floor as i32) as u32)) != 0) {
                            done = 0;
                        }
                        if done != 0 {
                            break 'l32;
                        }
                        j = j_next;
                    }
                }
                cur = (*cur).next;
            }
        }
    }
    return null_mut();
}

/// `mpIsland_8005AC14`
pub unsafe fn mpIsland_8005AC14(mut arg0: *mut Vec, mut arg1: f32) -> *mut mp_UnkStruct0 {
    let mut i: i32 = 0;
    if super::mplib::mpCheckFloor((*arg0).x, (*arg0).y, (*arg0).x, (*arg0).y + arg1, 0.0f32, null_mut(), core::ptr::addr_of_mut!(i) as *mut i32, null_mut(), null_mut(), (-1_i32), (-1_i32), (-1_i32), None, null_mut()) != 0 {
        return mpIsland_8005AB54(i);
    }
    return null_mut();
}

/// `mpIsland_8005AC8C`
pub unsafe fn mpIsland_8005AC8C(mut arg0: *mut mp_UnkStruct0) -> i32 {
    let mut temp_r3: *mut CollJoint = core::ptr::addr_of_mut!(*super::mplib::mpGetGroundCollJoint().offset((*arg0).x28 as isize)) as *mut CollJoint;
    if ((*temp_r3).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
        return 1;
    }
    if ((*temp_r3).flags & ((((CollJoint_B10 as i32) | (CollJoint_B9 as i32)) | (CollJoint_B8 as i32)) as u32)) != 0 {
        return 1;
    }
    return 0;
}

/// `mpIsland_8005ACE8`
pub unsafe fn mpIsland_8005ACE8(mut arg0: *mut mp_UnkStruct0, mut arg1: *mut Vec, mut arg2: *mut Vec) {
    let mut var_r31: i32 = 0;
    let mut var_r30: *mut CollLine = null_mut();
    let mut temp_r29: i32 = 0;
    let mut var_r28: i32 = 0;
    let mut temp_r3: *mut CollJoint = null_mut();
    let mut var_r27: i32 = 0;
    let mut i: i32 = 0;
    temp_r3 = core::ptr::addr_of_mut!(*super::mplib::mpGetGroundCollJoint().offset((*arg0).x28 as isize)) as *mut CollJoint;
    var_r31 = (*(*temp_r3).inner).ranges[(MapLineGroup_Floor as i32) as usize].start as i32;
    var_r30 = core::ptr::addr_of_mut!(*super::mplib::mpGetGroundCollLine().offset(var_r31 as isize)) as *mut CollLine;
    temp_r29 = (*(*temp_r3).inner).ranges[(MapLineGroup_Floor as i32) as usize].count as i32;
    var_r28 = 1;
    var_r27 = 1;
    if !arg1.is_null() {
        *arg1 = (*arg0).x8;
    } else {
        var_r28 = 0;
    }
    if !arg2.is_null() {
        *arg2 = (*arg0).x14;
    } else {
        var_r27 = 0;
    }
    {
        i = 0;
        'l33: while ((i < temp_r29) && (var_r28 != 0)) && (var_r27 != 0) {
            if (var_r28 != 0) && (((*(*var_r30).x0).v0_idx as i32) == ((*arg0).x4 as i32)) {
                super::mplib::mpFloorGetLeft(var_r31, arg1);
            } else if (var_r27 != 0) && (((*(*var_r30).x0).v1_idx as i32) == ((*arg0).x6 as i32)) {
                super::mplib::mpFloorGetRight(var_r31, arg2);
            }
            i = i + 1;
            var_r30 = var_r30.offset(1);
            var_r31 = var_r31 + 1;
        }
    }
}

/// `mpIsland_8005AE1C`: re-sort the floor (`arg0`) / ceiling (`arg1`) island lists against the vertex
/// range `[arg2, arg2 + arg3)`; `arg4` selects the side an overlapping island goes to. The C is a
/// goto state machine (`loop1_body` / `loop2_body`); each loop is a `while` here, with the
/// `add_to_prev_*` labels as flags.
pub unsafe fn mpIsland_8005AE1C(mut arg0: *mut *mut mp_UnkStruct0, mut arg1: *mut *mut mp_UnkStruct0, mut arg2: i32, mut arg3: i32, mut arg4: i32) {
    let mut prev_b: *mut mp_UnkStruct0 = null_mut();
    let mut prev_a: *mut mp_UnkStruct0 = null_mut();
    let vtx: *mut CollVtx = super::mplib::mpGetGroundCollVtx();
    let mut cur: *mut mp_UnkStruct0 = *arg0;
    let end: i32 = arg2 + arg3;
    let z_val: f32 = 0.0;

    while !cur.is_null() {
        let v0: u16 = (*cur).x4;
        let next: *mut mp_UnkStruct0 = (*cur).next;

        // add_to_prev_b_1: the island lies entirely below or entirely above the range
        let mut to_b = false;
        if (v0 as i32) < arg2 {
            if ((*cur).x6 as i32) < arg2 {
                to_b = true;
            }
        }
        if !to_b && !(end > v0 as i32) {
            if end <= (*cur).x6 as i32 {
                to_b = true;
            }
        }
        if !to_b {
            // do_assign_1
            (*cur).x8.x = (*vtx.offset(v0 as isize)).pos.x;
            (*cur).x8.y = (*vtx.offset((*cur).x4 as isize)).pos.y;
            (*cur).x8.z = z_val;
            (*cur).x14.x = (*vtx.offset((*cur).x6 as isize)).pos.x;
            (*cur).x14.y = (*vtx.offset((*cur).x6 as isize)).pos.y;
            (*cur).x14.z = z_val;

            if arg4 != 0 || ((*cur).x20 & 2) != 0 {
                to_b = true;
            } else {
                (*cur).x20 |= 2;
                (*cur).next = prev_a;
                prev_a = cur;
            }
        }
        if to_b {
            (*cur).next = prev_b;
            prev_b = cur;
        }
        cur = next;
    }

    cur = *arg1;
    while !cur.is_null() {
        let v0: u16 = (*cur).x4;
        let next: *mut mp_UnkStruct0 = (*cur).next;

        // add_to_prev_a_2
        let mut to_a = false;
        if (v0 as i32) < arg2 {
            if ((*cur).x6 as i32) < arg2 {
                to_a = true;
            }
        }
        if !to_a && !(end > v0 as i32) {
            if end <= (*cur).x6 as i32 {
                to_a = true;
            }
        }
        if !to_a {
            // do_assign_2
            (*cur).x8.x = (*vtx.offset(v0 as isize)).pos.x;
            (*cur).x8.y = (*vtx.offset((*cur).x4 as isize)).pos.y;
            (*cur).x8.z = z_val;
            (*cur).x14.x = (*vtx.offset((*cur).x6 as isize)).pos.x;
            (*cur).x14.y = (*vtx.offset((*cur).x6 as isize)).pos.y;
            (*cur).x14.z = z_val;

            if arg4 == 0 || ((*cur).x20 & 2) == 0 {
                to_a = true;
            } else {
                (*cur).x20 &= !2;
                (*cur).next = prev_b;
                prev_b = cur;
            }
        }
        if to_a {
            (*cur).next = prev_a;
            prev_a = cur;
        }
        cur = next;
    }

    *arg0 = prev_b;
    *arg1 = prev_a;
}

/// `mpIsland_8005B004`
pub unsafe fn mpIsland_8005B004(mut arg0: *mut *mut mp_UnkStruct0, mut arg1: *mut *mut mp_UnkStruct0, mut arg2: i32, mut arg3: i32, mut arg4: i32, mut arg5: i32, mut arg6: i32) {
    let mut _q0: [u8; 8] = [0; 8];
    let mut cur: *mut mp_UnkStruct0 = null_mut();
    let mut z_val: f32 = 0.0;
    let mut visited: [u8; 1536] = [0; 1536];
    let mut next: *mut mp_UnkStruct0 = null_mut();
    let mut prev: *mut mp_UnkStruct0 = null_mut();
    let mut mpisp: *mut mp_UnkStruct0 = null_mut();
    let mut lines: *mut CollLine = null_mut();
    let mut vtx: *mut CollVtx = null_mut();
    let mut joints: *mut CollJoint = null_mut();
    let mut line_idx: i32 = 0;
    let mut end_idx: i32 = 0;
    let mut start_idx: i32 = 0;
    let mut end_total: i32 = 0;
    let mut r#loop: i32 = 0;
    let mut type_flag: u32 = 0;
    let mut link: i16 = 0;
    let mut cycle_start: i32 = 0;
    let mut min_x: f32 = 0.0;
    let mut max_x: f32 = 0.0;
    let mut i: i32 = 0;
    type_flag = (arg3 | 16) as u32;
    prev = null_mut();
    cur = *arg0;
    'l34: while !cur.is_null() {
        next = (*cur).next;
        if ((*cur).x28 as i32) == arg2 {
            (*cur).next = *arg1;
            *arg1 = cur;
        } else {
            (*cur).next = prev;
            prev = cur;
        }
        cur = next;
    }
    *arg0 = prev;
    core::ptr::write_bytes(((visited.as_mut_ptr()) as *mut c_void) as *mut u8, 0, (1536_u64 as i32) as usize);
    joints = super::mplib::mpGetGroundCollJoint();
    joints = core::ptr::addr_of_mut!(*joints.offset(arg2 as isize)) as *mut CollJoint;
    lines = super::mplib::mpGetGroundCollLine();
    vtx = super::mplib::mpGetGroundCollVtx();
    z_val = 0.0f32;
    end_total = ((*(*joints).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as i32) + ((*(*joints).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].count as i32);
    line_idx = (*(*joints).inner).ranges[(MapLineGroup_Dynamic as i32) as usize].start as i32;
    'l35: while line_idx < end_total {
        'c35: {
            if ((visited[line_idx as usize] as i32) != 0) || (((*lines.offset(line_idx as isize)).flags & (31 as u32)) != type_flag) {
                line_idx = line_idx + 1;
                break 'c35;
            }
            end_idx = line_idx;
            r#loop = 0;
            'l36: while true {
                visited[end_idx as usize] = 1 as u8;
                link = (*(*lines.offset(end_idx as isize)).x0).prev_id0;
                if (link as i32) == (-1_i32) {
                    break 'l36;
                }
                if ((*lines.offset(link as isize)).flags & (31 as u32)) != type_flag {
                    break 'l36;
                }
                end_idx = link as i32;
                if (link as i32) == line_idx {
                    r#loop = 1;
                    break 'l36;
                }
            }
            if !(r#loop != 0) {
                start_idx = line_idx;
                'l37: while true {
                    visited[start_idx as usize] = 1 as u8;
                    link = (*(*lines.offset(start_idx as isize)).x0).next_id0;
                    if (link as i32) == (-1_i32) {
                        break 'l37;
                    }
                    if ((*lines.offset(link as isize)).flags & (31 as u32)) != type_flag {
                        break 'l37;
                    }
                    start_idx = link as i32;
                    if (link as i32) == line_idx {
                        r#loop = 1;
                        break 'l37;
                    }
                }
                if !(r#loop != 0) { (); } else { panic!("mpIsland_8005B004: assertion failed"); }
            } else {
                i = { start_idx = end_idx; start_idx };
                min_x = -f32::MAX;
                max_x = f32::MAX;
                cycle_start = i;
                'l38: loop {
                    if max_x > (*vtx.offset((*(*lines.offset(i as isize)).x0).v0_idx as isize)).pos.x {
                        max_x = (*vtx.offset((*(*lines.offset(i as isize)).x0).v0_idx as isize)).pos.x;
                        end_idx = i;
                    }
                    {
                        if min_x > (*vtx.offset((*(*lines.offset(i as isize)).x0).v1_idx as isize)).pos.x {
                            min_x = (*vtx.offset((*(*lines.offset(i as isize)).x0).v1_idx as isize)).pos.x;
                            start_idx = i;
                        }
                    }
                    link = (*(*lines.offset(i as isize)).x0).prev_id0;
                    *lines.offset(link as isize);
                    i = link as i32;
                    if !((link as i32) != cycle_start) { break 'l38; }
                }
            }
            if !{ mpisp = *arg1; mpisp }.is_null() {
                *arg1 = (*mpisp).next;
            } else {
                mpisp = ext::HSD_MemAlloc(44) as *mut mp_UnkStruct0;
                mpIsland_AssertSeg(mpisp);
            }
            (*mpisp).x20 = if arg6 != 0 { 0 } else { 2 };
            (*mpisp).next = null_mut();
            (*mpisp).x24 = end_idx as i16;
            (*mpisp).x26 = start_idx as i16;
            (*mpisp).x4 = (*(*lines.offset(end_idx as isize)).x0).v0_idx;
            (*mpisp).x6 = (*(*lines.offset(start_idx as isize)).x0).v1_idx;
            (*mpisp).x8.x = (*vtx.offset((*mpisp).x4 as isize)).pos.x;
            (*mpisp).x8.y = (*vtx.offset((*mpisp).x4 as isize)).pos.y;
            (*mpisp).x8.z = z_val;
            (*mpisp).x14.x = (*vtx.offset((*mpisp).x6 as isize)).pos.x;
            (*mpisp).x14.y = (*vtx.offset((*mpisp).x6 as isize)).pos.y;
            (*mpisp).x14.z = z_val;
            (*mpisp).x28 = arg2 as i16;
            (*mpisp).next = *arg0;
            *arg0 = mpisp;
        }
    }
}

/// `mpIsland_8005B334`
pub unsafe fn mpIsland_8005B334(mut arg0: i32, mut arg1: i32, mut arg2: i32, mut arg3: i32) {
    let mut temp: *mut mp_UnkStruct0 = null_mut();
    if !mpIsland_80458E88.x8.is_null() {
        (*mpIsland_80458E88.x8).next = null_mut();
    } else {
        mpIsland_80458E88.next = null_mut();
    }
    if !mpIsland_80458E88.xC.is_null() {
        (*mpIsland_80458E88.xC).next = null_mut();
    } else {
        mpIsland_80458E88.x4 = null_mut();
    }
    mpIsland_8005AE1C(core::ptr::addr_of_mut!(mpIsland_80458E88.next) as *mut *mut mp_UnkStruct0, core::ptr::addr_of_mut!(mpIsland_80458E88.x18) as *mut *mut mp_UnkStruct0, arg1, arg2, arg3);
    mpIsland_8005AE1C(core::ptr::addr_of_mut!(mpIsland_80458E88.x4) as *mut *mut mp_UnkStruct0, core::ptr::addr_of_mut!(mpIsland_80458E88.x1C) as *mut *mut mp_UnkStruct0, arg1, arg2, arg3);
    mpIsland_8005B004(core::ptr::addr_of_mut!(mpIsland_80458E88.x10) as *mut *mut mp_UnkStruct0, core::ptr::addr_of_mut!(mpIsland_80458E88.x20) as *mut *mut mp_UnkStruct0, arg0, 1, arg1, arg2, arg3);
    mpIsland_8005B004(core::ptr::addr_of_mut!(mpIsland_80458E88.x14) as *mut *mut mp_UnkStruct0, core::ptr::addr_of_mut!(mpIsland_80458E88.x20) as *mut *mut mp_UnkStruct0, arg0, 2, arg1, arg2, arg3);
    temp = mpIsland_80458E88.next;
    'l39: while (!temp.is_null()) && (!(*temp).next.is_null()) {
        temp = (*temp).next;
    }
    if !temp.is_null() {
        (*temp).next = mpIsland_80458E88.x10;
    } else {
        mpIsland_80458E88.next = mpIsland_80458E88.x10;
    }
    mpIsland_80458E88.x8 = temp;
    temp = mpIsland_80458E88.x4;
    'l40: while (!temp.is_null()) && (!(*temp).next.is_null()) {
        temp = (*temp).next;
    }
    if !temp.is_null() {
        (*temp).next = mpIsland_80458E88.x14;
    } else {
        mpIsland_80458E88.x4 = mpIsland_80458E88.x14;
    }
    mpIsland_80458E88.xC = temp;
}

/// `mpIsland_StageSlotFree`
pub unsafe fn mpIsland_StageSlotFree() {
    let mut lists: [*mut mp_UnkStruct0; 5] = [null_mut(); 5];
    let mut i: i32 = 0;
    lists[0 as usize] = mpIsland_80458E88.next;
    lists[1 as usize] = mpIsland_80458E88.x4;
    lists[2 as usize] = mpIsland_80458E88.x20;
    lists[3 as usize] = mpIsland_80458E88.x18;
    lists[4 as usize] = mpIsland_80458E88.x1C;
    {
        i = 0;
        'l41: while i < 5 {
            let mut p: *mut mp_UnkStruct0 = lists[i as usize];
            'l42: while !p.is_null() {
                let mut n: *mut mp_UnkStruct0 = (*p).next;
                ext::HSD_Free(p as *mut c_void);
                p = n;
            }
            i = i + 1;
        }
    }
    mpIsland_8005A6F8();
}

/// `mpIsland_ScriptRecycle`
unsafe fn mpIsland_ScriptRecycle(mut list: *mut *mut mp_UnkStruct0, mut dirty: *const u8, mut count: i32) {
    let mut at: *mut *mut mp_UnkStruct0 = list;
    'l43: while !(*at).is_null() {
        let mut cur: *mut mp_UnkStruct0 = *at;
        if ((((*cur).x28 as i32) >= 0) && (((*cur).x28 as i32) < count)) && (((*dirty.offset((*cur).x28 as isize)) as i32) != 0) {
            *at = (*cur).next;
            (*cur).next = mpIsland_80458E88.x20;
            mpIsland_80458E88.x20 = cur;
        } else {
            at = core::ptr::addr_of_mut!((*cur).next) as *mut *mut mp_UnkStruct0;
        }
    }
}

/// `mpIsland_ScriptBatch`
pub unsafe fn mpIsland_ScriptBatch(mut dirty: *const u8, mut count: i32) {
    let mut i: i32 = 0;
    let mut any: i32 = 0;
    let mut kind: i32 = 0;
    let mut joints: *mut CollJoint = super::mplib::mpGetGroundCollJoint();
    if count > 1024 {
        count = 1024;
    }
    {
        i = 0;
        'l44: while i < count {
            any = any | ((*dirty.offset(i as isize)) as i32);
            i = i + 1;
        }
    }
    if !(any != 0) {
        return;
    }
    if !mpIsland_80458E88.x8.is_null() {
        (*mpIsland_80458E88.x8).next = null_mut();
    } else {
        mpIsland_80458E88.next = null_mut();
    }
    if !mpIsland_80458E88.xC.is_null() {
        (*mpIsland_80458E88.xC).next = null_mut();
    } else {
        mpIsland_80458E88.x4 = null_mut();
    }
    mpIsland_ScriptRecycle(core::ptr::addr_of_mut!(mpIsland_80458E88.x10) as *mut *mut mp_UnkStruct0, dirty, count);
    mpIsland_ScriptRecycle(core::ptr::addr_of_mut!(mpIsland_80458E88.x14) as *mut *mut mp_UnkStruct0, dirty, count);
    {
        kind = 1;
        'l45: while kind <= 2 {
            let mut list: *mut *mut mp_UnkStruct0 = if kind == 1 { core::ptr::addr_of_mut!(mpIsland_80458E88.x10) as *mut *mut mp_UnkStruct0 } else { core::ptr::addr_of_mut!(mpIsland_80458E88.x14) as *mut *mut mp_UnkStruct0 };
            let mut tail: *mut *mut mp_UnkStruct0 = list;
            'l46: while !(*tail).is_null() {
                tail = core::ptr::addr_of_mut!((*(*tail)).next) as *mut *mut mp_UnkStruct0;
            }
            {
                i = 0;
                'l47: while i < count {
                    'c47: {
                        let mut fresh: *mut mp_UnkStruct0 = null_mut();
                        let mut joint: *mut CollJoint = null_mut();
                        let mut enabled: i32 = 0;
                        if !((*dirty.offset(i as isize)) != 0) {
                            break 'c47;
                        }
                        joint = core::ptr::addr_of_mut!(*joints.offset(i as isize)) as *mut CollJoint;
                        enabled = ((((*joint).flags & ((CollJoint_Enabled as i32) as u32)) != 0) && (!(((*joint).flags & (((CollJoint_Hidden as i32) | (CollJoint_B11 as i32)) as u32)) != 0))) as i32;
                        mpIsland_8005B004(core::ptr::addr_of_mut!(fresh) as *mut *mut mp_UnkStruct0, core::ptr::addr_of_mut!(mpIsland_80458E88.x20) as *mut *mut mp_UnkStruct0, i, kind, (*(*joint).inner).vtx_start as i32, (*(*joint).inner).vtx_count as i32, enabled);
                        *tail = fresh;
                        'l48: while !(*tail).is_null() {
                            tail = core::ptr::addr_of_mut!((*(*tail)).next) as *mut *mut mp_UnkStruct0;
                        }
                    }
                    i = i + 1;
                }
            }
            kind = kind + 1;
        }
    }
    if !mpIsland_80458E88.x8.is_null() {
        (*mpIsland_80458E88.x8).next = mpIsland_80458E88.x10;
    } else {
        mpIsland_80458E88.next = mpIsland_80458E88.x10;
    }
    if !mpIsland_80458E88.xC.is_null() {
        (*mpIsland_80458E88.xC).next = mpIsland_80458E88.x14;
    } else {
        mpIsland_80458E88.x4 = mpIsland_80458E88.x14;
    }
}
