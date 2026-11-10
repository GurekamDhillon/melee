//! Brawl music and sound effects from the player's own Brawl disc, mixed on top of the game's output.
//!
//! Presentation only: nothing here feeds the sim. The mix is added after the AX mixer's sub-frame (see
//! [`crate::Sound::sub_frame`]), and while nothing Brawl plays it adds nothing, so captures and the audio hash are
//! unchanged. Files load on a worker thread; a missing disc or an unknown name is silent.
//!
//! - [`play_music`]: a named stream sound (`snd_bgm_...`) loops on the music channel; the game's own .hps music is
//!   stopped and held off while it plays.
//! - [`play_sfx`]: a named (or `#id`) wave sound plays once.

pub mod reader;

use reader::Pcm;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// Anything Brawl playing (or loading). Read lock-free by the mixer: the off path is one load.
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// Brawl music owns the music channel (the game's .hps stream is held off).
static MUSIC_ON: AtomicBool = AtomicBool::new(false);
/// Bumped per music request so a slow load cannot start after a newer request or a stop.
static MUSIC_GEN: AtomicU32 = AtomicU32::new(0);

struct Voice {
    pcm: Arc<Pcm>,
    /// Position in source frames, 16.16 fixed point.
    pos: u64,
    step: u64,
    gain: i32, // /256
    looping: bool,
    /// Fade-out per sub-frame (gain units); 0 = none.
    fade: i32,
}

impl Voice {
    fn new(pcm: Arc<Pcm>, gain: i32, looping: bool) -> Voice {
        let step = ((pcm.rate as u64) << 16) / crate::RATE as u64;
        Voice { pcm, pos: 0, step: step.max(1), gain, looping, fade: 0 }
    }

    /// Adds one sub-frame. Returns false once finished.
    fn mix(&mut self, out: &mut [i32; crate::FRAME_SAMPLES * 2]) -> bool {
        let p = &*self.pcm;
        let n = p.frames();
        if n == 0 {
            return false;
        }
        let ch = p.channels;
        for i in 0..crate::FRAME_SAMPLES {
            let mut idx = (self.pos >> 16) as usize;
            if idx + 1 >= n {
                match (self.looping, p.loop_start) {
                    (true, Some(ls)) => {
                        self.pos -= ((n - ls) as u64) << 16;
                        idx = (self.pos >> 16) as usize;
                    }
                    (true, None) => {
                        self.pos &= 0xFFFF;
                        idx = 0;
                    }
                    _ => return false,
                }
            }
            let frac = (self.pos & 0xFFFF) as i32;
            let at = |f: usize, c: usize| p.samples[f * ch + c.min(ch - 1)] as i32;
            let nx = (idx + 1).min(n - 1);
            for c in 0..2 {
                let a = at(idx, c);
                let b = at(nx, c);
                let s = a + (((b - a) * frac) >> 16);
                out[i * 2 + c] += (s * self.gain) >> 8;
            }
            self.pos += self.step;
        }
        if self.fade > 0 {
            self.gain -= self.fade;
            if self.gain <= 0 {
                return false;
            }
        }
        true
    }
}

#[derive(Default)]
struct Mix {
    music: Option<Voice>,
    sfx: Vec<Voice>,
}

fn mix() -> &'static Mutex<Mix> {
    static M: OnceLock<Mutex<Mix>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(Mix::default()))
}

fn refresh_active(m: &Mix) {
    ACTIVE.store(m.music.is_some() || !m.sfx.is_empty() || LOADING.load(Ordering::Relaxed) > 0, Ordering::Relaxed);
}

static LOADING: AtomicU32 = AtomicU32::new(0);

/// Adds the Brawl voices to one 5 ms sub-frame of the game's output. A no-op while nothing Brawl plays.
pub fn mix_into(out: &mut [i16; crate::FRAME_SAMPLES * 2]) {
    if !ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let Ok(mut m) = mix().try_lock() else { return };
    let mut acc = [0i32; crate::FRAME_SAMPLES * 2];
    if let Some(v) = m.music.as_mut() {
        if !v.mix(&mut acc) {
            m.music = None;
            MUSIC_ON.store(false, Ordering::Relaxed);
        }
    }
    m.sfx.retain_mut(|v| v.mix(&mut acc));
    for (o, a) in out.iter_mut().zip(acc.iter()) {
        *o = (*o as i32 + a).clamp(-32768, 32767) as i16;
    }
    refresh_active(&m);
}

/// Whether Brawl music holds the music channel (the game's own stream start is skipped while it does).
pub fn music_on() -> bool {
    MUSIC_ON.load(Ordering::Relaxed)
}

/// Whether the player's Brawl sound files are present.
pub fn present() -> bool {
    static P: OnceLock<bool> = OnceLock::new();
    *P.get_or_init(|| reader::sound_dir().is_some())
}

fn archive() -> Option<&'static reader::Rsar> {
    static A: OnceLock<Option<reader::Rsar>> = OnceLock::new();
    A.get_or_init(|| {
        let d = reader::sound_dir()?;
        match reader::Rsar::open(&d.join(reader::ARCHIVE)) {
            Ok(a) => Some(a),
            Err(e) => {
                eprintln!("brawl audio: sound archive unreadable: {e}");
                None
            }
        }
    })
    .as_ref()
}

fn env_pct(k: &str, d: i32) -> i32 {
    std::env::var(k).ok().and_then(|v| v.trim().parse().ok()).unwrap_or(d).clamp(0, 200)
}

