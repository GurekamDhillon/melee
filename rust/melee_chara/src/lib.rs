//! Fighter characters (ft/kinds/* except ftCommon).
//!
//! Port group 4 (docs/rust-port/inventory.md). Raw c2rust output for this group's TUs is
//! written to `src/raw/` by `tools/rust-port/transpile.sh`; it is reference input for the
//! hand port and is deliberately not declared as a module here.
//! Status and blockers: docs/rust-port/group-4 fighter characters.md.
#![allow(dead_code)]

pub use melee_core;

/// Native structs for every record in `src/melee/ft/kinds/*/types.h`: the single generated set in
/// `melee_types` (GC layout on every record), re-exported under group 4's old path.
pub mod types_gen {
    pub use melee_types::*;
}
/// Hand ports of character functions, one module per C TU (verified: tests/chars_ref.rs).
pub mod chars;
/// Pure (Fighter-independent) helpers from the character TUs, ported by hand.
pub mod pure;
#[cfg(test)]
#[allow(clippy::all)]
mod pure_ref;

#[cfg(test)]
mod layout_tests {
    use crate::types_gen::*;
    use melee_core::gc::GcLayout;

    fn in_bounds<T: GcLayout>() {
        for f in T::GC_FIELDS {
            assert!(f.offset + f.size <= T::GC_SIZE, "{}.{} past sizeof", T::GC_NAME, f.name);
        }
    }

    #[test]
    fn fox_layout_matches_header_comments() {
        // types.h comments give Fighter-relative offsets (motion vars start at Fighter+0x2340).
        assert_eq!(ftFoxSpecialS::GC_SIZE, 0x48);
        assert_eq!(ftFoxSpecialS::gc_field("blendFrames").unwrap().offset + 0x2340, 0x2374);
        assert_eq!(ftFoxSpecialS::gc_field("ghostGObj").unwrap().offset + 0x2340, 0x2384);
        assert_eq!(ftFoxSpecialLw::gc_field("gravityDelay").unwrap().offset + 0x2340, 0x234C);
        assert_eq!(ftFox_DatAttrs::gc_field_at(4).unwrap().name, "x4_FOX_BLASTER_UNK2");
        in_bounds::<ftFoxSpecialS>();
        in_bounds::<ftFox_DatAttrs>();
        in_bounds::<ftFox_MotionVars>();
    }

    #[test]
    fn every_gc_table_is_in_bounds() {
        let mut kinds = 0;
        for &(name, size, _align, fields) in GC_RECORDS {
            for f in fields {
                assert!(f.offset + f.size <= size, "{name}.{} past sizeof", f.name);
            }
            kinds += name.starts_with("ft") as usize;
        }
        assert!(kinds > 150, "character records missing from melee_types ({kinds})");
    }
}
