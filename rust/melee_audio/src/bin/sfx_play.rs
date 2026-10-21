//! Plays a sound effect (and optionally a music track) from the user's disc through the ported
//! sound stack: smash2.sem + main.ssm loaded as lbAudioAx boots them, the sound started with
//! `lbAudioAx_800237A8` (the call the fighter code makes), mixed by the AX port and played on the
//! default output device in real time.
//!
//!   cargo run -p melee_audio --release --bin sfx_play -- [--iso DISC.iso] [--id N] [--repeat K]
//!       [--bgm menu01.hps] [--seconds S] [--wav out.wav] [--no-device] [--scan N]
//!
//! The disc defaults to $GW_ISO_VANILLA. Nothing read from it is written anywhere except the
//! optional --wav capture (a local file the user asked for; keep it out of git).
//! `--scan N` skips playback and prints, for script ids 0..N, the handle, the length in ms and
//! the peak of each sound mixed alone (headless, no device).

use std::time::{Duration, Instant};

use melee_audio::output::DriverConfig;
use melee_audio::sim::{self, Output};
use melee_audio::sound::{DiscFs, Sound};

/// The three SFX banks' ARAM sizes: retail main.ssm's slot (offsets_arr_803BC4E4[0]) and room
/// for the fighter/stage banks, 0x700000 in all like lbl_804D3870's default.
const BANKS: [i32; 3] = [2045824, 0x180000, 0x700000 - 2045824 - 0x180000];

fn boot(iso: &str) -> Result<Sound, String> {
    let mut s = Sound::new();
    s.boot(Box::new(DiscFs::open(iso)?), BANKS)?;
    Ok(s)
}

/// Mix one sound alone, headless: (handle, ms until silent, peak).
fn measure(iso_sound: &mut Sound, id: i32) -> (i32, u32, i32) {
    let h = iso_sound.lb_audio_ax_800237a8(id, 0x7F, 0x40);
    let mut peak = 0i32;
    let mut last_loud = 0u32;
    for f in 0..2000u32 {
        let out = iso_sound.sub_frame();
        let p = out.iter().map(|&x| (x as i32).abs()).max().unwrap_or(0);
        if p > 0 {
            last_loud = f + 1;
        }
        peak = peak.max(p);
        if f > 40 && iso_sound.active_nodes() == 0 && iso_sound.live_count() == 0 {
            break;
        }
    }
    iso_sound.audio_sfx_key_off_all();
    for _ in 0..200 {
        iso_sound.sub_frame();
    }
    (h, last_loud * 5, peak)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let has = |k: &str| args.iter().any(|a| a == k);
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("sfx_play: no disc (--iso or GW_ISO_VANILLA)");
        std::process::exit(2);
    };
    let parse = |s: String| -> i32 {
        if let Some(h) = s.strip_prefix("0x") {
            i32::from_str_radix(h, 16).unwrap_or(0)
        } else {
            s.parse().unwrap_or(0)
        }
    };

    let mut s = match boot(&iso) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sfx_play: {e}");
            std::process::exit(1);
        }
    };
    let sem = s.sem.as_ref().unwrap();
    println!(
        "sfx_play: smash2.sem {} banks, {} scripts; main.ssm in bank 0 ({} sounds)",
        sem.bank_first.len(),
        sem.scripts.len(),
        s.loaded_sounds()
    );

    if let Some(n) = get("--scan").map(parse) {
        for id in 0..n {
            let (h, ms, peak) = measure(&mut s, id);
            println!("id {id:5} (0x{id:04X}): handle {h:6}  {ms:5} ms  peak {peak}");
        }
        return;
    }

    let id = get("--id").map(parse).unwrap_or(0xC1);
    let repeat = get("--repeat").map(parse).unwrap_or(3).max(1);
    let seconds: f32 = get("--seconds").and_then(|x| x.parse().ok()).unwrap_or(if has("--bgm") { 8.0 } else { 3.0 });

    // Headless measurement first, so the run says what should be heard.
    let (h0, ms, peak) = measure(&mut s, id);
    println!("sfx_play: sound {id} (0x{id:X}) mixed headless: handle {h0}, {ms} ms, peak {peak}");

    s.ax.log = Some(Box::new(|m| println!("{m}")));
    sim::install(s);
    let mut cfg = DriverConfig::from_env(false);
    if let Some(w) = get("--wav") {
        cfg.dump = Some(w);
    }
    cfg.device = !has("--no-device");
    let mut out = Output::new(cfg);

    if let Some(bgm) = get("--bgm") {
        let ok = sim::play_bgm(&bgm);
        println!("sfx_play: bgm /audio/{bgm}: {}", if ok { "started" } else { "failed" });
    }

    let start = Instant::now();
    let mut frames = 0u32;
    let mut next_play = Duration::ZERO;
    let gap = Duration::from_secs_f32((seconds / repeat as f32).max(0.2));
    let mut played = 0;
    let mut peak_live = 0i32;
    while start.elapsed() < Duration::from_secs_f32(seconds) {
        if played < repeat && start.elapsed() >= next_play {
            let h = sim::lbaudioax_800237a8(id, 0x7F, 0x40);
            println!("sfx_play: lbAudioAx_800237A8({id}, 127, 64) -> handle {h} at {:.2}s", start.elapsed().as_secs_f32());
            played += 1;
            next_play += gap;
        }
        frames += out.tick();
        peak_live = peak_live.max(sim::with(|s| s.ax.output().iter().map(|&x| (x as i32).abs()).max().unwrap_or(0)).unwrap_or(0));
        std::thread::sleep(Duration::from_micros(16_667));
    }
    let d = &out.driver;
    println!(
        "sfx_play: device open: {}, {frames} sub-frames ({} ms), underruns {}, overruns {}",
        d.device_open(),
        frames * 5,
        d.ring.underruns.load(std::sync::atomic::Ordering::Relaxed),
        d.ring.overruns.load(std::sync::atomic::Ordering::Relaxed)
    );
    let _ = peak_live;
}
