//! tu::gm_1601 vs the C (gen/cref/tu/gm_1601.c: the whole TU, shipping contraction, 4-byte pointers;
//! regenerate with `python rust/melee_modes/gen/cref/tu/run_tu.py gm_1601`).
//!
//! Every callee out of the TU is a recording stub (tests/tu_gm_1601_stubs, generated); the pointer-returning
//! save-data getters hand out lazily random scratch memory; the game state the TU reads (the match results, the
//! scene state, the rules...) is synthetic and filled from the shared generator. The ops (tests/tu_gm_1601_ops) are
//! generated from the OPS block of the C driver. One line per case: op case ret state-hash trace.
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, dead_code)]
mod tu_gm_1601_hs;
mod tu_gm_1601_ops;
mod tu_gm_1601_stubs;
mod tu_gm_harness;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::tu::globals::{stage_info, CostumeListsForeachCharacter, HSD_PadMasterStatus, HSD_VIData};
use melee_modes::tu::gm_1601 as m;
use melee_types::*;
use tu_gm_1601_hs::*;
use tu_gm_harness::*;

pub const G_ME: u32 = 1;
pub const G_ME2: u32 = 2;
pub const G_VSS: u32 = 4;
pub const G_RULES: u32 = 8;
pub const G_FD: u32 = 16;
pub const G_NTD: u32 = 32;
pub const G_LBL: u32 = 64;
pub const G_TXT: u32 = 128;
pub const G_VSM: u32 = 256;
pub const G_SSS: u32 = 512;
pub const G_PID: u32 = 1024;
pub const G_DT: u32 = 2048;
pub const G_MISC: u32 = 4096;
pub const G_GLOB: u32 = 8192;
pub const G_MEX: u32 = 16384;
pub const G_DMD: u32 = 32768;
pub const G_GMS: u32 = 65536;
pub const G_G677: u32 = 131072;
pub const G_PERF: u32 = 262144;
pub const G_V3: u32 = 524288;
const NCASES: u32 = 600;

pub use m::lbl_8046B378;

pub static mut ME: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut ME2: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut VSS: VsSceneState = unsafe { core::mem::zeroed() };
pub static mut GRULES: GameRules = unsafe { core::mem::zeroed() };
pub static mut FD: [FighterData; 26] = unsafe { core::mem::zeroed() };
pub static mut NTD: [NameTagData; 4] = unsafe { core::mem::zeroed() };
pub static mut LBL: lbl_8046B488_t = unsafe { core::mem::zeroed() };
pub static mut TOYF: ToyModelFile = unsafe { core::mem::zeroed() };
pub static mut TXT: HSD_Text = unsafe { core::mem::zeroed() };
pub static mut VSM: VsModeData = unsafe { core::mem::zeroed() };
pub static mut SSS: SSSData = unsafe { core::mem::zeroed() };
pub static mut PID: [PlayerInitData; 6] = unsafe { core::mem::zeroed() };
pub static mut SMR: StartMeleeRules = unsafe { core::mem::zeroed() };
pub static mut DT: datetime = unsafe { core::mem::zeroed() };
pub static mut GST: GmStats = unsafe { core::mem::zeroed() };
pub static mut G677: gm_801677C0_s = unsafe { core::mem::zeroed() };
pub static mut V3A: Vec3 = unsafe { core::mem::zeroed() };
pub static mut V3B: Vec3 = unsafe { core::mem::zeroed() };
pub static mut OUT4: [u8; 4] = [0; 4];
pub static mut BYTES4: [u8; 4] = [0; 4];
pub static mut ITEMMASK: u64 = 0;
pub static mut MASK16: u16 = 0;
pub static mut HANDI: [i8; 16] = [0; 16];
pub static mut STC: m::MexStcIcns = unsafe { core::mem::zeroed() };
pub static mut MAJ: HSD_MatAnimJoint = unsafe { core::mem::zeroed() };
pub static mut MA: HSD_MatAnim = unsafe { core::mem::zeroed() };
pub static mut TA: HSD_TexAnim = unsafe { core::mem::zeroed() };
pub static mut AD: HSD_AObjDesc = unsafe { core::mem::zeroed() };
pub static mut ARCHD: [u8; 4] = [0; 4];
pub static mut DMD: DynamicModelDesc = unsafe { core::mem::zeroed() };
pub static mut ANIMS: [*mut HSD_AnimJoint; 4] = [null_mut(); 4];
pub static mut MATANIMS: [*mut HSD_MatAnimJoint; 4] = [null_mut(); 4];
pub static mut SHAPEANIMS: [*mut HSD_ShapeAnimJoint; 4] = [null_mut(); 4];