/// Resolves a name (`snd_bgm_...` / `snd_se_...`), a `#id`, or a bare stream file stem (`X18`).
fn lookup<'a>(a: &'a reader::Rsar, name: &str) -> Option<&'a reader::SoundEntry> {
    if let Some(id) = name.strip_prefix('#').and_then(|n| n.parse().ok()) {
        return a.by_id(id);
    }
    a.find(name)
}

fn load_music(name: &str) -> Result<Pcm, String> {
    let a = archive().ok_or("no Brawl sound archive")?;
    let dir = reader::sound_dir().ok_or("no Brawl sound folder")?;
    let rel = match lookup(a, name) {
        Some(s) => a.stream_file(s).ok_or("not a stream sound")?,
        None if !name.contains(['/', '\\', '.']) && name.len() <= 8 => format!("strm/{name}.brstm"),
        None => return Err("unknown stream".into()),
    };
    let rel = rel.trim_start_matches('/');
    if rel.contains("..") {
        return Err("bad stream path".into());
    }
    let bytes = std::fs::read(dir.join(rel)).map_err(|e| format!("stream not readable ({})", e.kind()))?;
    reader::decode_rstm(&bytes)
}

fn load_sfx(name: &str) -> Result<(Pcm, u8), String> {
    let a = archive().ok_or("no Brawl sound archive")?;
    let s = lookup(a, name).ok_or("unknown sound")?;
    Ok((a.decode_wave_sound(s)?, s.volume))
}

fn worker(f: impl FnOnce() + Send + 'static) {
    LOADING.fetch_add(1, Ordering::Relaxed);
    let spawned = crate::output::on_host_heap(|| std::thread::Builder::new().name("brawl-audio-load".into()).spawn(move || {
        f();
        LOADING.fetch_sub(1, Ordering::Relaxed);
    }));
    if spawned.is_err() {
        LOADING.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Starts a looping Brawl stream on the music channel (replacing any). Returns false when the files are missing
/// (the request is then silently dropped). The game's own music stops (see [`music_on`]).
fn play_music_impl(name: &str) -> bool {
    if !present() {
        return false;
    }
    let g = MUSIC_GEN.fetch_add(1, Ordering::Relaxed) + 1;
    MUSIC_ON.store(true, Ordering::Relaxed);
    crate::sim::with(|s| s.stream_stop());
    let name = name.to_string();
    worker(move || match load_music(&name) {
        Ok(p) => {
            let mut m = mix().lock().unwrap_or_else(|e| e.into_inner());
            if MUSIC_GEN.load(Ordering::Relaxed) == g {
                let gain = env_pct("MELEE_BRAWL_MUSIC_VOL", 60) * 256 / 100;
                m.music = Some(Voice::new(Arc::new(p), gain, true));
                ACTIVE.store(true, Ordering::Relaxed);
            }
        }
        Err(e) => {
            eprintln!("brawl audio: music '{name}' not played: {e}");
            if MUSIC_GEN.load(Ordering::Relaxed) == g {
                MUSIC_ON.store(false, Ordering::Relaxed);
            }
        }
    });
    ACTIVE.store(true, Ordering::Relaxed);
    true
}

/// Fades the Brawl music out (over `frames` video frames, 0 = at once) and gives the music channel back.
fn stop_music_impl(frames: u32) {
    MUSIC_GEN.fetch_add(1, Ordering::Relaxed);
    MUSIC_ON.store(false, Ordering::Relaxed);
    let mut m = mix().lock().unwrap_or_else(|e| e.into_inner());
    if frames == 0 {
        m.music = None;
    } else if let Some(v) = m.music.as_mut() {
        let subs = (frames as i32 * 10 / 3).max(1); // 16.7 ms frames, 5 ms sub-frames
        v.fade = (v.gain / subs).max(1);
    }
    refresh_active(&m);
}

/// Plays a Brawl wave sound once. Returns false when the files are missing.
fn play_sfx_impl(name: &str) -> bool {
    if !present() {
        return false;
    }
    let name = name.to_string();
    worker(move || match load_sfx(&name) {
        Ok((p, vol)) => {
            let mut m = mix().lock().unwrap_or_else(|e| e.into_inner());
            if m.sfx.len() < 16 {
                let gain = env_pct("MELEE_BRAWL_SFX_VOL", 70) * 2 * vol.max(1) as i32 / 100 * 256 / 255;
                m.sfx.push(Voice::new(Arc::new(p), gain, false));
                ACTIVE.store(true, Ordering::Relaxed);
            }
        }
        Err(e) => eprintln!("brawl audio: sfx '{name}' not played: {e}"),
    });
    ACTIVE.store(true, Ordering::Relaxed);
    true
}

/// Stops every Brawl voice at once (scene teardown).
fn stop_all_impl() {
    MUSIC_GEN.fetch_add(1, Ordering::Relaxed);
    MUSIC_ON.store(false, Ordering::Relaxed);
    let mut m = mix().lock().unwrap_or_else(|e| e.into_inner());
    m.music = None;
    m.sfx.clear();
    refresh_active(&m);
}

/// See the module docs; runs on the system heap (callers may be inside the game's arena scope).
pub fn play_music(name: &str) -> bool {
    crate::output::on_host_heap(|| play_music_impl(name))
}


/// See the module docs; runs on the system heap (callers may be inside the game's arena scope).
pub fn play_sfx(name: &str) -> bool {
    crate::output::on_host_heap(|| play_sfx_impl(name))
}


/// See the module docs; runs on the system heap (callers may be inside the game's arena scope).
pub fn stop_music(frames: u32) {
    crate::output::on_host_heap(|| stop_music_impl(frames))
}


/// See the module docs; runs on the system heap (callers may be inside the game's arena scope).
pub fn stop_all() {
    crate::output::on_host_heap(|| stop_all_impl())
}

