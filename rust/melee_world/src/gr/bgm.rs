//! Stage music selection: `Ground_801C24F8` (ground.c), reached from `Stage_80225074` at match
//! start. Some rows roll for the alternate track, so the pick draws from the simulation RNG.
//!
//! Native shape: the stage file's `grGroundParam->stage_params` rows are [`StageParam`] (read
//! from the DAT by [`StageParam::parse_all`]); the unlock checks the rows consult are
//! [`BgmUnlocks`]. `stage_info.unk8C.b0` is returned as [`BgmPick::alt_flag`] (`None` = left
//! as it was). Not ported: the TARGET_PC m-ex paths (`Ground_MexParamStkind`, the m-ex playlist
//! fallback, `stage_music_5050`) and `bgm == -2`'s `lbAudioAx_8002305C` per-character track
//! (TODO(group1 lbaudio)); the run panics if it reaches them.

/// One `StageParam` row (GC 0x64; only the music fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageParam {
    pub stkind: i32,
    pub x4: i32,
    pub x8: i32,
    pub x_c: u32,
    pub x10: u32,
    pub x14: i16,
    pub x16: i16,
}

impl StageParam {
    /// `param->stage_params[0..stage_param_count]` from the stage DAT's data section.
    /// `param` is the `grGroundParam` public offset; `reloc` resolves a pointer word.
    pub fn parse_all(d: &[u8], param: u32) -> Vec<StageParam> {
        let be32 = |at: u32| u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap());
        let be16 = |at: u32| i16::from_be_bytes(d[at as usize..at as usize + 2].try_into().unwrap());
        let (rows, n) = (be32(param + 0xB0), be32(param + 0xB4));
        (0..n)
            .map(|i| {
                let e = rows + i * 0x64;
                StageParam {
                    stkind: be32(e) as i32,
                    x4: be32(e + 4) as i32,
                    x8: be32(e + 8) as i32,
                    x_c: be32(e + 0xC),
                    x10: be32(e + 0x10),
                    x14: be16(e + 0x14),
                    x16: be16(e + 0x16),
                }
            })
            .collect()
    }
}

/// The save-data checks `Ground_801C24F8` makes.
pub trait BgmUnlocks {
    /// `gm_IsCKindUnlocked(CKind_Mars)`.
    fn mars(&self) -> bool;
    /// `gm_IsCKindUnlocked(CKind_CLink)`.
    fn clink(&self) -> bool;
    /// `Toy_803048C0(0x11A)`.
    fn toy_11a(&self) -> bool;
    /// `gm_80164ABC()` (every unlockable character unlocked).
    fn all_characters(&self) -> bool;
    /// `gm_80164600()`.
    fn gm_80164600(&self) -> bool;
}

const BGM_UNDEFINED: i32 = -1;
/// ground.c `RANDI_MAX`.
const RANDI_MAX: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BgmPick {
    /// The function's return (an alternate track was chosen).
    pub result: bool,
    /// `*arg2`.
    pub bgm: i32,
    /// `stage_info.unk8C.b0` when written.
    pub alt_flag: Option<bool>,
}

/// `Ground_801C24F8(stkind, arg1, &bgm)`. `randi` is `HSD_Randi`.
pub fn ground_801c24f8(rows: &[StageParam], stkind: i32, mut arg1: u32, unlocks: &dyn BgmUnlocks, randi: &mut dyn FnMut(i32) -> i32) -> BgmPick {
    let mut bgm = BGM_UNDEFINED;
    let mut result = false;
    let mut alt_flag = None;
    for p in rows {
        if p.stkind != stkind {
            continue;
        }
        if arg1 & 4 != 0 {
            let forced = arg1 & 0x40 != 0;
            // `x16 > HSD_Randi(RANDI_MAX) || forced`: the draw happens even when forced.
            let roll = |randi: &mut dyn FnMut(i32) -> i32| p.x16 as i32 > randi(RANDI_MAX) || forced;
            let alt = match p.x14 {
                0 => Some(forced),
                1 => Some(randi(0xC) == 0 || forced),
                8 => Some(false),
                2 => Some(roll(randi)),
                3 => Some(unlocks.mars() && roll(randi)),
                4 => Some(unlocks.clink() && roll(randi)),
                5 => Some(unlocks.toy_11a() && roll(randi)),
                6 => Some(unlocks.all_characters() && roll(randi)),
                7 => Some(unlocks.gm_80164600() && roll(randi)),
                _ => None,
            };
            match alt {
                Some(true) => arg1 |= 2,
                Some(false) => arg1 |= 1,
                None => {}
            }
        }
        if arg1 & 0x10 != 0 {
            alt_flag = Some(false);
            if arg1 & 1 != 0 {
                bgm = p.x_c as i32;
            } else if arg1 & 2 != 0 {
                if p.x10 != BGM_UNDEFINED as u32 {
                    bgm = p.x10 as i32;
                    result = true;
                } else {
                    bgm = p.x_c as i32;
                }
            }
        } else if arg1 & 0x20 != 0 {
            if arg1 & 1 != 0 {
                bgm = p.x_c as i32;
                alt_flag = Some(false);
            } else if arg1 & 2 != 0 {
                if p.x10 != BGM_UNDEFINED as u32 {
                    bgm = p.x10 as i32;
                    alt_flag = Some(true);
                    result = true;
                } else {
                    bgm = p.x_c as i32;
                    alt_flag = Some(false);
                }
            }
        } else if arg1 & 1 != 0 {
            bgm = p.x4;
            alt_flag = Some(false);
        } else if arg1 & 2 != 0 {
            if p.x8 as u32 != BGM_UNDEFINED as u32 {
                bgm = p.x8;
                alt_flag = Some(true);
                result = true;
            } else {
                bgm = p.x4;
                alt_flag = Some(false);
            }
        }
        break;
    }
    assert!(bgm != BGM_UNDEFINED, "Ground_801C24F8: no music row for stkind {stkind:#x} (m-ex playlist fallback not ported)");
    if bgm == -2 {
        panic!("Ground_801C24F8: per-character track lbAudioAx_8002305C (TODO(group1 lbaudio))");
    }
    BgmPick { result, bgm, alt_flag }
}

/// `Stage_80225074`'s `arg1` for a VS match (`gm_8016B238` / `gm_8016B3A0` false): 4, or 0x44
/// with the alternate-music button held (`fn_8016E5C0`'s result 1), 2 for 2, 1 for 3.
pub fn stage_80225074_flags(arg0: i32) -> u32 {
    match arg0 {
        0 => 4,
        3 => 1,
        2 => 2,
        1 => 0x44,
        _ => panic!("Stage_80225074: arg0 {arg0}"),
    }
}
