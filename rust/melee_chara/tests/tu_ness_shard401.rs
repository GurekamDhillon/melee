//! Smoke tests for the shard 401-408 TUs (ftnanaspecials, ftness, ftnessattack{hi4,lw4,s4},
//! ftnessspecial{hi,lw,n}): everything links, the pure blend factor behaves as the C clamp, and
//! the recording-hook path of ftnessattacks4 runs. One #[test]: hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::tu::{ftnanaspecials, ftness, ftnessattacklw4, ftnessattacks4};
use melee_types::*;

static mut SFX: std::vec::Vec<(i32, u8, u8)> = std::vec::Vec::new();

#[test]
fn smoke() {
    // Pure: 0.2f * (frame - 14) clamped to [0, 1]; 0 below, 1 at/after frame 19.
    for frame in -3..40 {
        let mut v = 0.20000000298023224_f32 * (frame as f32 - 14.0);
        if v >= 1.0 {
            v = 1.0;
        } else if v <= 0.0 {
            v = 0.0;
        }
        assert_eq!(ftnessattacklw4::release_blend(frame).to_bits(), v.to_bits());
    }
    assert_eq!(ftnessattacklw4::release_blend(14), 0.0);
    assert_eq!(ftnessattacklw4::release_blend(19), 1.0);

    // Stateful: the reflect callback plays one SFX through the hook.
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        ftnessattacks4::ext::FT_PLAYSFX = Some(|_, s, v, p| SFX.push((s, v, p)));
        ftnessattacks4::ftNs_AttackS4_OnReflect(gp);
        ftnessattacks4::cb_ftNs_AttackS4_OnReflect(gp);
        assert_eq!(SFX, vec![(0xE0, 0x7F, 0x40), (0xE0, 0x7F, 0x40)]);
    }

    // Data tables / no-op IASA callbacks.
    assert_eq!(ftness::ftNs_Init_DatFilename, "PlNs.dat");
    ftnanaspecials::ftPp_SpecialS_0_IASA(gp);
    ftnanaspecials::ftPp_SpecialS_1_IASA(gp);
}
