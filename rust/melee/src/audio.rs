//! Audio of the live game (`melee play`): the sound stack of `melee_audio` booted from the disc, the
//! 5 ms AX frames fed to the output device once per tick, and the sounds each scene of the C game
//! makes, taken from the C sources (`gm_Scene_Title_*`, `mnMain`, `mnCharSel`, `mnStageSel`, `gmVs`,
//! `ifStatus`, `gmResult`) with their `lbAudioAx_*` arguments.
//!
//! What is driven here is what the stand-in scenes of `play.rs` stand for: the menus' cursor and
//! confirm sounds, the menu music, the character announcer and voice on a pick, the VS scene's bank
//! loads and stage music, the Ready/Go/TIME!/GAME! announcer, the five-second timer ticks, the
//! pause duck, and the winner's theme. The fighters' own sounds come through the `lbAudioAx` hooks
//! (`install_audio`), as in `golden --audio`.

use melee_audio::sim;

/// `gmMainLib_8015ECB0` after `gmMainLib_8015ECBC` with nothing unlocked: menu01.hps.
const MENU_BGM: i32 = 0x34;

pub struct Audio {
    out: sim::Output,
    pub dev_open: bool,
}

/// CharacterKind of the roster's fighter files.
pub fn ckind_of(name: &str) -> i32 {
    (match name {
        "falcon" => melee_types::CKind_Captain,
        "fox" => melee_types::CKind_Fox,
        "falco" => melee_types::CKind_Falco,
        "marth" => melee_types::CKind_Emblem,
        "sheik" => melee_types::CKind_Seak,
        "peach" => melee_types::CKind_Peach,
        "samus" => melee_types::CKind_Samus,
        _ => melee_types::CKind_Captain,
    }) as i32
}

/// `gm_80168C5C`: the announcer's call for a character name (retail table).
fn name_call(ckind: i32) -> Option<i32> {
    Some(match ckind {
        0 => 0x7C830, 1 => 0x7C831, 2 => 0x7C835, 3 => 0x7C83A, 4 => 0x7C83F, 5 => 0x7C840,
        6 => 0x7C842, 7 => 0x7C844, 8 => 0x7C845, 9 => 0x7C846, 10 => 0x7C848, 11 => 0x7C84A,
        12 => 0x7C84B, 13 => 0x7C84D, 14 => 0x7C83B, 15 => 0x7C83D, 16 => 0x7C84E, 17 => 0x7C84F,
        19 => 0x7C850, 18 => 0x7C851, 20 => 0x7C834, 21 => 0x7C843, 22 => 0x7C832, 23 => 0x7C83C,
        24 => 0x7C84C, 25 => 0x7C836, 26 => 0x7C849, 29 => 0x7C838,
        _ => return None,
    })
}

/// The CSS icon's voice (`CSSIcon.sfx`, the 7th field of the icon rows) for the roster.
fn css_voice(ckind: i32) -> Option<i32> {
    Some(match ckind {
        0 => 0xC2,  // Captain Falcon
        2 => 0xC7,  // Fox
        20 => 0xC6, // Falco
        12 => 0xD1, // Peach
        16 => 0xD5, // Samus
        23 => 0xDA, // Marth
        19 => 0xD6, // Sheik (the Zelda icon)
        _ => return None,
    })
}

/// `ckind_victory_themes` (gm_1601.c): the BGM id of a character's victory theme.
fn victory_theme(ckind: i32) -> i32 {
    match ckind {
        2 | 20 => 0x10,
        12 => 0x16,
        16 => 0x19,
        23 => 0xE,
        19 => 0x15,
        _ => 0x11,
    }
}

/// `lbl_803D5630`: the last-five-seconds calls, indexed by the seconds left.
const TIMER_TICKS: [i32; 6] = [-1, 0x7C858, 0x7C862, 0x7C85F, 0x7C854, 0x7C853];

/// `lbAudioAx_80026F2C(flags)`, `8002702C(2, mask)`, `80027168()`: the bank request a scene makes.
fn request_banks(flags: u32, mask: u64) {
    sim::with(|s| {
        s.lb_audio_ax_80026f2c(flags);
        s.lb_audio_ax_8002702c(2, mask);
        s.lb_audio_ax_80027168();
    });
}

impl Audio {
    /// Boot the sound from the disc and open the output. `Err` leaves the game silent.
    pub fn boot(iso: &str) -> Result<Audio, String> {
        crate::install_audio(iso)?;
        let out = sim::Output::new(melee_audio::output::DriverConfig::from_env(false));
        Ok(Audio { out, dev_open: false })
    }

    /// Once per video frame: the game's own per-frame audio (`lbAudioAx_80027DF8`), then the 5 ms
    /// frames the device is owed.
    pub fn tick(&mut self) {
        sim::game_frame();
        self.out.tick();
        self.dev_open = self.out.driver.device_open();
    }

