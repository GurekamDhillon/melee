//! Theme tokens (lab-ui-spec.md section 7): one token set per theme over the same parts. Every token is a colour
//! (0xRRGGBBAA), a slice id or a number, so a theme switch only changes record colours and slice ids.
//! Ids match `GD_UI_TOK_*` / `GD_UI_THEME_*` in docs/specs/gd-ui-lab-v1.h.

pub const THEME_A: i32 = 0; // Instrument (default)
pub const THEME_C: i32 = 1; // Arcade
pub const THEME_E: i32 = 2; // Brutalist
pub const THEME_G: i32 = 3; // Neon

pub const TOK_PANEL_BG: i32 = 0;
pub const TOK_PANEL_BG_HUD: i32 = 1;
pub const TOK_BORDER: i32 = 2;
pub const TOK_TEXT: i32 = 3;
pub const TOK_MUTED: i32 = 4;
pub const TOK_ACCENT: i32 = 5;
pub const TOK_ACCENT_INK: i32 = 6;
pub const TOK_DIVIDER: i32 = 7;
pub const TOK_KEY_BORDER: i32 = 8;
pub const TOK_KEY_TEXT: i32 = 9;
pub const TOK_P1: i32 = 10;
pub const TOK_P2: i32 = 11;
pub const TOK_CELL_IDLE: i32 = 12;
pub const TOK_CELL_STARTUP: i32 = 13;
pub const TOK_CELL_ACTIVE: i32 = 14;
pub const TOK_CELL_RECOVERY: i32 = 15;
pub const TOK_CELL_STUN: i32 = 16;
pub const TOK_CELL_CAN_ACT: i32 = 17;
/// Slice ids: the panel fill and the panel border at the theme's radius (0: square, draw with RECTs).
pub const TOK_SLICE_FILL: i32 = 18;
pub const TOK_SLICE_BORDER: i32 = 19;
pub const TOK_BORDER_W: i32 = 20;
/// Offset shadow colour (alpha 0: none) and its x / y offset; glow slice (0: none).
pub const TOK_SHADOW: i32 = 21;
pub const TOK_SHADOW_X: i32 = 22;
pub const TOK_SHADOW_Y: i32 = 23;
pub const TOK_GLOW: i32 = 24;
pub const TOK_BASE_SIZE: i32 = 25;
pub const N_TOKENS: usize = 26;

const fn a(rgb: u32, alpha: u32) -> u32 {
    (rgb << 8) | alpha
}
const fn o(rgb: u32) -> u32 {
    (rgb << 8) | 0xFF
}

pub const THEMES: [[u32; N_TOKENS]; 4] = [
    // A Instrument: rgba(8,11,17,0.85) (HUD 0.82), 1px #2b3546, radius 4
    [a(0x080B11, 217), a(0x080B11, 209), o(0x2B3546), o(0xE8EDF5), o(0x8D9AB0), o(0x7EE0C3), o(0x06120E), o(0x222B39), o(0x3A475C), o(0xC7D0DE), o(0xC9504A), o(0x4A78C9),
     o(0x2A3344), o(0xE0B84A), o(0xE0564E), o(0x4A7FE0), o(0x9A6EE0), o(0x7EE0C3), 1, 4, 1, 0, 0, 0, 0, 13],
    // C Arcade: white, 3px #241d4a, radius 14, shadow 0 4px #241d4a
    [o(0xFFFFFF), o(0xFFFFFF), o(0x241D4A), o(0x241D4A), o(0x6B63A8), o(0x3DDC97), o(0x241D4A), o(0xE6E1F7), o(0x241D4A), o(0x241D4A), o(0xFF5D73), o(0x4FB6FF),
     o(0xE6E1F7), o(0x3DDC97), o(0xFF5D73), o(0x4FB6FF), o(0xB18CFF), o(0x3DDC97), 3, 6, 3, o(0x241D4A), 0, 4, 0, 13],
    // E Brutalist: white, 3px black, radius 0, shadow 6px 6px black
    [o(0xFFFFFF), o(0xFFFFFF), o(0x000000), o(0x000000), o(0x444444), o(0xFFE500), o(0x000000), o(0xDDDDDD), o(0x000000), o(0x000000), o(0xFF2E2E), o(0x2E5BFF),
     o(0xEEEEEE), o(0x000000), o(0xFF2E2E), o(0x777777), o(0xFFE500), o(0xFFE500), 0, 0, 3, o(0x000000), 6, 6, 0, 13],
    // G Neon: rgba(20,10,40,0.85), 1px #ff3ea5, radius 6, glow rgba(255,62,165,0.45)
    [a(0x140A28, 217), a(0x140A28, 209), o(0xFF3EA5), o(0xF6E9FF), o(0xB59AD6), o(0x00F0FF), o(0x0B0716), o(0x2A1A48), o(0xFF3EA5), o(0xF6E9FF), o(0xFF3EA5), o(0x00F0FF),
     o(0x231638), o(0xFFD23F), o(0xFF3EA5), o(0x00F0FF), o(0x9B5CFF), o(0x00F0FF), 2, 5, 1, 0, 0, 0, 7, 13],
];

/// A token of a theme (0 for an unknown id).
pub fn token(theme: i32, tok: i32) -> u32 {
    let t = THEMES.get(theme as usize).unwrap_or(&THEMES[0]);
    t.get(tok as usize).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn theme_a_values() {
        assert_eq!(token(THEME_A, TOK_ACCENT), 0x7EE0C3FF);
        assert_eq!(token(THEME_A, TOK_PANEL_BG) & 0xFF, 217);
        assert_eq!(token(THEME_G, TOK_BORDER), 0xFF3EA5FF);
        assert_eq!(token(9, TOK_TEXT), token(THEME_A, TOK_TEXT));
    }
}
