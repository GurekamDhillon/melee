//! The fighter sounds of the native VS sim: `ft_PlaySFX` and the voice / channel starters
//! (`melee_fighter::ft0877` / `ft0881`) called where the C game calls them, through the
//! `lbAudioAx` hooks (`install_hooks`). With no sound booted (`melee_audio::sim`) every start
//! answers -1 and nothing is heard, but the RNG draw `ft_PlaySFX` makes for a pitch-randomised
//! id (332..=370 after `ft_80087D0C`'s remap) is the C's, so the sim's seed stays in step.
//!
//! `HSD_Randi` inside those functions is a plain-`fn` hook; it reads the sim's `HsdRand` through a
//! pointer set for the duration of each call (`with_rand`).

use super::Player;
use melee_fighter::hooks as h;
use melee_runtime::random::HsdRand;
use std::sync::Once;

static mut RAND: *mut HsdRand = std::ptr::null_mut();

fn randi(n: i32) -> i32 {
    // SAFETY: only reached from `with_rand`, which keeps RAND pointing at the live `HsdRand`.
    unsafe { (*RAND).randi(n) }
}

static HOOKS: Once = Once::new();

/// Set every `lbAudioAx` hook the fighter code calls to `melee_audio::sim`, and `HSD_Randi` to the
/// sim's generator. Idempotent.
pub fn install_hooks() {
    use melee_audio::{lbaudio as lb, sim};
    HOOKS.call_once(|| unsafe {
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
        h::LBAUDIOAX_800263E8 = Some(|dir, g, beh, sfx, a, b, p0, p1, p2, s8, sc| {
            sim::lbaudioax_800263e8(dir, g as usize, beh as i32, sfx, a, b, p0, p1, p2, s8, sc)
        });
        h::LBAUDIOAX_800264E4 = Some(sim::lbaudioax_800264e4);
        h::LBAUDIOAX_80026510 = Some(|g| sim::lbaudioax_80026510(g as usize));
        h::LBAUDIOAX_800265C4 = Some(|g, voice| sim::lbaudioax_800265c4(g as usize, voice));
        h::HSD_RANDI = Some(randi);
    });
}

/// Run `f` with `HSD_Randi` drawing from `rand`.
fn with_rand<R>(rand: &mut HsdRand, f: impl FnOnce() -> R) -> R {
    install_hooks();
    unsafe { RAND = rand as *mut HsdRand };
    let r = f();
    unsafe { RAND = std::ptr::null_mut() };
    r
}

/// `ft_PlaySFX(fp, sfx, vol, pan)`.
pub fn play_sfx(p: &mut Player, rand: &mut HsdRand, sfx: i32, vol: u8, pan: u8) {
    with_rand(rand, || melee_fighter::ft0877::play_sfx(&mut p.fp, sfx, vol, pan));
}

/// `ftAction_80071B50`'s dispatch: behaviours 0-6 start `sfx` on a channel, 10-15 stop one.
pub fn sound_command(p: &mut Player, rand: &mut HsdRand, behavior: u32, sfx: i32, vol: u8, pan: u8) {
    use melee_fighter::{ft0877, ft0881};
    let f = &mut *p.fp;
    with_rand(rand, || match behavior {
        0 => ft0877::play_sfx(f, sfx, vol, pan),
        1 => ft0881::ft_80088478(f, sfx, vol, pan),
        2 => ft0881::ft_800881d8(f, sfx, vol, pan),
        3 => ft0881::ft_80088510(f, sfx, vol, pan),
        4 => ft0881::ft_800885a8(f, sfx, vol, pan),
        5 => ft0881::ft_80088640(f, sfx, vol, pan),
        6 => ft0881::ft_80088328(f, sfx, vol, pan),
        10 => ft0881::ft_80088828(f),
        11 => ft0881::ft_80088770(f),
        12 => ft0881::ft_80088884(f),
        13 => ft0881::ft_800888e0(f),
        14 => ft0881::ft_8008893c(f),
        15 => ft0881::ft_800887cc(f),
        _ => {}
    });
}

/// The stand-in `Fighter_GObj` of `p` (owner identity for the positional sound objects,
/// `user_data` = the Fighter), made on first use.
pub fn gobj_of(p: &mut Player) -> *mut melee_types::HSD_GObj {
    if p.snd_gobj == 0 {
        let mut g = Box::new(melee_types::HSD_GObj::zeroed());
        g.classifier = 4;
        p.snd_gobj = Box::into_raw(g) as usize;
    }
    let g = p.snd_gobj as *mut melee_types::HSD_GObj;
    unsafe { (*g).user_data = &mut *p.fp as *mut melee_types::Fighter as *mut core::ffi::c_void };
    g
}

/// `ft_80088C5C(gobj)`: stop the fighter's sound objects and channels.
pub fn ft_80088c5c(p: &mut Player) {
    let g = gobj_of(p);
    melee_fighter::ft0881::ft_80088c5c(g);
}

/// `ftCo_800D38B8(fp, id)`: a KO voice on the voice channel, repeated through the deferred list
/// unless `x2225_b6`.
pub fn ko_voice(p: &mut Player, id: i32) {
    melee_fighter::ft0881::ft_800881d8(&mut p.fp, id, 0x7F, 0x40);
    if p.fp.x2225_b6() == 0 {
        melee_fighter::ft0877::ft_8008805c(&p.fp, id);
    }
}

/// `Fighter_ChangeMotionState` without `Ft_MF_KeepSfx`: `ft_80088884`, `ft_800888E0`,
/// `ft_800887CC` stop the fighter's three looping-sound channels (tracks 0x42 / 0x4E / 0x2A + the
/// player's pair).
pub fn change_state_stops(p: &mut Player) {
    install_hooks();
    let f = &mut *p.fp;
    melee_fighter::ft0881::ft_80088884(f);
    melee_fighter::ft0881::ft_800888e0(f);
    melee_fighter::ft0881::ft_800887cc(f);
}

/// `ft_80088A50(fp)`: stop both channels of every voice bank and the deferred counters.
pub fn stop_all_channels(p: &mut Player) {
    install_hooks();
    melee_fighter::ft0881::ft_80088a50(&mut p.fp);
}
