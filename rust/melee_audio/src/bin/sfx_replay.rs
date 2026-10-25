//! Replays a C build's recorded audio calls (`MELEE_AUDIO_TRACE`, pc/platform/shim_ax.c) into the
//! ported sound stack and writes the 32 kHz stereo mix, for a sample-level comparison with the C
//! build's own `MELEE_AUDIO_DUMP` of the same run.
//!
//!   sfx_replay --iso DISC.iso --trace c.trace --out replay.wav [--ckinds 0,2] [--stkind 32]
//!       [--grkind N] [--bare] [--check c.wav]
//!   sfx_replay --iso DISC.iso --find-fid N        (which SSM holds sound N)
//!
//! `--check` compares the mix with the C capture sample by sample and exits 1 on any difference.

use melee_audio::replay::{boot_vs, mix_hash, replay};
use melee_audio::sound::{DiscFs, Sound};

fn wav_bytes(samples: &[i16]) -> Vec<u8> {
    let bytes = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + bytes as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + bytes).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&32000u32.to_le_bytes());
    b.extend_from_slice(&(32000u32 * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&bytes.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

fn read_wav(p: &str) -> Vec<i16> {
    let b = std::fs::read(p).unwrap_or_default();
    b.get(44..).unwrap_or(&[]).chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let iso = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()).expect("--iso or GW_ISO_VANILLA");
    if let Some(fid) = get("--find-fid").and_then(|x| x.parse::<i32>().ok()) {
        let mut s = Sound::new();
        s.boot(Box::new(DiscFs::open(&iso).expect("disc")), melee_audio::lbaudio::boot_bank_sizes()).expect("boot");
        for f in melee_audio::lbaudio::SSM_FILES {
            if s.load_ssm(&format!("/audio/us/{f}"), 2).is_ok() && s.has_sfx(fid) {
                println!("sound {fid} is in {f}");
            }
            s.sfx_unload_bank(2);
        }
        return;
    }
    let trace = std::fs::read_to_string(get("--trace").expect("--trace")).expect("trace");
    let out = get("--out").expect("--out");
    let ckinds: Vec<i32> = get("--ckinds").unwrap_or_else(|| "0,2".into()).split(',').filter_map(|x| x.parse().ok()).collect();
    let stkind: i32 = get("--stkind").and_then(|x| x.parse().ok()).unwrap_or(32);
    let grkind: i32 = get("--grkind").and_then(|x| x.parse().ok()).unwrap_or(melee_types::Gr_Kind_Last as i32);
    let mut s = if args.iter().any(|a| a == "--bare") {
        melee_audio::replay::boot_bare(&iso).expect("boot")
    } else {
        boot_vs(&iso, &ckinds, stkind, grkind).expect("boot")
    };
    let check = get("--check").map(|p| (read_wav(&p), p));
    let min = check.as_ref().map(|(c, _)| (c.len() / 320) as u32).unwrap_or(0);
    let (mix, st) = replay(&mut s, &trace, min);
    std::fs::write(&out, wav_bytes(&mix)).expect("write wav");
    eprintln!(
        "sfx_replay: {} calls, handles {} equal / {} different; {} sub-frames, mix hash {:016x} -> {out}",
        st.calls,
        st.handles_equal,
        st.handles_different,
        st.sub_frames,
        mix_hash(&mix)
    );
    if let Some((c, p)) = check {
        let n = c.len().min(mix.len());
        let (mut diff_sub, mut first, mut maxd) = (0usize, None, 0i32);
        for k in 0..n / 320 {
            let mut bad = false;
            for i in k * 320..(k + 1) * 320 {
                let d = (c[i] as i32 - mix[i] as i32).abs();
                if d != 0 {
                    bad = true;
                    maxd = maxd.max(d);
                }
            }
            if bad {
                diff_sub += 1;
                first.get_or_insert(k);
            }
        }
        println!(
            "sfx_replay: compared {} sub-frames ({} samples) against {p}: {diff_sub} differ, first {first:?}, max |diff| {maxd}, C mix hash {:016x}",
            n / 320,
            n,
            mix_hash(&c)
        );
        if diff_sub != 0 || c.len() != mix.len() {
            std::process::exit(1);
        }
    }
}
