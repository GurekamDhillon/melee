//! Jigglypuff (src/melee/ft/kinds/ftPurin).

/// The `&mut Fighter` entry points of the old hand-written module (tests/chars_dispatch.rs); the bodies are the
/// C-signature ports in `tu::ftpurinspecialn`.
pub mod ftpurinspecialn {
    use melee_types::Fighter;
    macro_rules! fwd {
        ($($n:ident),*) => {$(
            pub fn $n(fp: &mut Fighter) {
                crate::tu::ftpurinspecialn::$n(fp.gobj)
            }
        )*};
    }
    fwd!(
        ftPr_SpecialAirNChargeFull_Phys, ftPr_SpecialAirNChargeLoop_Phys, ftPr_SpecialAirNEnd_Phys, ftPr_SpecialAirNStart_Phys,
        ftPr_SpecialNEnd_Phys, ftPr_SpecialNHit_Phys, ftPr_SpecialNStart_Phys
    );
}
