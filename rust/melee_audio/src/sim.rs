//! The sound the simulation talks to: one process-wide [`Sound`] (the C's file statics), the
//! `lbAudioAx_*` functions the ported game code reaches through its hook tables (`melee_fighter::
//! hooks::LBAUDIOAX_800237A8` and friends take these as plain `fn` pointers), and [`Output`], the
//! once-per-video-frame tick that runs the 5 ms AX callbacks and feeds the device.
//!
//! Before [`install`] every call answers as a sound system with nothing loaded would: a start
//! returns -1, setters return false/0. The game logic then still runs, but a voice handle it
//! stores (`Fighter.x2160`) is -1 where the C build has a real handle; matching those needs the
//! sound booted from the same disc and ticked on the same schedule as the C build.

use std::sync::{Mutex, MutexGuard};

use crate::output::{Driver, DriverConfig};
use crate::Sound;

static SOUND: Mutex<Option<Sound>> = Mutex::new(None);

fn lock() -> MutexGuard<'static, Option<Sound>> {
    SOUND.lock().unwrap_or_else(|e| e.into_inner())
}

/// Make `s` the sound the game calls. Replaces any previous one.
pub fn install(s: Sound) {
    *lock() = Some(s);
}

/// Take the installed sound back out.
pub fn uninstall() -> Option<Sound> {
    lock().take()
}

pub fn is_installed() -> bool {
    lock().is_some()
}

/// Run `f` on the installed sound.
pub fn with<R>(f: impl FnOnce(&mut Sound) -> R) -> Option<R> {
    lock().as_mut().map(f)
}

/// `lbAudioAx_800237A8(sfx, vol, pan)`: start a sound effect; the voice handle or -1.
pub fn lbaudioax_800237a8(sfx: i32, vol: i32, pan: i32) -> i32 {
    with(|s| s.lb_audio_ax_800237a8(sfx, vol, pan)).unwrap_or(-1)
}

/// `lbAudioAx_80023870(sfx, vol, pan, track)`.
pub fn lbaudioax_80023870(sfx: i32, vol: i32, pan: i32, track: i32) -> i32 {
    with(|s| s.lb_audio_ax_80023870(sfx, vol, pan, track)).unwrap_or(-1)
}

/// `lbAudioAx_80024B94(handle, pitch)`.
pub fn lbaudioax_80024b94(handle: i32, pitch: i32) -> i32 {
    with(|s| s.lb_audio_ax_80024b94(handle, pitch)).unwrap_or(0)
}

/// `HSD_AudioSFXKeyOff(handle)`.
pub fn sfx_key_off(handle: i32) -> bool {
    with(|s| s.audio_sfx_key_off(handle)).unwrap_or(false)
}

/// `HSD_AudioSFXCheck(handle)`.
pub fn sfx_check(handle: i32) -> bool {
    with(|s| s.audio_sfx_check(handle)).unwrap_or(false)
}

/// Start a BGM file (`menu01.hps`) from the disc's /audio/, as lbAudioAx_80023F28 does.
pub fn play_bgm(file: &str) -> bool {
    with(|s| s.lb_play_bgm_file(file, 0x7F, 1)).unwrap_or(false)
}

/// `lbAudioAx_800233EC(id)`: Kirby's copy-ability voice remap. Without a sound, the id unchanged
/// (as the C answers with kirbytm.ssm not loaded and `id` outside bank 0x21).
pub fn lbaudioax_800233ec(id: i32) -> i32 {
    with(|s| s.lb_audio_ax_800233ec(id)).unwrap_or(id)
}

/// `lbAudioAx_80024DC4(id)`: defer a sound effect.
pub fn lbaudioax_80024dc4(id: i32) {
    with(|s| s.lb_audio_ax_80024dc4(id));
}
pub fn lbaudioax_80024fdc() {
    with(|s| s.lb_audio_ax_80024fdc());
}
pub fn lbaudioax_80024ff4() {
    with(|s| s.lb_audio_ax_80024ff4());
}
pub fn lbaudioax_8002500c(n: i32) {
    with(|s| s.lb_audio_ax_8002500c(n));
}
pub fn lbaudioax_80025038(n: i32) {
    with(|s| s.lb_audio_ax_80025038(n));
}

/// `lbAudioAx_800230C8` with the hook's `&mut` out-parameters.
pub fn lbaudioax_800230c8(i: i32, lo: &mut i32, hi: &mut i32) -> i32 {
    crate::lbaudio::lb_800230c8(i, Some(lo), Some(hi))
}

/// Start BGM `id` (`lbAudioAx_80023F28`); 1 = nothing started.
pub fn play_bgm_id(id: i32) -> i32 {
    with(|s| s.lb_audio_ax_80023f28(id)).unwrap_or(1)
}

/// `lbAudioAx_8002785C` (VS): load the players' and the stage's SSM banks.
pub fn load_match_banks(ckinds: &[i32], stkind: i32, grkind: i32) {
    with(|s| s.lb_load_match_banks(ckinds, stkind, grkind));
}

/// One game frame of audio: `lbAudioAx_80027DF8` (deferred sfx, the two loop voices).
pub fn game_frame() {
    with(|s| {
        s.step_sound_objs();
        s.lb_audio_ax_80027df8()
    });
}

static VCLOCK: Mutex<u32> = Mutex::new(0);

