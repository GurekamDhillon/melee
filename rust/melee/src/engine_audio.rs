//! Sound for the engine: the game's own `lbAudioAx_*` calls (every TU's hook) forwarded to `melee_audio::sim`, the sound stack that
//! the C audio proofs verified sample for sample (banks, voices, positional objects, the AX mixer). The engine runs the real scene
//! code, so the announcer, entry sounds, pause duck and time-up calls come from the game's own HUD / scene functions, not from a
//! hand-written schedule. Glue only: no game logic here (the install table is `engine_audio_gen.rs`, from `gen_audio_bridge.py`).
//!
//! Timeline (the C build's scripted VS load, `start_match_audio`): scene init and the bank loads run at DSP sub-frame 0, the BGM
//! starts 3 video frames later, and the first logic frame comes `PREROLL` frames after that. The bridge runs the 3 frames inside the
//! BGM call; the driver runs the preroll before the first tick.
#![allow(non_snake_case, clippy::missing_safety_doc)]

use melee_audio::{lbaudio as lb, sim};
use melee_types::HSD_GObj;

fn with<R>(d: R, f: impl FnOnce(&mut melee_audio::Sound) -> R) -> R {
    sim::with(f).unwrap_or(d)
}

/// A video frame of the scene LOAD (before the first logic tick): the audio update the scene loop runs, then the DSP.
pub fn load_frame() {
    sim::game_frame();
    dsp_frame();
}

/// Where the DSP clock output goes (`--audio-wav`): every sub-frame's 160 stereo samples.
pub static mut WAV: Option<Vec<i16>> = None;
static mut BGM_STARTED: bool = false;

/// The DSP sub-frames of one logic frame (`lbAudioAx_80027DF8`, the deferred sfx and loop voices, runs inside the scene tick).
pub fn dsp_frame() {
    unsafe {
        match (*std::ptr::addr_of_mut!(WAV)).as_mut() {
            Some(w) => {
                sim::virtual_frame_into(w);
            }
            None => {
                sim::virtual_frame();
            }
        }
    }
}

