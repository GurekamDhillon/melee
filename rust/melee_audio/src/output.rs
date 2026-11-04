//! Output side of shim_ax.c: the frame driver (`gw_ax_frame_tick`), master volume
//! (`gw_Audio_Volume` / `gw_Audio_SetVolume`, `audio.cfg`), `MELEE_AUDIO_DUMP` WAV capture, and
//! the device: a lock-free single-producer/single-consumer ring drained by a cpal callback
//! (the C used an SDL3 audio stream). None of this is game-visible.
//!
//! Pacing is as in the C: with a device, the number of 5 ms sub-frames to generate per call is
//! the shortfall of the ring against its target cushion (the device clock plays the DSP's 5 ms
//! interrupt); without one (turbo, headless, no device), the wall clock drives the synth.
//! Differences from the SDL path: cpal has no stream resampler, so when the device will not run
//! at 32 kHz the callback resamples to the device rate (a 32-tap Kaiser-windowed sinc), and the gain is applied there.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ax::Ax;
use crate::{FRAME_SAMPLES, RATE};

/// 512 ms of stereo frames (the C's 65536-byte ring).
pub const RING_FRAMES: u32 = 16384;
/// 80 ms of catch-up per call.
pub const MAX_BURST: u32 = 16;

/// Sub-frame production stamps kept for the latency measurement (more than the ring holds at any cushion used).
const STAMPS: usize = 128;

/// SPSC ring of stereo frames, each packed as `L as u16 | (R as u16) << 16`.
///
/// Besides the samples it carries the output-side bookkeeping, none of which feeds back into the mix: the cushion the
/// producer tops up to (`target`, in 5 ms sub-frames, raised by the consumer after an underrun and lowered again after a
/// clean stretch), and the latency measurement (each sub-frame's push time, read when the device takes its first sample).
pub struct Ring {
    buf: Box<[AtomicU32]>,
    /// frames produced / consumed (monotonic, wrapping)
    w: AtomicU32,
    r: AtomicU32,
    pub underruns: AtomicU32,
    pub overruns: AtomicU32,
    /// Underrun events (a starved stretch counts once; `underruns` counts frames).
    pub underrun_events: AtomicU32,
    /// Cushion in sub-frames the producer keeps buffered (adaptive; see [`Ring::target`]).
    target: AtomicU32,
    target_min: AtomicU32,
    target_max: AtomicU32,
    /// Push time (ns since `epoch`) of sub-frame k at `stamp[k % STAMPS]`.
    stamp: Box<[AtomicU64]>,
    epoch: Instant,
    /// Latency stats (microseconds): ring residency of each sub-frame plus the device's hold time.
    pub lat_sum_us: AtomicU64,
    pub lat_n: AtomicU64,
    pub lat_max_us: AtomicU64,
    pub lat_min_us: AtomicU64,
    /// Largest device callback seen (frames at the device rate) and the device rate.
    pub cb_frames_max: AtomicU32,
    /// The latest device callback's length (frames at the device rate).
    pub cb_frames_last: AtomicU32,
    pub dev_rate: AtomicU32,
}

/// A latency summary ([`Ring::latency`]).
#[derive(Clone, Copy, Debug, Default)]
pub struct Latency {
    pub mean_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub samples: u64,
    pub callback_frames: u32,
    pub device_rate: u32,
    pub target_ms: u32,
}

impl Default for Ring {
    fn default() -> Self {
        Self::new()
    }
}

