//! src/tu/gmevent.rs vs the C (gen/cref/tu/gmevent.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/mk_gmevent_driver.py` then
//! `python rust/melee_modes/gen/cref/run_tu.py gmevent`).
#![allow(static_mut_refs, non_snake_case, unused_unsafe, unused_assignments)]
mod mtu_harness;
use melee_modes::tu::gmevent as m;
use melee_modes::tu::gmevent::*;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmevent_gen.rs"]
mod g;

const NLV: usize = 51;
pub static mut ST: GameModeState = unsafe { core::mem::zeroed() };
pub static mut GM: gmm_x0 = unsafe { core::mem::zeroed() };
pub static mut PCS: PreloadedGameModeState = unsafe { core::mem::zeroed() };
pub static mut VSC: VsSceneController = unsafe { core::mem::zeroed() };
pub static mut VSS: VsSceneState = unsafe { core::mem::zeroed() };
pub static mut MEND: MatchEnd = unsafe { core::mem::zeroed() };
pub static mut GNEW: HSD_GObj = unsafe { core::mem::zeroed() };
pub static mut GOA: [HSD_GObj; 4] = unsafe { core::mem::zeroed() };
pub static mut G1: HSD_GObj = unsafe { core::mem::zeroed() };
pub static mut PID: PlayerInitData = unsafe { core::mem::zeroed() };
pub static mut SRC: gm_801BAB40_src = unsafe { core::mem::zeroed() };
static mut LVP: [*mut gm_804D6900_t; NLV] = [core::ptr::null_mut(); NLV];
static mut LV: [gm_804D6900_t; NLV] = unsafe { core::mem::zeroed() };
static mut EVI: [gm_evinit; NLV] = unsafe { core::mem::zeroed() };
static mut EVB: [gm_evbonus; NLV] = unsafe { core::mem::zeroed() };
static mut EVS: [gm_evstage_table; NLV] = unsafe { core::mem::zeroed() };
static mut PIN: [[gm_801BAB40_src; 6]; NLV] = unsafe { core::mem::zeroed() };
static mut X4: [gm_804D6900_x4_t; NLV] = unsafe { core::mem::zeroed() };
static mut X4SRC: [gm_801BAB40_src; NLV] = unsafe { core::mem::zeroed() };
pub static mut CHL: [u8; 64] = [0; 64];
const SPECIAL: [u32; 16] = [4, 9, 12, 15, 19, 21, 23, 27, 29, 35, 39, 43, 44, 48, 36, 49];

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

unsafe fn zero<T>(p: *mut T) {
    core::ptr::write_bytes(p, 0, 1)
}

unsafe fn build_levels(charlist: bool, nulls: bool, ptrmode: bool) {
    for l in 0..NLV {
        g::fill_gm_804D6900_t(&mut LV[l]);
        LV[l].kind = (fv() % 3) as u8;
        LV[l].flags = ((fv() & 0x1F) | ((fv() % 6) << 5)) as u8;
        g::fill_gm_evinit(&mut EVI[l]);
        g::fill_gm_evbonus(&mut EVB[l]);
        g::fill_gm_evstage_table(&mut EVS[l]);
        EVS[l].count = (1 + fv() % 6) as u8;
        for j in 0..6 {
            g::fill_gm_801BAB40_src(&mut PIN[l][j]);
            EVS[l].entries[j] = &raw mut PIN[l][j];
        }
        if PIN[l][4].c_kind as i32 == ChKind_None as i32 {
            PIN[l][4].c_kind = 5;
        }
        LV[l].evinit = &raw mut EVI[l];
        LV[l].evbonus = &raw mut EVB[l];
        LV[l].evstage_table = &raw mut EVS[l];
        for j in 0..5 {
            LV[l].player_init[j] =
                if j == 0 || j == 4 || !nulls || (fv() & 3) != 0 { &raw mut PIN[l][j] } else { core::ptr::null_mut() };
        }
        X4[l].x0 = (fv() % 400) as i32;
        g::fill_gm_801BAB40_src(&mut X4SRC[l]);
        X4[l].x4 = if ptrmode { &raw mut X4SRC[l] as isize } else { fv() as i32 as isize };
        LV[l].x4 = if charlist { &raw mut CHL as *mut gm_804D6900_x4_t } else { &raw mut X4[l] };
        LVP[l] = &raw mut LV[l];
    }
    m::gm_804D6900[0] = &raw mut LVP as *mut *mut gm_804D6900_t;
}

