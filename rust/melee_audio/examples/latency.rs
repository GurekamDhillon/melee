//! Output latency probe: plays a quiet 440 Hz tone through one AX voice on the default device with the live path's
//! pacing (the mixer thread's 1 ms top-up loop at time-critical priority), and prints the measured latency: from a mixed
//! 5 ms sub-frame being handed to the ring to the device playing its first sample (ring residency + the device queue as
//! cpal reports it). No input, no window. Honours MELEE_VOLUME, MELEE_AUDIO_LOW_LATENCY / MELEE_AUDIO_MODE=low,
//! MELEE_AUDIO_LATENCY_MS, MELEE_AUDIO_BUFFER_MS.
//!   cargo run -p melee_audio --release --example latency -- [seconds]

use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use melee_audio::output::{Driver, DriverConfig};
use melee_audio::Ax;

fn main() {
    let secs: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(10);
    let mut aram = Vec::new();
    for i in 0..32000 {
        let s = ((i as f32 * 440.0 * std::f32::consts::TAU / 32000.0).sin() * 4000.0) as i16;
        aram.extend_from_slice(&s.to_be_bytes());
    }
    let mut ax = Ax::new();
    ax.log = Some(Box::new(|s| println!("{s}")));
    let v = ax.acquire_voice(1, 0, 0, |_, _| {}).unwrap();
    let mut addr = [0u8; 0x10];
    addr[1] = 1;
    addr[3] = 0xA;
    addr[8..12].copy_from_slice(&(aram.len() as u32 * 2 - 1).to_be_bytes());
    ax.set_voice_addr(v, &addr);
    ax.set_voice_adpcm(v, &[0u8; 0x28]);
    ax.set_voice_src(v, &[0, 1, 0, 0]);
    ax.set_voice_ve(v, &[0x7F, 0xFF, 0, 0]);
    let mut mix = [0u8; 0x24];
    mix[0..2].copy_from_slice(&0x7FFFu16.to_be_bytes());
    mix[4..6].copy_from_slice(&0x7FFFu16.to_be_bytes());
    ax.set_voice_mix(v, &mix);
    ax.set_voice_state(v, 1);

    melee_audio::sim::raise_mixer_priority();
    let mut d = Driver::new(DriverConfig::from_env(false));
    let start = Instant::now();
    let mut opened: Option<Instant> = None;
    let mut warmed = false;
    loop {
        d.tick(&mut ax, true, |ax| *ax.run_frame(&aram));
        // the clock starts at the device's first consumed sub-frame (a stream can take a second or more to start pulling)
        if opened.is_none() && d.device_open() && d.ring.lat_n.load(Relaxed) > 0 {
            opened = Some(Instant::now());
        }
        let since = opened.map_or(Duration::ZERO, |t| t.elapsed());
        if since > Duration::from_secs(secs + 1) || start.elapsed() > Duration::from_secs(secs + 10) {
            break;
        }
        // one second after that: drop the start-up (the first callback fills the whole device buffer)
        if !warmed && since > Duration::from_secs(1) {
            d.ring.reset_latency();
            d.ring.underruns.store(0, Relaxed);
            d.ring.underrun_events.store(0, Relaxed);
            warmed = true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let l = d.ring.latency();
    println!(
        "latency: mean {:.2} ms, min {:.2}, max {:.2} over {} sub-frames; device {} Hz, largest callback {} frames ({:.2} ms), latest {} frames; cushion now {} ms; underruns {} frames in {} events, overruns {}",
        l.mean_ms,
        l.min_ms,
        l.max_ms,
        l.samples,
        l.device_rate,
        l.callback_frames,
        l.callback_frames as f64 * 1000.0 / l.device_rate.max(1) as f64,
        d.ring.cb_frames_last.load(Relaxed),
        l.target_ms,
        d.ring.underruns.load(Relaxed),
        d.ring.underrun_events.load(Relaxed),
        d.ring.overruns.load(Relaxed)
    );
}
