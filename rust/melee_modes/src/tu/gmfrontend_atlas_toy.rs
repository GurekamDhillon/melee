//! src/melee/gm/gmfrontend_atlas_toy.inc (included by gmfrontend.c): the part gm_1A3F.c reaches,
//! `Fad_ToyWindowKnown`. The three framed-scene windows (x, y, w, h on the 640x480 canvas) are
//! unmeasured in the source (all zero), so no scene has a window and the policy mask stays empty.
//! Checked against the C text itself: `gen/cref/cut/atlas_toy.c`, `tests/cut_atlas_toy.rs`.
#![allow(non_snake_case, non_upper_case_globals)]

use melee_types::*;

static FAT_GALLERY_WIN: [i32; 4] = [0, 0, 0, 0];
static FAT_LOTTERY_WIN: [i32; 4] = [0, 0, 0, 0];
static FAT_COLLECTION_WIN: [i32; 4] = [0, 0, 0, 0];

fn fat_window(kind: u8) -> Option<&'static [i32; 4]> {
    match kind as GameSceneKind {
        GS_TOY_GALLERY => Some(&FAT_GALLERY_WIN),
        GS_TOY_LOTTERY => Some(&FAT_LOTTERY_WIN),
        GS_TOY_COLLECTION => Some(&FAT_COLLECTION_WIN),
        _ => None,
    }
}

/// gm_801A4014 asks before the scene's on_enter: is there a window to frame?
pub fn Fad_ToyWindowKnown(kind: u8) -> i32 {
    match fat_window(kind) {
        Some(w) => (w[2] > 0 && w[3] > 0) as i32,
        None => 0,
    }
}