unsafe fn hash_all() {
    if TRACEON { g::hash_StartMeleeRules(&m::vs_enter_data.rules); println!("H rules {:08X}", H); for q in 0..6 { g::hash_PlayerInitData(&m::vs_enter_data.players[q]); println!("H player{} {:08X}", q, H); } }
    if TRACEON {
        let e = &GM.vs.unk_530;
        println!("H ev x10 {} x18 {} x20 {} x24 {} x28 {} x2C {} x30 {} x34 {} x38 {} x3C {} x40 {} bits {}{}{}{}{}{}{}{}", e.x10, e.x18, e.x20, e.x24, e.x28, e.x2C, e.x30, e.x34, e.x38, e.x3C, e.x40, e.xB_0(), e.xB_1(), e.xB_2(), e.xB_3(), e.xB_4(), e.xB_5(), e.xB_6(), e.xB_7());
    }
    g::hash_EventData(&GM.vs.unk_530); if TRACEON { println!("H EventData {:08X}", H); }
    g::hash_CSSData(&m::css_data); if TRACEON { println!("H CSSData {:08X}", H); }
    g::hash_StartMeleeData(&m::vs_enter_data); if TRACEON { println!("H StartMeleeData {:08X}", H); }
    g::hash_MatchExitInfo(&m::vs_exit_data[0]); if TRACEON { println!("H MatchExitInfo {:08X}", H); }
    g::hash_MatchExitInfo(&m::vs_exit_data[1]); if TRACEON { println!("H MatchExitInfo {:08X}", H); }
    g::hash_PreloadedGameModeState(&PCS); if TRACEON { println!("H PreloadedGameModeState {:08X}", H); }
    g::hash_VsSceneController(&VSC); if TRACEON { println!("H VsSceneController {:08X}", H); }
    hm(VSS.timer_seconds);
    hm(VSS.unk_2C as u32);
    g::hash_MatchEnd(&MEND); if TRACEON { println!("H MatchEnd {:08X}", H); }
    g::hash_PlayerInitData(&PID); if TRACEON { println!("H PlayerInitData {:08X}", H); }
    g::hash_gm_801BAB40_src(&SRC); if TRACEON { println!("H gm_801BAB40_src {:08X}", H); }
}