pub unsafe extern "C" fn cbR(_a: i32) -> i32 {
    0
}

fn shash(s: *const i8) -> u32 {
    if s.is_null() {
        0
    } else {
        let b = unsafe { core::ffi::CStr::from_ptr(s as *const _).to_bytes() };
        hbuf(b.as_ptr(), b.len()).wrapping_add(1)
    }
}

fn install_manual() {
    use m::ext::hook as h;
    unsafe {
        h::memzero = Some(|p, n| {
            t(2000);
            t(n as u32);
            core::ptr::write_bytes(p as *mut u8, 0, n as usize);
        });
        h::HSD_Randi = Some(|max| {
            t(2001);
            t(max as u32);
            if max > 0 {
                (rnd() % max as u32) as i32
            } else {
                0
            }
        });
        h::gmVs_GetSceneState = Some(|| {
            t(2002);
            addr_of_mut!(VSS)
        });
        h::gmMainLib_GetGameRules = Some(|| {
            t(2003);
            addr_of_mut!(GRULES)
        });
        h::GetPersistentFighterData = Some(|sk| {
            t(2004);
            t(sk as u32);
            addr_of_mut!(FD[(sk as u32 % 26) as usize])
        });
        h::GetPersistentNameData = Some(|n| {
            t(2005);
            t(n as u32);
            addr_of_mut!(NTD[(n as u32 % 4) as usize])
        });
        h::gm_1601_GetUnkData = Some(|| {
            t(2006);
            addr_of_mut!(LBL)
        });
        h::Toy_8030813C = Some(|id| {
            t(2007);
            t(id as u32);
            addr_of_mut!(TOYF)
        });
        h::gm_GetCurrentGameMode = Some(|| {
            t(2008);
            let r = rnd();
            if r % 3 == 0 {
                31
            } else {
                (rnd() % 45) as u8
            }
        });
        h::Stage_80224E38 = Some(|v, id| {
            t(2009);
            t(id as u32);
            (*v).x = rg();
            (*v).y = rg();
            (*v).z = rg();
        });
        h::lbTime_8000B028 = Some(|tm, secs| {
            t(2010);
            t(secs);
            fl_OSCalendarTime(&mut *tm);
        });
        h::lbDvd_8001819C = Some(|name| {
            t(2011);
            t(shash(name));
            if rnd() & 1 != 0 {
                ARCHD.as_mut_ptr() as *mut HSD_Archive
            } else {
                null_mut()
            }
        });
        h::HSD_ArchiveGetPublicAddress = Some(|a, s| {
            t(2012);
            t(!a.is_null() as u32);
            t(shash(s));
            addr_of_mut!(STC) as *mut c_void
        });
        h::HSD_SisLib_803A6B98 = Some(|tx, x, y, s| {
            t(2013);
            t((tx == addr_of_mut!(TXT)) as u32);
            t(x.to_bits());
            t(y.to_bits());
            t(shash(s));
            rnd() as i32
        });
        h::HSD_SisLib_803A70A0 = Some(|tx, e, f| {
            t(2014);
            t((tx == addr_of_mut!(TXT)) as u32);
            t(e as u32);
            t(shash(f));
            rnd() as i32
        });
        h::gmMainLib_8015CE44 = Some(|a, b| {
            t(2015);
            t(a as u32);
            t(b as u32);
            if rnd() % 4 == 0 {
                return null_mut();
            }
            addr_of_mut!(HANDI[(a as u32 % 16) as usize])
        });
        h::fn_80169434 = Some(|f| {
            t(2016);
            t(f.is_some() as u32);
        });
        h::hsd_80392528 = Some(|e| {
            t(2017);
            t((e.map(|f| f as usize) == Some(m::fn_801652D8 as usize)) as u32);
        });
    }
}

