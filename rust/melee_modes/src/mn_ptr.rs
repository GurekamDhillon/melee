//! Round 3: `src/melee/mn/` leaf functions that take pointers to game structs. Conventions and
//! verification as in `gm_ptr.rs` (`tests/oracle_ptr.rs`).
#![allow(non_snake_case)]

use melee_core::fp::fmadds;
use melee_types as mt;
use mt::{AnimLoopSettings, HSD_Text, Vec as Vec3};

/// mncharsel.c `loadStickValue`.
pub fn loadStickValue(value: &i8) -> f32 {
    *value as f32
}

/// mncharsel.c `mnCharSel_MexPackRows`: spread `n_avail` icons over `rows` rows of capacity
/// `row_cap[r]`, as evenly as the caps allow; `take[r]` gets each row's share.
pub fn mnCharSel_MexPackRows(row_cap: &[i32], rows: i32, n_avail: i32, take: &mut [i32]) -> i32 {
    if rows <= 0 || n_avail <= 0 {
        for t in &mut take[..rows.max(0) as usize] {
            *t = 0;
        }
        return 0;
    }
    let n = rows as usize;
    let mut placed = 0i32;
    for r in 0..n {
        let share = n_avail / rows + if (r as i32) < n_avail % rows { 1 } else { 0 };
        take[r] = if share > row_cap[r] { row_cap[r] } else { share };
        placed = placed.wrapping_add(take[r]);
    }
    while placed < n_avail {
        let mut progress = false;
        let mut r = 0;
        while r < n && placed < n_avail {
            if take[r] < row_cap[r] {
                take[r] += 1;
                placed += 1;
                progress = true;
            }
            r += 1;
        }
        if !progress {
            break;
        }
    }
    placed
}

/// mndiagram.c `mnDiagram_TextSetPos`.
pub fn mnDiagram_TextSetPos(text: &mut HSD_Text, x: f32, y: f32, z: f32) {
    text.pos_x = x;
    text.pos_y = y;
    text.pos_z = z;
}

/// mndiagram3.c `mnDiagram3_GetRowStat`: the stat shown on a row, wrapping at 24 (names) or 21.
pub fn mnDiagram3_GetRowStat(data: &mt::Diagram3, offset: u8, row: u8) -> u8 {
    let limit: i32 = if data.is_name_mode != 0 { 24 } else { 21 };
    let value = offset as i32 + row as i32;
    if value >= limit { (value - limit) as u8 } else { value as u8 }
}

/// mngallery.c `float_order_helper`.
pub fn float_order_helper(sobj: &mut mt::HSD_SObj) {
    sobj.x10 = 96.0;
    sobj.x14 = 72.0;
}

/// mninfo.c `mnInfo_80251AFC_inline_3`.
pub fn mnInfo_80251AFC_inline_3(ids: &[u8], i: i32) -> i32 {
    ids[i as usize] as i32
}

/// mninfo.c `mnInfo_80252720`: reset the info screen's user data.
pub fn mnInfo_80252720(data: &mut mt::MnInfoData) {
    data.scroll_idx = 0;
    data.anim_timer = 10;
    data.description = core::ptr::null_mut();
    data.left_column = [core::ptr::null_mut(); 4];
    data.right_column = [core::ptr::null_mut(); 4];
}

/// mninfobonus.c `textSize`.
pub fn textSize(text: &mut HSD_Text) {
    text.font_size.x = 0.0521;
    text.font_size.y = 0.0521;
}

/// mninfobonus.c `mnInfoBonus_80252ADC_inline`.
pub fn mnInfoBonus_80252ADC_inline(text: &mut HSD_Text) {
    text.x34.x = 0.034;
    text.x34.y = 0.034;
}

/// mnmain.c `GetAnimStartFrame`.
pub fn GetAnimStartFrame(anim_loop: &AnimLoopSettings) -> f32 {
    anim_loop.start_frame
}

/// mnmain.c `GetAnimEndFrame`.
pub fn GetAnimEndFrame(anim_loop: &AnimLoopSettings) -> f32 {
    anim_loop.end_frame
}

/// mnname.c `readNameTerminator` (a `volatile` read).
pub fn readNameTerminator(terminator: &i8) -> i8 {
    // SAFETY: a reference is valid for reads.
    unsafe { core::ptr::read_volatile(terminator) }
}

/// The two bytes of the C's `"　"` (U+3000, the full-width space) as the shipping build
/// compiles them: the source file is UTF-8 and clang's execution charset is UTF-8, so they
/// are E3 80 (the GameCube's Shift-JIS build had 81 40).
pub const FULLWIDTH_SPACE: [u8; 2] = [0xE3, 0x80];