impl Ring {
    pub fn new() -> Self {
        Ring {
            buf: (0..RING_FRAMES).map(|_| AtomicU32::new(0)).collect(),
            w: AtomicU32::new(0),
            r: AtomicU32::new(0),
            underruns: AtomicU32::new(0),
            overruns: AtomicU32::new(0),
            underrun_events: AtomicU32::new(0),
            target: AtomicU32::new(12),
            target_min: AtomicU32::new(12),
            target_max: AtomicU32::new(12),
            stamp: (0..STAMPS).map(|_| AtomicU64::new(0)).collect(),
            epoch: Instant::now(),
            lat_sum_us: AtomicU64::new(0),
            lat_n: AtomicU64::new(0),
            lat_max_us: AtomicU64::new(0),
            lat_min_us: AtomicU64::new(u64::MAX),
            cb_frames_max: AtomicU32::new(0),
            cb_frames_last: AtomicU32::new(0),
            dev_rate: AtomicU32::new(0),
        }
    }
    /// Set the cushion: start at `target` sub-frames, adapt between `min` and `max`.
    pub fn set_target(&self, target: u32, min: u32, max: u32) {
        let min = min.max(1);
        let max = max.max(min);
        self.target_min.store(min, Ordering::Relaxed);
        self.target_max.store(max, Ordering::Relaxed);
        self.target.store(target.clamp(min, max), Ordering::Relaxed);
    }
    /// The cushion the producer tops the ring up to, in 5 ms sub-frames.
    pub fn target(&self) -> u32 {
        self.target.load(Ordering::Relaxed)
    }
    /// Consumer: an underrun happened; keep one more sub-frame from now on (up to the max).
    fn widen(&self) {
        let (t, max) = (self.target.load(Ordering::Relaxed), self.target_max.load(Ordering::Relaxed));
        if t < max {
            self.target.store(t + 1, Ordering::Relaxed);
        }
    }
    /// Consumer: a long clean stretch; give one sub-frame back (down to the min).
    fn narrow(&self) {
        let (t, min) = (self.target.load(Ordering::Relaxed), self.target_min.load(Ordering::Relaxed));
        if t > min {
            self.target.store(t - 1, Ordering::Relaxed);
        }
    }
    fn now_ns(&self) -> u64 {
        self.epoch.elapsed().as_nanos() as u64
    }
    /// Consumer: the device is about to take frame `r`, a sub-frame's first sample; `queued_ns` is how long the device
    /// holds it before it is heard.
    fn note_latency(&self, r: u32, queued_ns: u64) {
        let k = (r / FRAME_SAMPLES as u32) as usize % STAMPS;
        let t = self.stamp[k].load(Ordering::Relaxed);
        if t == 0 {
            return;
        }
        let us = (self.now_ns().saturating_sub(t) + queued_ns) / 1000;
        self.lat_sum_us.fetch_add(us, Ordering::Relaxed);
        self.lat_n.fetch_add(1, Ordering::Relaxed);
        self.lat_max_us.fetch_max(us, Ordering::Relaxed);
        self.lat_min_us.fetch_min(us, Ordering::Relaxed);
    }
    /// Forget the latency stats (after a warm-up).
    pub fn reset_latency(&self) {
        self.lat_sum_us.store(0, Ordering::Relaxed);
        self.lat_n.store(0, Ordering::Relaxed);
        self.lat_max_us.store(0, Ordering::Relaxed);
        self.lat_min_us.store(u64::MAX, Ordering::Relaxed);
        self.cb_frames_max.store(0, Ordering::Relaxed);
    }
    /// Output latency measured so far: from a mixed sub-frame being handed to the ring to the device playing its first
    /// sample (ring residency, measured, plus the device's queue at that callback as cpal reports it).
    pub fn latency(&self) -> Latency {
        let n = self.lat_n.load(Ordering::Relaxed);
        let ms = |us: u64| us as f64 / 1000.0;
        Latency {
            mean_ms: if n == 0 { 0.0 } else { ms(self.lat_sum_us.load(Ordering::Relaxed)) / n as f64 },
            min_ms: if n == 0 { 0.0 } else { ms(self.lat_min_us.load(Ordering::Relaxed)) },
            max_ms: ms(self.lat_max_us.load(Ordering::Relaxed)),
            samples: n,
            callback_frames: self.cb_frames_max.load(Ordering::Relaxed),
            device_rate: self.dev_rate.load(Ordering::Relaxed),
            target_ms: self.target() * 5,
        }
    }
    /// Frames currently buffered.
    pub fn len(&self) -> u32 {
        self.w.load(Ordering::Acquire).wrapping_sub(self.r.load(Ordering::Acquire))
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Producer: push interleaved stereo; drops what does not fit (counted as an overrun).
    pub fn push(&self, stereo: &[i16]) {
        let w = self.w.load(Ordering::Relaxed);
        let avail = RING_FRAMES - w.wrapping_sub(self.r.load(Ordering::Acquire));
        let mut n = (stereo.len() / 2) as u32;
        if w % FRAME_SAMPLES as u32 == 0 {
            let k = (w / FRAME_SAMPLES as u32) as usize % STAMPS;
            self.stamp[k].store(self.now_ns().max(1), Ordering::Relaxed);
        }
        if avail < n {
            self.overruns.fetch_add(1, Ordering::Relaxed);
            n = avail;
        }
        for k in 0..n {
            let l = stereo[k as usize * 2] as u16 as u32;
            let r = stereo[k as usize * 2 + 1] as u16 as u32;
            self.buf[(w.wrapping_add(k) % RING_FRAMES) as usize].store(l | r << 16, Ordering::Relaxed);
        }
        self.w.store(w.wrapping_add(n), Ordering::Release);
    }
    /// Consumer: one frame, or None on underrun.
    pub fn pop(&self) -> Option<(i16, i16)> {
        self.pop_timed(None)
    }
    /// [`Ring::pop`], measuring the latency at each sub-frame start when `queued_ns` (the device's hold time) is given.
    pub fn pop_timed(&self, queued_ns: Option<u64>) -> Option<(i16, i16)> {
        let r = self.r.load(Ordering::Relaxed);
        if self.w.load(Ordering::Acquire) == r {
            return None;
        }
        if let Some(q) = queued_ns {
            if r % FRAME_SAMPLES as u32 == 0 {
                self.note_latency(r, q);
            }
        }
        let v = self.buf[(r % RING_FRAMES) as usize].load(Ordering::Relaxed);
        self.r.store(r.wrapping_add(1), Ordering::Release);
        Some((v as u16 as i16, (v >> 16) as u16 as i16))
    }
}

// ---- master volume --------------------------------------------------------------------------

fn cfg_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("audio.cfg")))
        .unwrap_or_else(|| PathBuf::from("audio.cfg"))
}

