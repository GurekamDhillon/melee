//! Sound effects triggered from ported game code reach the ported sound stack: `ft_PlaySFX`
//! (melee_fighter::ft0877::play_sfx) and the voice channel `ft_800881D8` start sounds from the
//! user's disc through `melee_audio::sim`, store real voice handles in the Fighter, and the mix
//! is audible. Needs GW_ISO_VANILLA (skips without it); nothing from the disc is written.
//!
//! The four lbAudioAx bank-table lookups ft_80087d0c makes (800233EC, 80023130, 800230C8,
//! 80023220) are melee_audio::lbaudio's ports. The second half loads a match's banks the way
//! lbAudioAx_8002785C does (Captain, Fox, Final Destination), plays a captain.ssm sound and the
//! stage BGM, and checks both are audible.

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
    snd.boot(Box::new(melee_audio::sound::DiscFs::open(&iso).unwrap()), melee_audio::lbaudio::boot_bank_sizes())
        .unwrap();
    melee_audio::sim::install(snd);
    unsafe {
        h::LBAUDIOAX_800237A8 = Some(melee_audio::sim::lbaudioax_800237a8);
        h::LBAUDIOAX_80023870 = Some(melee_audio::sim::lbaudioax_80023870);
        h::LBAUDIOAX_80024B94 = Some(melee_audio::sim::lbaudioax_80024b94);
        h::LBAUDIOAX_800233EC = Some(melee_audio::sim::lbaudioax_800233ec);
        h::LBAUDIOAX_80023130 = Some(melee_audio::lbaudio::lb_80023130);
        h::LBAUDIOAX_800230C8 = Some(melee_audio::sim::lbaudioax_800230c8);
        h::LBAUDIOAX_80023220 = Some(melee_audio::lbaudio::lb_80023220);
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

    // A match's banks: captain.ssm, fox.ssm and last.ssm (FD, GrKind 37) committed into bank 2.
    melee_audio::sim::load_match_banks(&[0, 2], 0x20, 37);
    let st = melee_audio::sim::with(|s| s.lb.state).unwrap();
    for b in [6usize, 11, 0x35] {
        assert_eq!(st[b], 2, "bank {} not loaded", melee_audio::lbaudio::SSM_FILES[b]);
    }
    assert_eq!(st.iter().filter(|&&x| x == 2).count(), 3);
    // A captain.ssm sound (its bank's first id), through the same path ft_PlaySFX takes.
    f.x2160 = -1;
    melee_fighter::ft0877::play_sfx(&mut f, melee_audio::lbaudio::BANK_RANGE[6][0], 0x7F, 0x40);
    assert!(f.x2160 >= 0, "captain sfx handle {}", f.x2160);
    let peak = sub_frames(30);
    assert!(peak > 1000, "captain.ssm sound silent (peak {peak})");
    sub_frames(400);
    // The FD BGM (Ground_801C24F8's pick in vs_match is 78, sp_end.hps).
    assert_eq!(melee_audio::sim::play_bgm_id(78), 0);
    assert_eq!(melee_audio::sim::play_bgm_id(78), 1, "a playing BGM restarted");
    let peak = sub_frames(300);
    assert!(peak > 1000, "BGM silent (peak {peak})");
    melee_audio::sim::uninstall();
}
