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

/// One recorded scenario: boot the VS scene for `ckinds` / `stkind` / `grkind`, replay, compare.
fn check(trace: &str, ckinds: &[i32], stkind: i32, grkind: i32, sub_frames: u32, hash: u64, starts: u32) {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    };
    let mut s = boot_vs(&iso, ckinds, stkind, grkind).unwrap();
    let (mix, st) = replay(&mut s, trace, sub_frames);
    assert_eq!(st.handles_different, 0);
    assert_eq!(st.handles_equal, starts);
    assert_eq!(mix.len(), sub_frames as usize * 320);
    assert_eq!(mix_hash(&mix), hash);
}

/// CPU Fox v Falco, Dreamland (pad_idle.lua, 3600 frames, 66 s): 670 voices, KOs, reverb.
#[test]
fn cpu_fox_falco_dreamland_mix_equals_c() {
    check(include_str!("data/cpu_fox_falco_dl.trace"), &[2, 20], 17, 13, 13283, 0x4e6630b75131f376, 670);
}

/// CPU Peach v Samus, Yoshi's Story, items on: 2075 VOL / PAN updates (positional item and
/// fighter sounds), 11 key-offs.
#[test]
fn cpu_peach_samus_ys_items_mix_equals_c() {
    check(include_str!("data/cpu_peach_samus_ys.trace"), &[12, 16], 8, 10, 12106, 0x6300687c704396b5, 652);
}