pub fn ax_8002305C(ckind: i32, which: i32) -> i32 { lb::lb_8002305c(ckind, which) }
pub fn ax_80023090(idx: i32) -> i32 { lb::lb_80023090(idx) }
pub fn ax_800233EC(id: i32) -> i32 { sim::lbaudioax_800233ec(id) }
pub fn ax_80023694() -> i32 { with(0, |s| s.lb_audio_ax_80023694()) }
pub fn ax_800236B8(voice: i32) -> i32 { with(0, |s| s.lb_audio_ax_800236b8(voice)) }
pub fn ax_800236DC() -> i32 { with(0, |s| s.lb_audio_ax_800236dc()) }
/// `lbAudioAx_80023710(voice)`: whether a voice is still playing (HSD_AudioSFXCheck).
pub fn ax_80023710(voice: i32) -> i32 { with(false, |s| s.audio_sfx_check(voice)) as i32 }
pub fn ax_80023730() -> i32 { with(false, |s| s.lb_audio_ax_80023730()) as i32 }
pub fn ax_800237A8(id: i32, vol: i32, pan: i32) -> i32 {
    if std::env::var_os("MELEE_AUDIO_BT").is_some_and(|v| v.to_str().and_then(|v| v.parse::<i32>().ok()) == Some(id)) {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let names: Vec<&str> = bt.lines().filter_map(|l| l.trim().split_once(": ").map(|x| x.1)).filter(|l| l.starts_with("melee_")).take(8).collect();
        eprintln!("audio: sfx {id} from {}", names.join(" < "));
    }
    sim::lbaudioax_800237a8(id, vol, pan)
}
pub fn ax_80023870(id: i32, vol: i32, pan: i32, track: i32) -> i32 {
    if std::env::var_os("MELEE_AUDIO_BT").is_some_and(|v| v.to_str().and_then(|v| v.parse::<i32>().ok()) == Some(id)) {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let names: Vec<&str> = bt.lines().filter_map(|l| l.trim().split_once(": ").map(|x| x.1)).filter(|l| l.starts_with("melee_")).take(8).collect();
        eprintln!("audio: voice sfx {id} track {track} from {}", names.join(" < "));
    }
    sim::lbaudioax_80023870(id, vol, pan, track)
}
pub fn ax_80023B24(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_80023b24(id)) }
/// `lbAudioAx_80023F28(bgm)`: the match BGM. The first call of a run starts 3 video frames after the scene's bank loads (the C's load).
pub fn ax_80023F28(id: i32) -> i32 {
    if std::env::var_os("MELEE_AUDIO_DBG").is_some() { eprintln!("audio: BGM {id}"); }
    unsafe {
        if !BGM_STARTED {
            BGM_STARTED = true;
            for _ in 0..3 {
                load_frame();
            }
        }
    }
    sim::play_bgm_id(id)
}
pub fn ax_800240B4(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_800240b4(id)) }
pub fn ax_80024304(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_80024304(id)) }
pub fn ax_80024B94(handle: i32, pitch: i32) -> i32 { sim::lbaudioax_80024b94(handle, pitch) }
/// `lbAudioAx_80027DF8`: the per-frame audio update, called from the scene's own frame (so it runs where the C runs it).
pub fn ax_80027DF8() {
    FRAME_RAN.store(true, std::sync::atomic::Ordering::Relaxed);
    sim::game_frame()
}
/// Set when the engine's scene loop ran the per-frame audio update; the live player's own tick then skips its copy.
static FRAME_RAN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Whether the scene loop ran `lbAudioAx_80027DF8` since the last call (and clear it).
pub fn take_frame_ran() -> bool { FRAME_RAN.swap(false, std::sync::atomic::Ordering::Relaxed) }
pub fn ax_80024030(n: i32) {
    if std::env::var_os("MELEE_AUDIO_BT").is_some() {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let names: Vec<&str> = bt.lines().filter_map(|l| l.trim().split_once(": ").map(|x| x.1)).filter(|l| l.starts_with("melee_")).take(8).collect();
        eprintln!("audio: menu sound {n} from {}", names.join(" < "));
    }
    with((), |s| s.lb_audio_ax_80024030(n))
}
pub fn ax_8002411C(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_8002411c(id)) }
pub fn ax_80024184(id: i32, vol: i32, pan: i32, track: i32) -> i32 { with(0, |s| s.lb_audio_ax_80024184(id, vol, pan, track)) }
pub fn ax_8002438C(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_8002438c(id)) }
pub fn ax_800243F4(id: i32) -> i32 { with(0, |s| s.lb_audio_ax_800243f4(id)) }
pub fn ax_800245D4(v: i32) { with((), |s| s.lb_audio_ax_800245d4(v)) }
pub fn ax_800245F4(v: i32) -> i32 { with(0, |s| s.lb_audio_ax_800245f4(v)) }
pub fn ax_80024614(v: i32) { with((), |s| s.lb_audio_ax_80024614(v)) }
pub fn ax_80024634(v: i32) { with((), |s| s.lb_audio_ax_80024634(v)) }
pub fn ax_80024B1C(voice: i32, pan: i32) { with((), |s| s.lb_audio_ax_80024b1c(voice, pan)) }
pub fn ax_80024B58(voice: i32, vol: i32) { with((), |s| s.lb_audio_ax_80024b58(voice, vol)) }
pub fn ax_80024BD0() -> i32 { with(false, |s| s.lb_audio_ax_80024bd0()) as i32 }
pub fn ax_80024C08(mode: u32) -> i32 { with(false, |s| s.lb_audio_ax_80024c08(mode)) as i32 }
pub fn ax_80024C84() { with((), |s| s.lb_audio_ax_80024c84()) }
pub fn ax_80024D50() { with((), |s| s.lb_audio_ax_80024d50()) }
/// `lbAudioAx_80024D78(which)`: the echo byte of the current stage's row (`grkind` = `Stage_8022519C(Stage_80225194())`).
pub fn ax_80024D78(which: i32) {
    let grkind = unsafe { melee_world::tu::ground::stage_info.grkind as i32 };
    with((), |s| s.lb_audio_ax_80024d78(which as usize, grkind))
}
pub fn ax_80024DC4(id: i32) { sim::lbaudioax_80024dc4(id) }
pub fn ax_80024E50(pause: i32) { with((), |s| s.lb_audio_ax_80024e50(pause != 0)) }
pub fn ax_80024E84(on: i32) { with((), |s| s.lb_audio_ax_80024e84(on != 0)) }
pub fn ax_80024F08() { with((), |s| s.lb_audio_ax_80024f08()) }
pub fn ax_80024F6C() { with((), |s| s.lb_audio_ax_80024f6c()) }
/// A sound object is an integer handle here; the game stores the GObj pointer it gets back and hands it to the other calls.
#[allow(clippy::too_many_arguments)]
pub fn ax_800263E8(f1: f32, owner: *mut HSD_GObj, mode: u32, sfx: i32, a: i32, b: i32, p0: i32, p1: i32, p2: i32, s8: i32, sc: i32) -> *mut HSD_GObj {
    if std::env::var_os("MELEE_AUDIO_BT").is_some() {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let names: Vec<&str> = bt.lines().filter_map(|l| l.trim().split_once(": ").map(|x| x.1)).filter(|l| l.starts_with("melee_")).take(6).collect();
        eprintln!("audio: sound object sfx {sfx} mode {mode} from {}", names.join(" < "));
    }
    sim::lbaudioax_800263e8(f1, owner as usize, mode as i32, sfx, a, b, p0, p1, p2, s8, sc) as usize as *mut HSD_GObj
}
pub fn ax_800264E4(obj: *mut HSD_GObj) -> i32 { sim::lbaudioax_800264e4(obj as usize as i32) }
pub fn ax_80026510(owner: *mut HSD_GObj) -> i32 { sim::lbaudioax_80026510(owner as usize) as i32 }
pub fn ax_800265C4(owner: *mut HSD_GObj, voice: i32) -> i32 { sim::lbaudioax_800265c4(owner as usize, voice) as i32 }
pub fn ax_80026E84(ckind: i32) -> u64 { let r = lb::lb_80026e84(ckind); if std::env::var_os("MELEE_AUDIO_DBG").is_some() { eprintln!("audio: 80026E84({ckind}) = {r:x}"); } r }
pub fn ax_80026EBC(grkind: u32) -> u64 { let r = lb::lb_80026ebc_grkind(grkind as i32); if std::env::var_os("MELEE_AUDIO_DBG").is_some() { eprintln!("audio: 80026EBC({grkind}) = {r:x}"); } r }
pub fn ax_80026F2C(flags: u32) { with((), |s| s.lb_audio_ax_80026f2c(flags)) }
pub fn ax_8002702C(flags: u32, mask: u64) {
    if std::env::var_os("MELEE_AUDIO_DBG").is_some() { eprintln!("audio: banks flags {flags:x} mask {mask:x}"); }
 with((), |s| s.lb_audio_ax_8002702c(flags, mask)) }