/// `atoi`: leading whitespace, optional sign, digits; 0 when none.
fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (neg, rest) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let mut v: i64 = 0;
    for c in rest.bytes().take_while(u8::is_ascii_digit) {
        v = (v * 10 + (c - b'0') as i64).min(i32::MAX as i64 + 1);
    }
    (if neg { -v } else { v }).clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

/// Master volume 0..100: MELEE_VOLUME, else `volume = N` in audio.cfg beside the exe, else 100.
pub fn load_volume() -> i32 {
    let mut vol = 100;
    if let Ok(text) = std::fs::read_to_string(cfg_path()) {
        for line in text.lines() {
            let l = line.trim_start();
            if let Some(rest) = l.strip_prefix("volume") {
                if let Some(v) = rest.trim_start().strip_prefix('=') {
                    let v = v.trim_start();
                    if v.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+') {
                        vol = atoi(v).clamp(0, 100);
                    }
                }
            }
        }
    }
    if let Ok(env) = std::env::var("MELEE_VOLUME") {
        if !env.is_empty() {
            vol = atoi(&env).clamp(0, 100);
        }
    }
    vol
}

/// Write audio.cfg (the settings menu's gw_Audio_SetVolume).
pub fn save_volume(percent: i32) -> std::io::Result<()> {
    std::fs::write(
        cfg_path(),
        format!("# melee-pc audio settings (the game rewrites this file)\nvolume = {}\n", percent.clamp(0, 100)),
    )
}

// ---- WAV capture ----------------------------------------------------------------------------

/// 32 kHz stereo s16 WAV; sizes refreshed about once a second so a crash leaves a playable file.
pub struct Wav {
    f: File,
    bytes: u32,
}

impl Wav {
    pub fn create(path: &str) -> std::io::Result<Wav> {
        let mut f = File::create(path)?;
        f.write_all(&Self::header(0))?;
        Ok(Wav { f, bytes: 0 })
    }
    fn header(data: u32) -> [u8; 44] {
        let mut h = [0u8; 44];
        h[0..4].copy_from_slice(b"RIFF");
        h[4..8].copy_from_slice(&(36 + data).to_le_bytes());
        h[8..16].copy_from_slice(b"WAVEfmt ");
        h[16] = 16;
        h[20] = 1; // PCM
        h[22] = 2; // stereo
        h[24..28].copy_from_slice(&RATE.to_le_bytes());
        h[28..32].copy_from_slice(&(RATE * 4).to_le_bytes());
        h[32] = 4;
        h[34] = 16;
        h[36..40].copy_from_slice(b"data");
        h[40..44].copy_from_slice(&data.to_le_bytes());
        h
    }
    pub fn write(&mut self, stereo: &[i16]) {
        let bytes: Vec<u8> = stereo.iter().flat_map(|s| s.to_le_bytes()).collect();
        if self.f.write_all(&bytes).is_err() {
            return;
        }
        self.bytes = self.bytes.wrapping_add(bytes.len() as u32);
        if self.bytes % (FRAME_SAMPLES as u32 * 4 * 200) == 0 {
            let _ = self.finish();
        }
    }
    fn finish(&mut self) -> std::io::Result<()> {
        let pos = self.f.stream_position()?;
        self.f.seek(SeekFrom::Start(0))?;
        self.f.write_all(&Self::header(self.bytes))?;
        self.f.seek(SeekFrom::Start(pos))?;
        self.f.flush()
    }
}

impl Drop for Wav {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

// ---- the device -----------------------------------------------------------------------------

#[cfg(feature = "device")]
mod device {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    pub struct Device {
        _stream: cpal::Stream,
        pub rate: u32,
    }

