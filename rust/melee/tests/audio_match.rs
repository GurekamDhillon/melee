//! The native VS sim's own sound calls (`melee golden --audio`, no replayed trace) mix to exactly
//! what the C build captured (MELEE_TURBO, MELEE_AUDIO_DUMP) for the same scripted match: the
//! FNV-1a of the s16 stereo mix equals the hash of the C build's WAV (the constants are the ones
//! melee_audio/tests/replay.rs pins for the replayed traces). Needs the user's disc
//! ($GW_ISO_VANILLA); skipped without. Headless: no window, no device, no input.
//!
//! * Falcon v Fox, Final Destination, pad_fixed.lua, 1800 frames: 6106 DSP sub-frames (106 of
//!   scene load before the first logic frame).
//! * Falcon v Fox, FD, idle pads, START at logic frame 300 and again 120 frames later
//!   (pad_pause.lua): the pause sound and ducking, the resume and the indicator's sound. The C run
//!   ends 2440 sub-frames in; the native one is hashed over those.
//! * Falcon v Fox, FD, idle pads, a one-minute time match: the timer callouts (5..1), the clock
//!   running out (every track off, the music fades), TIME!; the C goes on to sudden death (a new
//!   scene), the native run is hashed up to there (12856 sub-frames).
//! * Marth v Sheik, Battlefield, pad_fixed.lua: the native sim runs to frame 162 (the first
//!   smash attack is not ported), 843 sub-frames, 300 of them before the first logic frame.

fn native_mix(scenario: &str, preroll: u32) -> (usize, u64) {
    native_mix_with(scenario, preroll, "1800", &[], usize::MAX)
}

/// `frames` logic frames, extra golden args, and only the first `keep` sub-frames hashed.
fn native_mix_with(scenario: &str, preroll: u32, frames: &str, extra: &[&str], keep: usize) -> (usize, u64) {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        return (0, 0);
    };
    let dir = std::env::temp_dir().join(format!("melee_audio_match_{scenario}"));
    std::fs::create_dir_all(&dir).unwrap();
    let (csv, wav) = (dir.join("g.csv"), dir.join("n.wav"));
    let st = std::process::Command::new(env!("CARGO_BIN_EXE_melee"))
        .args(["golden", "--iso", &iso, "--scenario", scenario, "--frames", frames, "--audio", "--audio-preroll", &preroll.to_string(), "--out"])
        .arg(&csv)
        .args(extra)
        .arg("--audio-wav")
        .arg(&wav)
        .stdin(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let b = std::fs::read(&wav).unwrap();
    let pcm = &b[44..];
    let pcm = &pcm[..pcm.len().min(keep.saturating_mul(640))];
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

#[test]
fn native_pause_resume_mix_equals_c() {
    let (n, h) = native_mix_with("pad_falcon_fox_fd", 29, "582", &["--idle-pads", "--pause", "300:420"], 2440);
    if n == 0 {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    }
    assert_eq!(n, 2440);
    assert_eq!(h, 0x5bbd8efe96e34336);
}

#[test]
fn native_timer_callouts_and_time_up_mix_equals_c() {
    let (n, h) = native_mix_with("pad_falcon_fox_fd", 29, "4600", &["--idle-pads", "--time-secs", "60"], 12856);
    if n == 0 {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    }
    assert_eq!(n, 12856);
    assert_eq!(h, 0x0aa7f776da0d34a7);
}
