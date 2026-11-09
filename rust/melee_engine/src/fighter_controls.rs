//! Per-player fighter controls in a match (docs/geno.md section 26): pc/platform/gw_fighter_controls_runtime.inc and
//! gw_replay.c's gw_FighterControls_MatchBegin. At match start every human port's choice (Tap jump; a Geno define's
//! layout and extras) is frozen into a button map; controller.c's HSD_PadRenewMasterStatus maps each raw sample once
//! (`FighterControls_Apply`) and the jump requests read `FighterControls_TapJumpOff`. The mapping itself is
//! melee_script::geno::controls (gw_fighter_controls_model.h); a define's schema comes from the Geno registry.
//!
//! The frozen table is a static, so a snapshot carries it and a load puts it back (the C copies it by hand:
//! gw_FighterControls_StateSave / StateLoad). Choices are read only when a match is frozen, never from the table's
//! users: what was frozen is what plays, as in the C.
use melee_script::geno::controls::{self as fc, BUTTONS};

#[derive(Clone, Copy)]
struct Frozen {
    active: bool,
    ck: i32,
    choice: u32,
    map: [u16; BUTTONS],
}
const NONE: Frozen = Frozen { active: false, ck: -1, choice: 0, map: [0; BUTTONS] };

/// fc_match: one frozen mapping per port.
static mut MATCH: [Frozen; 4] = [NONE; 4];

/// Where a match's choices come from (gw_FighterControls_MatchBegin): a netplay session's agreed words (the room's
/// `J` exchange: host's for player 1, guest's for player 2, gw_Netplay_FighterControls), an offline replay's recorded
/// words per controller (the GDC1 block, rp.controls), else each player's saved preference.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Session {
    Local,
    Netplay([u32; 2]),
    Replay([u32; 4]),
}
static mut SESSION: Session = Session::Local;

/// Set by the `melee` crate when a netplay match is agreed or a replay starts playing; Local when it ends.
pub fn set_session(s: Session) {
    unsafe { SESSION = s };
}

pub fn session() -> Session {
    unsafe { SESSION }
}

/// gw_Netplay_FighterControls(player): the agreed word of netplay player 0 / 1, 0 for the others
fn netplay_word(w: &[u32; 2], player: i32) -> u32 {
    if (0..2).contains(&player) { w[player as usize] } else { 0 }
}

/// gw_Replay_ControlsRaw: `!rp.active || rp.live`. An offline replay's samples were mapped when it was recorded.
pub fn controls_raw() -> bool {
    !matches!(session(), Session::Replay(_))
}

/// fc_ck_valid
fn ck_valid(ck: i32) -> bool {
    (0..128).contains(&ck) && ck != 33
}

fn schema(ck: i32) -> Option<melee_script::geno::controls::Schema> {
    ck_valid(ck).then(|| crate::geno_host::ControlsSchema(ck)).flatten()
}

/// gw_FighterControls_Normalize
pub fn Normalize(ck: i32, choice: u32) -> u32 {
    if ck_valid(ck) { fc::normalize(schema(ck).as_ref(), choice) } else { 0 }
}

/// gw_FighterControls_Count: layouts (field 0) or extras (1) the fighter declares
pub fn Count(ck: i32, field: i32) -> i32 {
    match schema(ck) {
        Some(s) if field == fc::LAYOUTS => s.layout.len() as i32,
        Some(s) if field == fc::EXTRAS => s.extra.len() as i32,
        _ => 0,
    }
}

/// gw_FighterControls_Name: a layout's or extra's name, "" when there is none
pub fn Name(ck: i32, field: i32, index: i32) -> String {
    let Some(s) = schema(ck) else { return String::new() };
    let rows = if field == fc::LAYOUTS { &s.layout } else if field == fc::EXTRAS { &s.extra } else { return String::new() };
    if index < 0 { String::new() } else { rows.get(index as usize).map_or(String::new(), |m| m.name.clone()) }
}

// ---- saved preferences (gw_fighter_controls_runtime.inc fc_tap_key / fc_preference_key / fc_read_saved / fc_save):
// Tap jump is one key per player, shared by every fighter; a layout and extras one key per player and fighter, only for a
// fighter that declares them, under its stable content identity (never a roster-order alias). The `melee` crate
// installs the store (STORE: a key's saved text, and a write that answers whether it was saved); without one (tests,
// tools) the values live for the process.

/// (read a key's saved text, write a key's value -> saved)
pub static mut STORE: Option<(fn(&str) -> Option<String>, fn(&str, u32) -> bool)> = None;
static MEMORY: std::sync::Mutex<Vec<(String, u32)>> = std::sync::Mutex::new(Vec::new());

