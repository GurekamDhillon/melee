#![allow(non_snake_case)]
use melee_runtime::tu::dbeffect::pad;
use melee_runtime::tu::dbsound::{db_peak_hold, DbSound, DbSoundHost};
use melee_types::{DevText, GXColor, HSD_GObj};

/// db_peak_hold against the C body cut from fn_UpdateSoundInfo (oracle/tu_dbinit_ref.sh).
#[test]
fn peak_hold_matches_c() {
    let (mut p, mut h, mut n) = (0, 0, 0);
    for l in include_str!("data/tu_dbinit_ref.txt").lines().filter(|l| l.starts_with("peak ")) {
        let v: Vec<i32> = l[5..].split(' ').map(|x| x.parse().unwrap()).collect();
        db_peak_hold(&mut p, &mut h, v[0]);
        assert_eq!((p, h), (v[1], v[2]), "row {n}");
        n += 1;
    }
    assert_eq!(n, 600);
}

struct H { down: u32, pressed: u32, text: DevText, log: Vec<String>, mutes: Vec<(i32, i32)>, voices: i32 }
impl DbSoundHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn DevText_GetGObj(&mut self) -> *mut HSD_GObj { core::ptr::null_mut() }
    fn DevText_Create(&mut self, id: i32, x: i32, y: i32, w: i32, h: i32, _: *mut i8) -> *mut DevText {
        assert_eq!((id, x, y, w, h), (9, 420, 60, 18, 3));
        &mut self.text
    }
    fn DevText_Show(&mut self, _: *mut HSD_GObj, _: *mut DevText) {}
    fn DevText_HideCursor(&mut self, _: *mut DevText) {}
    fn DevText_SetBGColor(&mut self, _: *mut DevText, _: GXColor) {}
    fn DevText_SetTextColor(&mut self, _: *mut DevText, _: GXColor) {}
    fn DevText_SetScale(&mut self, _: *mut DevText, x: f32, y: f32) { assert_eq!((x, y), (12.0, 16.0)) }
    fn DevText_ShowBackground(&mut self, _: *mut DevText) { self.log.push("showbg".into()) }
    fn DevText_ShowText(&mut self, _: *mut DevText) {}
    fn DevText_HideBackground(&mut self, _: *mut DevText) { self.log.push("hidebg".into()) }
    fn DevText_HideText(&mut self, _: *mut DevText) {}
    fn DevText_Erase(&mut self, _: *mut DevText) {}
    fn DevText_SetCursorXY(&mut self, _: *mut DevText, _: i32, _: i32) {}
    fn DevText_Printf(&mut self, _: *mut DevText, s: &str) { self.log.push(s.into()) }
    fn lbAudioAx_80025098(&mut self, _: bool) {}
    fn lbAudioAx_80028B2C(&mut self) -> i32 { self.voices }
    fn lbAudioAx_80028B4C(&mut self) -> i32 { self.voices + 1 }
    fn lbAudioAx_80025064(&mut self, a: i32, b: i32) { self.mutes.push((a, b)) }
}

#[test]
fn x_dleft_cycles_toggles_and_overlay() {
    let mut s = Box::new(DbSound::default());
    let mut h = H { down: 0, pressed: 0, text: unsafe { core::mem::zeroed() }, log: vec![], mutes: vec![], voices: 5 };
    s.fn_SetupSoundInfo(&mut h);
    s.fn_CheckSoundInfo(&mut h, 0);
    assert!(h.mutes.is_empty());
    h.down = pad::DPADLEFT;
    h.pressed = pad::X;
    for _ in 0..4 {
        s.fn_CheckSoundInfo(&mut h, 0);
    }
    assert_eq!(h.mutes, [(0, 2), (0, 0), (1, 0), (1, 2)]);
    assert_eq!(s.show_sound_info, 4);
    assert!(h.log.contains(&"FGM:ON   BGM:ON ".to_string()));
    assert!(h.log.contains(&"PVoice    6    5\n".to_string()));
    assert!(h.log.contains(&"VVoice    6    6".to_string()));
    // other players never toggle
    s.fn_CheckSoundInfo(&mut h, 1);
    assert_eq!(h.mutes.len(), 4);
}