/// `lbAudioAx_80027648()`: wait for the requested banks. melee_audio's bank loads complete inside the request, so there is nothing to wait for
/// (the TU default would spin on melee_runtime's own, unmixed, bank manager).
pub fn ax_80027648() {}
pub fn ax_80027168() { with((), |s| s.lb_audio_ax_80027168()) }
pub fn ax_80027AB0(id: i32) {
    let us = melee_runtime::tu::lblanguage::lbLang_IsSettingUS();
    with((), |s| s.lb_audio_ax_80027ab0(id, us))
}
/// `lbAudioAx_8002785C()` (gmvs.c, the VS scene): the players' and the stage's SSM banks. The C's own bank manager lives in the lbaudio_ax TU's
/// statics; the sound stack that owns the banks is melee_audio's, so the scene's request goes there with what the game says is in the match.
pub fn ax_8002785C() {
    use melee_fighter::tu::player as pl;
    let mut ck = Vec::new();
    for i in 0..4 {
        if pl::Player_GetPlayerSlotType(i) as i32 != melee_types::Gm_PKind_NA as i32 {
            ck.push(pl::Player_GetPlayerCharacter(i) as i32);
        }
    }
    let (stkind, grkind) = unsafe {
        let st = melee_world::tu::stage::Stage_80225194();
        (st as i32, melee_world::tu::stage::Stage_8022519C(st) as i32)
    };
    if std::env::var_os("MELEE_AUDIO_DBG").is_some() { eprintln!("audio: match banks ckinds {ck:?} stkind {stkind} grkind {grkind}"); }
    sim::load_match_banks(&ck, stkind, grkind);
}
/// The scene's BGM pick (`gmvs.c`: `Stage_80225074`, which draws once and calls `lbAudioAx_80023F28`): the engine's boot flow does not reach it yet.
pub fn start_stage_bgm() {
    unsafe {
        use melee_world::tu::{ground, stage};
        let a = melee_runtime::tu::random::global::state().logic_seed();
        // The C golden machine's save has every character unlocked (FD's row rolls for its alternate track: one draw).
        melee_world::tu::ground::ext::hook::gm_80164ABC = Some(|| 1);
        melee_world::tu::ground::ext::hook::gm_IsCKindUnlocked = Some(|_| 1);
        let mut bgm = 0i32;
        // `Stage_80225074(0)`: no alternate-music button held -> flags 4
        let tmp = ground::Ground_801C28AC(stage::Stage_80225194(), 4, &mut bgm);
        ax_80023F28(bgm);
        ground::Ground_801C5A84(bgm);
        ground::Ground_801C5AA4(tmp);
        if std::env::var_os("MELEE_AUDIO_DBG").is_some() {
            eprintln!("audio: stage BGM {bgm}, seed {a:08x} -> {:08x}", melee_runtime::tu::random::global::state().logic_seed());
        }
    }
}
pub fn ax_80027DBC() { with((), |s| s.lb_audio_ax_80027dbc()) }
pub fn ax_80028B6C() { with((), |s| s.lb_audio_ax_80028b6c()) }
pub fn ax_80028B90() { with((), |s| s.lb_audio_ax_80028b90()) }

