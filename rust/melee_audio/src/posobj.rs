//! The positional / scripted sound objects of `lbaudio_ax.c` (`lbAudioAx_800263E8` and its
//! `fn_80025FAC` / `fn_800262A0` / pan and volume callbacks). In the C each is a GObj with a
//! per-frame proc over a `lbAudioAx_UserData`; here it is a [`SoundObj`] in `LbState::objs`,
//! stepped once per game frame by [`Sound::step_sound_objs`] (the proc phase). Owner objects are
//! opaque `usize` ids (the GObj pointer); what the C reads from them (class, position) and the
//! camera and the game RNG come in through [`PosEnv`].
//!
//! Float arithmetic follows the C (single precision); float-to-int casts follow the x86 build
//! (`cvttss2si`: NaN and out-of-range give `i32::MIN`).

use crate::lbaudio::SFX_NONE;
use crate::Sound;

pub const CLASS_FIGHTER: i32 = 4;
pub const CLASS_ITEM: i32 = 6;

/// What the sound objects read from the game. All optional; an unset one answers as the C would
/// with nothing there (no owner class: the object frees itself on its first step).
#[derive(Clone, Copy, Debug, Default)]
pub struct PosEnv {
    /// `gobj->classifier`.
    pub class_of: Option<fn(usize) -> i32>,
    /// `ftLib_80086644(gobj, &pos)`: the fighter's position (x, y, z).
    pub fighter_pos: Option<fn(usize) -> [f32; 3]>,
    /// `it_8026B294(gobj, &pos)`.
    pub item_pos: Option<fn(usize) -> [f32; 3]>,
    /// `Camera_800307D0`: (left, center, right).
    pub camera: Option<fn() -> (f32, f32, f32)>,
    /// `HSD_Randi(n)`.
    pub randi: Option<fn(i32) -> i32>,
}

/// `lbAudioAx_UserData`.
#[derive(Clone, Debug)]
pub struct SoundObj {
    pub id_handle: usize,
    pub owner: usize,
    /// xC: the handler index 0..9.
    pub mode: i32,
    pub sfx: i32,
    pub start_vol: i32,
    pub end_vol: i32,
    pub vol: i32,
    pub pan_left: i32,
    pub pan_right: i32,
    pub pan: i32,
    pub voice: i32,
    pub cur: i32,
    pub end: i32,
    pub x3c: f32,
    pub track: i32,
    pub x44: bool,
}

fn cvtt(f: f32) -> i32 {
    if f.is_nan() || f >= 2147483648.0 || f < -2147483648.0 {
        i32::MIN
    } else {
        melee_core::ppc::fctiwz_f32(f)
    }
}

/// `calcPan(current, end, left, right)`.
pub fn calc_pan(mut current: i32, end: i32, left: i32, right: i32) -> i32 {
    if current > end {
        current = end;
    }
    if left != right {
        let mut result = if left <= right {
            left as f32 + current as f32 * ((right as f32) - left as f32) / end as f32
        } else {
            right as f32 + current as f32 * ((left as f32) - right as f32) / end as f32
        };
        if result < 0.0 {
            result = 0.0;
        }
        if result > 127.0 {
            result = 127.0;
        }
        return cvtt(result);
    }
    64
}

impl Sound {
    /// `soundGetPosition` + `fn_800251EC`: the pan from the owner's x between the camera's edges.
    /// True = the object frees itself (no owner, or an owner that is neither fighter nor item).
    fn obj_pan_from_owner(&mut self, i: usize) -> bool {
        let env = self.lb.env;
        let o = self.lb.objs[i].owner;
        if o == 0 {
            return true;
        }
        let pos = match env.class_of.map(|f| f(o)) {
            Some(CLASS_FIGHTER) => env.fighter_pos.map(|f| f(o)),
            Some(CLASS_ITEM) => env.item_pos.map(|f| f(o)),
            _ => None,
        };
        let Some(pos) = pos else { return true };
        let (cl, cc, cr) = env.camera.map(|f| f()).unwrap_or((0.0, 0.0, 0.0));
        let ob = &mut self.lb.objs[i];
        let mut pan = 64.0f32;
        let x = pos[0];
        if cl < cc && cc < cr && cl < x && x < cr {
            if cc < x {
                if ob.pan_right > 0x40 {
                    let ratio = (x - cc) / (cr - cc);
                    let temp = (ob.pan_right - 0x40) as f32 * ratio;
                    pan = 64.0 + temp.abs();
                }
            } else if cc > x && ob.pan_left < 0x40 {
                let ratio = (cc - x) / (cc - cl);
                let temp = (0x40 - ob.pan_left) as f32 * ratio;
                pan = 64.0 - temp.abs();
            }
        }
        ob.pan = cvtt(pan);
        false
    }

