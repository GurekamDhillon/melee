//! tu::ftkirbyspecials (ftkirbyspecials.c): smoke test of the Hammer Flip hat item spawn / cleanup, the
//! Coll handlers and the attribute readers through recording hooks. No float arithmetic beyond three
//! plain products (no fused sites; tools/rust-port/fma_sites.py). One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialn as n;
use melee_chara::tu::ftkirbyspecials as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();

fn log(s: String) {
    unsafe { LOG.push(s) }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut hat: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut dummy: Box<[u8; 64]> = Box::new([0; 64]);
    let mut parts: std::vec::Vec<FighterBone> = (0..100).map(|_| unsafe { core::mem::zeroed() }).collect();
    for p in parts.iter_mut() {
        p.joint = dummy.as_mut_ptr() as *mut _;
    }
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    static mut HAT: *mut HSD_GObj = core::ptr::null_mut();
    unsafe {
        HAT = &mut *hat;
        t::ext::LB_8000B1CC = Some(|_, off, out| unsafe {
            log(format!("lb {}", off.is_null()));
            (*out).x = 1.0;
            (*out).y = 2.0;
            (*out).z = 3.0;
        });
        t::ext::IT_802ADC54 = Some(|_, pos, part, ga, dir| unsafe {
            log(format!("spawn {} {} {} {} {}", (*pos).x, (*pos).y, part, ga, dir));
            HAT
        });
        t::ext::IT_802ADC34 = Some(|_| log("free".into()));
        t::ext::EFSYNC_SPAWN = Some(|id, _, a| {
            log(format!("ef {id} {}", a.len()));
            core::ptr::null_mut()
        });
        t::ext::OSREPORT = Some(|s| log(format!("report {s}")));
        t::ext::ASSERT = Some(|file, line, c| log(format!("assert {file} {line:#x} {c}")));
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCO_FALL_ENTER = Some(|_| log("fall".into()));
        st::FTCO_LANDINGFALLSPECIAL_ENTER = Some(|_, a, l| log(format!("landfs {a} {l}")));
    }
    // fn_800F53AC: cmd 1 spawns the hat item at the hand, wires the callbacks and the effect.
    f.cmd_vars[0] = 1;
    f.facing_dir = -1.0;
    f.ground_or_air = GA_Air as _;
    t::fn_800F53AC(gp);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(unsafe { f.u.kb.hat.x0 }, unsafe { HAT });
    assert_eq!(f.x1984_heldItemSpec, unsafe { HAT });
    assert!(f.death2_cb.is_some() && f.take_dmg_cb.is_some());
    assert_eq!(f.x2219_b0(), 1);
    // cmd 2 frees it again.
    f.cmd_vars[0] = 2;
    t::fn_800F53AC(gp);
    assert!(unsafe { f.u.kb.hat.x0 }.is_null() && f.cmd_vars[0] == 0);
    // a foreign death2 callback trips the assert path.
    f.cmd_vars[0] = 1;
    f.death2_cb = Some(melee_chara::chars::cb::ext__efLib_PauseAll);
    t::fn_800F53AC(gp);
    // Coll: SpecialS frees the hat and falls once ground is lost; SpecialAirS lands with lag.
    t::ftKb_SpecialS_Coll(gp);
    unsafe { f.u.kb.x64 = 1 };
    da.specials_landing_lag = 9.0;
    unsafe { f.u.kb.hat.x0 = HAT };
    t::ftKb_SpecialAirS_Coll(gp);
    assert_eq!(unsafe { f.u.kb.x64 }, 0);
    t::ftKb_SpecialAirLw_800F539C(gp);
    assert!(unsafe { f.u.kb.hat.x0 }.is_null());
    t::ftKb_SpecialAirLw_800F5524(gp);
    let got = unsafe { LOG.join(",") };
    assert_eq!(
        got,
        "lb true,spawn 1 2 44 1 -1,ef 1175 2,free,lb true,spawn 1 2 44 1 -1,report Other Dead_Proc Existence,assert ftkirbyspecials.c 0x66 0,ef 1175 2,free,fall,free,landfs 0 9"
    );

    // attribute readers.
    da.specialn_unk1 = 1.0;
    da.specialn_swallow_star_y_release = 2.0;
    da.specialn_unk2 = 3.0;
    da.specialn_unk3 = 4.0;
    da.specialn_unk4 = 5.0;
    da.specialn_unk5 = 6.0;
    let mut bx = ftCollisionBox::zeroed();
    t::ftKb_SpecialN_800F5820(gp, &mut bx, 0.5);
    assert_eq!((bx.top, bx.bottom, bx.left.x, bx.left.y, bx.right.x, bx.right.y), (0.5, 1.0, 1.5, 2.0, 2.5, 3.0));
    f.cur_pos = Vec3 { x: 7.0, y: 8.0, z: 9.0 };
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    t::ftKb_SpecialN_800F5800(gp, &mut v);
    assert_eq!((v.x, v.y, v.z), (7.0, 8.0, 9.0));
    // the Kirby ftData accessors (ftKb_KirbyFtData reads gFtDataList).
    let mut cd = Box::new(ftCommonData::zeroed());
    cd.x9C_radians = 0.25;
    cd.xA0_radians = 0.75;
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    fd.ext_attr = &mut *cd as *mut ftCommonData as *mut _;
    let mut items: [*mut core::ffi::c_void; 8] = [core::ptr::null_mut(); 8];
    items[4] = 0x1234 as *mut _;
    fd.x48_items = (items.as_mut_ptr()) as *mut _;
    unsafe { melee_fighter::tu::ftdata::gFtDataList[Ft_Kind_Kirby as usize] = &mut *fd };
    f.ft_data = &mut *fd;
    let mut out = Vec2 { x: 0.0, y: 0.0 };
    t::ftKb_SpecialN_800F5874(&mut out);
    assert_eq!((out.x, out.y), (0.25, 0.75));
    assert_eq!(t::ftKb_SpecialN_800F5898(gp) as usize, 0x1234);
    assert_eq!(n::ftKb_KirbyFtData(), &mut *fd as *mut ftData);
    unsafe { melee_fighter::tu::ftdata::gFtDataList[Ft_Kind_Kirby as usize] = core::ptr::null_mut() };
    drop(parts);
}