    fn menu(&self, n: i32) {
        sim::with(|s| s.lb_audio_ax_80024030(n));
    }
    pub fn back(&self) {
        self.menu(0)
    }
    pub fn forward(&self) {
        self.menu(1)
    }
    pub fn moved(&self) {
        self.menu(2)
    }
    /// The refusal sound (mnCharSel / mnStageSel play 3 for a pick that is not allowed).
    pub fn denied(&self) {
        self.menu(3)
    }

    /// Every scene start (`lbAudioAx_80028690` from the scene loop).
    pub fn scene_start(&self) {
        sim::scene_init();
    }

    pub fn stop_music(&self) {
        sim::with(|s| s.lb_audio_ax_800236dc());
    }

    pub fn menu_music(&self) {
        sim::play_bgm_id(MENU_BGM);
    }

    /// `gm_Scene_Title_OnEnter`: stop the music, nr_title.
    pub fn title(&self) {
        self.stop_music();
        request_banks((1 << 1) | (1 << 4), 4);
    }

    /// `gm_Scene_Title_OnFrame` on START: the menu banks, then the confirm sound.
    pub fn title_start(&self) {
        sim::with(|s| {
            s.lb_audio_ax_80026f2c(0x1C);
            s.lb_audio_ax_8002702c(0xC, 0);
            s.lb_audio_ax_80027168();
        });
        self.forward();
    }

    /// `mnCharSel_Scene_OnEnter`: nr_select, then the menu music.
    pub fn css(&self) {
        request_banks(0x12, 8);
        self.menu_music();
    }

    /// A fighter chosen on the character select: its icon voice on the per-icon track, then the
    /// announcer's call (`gm_80168C5C`).
    pub fn css_pick(&self, ckind: i32, icon: usize) {
        sim::with(|s| {
            if let Some(v) = css_voice(ckind) {
                s.lb_audio_ax_80023870(v, 0x7F, 0x40, icon as i32 + 0x8A);
            }
            if let Some(c) = name_call(ckind) {
                s.lb_audio_ax_800243f4(c);
            }
        });
    }

    /// The VS scene's audio start (`gm_LoadAnnouncer`, `lbAudioAx_8002785C`, the stage's BGM).
    pub fn match_start(&self, ckinds: &[i32], stkind: i32, grkind: i32, bgm: i32) {
        sim::scene_init();
        sim::load_announcer();
        sim::load_match_banks(ckinds, stkind, grkind);
        sim::play_bgm_id(bgm);
    }

    /// Announcer and timer sounds for the match frame `f` (the sim's `frame` after the step).
    pub fn match_frame(&self, f: i32, go_frame: i32, secs: i32) {
        match_frame_calls(f, go_frame, secs);
    }

    /// `ifStatus_802F6EA4` element 0 (TIME!) or 5 (GAME!): the end-of-match call.
    pub fn match_end_call(&self, timeout: bool) {
        sim::lbaudioax_800237a8(if timeout { 0x7C860 } else { 0x7C855 }, 0x7F, 0x40);
    }

    /// `gmVs` pause: duck the music and sfx, the pause sound.
    pub fn pause(&self, on: bool) {
        sim::set_pause_duck(on);
        if on {
            self.menu(5);
        }
    }

    /// Leaving the VS scene (`gmVs` end: `80023694`, `80024C84`).
    pub fn match_leave(&self) {
        sim::set_pause_duck(false);
        sim::with(|s| {
            s.lb_audio_ax_80023694();
            s.lb_audio_ax_80024c84();
        });
    }

    /// The results screen: the winner's theme (none on a draw) and their name called.
    pub fn results(&self, winner: Option<i32>) {
        self.stop_music();
        request_banks(0x12, 0x18);
        if let Some(ck) = winner {
            sim::play_bgm_id(victory_theme(ck));
            sim::with(|s| {
                if let Some(c) = name_call(ck) {
                    s.lb_audio_ax_800243f4(c);
                }
            });
        }
    }
}

/// The announcer and timer calls for the match frame `f` (the sim's `frame` after the step).
/// `go_frame` is the frame the Ready HUD ends on, `secs` the time limit.
pub fn match_frame_calls(f: i32, go_frame: i32, secs: i32) {
    let play = |id: i32| {
        sim::lbaudioax_800237a8(id, 0x7F, 0x40);
    };
    // ifStatus element 3 (Ready, 0x7C85D) plays at its first animation frame, element 4 (Go,
    // 0x7C857) ten frames into its animation, which starts when Ready ends.
    if f == 0 {
        play(0x7C85D);
    }
    if go_frame != i32::MAX && f == go_frame + 10 {
        play(0x7C857);
    }
    // gmVs per-frame: every 60 frames the timer ticks; under six seconds left it calls out.
    if f > 0 && f % 60 == 0 {
        let left = secs - (f / 60 - 1);
        if (1..6).contains(&left) {
            play(TIMER_TICKS[left as usize]);
        }
    }
}
