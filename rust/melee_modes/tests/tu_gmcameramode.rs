//! src/tu/gmcameramode.rs vs the C (gen/cref/tu/gmcameramode.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_modes/gen/cref/run_tu.py gmcameramode`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::gmcameramode as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/gmcameramode_gen.rs"]
mod g;

pub static mut ST: GameModeState = unsafe { core::mem::zeroed() };
pub static mut GM: gmm_x0 = unsafe { core::mem::zeroed() };
pub static mut PCS: PreloadedGameModeState = unsafe { core::mem::zeroed() };
pub static mut AR: [u8; 16] = [0; 16];
pub static mut KOC: [u8; 8] = [0; 8];
pub static mut EXI: i32 = 0;

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn gmcameramode_matches_c() {
    unsafe {
        g::install();
        reg(&raw const ST);
        reg(&raw const GM);
        reg(&raw const PCS);
        reg(&raw const AR);
        reg(&raw const KOC);
        reg(&raw const EXI);
        reg(&raw const m::gm_Mode_Camera_States);
        reg(&raw const m::gm_804D68C8);
        reg(&raw const m::ext::gmVsMelee_CssData);
        reg(&raw const m::ext::gmVsMelee_SssData);
        reg(&raw const m::ext::gmVsMelee_StartData);
        reg(&raw const m::ext::gmVsMelee_VsExitInfo);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_gmcameramode.txt"));
        for op in 0..11u32 {
            for k in 0..400u32 {
                case(op, k);
                ST = core::mem::zeroed();
                GM = core::mem::zeroed();
                PCS = core::mem::zeroed();
                m::ext::gmVsMelee_CssData = core::mem::zeroed();
                m::ext::gmVsMelee_SssData = core::mem::zeroed();
                m::ext::gmVsMelee_StartData = core::mem::zeroed();
                m::ext::gmVsMelee_VsExitInfo = core::mem::zeroed();
                m::ext::gmMainLib_804D3EE0 = &raw mut GM;
                for i in 0..15 {
                    g::fill_VsModeData(&mut GM.modes.table[i]);
                }
                g::fill_PreloadedGameModeState(&mut PCS);
                g::fill_CSSData(&mut m::ext::gmVsMelee_CssData);
                g::fill_SSSData(&mut m::ext::gmVsMelee_SssData);
                g::fill_StartMeleeData(&mut m::ext::gmVsMelee_StartData);
                EXI = (fv() % 4) as i32;
                match op {
                    0 => m::gm_801B23F0(),
                    1 => m::gm_801B24B4(&raw mut ST),
                    2 => {
                        ST.info.exit_data = &raw mut EXI as *mut _;
                        m::gm_801B2510(&raw mut ST)
                    }
                    3 => {
                        ST.info.enter_data = &raw mut m::ext::gmVsMelee_CssData as *mut _;
                        m::gm_801B254C(&raw mut ST)
                    }
                    4 => {
                        ST.info.exit_data = &raw mut m::ext::gmVsMelee_CssData as *mut _;
                        m::gm_801B25D4(&raw mut ST)
                    }
                    5 => {
                        ST.info.enter_data = &raw mut m::ext::gmVsMelee_SssData as *mut _;
                        m::gm_801B26AC(&raw mut ST)
                    }
                    6 => {
                        ST.info.exit_data = &raw mut m::ext::gmVsMelee_SssData as *mut _;
                        m::gm_801B2704(&raw mut ST)
                    }
                    7 => {
                        ST.info.enter_data = &raw mut m::ext::gmVsMelee_StartData as *mut _;
                        m::gm_PrepCameraModeVSScene(&raw mut ST)
                    }
                    8 => m::gm_801B2AF8(&raw mut ST),
                    9 => m::gm_Mode_Camera_OnInit(),
                    _ => {
                        for i in 0..5 {
                            g::hash_GameModeState(&m::gm_Mode_Camera_States[i]);
                        }
                    }
                }
                for i in 0..15 {
                    g::hash_VsModeData(&GM.modes.table[i]);
                }
                g::hash_PreloadedGameModeState(&PCS);
                g::hash_CSSData(&m::ext::gmVsMelee_CssData);
                g::hash_SSSData(&m::ext::gmVsMelee_SssData);
                g::hash_StartMeleeData(&m::ext::gmVsMelee_StartData);
                c.line(op, k, 0);
            }
        }
        assert_eq!(c.done(), 11 * 400);
    }
}
