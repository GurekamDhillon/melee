//! `sislib_font.c` is data-only (zero functions).
//! The 287-glyph atlas is disc-derived and must be supplied by the asset loader.
//! No font bytes are embedded in the source port.
#![allow(non_upper_case_globals)]
use melee_types::TextGlyphTexture;
pub const HSD_SISLIB_FONT_GLYPHS: usize = 287;
pub static mut HSD_SisLib_FontAtlas: *const TextGlyphTexture = core::ptr::null();
