//! The volume, ducking, pause and sound-effect entry points of `lbaudio_ax.c` that sit between the
//! game and the synth: the music / sfx / voice volume settings and `fn_80024654` (the per-frame
//! recompute that scales every SFX channel and the stream), the pause/duck switches, sound mode,
//! the per-category sfx starters (`lbAudioAx_80024184`, `80024304`, `800243F4`, ...), the
//! on-demand bank sfx (`80023B24`), and the scene init (`fn_800244F4`, `80028690`).
//!
//! Every function keeps the C's names in its doc line; the file statics (`lbl_804D387C`...) live in
//! [`Vol`], which `LbState` owns. Float expressions are written in the C's order and precision
//! (single precision, no fused multiply-add: `tools/rust-port/fma_sites.py` finds none).

use crate::lbaudio::{stage_audio, BANK_INFO, SFX_NONE, SSM_FILES};
use crate::Sound;

/// The volume statics.
#[derive(Clone, Debug)]
pub struct Vol {
    /// lbl_804D387C: music volume setting 0..0x7F.
    pub music: i32,
    /// lbl_804D3884: the voice/"sfx 2" setting.
    pub voice: i32,
    /// lbl_804D388C: the sfx volume setting.
    pub sfx: i32,
    /// synth_volume: the stream volume last computed.
    pub synth_volume: f32,
    /// lbl_804D38C0 / 38C4 / 38C8: music, voice and sfx on-off scale factors.
    pub f_c0: f32,
    pub f_c4: f32,
    pub f_c8: f32,
    /// lbl_804D38CC: the fade-down counter (0x7F = full).
    pub fade: i32,
    /// lbl_804D38E4 / 38E8: the pause duck of the stage sfx / of the voices (1.0 or 0.2).
    pub duck_e4: f32,
    pub duck_e8: f32,
    /// The u8 results fn_80024654 sends: 3888, 3898, 38A0, 38A8.
    pub o_3888: i32,
    pub o_3898: i32,
    pub o_38a0: i32,
    pub o_38a8: i32,
    /// "Last sent" caches: lbl_804D63F0 (f32), 63F4, 63FC, 6400, 6404, and 38DC.
    pub c_stream: f32,
    pub c_63f4: i32,
    pub c_63fc: i32,
    pub c_6400: i32,
    pub c_6404: i32,
    pub c_echo: i32,
    /// `paused` (AXDriverPause).
    pub paused: bool,
    /// `sound_mode`: -1 never set (the C's initial 1 is what the boot overwrites).
    pub sound_mode: i32,
    /// lbl_804D3878: the language the banks were loaded for (-1 none).
    pub lang: i32,
    /// `debug_enabled`.
    pub debug: bool,
}

impl Default for Vol {
    fn default() -> Self {
        Vol {
            music: 0x7F,
            voice: 0x7F,
            sfx: 0x7F,
            synth_volume: 1.0,
            f_c0: 1.0,
            f_c4: 1.0,
            f_c8: 1.0,
            fade: 0x7F,
            duck_e4: 1.0,
            duck_e8: 1.0,
            o_3888: 0x7F,
            o_3898: 0x7F,
            o_38a0: 0x7F,
            o_38a8: 0x7F,
            c_stream: 0.0,
            c_63f4: 0,
            c_63fc: 0,
            c_6400: 0,
            c_6404: 0,
            c_echo: -1,
            paused: false,
            sound_mode: 1,
            lang: -1,
            debug: false,
        }
    }
}

