//! The C build's recorded audio calls for a scripted VS match (Falcon v Fox, Final Destination,
//! tools/rust-port/golden/pad_fixed.lua, MELEE_TURBO) replayed into the port: the mix must equal
//! the C build's MELEE_AUDIO_DUMP capture (fingerprint recorded from it) and every voice handle
//! must equal the one the C returned. Needs the user's disc ($GW_ISO_VANILLA); skipped without.

use melee_audio::replay::{boot_vs, mix_hash, replay};

const C_MIX_HASH: u64 = 0xf4956676e79a4037;
const C_SUB_FRAMES: u32 = 6106;

#[test]
fn scripted_match_mix_equals_c() {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    };
    let trace = include_str!("data/vs_fd_falcon_fox.trace");
    let mut s = boot_vs(&iso, &[0, 2], 32, melee_types::Gr_Kind_Last as i32).unwrap();
    let (mix, st) = replay(&mut s, trace, C_SUB_FRAMES);
    assert_eq!(st.handles_different, 0);
    assert_eq!(st.handles_equal, 176);
    assert_eq!(mix.len(), C_SUB_FRAMES as usize * 320);
    assert_eq!(mix_hash(&mix), C_MIX_HASH);
}
