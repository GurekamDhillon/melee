//! Plays a 1 s 440 Hz tone through one AX voice (PCM16 in a synthetic ARAM) on the default cpal
//! device, driven by the real frame driver at 60 Hz. `cargo run -p melee_audio --example tone`.
//! Prints whether the device opened, sub-frames generated, and ring under/overruns.

use std::time::{Duration, Instant};

use melee_audio::output::{Driver, DriverConfig};
use melee_audio::Ax;

fn main() {
    // 1 s of 440 Hz PCM16, big-endian like ARAM.
    let mut aram = Vec::new();
    for i in 0..32000 {
        let s = ((i as f32 * 440.0 * std::f32::consts::TAU / 32000.0).sin() * 12000.0) as i16;
        aram.extend_from_slice(&s.to_be_bytes());
    }
    let mut ax = Ax::new();
    ax.log = Some(Box::new(|s| println!("{s}")));
    let v = ax.acquire_voice(1, 0, 0, |_, _| {}).unwrap();
    let mut addr = [0u8; 0x10];
    addr[1] = 1; // loop
    addr[3] = 0xA; // PCM16
    addr[8..12].copy_from_slice(&(aram.len() as u32 * 2 - 1).to_be_bytes()); // end (nibbles)
    ax.set_voice_addr(v, &addr);
    ax.set_voice_adpcm(v, &[0u8; 0x28]);
    ax.set_voice_src(v, &[0, 1, 0, 0]); // ratio 1.0
    ax.set_voice_ve(v, &[0x7F, 0xFF, 0, 0]);
    let mut mix = [0u8; 0x24];
    mix[0..2].copy_from_slice(&0x7FFFu16.to_be_bytes());
    mix[4..6].copy_from_slice(&0x7FFFu16.to_be_bytes());
    ax.set_voice_mix(v, &mix);
    ax.set_voice_state(v, 1);

    let mut d = Driver::new(DriverConfig::from_env(false));
    let start = Instant::now();
    let mut total = 0;
    while start.elapsed() < Duration::from_millis(1200) {
        total += d.tick(&mut ax, true, |ax| *ax.run_frame(&aram));
        std::thread::sleep(Duration::from_micros(16_667));
    }
    println!(
        "device open: {}, sub-frames: {total} ({} ms), underruns {}, overruns {}",
        d.device_open(),
        total * 5,
        d.ring.underruns.load(std::sync::atomic::Ordering::Relaxed),
        d.ring.overruns.load(std::sync::atomic::Ordering::Relaxed)
    );
}