/// The track a sfx id maps to in `lbAudioAx_80024184`.
fn track_80024184(id: i32, track: i32) -> i32 {
    match id {
        0x56 => 9,
        0x57 => 8,
        0x58 => 7,
        0x59 => 0xC,
        0x5A => 0xB,
        0x5B => 0xA,
        0x6F => 0xD,
        0x70 => 0xE,
        0x71 => 0xF,
        0x65 => 0x10,
        0x66 => 0x11,
        0x67 => 0x12,
        0x54 => 0x17,
        0x12E => 0x18,
        0x20D => {
            if track != -1 {
                track
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// The fall-through ladder of `lbAudioAx_800243F4`: the ids in the order of the C's cases, last
/// case first. A case adds one to the track for itself and for each case below it.
const LADDER_800243F4: [i32; 26] = [
    0x7C830, 0x7C831, 0x7C835, 0x7C83A, 0x7C83F, 0x7C840, 0x7C842, 0x7C844, 0x7C845, 0x7C846, 0x7C848,
    0x7C84A, 0x7C84B, 0x7C84D, 0x7C83B, 0x7C83D, 0x7C84E, 0x7C84F, 0x7C850, 0x7C851, 0x7C834, 0x7C843,
    0x7C832, 0x7C83C, 0x7C84C, 0x7C836,
];
/// ...and the two cases after the ladder's `0x7C836` that end it (`0x7C849`, `0x7C838`).
const LADDER_TAIL: [i32; 2] = [0x7C849, 0x7C838];

fn track_800243f4(id: i32) -> i32 {
    // Case labels in source order: 0x7C830 sets track = 0xCB then falls through; every following
    // label (including the tail ones) adds 1, and the last one breaks.
    let mut all = LADDER_800243F4.to_vec();
    all.extend_from_slice(&LADDER_TAIL);
    let Some(start) = all.iter().position(|&x| x == id) else {
        return 0xCA;
    };
    let mut track = 0xCA;
    if start == 0 {
        track = 0xCB;
    }
    // From the matched case down to the end, one `track += 1` per remaining case label after
    // the first one's own statement: the matched label itself runs its statement.
    for _ in start.max(1)..all.len() {
        track += 1;
    }
    track
}

impl Sound {
    fn upd_all(&mut self, vol: i32, ch: usize) {
        self.sfx_update_all_volume(vol, 4, ch);
    }

    /// `fn_80024654(first)`: recompute the music, sfx and voice volumes from the settings, the
    /// on/off factors and the ducks, and send what changed (everything when `first`).
    pub fn lb_fn_80024654(&mut self, first: bool) {
        let v = &mut self.lb.vol;
        v.synth_volume = (v.music as f32 / 127.0f32) * v.f_c0;
        let sv = v.synth_volume;
        let mut send_stream = false;
        if first || v.c_stream != sv {
            v.c_stream = sv;
            send_stream = true;
        }
        let temp = v.fade as f32 / 127.0f32;
        let f_b4 = v.f_c8 * temp;
        let base = 0.88f32 * ((v.sfx * 2) as f32);
        let o_3898 = melee_core::ppc::fctiwz_f32(f_b4 * base) as u8 as i32;
        let f_b8 = temp * (v.f_c8 * v.duck_e4);
        let o_38a0 = melee_core::ppc::fctiwz_f32(f_b8 * base) as u8 as i32;
        let f_bc = v.f_c8 * v.duck_e4;
        let o_38a8 = melee_core::ppc::fctiwz_f32(f_bc * base) as u8 as i32;
        let f_ac = temp * (self.lb.duck * (v.f_c4 * v.duck_e8));
        let o_3888 = melee_core::ppc::fctiwz_f32(f_ac * ((v.voice * 2) as f32)) as u8 as i32;
        v.o_3898 = o_3898;
        v.o_38a0 = o_38a0;
        v.o_38a8 = o_38a8;
        v.o_3888 = o_3888;
        let (c_fc, c_00, c_04, c_f4) = (v.c_63fc, v.c_6400, v.c_6404, v.c_63f4);
        let echo = self.lb.stage_echo;
        let c_echo = v.c_echo;
        if send_stream {
            self.stream_set_volume(sv);
        }
        if first || c_fc != o_3898 {
            self.upd_all(o_3898, 2);
            self.upd_all(o_3898, 3);
            self.lb.vol.c_63fc = o_3898;
        }
        if first || c_00 != o_38a0 {
            for ch in 4..=7 {
                self.upd_all(o_38a0, ch);
            }
            self.lb.vol.c_6400 = o_38a0;
        }
        if first || c_04 != o_38a8 {
            self.upd_all(o_38a8, 8);
            self.lb.vol.c_6404 = o_38a8;
        }
        if first || c_f4 != o_3888 {
            self.upd_all(o_3888, 1);
            self.lb.vol.c_63f4 = o_3888;
        }
        if first || c_echo != echo {
            self.audio_sfx_set_mix_group(5, 1, 0x20);
            self.audio_sfx_set_mix_group(6, 1, 0x20);
            self.audio_sfx_set_mix_group(8, 1, echo as u8 as i8);
            self.audio_sfx_set_mix_group(7, 1, echo as u8 as i8);
            self.lb.vol.c_echo = echo;
        }
    }

    /// `lbAudioAx_800245D4(v)`: music volume setting.
    pub fn lb_audio_ax_800245d4(&mut self, v: i32) {
        self.lb.vol.music = v.clamp(0, 0x7F);
    }

    /// `lbAudioAx_800245F4(v)`: sfx volume setting.
    pub fn lb_audio_ax_800245f4(&mut self, v: i32) -> i32 {
        self.lb.vol.sfx = v.clamp(0, 0x7F);
        self.lb.vol.sfx
    }

    /// `lbAudioAx_80024614(v)`: voice volume setting.
    pub fn lb_audio_ax_80024614(&mut self, v: i32) {
        self.lb.vol.voice = v.clamp(0, 0x7F);
    }

    /// `lbAudioAx_80024634(v)`: the echo (aux send) byte.
    pub fn lb_audio_ax_80024634(&mut self, v: i32) {
        self.lb.stage_echo = v.clamp(0, 0xFF);
    }

    /// `lbAudioAx_80024D78(which)`: the echo byte of the current stage's table row (`grkind` is
    /// `Stage_8022519C(Stage_80225194())`).
    pub fn lb_audio_ax_80024d78(&mut self, which: usize, grkind: i32) {
        self.lb.stage_echo = stage_audio(grkind)[which] as i32;
    }

    /// `lbAudioAx_80025064(music_on, sfx_on)`: lbl_804D38C4 / 38C8 (the voice and the sfx/music
    /// on-off scale).
    pub fn lb_audio_ax_80025064(&mut self, a: bool, b: bool) {
        self.lb.vol.f_c4 = if a { 1.0 } else { 0.0 };
        self.lb.vol.f_c8 = if b { 1.0 } else { 0.0 };
    }

    /// `lbAudioAx_80025098(debug)`.
    pub fn lb_audio_ax_80025098(&mut self, d: bool) {
        self.lb.vol.debug = d;
    }

    /// `lbAudioAx_80028B6C`: one step of the fade-down counter.
    pub fn lb_audio_ax_80028b6c(&mut self) {
        self.lb.vol.fade = (self.lb.vol.fade - 1).max(0);
    }

    /// `lbAudioAx_80028B90`: the fade counter back to full.
    pub fn lb_audio_ax_80028b90(&mut self) {
        self.lb.vol.fade = 0x7F;
    }

    /// `lbAudioAx_80024B1C(voice, pan)`.
    pub fn lb_audio_ax_80024b1c(&mut self, voice: i32, pan: i32) {
        self.audio_sfx_set_pan(voice, (pan.clamp(0, 0x7F) * 2) as u8);
    }

    /// `lbAudioAx_80023694`: key off every sfx.
    pub fn lb_audio_ax_80023694(&mut self) -> i32 {
        self.audio_sfx_key_off_all();
        -1
    }

    /// `lbAudioAx_800236B8(voice)`.
    pub fn lb_audio_ax_800236b8(&mut self, voice: i32) -> i32 {
        self.audio_sfx_key_off(voice);
        -1
    }

    /// `lbAudioAx_80023730`: whether the BGM plays.
    pub fn lb_audio_ax_80023730(&self) -> bool {
        self.stream_check()
    }

    /// `lbAudioAx_80024BD0`: read the synth's sound mode; true = not mode 1.
    pub fn lb_audio_ax_80024bd0(&mut self) -> bool {
        let m = self.sound_mode;
        self.lb.vol.sound_mode = m as i32;
        m != 1
    }

    /// `lbAudioAx_80024C08(mode)`: set the sound mode (0 stereo-ish `1`, 1 mono `0`); true if set.
    pub fn lb_audio_ax_80024c08(&mut self, mode: u32) -> bool {
        const TO_SYNTH: [i32; 2] = [1, 0];
        if mode > 1 {
            return false;
        }
        let want = TO_SYNTH[mode as usize];
        if self.lb.vol.sound_mode != want {
            self.lb.vol.sound_mode = want;
            self.set_sound_mode(want as u32);
        }
        true
    }

    /// `lbAudioAx_80024C84`: reset the pause/duck state (a scene change).
    pub fn lb_audio_ax_80024c84(&mut self) {
        let lb = &mut self.lb;
        lb.muted = 0;
        lb.vol.paused = false;
        lb.vol.duck_e4 = 1.0;
        lb.vol.duck_e8 = 1.0;
        lb.duck = 1.0;
        lb.hb1_vol = 0;
        lb.hb2_vol = 0;
        lb.hb1 = 0;
        lb.hb2 = 0;
        lb.hb1_voice = -1;
        lb.hb2_voice = -1;
        lb.hb1_timer = 0;
        lb.hb2_timer = 0;
        self.audio_sfx_key_off_track(5);
        self.audio_sfx_key_off_track(6);
        let lb = &mut self.lb;
        lb.flush_deferred = 0;
        lb.vol.f_c0 = 1.0;
        self.pstream_resume_ch(5);
        self.pstream_resume_ch(6);
        self.pstream_resume_ch(8);
        self.pstream_resume_ch(7);
        self.lb.stage_echo = 1;
        self.lb.vol.fade = 0x7F;
    }

    /// `lbAudioAx_80024D50`: reset, then have the deferred sfx play out on the next frame.
    pub fn lb_audio_ax_80024d50(&mut self) {
        self.lb_audio_ax_80024c84();
        self.lb.flush_deferred = 1;
    }

    /// `lbAudioAx_80024E50(pause)`: pause or resume the BGM stream (AXDriverPause / Resume).
    pub fn lb_audio_ax_80024e50(&mut self, pause: bool) {
        self.lb.vol.paused = pause;
        if pause {
            self.stream_pause();
        } else {
            self.stream_resume();
        }
    }

    /// `lbAudioAx_80024E84(duck)`: the pause menu's duck (stage sfx to 0.2, the stage streams
    /// paused) on or off.
    pub fn lb_audio_ax_80024e84(&mut self, on: bool) {
        self.lb.muted = on as i32;
        let d = if on { 0.2f32 } else { 1.0f32 };
        self.lb.vol.duck_e4 = d;
        self.lb.vol.duck_e8 = d;
        for ch in [5, 6, 8, 7] {
            if on {
                self.pstream_pause_ch(ch);
            } else {
                self.pstream_resume_ch(ch);
            }
        }
    }

    /// `lbAudioAx_80024F08`: silence the stream and pause channels 2..9.
    pub fn lb_audio_ax_80024f08(&mut self) {
        self.stream_set_volume(0.0);
        for ch in 2..=9 {
            self.pstream_pause_ch(ch);
        }
    }

    /// `lbAudioAx_80024F6C`: undo [`Self::lb_audio_ax_80024f08`].
    pub fn lb_audio_ax_80024f6c(&mut self) {
        let sv = self.lb.vol.synth_volume;
        self.stream_set_volume(sv);
        for ch in [2, 3, 4, 9] {
            self.pstream_resume_ch(ch);
        }
        if self.lb.muted == 0 {
            for ch in [5, 6, 8, 7] {
                self.pstream_resume_ch(ch);
            }
        }
    }

    /// `fn_800244F4`: the scene-init reset of every volume static (`gm_801603B0` is the caller's).
    pub fn lb_fn_800244f4(&mut self) {
        let old = self.lb.vol.clone();
        self.lb.vol = Vol {
            lang: old.lang,
            sound_mode: old.sound_mode,
            debug: old.debug,
            c_stream: 0.0,
            ..Vol::default()
        };
        let lb = &mut self.lb;
        lb.stage_echo = 1;
        lb.muted = 0;
        lb.duck = 1.0;
        lb.flush_deferred = 0;
        lb.hb1 = 0;
        lb.hb2 = 0;
        lb.hb1_vol = 0;
        lb.hb2_vol = 0;
        lb.hb1_timer = 0;
        lb.hb2_timer = 0;
        lb.hb1_voice = -1;
        lb.hb2_voice = -1;
    }

    // ---- sfx starters ----------------------------------------------------------------------

    /// `lbAudioAx_80024030(n)`: one of the 11 fixed menu/announcer sounds.
    pub fn lb_audio_ax_80024030(&mut self, n: i32) {
        const T: [[i32; 3]; 11] = [
            [0xAC, 0x13, 3], [0xAD, 0x14, 3], [0xAE, 0, 3], [0xAF, 0, 3], [0xB0, 0x15, 3], [0xB1, 0, 2],
            [0xC1, 0, 2], [0xBC, 0, 2], [0xDB, 0, 3], [0xDC, 0, 3], [0xDD, 0, 3],
        ];
        if (0..0xB).contains(&n) {
            let r = T[n as usize];
            self.lb_start(r[0], 0x7F, 0x40, r[1], r[2]);
        }
    }

    /// `lbAudioAx_800240B4(id)`: channel 5.
    pub fn lb_audio_ax_800240b4(&mut self, id: i32) -> i32 {
        self.lb_start(id, 0x7F, 0x40, 0, 5)
    }

    /// `lbAudioAx_8002411C(id)`: channel 6.
    pub fn lb_audio_ax_8002411c(&mut self, id: i32) -> i32 {
        self.lb_start(id, 0x7F, 0x40, 0, 6)
    }

    /// `lbAudioAx_80024184(id, vol, pan, track)`: sfx with the per-id track.
    pub fn lb_audio_ax_80024184(&mut self, id: i32, vol: i32, pan: i32, track: i32) -> i32 {
        let t = track_80024184(id, track);
        self.lb_start(id, vol, pan, t, 7)
    }

    /// `lbAudioAx_80024304(id)`: the three 0x8A..0x8C hit sounds share one track.
    pub fn lb_audio_ax_80024304(&mut self, mut id: i32) -> i32 {
        let mut track = 0;
        if matches!(id, 0x8A | 0x8B | 0x8C) {
            track = 0x16;
            id = 0x8B;
        }
        self.lb_start(id, 0x7F, 0x40, track, 7)
    }

    /// `lbAudioAx_8002438C(id)`: channel 8.
    pub fn lb_audio_ax_8002438c(&mut self, id: i32) -> i32 {
        self.lb_start(id, 0x7F, 0x40, 0, 8)
    }

    /// `lbAudioAx_800243F4(id)`: the announcer ids, one track each.
    pub fn lb_audio_ax_800243f4(&mut self, id: i32) -> i32 {
        let t = track_800243f4(id);
        self.lb_start(id, 0x7F, 0x40, t, 7)
    }

    /// `lbAudioAx_80023B24(id)`: play `id`, first loading its bank into SFX bank 2 if it is a
    /// group-4/5-style bank that is not resident (the async load run to completion).
    pub fn lb_audio_ax_80023b24(&mut self, id: i32) -> i32 {
        let slot = crate::lbaudio::lb_80023130(id) as usize;
        if slot < BANK_INFO.len() && BANK_INFO[slot][1] != 5 {
            self.audio_sfx_key_off_all();
            if self.lb.state[slot] != 2 {
                self.sfx_unload_bank(2);
                for k in 0..crate::lbaudio::LBAX_N {
                    if BANK_INFO[k][2] != 5 {
                        self.lb.requested[k] = -1;
                        self.lb.wanted[k] = -1;
                        self.lb.state[k] = -1;
                        self.lb.entry[k] = -1;
                    }
                }
                let path = format!("{}{}", self.lb.ssm_dir, SSM_FILES[slot]);
                if let Ok(e) = self.load_ssm(&path, 2) {
                    self.lb.entry[slot] = e;
                }
                self.lb.state[slot] = 2;
            }
        }
        self.lb_start(id, 0x7F, 0x40, 0, 7)
    }

    /// `lbAudioAx_80027DBC`: leaving a scene: key off everything, stop the BGM, reset the duck
    /// state.
    pub fn lb_audio_ax_80027dbc(&mut self) {
        self.audio_sfx_key_off_all();
        self.lb_audio_ax_800236dc();
        self.lb_audio_ax_80024c84();
    }

    fn load_slot_into(&mut self, slot: usize, bank: usize) {
        if self.lb.state[slot] < 1 {
            let path = format!("{}{}", self.lb.ssm_dir, SSM_FILES[slot]);
            if let Ok(e) = self.load_ssm(&path, bank) {
                self.lb.entry[slot] = e;
            }
            self.lb.state[slot] = 2;
        }
    }

    /// The language-change block shared by `lbAudioAx_80028690` and `80027AB0`: reload smash2.sem
    /// and the resident banks (bank 1: 0x33, 1, 0x36; with `with_bank2`, bank 2: slot 2).
    fn reload_for_lang(&mut self, lang: i32, with_bank2: bool) {
        self.lb.vol.lang = lang;
        let _ = self.load_sem(&format!("{}smash2.sem", self.lb.ssm_dir));
        for i in 1..crate::lbaudio::LBAX_N {
            self.lb.entry[i] = -1;
            self.lb.requested[i] = -1;
            self.lb.wanted[i] = -1;
            self.lb.state[i] = -1;
        }
        self.sfx_unload_bank(1);
        self.sfx_unload_bank(2);
        self.load_slot_into(0x33, 1);
        self.load_slot_into(1, 1);
        self.load_slot_into(0x36, 1);
        if with_bank2 {
            self.load_slot_into(2, 2);
        }
    }

    /// `lbAudioAx_80028690`: the scene audio init (every scene start): volume reset, then, the
    /// first time or after a language change, bank 0 and the resident banks 1 and 2.
    pub fn lb_audio_ax_80028690(&mut self) {
        self.lb_audio_ax_80024bd0();
        self.lb_fn_800244f4();
        self.lb_fn_80024654(true);
        // setup_audio_lang with a US save: /audio/us/.
        self.lb.ssm_dir = "/audio/us/";
        if self.lb.vol.lang == -1 {
            self.audio_sfx_key_off_all();
            self.sfx_unload_bank(0);
            self.load_slot_into(0, 0);
        }
        if self.lb.vol.lang != 1 {
            self.reload_for_lang(1, true);
        }
        for i in 0..=16 {
            self.lb.deferred[i] = SFX_NONE;
            self.lb.deferred_timer[i] = 0;
        }
        let lb = &mut self.lb;
        lb.vol.duck_e4 = 1.0;
        lb.vol.duck_e8 = 1.0;
        lb.duck = 1.0;
        lb.muted = 0;
        lb.hb1 = 0;
        lb.hb2 = 0;
        lb.hb1_vol = 0;
        lb.hb2_vol = 0;
        lb.hb1_voice = -1;
        lb.hb2_voice = -1;
        lb.hb1_timer = 0;
        lb.hb2_timer = 0;
        lb.vol.f_c4 = 1.0;
        lb.vol.f_c8 = 1.0;
    }

    /// `lbAudioAx_80027AB0(id)`: a language switch at runtime (the menu's US/JP setting).
    pub fn lb_audio_ax_80027ab0(&mut self, id: i32, lang_us: bool) {
        self.lb.ssm_dir = if lang_us { "/audio/us/" } else { "/audio/" };
        let lang = lang_us as i32;
        self.audio_sfx_key_off_all();
        if self.lb.vol.lang != lang {
            self.lb.vol.lang = lang;
            let _ = self.load_sem(&format!("{}smash2.sem", self.lb.ssm_dir));
            if id != 0x83D61 {
                self.lb_audio_ax_800237a8(id, 0x7F, 0x40);
            }
            self.reload_for_lang(lang, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announcer_track_ladder() {
        assert_eq!(track_800243f4(0x1234), 0xCA);
        assert_eq!(track_800243f4(0x7C830), 0xCB + 27);
        assert_eq!(track_800243f4(0x7C831), 0xCA + 27);
        assert_eq!(track_800243f4(0x7C838), 0xCA + 1);
        assert_eq!(track_800243f4(0x7C849), 0xCA + 2);
    }

    #[test]
    fn track_table_80024184() {
        assert_eq!(track_80024184(0x56, 5), 9);
        assert_eq!(track_80024184(0x20D, -1), 0);
        assert_eq!(track_80024184(0x20D, 4), 4);
        assert_eq!(track_80024184(0x1, 4), 0);
    }
}