    /// Band-limited 32 kHz -> device-rate conversion: a Kaiser-windowed sinc (beta 7, `TAPS` taps, `PHASES` sub-sample
    /// phases, cut at 95% of the lower Nyquist), unity DC gain per phase. Built on the opening thread; the callback only reads it.
    const TAPS: usize = 32;
    const PHASES: usize = 256;
    fn sinc_table(rate: u32) -> Vec<f32> {
        fn i0(x: f64) -> f64 {
            let (mut s, mut t) = (1.0, 1.0);
            for k in 1..32 {
                t *= (x / (2.0 * k as f64)).powi(2);
                s += t;
            }
            s
        }
        let cutoff = 0.5 * 0.95 * (rate as f64 / RATE as f64).min(1.0); // cycles per input sample
        let beta = 7.0;
        let mut k = vec![0f32; (PHASES + 1) * TAPS];
        for p in 0..=PHASES {
            let frac = p as f64 / PHASES as f64;
            let mut row = [0f64; TAPS];
            let mut sum = 0.0;
            for (j, w) in row.iter_mut().enumerate() {
                // tap j weighs input sample (newest - (TAPS-1) + j); the output point sits between taps TAPS/2-1 and TAPS/2
                let x = j as f64 - (TAPS / 2 - 1) as f64 - frac;
                let r = x / (TAPS / 2) as f64;
                let win = if r.abs() >= 1.0 { 0.0 } else { i0(beta * (1.0 - r * r).sqrt()) / i0(beta) };
                let sn = if x == 0.0 { 2.0 * cutoff } else { (2.0 * std::f64::consts::PI * cutoff * x).sin() / (std::f64::consts::PI * x) };
                *w = sn * win;
                sum += *w;
            }
            for j in 0..TAPS {
                k[p * TAPS + j] = (row[j] / sum) as f32;
            }
        }
        k
    }

    #[derive(Clone)]
    struct Feed {
        ring: Arc<Ring>,
        gain: Arc<AtomicU32>,
        /// 16.16 phase step of the 32 kHz source per device frame.
        step: u32,
        frac: u32,
        /// The last `TAPS` source frames (`hist[pos]` the oldest), for the sinc.
        hist: [(f32, f32); TAPS],
        pos: usize,
        kernel: Vec<f32>,
        /// The ring has delivered once: silence before the first mix (device opened ahead of the game) is not an underrun.
        primed: bool,
        /// Underrun handling: the last frame played and the gain ramp (1.0 = the ring's frames as they are). On a starved
        /// pop the output decays from the last frame to silence over `FADE_OUT` frames instead of stepping to 0 (a click);
        /// when frames arrive again they fade in over `FADE_IN`. Output side only: the ring's frames are never altered.
        last: (f32, f32),
        ramp: f32,
        starved: bool,
        /// Device frames since the last underrun (the cushion gives a sub-frame back after `NARROW_AFTER_S` clean seconds).
        clean: u64,
        /// The device's hold time for the frame being written (ns), for the latency measurement.
        queued_ns: u64,
        /// Nominal 16.16 step, and the drift correction: the step is nudged by the ring's offset from its cushion (at most
        /// `MAX_SKEW`), so a device clock a little off 32 kHz (or the producer's 5 ms granularity) is absorbed by a pitch
        /// change far below hearing instead of the fill wandering into an underrun. Only when resampling (rate != 32 kHz).
        nominal: u32,
        rate: u32,
    }

    /// 2 ms of decay to silence, 4 ms of fade-in, at the device rate (set at open).
    const FADE_OUT_MS: f32 = 2.0;
    const FADE_IN_MS: f32 = 4.0;
    const NARROW_AFTER_S: u64 = 10;
    /// 0.2% at most (about 3.5 cents).
    const MAX_SKEW: f64 = 0.002;