    /// fn_800253D8 / fn_800256BC: the mirrored sweep pan.
    fn obj_sweep_pan(&mut self, i: usize) {
        let ob = &mut self.lb.objs[i];
        let p = calc_pan(ob.cur, ob.end, ob.pan_left, ob.pan_right);
        ob.pan = if ob.x3c == 1.0 { p } else { 0x7F - p };
    }

    /// The handler table `lbl_803BCA24`; true = free the object.
    fn obj_handler(&mut self, i: usize) -> bool {
        match self.lb.objs[i].mode {
            0 => self.obj_pan_from_owner(i),
            1 | 2 => {
                self.obj_sweep_pan(i);
                false
            }
            3 => {
                if !self.lb.objs[i].x44 {
                    self.lb.objs[i].x44 = true;
                    self.obj_pan_from_owner(i);
                }
                false
            }
            4 | 5 => {
                if !self.lb.objs[i].x44 {
                    self.lb.objs[i].x44 = true;
                    self.obj_sweep_pan(i);
                }
                false
            }
            6 => {
                let ob = &mut self.lb.objs[i];
                ob.pan = calc_pan(ob.cur, ob.end, ob.pan_left, ob.pan_right);
                false
            }
            7 => {
                let ob = &mut self.lb.objs[i];
                ob.pan = 0x7F - calc_pan(ob.cur, ob.end, ob.pan_left, ob.pan_right);
                false
            }
            _ => {
                let ob = &mut self.lb.objs[i];
                if ob.cur <= ob.end {
                    let frac = ob.cur as f32 / ob.end as f32;
                    let span = (ob.end_vol as f32 - ob.start_vol as f32).abs();
                    ob.vol = if ob.start_vol < ob.end_vol {
                        ob.start_vol.wrapping_add(cvtt(frac * span))
                    } else {
                        ob.end_vol.wrapping_sub(cvtt(frac * span))
                    };
                } else {
                    ob.vol = 0x7F;
                }
                false
            }
        }
    }

    /// `lbAudioAx_800263E8(f1, owner, mode, sfx, start_vol, end_vol, pan_left, pan_right,
    /// end_frame, track, voice)`: start a sound object. Returns its handle (the GObj), or None
    /// where the C returns NULL.
    #[allow(clippy::too_many_arguments)]
    pub fn lb_audio_ax_800263e8(
        &mut self, f1: f32, owner: usize, mode: i32, sfx: i32, start_vol: i32, end_vol: i32,
        pan_left: i32, pan_right: i32, end_frame: i32, track: i32, voice: i32,
    ) -> Option<usize> {
        if sfx >= SFX_NONE || !(0..=9).contains(&mode) {
            return None;
        }
        let x3c = if f1 == 0.0 {
            let r = self.lb.env.randi.map(|f| f(2)).unwrap_or(0);
            if r == 0 { 1.0 } else { -1.0 }
        } else {
            f1
        };
        self.lb.obj_seq += 1;
        let mut ob = SoundObj {
            id_handle: self.lb.obj_seq,
            owner,
            mode,
            sfx,
            start_vol,
            end_vol,
            vol: 0x7F,
            pan_left,
            pan_right,
            pan: 0x40,
            voice: -1,
            cur: 0,
            end: end_frame.max(0),
            x3c,
            track,
            x44: false,
        };
        if start_vol == end_vol {
            ob.vol = start_vol;
        }
        let handle = ob.id_handle;
        let (vol, pan) = (ob.vol, ob.pan);
        if mode == 9 {
            ob.voice = voice;
            self.lb.objs.push(ob);
        } else {
            self.lb.objs.push(ob);
            let v = if track == 0 {
                self.lb_audio_ax_800237a8(sfx, vol, pan)
            } else {
                self.lb_audio_ax_80023870(sfx, vol, pan, track)
            };
            self.lb.objs.last_mut().unwrap().voice = v;
        }
        Some(handle)
    }