/// Boot the sound stack from the disc (`Sound::boot`: lbAudioAx's first init, smash2.sem, main.ssm), what the positional objects read
/// from the game (owner class and position, the real camera, the game RNG), and the hooks.
pub fn install(iso: &str, trace: bool) -> Result<(), String> {
    boot_sound(iso, trace)?;
    attach(true);
    Ok(())
}

/// Boot the sound stack from the disc and what the positional objects read from the game (no engine hooks yet: the menus use this alone).
pub fn boot_sound(iso: &str, trace: bool) -> Result<(), String> {
    let mut snd = melee_audio::Sound::new();
    snd.boot(Box::new(melee_audio::sound::DiscFs::open(iso)?), lb::boot_bank_sizes())?;
    if trace {
        snd.trace = Some(Vec::new());
    }
    sim::install(snd);
    sim::set_env(melee_audio::posobj::PosEnv {
        class_of: Some(|g| unsafe { (*(g as *const HSD_GObj)).classifier as i32 }),
        fighter_pos: Some(|g| unsafe {
            let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            melee_items::tu::ftlib::ftLib_GetPos(g as *mut _, &mut p);
            [p.x, p.y, p.z]
        }),
        item_pos: Some(|g| unsafe {
            let mut p = melee_types::Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            melee_items::tu::it_26B1::it_8026B294(g as *mut _, &mut p);
            [p.x, p.y, p.z]
        }),
        camera: Some(|| unsafe {
            let (mut l, mut c, mut r) = (0.0f32, 0.0f32, 0.0f32);
            melee_world::tu::camera::Camera_800307D0(&mut l, &mut c, &mut r);
            (l, c, r)
        }),
        randi: Some(melee_runtime::tu::random::global::HSD_Randi),
    });
    Ok(())
}

