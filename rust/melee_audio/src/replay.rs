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
    // Traces that record the HPS start's deferred completions (SHDR / SDATA) replay them in place.
    s.defer_stream = trace.lines().any(|l| l.split_whitespace().nth(1) == Some("SHDR"));
    s.defer_refill = trace.lines().any(|l| l.split_whitespace().nth(1) == Some("HDONE"));
    s.refill_chunked = trace.lines().any(|l| l.split_whitespace().nth(1) == Some("DCHUNK"));
    let dbg = std::env::var("MELEE_REPLAY_DBG").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))
    });
    let mut mix: Vec<i16> = Vec::new();
    let mut st = ReplayStats::default();
    let mut last_handle = i32::MIN;
    // HSD_SynthSFXLoad requests (path, bank), served in order by the DVD queue; the file is
    // in the bank when BANKDONE reports the completion.
    let mut pending: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::new();
    for line in trace.lines() {
        let mut it = line.split_whitespace();
        let (Some(sub), Some(op)) = (it.next().and_then(|x| x.parse::<u32>().ok()), it.next()) else { continue };
        while st.sub_frames < sub {
            if let Some((lo, hi)) = dbg {
                if (lo..=hi).contains(&st.sub_frames) {
                    dump_voices(s, st.sub_frames);
                }
            }
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
            "SHDR" => s.stream_header_done(),
            "SDATA" => s.stream_data_done(),
            "DCHUNK" => s.stream_refill_chunk(n(0).max(0) as usize),
            "HHDR" => s.stream_refill_header_done(),
            "HDONE" => s.stream_refill_done(),
            "DSTOP" => {
                s.stream_stop();
            }
            "DPAUSE" => {
                s.stream_pause();
            }
            "DRESUME" => {
                s.stream_resume();
            }
            "BANKREQ" => pending.push_back((rest.first().copied().unwrap_or("").to_string(), n(1) as usize)),
            "BANKDONE" => match pending.pop_front() {
                Some((path, bank)) => {
                    assert_eq!(bank, n(1) as usize, "BANKDONE for another bank than the oldest request ({path})");
                    s.load_ssm(&path, bank).unwrap_or_else(|e| panic!("replay: {e}"));
                }
                None => eprintln!("replay: BANKDONE without a request"),
            },
            "BANKUNL" => s.sfx_unload_bank(n(0) as usize),
            "BANKDFL" => s.sfx_bank_deflag(n(0) as usize),
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

/// The sound stack as the C build has it at the first game call of a menu-flow capture: the
/// audio engine initialised (banks allocated, aux effects, smash2.sem) but nothing loaded; every
/// bank load, unload, announcer and scene call comes from the trace itself.
pub fn boot_bare(iso: &str) -> Result<Sound, String> {
    let mut s = Sound::new();
    s.boot_engine(Box::new(crate::sound::DiscFs::open(iso)?), crate::lbaudio::boot_bank_sizes())?;
    Ok(s)
}

/// Debug aid (`MELEE_REPLAY_DBG=lo-hi`): per-voice mixer state before a sub-frame, in the shape of
/// the C shim's `MELEE_AX_DBG` lines.
fn dump_voices(s: &Sound, sf: u32) {
    for v in 0..64 {
        if !s.ax.in_use(v) {
            continue;
        }
        let x = s.ax.voice(v);
        eprintln!(
            "RPDBG sf {sf} v{v} active={} vol={} d={} ratio={} mix={:?} cur={} yn={},{} ps={}",
            x.dec.active as u8, x.volume, x.volume_delta, x.ratio, x.mix, x.dec.cur_addr, x.dec.yn1, x.dec.yn2, x.dec.pred_scale
        );
    }
}