    impl Feed {
        /// Once per callback: steer the resampler step by the ring's fill against its cushion.
        fn steer(&mut self) {
            if self.nominal == 0x10000 {
                return;
            }
            let fill = self.ring.len() as f64 / FRAME_SAMPLES as f64; // sub-frames buffered
            let want = self.ring.target() as f64 - 0.5; // the producer keeps target-1 .. target
            let err = ((fill - want) / want.max(1.0)).clamp(-1.0, 1.0);
            let skew = (err * MAX_SKEW).clamp(-MAX_SKEW, MAX_SKEW);
            self.step = ((self.nominal as f64) * (1.0 + skew)).round().max(1.0) as u32;
        }
        fn frame(&mut self) -> (f32, f32) {
            let g = f32::from_bits(self.gain.load(Ordering::Relaxed)) / 32768.0;
            if self.step == 0x10000 && self.nominal == 0x10000 {
                let a = self.pop();
                return (a.0 * g, a.1 * g);
            }
            self.frac += self.step;
            while self.frac >= 0x10000 {
                self.frac -= 0x10000;
                let v = self.pop();
                self.hist[self.pos] = v;
                self.pos = (self.pos + 1) % TAPS;
            }
            let p = ((self.frac as u64 * PHASES as u64) >> 16) as usize;
            let row = &self.kernel[p * TAPS..(p + 1) * TAPS];
            let (mut l, mut r) = (0f32, 0f32);
            for (j, &w) in row.iter().enumerate() {
                let (a, b) = self.hist[(self.pos + j) % TAPS];
                l += a * w;
                r += b * w;
            }
            (l * g, r * g)
        }
        fn pop(&mut self) -> (f32, f32) {
            let rate = self.rate.max(1) as f32;
            match self.ring.pop_timed(Some(self.queued_ns)) {
                Some(v) => {
                    self.primed = true;
                    let v = (v.0 as f32, v.1 as f32);
                    if self.starved {
                        self.starved = false;
                        self.ramp = 0.0;
                    }
                    if self.ramp < 1.0 {
                        self.ramp = (self.ramp + 1000.0 / (FADE_IN_MS * rate)).min(1.0);
                    }
                    self.clean += 1;
                    if self.clean >= NARROW_AFTER_S * rate as u64 {
                        self.clean = 0;
                        self.ring.narrow();
                    }
                    self.last = v;
                    (v.0 * self.ramp, v.1 * self.ramp)
                }
                None => {
                    if !self.primed {
                        return (0.0, 0.0);
                    }
                    self.ring.underruns.fetch_add(1, Ordering::Relaxed);
                    if !self.starved {
                        self.starved = true;
                        self.clean = 0;
                        self.ring.underrun_events.fetch_add(1, Ordering::Relaxed);
                        self.ring.widen();
                    }
                    self.ramp = (self.ramp - 1000.0 / (FADE_OUT_MS * rate)).max(0.0);
                    (self.last.0 * self.ramp, self.last.1 * self.ramp)
                }
            }
        }
    }

    fn fill<T: cpal::SizedSample + cpal::FromSample<f32>>(feed: &mut Feed, data: &mut [T], ch: usize, info: &cpal::OutputCallbackInfo) {
        let frames = (data.len() / ch.max(1)) as u32;
        feed.ring.cb_frames_max.fetch_max(frames, Ordering::Relaxed);
        feed.ring.cb_frames_last.store(frames, Ordering::Relaxed);
        let ts = info.timestamp();
        let base = ts.playback.duration_since(&ts.callback).map_or(0, |d| d.as_nanos() as u64);
        let per = 1_000_000_000u64 / feed.rate.max(1) as u64;
        feed.steer();
        for (i, f) in data.chunks_mut(ch).enumerate() {
            feed.queued_ns = base + i as u64 * per;
            let (l, r) = feed.frame();
            for (c, s) in f.iter_mut().enumerate() {
                let v = match c {
                    0 => l,
                    1 => r,
                    _ => 0.0,
                };
                *s = T::from_sample(if ch == 1 { (l + r) * 0.5 } else { v });
            }
        }
    }

    impl Device {
        /// The default output device, preferring 32 kHz stereo. `buffer_ms` > 0 asks for a device buffer of that length
        /// (WASAPI shared mode, event driven: the audio engine rounds it up to its period; cpal 0.15 has no exclusive
        /// mode); when the device refuses it the default buffer is used.
        pub fn open(ring: Arc<Ring>, gain: Arc<AtomicU32>, buffer_ms: u32) -> Result<Device, String> {
            let host = cpal::default_host();
            let dev = host.default_output_device().ok_or("no default output device")?;
            let want = cpal::SampleRate(RATE);
            let cfg = dev
                .supported_output_configs()
                .ok()
                .and_then(|mut it| {
                    it.find(|c| c.channels() >= 2 && c.min_sample_rate() <= want && c.max_sample_rate() >= want)
                })
                .map(|c| c.with_sample_rate(want))
                .map_or_else(|| dev.default_output_config().map_err(|e| e.to_string()), Ok)?;
            let rate = cfg.sample_rate().0;
            let ch = cfg.channels() as usize;
            let fmt = cfg.sample_format();
            let mut config: cpal::StreamConfig = cfg.into();
            ring.dev_rate.store(rate, Ordering::Relaxed);
            let mut feed = Feed {
                ring,
                gain,
                step: (((RATE as u64) << 16) / rate as u64).max(1) as u32,
                frac: 0,
                hist: [(0.0, 0.0); TAPS],
                pos: 0,
                kernel: if rate == RATE { Vec::new() } else { sinc_table(rate) },
                primed: false,
                last: (0.0, 0.0),
                ramp: 1.0,
                starved: false,
                clean: 0,
                queued_ns: 0,
                nominal: 0,
                rate,
            };
            if rate == RATE {
                feed.step = 0x10000;
            }
            feed.nominal = feed.step;
            macro_rules! build {
                ($t:ty, $config:expr, $feed:expr) => {{
                    let mut feed = $feed;
                    dev.build_output_stream(
                        $config,
                        move |d: &mut [$t], info: &cpal::OutputCallbackInfo| fill(&mut feed, d, ch, info),
                        |e| eprintln!("melee_audio: stream error: {e}"),
                        None,
                    )
                }};
            }
            let try_build = |config: &cpal::StreamConfig, feed: Feed| match fmt {
                cpal::SampleFormat::F32 => build!(f32, config, feed),
                cpal::SampleFormat::I16 => build!(i16, config, feed),
                cpal::SampleFormat::U16 => build!(u16, config, feed),
                cpal::SampleFormat::I32 => build!(i32, config, feed),
                _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
            };
            if !matches!(fmt, cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::U16 | cpal::SampleFormat::I32) {
                return Err(format!("unsupported sample format {fmt:?}"));
            }
            let stream = if buffer_ms > 0 {
                config.buffer_size = cpal::BufferSize::Fixed((rate * buffer_ms / 1000).max(32));
                match try_build(&config, feed.clone()) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("melee_audio: a {buffer_ms} ms device buffer was refused ({e}); using the default buffer");
                        config.buffer_size = cpal::BufferSize::Default;
                        try_build(&config, feed).map_err(|e| e.to_string())?
                    }
                }
            } else {
                try_build(&config, feed).map_err(|e| e.to_string())?
            };
            stream.play().map_err(|e| e.to_string())?;
            Ok(Device { _stream: stream, rate })
        }