/// Route the game's lbAudioAx calls of the NEXT engine boot to the installed sound (`sim::install` done by the caller), and run the
/// C's scene start (`start_match_audio`): scene init and the announcer bank, before the game's own bank loads. `wait_bgm`: the first
/// BGM call runs the 3 load frames the captures have (a live player pumps its own DSP and passes false).
pub fn attach(wait_bgm: bool) {
    unsafe {
        BGM_STARTED = !wait_bgm;
        melee_engine::engine::POST_LINK = Some(post_link);
    }
    sim::scene_init();
    sim::load_announcer();
}

/// A bridge function's result as the hook's declared return type (some TUs declare a hook `()` that others declare `int`).
pub trait Cast<A> {
    fn cast(a: A) -> Self;
}
impl Cast<i32> for () { fn cast(_: i32) {} }
impl Cast<i32> for i32 { fn cast(a: i32) -> i32 { a } }
impl Cast<i32> for u32 { fn cast(a: i32) -> u32 { a as u32 } }
impl Cast<i32> for bool { fn cast(a: i32) -> bool { a != 0 } }
impl Cast<u64> for () { fn cast(_: u64) {} }
impl Cast<u64> for u64 { fn cast(a: u64) -> u64 { a } }
impl Cast<*mut HSD_GObj> for () { fn cast(_: *mut HSD_GObj) {} }
impl Cast<*mut HSD_GObj> for *mut HSD_GObj { fn cast(a: *mut HSD_GObj) -> *mut HSD_GObj { a } }
pub fn cast<A, B: Cast<A>>(a: A) -> B { B::cast(a) }

