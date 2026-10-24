//! Replay of a C build's recorded audio calls (`MELEE_AUDIO_TRACE`, `pc/platform/shim_ax.c`) into
//! the ported sound stack, to compare the mix with the C build's `MELEE_AUDIO_DUMP` of the same
//! run. A trace line is `<sub-frame> OP args`: the call (axdriver.c / synth.c) and the number of
//! 5 ms DSP callbacks that had run when it was made. `STARTRET` lines carry the voice handle the
//! C returned for the preceding `START`.

use crate::sound::Sound;

#[derive(Default, Debug, Clone, Copy)]
pub struct ReplayStats {
    pub calls: u32,
    pub handles_equal: u32,
    pub handles_different: u32,
    pub sub_frames: u32,
}

/// FNV-1a 64 over the little-endian bytes of the samples (the proof's compact fingerprint).
pub fn mix_hash(samples: &[i16]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for s in samples {
        for b in s.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
    }
    h
}

/// Apply the trace to a booted `Sound` (scene init, announcer and match banks already loaded),
/// returning the interleaved s16 stereo mix, at least `min_sub_frames` long.
pub fn replay(s: &mut Sound, trace: &str, min_sub_frames: u32) -> (Vec<i16>, ReplayStats) {
    let mut mix: Vec<i16> = Vec::new();
    let mut st = ReplayStats::default();
    let mut last_handle = i32::MIN;
    for line in trace.lines() {
        let mut it = line.split_whitespace();
        let (Some(sub), Some(op)) = (it.next().and_then(|x| x.parse::<u32>().ok()), it.next()) else { continue };
        while st.sub_frames < sub {
            mix.extend_from_slice(&s.sub_frame());
            st.sub_frames += 1;
        }
        let rest: Vec<&str> = it.collect();
        let n = |i: usize| rest.get(i).and_then(|x| x.parse::<i64>().ok()).unwrap_or(0) as i32;
        st.calls += 1;
        match op {
            "START" => last_handle = s.audio_sfx_start_param(n(0), n(1) as u8, n(2) as u8, n(3), n(4)),
            "STARTRET" => {
                if last_handle == n(0) {
                    st.handles_equal += 1;
                } else {
                    st.handles_different += 1;
                }
            }
            "KO" => {
                s.audio_sfx_key_off(n(0));
            }
            "KOALL" => s.audio_sfx_key_off_all(),
            "KOTRK" => s.audio_sfx_key_off_track(n(0)),
            "PAN" => {
                s.audio_sfx_set_pan(n(0), n(1) as u8);
            }
            "VOL" => {
                s.audio_sfx_set_volume_ex(n(0), n(1) as u8);
            }
            "PITCH" => {
                s.audio_sfx_set_pitch_fid(n(0), n(1) as i16);
            }
            "MIX" => {
                s.audio_sfx_set_mix(n(0), n(1), n(2) as u8);
            }
            "MIXGRP" => {
                s.audio_sfx_set_mix_group(n(0), n(1), n(2) as i8);
            }
            "PAUSE" => {
                s.pstream_pause_ch(n(0));
            }
            "RESUME" => {
                s.pstream_resume_ch(n(0));
            }
            "PSTART" => {
                s.pstream_start_ch_param(rest.first().copied().unwrap_or(""), n(1) as u8, n(2));
            }
            "UAV" => s.sfx_update_all_volume(n(0), n(1) as u16, n(2) as usize),
            "SSV" => s.stream_set_volume(f32::from_bits(n(0) as u32)),
            other => eprintln!("replay: unknown op {other}"),
        }
    }
    while st.sub_frames < min_sub_frames {
        mix.extend_from_slice(&s.sub_frame());
        st.sub_frames += 1;
    }
    (mix, st)
}

/// The sound stack as the VS scene leaves it before the first call of the match: boot, scene
/// init, the announcer bank, the players' and the stage's banks.
pub fn boot_vs(iso: &str, ckinds: &[i32], stkind: i32, grkind: i32) -> Result<Sound, String> {
    let mut s = Sound::new();
    s.boot(Box::new(crate::sound::DiscFs::open(iso)?), crate::lbaudio::boot_bank_sizes())?;
    s.lb_audio_ax_80028690();
    s.lb_load_announcer();
    s.lb_load_match_banks(ckinds, stkind, grkind);
    Ok(s)
}
