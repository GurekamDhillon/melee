//! The synth/axdriver port on synthetic banks (no disc): handle allocation, a script starting a
//! sound and the pool reclaiming it, key-off, pause channels, and an HPS stream played across
//! its blocks. The disc-backed run is `src/bin/sfx_play.rs` and `melee/tests/audio_sfx.rs`.

use melee_audio::disc::StreamSource;
use melee_audio::sound::{AudioFs, Sound};

fn be(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

/// `frames` DSP-ADPCM frames of a loud square wave (ps 0x0B, coefficients 0).
fn adpcm_square(frames: usize) -> Vec<u8> {
    let mut d = Vec::new();
    for _ in 0..frames {
        d.push(0x0B);
        d.extend_from_slice(&[0x77, 0x77, 0x77, 0x99, 0x99, 0x99, 0x99]);
    }
    d
}

/// A one-sound SSM: sound `base`, one ADPCM voice of `frames` frames, no loop.
fn ssm(base: u32, frames: usize) -> Vec<u8> {
    let data = adpcm_square(frames);
    let mut hdr = Vec::new(); // starts at file 0x10
    be(&mut hdr, 1); // voices
    be(&mut hdr, 32000); // rate
    let mut vb = vec![0u8; 0x40];
    vb[0..2].copy_from_slice(&0u16.to_be_bytes()); // loopFlag
    vb[2..4].copy_from_slice(&0u16.to_be_bytes()); // ADPCM
    vb[8..12].copy_from_slice(&((data.len() * 2 - 1) as u32).to_be_bytes()); // end (nibbles)
    vb[12..16].copy_from_slice(&2u32.to_be_bytes()); // current: past the first header
    vb[0x10 + 0x22..0x10 + 0x24].copy_from_slice(&0x0Bu16.to_be_bytes()); // pred_scale
    hdr.extend_from_slice(&vb);
    let header_size = hdr.len() as u32;
    let mut f = Vec::new();
    be(&mut f, header_size);
    be(&mut f, data.len() as u32);
    be(&mut f, 1); // sounds
    be(&mut f, base);
    f.resize(0x10, 0);
    f.extend_from_slice(&hdr);
    let at = ((header_size + 0x10 + 31) & !31) as usize;
    f.resize(at, 0);
    f.extend_from_slice(&data);
    f
}

/// smash2.sem with one bank of `scripts`, each a list of command words.
fn sem(scripts: &[Vec<u32>]) -> Vec<u8> {
    let mut w: Vec<u32> = vec![0, 0, 1, 0, scripts.len() as u32];
    let table = w.len();
    w.extend(std::iter::repeat_n(0, scripts.len()));
    w.push(0); // nC
    for (i, s) in scripts.iter().enumerate() {
        w[table + i] = (w.len() * 4) as u32;
        w.extend_from_slice(s);
    }
    w.iter().flat_map(|x| x.to_be_bytes()).collect()
}

struct MemFs(Vec<(String, Vec<u8>)>);
impl AudioFs for MemFs {
    fn read(&mut self, path: &str) -> Option<(i32, Vec<u8>)> {
        self.0.iter().enumerate().find(|(_, (p, _))| p == path).map(|(i, (_, d))| (i as i32 + 1, d.clone()))
    }
    fn open_stream(&mut self, path: &str) -> Option<(i32, Box<dyn StreamSource>)> {
        let (e, d) = self.read(path)?;
        Some((e, Box::new(d)))
    }
}

const PLAY: u32 = 1 << 24;
const WAIT: u32 = 0;
const END: u32 = 15 << 24;

fn booted(extra: Vec<(String, Vec<u8>)>) -> Sound {
    let mut files = vec![
        ("/audio/us/smash2.sem".to_string(), sem(&[vec![PLAY | 7, WAIT | 30, END], vec![PLAY | 7, END]])),
        ("/audio/us/main.ssm".to_string(), ssm(7, 400)),
    ];
    files.extend(extra);
    let mut s = Sound::new();
    s.boot(Box::new(MemFs(files)), [0x10000, 0x10000, 0x10000]).unwrap();
    s
}

fn run(s: &mut Sound, n: usize) -> i32 {
    let mut peak = 0;
    for _ in 0..n {
        let o = s.sub_frame();
        peak = peak.max(o.iter().map(|&x| (x as i32).abs()).max().unwrap());
    }
    peak
}

#[test]
fn handles_come_from_the_virtual_voice_pool() {
    let mut s = booted(vec![]);
    let a = s.lb_audio_ax_800237a8(0, 0x7F, 0x40);
    let b = s.lb_audio_ax_800237a8(1, 0x7F, 0x40);
    // AXDriverLink prepends 0..95, so the free list hands out 95, 94, ...; vidhigh counts starts.
    assert_eq!(a, 95);
    assert_eq!(b, (1 << 7) | 94);
    assert_eq!(s.lb_audio_ax_800237a8(9, 0x7F, 0x40), -1, "a script id past the bank");
    assert_eq!(s.live_count(), 2);
    // Nothing reaches the synth before the first callback runs the scripts.
    assert_eq!(s.active_nodes(), 0);
    let peak = run(&mut s, 3);
    assert!(peak > 1000, "silent: {peak}");
    assert!(s.audio_sfx_check(a));
    assert!(!s.audio_sfx_check(b), "script 1 ends (keys off) right after it plays");
}

#[test]
fn a_script_plays_then_its_voice_returns_to_the_pool() {
    let mut s = booted(vec![]);
    let h = s.lb_audio_ax_800237a8(1, 0x7F, 0x40); // PLAY, END: the end keys off at once
    run(&mut s, 2);
    assert!(!s.audio_sfx_check(h));
    let mut s = booted(vec![]);
    let h = s.lb_audio_ax_800237a8(0, 0x7F, 0x40); // PLAY, WAIT 30, END
    assert!(run(&mut s, 10) > 1000);
    assert!(s.audio_sfx_check(h));
    run(&mut s, 40); // the wait ends: key-off, fade, free
    assert!(!s.audio_sfx_check(h));
    run(&mut s, 200);
    assert_eq!(s.live_count(), 0);
    assert_eq!(s.active_nodes(), 0);
    // The freed slot is the head of the free list again.
    assert_eq!(s.lb_audio_ax_800237a8(0, 0x7F, 0x40) & 0x7F, 95);
}

#[test]
fn key_off_and_paused_channels() {
    let mut s = booted(vec![]);
    let h = s.lb_audio_ax_800237a8(0, 0x7F, 0x40);
    run(&mut s, 5);
    assert!(s.audio_sfx_key_off(h));
    assert!(!s.audio_sfx_key_off(h), "a second key-off finds no voice");
    run(&mut s, 60);
    assert!(run(&mut s, 10) == 0, "still sounding after key-off");
    s.pstream_pause_ch(7);
    assert_eq!(s.lb_audio_ax_800237a8(0, 0x7F, 0x40), -1, "channel 7 is paused");
    s.pstream_resume_ch(7);
    assert!(s.lb_audio_ax_800237a8(0, 0x7F, 0x40) >= 0);
}

/// A mono HPS of `blocks` ADPCM blocks (`frames` frames each), the last one ending the stream.
fn hps(blocks: usize, frames: usize) -> Vec<u8> {
    let mut f = vec![0u8; 0x80];
    f[0..8].copy_from_slice(b" HALPST\0");
    f[8..12].copy_from_slice(&32000u32.to_be_bytes());
    f[12..16].copy_from_slice(&1u32.to_be_bytes());
    // Channel 0: AXPBADDR (loop on: each block's end continues at the next block's start).
    f[0x10..0x12].copy_from_slice(&1u16.to_be_bytes());
    f[0x20 + 0x22..0x20 + 0x24].copy_from_slice(&0x0Bu16.to_be_bytes());
    let data = adpcm_square(frames);
    for b in 0..blocks {
        let at = f.len() as u32;
        let next = if b + 1 == blocks { u32::MAX } else { at + 0x20 + data.len() as u32 };
        let mut h = vec![0u8; 0x20];
        h[0..4].copy_from_slice(&(data.len() as u32).to_be_bytes());
        h[4..8].copy_from_slice(&((data.len() * 2 - 1) as u32).to_be_bytes());
        h[8..12].copy_from_slice(&next.to_be_bytes());
        h[0xC..0xE].copy_from_slice(&0x0Bu16.to_be_bytes()); // loop pred_scale
        f.extend_from_slice(&h);
        f.extend_from_slice(&data);
    }
    f
}

#[test]
fn the_music_stream_plays_through_every_block() {
    let (blocks, frames) = (6, 800); // 800 frames = 11200 samples = 350 ms a block
    let mut s = booted(vec![("/audio/test.hps".to_string(), hps(blocks, frames))]);
    assert!(s.lb_play_bgm_file("test.hps", 0x7F, 1));
    let mut sounding = 0;
    for _ in 0..(blocks * 70 + 200) {
        let o = s.sub_frame();
        if o.iter().any(|&x| x.unsigned_abs() > 1000) {
            sounding += 1;
        }
    }
    // 6 blocks x 350 ms = 2100 ms = 420 sub-frames (the reverb tail adds a little).
    eprintln!("stream sounded for {sounding} sub-frames");
    assert!((400..=460).contains(&sounding), "stream sounded for {sounding} sub-frames");
    assert!(!s.stream_check(), "the stream did not end at its last block");
}
