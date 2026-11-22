//! The Music and SFX volume settings (melee_audio::ax::set_music_volume / set_sfx_volume) on synthetic banks: each
//! scales only its own voices, and 100 / 100 is the unscaled mix bit for bit. One test in its own binary: the
//! gains are process-wide statics.

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


/// `n` sub-frames of the mix: the stream (when `music`) and one long SFX (when `sfx`) started together.
fn capture(music: bool, sfx: bool, n: usize) -> Vec<i16> {
    let mut s = booted(vec![("/audio/test.hps".to_string(), hps(4, 800))]);
    if music {
        assert!(s.lb_play_bgm_file("test.hps", 0x7F, 1));
    }
    if sfx {
        assert!(s.lb_audio_ax_800237a8(0, 0x7F, 0x40) >= 0);
    }
    let mut out = Vec::new();
    for _ in 0..n {
        out.extend_from_slice(&s.sub_frame());
    }
    out
}

fn energy(v: &[i16]) -> f64 {
    v.iter().map(|&x| (x as f64) * (x as f64)).sum()
}

#[test]
fn music_and_sfx_gains_apply_separately() {
    use melee_audio::ax::{set_music_volume, set_sfx_volume};
    const N: usize = 60;
    set_music_volume(100);
    set_sfx_volume(100);
    let both = capture(true, true, N);
    let music_only = capture(true, false, N);
    let sfx_only = capture(false, true, N);
    assert!(energy(&music_only) > 0.0 && energy(&sfx_only) > 0.0);
    // Music 0: exactly the SFX-only mix; SFX 0: exactly the music-only mix.
    set_music_volume(0);
    assert_eq!(capture(true, true, N), sfx_only, "music 0 left music in the mix");
    set_music_volume(100);
    set_sfx_volume(0);
    assert_eq!(capture(true, true, N), music_only, "sfx 0 left sfx in the mix");
    // Half music: the music part's energy drops to about a quarter, the SFX untouched.
    set_sfx_volume(0);
    set_music_volume(50);
    let half = capture(true, true, N);
    let r = energy(&half) / energy(&music_only);
    assert!((0.22..0.28).contains(&r), "music 50%: energy ratio {r}");
    set_music_volume(100);
    set_sfx_volume(25);
    let q = capture(false, true, N);
    let r = energy(&q) / energy(&sfx_only);
    assert!((0.05..0.075).contains(&r), "sfx 25%: energy ratio {r}");
    // Back at 100 / 100: the unscaled mix, bit for bit.
    set_sfx_volume(100);
    assert_eq!(capture(true, true, N), both);
}