        /// [`Device::open`] on a thread of its own, which then keeps the stream until the returned handle is dropped.
        /// Opening the device (WASAPI activation, RPC to the audio service) took 20-60% of the first match tick; done
        /// from the driver's creation on, it is ready (or nearly) by the time the game first needs it.
        pub fn open_async(ring: Arc<Ring>, gain: Arc<AtomicU32>, buffer_ms: u32) -> std::sync::mpsc::Receiver<Result<(u32, Held), String>> {
            let (tx, rx) = std::sync::mpsc::channel();
            let spawned = std::thread::Builder::new().name("audio open".into()).spawn(move || {
                let (stop_tx, stop) = std::sync::mpsc::channel::<()>();
                match Device::open(ring, gain, buffer_ms) {
                    Ok(d) => {
                        if tx.send(Ok((d.rate, Held(stop_tx)))).is_ok() {
                            // blocks until the Held handle is dropped (its sender closes the channel)
                            let _ = stop.recv();
                        }
                        drop(d);
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                }
            });
            if let Err(e) = spawned {
                let (tx, rx2) = std::sync::mpsc::channel();
                let _ = tx.send(Err(format!("audio open thread: {e}")));
                return rx2;
            }
            rx
        }
    }

    /// Keeps an [`Device::open_async`] stream alive; dropping it closes the stream.
    pub struct Held(#[allow(dead_code)] std::sync::mpsc::Sender<()>);
}

// ---- frame driver ---------------------------------------------------------------------------

/// Driver settings (the C read these from the environment; [`DriverConfig::from_env`] still does).
#[derive(Clone, Debug)]
pub struct DriverConfig {
    /// Turbo: keep the 5 ms synth/DSP clock but never open a device (MELEE_TURBO).
    pub turbo: bool,
    /// Ring cushion in sub-frames (MELEE_AUDIO_LATENCY_MS / 5, from 5..200 ms; default 12, low-latency mode 2). The
    /// cushion adapts: +1 sub-frame per underrun up to `max_frames`, -1 after 10 s without one, never below this.
    pub target_frames: u32,
    pub max_frames: u32,
    /// Device buffer request in ms (0 = the device default). Low-latency mode asks for 5 ms.
    pub buffer_ms: u32,
    /// MELEE_AUDIO_LOW_LATENCY=1 (or MELEE_AUDIO_MODE=low): small cushion, small device buffer.
    pub low_latency: bool,
    pub nofx: bool,
    pub dump: Option<String>,
    /// Open the output device (false for tests / headless).
    pub device: bool,
}

impl DriverConfig {
    pub fn from_env(turbo: bool) -> Self {
        let flag = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty() && v != "0");
        let low_latency = flag("MELEE_AUDIO_LOW_LATENCY").is_some()
            || std::env::var("MELEE_AUDIO_MODE").is_ok_and(|v| v.eq_ignore_ascii_case("low"));
        let (mut target_frames, mut buffer_ms) = if low_latency { (2, 5) } else { (12, 0) };
        if let Ok(v) = std::env::var("MELEE_AUDIO_LATENCY_MS") {
            target_frames = ((atoi(&v).clamp(5, 200) / 5) as u32).max(1);
        }
        if let Ok(v) = std::env::var("MELEE_AUDIO_BUFFER_MS") {
            buffer_ms = atoi(&v).clamp(0, 100) as u32;
        }
        let max_frames = target_frames.max(if low_latency { 8 } else { target_frames + 8 });
        DriverConfig {
            turbo,
            target_frames,
            max_frames,
            buffer_ms,
            low_latency,
            nofx: std::env::var_os("MELEE_AUDIO_NOFX").is_some(),
            dump: std::env::var("MELEE_AUDIO_DUMP").ok().filter(|s| !s.is_empty()),
            device: true,
        }
    }
}