pub fn ckgen() -> i32 {
    let r = rnd();
    if r % 9 == 0 {
        (34 + rnd() % 94) as i32
    } else {
        (rnd() % 33) as i32
    }
}
pub fn ckpgen() -> i32 {
    let r = rnd();
    if r % 9 == 0 {
        (34 + rnd() % 94) as i32
    } else {
        (rnd() % 26) as i32
    }
}
pub fn ckngen() -> i32 {
    let r = rnd();
    if r % 5 == 0 {
        return 33;
    }
    if r % 7 == 0 {
        return 19;
    }
    (rnd() % 26) as i32
}
pub fn modegen() -> i32 {
    (rnd() % 45) as i32
}

unsafe fn fix_me(me: *mut MatchEnd, k: u32) {
    fl_MatchEnd(&mut *me);
    let me = &mut *me;
    me.is_teams = (rnd() % 3 == 0) as u8;
    me.match_kind = (rnd() % 4) as u8;
    me.sd_penalty = (rnd() % 5) as i8 - 2;
    me.outcome = (rnd() % 10) as u8;
    me.n_winners = (rnd() % 7) as u8;
    me.n_team_winners = (rnd() % 6) as u8;
    for i in 0..6 {
        let p = &mut me.player_standings[i];
        p.pkind = (rnd() % 4) as u8;
        p.team = (rnd() % 5) as u8;
        p.is_small_loser = (rnd() % 4) as u8;
        p.is_big_loser = (rnd() % 6) as u8;
        p.x4 = (if rnd() % 8 == 0 { 0x78 } else { rnd() % 120 }) as u8;
        p.ckind = ckpgen() as i8;
        if k & 1 != 0 {
            p.x20 = rnd() % 5;
            p.x24 = (rnd() % 5) as i32;
            p.x30 = rnd() % 7;
            p.score = (rnd() % 9) as i32 - 4;
            p.x44 = rnd() % 4;
            p.x50 = rnd() % 4;
            p.x28 = if rnd() % 3 != 0 { rnd() % 3000 } else { 0 };
            p.x1C = (rnd() % 40) as i32 - 20;
            p.x9 = (rnd() % 5) as u8;
            p.stocks = (rnd() % 5) as i8 - 1;
            p.self_destructs = (rnd() % 4) as u16;
            for j in 0..4 {
                p.kills[j] = (rnd() % 4) as u16;
            }
        }
    }
    for i in 0..5 {
        let t = &mut me.team_standings[i];
        t.active = (rnd() % 2) as u8;
        t.is_big_loser = (rnd() % 5) as u8;
        t.is_small_loser = (rnd() % 5) as u8;
        if k & 1 != 0 {
            t.score = (rnd() % 9) as i32 - 4;
            t.subscore = (rnd() % 7) as i32 - 2;
        }
    }
    me.player_standings[0].pkind = (rnd() % 3) as u8;
}

