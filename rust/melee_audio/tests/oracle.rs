//! Bit-exact check against pc/platform/shim_ax.c compiled as the shipping build compiles shims
//! (oracle/build.sh -> tests/data/ax_oracle.txt):
//!   D/O/E lines: 4000 random decoder states x 64 gw_ax_decode_one calls (every sample + end state);
//!   the rest: a 30000-step random AX API script replayed call for call, comparing every
//!   gw_ax_run_frame's 320 output samples + game-visible VPB fields (FNV-1a 64), full frames every
//!   97th, every call's return value, and the clip counters.

use std::sync::Arc;

use melee_audio::adpcm::Decoder;
use melee_audio::Ax;

const DATA: &str = include_str!("data/ax_oracle.txt");

fn aram_fill(n: usize, seed: u32) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let b = (x >> 24) as u8;
            if i & 7 == 0 {
                b & 0x7F
            } else {
                b
            }
        })
        .collect()
}

fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

#[test]
fn adpcm_decode_matches_c() {
    let aram = aram_fill(4096, 0xA11CE5);
    let mut lines = DATA.lines().peekable();
    let mut cases = 0;
    let mut samples = 0;
    while let Some(l) = lines.next_if(|l| l.starts_with("D ")) {
        let v: Vec<i64> = l[2..].split(' ').map(|t| t.parse().unwrap()).collect();
        let mut d = Decoder::default();
        for i in 0..8 {
            d.coef[i] = [v[i * 2] as i16, v[i * 2 + 1] as i16];
        }
        let r = &v[16..];
        d.pred_scale = r[0] as u8;
        d.yn1 = r[1] as i16;
        d.yn2 = r[2] as i16;
        d.format = r[3] as u16;
        d.cur_addr = r[4] as u32;
        d.end_addr = r[5] as u32;
        d.loop_addr = r[6] as u32;
        d.loop_flag = r[7] as u16;
        d.loop_pred_scale = r[8] as u8;
        d.loop_yn1 = r[9] as i16;
        d.loop_yn2 = r[10] as i16;
        d.active = true;
        let o = lines.next().unwrap();
        let want: Vec<i16> = o[2..].split(' ').map(|t| t.parse().unwrap()).collect();
        let got: Vec<i16> = (0..64).map(|_| d.decode_one(&aram)).collect();
        assert_eq!(got, want, "decode case {cases}: {l}");
        let e = lines.next().unwrap();
        let ge = format!(
            "E {} {} {} {} {} {}",
            d.pred_scale, d.yn1, d.yn2, d.cur_addr, d.active as i32, d.state
        );
        assert_eq!(ge, e, "decode case {cases} end state");
        cases += 1;
        samples += 64;
    }
    assert_eq!(cases, 4000);
    eprintln!("{cases} decoder cases, {samples} samples bit-exact");
}