/// The controller profile part of a key: `d<device>` for a recognised device, else `f<port>`, then `p<profile>`. This
/// port has no controller profiles yet, so every key is `f<port>_p0` (the C's no-device form).
fn player_part(port: i32) -> String {
    format!("fc_f{port}_p0_s{port}")
}

/// fc_tap_key
fn tap_key(port: i32) -> String {
    format!("{}_tap", player_part(port))
}

/// fc_preference_key: retail fighters by their kind, a Geno define by its 64-bit content id (as the C uses the m-ex
/// identity hash); None for anything else (never saved under a roster-order-dependent alias).
fn preference_key(port: i32, ck: i32) -> Option<String> {
    let id = if (0..33).contains(&ck) {
        format!("retail{ck:02}")
    } else {
        format!("{:016x}", crate::geno_host::DefineContentId(ck)?)
    };
    Some(format!("{}_{id}", player_part(port)))
}

/// fc_read_saved: decimal digits only (no sign), fitting 32 bits; anything else reads as unset
fn read_saved(key: &str) -> Option<u32> {
    let text = match unsafe { STORE } {
        Some((read, _)) => read(key)?,
        None => return MEMORY.lock().unwrap_or_else(|e| e.into_inner()).iter().find(|x| x.0 == key).map(|x| x.1),
    };
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<u64>().ok().filter(|&v| v <= u32::MAX as u64).map(|v| v as u32)
}

/// fc_save
fn save(key: &str, value: u32, port: i32) -> bool {
    let ok = match unsafe { STORE } {
        Some((_, write)) => write(key, value),
        None => {
            let mut m = MEMORY.lock().unwrap_or_else(|e| e.into_inner());
            match m.iter_mut().find(|x| x.0 == key) {
                Some(x) => x.1 = value,
                None => m.push((key.to_string(), value)),
            }
            true
        }
    };
    if !ok {
        eprintln!("fighter controls: P{} preference save failed", port + 1);
    }
    ok
}

fn has_options(ck: i32) -> bool {
    schema(ck).is_some_and(|s| s.any())
}

/// gw_FighterControls_Get: the player's saved choice for a fighter, normalized
pub fn Get(port: i32, ck: i32) -> u32 {
    if !(0..4).contains(&port) {
        return 0;
    }
    let mut choice = read_saved(&tap_key(port)).unwrap_or(0) & fc::TAP_MASK;
    if has_options(ck) {
        if let Some(k) = preference_key(port, ck) {
            choice |= read_saved(&k).unwrap_or(0) & !fc::TAP_MASK;
        }
    }
    Normalize(ck, choice)
}

/// gw_FighterControls_Set: saves a choice (normalized; the layout / extras part only for a fighter with options). The
/// panel changes one row per call, so at most one key is written and a failed save leaves both as they were. 0 for a
/// bad port or kind, or when the save failed.
pub fn Set(port: i32, ck: i32, choice: u32) -> i32 {
    if !(0..4).contains(&port) || !ck_valid(ck) {
        return 0;
    }
    let choice = Normalize(ck, choice);
    let old = Get(port, ck);
    if (choice ^ old) & fc::TAP_MASK != 0 && !save(&tap_key(port), choice & fc::TAP_MASK, port) {
        return 0;
    }
    if (choice ^ old) & !fc::TAP_MASK != 0 {
        let Some(k) = preference_key(port, ck).filter(|_| has_options(ck)) else { return 0 };
        if !save(&k, choice & !fc::TAP_MASK, port) {
            return 0;
        }
    }
    eprintln!("fighter controls: saved P{} ck {ck} choice {choice:04x}", port + 1);
    1
}

/// gw_FighterControls_MatchSet
pub fn MatchSet(port: i32, ck: i32, choice: u32) {
    if !(0..4).contains(&port) {
        return;
    }
    let m = unsafe { &mut *core::ptr::addr_of_mut!(MATCH) };
    m[port as usize] = NONE;
    if !ck_valid(ck) {
        return;
    }
    let s = schema(ck);
    let choice = fc::normalize(s.as_ref(), choice);
    m[port as usize] = Frozen { active: true, ck, choice, map: fc::build(s.as_ref(), choice) };
    eprintln!("fighter controls: frozen P{} ck {ck} choice {choice:04x}", port + 1);
}

fn frozen(port: i32) -> Option<Frozen> {
    if !(0..4).contains(&port) {
        return None;
    }
    let f = unsafe { (*core::ptr::addr_of!(MATCH))[port as usize] };
    f.active.then_some(f)
}

/// gw_FighterControls_MatchGet
pub fn MatchGet(port: i32) -> u32 {
    frozen(port).map_or(0, |f| f.choice)
}

/// gw_FighterControls_MatchCK
pub fn MatchCK(port: i32) -> i32 {
    frozen(port).map_or(-1, |f| f.ck)
}