unsafe fn setup(g: u32, k: u32) {
    if g & G_ME != 0 {
        fix_me(addr_of_mut!(ME), k);
    }
    if g & G_ME2 != 0 {
        fix_me(addr_of_mut!(ME2), k + 1);
    }
    if g & G_VSS != 0 {
        fl_VsSceneState(&mut VSS);
    }
    if g & G_RULES != 0 {
        fl_GameRules(&mut GRULES);
        GRULES.mode = (rnd() % 5) as u8;
        GRULES.handicap = (rnd() % 4) as u8;
        GRULES.stage_sel = (rnd() % 6) as u8;
    }
    if g & G_FD != 0 {
        for i in 0..26 {
            fl_FighterData(&mut FD[i]);
        }
    }
    if g & G_NTD != 0 {
        for i in 0..4 {
            fl_NameTagData(&mut NTD[i]);
        }
    }
    if g & G_LBL != 0 {
        fl_lbl_8046B488_t(&mut LBL);
    }
    if g & G_TXT != 0 {
        fl_HSD_Text(&mut TXT);
    }
    if g & G_VSM != 0 {
        fl_VsModeData(&mut VSM);
        for i in 0..6 {
            VSM.start.players[i].handicap = (1 + rnd() % 9) as i8;
        }
    }
    if g & G_SSS != 0 {
        fl_SSSData(&mut SSS);
        SSS.vs.loser = (rnd() % 5) as i8 - 1;
        SSS.vs.winner = if rnd() % 3 == 0 { -1 } else { (rnd() % 4) as i8 };
        SSS.vs.ordered_stage_index = (rnd() % 31) as i8 - 1;
        for i in 0..6 {
            SSS.vs.start.players[i].slot_type = (rnd() % 5) as u8;
        }
    }
    if g & G_PID != 0 {
        for i in 0..6 {
            fl_PlayerInitData(&mut PID[i]);
        }
        fl_StartMeleeRules(&mut SMR);
    }
    if g & G_DT != 0 {
        fl_datetime(&mut DT);
    }
    if g & G_GMS != 0 {
        fl_GmStats(&mut GST);
    }
    if g & G_G677 != 0 {
        fl_gm_801677C0_s(&mut G677);
    }
    if g & G_V3 != 0 {
        fl_Vec(&mut V3A);
        fl_Vec(&mut V3B);
    }
    if g & G_MISC != 0 {
        for i in 0..4 {
            OUT4[i] = rnd() as u8;
        }
        for i in 0..4 {
            BYTES4[i] = (1 + rnd() % 9) as u8;
        }
        let lo = rnd() as u64;
        let hi = rnd() as u64;
        ITEMMASK = (hi << 32) | lo;
        MASK16 = rnd() as u16;
        for i in 0..16 {
            HANDI[i] = (1 + rnd() % 9) as i8;
        }
    }
    if g & G_GLOB != 0 {
        for i in 0..4 {
            HSD_PadMasterStatus[i].err = (rnd() % 2) as i8;
        }
        stage_info.unk8C.set_b4((rnd() & 1) as u8);
        stage_info.unk8C.set_b5((rnd() & 1) as u8);
        HSD_VIData.perf.frame_renew = rnd() as i32;
        CostumeListsForeachCharacter[Ft_Kind_Mario as usize].numCostumes = rnd() as u8;
    }
    if g & G_MEX != 0 {
        STC.reserved = rnd() as u16;
        STC.stride = rnd() as u16;
        STC.matanim = if rnd() & 1 != 0 { addr_of_mut!(MAJ) } else { null_mut() };
        MAJ.matanim = if rnd() & 1 != 0 { addr_of_mut!(MA) } else { null_mut() };
        MA.texanim = if rnd() & 1 != 0 { addr_of_mut!(TA) } else { null_mut() };
        TA.aobjdesc = if rnd() & 1 != 0 { addr_of_mut!(AD) } else { null_mut() };
        AD.end_frame = rg();
    }
    if g & G_DMD != 0 {
        DMD.anims = if rnd() & 1 != 0 { ANIMS.as_mut_ptr() } else { null_mut() };
        DMD.matanims = if rnd() & 1 != 0 { MATANIMS.as_mut_ptr() } else { null_mut() };
        DMD.shapeanims = if rnd() & 1 != 0 { SHAPEANIMS.as_mut_ptr() } else { null_mut() };
        for i in 0..4 {
            ANIMS[i] = if rnd() & 1 != 0 { 1usize as *mut HSD_AnimJoint } else { null_mut() };
            MATANIMS[i] = if rnd() & 1 != 0 { 1usize as *mut HSD_MatAnimJoint } else { null_mut() };
            SHAPEANIMS[i] = if rnd() & 1 != 0 { 1usize as *mut HSD_ShapeAnimJoint } else { null_mut() };
        }
    }
    if g & G_PERF != 0 {
        fl_lbl_8046B378_t(&mut m::lbl_8046B378);
    }
}