/// After the link table: the TU hooks (generated) and the fighter code's own `melee_fighter::hooks` lbAudioAx hooks, which the link table
/// would otherwise point at the lbaudio_ax TU.
fn post_link() {
    crate::engine_audio_gen::install_gen();
    // melee_modes::tu_ext's sound-bank calls (gm_1879's intro scenes): not in the generated table (no ext::hook statics)
    unsafe {
        use melee_modes::tu_ext as t;
        t::HOOK_lbAudioAx_80026E84 = Some(|a0| ax_80026E84(a0));
        t::HOOK_lbAudioAx_80026F2C = Some(|a0| ax_80026F2C(a0));
        t::HOOK_lbAudioAx_8002702C = Some(|a0, a1| ax_8002702C(a0, a1));
        t::HOOK_lbAudioAx_80027168 = Some(|| ax_80027168());
        t::HOOK_lbAudioAx_80027648 = Some(|| ax_80027648());
    }
    // the todo_fns! cutscenes of Adventure (vi0801, vi1101, vi1201v1 / v2, vi1202): their sound calls
    unsafe {
        melee_items::tu::vi0801::ext::hook::lbAudioAx_800236DC = Some(|| { let _ = ax_800236DC(); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80023F28 = Some(|a0| { let _ = ax_80023F28(a0 as _); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80024E50 = Some(|a0| { let _ = ax_80024E50(a0 as _); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80026F2C = Some(|a0| { let _ = ax_80026F2C(a0 as _); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_8002702C = Some(|a0, a1| { let _ = ax_8002702C(a0 as _, a1 as _); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80027168 = Some(|| { let _ = ax_80027168(); });
        melee_items::tu::vi0801::ext::hook::lbAudioAx_80027648 = Some(|| { let _ = ax_80027648(); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_800236DC = Some(|| { let _ = ax_800236DC(); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_800237A8 = Some(|a0, a1, a2| { let _r = ax_800237A8(a0 as _, a1 as _, a2 as _); _r as _ });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80023F28 = Some(|a0| { let _ = ax_80023F28(a0 as _); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80024E50 = Some(|a0| { let _ = ax_80024E50(a0 as _); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80026E84 = Some(|a0| { let _r = ax_80026E84(a0 as _); _r as _ });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80026F2C = Some(|a0| { let _ = ax_80026F2C(a0 as _); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_8002702C = Some(|a0, a1| { let _ = ax_8002702C(a0 as _, a1 as _); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80027168 = Some(|| { let _ = ax_80027168(); });
        melee_items::tu::vi1101::ext::hook::lbAudioAx_80027648 = Some(|| { let _ = ax_80027648(); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_800236DC = Some(|| { let _ = ax_800236DC(); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_800237A8 = Some(|a0, a1, a2| { let _r = ax_800237A8(a0 as _, a1 as _, a2 as _); _r as _ });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80023F28 = Some(|a0| { let _ = ax_80023F28(a0 as _); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80024E50 = Some(|a0| { let _ = ax_80024E50(a0 as _); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80026F2C = Some(|a0| { let _ = ax_80026F2C(a0 as _); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_8002702C = Some(|a0, a1| { let _ = ax_8002702C(a0 as _, a1 as _); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80027168 = Some(|| { let _ = ax_80027168(); });
        melee_items::tu::vi1201v1::ext::hook::lbAudioAx_80027648 = Some(|| { let _ = ax_80027648(); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80023F28 = Some(|a0| { let _ = ax_80023F28(a0 as _); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80024E50 = Some(|a0| { let _ = ax_80024E50(a0 as _); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80026F2C = Some(|a0| { let _ = ax_80026F2C(a0 as _); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_8002702C = Some(|a0, a1| { let _ = ax_8002702C(a0 as _, a1 as _); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80027168 = Some(|| { let _ = ax_80027168(); });
        melee_items::tu::vi1201v2::ext::hook::lbAudioAx_80027648 = Some(|| { let _ = ax_80027648(); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80023694 = Some(|| { let _ = ax_80023694(); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_800236DC = Some(|| { let _ = ax_800236DC(); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80023F28 = Some(|a0| { let _ = ax_80023F28(a0 as _); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80024E50 = Some(|a0| { let _ = ax_80024E50(a0 as _); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80026F2C = Some(|a0| { let _ = ax_80026F2C(a0 as _); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_8002702C = Some(|a0, a1| { let _ = ax_8002702C(a0 as _, a1 as _); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80027168 = Some(|| { let _ = ax_80027168(); });
        melee_items::tu::vi1202::ext::hook::lbAudioAx_80027648 = Some(|| { let _ = ax_80027648(); });
    }
    // the scene loop's per-frame audio update (gmscene.c calls lbAudioAx_80027DF8 every iteration): the sound objects' pan / volume
    // steps, the deferred sfx, the two loop voices
    unsafe { melee_modes::tu::gmscene::ext::AUDIO_FRAME = Some(ax_80027DF8) };
    unsafe { melee_modes::tu::gmscene::ext::AUDIO_PROCS = Some(sim::sound_obj_procs) };
    {
        use melee_fighter::hooks as h;
        unsafe {
            h::LBAUDIOAX_800237A8 = Some(sim::lbaudioax_800237a8);
            h::LBAUDIOAX_80023870 = Some(sim::lbaudioax_80023870);
            h::LBAUDIOAX_80024B94 = Some(sim::lbaudioax_80024b94);
            h::LBAUDIOAX_800233EC = Some(sim::lbaudioax_800233ec);
            h::LBAUDIOAX_80023130 = Some(lb::lb_80023130);
            h::LBAUDIOAX_800230C8 = Some(sim::lbaudioax_800230c8);
            h::LBAUDIOAX_80023220 = Some(lb::lb_80023220);
            h::LBAUDIOAX_80024DC4 = Some(sim::lbaudioax_80024dc4);
            h::LBAUDIOAX_80024FDC = Some(sim::lbaudioax_80024fdc);
            h::LBAUDIOAX_80024FF4 = Some(sim::lbaudioax_80024ff4);
            h::LBAUDIOAX_8002500C = Some(sim::lbaudioax_8002500c);
            h::LBAUDIOAX_80025038 = Some(sim::lbaudioax_80025038);
            h::LBAUDIOAX_800263E8 = Some(|dir, g, beh, sfx, a, b, p0, p1, p2, s8, sc| sim::lbaudioax_800263e8(dir, g as usize, beh as i32, sfx, a, b, p0, p1, p2, s8, sc));
            h::LBAUDIOAX_800264E4 = Some(sim::lbaudioax_800264e4);
            h::LBAUDIOAX_80026510 = Some(|g| sim::lbaudioax_80026510(g as usize));
            h::LBAUDIOAX_800265C4 = Some(|g, voice| sim::lbaudioax_800265c4(g as usize, voice));
            h::HSD_RANDI = Some(|n| unsafe { melee_runtime::tu::random::global::HSD_Randi(n) });
        }
    }
}

/// `melee_modes::tu::ext_hook::AUDIO_ROUTE`: the menus' (string-hook TUs') lbAudioAx calls into the same sound stack as the match.
pub fn route_by_name(name: &str, a: &[u64]) -> Option<u64> {
    let a = |i: usize| a.get(i).copied().unwrap_or(0);
    let a = [a(0), a(1), a(2), a(3), a(4), a(5), a(6), a(7), a(8), a(9), a(10)];
    Some(match name {
        "lbAudioAx_80023090" => ax_80023090(a[0] as i32) as i64 as u64,
        "lbAudioAx_80023694" => ax_80023694() as i64 as u64,
        "lbAudioAx_800236DC" => ax_800236DC() as i64 as u64,
        "lbAudioAx_800237A8" => ax_800237A8(a[0] as i32, a[1] as i32, a[2] as i32) as i64 as u64,
        "lbAudioAx_80023870" => ax_80023870(a[0] as i32, a[1] as i32, a[2] as i32, a[3] as i32) as i64 as u64,
        "lbAudioAx_80023B24" => ax_80023B24(a[0] as i32) as i64 as u64,
        "lbAudioAx_80023F28" => ax_80023F28(a[0] as i32) as i64 as u64,
        "lbAudioAx_80024B94" => ax_80024B94(a[0] as i32, a[1] as i32) as i64 as u64,
        "lbAudioAx_80024030" => { ax_80024030(a[0] as i32); 0 }
        "lbAudioAx_8002411C" => ax_8002411C(a[0] as i32) as i64 as u64,
        "lbAudioAx_8002438C" => ax_8002438C(a[0] as i32) as i64 as u64,
        "lbAudioAx_800245F4" => ax_800245F4(a[0] as i32) as i64 as u64,
        "lbAudioAx_80024614" => { ax_80024614(a[0] as i32); 0 }
        "lbAudioAx_80024BD0" => ax_80024BD0() as i64 as u64,
        "lbAudioAx_80024C08" => ax_80024C08(a[0] as u32) as i64 as u64,
        "lbAudioAx_80024E50" => { ax_80024E50(a[0] as i32); 0 }
        "lbAudioAx_80024F08" => { ax_80024F08(); 0 }
        "lbAudioAx_80024F6C" => { ax_80024F6C(); 0 }
        "lbAudioAx_80026E84" => ax_80026E84(a[0] as i32) as i64 as u64,
        "lbAudioAx_80026EBC" => ax_80026EBC(a[0] as u32) as i64 as u64,
        "lbAudioAx_80026F2C" => { ax_80026F2C(a[0] as u32); 0 }
        "lbAudioAx_8002702C" => { ax_8002702C(a[0] as u32, a[1] as u64); 0 }
        "lbAudioAx_80027648" => { ax_80027648(); 0 }
        "lbAudioAx_80027168" => { ax_80027168(); 0 }
        "lbAudioAx_80027AB0" => { ax_80027AB0(a[0] as i32); 0 }
        "lbAudioAx_80027DBC" => { ax_80027DBC(); 0 }
        "lbAudioAx_80028B6C" => { ax_80028B6C(); 0 }
        "lbAudioAx_80028B90" => { ax_80028B90(); 0 }
        _ => return None,
    })
}
