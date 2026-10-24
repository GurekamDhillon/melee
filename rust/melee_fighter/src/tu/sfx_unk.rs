//! Native port of src/melee/sfx/sfx_unk.c: crowd sound-effect manager setup (creates the
//! per-frame proc gobj and resets the `crowdsfx` state). The state, `fn_803219AC` and
//! `gCrowdConfig` come from [`crate::tu::crowdsfx`] / [`crate::tu::fighter`].
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, unused_unsafe)]

use super::crowdsfx::{self, ext, fn_803219AC};
use crate::types::*;

pub unsafe fn sfx_setupCrowdSFX() {
    let gobj = ext::GObj_Create(0x16, 0x17, 0);
    ext::HSD_GObj_SetupProc(gobj, Some(fn_803219AC), 0x13);
    crowdsfx::crowdsfx_ptr = &raw mut crowdsfx::crowdsfx;
    sfx_initCrowdSFX(crowdsfx::crowdsfx_ptr);
}

pub unsafe fn sfx_initCrowdSFX(s: *mut CrowdSFX_UnkStruct) {
    let cfg = &*(crate::tu::fighter::gCrowdConfig as *const CrowdConfig);
    let s = &mut *s;
    s.x0 = 0;
    s.x4 = 0x10000;
    s.x8 = 0.0;
    s.xC = 0;
    s.x10 = cfg.cheer_limit;
    s.x14 = 0x83D60;
    s.x18 = cfg.max_gasp_count;
    s.x1C = 0;
    s.x20 = 0;
    s.x24 = 0;
    s.x2C = -1;
    s.x28 = -1;
}