    /// `lbAudioAx_800264E4(obj)`: the object's voice handle, -1 for none.
    pub fn lb_audio_ax_800264e4(&self, handle: Option<usize>) -> i32 {
        handle
            .and_then(|h| self.lb.objs.iter().find(|o| o.id_handle == h))
            .map(|o| o.voice)
            .unwrap_or(-1)
    }

    /// `lbAudioAx_80026510(owner)`: key off and free every object of `owner`; whether any.
    pub fn lb_audio_ax_80026510(&mut self, owner: usize) -> bool {
        if owner == 0 {
            return false;
        }
        let mut count = 0;
        let mut i = 0;
        while i < self.lb.objs.len() {
            if self.lb.objs[i].owner == owner {
                let v = self.lb.objs[i].voice;
                if v != -1 {
                    self.audio_sfx_key_off(v);
                }
                self.lb.objs.remove(i);
                count += 1;
            } else {
                i += 1;
            }
        }
        count != 0
    }

    /// `lbAudioAx_800265C4(owner, voice)`: key off and free the first object of `owner` playing
    /// `voice`.
    pub fn lb_audio_ax_800265c4(&mut self, owner: usize, voice: i32) -> bool {
        if let Some(i) = self.lb.objs.iter().position(|o| o.owner == owner && o.voice != -1 && o.voice == voice) {
            self.audio_sfx_key_off(voice);
            self.lb.objs.remove(i);
            return true;
        }
        false
    }

    /// `fn_800262A0` for every live object, in creation order: the proc phase.
    pub fn step_sound_objs(&mut self) {
        let mut i = 0;
        while i < self.lb.objs.len() {
            if self.obj_handler(i) {
                self.lb.objs.remove(i);
                continue;
            }
            let (voice, pan, vol, cur, end) = {
                let o = &self.lb.objs[i];
                (o.voice, o.pan, o.vol, o.cur, o.end)
            };
            if voice != -1 {
                self.lb_audio_ax_80024b1c(voice, pan);
                self.lb_audio_ax_80024b58(voice, vol);
            }
            if cur != -1 && end != -1 && (cur >= end || end == 0) {
                self.lb.objs.remove(i);
            } else {
                self.lb.objs[i].cur += 1;
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_pan_sweeps() {
        assert_eq!(calc_pan(0, 60, 0, 127), 0);
        assert_eq!(calc_pan(30, 60, 0, 127), 63);
        assert_eq!(calc_pan(60, 60, 0, 127), 127);
        assert_eq!(calc_pan(90, 60, 127, 0), 127);
        assert_eq!(calc_pan(5, 60, 40, 40), 64);
    }

    #[test]
    fn object_lifetime_and_owner_free() {
        let mut s = Sound::new();
        // no sound loaded: starts return -1, the object still lives its end_frame
        let h = s.lb_audio_ax_800263e8(1.0, 7, 6, 0x100, 127, 127, 0, 127, 3, 0, -1);
        assert!(h.is_some());
        s.step_sound_objs();
        s.step_sound_objs();
        assert_eq!(s.lb.objs.len(), 1);
        s.step_sound_objs();
        s.step_sound_objs();
        assert_eq!(s.lb.objs.len(), 0);
        // mode 0 with no env: freed on the first step
        s.lb_audio_ax_800263e8(1.0, 7, 0, 0x100, 127, 127, 0, 127, 60, 0, -1);
        s.step_sound_objs();
        assert!(s.lb.objs.is_empty());
        s.lb_audio_ax_800263e8(1.0, 9, 6, 0x100, 127, 127, 0, 127, 60, 0, -1);
        assert!(s.lb_audio_ax_80026510(9));
        assert!(!s.lb_audio_ax_80026510(9));
    }

    #[test]
    fn pan_from_owner_position() {
        fn class(_: usize) -> i32 { CLASS_FIGHTER }
        fn pos(_: usize) -> [f32; 3] { [50.0, 0.0, 0.0] }
        fn cam() -> (f32, f32, f32) { (-100.0, 0.0, 100.0) }
        let mut s = Sound::new();
        s.lb.env = PosEnv { class_of: Some(class), fighter_pos: Some(pos), item_pos: None, camera: Some(cam), randi: None };
        s.lb_audio_ax_800263e8(1.0, 3, 0, 0x100, 127, 127, 0, 127, 60, 0, -1);
        s.step_sound_objs();
        // pan_right 127: 64 + 63 * 0.5 = 95.5 -> 95
        assert_eq!(s.lb.objs[0].pan, 95);
    }
}
