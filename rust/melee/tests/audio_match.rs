//! The native VS sim's own sound calls (`melee golden --audio`, no replayed trace) mix to exactly
//! what the C build captured (MELEE_TURBO, MELEE_AUDIO_DUMP) for the same scripted match: the
//! FNV-1a of the s16 stereo mix equals the hash of the C build's WAV (the constants are the ones
//! melee_audio/tests/replay.rs pins for the replayed traces). Needs the user's disc
//! ($GW_ISO_VANILLA); skipped without. Headless: no window, no device, no input.
//!
//! * Falcon v Fox, Final Destination, pad_fixed.lua, 1800 frames: 6106 DSP sub-frames (106 of
//!   scene load before the first logic frame).
//! * Marth v Sheik, Battlefield, pad_fixed.lua: the native sim runs to frame 162 (the first
//!   smash attack is not ported), 843 sub-frames, 300 of them before the first logic frame.

fn native_mix(scenario: &str, preroll: u32) -> (usize, u64) {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        return (0, 0);
    };
    let dir = std::env::temp_dir().join(format!("melee_audio_match_{scenario}"));
    std::fs::create_dir_all(&dir).unwrap();
    let (csv, wav) = (dir.join("g.csv"), dir.join("n.wav"));
    let st = std::process::Command::new(env!("CARGO_BIN_EXE_melee"))
        .args(["golden", "--iso", &iso, "--scenario", scenario, "--frames", "1800", "--audio", "--audio-preroll", &preroll.to_string(), "--out"])
        .arg(&csv)
        .arg("--audio-wav")
        .arg(&wav)
        .stdin(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let b = std::fs::read(&wav).unwrap();
    let pcm = &b[44..];
    let mut h = 0xcbf29ce484222325u64;
    for &x in pcm {
        h = (h ^ x as u64).wrapping_mul(0x100000001b3);
    }
    (pcm.len() / 640, h)
}

#[test]
fn native_falcon_fox_fd_mix_equals_c() {
    let (n, h) = native_mix("pad_falcon_fox_fd", 29);
    if n == 0 {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    }
    assert_eq!(n, 6106);
    assert_eq!(h, 0xf4956676e79a4037);
}

#[test]
fn native_marth_sheik_bf_mix_equals_c() {
    let (n, h) = native_mix("pad_marth_sheik_bf", 87);
    if n == 0 {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    }
    assert_eq!(n, 843);
    assert_eq!(h, 0x4071ad16fd1751fd);
}