/// gw_ax_frame_tick's state.
pub struct Driver {
    pub cfg: DriverConfig,
    pub ring: Arc<Ring>,
    gain: Arc<AtomicU32>,
    volume: i32,
    #[cfg(feature = "device")]
    device: Option<device::Held>,
    #[cfg(feature = "device")]
    opening: Option<std::sync::mpsc::Receiver<Result<(u32, device::Held), String>>>,
    dev_ok: bool,
    attempted: bool,
    wav: Option<Wav>,
    last_tick: Option<Instant>,
}

const FRAME_DUR: Duration = Duration::from_millis(5);
const STALL: Duration = Duration::from_millis(100);

impl Driver {
    pub fn new(cfg: DriverConfig) -> Self {
        let ring = Arc::new(Ring::new());
        ring.set_target(cfg.target_frames, cfg.target_frames, cfg.max_frames);
        let gain = Arc::new(AtomicU32::new(1.0f32.to_bits()));
        #[cfg(feature = "device")]
        let opening = (cfg.device && !cfg.turbo).then(|| device::Device::open_async(ring.clone(), gain.clone(), cfg.buffer_ms));
        Driver {
            cfg,
            ring,
            gain,
            volume: -1,
            #[cfg(feature = "device")]
            device: None,
            #[cfg(feature = "device")]
            opening,
            dev_ok: false,
            attempted: false,
            wav: None,
            last_tick: None,
        }
    }

    /// Whether an output device is playing.
    pub fn device_open(&self) -> bool {
        self.dev_ok
    }

    pub fn volume(&mut self) -> i32 {
        if self.volume < 0 {
            self.volume = load_volume();
        }
        self.volume
    }

    /// gw_Audio_SetVolume: live gain change + audio.cfg save.
    pub fn set_volume(&mut self, ax: &mut Ax, percent: i32) {
        self.volume = percent.clamp(0, 100);
        self.apply_volume();
        let _ = save_volume(self.volume);
        if let Some(l) = ax.log.as_mut() {
            l(&format!("gw: AX: master volume {}%", self.volume));
        }
    }

    /// A live gain change only (the port's settings overlay; it saves the value itself).
    pub fn set_volume_live(&mut self, percent: i32) {
        self.volume = percent.clamp(0, 100);
        self.apply_volume();
    }

    fn apply_volume(&mut self) {
        let v = self.volume() as f32 / 100.0;
        self.gain.store(v.to_bits(), Ordering::Relaxed);
    }

    fn open(&mut self, ax: &mut Ax) {
        if self.attempted {
            return;
        }
        self.attempted = true;
        let log = |ax: &mut Ax, s: String| {
            if let Some(l) = ax.log.as_mut() {
                l(&s);
            }
        };
        if self.cfg.turbo {
            log(ax, "gw: AX: turbo mute (virtual 5 ms DSP clock, no output device)".into());
            return;
        }
        ax.nofx = self.cfg.nofx;
        if let Some(p) = self.cfg.dump.clone() {
            match Wav::create(&p) {
                Ok(w) => {
                    self.wav = Some(w);
                    log(ax, format!("gw: AX: capturing mix to {p}"));
                }
                Err(_) => log(ax, format!("gw: AX: cannot open {p} for capture")),
            }
        }
        if !self.cfg.device {
            return;
        }
        #[cfg(feature = "device")]
        {
            if self.opening.is_none() {
                self.opening = Some(device::Device::open_async(self.ring.clone(), self.gain.clone(), self.cfg.buffer_ms));
            }
            self.poll_open(ax);
        }
    }

    /// The device being opened ([`device::Device::open_async`]): adopt it once it is ready. Until then the frames
    /// run on the wall clock as without a device (never blocks the game thread).
    #[cfg(feature = "device")]
    fn poll_open(&mut self, ax: &mut Ax) {
        let log = |ax: &mut Ax, s: String| {
            if let Some(l) = ax.log.as_mut() {
                l(&s);
            }
        };
        let Some(rx) = self.opening.as_ref() else { return };
        let opened = match rx.try_recv() {
            Ok(r) => r,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("audio open thread ended".into()),
        };
        self.opening = None;
        match opened {
            Ok((rate, d)) => {
                self.apply_volume();
                self.device = Some(d);
                self.dev_ok = true;
                log(ax, format!("gw: AX: master volume {}%", self.volume()));
                log(
                    ax,
                    format!(
                        "gw: AX: audio device open ({rate} Hz device, 32 kHz stereo s16 mix, {} ms cushion{}, device buffer {})",
                        self.cfg.target_frames * 5,
                        if self.cfg.low_latency { ", low-latency mode" } else { "" },
                        if self.cfg.buffer_ms > 0 { format!("{} ms requested", self.cfg.buffer_ms) } else { "default".into() }
                    ),
                );
            }
            Err(e) => log(ax, format!("gw: AX: audio device failed: {e}")),
        }
    }

