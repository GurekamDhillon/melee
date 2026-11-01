//! src/melee/gm/gm_1B03.c: VS setup helpers (team tints, sudden death, per-slot player init, the CSS
//! player/opponent accessors). The functions group 6 ported earlier (`gm_ptr`, `gm_r4`) are re-exported;
//! the rest are new here. All of them run in the state oracle (tests/tu_gm_1B03.rs over
//! gen/turef/tu_gm_1B03_ref.c, the game's gm_1B03.c with its callees stubbed) and are compared field by
//! field. The three `static` helpers of gmscenelaunch.h that this TU compiles in (`SceneLaunch_HudCount`,
//! `SceneLaunch_UntintedEnemy`, `SceneLaunch_ApplyTeamCostumes`) are ported privately, with their port-side
//! callees as hooks.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use melee_types as mt;
#[allow(unused_imports)]
use mt::CSSData as CSSData_gm_ptr;
#[allow(unused_imports)]
use mt::MatchEnd as MatchEnd_gm_ptr;
#[allow(unused_imports)]
use mt::PlayerInitData;
#[allow(unused_imports)]
use mt::StartMeleeData as StartMeleeData_gm_ptr;
#[allow(unused_imports)]
use crate::gm_ptr::{PKIND_NA as PKIND_NA_gm_ptr};
use melee_types::*;

pub use crate::gm_ptr::{gm_801B05F4, gm_801B0730};pub use crate::gm_r4::{gm_SetupCpuPlayer, gm_SetupHumanPlayer};

const PKIND_NA: u8 = Gm_PKind_NA as u8;
const PKIND_CPU: u8 = Gm_PKind_Cpu as u8;
const GM_MAX: usize = GM_MAX_PLAYERS as usize;

/// Calls into TUs that are not ported yet (gm_1884.c VS init, gm_1601.c costume tables, the port's scene launch).
/// Each is a settable hook; an unset hook panics.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:path) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    /// `gm_InitVsMode(vs)` (gm/gm_1601.c or gmmain_lib.c): resets a `VsModeData` to the defaults.
    pub static mut GM_INITVSMODE: Option<fn(*mut VsModeData) > = None;
            #[allow(non_snake_case)]
            pub fn gm_InitVsMode(vs: *mut VsModeData)  {
                match unsafe { GM_INITVSMODE } {
                    Some(f) => f(vs),
                    None => { unsafe { crate::tu::gm_1601::gm_InitVsMode(vs as _); } return; },
                }
            }
    /// `SceneLaunch_Active()` / `SceneLaunch_EnemyTeamColors()` (pc/platform/gw_runtime.c, TARGET_PC).
    hook!(SCENELAUNCH_ACTIVE, SceneLaunch_Active, () -> i32);
    hook!(SCENELAUNCH_ENEMYTEAMCOLORS, SceneLaunch_EnemyTeamColors, () -> i32);
    /// The team-colour tables of gm_1601.c.
    pub static mut GM_801692BC: Option<fn(i8) -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_801692BC(ckind: i8) -> u8 {
                match unsafe { GM_801692BC } {
                    Some(f) => f(ckind),
                    None => { return unsafe { crate::tu::gm_1601::gm_801692BC(ckind as _) as _ }; },
                }
            }
    pub static mut GM_80169290: Option<fn(i8) -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_80169290(ckind: i8) -> u8 {
                match unsafe { GM_80169290 } {
                    Some(f) => f(ckind),
                    None => { return unsafe { crate::tu::gm_1601::gm_80169290(ckind as _) as _ }; },
                }
            }
    pub static mut GM_80169264: Option<fn(i8) -> u8> = None;
            #[allow(non_snake_case)]
            pub fn gm_80169264(ckind: i8) -> u8 {
                match unsafe { GM_80169264 } {
                    Some(f) => f(ckind),
                    None => { return unsafe { crate::tu::gm_1601::gm_80169264(ckind as _) as _ }; },
                }
            }
}