/// A deterministic DSP schedule for headless runs: 10 sub-frames per 3 video frames (5 ms
/// against 1/60 s), no device. The C build paces AX by the device or the wall clock instead, so
/// a handle that depends on when a voice was freed can differ from a C run; this keeps the
/// native run itself reproducible. Returns the sub-frames run.
pub fn virtual_frame() -> u32 {
    let mut c = VCLOCK.lock().unwrap_or_else(|e| e.into_inner());
    *c += 10;
    let n = *c / 3;
    *c %= 3;
    with(|s| {
        for _ in 0..n {
            s.sub_frame();
        }
    });
    n
}

/// [`virtual_frame`] that also appends each sub-frame's 160 interleaved s16 stereo samples (the
/// mix `shim_ax.c` clips and writes to `MELEE_AUDIO_DUMP`) to `out`.
pub fn virtual_frame_into(out: &mut Vec<i16>) -> u32 {
    let mut c = VCLOCK.lock().unwrap_or_else(|e| e.into_inner());
    *c += 10;
    let n = *c / 3;
    *c %= 3;
    with(|s| {
        for _ in 0..n {
            out.extend_from_slice(&s.sub_frame());
        }
    });
    n
}

/// The output side: the frame driver (device clock, WAV dump, turbo mute) over the installed
/// sound. Call [`Output::tick`] once per video frame on the game thread.
pub struct Output {
    pub driver: Driver,
}

impl Output {
    pub fn new(cfg: DriverConfig) -> Self {
        Output { driver: Driver::new(cfg) }
    }

    /// gw_ax_frame_tick: generate the sub-frames the device (or the wall clock) is owed.
    /// Returns how many 5 ms sub-frames ran.
    pub fn tick(&mut self) -> u32 {
        let mut g = lock();
        let Some(s) = g.as_mut() else { return 0 };
        let registered = s.ax_callback_registered;
        // Driver::tick wants the Ax and a closure over it; the closure needs the whole Sound,
        // so lend the Ax out and run the synth around it.
        let mut ax = std::mem::take(&mut s.ax);
        let n = self.driver.tick(&mut ax, registered, |ax| {
            std::mem::swap(&mut s.ax, ax);
            let out = s.sub_frame();
            std::mem::swap(&mut s.ax, ax);
            out
        });
        s.ax = ax;
        n
    }
}

/// Set what the positional sound objects read from the game (class, positions, camera, RNG).
pub fn set_env(env: crate::posobj::PosEnv) {
    with(|s| s.lb.env = env);
}

/// `lbAudioAx_800263E8(...)`: start a sound object owned by `owner` (a GObj pointer as an
/// integer); the object's handle as an i32 (0 where the C returns NULL).
#[allow(clippy::too_many_arguments)]
pub fn lbaudioax_800263e8(
    f1: f32, owner: usize, mode: i32, sfx: i32, start_vol: i32, end_vol: i32, pan_left: i32,
    pan_right: i32, end_frame: i32, track: i32, voice: i32,
) -> i32 {
    with(|s| s.lb_audio_ax_800263e8(f1, owner, mode, sfx, start_vol, end_vol, pan_left, pan_right, end_frame, track, voice))
        .flatten()
        .map(|h| h as i32)
        .unwrap_or(0)
}

/// `lbAudioAx_800264E4(obj)`: the object's voice handle (-1 for none).
pub fn lbaudioax_800264e4(handle: i32) -> i32 {
    with(|s| s.lb_audio_ax_800264e4((handle > 0).then_some(handle as usize))).unwrap_or(-1)
}

/// `lbAudioAx_80026510(owner)`.
pub fn lbaudioax_80026510(owner: usize) -> bool {
    with(|s| s.lb_audio_ax_80026510(owner)).unwrap_or(false)
}

/// `lbAudioAx_800265C4(owner, voice)`.
pub fn lbaudioax_800265c4(owner: usize, voice: i32) -> bool {
    with(|s| s.lb_audio_ax_800265c4(owner, voice)).unwrap_or(false)
}

/// The volume settings the menus and the pause code reach (`lbAudioAx_800245D4` music,
/// `800245F4` sfx, `80024614` voice, `80025064` voice/sfx on-off, `80024E84` pause duck,
/// `80024E50` BGM pause, `80024F08` / `80024F6C` full silence on/off, `80024C84` scene reset).
pub fn set_music_volume(v: i32) {
    with(|s| s.lb_audio_ax_800245d4(v));
}
pub fn set_sfx_volume(v: i32) -> i32 {
    with(|s| s.lb_audio_ax_800245f4(v)).unwrap_or(0)
}
pub fn set_voice_volume(v: i32) {
    with(|s| s.lb_audio_ax_80024614(v));
}
pub fn set_voice_sfx_on(a: bool, b: bool) {
    with(|s| s.lb_audio_ax_80025064(a, b));
}
pub fn set_pause_duck(on: bool) {
    with(|s| s.lb_audio_ax_80024e84(on));
}
pub fn pause_bgm(pause: bool) {
    with(|s| s.lb_audio_ax_80024e50(pause));
}
pub fn silence(on: bool) {
    with(|s| if on { s.lb_audio_ax_80024f08() } else { s.lb_audio_ax_80024f6c() });
}
pub fn scene_reset() {
    with(|s| s.lb_audio_ax_80024c84());
}
pub fn scene_init() {
    with(|s| s.lb_audio_ax_80028690());
}