#[test]
fn gmevent_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const GM);
        reg(&raw const PCS);
        reg(&raw const VSC);
        reg(&raw const VSS);
        reg(&raw const MEND);
        reg(&raw const GNEW);
        reg(&raw const GOA);
        reg(&raw const G1);
        reg(&raw const PID);
        reg(&raw const SRC);
        reg(&raw const m::css_data);
        reg(&raw const m::vs_enter_data);
        reg(&raw const m::vs_exit_data);
        reg(&raw const m::gm_804D6900);
        reg(&raw const m::gm_Mode_Event_States);
        reg(&raw const CHL);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmevent.txt"));
        for op in 0..62_u32 {
            for k in 0..300_u32 {
                case(op, k);
                let mut ret = 0u32;
                zero(&raw mut ST);
                zero(&raw mut GM);
                m::ext::gmMainLib_804D3EE0 = &raw mut GM;
                g::fill_EventData(&mut GM.vs.unk_530);
                GM.vs.unk_530.unk_535 =
                    if (fv() & 3) == 0 { SPECIAL[(fv() % 16) as usize] as u8 } else { (fv() % 51) as u8 };
                GM.vs.unk_530.x20 = (fv() % 6) as i32;
                g::fill_CSSData(&mut m::css_data);
                g::fill_StartMeleeData(&mut m::vs_enter_data);
                g::fill_MatchExitInfo(&mut m::vs_exit_data[0]);
                g::fill_MatchExitInfo(&mut m::vs_exit_data[1]);
                g::fill_PreloadedGameModeState(&mut PCS);
                g::fill_VsSceneController(&mut VSC);
                VSS.timer_seconds = fv();
                VSS.unk_2C = fv() as u16;
                g::fill_MatchEnd(&mut MEND);
                g::fill_PlayerInitData(&mut PID);
                g::fill_gm_801BAB40_src(&mut SRC);
                for i in 0..20 {
                    CHL[i] = (fv() % 25) as u8;
                }
                CHL[20] = 33;
                ST.id = fv() as u8;
                build_levels(op == 5 || op == 6, op == 6 || op == 7 || op == 9 || op == 10, op == 7 || op == 9 || op == 10 || op == 12 || op == 13 || op == 37);
                match op {

                    0 => { m::gm_801BA8FC(); }
                    1 => { { let lo = (fv() % 4) as i32; let hi = lo + (fv() % (5 - lo) as u32) as i32; let a3 = (fv() & 1) as i32; m::gm_801BA938(&raw mut GM.vs.unk_530, lo, hi, a3); } }
                    2 => { ST.info.enter_data = &raw mut m::css_data as *mut _; m::onEnterCss(&raw mut ST); }
                    3 => { ST.info.exit_data = &raw mut m::css_data as *mut _; m::onExitCss(&raw mut ST); }
                    4 => { m::gm_801BAB40(&raw mut PID, &raw mut SRC); }
                    5 => { { let n = (fv() % 6) as i32; ST.info.enter_data = &raw mut m::vs_enter_data as *mut _; ret = m::gm_801BAC9C(&raw mut ST, n) as u32; } }
                    6 => { ST.info.enter_data = &raw mut m::vs_enter_data as *mut _; m::onEnterVs(&raw mut ST); }
                    7 => { ST.info.exit_data = &raw mut m::vs_exit_data[1] as *mut _; m::onExitVs(&raw mut ST); }
                    8 => { m::gm_Mode_Event_OnInit(); }
                    9 => { m::gm_801BBB64(); }
                    10 => { m::gm_Mode_Event_OnLoad(); }
                    11 => { m::gm_Mode_Event_OnUnload(); }
                    12 => { m::fn_801BBFE8(); }
                    13 => { m::gm_801BC00C(); }
                    14 => { ret = m::gm_801BC488() as u32; }
                    15 => { m::gm_801BC4F4(&raw mut G1); }
                    16 => { m::gm_801BC670(&raw mut G1); }
                    17 => { m::gm_801BC754(&raw mut G1); }
                    18 => { m::gm_801BC9E8(&raw mut G1); }
                    19 => { m::gm_801BCAF0(&raw mut G1); }
                    20 => { m::gm_801BCC9C(&raw mut G1); }
                    21 => { m::gm_801BCF20(&raw mut G1); }
                    22 => { m::gm_801BCF40(&raw mut G1); }
                    23 => { m::gm_801BD028(&raw mut G1); }
                    24 => { m::gm_801BD164(&raw mut G1); }
                    25 => { m::gm_801BD30C(&raw mut G1); }
                    26 => { m::gm_801BD44C(&raw mut G1); }
                    27 => { m::gm_801BD46C(&raw mut G1); }
                    28 => { m::gm_801BD658(&raw mut G1); }
                    29 => { m::gm_801BD7FC(&raw mut G1); }
                    30 => { m::gm_801BD93C(&raw mut G1); }
                    31 => { m::gm_801BDAD4(&raw mut G1); }
                    32 => { m::gm_801BDAF4(&raw mut G1); }
                    33 => { m::gm_801BDC08(&raw mut G1); }
                    34 => { m::gm_801BDD44(&raw mut G1); }
                    35 => { m::gm_801BDE94(&raw mut G1); }
                    36 => { m::gm_801BE37C(&raw mut G1); }
                    37 => { m::gm_801BE39C(&raw mut G1); }
                    38 => { m::gm_801BE618(&raw mut G1); }
                    39 => { m::gm_801BE638(&raw mut G1); }
                    40 => { m::failEvent(&raw mut G1); }
                    41 => { ret = pi(m::gm_GetEventData()); }
                    42 => { ret = m::gm_GetNextColor(fv() as u8) as u32; }
                    43 => { m::gm_801BBB64_inline(&raw mut GM.vs.unk_530); }
                    44 => { m::gm_801BC00C_inline(&raw mut SRC); }
                    45 => { ret = m::gm_801BC00C_GetCharacter(&raw mut SRC) as i32 as u32; }
                    46 => { ret = m::gm_801BC00C_GetCharacterKind(&raw mut SRC); }
                    47 => { m::gm_801BEA10(fv() as i32); }
                    48 => { m::gm_801BEA4C(fv() as i32); }
                    49 => { { let a = fv() as i32; let b = (fv() % 3) as i32; m::gm_801BEA88(a, b); } }
                    50 => { m::gm_801BEAF0(fv() as i32); }
                    51 => { m::gm_801BEB2C(fv() as i32); }
                    52 => { m::gm_801BEB68(fv() as i32); }
                    53 => { m::gm_801BEB74(fv() as u8); }
                    54 => { ret = m::gm_801BEB80() as u32; }
                    55 => { ret = m::gm_801BEB8C((fv() % 51) as u8) as u32; }
                    56 => { ret = m::gm_801BEBA8((fv() % 51) as u8) as u32; }
                    57 => { ret = m::gm_801BEBC0((fv() % 56) as u8) as u32; }
                    58 => { ret = m::gm_801BEBF8(m::gm_803DF918[(fv() % 51) as usize]) as u32; }
                    59 => { ret = pi(m::gm_801BEC54()); }
                    60 => { { for i in 0..3 { g::hash_GameModeState(&m::gm_Mode_Event_States[i]); } } }
                    61 => { { for i in 0..51 { hm(fi((*m::gm_803DF94C[i]).x0)); hm(fi((*m::gm_803DF94C[i]).x4)); } } }
                    _ => unreachable!(),
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 62 * 300);
    }
}
