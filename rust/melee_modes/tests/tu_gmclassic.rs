//! src/tu/gmclassic.rs vs the C (gen/cref/tu/gmclassic.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmclassic`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmclassic as m;
use melee_modes::tu::gmclassic::gmClassicMatchup;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmclassic_gen.rs"]
mod g;

pub static mut ST: GameModeState = unsafe { core::mem::zeroed() };
pub static mut AD: UnkAllstarData = unsafe { core::mem::zeroed() };
pub static mut G528: gmm_x0_528_t = unsafe { core::mem::zeroed() };
pub static mut PCS: PreloadedGameModeState = unsafe { core::mem::zeroed() };
pub static mut CSS: CSSData = unsafe { core::mem::zeroed() };
pub static mut SMD: StartMeleeData = unsafe { core::mem::zeroed() };
pub static mut MEI: MatchExitInfo = unsafe { core::mem::zeroed() };
pub static mut R98: gmm_retval_ED98 = unsafe { core::mem::zeroed() };
pub static mut RB0: gmm_retval_EDB0 = unsafe { core::mem::zeroed() };
pub static mut RBC: gmm_retval_EDBC = unsafe { core::mem::zeroed() };
pub static mut TIMEV: u32 = 0;
pub static mut BESTV: i32 = 0;
pub static mut BUF: [u8; 0x200] = [0; 0x200];
pub static mut UNL: bool = false;

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

unsafe fn hash_all() {
    g::hash_UnkAllstarData(&AD);
    g::hash_gmm_x0_528_t(&G528);
    g::hash_PreloadedGameModeState(&PCS);
    g::hash_CSSData(&CSS);
    g::hash_StartMeleeData(&SMD);
    g::hash_MatchExitInfo(&MEI);
    g::hash_DebugGameOverData(&m::ext::gmClassic_80470850);
    g::hash_gmClassicRuntimeData(&m::gmClassicRuntimeBuf);
    g::hash_gmClassic_803DDEC8Data(&m::gmClassic_803DDEC8);
    g::hash_gmm_retval_ED98(&R98);
    g::hash_gmm_retval_EDB0(&RB0);
    g::hash_gmm_retval_EDBC(&RBC);
    hm(TIMEV);
    hm(BESTV as u32);
    for i in 0..0x200 {
        hm(BUF[i] as u32);
    }
    hm(pi(m::gmClassic_804D68D0));
}

/// a valid shuffled order in the four order tables, as OnLoad leaves them
unsafe fn make_orders() {
    let rt = &raw mut m::gmClassicRuntimeBuf as *mut u8;
    let t = &raw mut m::gmClassic_803DDEC8;
    m::gmClassic_InitMatchupOrder((&raw mut (*t).x2B0) as *const gmClassicMatchup, m::gm_804908A0().add(0x60), rt, 0x80);
    m::gmClassic_InitMatchupOrder((&raw mut (*t).x26C) as *const gmClassicMatchup, m::gm_804908A0().add(0x54), rt, 0x74);
    m::gmClassic_InitMatchupOrder((&raw mut (*t).x1B8) as *const gmClassicMatchup, m::gm_804908A0().add(0x34), rt, 0x54);
    m::gmClassic_InitMatchupOrder((&raw mut (*t).x0CC) as *const gmClassicMatchup, m::gm_804908A0().add(0x0C), rt, 0x2C);
}

unsafe fn pool_ptr(pool: u32) -> *mut gmClassicMatchup {
    let t = &raw mut m::gmClassic_803DDEC8;
    match pool {
        0 => (&raw mut (*t).x0CC) as *mut gmClassicMatchup,
        1 => (&raw mut (*t).x1B8) as *mut gmClassicMatchup,
        2 => (&raw mut (*t).x26C) as *mut gmClassicMatchup,
        3 => (&raw mut (*t).x2B0) as *mut gmClassicMatchup,
        _ => (&raw mut (*t).x0C0) as *mut gmClassicMatchup,
    }
}

