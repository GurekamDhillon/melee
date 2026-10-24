//! The native VS sim's own sound calls (`melee golden --audio`, no replayed trace) mix to exactly
//! what the C build captured for the same scripted match (Falcon v Fox, Final Destination,
//! tools/rust-port/golden/pad_fixed.lua, MELEE_TURBO, 1800 frames): 6106 DSP sub-frames, FNV-1a
//! of the s16 stereo mix equal to the hash of the C build's MELEE_AUDIO_DUMP (the same constant
//! melee_audio/tests/replay.rs pins for the replayed trace). Needs the user's disc
//! ($GW_ISO_VANILLA); skipped without. Headless: no window, no device, no input.

const C_MIX_HASH: u64 = 0xf4956676e79a4037;
const C_SUB_FRAMES: usize = 6106;

#[test]
fn native_match_mix_equals_c() {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("skipped: no GW_ISO_VANILLA");
        return;
    };
    let dir = std::env::temp_dir().join("melee_audio_match_test");
    std::fs::create_dir_all(&dir).unwrap();
    let (csv, wav) = (dir.join("g.csv"), dir.join("n.wav"));
    let st = std::process::Command::new(env!("CARGO_BIN_EXE_melee"))
        .args(["golden", "--iso", &iso, "--frames", "1800", "--audio", "--out"])
        .arg(&csv)
        .arg("--audio-wav")
        .arg(&wav)
        .stdin(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(st.success());
    let b = std::fs::read(&wav).unwrap();
    let pcm = &b[44..];
    assert_eq!(pcm.len(), C_SUB_FRAMES * 320 * 2);
    let mut h = 0xcbf29ce484222325u64;
    for &x in pcm {
        h = (h ^ x as u64).wrapping_mul(0x100000001b3);
    }
    assert_eq!(h, C_MIX_HASH);
}
