//! Native map-collision types (src/melee/mp/types.h, forward.h).
//!
//! Little-endian, native-pointer structs. Each carries a [`melee_core::gc::GcLayout`] table
//! recording the GameCube offsets (taken from the `/* +XX */` annotations and ASSERT_SIZEs in
//! types.h; TODO: regenerate from the clang ppc32 record-layout dump once that tool exists).
//!
//! Guest pointers that index DAT arrays become indices (`CollLine::x0`); pointers to engine
//! objects owned by other groups stay raw `*mut c_void` until those types exist.

use core::ffi::c_void;

use crate::vec::Vec2;

pub const COLL_LINE_FLOOR: u32 = 1 << 0;
pub const COLL_LINE_CEILING: u32 = 1 << 1;
pub const COLL_LINE_RIGHT_WALL: u32 = 1 << 2;
pub const COLL_LINE_LEFT_WALL: u32 = 1 << 3;

pub const LINE_FLAG_KIND: u32 = 0xF;
pub const LINE_FLAG_EMPTY: u32 = 1 << 7;
pub const LINE_FLAG_PLATFORM: u32 = 1 << 8;
pub const LINE_FLAG_LEDGE: u32 = 1 << 9;
pub const LINE_FLAG_ENABLED: u32 = 1 << 16;
pub const LINE_FLAG_HIDDEN: u32 = 1 << 18;

pub const COLL_JOINT_B8: u32 = 1 << 8;
pub const COLL_JOINT_B9: u32 = 1 << 9;
pub const COLL_JOINT_B10: u32 = 1 << 10;
pub const COLL_JOINT_B11: u32 = 1 << 11;
pub const COLL_JOINT_TOO_FAR: u32 = 1 << 12;
pub const COLL_JOINT_ENABLED: u32 = 1 << 16;
pub const COLL_JOINT_HIDDEN: u32 = 1 << 18;

/// `MapLine` (DAT data).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapLine {
    pub v0_idx: u16,
    pub v1_idx: u16,
    pub prev_id0: i16,
    pub next_id0: i16,
    pub prev_id1: i16,
    pub next_id1: i16,
    pub hi_flags: u16,
    pub lo_flags: u16,
}
melee_core::gc_layout!(MapLine, "MapLine", size 0x10, align 2, [
    v0_idx @ 0x0: 2 "u16",
    v1_idx @ 0x2: 2 "u16",
    prev_id0 @ 0x4: 2 "s16",
    next_id0 @ 0x6: 2 "s16",
    prev_id1 @ 0x8: 2 "s16",
    next_id1 @ 0xA: 2 "s16",
    hi_flags @ 0xC: 2 "u16",
    lo_flags @ 0xE: 2 "u16",
]);

/// `CollLine`. On the GameCube `x0` is a `MapLine*` into the DAT line array; natively it is
/// that array's index.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CollLine {
    pub x0: u32,
    pub flags: u32,
}
melee_core::gc_layout!(CollLine, "CollLine", size 0x8, align 4, [
    x0 @ 0x0: 4 "MapLine*",
    flags @ 0x4: 4 "u32",
]);

/// `CollVtx`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CollVtx {
    pub x0: f32,
    pub x4: f32,
    pub pos: Vec2,
    pub x10: f32,
    pub x14: f32,
}
melee_core::gc_layout!(CollVtx, "CollVtx", size 0x18, align 4, [
    x0 @ 0x0: 4 "f32",
    x4 @ 0x4: 4 "f32",
    pos @ 0x8: 8 "Vec2",
    x10 @ 0x10: 4 "float",
    x14 @ 0x14: 4 "float",
]);

