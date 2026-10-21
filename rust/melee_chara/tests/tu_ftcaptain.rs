//! Smoke test for tu::ftcaptain: OnLoad / OnLoadForGanon (PUSH_ATTRS) and LoadSpecialAttrs
//! (COPY_ATTRS) move ftCaptain_DatAttrs between ext_attr, dat_attrs and the backup.
use melee_chara::tu::ftcaptain as t;
use melee_types::*;

#[test]
fn attrs_push_and_copy() {
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut ext = Box::new(ftCaptain_DatAttrs::zeroed());
    let mut backup = Box::new(ftCaptain_DatAttrs::zeroed());
    let mut live = Box::new(ftCaptain_DatAttrs::zeroed());
    ext.specialn_vel_x = 3.5;
    ext.specials_gr_vel_x = -1.25;
    fd.ext_attr = &mut *ext as *mut _ as *mut _;
    f.ft_data = &mut *fd;
    f.dat_attrs_backup = &mut *backup as *mut _ as *mut _;
    f.dat_attrs = &mut *live as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp = &mut *g as *mut HSD_GObj;

    t::ftCa_Init_LoadSpecialAttrs(gp);
    assert_eq!(live.specialn_vel_x, 3.5);
    assert_eq!(live.specials_gr_vel_x, -1.25);

    t::ftCa_Init_OnLoad(gp);
    assert_eq!(f.can_walljump(), 1);
    assert_eq!(f.dat_attrs, &mut *backup as *mut _ as *mut _);
    assert_eq!(backup.specialn_vel_x, 3.5);
    assert_eq!(t::ftCa_Init_CostumeStrings[2][0], "PlCaRe.");
}