/// gmscenelaunch.h `SceneLaunch_HudCount`: the highest occupied slot (P5/P6) selects the HUD layout.
pub fn SceneLaunch_HudCount(start: &mut StartMeleeData) {
    melee_core::ct!("SceneLaunch_HudCount");
    for i in 4..GM_MAX {
        if start.players[i].slot_type != PKIND_NA {
            start.rules.set_x0_3((i + 1) as u32);
        }
    }
}

/// gmscenelaunch.h `SceneLaunch_UntintedEnemy`: an opposing-team slot that keeps its base costume.
pub fn SceneLaunch_UntintedEnemy(start: &StartMeleeData, i: usize) -> bool {
    melee_core::ct!("SceneLaunch_UntintedEnemy");
    ext::SceneLaunch_Active() != 0
        && ext::SceneLaunch_EnemyTeamColors() != 0
        && start.rules.is_teams != 0
        && start.players[i].slot_type != PKIND_NA
        && start.players[i].team != start.players[0].team
}

/// gmscenelaunch.h `SceneLaunch_ApplyTeamCostumes`: share costumes without duplicate tinting on the opposing team.
pub fn SceneLaunch_ApplyTeamCostumes(start: &mut StartMeleeData) {
    melee_core::ct!("SceneLaunch_ApplyTeamCostumes");
    for i in 0..GM_MAX {
        if SceneLaunch_UntintedEnemy(start, i) {
            let p = &mut start.players[i];
            p.color = if p.team == 1 {
                ext::gm_801692BC(p.ckind)
            } else if p.team == 2 {
                ext::gm_80169290(p.ckind)
            } else {
                ext::gm_80169264(p.ckind)
            };
            p.sub_color = 0;
        }
    }
}

/// `gm_SetupSubColors(start)`: in team play, bump the tint of a player who shares character, team and tint with another.
pub fn gm_SetupSubColors(start: &mut StartMeleeData) {
    melee_core::ct!("gm_SetupSubColors");
    if start.rules.is_teams != 1 {
        return;
    }
    for i in 0..GM_MAX {
        // TARGET_PC
        if SceneLaunch_UntintedEnemy(start, i) {
            start.players[i].sub_color = 0;
            continue;
        }
        for j in 0..GM_MAX {
            // TARGET_PC
            if SceneLaunch_UntintedEnemy(start, j) {
                continue;
            }
            if i == j {
                continue;
            }
            if start.players[i].team == start.players[j].team
                && start.players[i].ckind == start.players[j].ckind
                && start.players[i].sub_color == start.players[j].sub_color
            {
                start.players[j].sub_color = start.players[j].sub_color.wrapping_add(1);
            }
        }
    }
}

/// `SceneLaunch_SixPresentationTest(force)` (TARGET_PC): the six-player HUD/tint self test; 0 on success, 1 on failure.
pub fn SceneLaunch_SixPresentationTest(force: i32) -> i32 {
    melee_core::ct!("SceneLaunch_SixPresentationTest");
    let mut start = StartMeleeData::zeroed();
    start.rules.set_x0_3(4);
    start.rules.is_teams = 1;
    for i in 0..GM_MAX {
        start.players[i].slot_type = PKIND_CPU;
        start.players[i].ckind = CKind_Fox as i8;
        start.players[i].team = if i != 0 { 1 } else { 0 };
    }
    SceneLaunch_HudCount(&mut start);
    if start.rules.x0_3() != 6 {
        return 1;
    }
    SceneLaunch_ApplyTeamCostumes(&mut start);
    gm_SetupSubColors(&mut start);
    for i in 1..GM_MAX {
        if force != 0 {
            if start.players[i].sub_color != 0 || start.players[i].color != ext::gm_801692BC(CKind_Fox as i8) {
                return 1;
            }
        } else {
            if start.players[i].sub_color > 4 {
                return 1;
            }
            for j in 1..i {
                if start.players[i].sub_color == start.players[j].sub_color {
                    return 1;
                }
            }
        }
    }
    start.players[4].slot_type = PKIND_NA;
    start.rules.set_x0_3(4);
    SceneLaunch_HudCount(&mut start);
    if start.rules.x0_3() != 6 {
        return 1;
    }
    start.players[5].slot_type = PKIND_NA;
    start.rules.set_x0_3(4);
    SceneLaunch_HudCount(&mut start);
    (start.rules.x0_3() != 4) as i32
}

