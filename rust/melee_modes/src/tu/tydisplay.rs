//! tydisplay.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use melee_core::fp::fmadds;

// ---- folded from hud.rs (consolidation steps 21/22) ----
/// tydisplay.c `_tyDisplay_80318CB4_calc_dist_sq_fp`: `dx*dx + dz*dz`, fused as
/// fmuladd(dx, dx, dz*dz).
pub fn _tyDisplay_80318CB4_calc_dist_sq_fp(dz: f32, dx: f32) -> f32 {
    melee_core::ct!("_tyDisplay_80318CB4_calc_dist_sq_fp");
    fmadds(dx, dx, dz * dz)
}

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// tydisplay.c `tyDisplay_SetGridSize_fp`: grow x75/x76 (columns capped at 0x14) until they
/// cover `x08` trophies; kinds 0 and 1 then add a border row and column.
pub fn tyDisplay_SetGridSize_fp(cfg: &mut mt::TyDspConfig, grid: &mt::TyDspGrid) {
    melee_core::ct!("tyDisplay_SetGridSize_fp");
    let border = match grid.x00 {
        0 | 1 => true,
        2 | 3 => false,
        _ => return,
    };
    let count = cfg.x08;
    let mut toggle = 0;
    cfg.x75 = 1;
    cfg.x76 = 1;
    while (cfg.x75 as i8 as i32) * (cfg.x76 as i8 as i32) < count {
        toggle ^= 1;
        if toggle != 0 && (cfg.x75 as i8) < 0x14 {
            cfg.x75 = cfg.x75.wrapping_add(1);
        } else {
            cfg.x76 = cfg.x76.wrapping_add(1);
        }
    }
    if border {
        cfg.x75 = cfg.x75.wrapping_add(1);
        cfg.x76 = cfg.x76.wrapping_add(1);
    }
}