unsafe fn state_hash(g: u32) -> u32 {
    let mut h: u32 = 0;
    h = h.wrapping_mul(31).wrapping_add(tu_gm_1601_stubs::pool_hash());
    h = hm(h, m::lbl_804D6598 as i32 as u32);
    h = hm(h, m::lbl_804D659A as u32);
    if g & G_ME != 0 {
        h = hs_MatchEnd(&ME, h);
    }
    if g & G_ME2 != 0 {
        h = hs_MatchEnd(&ME2, h);
    }
    if g & G_VSS != 0 {
        h = hs_VsSceneState(&VSS, h);
    }
    if g & G_RULES != 0 {
        h = hs_GameRules(&GRULES, h);
    }
    if g & G_FD != 0 {
        for i in 0..26 {
            h = hs_FighterData(&FD[i], h);
        }
    }
    if g & G_NTD != 0 {
        for i in 0..4 {
            h = hs_NameTagData(&NTD[i], h);
        }
    }
    if g & G_LBL != 0 {
        h = hs_lbl_8046B488_t(&LBL, h);
    }
    if g & G_TXT != 0 {
        h = hs_HSD_Text(&TXT, h);
    }
    if g & G_VSM != 0 {
        h = hs_VsModeData(&VSM, h);
    }
    if g & G_SSS != 0 {
        h = hs_SSSData(&SSS, h);
    }
    if g & G_PID != 0 {
        for i in 0..6 {
            h = hs_PlayerInitData(&PID[i], h);
        }
        h = hs_StartMeleeRules(&SMR, h);
    }
    if g & G_DT != 0 {
        h = hs_datetime(&DT, h);
    }
    if g & G_GMS != 0 {
        h = hs_GmStats(&GST, h);
    }
    if g & G_G677 != 0 {
        h = hs_gm_801677C0_s(&G677, h);
    }
    if g & G_V3 != 0 {
        h = hs_Vec(&V3A, h);
        h = hs_Vec(&V3B, h);
    }
    if g & G_MISC != 0 {
        for i in 0..4 {
            h = hm(h, OUT4[i] as u32);
        }
        for i in 0..4 {
            h = hm(h, BYTES4[i] as u32);
        }
        h = hm(h, ITEMMASK as u32);
        h = hm(h, (ITEMMASK >> 32) as u32);
        h = hm(h, MASK16 as u32);
        for i in 0..16 {
            h = hm(h, HANDI[i] as i32 as u32);
        }
    }
    if g & G_GLOB != 0 {
        for i in 0..4 {
            h = hm(h, HSD_PadMasterStatus[i].err as i32 as u32);
        }
        h = hm(h, stage_info.unk8C.b4() as u32);
        h = hm(h, stage_info.unk8C.b5() as u32);
        h = hm(h, HSD_VIData.perf.frame_renew as u32);
        h = hm(h, CostumeListsForeachCharacter[Ft_Kind_Mario as usize].numCostumes as u32);
    }
    h = hs_lbl_8046B378_t(&m::lbl_8046B378, h);
    h
}

#[test]
fn gm_1601_matches_c() {
    tu_gm_1601_stubs::install();
    install_manual();
    unsafe {
        TOYF.trophy_id = 5;
        for (i, b) in b"toyarch\0".iter().enumerate() {
            TOYF.archive_name[i] = *b as i8;
        }
        for (i, b) in b"toysymbol\0".iter().enumerate() {
            TOYF.symbol_name[i] = *b as i8;
        }
    }
    let mut c = Check::new(include_str!("data/tu_gm_1601.txt"));
    for op in 1..=tu_gm_1601_ops::NOPS {
        let g = tu_gm_1601_ops::op_groups(op);
        for k in 0..NCASES {
            case(op, k);
            unsafe {
                tu_gm_1601_stubs::pool_reset();
                m::lbl_804D6598 = rnd() as i8;
                m::lbl_804D659A = (rnd() % 5) as u16;
                setup(g, k);
                let ret = tu_gm_1601_ops::run_op(op);
                c.line(op, k, ret, state_hash(g));
            }
        }
    }
    assert_eq!(c.done(), tu_gm_1601_ops::NOPS * NCASES);
}
