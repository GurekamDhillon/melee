//! toy.c: ports folded from the hand-written modules (consolidation steps 21/22).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::TyDisplayData;

// ---- folded from hud_r5.rs (consolidation steps 21/22) ----
/// toy.c `_Toy_8030FE48_link_entries_fp`: link the first `entry_count` entries into a ring.
pub fn _Toy_8030FE48_link_entries_fp(data: &mut TyDisplayData, entry_count: i32) {
    melee_core::ct!("_Toy_8030FE48_link_entries_fp");
    let base = data.entries.as_mut_ptr();
    let last = entry_count.wrapping_sub(1);
    let last_entry = base.wrapping_offset(last as isize);
    for i in 0..entry_count {
        let e = &mut data.entries[i as usize];
        e.prev = if i == 0 { last_entry } else { base.wrapping_offset(i as isize - 1) };
        e.next = if i == last { base } else { base.wrapping_offset(i as isize + 1) };
    }
}
