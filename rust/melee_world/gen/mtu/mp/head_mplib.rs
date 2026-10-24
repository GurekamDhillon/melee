//! Port of src/melee/mp/mplib.c: the map collision library (the ground collision vertex / line /
//! joint tables, line walking and the floor / ceiling / wall segment queries, joint list editing,
//! bounding checks and the debug drawing). Generated from the C by `gen/mtu/c2rs.py` (libclang AST:
//! every implicit conversion is the C one, `-ffp-contract=on` multiply-adds become `fmadds` /
//! `mul_add`, checked against `tools/rust-port/fma_sites.py`), with the goto-state-machine functions
//! that the tool cannot structure in `gen/mtu/hand/`. `bool` is MSL `int`. Callees of unported TUs
//! are `ext` hooks named like the C (unset: panics "TODO"); the TARGET_PC script hooks
//! (`ScriptGame_*`) are ext hooks too.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports, dead_code, unused_assignments, unused_variables, unused_mut, unused_parens, unused_labels, unused_comparisons, clippy::all)]

use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

use melee_core::fp::fmadds;
use melee_types::*;

/// `LINEID_CHECK` (hand ports): `HSD_ASSERTREPORT(line, 0, "%s:%d:not found lineID=%d")`
unsafe fn lineid_check(line: u32, line_id: i32) {
    if line_id == -1 || line_id >= (*mpLib_804D64B4).line_count {
        panic!("src/melee/mp/mplib.c:{}:not found lineID={}", line, line_id);
    }
}

/// `PSMTXMultVec` (dolphin/mtx; ported in melee_runtime::mtx): `*dst = m * *src`
unsafe fn PSMTXMultVec(m: *mut [f32; 4], src: *mut Vec, dst: *mut Vec) {
    let r = melee_runtime::mtx::mult_vec(
        core::slice::from_raw_parts(m as *const [f32; 4], 3),
        &melee_core::vec::Vec3 { x: (*src).x, y: (*src).y, z: (*src).z },
    );
    (*dst).x = r.x;
    (*dst).y = r.y;
    (*dst).z = r.z;
}

/// `mp_script_isolation` (script_stage_isolation.inc, included at the end of mplib.c on PC)
pub struct MpScriptIsolation {
    pub map: *mut MapCollData,
    pub lines: *mut CollLine,
    pub joints: *mut CollJoint,
    pub nlines: i32,
    pub njoints: i32,
    pub line_flags: [u32; 1536],
    pub joint_flags: [u32; 256],
    pub ranges: [[MapLineRange; 5]; 256],
}
pub static mut mp_script_isolation: MpScriptIsolation = unsafe { core::mem::zeroed() };

/// `mpScriptStageIsolationActive`
pub unsafe fn mpScriptStageIsolationActive() -> i32 {
    ((!mp_script_isolation.map.is_null())
        && mp_script_isolation.map == mpLib_804D64B4
        && mp_script_isolation.lines == groundCollLine
        && mp_script_isolation.joints == groundCollJoint) as i32
}

/// `mpScriptStageIsolationSet`: hides the original ranges (lines disabled, joint ranges emptied) for a script stage, or restores them
pub unsafe fn mpScriptStageIsolationSet(lines: i32, joints: i32, isolate: i32) -> i32 {
    if isolate == 0 {
        // A replaced/freed scene cannot be restored through stale heap pointers.
        if mpScriptStageIsolationActive() != 0 {
            for i in 0..mp_script_isolation.nlines {
                (*groundCollLine.offset(i as isize)).flags = mp_script_isolation.line_flags[i as usize];
            }
            for i in 0..mp_script_isolation.njoints {
                let cj = groundCollJoint.offset(i as isize);
                (*cj).flags = mp_script_isolation.joint_flags[i as usize];
                for k in 0..5usize {
                    (*(*cj).inner).ranges[k] = mp_script_isolation.ranges[i as usize][k];
                }
            }
        }
        mp_script_isolation = core::mem::zeroed();
        return 1;
    }
    if !mp_script_isolation.map.is_null() {
        return mpScriptStageIsolationActive();
    }
    if mpLib_804D64B4.is_null()
        || groundCollLine.is_null()
        || groundCollJoint.is_null()
        || lines <= 0
        || joints <= 0
        || lines > 1536
        || joints > 256
        || lines > (*mpLib_804D64B4).line_count
        || joints > (*mpLib_804D64B4).joint_count
    {
        return 0;
    }
    // Validate every range before writing any state.
    for i in 0..joints {
        let cj = groundCollJoint.offset(i as isize);
        if (*cj).inner != (*mpLib_804D64B4).joints.offset(i as isize) {
            return 0;
        }
        for k in 0..5usize {
            let r = &(*(*cj).inner).ranges[k];
            if (r.count as i32) < 0 || (r.count != 0 && ((r.start as i32) < 0 || (r.start as i32) + (r.count as i32) > lines)) {
                return 0;
            }
        }
    }
    mp_script_isolation.map = mpLib_804D64B4;
    mp_script_isolation.lines = groundCollLine;
    mp_script_isolation.joints = groundCollJoint;
    mp_script_isolation.nlines = lines;
    mp_script_isolation.njoints = joints;
    for i in 0..lines {
        let l = groundCollLine.offset(i as isize);
        mp_script_isolation.line_flags[i as usize] = (*l).flags;
        (*l).flags &= !(LINE_FLAG_ENABLED as u32);
    }
    for i in 0..joints {
        let cj = groundCollJoint.offset(i as isize);
        mp_script_isolation.joint_flags[i as usize] = (*cj).flags;
        for k in 0..5usize {
            mp_script_isolation.ranges[i as usize][k] = (*(*cj).inner).ranges[k];
            (*(*cj).inner).ranges[k].count = 0;
        }
    }
    1
}