/// `gm_SetupSuddenDeath(start, end)`: restart the match as one-stock sudden death between the players who did not lose.
pub fn gm_SetupSuddenDeath(start: &mut StartMeleeData, end: &MatchEnd) {
    melee_core::ct!("gm_SetupSuddenDeath");
    start.rules.set_match_kind(MatchKind_Stock as u32);
    start.rules.set_timer_enabled(0);
    start.rules.set_x2_5(0);

    for i in 0..GM_MAX {
        if start.players[i].slot_type != PKIND_NA {
            let var_r7 = gm_801B0474_inline(end, i as i32);
            if end.is_teams == 1 {
                let team = end.player_standings[i].team as usize;
                player_standings_inline(start, end, i as i32, end.team_standings[team].is_big_loser as u32, var_r7);
            } else {
                player_standings_inline(start, end, i as i32, end.player_standings[i].is_big_loser as u32, var_r7);
            }
        }
    }
}

/// `gm_801B06B0(css_data, type, c_kind, stocks, color, arg5, level, slot)`: reset the VS data and seed the CSS player's slot.
pub fn gm_801B06B0(css_data: &mut CSSData, ty: u8, c_kind: i8, stocks: i8, color: i8, arg5: u8, level: u8, slot: u8) {
    melee_core::ct!("gm_801B06B0");
    ext::gm_InitVsMode(&mut css_data.vs);
    css_data.match_type = ty;
    css_data.unk_0x0 = (slot as u16).wrapping_add(1);
    let p = &mut css_data.vs.start.players[slot as usize];
    p.ckind = c_kind;
    p.stocks = stocks;
    p.color = color as u8;
    p.cpu_level = level;
    p.nametag = arg5;
    css_data.vs.start.players[0].cpu_level = level;
    css_data.vs.start.players[0].stocks = stocks;
}

// ---- folded from gm_ptr.rs (consolidation steps 21/22) ----
/// gm_1B03.c `player_standings_inline`.
pub fn player_standings_inline(arg0: &mut StartMeleeData_gm_ptr, arg1: &MatchEnd_gm_ptr, i: i32, is_big_loser: u32, var_r7: i32) {
    melee_core::ct!("player_standings_inline");
    let i = i as usize;
    if is_big_loser == 0 && var_r7 > 0 {
        let mut ckind = arg1.player_standings[i].ckind;
        if ckind == 0x12 || ckind == 0x13 {
            ckind = if arg1.player_standings[i].ftkind == 7 { 0x13 } else { 0x12 };
        }
        arg0.players[i].ckind = ckind;
        arg0.players[i].stocks = 1;
        arg0.players[i].damage1 = 300;
    } else {
        arg0.players[i].slot_type = PKIND_NA_gm_ptr;
    }
}

/// gm_1B03.c `gm_801B0474_inline`.
pub fn gm_801B0474_inline(arg1: &MatchEnd_gm_ptr, i: i32) -> i32 {
    melee_core::ct!("gm_801B0474_inline");
    if arg1.match_kind == 1 {
        let p = &arg1.player_standings[i as usize];
        if arg1.outcome == mt::OUTCOME_TIMEOUT as u8 {
            return p.stocks as i32;
        }
        let stocks = p.stocks as u8;
        if p.x28 < arg1.frame_count || stocks != 0 {
            return stocks as i32;
        }
    }
    1
}

