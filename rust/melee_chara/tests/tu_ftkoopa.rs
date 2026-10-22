//! Smoke test for tu::ftkoopa (stateful; no float arithmetic, so no fused sites) through
//! recording hooks. One #[test]: the hooks are process globals.
#![allow(static_mut_refs, unused_unsafe)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkoopa as t;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

#[test]
fn smoke() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let fd: &'static mut ftData = Box::leak(Box::new(unsafe { core::mem::zeroed() }));
    let ea: &'static mut ftKoopaAttributes = Box::leak(Box::new(ftKoopaAttributes::zeroed()));
    let backup: &'static mut ftKoopaAttributes = Box::leak(Box::new(ftKoopaAttributes::zeroed()));
    let items: &'static mut [*mut core::ffi::c_void; 2] = Box::leak(Box::new([0x1000 as *mut _, 0x2000 as *mut _]));
    fd.ext_attr = ea as *mut _ as *mut _;
    fd.x48_items = items.as_mut_ptr() as *mut _;
    f.ft_data = fd;
    f.dat_attrs_backup = backup as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FTPARTS_80074A4C = Some(|_, a, b| log(format!("parts {a} {b}")));
        st::ITISHEAVY = Some(|_| 1);
        st::FTANIM_80070CC4 = Some(|_, fl| log(format!("CC4 {fl}")));
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("item {:x} {k}", a as usize)));
        t::ext::FTCO_800DE2A8 = Some(|_, _| log("DE2A8".into()));
        t::ext::FTCO_800DE7C0 = Some(|_, a, b| log(format!("DE7C0 {a} {b}")));
    }

    // OnLoad copies the attributes into the backup, points dat_attrs at it, registers the flame.
    ea.x0 = 3.0;
    ea.x10 = 4.0;
    ea.x18 = 5.0;
    t::ftKp_Init_OnLoad(gp);
    assert_eq!(f.dat_attrs, backup as *mut _ as *mut _);
    assert_eq!(backup.x10, 4.0);
    assert_eq!(f.x2226_b1(), 1);
    assert_eq!(take(), format!("item 1000 {}", It_Kind_Koopa_Flame));

    // OnDeath: Bowser's armor and flame meters reset from the attributes.
    t::ftKp_Init_OnDeath(gp);
    assert_eq!(f.dmg.armor0, 3.0);
    assert_eq!(unsafe { (f.u.kp.x222C, f.u.kp.x2230) }, (4.0, 5.0));
    assert_eq!(take(), "parts 0 0");

    // LoadSpecialAttrs refreshes the live copy from ext_attr.
    ea.x10 = 9.0;
    t::ftKp_Init_LoadSpecialAttrs(gp);
    assert_eq!(backup.x10, 9.0);

    // Item pickup / drop with a heavy item only run the bone-anim part.
    t::ftKp_Init_OnItemPickup(gp, 1);
    t::ftKp_Init_OnItemDrop(gp, 1);
    assert_eq!(take(), "CC4 1");

    // The thrown-victim handler: facing flips on the throw flag; the victim is handed over once.
    f.facing_dir = 1.0;
    f.victim_gobj = &mut *vg;
    f.cmd_vars[0] = 1;
    unsafe { f.anon_2210.anon_0.set_throw_flags_b4(1) };
    t::ftKp_SpecialS_80132E30(gp);
    assert_eq!(f.facing_dir, -1.0);
    assert_eq!(unsafe { f.mv.kp.unk1.xC }, 1);
    assert_eq!(unsafe { f.anon_2210.anon_0.throw_flags_b4() }, 0);
    assert_eq!(f.cmd_vars[0], 0);
    assert_eq!(take(), "DE2A8,DE7C0 0 0");
    t::ftKp_SpecialS_80132E30(gp);
    assert_eq!(take(), "");

    // The attribute getters take the fighter.
    ea.x4C = 1.5;
    backup.x4C = 1.5;
    assert_eq!(t::ftKp_SpecialS_80132DC0(&f), 1.5);
}