pub const MAP_LINE_GROUP_FLOOR: usize = 0;
pub const MAP_LINE_GROUP_CEILING: usize = 1;
pub const MAP_LINE_GROUP_RIGHT_WALL: usize = 2;
pub const MAP_LINE_GROUP_LEFT_WALL: usize = 3;
pub const MAP_LINE_GROUP_DYNAMIC: usize = 4;
pub const MAP_LINE_GROUP_COUNT: usize = 5;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapLineRange {
    pub start: i16,
    pub count: i16,
}
melee_core::gc_layout!(MapLineRange, "MapLineRange", size 0x4, align 2, [
    start @ 0x0: 2 "s16",
    count @ 0x2: 2 "s16",
]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapJoint {
    pub ranges: [MapLineRange; MAP_LINE_GROUP_COUNT],
    pub left_bound: f32,
    pub bottom_bound: f32,
    pub right_bound: f32,
    pub top_bound: f32,
    pub vtx_start: i16,
    pub vtx_count: i16,
}
melee_core::gc_layout!(MapJoint, "MapJoint", size 0x28, align 4, [
    ranges @ 0x0: 20 "MapLineRange[5]",
    left_bound @ 0x14: 4 "float",
    bottom_bound @ 0x18: 4 "float",
    right_bound @ 0x1C: 4 "float",
    top_bound @ 0x20: 4 "float",
    vtx_start @ 0x24: 2 "s16",
    vtx_count @ 0x26: 2 "s16",
]);

/// `mpLib_JointCollisionCallback`: `void (*)(Ground*, int, CollData*, int, mpLib_GroundEnum, float)`.
/// Callbacks into gr/ft/it; stubbed as an opaque fn pointer until group 3/5 types exist.
pub type JointCollisionCallback = Option<unsafe extern "C" fn()>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CollJoint {
    pub next: *mut CollJoint,
    pub inner: *mut MapJoint,
    pub flags: u32,
    pub x_c: i16,
    /// C bitfield `u8 xE : 1` (PPC bit 0 of the byte at +0xE).
    pub x_e: bool,
    pub bounding_min: Vec2,
    pub bounding_max: Vec2,
    /// `HSD_JObj*` (group 1).
    pub x20: *mut c_void,
    pub cb_0: JointCollisionCallback,
    /// `Ground*`.
    pub cb_data_0: *mut c_void,
    pub cb_1: JointCollisionCallback,
    /// `Ground*`.
    pub cb_data_1: *mut c_void,
}
melee_core::gc_layout!(CollJoint, "CollJoint", size 0x34, align 4, [
    next @ 0x0: 4 "CollJoint*",
    inner @ 0x4: 4 "MapJoint*",
    flags @ 0x8: 4 "u32",
    x_c @ 0xC: 2 "s16",
    x_e @ 0xE: 1 "u8" bits 0 + 1,
    bounding_min @ 0x10: 8 "Vec2",
    bounding_max @ 0x18: 8 "Vec2",
    x20 @ 0x20: 4 "HSD_JObj*",
    cb_0 @ 0x24: 4 "mpLib_JointCollisionCallback",
    cb_data_0 @ 0x28: 4 "Ground*",
    cb_1 @ 0x2C: 4 "mpLib_JointCollisionCallback",
    cb_data_1 @ 0x30: 4 "Ground*",
]);

/// `MapCollData` (DAT header). Natively the arrays it points at are owned slices.
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct MapCollData {
    pub verts: Vec<Vec2>,
    pub vert_count: i32,
    pub lines: Vec<MapLine>,
    pub line_count: i32,
    pub ranges: [MapLineRange; MAP_LINE_GROUP_COUNT],
    pub joints: Vec<MapJoint>,
    pub joint_count: i32,
    pub x2c: i32,
}
melee_core::gc_layout!(MapCollData, "MapCollData", size 0x30, align 4, [
    verts @ 0x0: 4 "Vec2*",
    vert_count @ 0x4: 4 "int",
    lines @ 0x8: 4 "MapLine*",
    line_count @ 0xC: 4 "int",
    ranges @ 0x10: 20 "MapLineRange[5]",
    joints @ 0x24: 4 "MapJoint*",
    joint_count @ 0x28: 4 "int",
    x2c @ 0x2C: 4 "int",
]);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MpCollisionBox {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}
melee_core::gc_layout!(MpCollisionBox, "mpCollisionBox", size 0x10, align 4, [
    top @ 0x0: 4 "float",
    bottom @ 0x4: 4 "float",
    left @ 0x8: 4 "float",
    right @ 0xC: 4 "float",
]);
