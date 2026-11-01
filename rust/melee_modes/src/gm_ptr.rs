
//! Round 3: `src/melee/gm/` leaf functions that take pointers to game structs (no calls, no
//! globals). Same names as the C. The structs are the melee_types records (native layout, GC
//! offsets in their `GcLayout`). Checked by `tests/oracle_ptr.rs`: the C runs over a seeded
//! buffer viewed as its own struct type, the Rust over the same bytes viewed as the record,
//! and the return value and the bytes afterwards must match.
//!
//! Pointer conventions: a pointer the C dereferences is a reference here (a slice when the C
//! indexes it, `Option` when the C checks it for NULL); a pointer the C only stores or
//! compares stays a raw pointer.
#![allow(non_snake_case)]

use melee_types as mt;

use mt::{BracketEntry, CSSData, GameModeState, HSD_GObj, HSD_JObj, HSD_Text, HSD_psAppSRT, MatchEnd, PlayerInitData,
         StartMeleeData, TmData};

pub(crate) const CHKIND_NONE: i8 = mt::ChKind_None as i8;

pub(crate) const PKIND_NA: u8 = mt::Gm_PKind_NA as u8;

pub(crate) const PKIND_HUMAN: u8 = mt::Gm_PKind_Human as u8;

/// gm/forward.h `GM_NAMETAG_COUNT`.
pub const GM_NAMETAG_COUNT: u8 = 120;

/// `gmClassicMatchup` (gmclassic.c; the .c defines it, so melee_types only has it opaque).
/// TODO(group0): generate records defined in .c files.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct gmClassicMatchup {
    pub x00: u16,
    /// `union { s8 x02[3]; u8 x02_u8[3]; }`
    pub x02: [u8; 3],
    pub x05: u8,
}

melee_core::gc_layout!(gmClassicMatchup, "gmClassicMatchup", size 6, align 2, [
    x00 @ 0x0: 2 "u16",
    x02 @ 0x2: 3 "u8[3]",
    x05 @ 0x5: 1 "u8",
]);

/// `team_standings[i].active` for i up to 5. The C's second team loop in `gm_80166CCC` runs
/// to 6 over a 5-entry array, so it reads `team_standings[5].active`, a byte inside
/// `player_standings[0]`. MatchEnd has no pointer before player_standings, so that byte is
/// the same one on the GameCube and natively.
pub(crate) fn team_active(me: &MatchEnd, i: usize) -> u8 {
    let off = core::mem::offset_of!(MatchEnd, team_standings)
        + i * core::mem::size_of::<mt::MatchTeamData>()
        + core::mem::offset_of!(mt::MatchTeamData, active);
    debug_assert!(off < core::mem::size_of::<MatchEnd>());
    // SAFETY: `off` lies inside *me (checked above for i <= 5).
    unsafe { *(me as *const MatchEnd as *const u8).add(off) }
}

pub use crate::tu::gm_1A3F::firstState_fp as firstState;
pub use crate::tu::gm_1601::fn_80161C90_count_players;
pub use crate::tu::gm_1601::fn_80165418_fp as fn_80165418;
pub use crate::tu::gm_1601::fn_801654A0_fp as fn_801654A0;
pub use crate::tu::gm_1601::fn_80165548;
pub use crate::tu::gm_1601::fn_801656A8;
pub use crate::tu::gm_1601::fn_80166CBC;
pub use crate::tu::gm_1601::fn_80167194;
pub use crate::tu::gm_181A::fn_80181E18_ComputeRemaining100;
pub use crate::tu::gmtoulib::fn_8018B090_inline4;
pub use crate::tu::gmtoulib::fn_8018F62C;
pub use crate::tu::gmtou_0::fn_80191240_save_root_jobj;
pub use crate::tu::gmtou_0::fn_80191FD4_is_selected_fp as fn_80191FD4_is_selected;
pub use crate::tu::gmtou_0::fn_80195CCC_IsUniqueEntry_fp as fn_80195CCC_IsUniqueEntry;
pub use crate::tu::gmtou_1::fn_80196564_fp as fn_80196564;
pub use crate::tu::gmtou_1::fn_80196594_fp as fn_80196594;
pub use crate::tu::gmsupersudden::fn_801B8C5C_fp as fn_801B8C5C;
pub use crate::tu::gmtiny::fn_801B8E50_fp as fn_801B8E50;
pub use crate::tu::gmgiant::fn_801B9060_fp as fn_801B9060;
pub use crate::tu::gmslomo::fn_801BA5B4_fp as fn_801BA5B4;
pub use crate::tu::gmlightning::fn_801BA7AC_fp as fn_801BA7AC;
pub use crate::tu::gmvs::getPort_fp as getPort;
pub use crate::tu::gm_1798::get_big_loser_fp as get_big_loser;
pub use crate::tu::gmcamera::gmCamera_801A2BF0_get_translate_x_fp as gmCamera_801A2BF0_get_translate_x;
pub use crate::tu::gmclassic::gmClassic_GetMatchupCount;
pub use crate::tu::gmmain_lib::gmMainLib_8015EF30_fp as gmMainLib_8015EF30;
pub use crate::tu::gmmain_lib::gmMainLib_AdjustNameTag_fp as gmMainLib_AdjustNameTag;
pub use crate::tu::gmmain_lib::gmMainLib_ClearNameTag_fp as gmMainLib_ClearNameTag;
pub use crate::tu::gmresult::gmResultFindNth_fp as gmResultFindNth;
pub use crate::tu::gmtoulib::gmTournament_SetBracketByes;
pub use crate::tu::gmvsmelee::gmVsMelee_WasAnyPlayerHuman_fp as gmVsMelee_WasAnyPlayerHuman;
pub use crate::tu::gm_1601::gm_80166CCC;
pub use crate::tu::gm_1601::gm_801677C0_fp as gm_801677C0;
pub use crate::tu::gm_1601::gm_80168940_fp as gm_80168940;
pub use crate::tu::gmregcommon::gm_8017BE8C_fp as gm_8017BE8C;
pub use crate::tu::gm_17c0::gm_8017CE34_CountEnemies_fp as gm_8017CE34_CountEnemies;
pub use crate::tu::gm_17c0::gm_8017CE34_GetCpuLevel_fp as gm_8017CE34_GetCpuLevel;
pub use crate::tu::gm_17c0::gm_8017DB58_fp as gm_8017DB58;
pub use crate::tu::gm_17c0::gm_8017DB6C_fp as gm_8017DB6C;
pub use crate::tu::gm_17c0::gm_8017DB78_fp as gm_8017DB78;
pub use crate::tu::gm_181A::gm_80182578_GetIndexFromPointer;
pub use crate::tu::gmregenddisp::gm_801A9094_get_entry;
pub use crate::tu::gmstaffroll::gm_801AB200_SetAppSRT;
pub use crate::tu::gm_1B03::gm_801B0474_inline;
pub use crate::tu::gm_1B03::gm_801B05F4_fp as gm_801B05F4;
pub use crate::tu::gm_1B03::gm_801B0730_fp as gm_801B0730;
pub use crate::tu::gm_1B03::gm_801B07B4;
pub use crate::tu::gm_1B03::gm_801B07E8;
pub use crate::tu::gm_1601::is_character_unlocked;
pub use crate::tu::gmresult::matchWasSkipped_fp as matchWasSkipped;
pub use crate::tu::gm_1B03::player_standings_inline;
pub use crate::tu::gm_1884::resetText;