/// mnname.c `checkStringRest`: only full-width spaces up to `terminator`.
pub fn checkStringRest(ptr: &[i8], terminator: i8) -> bool {
    let c = FULLWIDTH_SPACE[0] as i8;
    let mut i = 0;
    while terminator != ptr[i] {
        if c != ptr[i] || FULLWIDTH_SPACE[1] as i8 != ptr[i + 1] {
            return false;
        }
        i += 2;
    }
    true
}

/// mnname.c `mnName_80238C34_inline`.
pub fn mnName_80238C34_inline(anim: &AnimLoopSettings) -> f32 {
    anim.end_frame
}

/// mnnamenew.c `mnNameNew_8023BAA8`: the key the cursor moves to on the name-entry grid
/// (`arg1`: 1 left, 2 right, 4 down, 8 up; keys 0x32..0x39 are the bottom row).
pub fn mnNameNew_8023BAA8(arg0: &mt::NameNewEntry, arg1: i32, arg2: u8) -> i32 {
    let k = arg2 as i32;
    let last = arg0.last_key_sel as i32;
    let by_mode = |mode: u8| match mode {
        0 => Some(0x33),
        1 => Some(0x34),
        2 => Some(0x35),
        _ => None,
    };
    if (0x32..0x3A).contains(&arg2) {
        if arg1 & 1 != 0 {
            if k == 0x39 {
                return (last / 5) * 5 + 4;
            }
            if k > 0x32 {
                return k - 1;
            }
            return 0x38;
        }
        if arg1 & 2 != 0 {
            if k == 0x39 {
                return (last / 5) * 5;
            }
            if k == 0x38 {
                return 0x32;
            }
            return k + 1;
        }
        if arg1 & 4 != 0 {
            if k != 0x38 && k != 0x39 {
                return last % 5;
            }
        } else if arg1 & 8 != 0 && k != 0x38 && k != 0x39 {
            return last % 5 + 0x2D;
        }
    } else {
        if arg1 & 1 != 0 {
            return if k % 5 != 0 { k - 1 } else { 0x39 };
        }
        if arg1 & 2 != 0 {
            return if k % 5 == 4 { 0x39 } else { k + 1 };
        }
        if arg1 & 4 != 0 {
            if k / 5 < 9 {
                return k + 5;
            }
            if let Some(r) = by_mode(arg0.mode) {
                return r;
            }
        } else if arg1 & 8 != 0 {
            if k / 5 != 0 {
                return k - 5;
            }
            if let Some(r) = by_mode(arg0.mode) {
                return r;
            }
        }
    }
    k
}

/// mnnamenew.c `GlyphVariantCount`.
pub fn GlyphVariantCount(count: u16, out: &mut i32) {
    *out = (count & 0xFF) as i32;
}

/// mnsound.c `getVolumePosition`: the slider position for a sound/music mix of -100..100
/// (one fused site).
pub fn getVolumePosition(sound_music_mix: u8, pos_0: &Vec3, pos_1: &Vec3) -> f32 {
    fmadds((sound_music_mix as i8 as i32 + 100) as f32 / 200.0, pos_1.x - pos_0.x, pos_0.x)
}

/// mnstagesw.c `mnStageSw_CountEnabled` (NUM_STAGES = 29 flags from byte 2).
pub fn mnStageSw_CountEnabled(user_data: &[u8]) -> i32 {
    user_data[2..2 + 29].iter().filter(|&&b| b != 0).count() as i32
}

/// mn_22EC.c `mn_8022F410`: step `*x` toward `*target` by `dx`; returns the direction.
pub fn mn_8022F410(x: &mut f32, target: &f32, dx: f32) -> i32 {
    let t = *target;
    if *x == t {
        return 0;
    }
    if *x < t {
        let a = *x + dx;
        *x = if a > t { t } else { a };
        1
    } else {
        let a = *x - dx;
        *x = if a < t { t } else { a };
        -1
    }
}

/// mn_22EC.c `mn_8022F470`: the integer version (`-fwrapv`).
pub fn mn_8022F470(x: &mut i32, target: &i32, dx: i32) -> i32 {
    let t = *target;
    if *x == t {
        return 0;
    }
    if *x < t {
        let a = x.wrapping_add(dx);
        *x = if a > t { t } else { a };
        1
    } else {
        let a = x.wrapping_sub(dx);
        *x = if a < t { t } else { a };
        -1
    }
}