    /// gw_ax_frame_tick, called once per video frame on the game thread. `callback_registered`
    /// is whether the game has registered its AX callback (AXRegisterCallback; nothing runs
    /// until it has). `sub_frame` is called once per 5 ms sub-frame to generate: it runs the
    /// game's AX callback, then `ax.run_frame(aram)`, and returns that output. Returns the
    /// number of sub-frames generated.
    pub fn tick(
        &mut self,
        ax: &mut Ax,
        callback_registered: bool,
        mut sub_frame: impl FnMut(&mut Ax) -> [i16; FRAME_SAMPLES * 2],
    ) -> u32 {
        let now = Instant::now();
        if !callback_registered {
            self.last_tick = Some(now);
            return 0;
        }
        if !self.attempted {
            self.open(ax);
        }
        #[cfg(feature = "device")]
        if self.opening.is_some() {
            self.poll_open(ax);
        }
        let mut n = if self.dev_ok {
            let buffered = self.ring.len() / FRAME_SAMPLES as u32;
            self.ring.target().saturating_sub(buffered)
        } else {
            let Some(last) = self.last_tick else {
                self.last_tick = Some(now);
                return 0;
            };
            let elapsed = now.saturating_duration_since(last);
            if elapsed > STALL {
                self.last_tick = Some(now);
                return 0;
            }
            let n = (elapsed.as_nanos() / FRAME_DUR.as_nanos()) as u32;
            self.last_tick = Some(last + FRAME_DUR * n);
            n
        };
        n = n.min(MAX_BURST);
        ax.underruns = self.ring.underruns.load(Ordering::Relaxed);
        ax.overruns = self.ring.overruns.load(Ordering::Relaxed);
        for _ in 0..n {
            let out = sub_frame(ax);
            if self.cfg.turbo {
                continue; // the voice writeback is game-visible; the sound is not
            }
            if let Some(w) = self.wav.as_mut() {
                w.write(&out);
            }
            if self.dev_ok {
                self.ring.push(&out);
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_roundtrip_and_overrun() {
        let r = Ring::new();
        assert_eq!(r.pop(), None);
        r.push(&[1, -2, 3, -4]);
        assert_eq!(r.len(), 2);
        assert_eq!(r.pop(), Some((1, -2)));
        assert_eq!(r.pop(), Some((3, -4)));
        let big = vec![7i16; (RING_FRAMES as usize + 10) * 2];
        r.push(&big);
        assert_eq!(r.len(), RING_FRAMES);
        assert_eq!(r.overruns.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn master_volume_env_clamps_and_sets_gain() {
        // Only test touching MELEE_VOLUME (process-global).
        for (env, want) in [("55", 55), ("250", 100), ("-3", 0), ("abc", 0), ("0", 0)] {
            std::env::set_var("MELEE_VOLUME", env);
            assert_eq!(load_volume(), want, "MELEE_VOLUME={env}");
        }
        std::env::set_var("MELEE_VOLUME", "40");
        let mut d = Driver::new(DriverConfig { turbo: true, target_frames: 0, max_frames: 0, buffer_ms: 0, low_latency: false, nofx: false, dump: None, device: false });
        assert_eq!(d.volume(), 40);
        d.apply_volume();
        assert_eq!(f32::from_bits(d.gain.load(Ordering::Relaxed)), 0.4);
        std::env::remove_var("MELEE_VOLUME");
    }

    #[test]
    fn atoi_like_c() {
        assert_eq!(atoi("  42x"), 42);
        assert_eq!(atoi("-7"), -7);
        assert_eq!(atoi("abc"), 0);
    }

    #[test]
    fn wall_clock_tick_without_device() {
        let mut ax = Ax::new();
        let mut d = Driver::new(DriverConfig { turbo: false, target_frames: 12, max_frames: 20, buffer_ms: 0, low_latency: false, nofx: false, dump: None, device: false });
        let aram = vec![0u8; 64];
        assert_eq!(d.tick(&mut ax, true, |ax| *ax.run_frame(&aram)), 0); // first call arms the clock
        std::thread::sleep(Duration::from_millis(22));
        let n = d.tick(&mut ax, true, |ax| *ax.run_frame(&aram));
        assert!((4..=MAX_BURST).contains(&n), "{n}");
    }
}