/// gm_1B03.c `gm_801B05F4_fp`: port slot and team (slots 2 and 3 swap teams).
pub fn gm_801B05F4_fp(player: &mut PlayerInitData, mut slot: i32) {
    melee_core::ct!("gm_801B05F4_fp");
    player.slot = slot.wrapping_add(1) as u8;
    if slot == 2 {
        slot = 3;
    } else if slot == 3 {
        slot = 2;
    }
    player.team = slot as u8;
}

/// gm_1B03.c `gm_801B0730_fp`: read the CSS player's choices (`unk_0x0` is the 1-based slot).
pub fn gm_801B0730_fp(css_data: &CSSData_gm_ptr, c_kind: Option<&mut i8>, stocks: Option<&mut u8>, color: Option<&mut u8>,
                   nametag: Option<&mut u8>, level: Option<&mut u8>) {
    melee_core::ct!("gm_801B0730_fp");
    let p = &css_data.vs.start.players[(css_data.unk_0x0 as i32 - 1) as usize];
    if let Some(c) = c_kind {
        *c = p.ckind;
    }
    if let Some(s) = stocks {
        *s = p.stocks as u8;
    }
    if let Some(c) = color {
        *c = p.color;
    }
    if let Some(l) = level {
        *l = p.cpu_level;
    }
    if let Some(n) = nametag {
        *n = p.nametag;
    }
}

/// gm_1B03.c `gm_801B07B4`: write the opponent's choices (slot 1, or 0 when `arg6 != 0`).
pub fn gm_801B07B4(css_data: &mut CSSData_gm_ptr, c_kind: i8, stocks: i8, color: i8, arg4: u8, level: u8, arg6: u8) {
    melee_core::ct!("gm_801B07B4");
    let p = &mut css_data.vs.start.players[if arg6 == 0 { 1 } else { 0 }];
    p.ckind = c_kind;
    p.stocks = stocks;
    p.color = color as u8;
    p.cpu_level = level;
    p.nametag = arg4;
}

/// gm_1B03.c `gm_801B07E8`: read the opponent's choices.
pub fn gm_801B07E8(css_data: &CSSData_gm_ptr, c_kind: Option<&mut i8>, stocks: Option<&mut i8>, color: Option<&mut i8>,
                   arg4: Option<&mut i8>, level: Option<&mut u8>) {
    melee_core::ct!("gm_801B07E8");
    let slot = if css_data.unk_0x0 as i32 - 1 == 0 { 1 } else { 0 };
    let p = &css_data.vs.start.players[slot];
    if let Some(c) = c_kind {
        *c = p.ckind;
    }
    if let Some(s) = stocks {
        *s = p.stocks;
    }
    if let Some(c) = color {
        *c = p.color as i8;
    }
    if let Some(l) = level {
        *l = p.cpu_level;
    }
    if let Some(n) = arg4 {
        *n = p.nametag as i8;
    }
}

// ---- folded from gm_r4.rs (consolidation steps 21/22) ----
/// gm_1B03.c `gm_SetupHumanPlayer_fp`.
pub fn gm_SetupHumanPlayer_fp(player: &mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
    melee_core::ct!("gm_SetupHumanPlayer_fp");
    player.slot_type = mt::Gm_PKind_Human as u8;
    player.ckind = ckind as i8;
    player.color = color;
    player.stocks = stocks as i8;
    gm_801B05F4(player, slot as i32);
}

/// gm_1B03.c `gm_SetupCpuPlayer_fp`: as the human setup, then team 4.
pub fn gm_SetupCpuPlayer_fp(player: &mut PlayerInitData, ckind: u8, color: u8, stocks: u8, slot: u8) {
    melee_core::ct!("gm_SetupCpuPlayer_fp");
    player.slot_type = mt::Gm_PKind_Cpu as u8;
    player.ckind = ckind as i8;
    player.color = color;
    player.stocks = stocks as i8;
    gm_801B05F4(player, slot as i32);
    player.team = 4;
}