#[test]
fn mixer_script_matches_c() {
    let aram = aram_fill(1 << 18, 0xBEEF01);
    let clips: Vec<Arc<[i16]>> = (0..4)
        .map(|c: usize| {
            let ch = 1 + (c & 1);
            let frames = 200 + c * 700;
            (0..frames * ch).map(|i| ((((i * 37 + c * 1000) % 4000) * 8) as i32 - 16000) as i16).collect()
        })
        .collect();
    let clip_frames = |c: usize| (200 + c * 700) as i32;
    let mut ax = Ax::new();
    let (mut frames, mut full, mut ops, mut audible) = (0, 0, 0, 0);
    let fx_block_rev = |hex: &str| {
        let mut b = vec![0u8; 0x160];
        b[0x13C..0x13C + 0x18].copy_from_slice(&unhex(hex));
        b
    };
    let fx_block_dly = |hex: &str| {
        let mut b = vec![0u8; 0x60];
        b[0x3C..0x3C + 0x24].copy_from_slice(&unhex(hex));
        b
    };
    for l in DATA.lines() {
        let t: Vec<&str> = l.split(' ').collect();
        let n = |i: usize| -> i64 { t[i].parse().unwrap() };
        let ix = || t[1].parse::<usize>().unwrap();
        match t[0] {
            "D" | "O" | "E" => continue,
            "acq" => {
                let got = ax.acquire_voice(n(1) as u32, 0, n(2) as u32, |_, _| panic!("no callbacks"));
                assert_eq!(got.map_or(-1, |i| i as i64), n(3), "{l}");
            }
            "free" => ax.free_voice(ix()),
            "prio" => ax.set_voice_priority(ix(), n(2) as u32),
            "state" => ax.set_voice_state(ix(), n(2) as u16),
            "mix" => ax.set_voice_mix(ix(), &unhex(t[2])),
            "itdon" => ax.set_voice_itd_on(ix()),
            "itdt" => ax.set_voice_itd_target(ix(), n(2) as u16, n(3) as u16),
            "ve" => ax.set_voice_ve(ix(), &unhex(t[2])),
            "vedelta" => ax.set_voice_ve_delta(ix(), n(2) as i16),
            "addr" => ax.set_voice_addr(ix(), &unhex(t[2])),
            "loop" => ax.set_voice_loop(ix(), n(2) as u16),
            "loopaddr" => ax.set_voice_loop_addr(ix(), n(2) as u32),
            "endaddr" => ax.set_voice_end_addr(ix(), n(2) as u32),
            "curaddr" => ax.set_voice_current_addr(ix(), n(2) as u32),
            "adpcm" => ax.set_voice_adpcm(ix(), &unhex(t[2])),
            "adpcmloop" => ax.set_voice_adpcm_loop(ix(), &unhex(t[2])),
            "src" => ax.set_voice_src(ix(), &unhex(t[2])),
            "srcratio" => ax.set_voice_src_ratio(ix(), f32::from_bits(n(2) as u32)),
            "revinit" => assert_eq!(ax.fx_reverb_std_init(1, &fx_block_rev(t[2])) as i64, n(3), "{l}"),
            "dlyinit" => assert_eq!(ax.fx_delay_init(2, &fx_block_dly(t[2])) as i64, n(3), "{l}"),
            "revshut" => {
                ax.fx_reverb_std_shutdown(n(1) as u32);
            }
            "dlyshut" => {
                ax.fx_delay_shutdown(n(1) as u32);
            }
            "auxa" => ax.register_aux_a(n(1) != 0, n(2) as u32),
            "auxb" => ax.register_aux_b(n(1) != 0, n(2) as u32),
            "clip" => {
                let c = n(1) as usize;
                let r = ax.play_owned_clip(clips[c].clone(), clip_frames(c), 1 + (c as i32 & 1), n(2) as i32, n(3) as u32);
                assert_eq!(r as i64, n(4), "{l}");
            }
            "stopclip" => assert_eq!(ax.stop_clips(&clips[n(1) as usize]) as i64, n(2), "{l}"),
            "stopowned" => assert_eq!(ax.stop_owned_clips(&clips[n(1) as usize], n(2) as u32) as i64, n(3), "{l}"),
            "frame" => {
                let out = *ax.run_frame(&aram);
                let mut h = 0xCBF29CE484222325u64;
                let bytes: Vec<u8> = out.iter().flat_map(|s| s.to_le_bytes()).collect();
                h = fnv(h, &bytes);
                for i in 0..64 {
                    let v = ax.vpb(i);
                    h = fnv(h, &v[0x0C..0x10]);
                    h = fnv(h, &v[0x14..0x1C]);
                    h = fnv(h, &v[0x146..0x148]);
                    h = fnv(h, &v[0x16E..0x170]);
                    h = fnv(h, &v[0x19C..0x19E]);
                    h = fnv(h, &v[0x1B2..0x1B6]);
                }
                if t.len() > 3 {
                    let want: Vec<i16> = t[4..].iter().map(|s| s.parse().unwrap()).collect();
                    assert_eq!(&out[..], &want[..], "frame {} full output", t[1]);
                    full += 1;
                }
                assert_eq!(format!("{h:016x}"), t[2], "frame {} hash (output + VPB writeback)", t[1]);
                if out.iter().any(|&s| s != 0) {
                    audible += 1;
                }
                frames += 1;
                continue;
            }
            "clipstat" => {
                assert_eq!(ax.clip_stat(0) as i64, n(1));
                assert_eq!(ax.clip_stat(1) as i64, n(2));
                continue;
            }
            other => panic!("unknown op {other}"),
        }
        ops += 1;
    }
    assert!(frames > 6000 && full > 60, "{frames} frames");
    assert!(audible * 2 > frames, "only {audible} of {frames} frames audible: the script is too quiet to test much");
    eprintln!("{ops} AX calls, {frames} frames ({audible} non-silent, {full} compared sample by sample) bit-exact");
}
