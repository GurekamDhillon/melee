//! Smoke test of tu::ftpikachuspeciallw (Thunder) through recording hooks.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftpikachuspeciallw as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> std::vec::Vec<String> {
    unsafe { core::mem::take(&mut LOG) }
}
static mut FRAMES: i32 = 0;
static mut THUNDER_POS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut BUSY: i32 = 0;

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftPikachuAttributes = Box::leak(Box::new(ftPikachuAttributes::zeroed()));
    let mut bones: std::vec::Vec<FighterBone> = (0..2).map(|_| unsafe { core::mem::zeroed() }).collect();
    f.parts = bones.as_mut_ptr();
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    let thunder = 0x1000usize as *mut Item_GObj;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, s, b, _| log(format!("ms {ms} {fl:x} {a} {s} {b}")));
        st::FTANIM_8006EBA4 = Some(|_| {});
        st::FTANIM_ISFRAMESREMAINING = Some(|_| FRAMES);
        st::FTCOMMON_8007EBAC = Some(|_, a, b| log(format!("rumble {a} {b}")));
        t::ext::CAMERA_REQUESTQUAKE = Some(|k, p| log(format!("quake {k} {}", (*p).x)));
        t::ext::EFSYNC_SPAWN = Some(|id, _, va| {
            log(format!("efsync {id} {} {}", va.len(), (*(va[0] as *mut Vec3)).y));
            core::ptr::null_mut()
        });
        t::ext::EFASYNC_SPAWN = Some(|_, _, k, gfx, _| log(format!("efasync {k} {gfx}")));
        t::ext::IT_802B1FE8 = Some(|_, out| *out = THUNDER_POS);
        t::ext::IT_802B1DEC = Some(|_| BUSY);
        t::ext::IT_802B1FC8 = Some(|_| log("strike".into()));
        t::ext::IT_802B1DF8 = Some(|_, pos, vel, c, d, k| {
            log(format!("cloud {} {} {} {c} {d} {k}", (*pos).y, (*vel).y, (*vel).x));
            0x2000usize as *mut Item_GObj
        });
    }

    // CheckProperty: Thunder property values 1, 2 and 4..=13 are true.
    for (v, want) in [(0u8, false), (1, true), (2, true), (3, false), (4, true), (13, true), (14, false), (15, false)] {
        unsafe { f.x2070.anon_0.set_x2071_b0_3(v) };
        assert_eq!(t::ftPk_SpecialLw_CheckProperty(gp), want, "{v}");
    }

    // 80127608: shake and rumble.
    f.cur_pos.x = 4.0;
    t::ftPk_SpecialLw_80127608(gp);
    assert_eq!(take(), ["quake 2 4", "rumble 11 0"]);

    // 8012765C: not armed / no cloud -> false; close enough strikes once.
    unsafe { f.mv.pk.speciallw.x4 = 0 };
    assert!(!t::ftPk_SpecialLw_8012765C(gp));
    unsafe { f.mv.pk.speciallw.x4 = 1 };
    assert!(!t::ftPk_SpecialLw_8012765C(gp));
    unsafe { f.mv.pk.speciallw.x0 = thunder };
    da.xC4 = 5.0;
    da.xBC = 2.0;
    da.xC8 = 3.0;
    f.cur_pos = Vec3 { x: 10.0, y: 10.0, z: 0.0 };
    unsafe { THUNDER_POS = Vec3 { x: 8.0, y: 12.0, z: 0.0 } };
    assert!(t::ftPk_SpecialLw_8012765C(gp));
    assert_eq!(take(), ["strike"]);
    unsafe { BUSY = 1 };
    assert!(!t::ftPk_SpecialLw_8012765C(gp));
    unsafe {
        BUSY = 0;
        THUNDER_POS.x = 20.0;
    }
    assert!(!t::ftPk_SpecialLw_8012765C(gp));
    unsafe {
        THUNDER_POS.x = 8.0;
        THUNDER_POS.y = 20.0;
    }
    assert!(!t::ftPk_SpecialLw_8012765C(gp));
    assert!(take().is_empty());

    // SpawnEffect: needs the throw flag and no cloud yet; consumes the flag.
    unsafe { f.mv.pk.speciallw.x0 = core::ptr::null_mut() };
    da.xD0 = 1.0;
    da.xCC = 2.0;
    da.xC0 = -3.0;
    da.xD4 = 4;
    da.xD8 = 5;
    da.xDC = 6;
    t::ftPk_SpecialLw_SpawnEffect(gp);
    assert!(take().is_empty());
    unsafe { f.anon_2210.anon_0.set_throw_flags_b0(1) };
    t::ftPk_SpecialLw_SpawnEffect(gp);
    assert_eq!(take(), ["efsync 1219 1 13", "cloud 11 -3 0 4 5 6"]);
    assert_eq!(unsafe { f.anon_2210.anon_0.throw_flags_b0() }, 0);
    assert_eq!(unsafe { f.mv.pk.speciallw.x0 } as usize, 0x2000);

    // Start anim end: Loop0 motion with the callbacks; throw flags cleared.
    unsafe { f.anon_2210.throw_flags = 0xff };
    unsafe { FRAMES = 1 };
    t::ftPk_SpecialLwStart_Anim(gp);
    assert!(take().is_empty());
    unsafe { FRAMES = 0 };
    t::ftPk_SpecialLwStart_Anim(gp);
    assert_eq!(take(), ["ms 360 800 0 1 0"]);
    assert_eq!(unsafe { f.anon_2210.throw_flags }, 0);
    assert!(f.take_dmg_cb.is_some() && f.accessory4_cb.is_some());
    t::ftPk_SpecialAirLwStart_Anim(gp);
    assert_eq!(take(), ["ms 364 800 0 1 0"]);

    // Loop0: state 3 or the script flag ends the loop; a strike enters Loop1 with the cloud effect.
    unsafe { f.mv.pk.specialhi.x4 = 3 };
    t::ftPk_SpecialLwLoop0_Anim(gp);
    assert_eq!(take(), ["ms 362 0 0 1 0"]);
    assert!(f.take_dmg_cb.is_none());
    t::ftPk_SpecialAirLwLoop0_Anim(gp);
    assert_eq!(take(), ["ms 366 0 0 1 0"]);
    unsafe {
        f.mv.pk.speciallw.x4 = 1;
        f.mv.pk.speciallw.x0 = thunder;
    }
    da.xB4 = 9.0;
    unsafe { THUNDER_POS = Vec3 { x: 8.0, y: 12.0, z: 0.0 } };
    t::ftPk_SpecialLwLoop0_Anim(gp);
    assert_eq!(take(), ["strike", "ms 361 0 0 1 0", "efasync 0 1216"]);
    unsafe { f.mv.pk.speciallw.x0 = thunder };
    t::ftPk_SpecialAirLwLoop0_Anim(gp);
    assert_eq!(take(), ["strike", "ms 365 0 0 1 0", "efasync 0 1216"]);
    assert_eq!(f.self_vel.y, 9.0);
}
