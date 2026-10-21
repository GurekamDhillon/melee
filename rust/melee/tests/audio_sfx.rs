//! Sound effects triggered from ported game code reach the ported sound stack: `ft_PlaySFX`
//! (melee_fighter::ft0877::play_sfx) and the voice channel `ft_800881D8` start sounds from the
//! user's disc through `melee_audio::sim`, store real voice handles in the Fighter, and the mix
//! is audible. Needs GW_ISO_VANILLA (skips without it); nothing from the disc is written.
//!
//! The four lbAudioAx bank-table lookups ft_80087d0c makes (800233EC, 80023130, 800230C8,
//! 80023220) belong to group 1 and are not ported; this test stubs them to the vanilla answer
//! for a main.ssm sound (id unchanged, bank 0, no size-variant offset).

use melee_fighter::hooks as h;
use melee_fighter::types::Fighter;

fn sub_frames(n: u32) -> i32 {
    let mut peak = 0;
    for _ in 0..n {
        melee_audio::sim::virtual_frame();
        peak = peak.max(melee_audio::sim::with(|s| s.ax.output().iter().map(|&x| (x as i32).abs()).max().unwrap()).unwrap());
    }
    peak
}

#[test]
fn fighter_sfx_play_through_the_sound_stack() {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("audio_sfx: GW_ISO_VANILLA not set, skipped");
        return;
    };
    let mut snd = melee_audio::Sound::new();
    snd.boot(Box::new(melee_audio::sound::DiscFs::open(&iso).unwrap()), [2045824, 0x180000, 0x700000 - 2045824 - 0x180000])
        .unwrap();
    melee_audio::sim::install(snd);
    unsafe {
        h::LBAUDIOAX_800237A8 = Some(melee_audio::sim::lbaudioax_800237a8);
        h::LBAUDIOAX_80023870 = Some(melee_audio::sim::lbaudioax_80023870);
        h::LBAUDIOAX_80024B94 = Some(melee_audio::sim::lbaudioax_80024b94);
        // Test stubs for group 1's tables (see the file comment).
        h::LBAUDIOAX_800233EC = Some(|id| id);
        h::LBAUDIOAX_80023130 = Some(|_| 0);
        h::LBAUDIOAX_800230C8 = Some(|_, _, _| 1);
        h::LBAUDIOAX_80023220 = Some(|_| 0);
    }

    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    f.x2160 = -1;
    melee_fighter::ft0877::play_sfx(&mut f, 0xC1, 0x7F, 0x40);
    let h1 = f.x2160;
    assert!(h1 >= 0, "ft_PlaySFX stored handle {h1}");
    // The first virtual voice is pool slot 95 (the free list is built by prepending 0..95).
    assert_eq!(h1 & 0x7F, 95);
    let peak = sub_frames(6);
    assert!(peak > 1000, "ft_PlaySFX's sound is silent (peak {peak})");
    assert!(melee_audio::sim::sfx_check(h1), "HSD_AudioSFXCheck({h1}) is false while it plays");

    // A voice channel: ft_800881D8 keys channel 0x1E + 2*player off, then starts the sound.
    f.x2144 = -1;
    melee_fighter::ft0881::ft_800881d8(&mut f, 0xAA, 0x7F, 0x40);
    let h2 = f.x2144;
    assert!(h2 >= 0 && h2 != h1, "voice handle {h2}");
    assert_eq!(h2 >> 7, (h1 >> 7) + 1, "vidhigh advances once per start");
    let peak = sub_frames(30);
    assert!(peak > 1000, "voice silent (peak {peak})");

    // Stop it the way the game does (0x83D61 on the channel), then everything decays out.
    melee_fighter::ft0881::ft_800881d8(&mut f, 0x83D61, 0, 0x40);
    sub_frames(200);
    assert!(!melee_audio::sim::sfx_check(h2), "the voice still plays after its key-off");
    assert!(!melee_audio::sim::sfx_check(h1), "ft_PlaySFX's sound never ended");
    melee_audio::sim::uninstall();
}