/// gw_FighterControls_MatchClear
pub fn MatchClear() {
    unsafe { *core::ptr::addr_of_mut!(MATCH) = [NONE; 4] };
    eprintln!("fighter controls: match mapping cleared");
}

/// gw_FighterControls_Apply: the port's frozen map over a raw button word (identity without a frozen port)
pub fn Apply(port: i32, buttons: i32) -> i32 {
    match frozen(port) {
        Some(f) => fc::apply(&f.map, buttons),
        None => buttons,
    }
}

/// gw_FighterControls_TapJumpOff: the game-side jump gate (ftCo_Jump, aerial jumps, the held-stick multijump)
pub fn TapJumpOff(port: i32) -> i32 {
    frozen(port).is_some_and(|f| f.choice & fc::TAP_MASK != 0) as i32
}

/// gw_FighterControls_MatchBegin (fn_8016E730, after match setup / replay restoration): each human controller of the
/// first four players gets its choice frozen. PlayerInitData +0 is the CharacterKind, +1 the slot type (0 human); the
/// controller is the one gmvs.c gives the player (`Player_SetPadPort`): +4 `slot` - 1, or the player's own index when
/// slot is 0. The C (gw_replay.c) reads +7 as the controller, which is `sub_color` (the team shade, mn/types.h;
/// Slippi's Game Start agrees): every human was frozen onto controller 0 and the last one won, so P2's mapping never
/// applied and a netplay host played with the guest's layout. Fixed here and in the C (port/c-controls-slot).
pub unsafe fn MatchBegin(d: *mut melee_types::StartMeleeData) {
    MatchClear();
    if d.is_null() {
        return;
    }
    for p in 0..4 {
        let pl = &(*d).players[p];
        let port = if pl.slot == 0 { p as i32 } else { pl.slot as i32 - 1 };
        let ck = pl.ckind as i32;
        if pl.slot_type != 0 || port >= 4 {
            continue;
        }
        let (choice, how) = match session() {
            Session::Replay(w) => (w[port as usize], "replay"),
            Session::Netplay(w) => (netplay_word(&w, p as i32), "online"),
            Session::Local => (Get(port, ck), "offline"),
        };
        MatchSet(port, ck, choice);
        eprintln!("fighter controls: frozen P{} controller {} ck {ck} choice 0x{:04x} ({how})", p + 1, port + 1, MatchGet(port));
    }
}

/// Points the game's hooks here (before seams_gen fills the rest with "feature off").
pub fn install() {
    unsafe {
        melee_runtime::tu::controller::ext::hook::FighterControls_Apply = Some(Apply);
        melee_runtime::tu::controller::ext::hook::Replay_ControlsRaw = Some(|| controls_raw() as i32);
        melee_fighter::platform::hook::FighterControls_TapJumpOff = Some(TapJumpOff);
        melee_modes::tu::gmvs::ext::hook::FighterControls_MatchBegin = Some(|d| MatchBegin(d as *mut melee_types::StartMeleeData));
    }
}