#[test]
fn gmclassic_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const AD);
        reg(&raw const G528);
        reg(&raw const PCS);
        reg(&raw const CSS);
        reg(&raw const SMD);
        reg(&raw const MEI);
        reg(&raw const R98);
        reg(&raw const RB0);
        reg(&raw const RBC);
        reg(&raw const TIMEV);
        reg(&raw const BESTV);
        reg(&raw const BUF);
        reg(&raw const m::gmClassicRuntimeBuf);
        reg(&raw const m::gmClassic_803DDEC8);
        reg(&raw const m::gm_804D4318);
        reg(&raw const m::gm_804D4320);
        reg(&raw const m::gm_804D4328);
        reg(&raw const m::gmClassic_804D68D0);
        reg(&raw const m::gm_Mode_Classic_States);
        reg(&raw const m::ext::gmClassic_80470708);
        reg(&raw const m::ext::gmClassic_80470850);
        reg(&raw const m::ext::gmClassic_8047086C);
        reg(&raw const m::ext::gmClassic_80472AF8);
        g::freg_all();
        let init = m::gmClassic_803DDEC8;
        let mut c = Check::new(include_str!("data/mtu_gmclassic.txt"));
        for op in 0..17u32 {
            for k in 0..500u32 {
                case(op, k);
                ST = core::mem::zeroed();
                m::gmClassic_803DDEC8 = init;
                g::fill_UnkAllstarData(&mut AD);
                g::fill_gmm_x0_528_t(&mut G528);
                g::fill_PreloadedGameModeState(&mut PCS);
                g::fill_CSSData(&mut CSS);
                g::fill_StartMeleeData(&mut SMD);
                g::fill_MatchExitInfo(&mut MEI);
                g::fill_DebugGameOverData(&mut m::ext::gmClassic_80470850);
                g::fill_gmClassicRuntimeData(&mut m::gmClassicRuntimeBuf);
                g::fill_gmm_retval_ED98(&mut R98);
                g::fill_gmm_retval_EDB0(&mut RB0);
                g::fill_gmm_retval_EDBC(&mut RBC);
                TIMEV = fv();
                BESTV = fv() as i32;
                for i in 0..0x200 {
                    BUF[i] = fv() as u8;
                }
                m::gmClassic_804D68D0 = core::ptr::null_mut();
                UNL = op == 6 || op == 8 || op == 10 || op == 13;
                ST.id = fv() as u8;
                let t = &raw mut m::gmClassic_803DDEC8;
                for i in 0..11 {
                    (*t).x00[i].xC = (&raw mut (*t).x0CC[(fv() % 39) as usize]) as *mut _;
                }
                let mut ret = 0u32;
                let st = &raw mut ST;
                match op {
                    0 => {
                        ST.info.enter_data = &raw mut m::gmClassicRuntimeBuf.intro as *mut _;
                        m::gmClassic_801B3500(st)
                    }
                    1 => {
                        ST.info.enter_data = &raw mut SMD as *mut _;
                        m::gmClassic_801B3A34(st)
                    }
                    2 => {
                        ST.info.exit_data = &raw mut MEI as *mut _;
                        m::gmClassic_801B3B40(st)
                    }
                    3 => {
                        ST.info.enter_data = &raw mut m::ext::gmClassic_80470850 as *mut _;
                        m::gmClassic_801B3D44(st)
                    }
                    4 => {
                        ST.info.exit_data = &raw mut m::ext::gmClassic_80470850 as *mut _;
                        m::gmClassic_801B3D84(st)
                    }
                    5 => {
                        ST.info.enter_data = &raw mut CSS as *mut _;
                        m::gmClassic_801B3DD8(st)
                    }
                    6 => {
                        m::gmClassic_803DDEC8 = init;
                        make_orders();
                        ST.info.exit_data = &raw mut CSS as *mut _;
                        m::gmClassic_801B3E44(st)
                    }
                    7 => m::gmClassic_801B3F18(st),
                    8 => m::gm_Mode_Classic_OnLoad(),
                    9 => m::gm_Mode_Classic_OnInit(),
                    10 => {
                        m::gmClassic_803DDEC8 = init;
                        make_orders();
                        ret = pi(m::gmClassic_801B2D54((&raw mut (*t).x00) as *mut gm_803DDEC8Struct));
                    }
                    11 => {
                        let pool = fv() % 5;
                        let tbl = pool_ptr(pool);
                        let mut order = [0u8; 64];
                        for i in 0..64usize {
                            order[i] = (fv() % 2 + (i as u32) % 9) as u8;
                        }
                        for i in 0..11 {
                            (*t).x00[i].xC = if fv() & 1 != 0 {
                                core::ptr::null_mut()
                            } else {
                                (&raw mut (*t).x0CC[(fv() % 39) as usize]) as *mut _
                            };
                        }
                        ret = pi(m::gmClassic_801B2BA4(tbl, order.as_ptr(), (&raw mut (*t).x00) as *mut gm_803DDEC8Struct));
                    }
                    12 => {
                        let pool = fv() % 4;
                        let tbl = pool_ptr(pool);
                        let off = (fv() % 0x80) as i32;
                        m::gmClassic_InitMatchupOrder(tbl, (&raw mut BUF as *mut u8).add(0x10), (&raw mut BUF as *mut u8).add(0x40), off);
                    }
                    13 => ret = m::gm_OnePSceneDirect(&raw mut G528, &raw mut AD.x0, fv() as i32) as u32,
                    14 => {
                        let pool = fv() % 5;
                        let n = match pool {
                            0 => m::gmClassic_GetMatchupCount(&(*t).x0CC),
                            1 => m::gmClassic_GetMatchupCount(&(*t).x1B8),
                            2 => m::gmClassic_GetMatchupCount(&(*t).x26C),
                            3 => m::gmClassic_GetMatchupCount(&(*t).x2B0),
                            _ => m::gmClassic_GetMatchupCount(&(*t).x0C0),
                        };
                        ret = n as u32;
                    }
                    15 => {
                        let e = (&raw mut (*t).x00[(fv() % 11) as usize]) as *mut gm_803DDEC8Struct;
                        ret = m::gmClassic_GetStKind(e, &raw mut AD) as u32;
                    }
                    _ => {
                        for i in 0..26 {
                            g::hash_GameModeState(&m::gm_Mode_Classic_States[i]);
                        }
                    }
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 17 * 500);
    }
}
