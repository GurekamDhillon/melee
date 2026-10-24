//! `src/sysdolphin/baselib/gobjgxlink.c`: the GX (render) link lists, sorted by render priority.
//!
//! Native ports on [`GObjSystem`] (`crate::gobj`, oracle `tests/oracle_gobj.rs`); the C names are
//! exposed here. The `HSD_ASSERT`s of the C are `assert!`s in the methods.
#![allow(non_snake_case)]
use crate::gobj::{GObjId, GObjSystem};

/// `GObj_GXReorder(gobj, hiprio_gobj)`.
pub fn GObj_GXReorder<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, hiprio: Option<GObjId>) {
    sys.gx_reorder(gobj, hiprio)
}

/// `GObj_SetupGXLink(gobj, render_cb, gx_link, priority)`.
pub fn GObj_SetupGXLink<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, render_cb: Option<F>, gx_link: u8, priority: u32) {
    sys.setup_gx_link(gobj, render_cb, gx_link, priority)
}

/// `GObj_SetupGXLinkMax(gobj, render_cb, priority)`.
pub fn GObj_SetupGXLinkMax<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, render_cb: Option<F>, priority: u32) {
    sys.setup_gx_link_max(gobj, render_cb, priority)
}

/// `GObj_SetupGXLinkMaxSorted(gobj, render_cb, priority)`.
pub fn GObj_SetupGXLinkMaxSorted<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, render_cb: Option<F>, priority: u32) {
    sys.setup_gx_link_max_sorted(gobj, render_cb, priority)
}

/// `HSD_GObjGXLink_8039084C(gobj)`: unlink from the GX list.
pub fn HSD_GObjGXLink_8039084C<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId) {
    sys.gxlink_8039084c(gobj)
}

/// `HSD_GObjGXLink_80390908(gobj, gx_link, priority)`.
pub fn HSD_GObjGXLink_80390908<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, gx_link: u8, priority: u8) {
    sys.gxlink_80390908(gobj, gx_link, priority)
}

/// `HSD_GObjGXLink_803909D8(gobj, other)`: take `other`'s link and priority, ahead of it.
pub fn HSD_GObjGXLink_803909D8<F: Copy, O: Copy>(sys: &mut GObjSystem<F, O>, gobj: GObjId, other: GObjId) {
    sys.gxlink_803909d8(gobj, other)
}