/// Ports of pc/tests/fighter_controls_test.c's freeze and preference asserts (the registry and the frozen table are
/// process-wide: geno_host::TEST_REGISTRY).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::geno_host as gh;

    const DEFINE: &str = "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"controls-fixture\",\"name\":\"Controls Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"controls\":{\"layouts\":[{\"name\":\"Swap attacks\",\"buttons\":{\"A\":\"B\",\"B\":\"A\",\"Y\":\"B\"}},{\"name\":\"Standard\",\"buttons\":{\"A\":\"A\",\"B\":\"B\"}}],\"extras\":[{\"name\":\"Y boost\",\"buttons\":{\"Y\":[\"A\",\"B\"]}}]}}]}\0";

    #[test]
    fn freeze_apply_tap_and_preferences_like_c() {
        let _l = gh::TEST_REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
        MEMORY.lock().unwrap().clear();
        assert_eq!(gh::TestInstall(DEFINE.as_ptr() as *const i8), 1);
        let ck = gh::DefineResolve("controls-fixture");
        assert!(ck >= 34, "ck {ck}");
        assert_eq!((Count(ck, fc::LAYOUTS), Count(ck, fc::EXTRAS), Name(ck, fc::LAYOUTS, 1), Name(ck, fc::EXTRAS, 0)), (2, 1, "Standard".into(), "Y boost".into()));
        assert_eq!((Count(8, 0), Name(ck, fc::LAYOUTS, 2), Name(ck, 7, 0)), (0, String::new(), String::new()));

        // a fighter with no options: Apply is the identity; Tap jump off changes no button, only the jump gate
        for (tap, off) in [(fc::TAP_ON, 0), (fc::TAP_OFF, 1)] {
            MatchSet(0, 8, tap);
            for bits in [0u32, 0x100, 0x800, 0xc00, 0x1f7f, 0x8000_0100] {
                assert_eq!(Apply(0, bits as i32), bits as i32);
            }
            assert_eq!(TapJumpOff(0), off);
        }
        assert_eq!((TapJumpOff(-1), TapJumpOff(4), Apply(-1, 0x100), Apply(4, 0x100)), (0, 0, 0x100, 0x100));

        // the define: layout 0 (Swap attacks) is the default; Y boost; normalize drops what the schema lacks
        MatchSet(0, ck, 0);
        assert_eq!((Apply(0, 0x100), Apply(0, 0x200), Apply(0, 0x800)), (0x200, 0x100, 0x200));
        MatchSet(0, ck, 1 | 0x100);
        assert_eq!((Apply(0, 0x800), TapJumpOff(0), MatchGet(0), MatchCK(0)), (0x300, 1, 0x101, ck));
        assert_eq!(Normalize(ck, 0xffff_ffff), 0x101);
        assert_eq!((Normalize(8, 0xffff), Normalize(33, 1), Normalize(-1, 1)), (1, 0, 0));
        MatchClear();
        assert_eq!((MatchGet(0), MatchCK(0), Apply(0, 0x100), TapJumpOff(0)), (0, -1, 0x100, 0));

        // preferences: Tap jump belongs to the player (every fighter), a layout to the player and fighter
        assert_eq!(Set(0, ck, 1), 1);
        assert_eq!((Get(0, ck), Get(0, 8)), (1, 1));
        assert_eq!(Set(0, ck, 1 | 4), 1);
        assert_eq!((Get(0, ck), Get(0, 8)), (1 | 4, 1));
        assert_eq!(Set(0, 8, 1 | 4), 1, "no options on ck 8: the layout bits are dropped, Tap jump kept");
        assert_eq!(Get(0, 8), 1);
        assert_eq!((Get(1, ck), Get(0, 255), Set(4, ck, 1), Set(0, 33, 1)), (0, 0, 0, 0));
        assert_eq!(Set(0, ck, 4), 1);
        assert_eq!(Get(0, ck), 4);

        // MatchBegin: human controllers of the first four players only, each from its own preference (or the hook)
        let mut d = melee_types::StartMeleeData::zeroed();
        d.players[0].ckind = ck as i8;
        d.players[0].slot = 0; // its own index: controller 0
        d.players[1].ckind = 8;
        d.players[1].slot = 3; // controller 2
        d.players[2].ckind = 8;
        d.players[2].slot_type = 1; // a CPU
        d.players[2].slot = 4;
        d.players[3].slot = 5; // no such controller
        d.players[3].sub_color = 1; // a team shade is not a controller
        unsafe { MatchBegin(&mut d) };
        assert_eq!((MatchCK(0), MatchGet(0), MatchCK(2), MatchGet(2), MatchCK(3), MatchCK(1)), (ck, 4, 8, 0, -1, -1));
        // an offline replay: each controller's recorded word, and its samples are not mapped again
        set_session(Session::Replay([0, 0, fc::TAP_OFF, 0]));
        unsafe { MatchBegin(&mut d) };
        assert_eq!((TapJumpOff(2), MatchGet(0), controls_raw()), (1, 0, false));
        // netplay: player 1 the host's word, player 2 the guest's (by player, not controller), the rest 0
        set_session(Session::Netplay([1 | 4, fc::TAP_OFF]));
        unsafe { MatchBegin(&mut d) };
        assert_eq!((MatchGet(0), MatchGet(2), controls_raw()), (1 | 4, fc::TAP_OFF, true));
        set_session(Session::Local);
        // the regression (gw_replay.c read +7 = sub_color): P2 on controller 1 with a remapping layout is mapped there,
        // and P1's controller 0 keeps its own map; a team shade changes nothing
        let mut e = melee_types::StartMeleeData::zeroed();
        e.players[0].ckind = 8;
        e.players[0].slot = 1; // controller 0
        e.players[0].sub_color = 2;
        e.players[1].ckind = ck as i8;
        e.players[1].slot = 2; // controller 1
        e.players[1].sub_color = 3;
        for p in 2..6 {
            e.players[p].slot_type = melee_types::Gm_PKind_NA as u8;
        }
        assert_eq!(Set(1, ck, 0), 1);
        unsafe { MatchBegin(&mut e) };
        assert_eq!((MatchCK(0), MatchCK(1), MatchCK(2), MatchCK(3)), (8, ck, -1, -1));
        assert_eq!((Apply(1, 0x100), Apply(1, 0x800), Apply(0, 0x100)), (0x200, 0x200, 0x100), "P2's Swap attacks on controller 1 only");
        MatchClear();
        MEMORY.lock().unwrap().clear();
        gh::TestRestore();
    }
}
