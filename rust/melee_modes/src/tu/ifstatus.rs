//! ifstatus.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;

// ---- folded from hud_ptr.rs (consolidation steps 21/22) ----
/// ifstatus.c `ifStatus_GetStoredTranslationX_fp`.
pub fn ifStatus_GetStoredTranslationX_fp(state: &mt::IfDamageState, index: i32) -> f32 {
    melee_core::ct!("ifStatus_GetStoredTranslationX_fp");
    state.translation_x[index as usize]
}
