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

/// A scene-spanning capture: the trace carries the audio init, every bank load / unload and the
/// HPS start's deferred completions (BANK*, SHDR / SDATA), so it replays on a bare engine.
fn check_bare(trace: &str, sub_frames: u32, hash: u64, starts: u32) {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    };
    let mut s = melee_audio::replay::boot_bare(&iso).unwrap();
    let (mut mix, st) = replay(&mut s, trace, sub_frames);
    assert_eq!(st.handles_different, 0);
    assert_eq!(st.handles_equal, starts);
    // A capture can end before the trace does (the run was cut at its time limit): compare the
    // part both have.
    assert!(mix.len() >= sub_frames as usize * 320);
    mix.truncate(sub_frames as usize * 320);
    assert_eq!(mix_hash(&mix), hash);
}

/// Opening movie, title screen and the attract demo (pad_menu: START skips the memory card, movie
/// and title): menu sounds, opening.hps, per-scene fighter banks, nr_vs, greatbay.hps.
#[test]
fn title_and_attract_demo_mix_equals_c() {
    check_bare(include_str!("data/menu_title_attract.trace"), 16412, 0x75e116914be22045, 4);
}

/// The character select screen, cursor moved and A / B / START pressed: the menu click sounds.
#[test]
fn css_menu_clicks_mix_equals_c() {
    check_bare(include_str!("data/menu_css_clicks.trace"), 5650, 0xbc03c1b9ce2de179, 13);
}

/// Falcon v Fox on FD, P1 teleported out of the stage with one stock: GAME!, the KO, the results
/// screen with its victory theme.
#[test]
fn stock_out_game_and_results_mix_equals_c() {
    check_bare(include_str!("data/vs_ko_game_results.trace"), 5780, 0x7bf74c18cb10c50f, 18);
}

/// One-minute time match to the end: callouts, TIME!, the sudden-death scene.
#[test]
fn time_up_and_sudden_death_mix_equals_c() {
    check_bare(include_str!("data/vs_timeup_sudden_death.trace"), 18233, 0xbb27f2611d767fb9, 29);
}

/// Title, the memory-card / unlock notices, the retail main menu (cursor up and down, 1P / VS
/// and back): the s_info and menu01 streams, block refills landing chunk by chunk (DCHUNK) while
/// the previous stream fades out.
#[test]
fn frontend_menu_streams_mix_equals_c() {
    check_bare(include_str!("data/menu_frontend.trace"), 4483, 0x7db0156cba82a71d, 17);
}
