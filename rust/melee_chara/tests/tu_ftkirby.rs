//! tu::ftkirby (src/melee/ft/kinds/ftKirby/ftkirby.c): no fused sites and no pure math. Smoke tests of
//! the init callbacks, the costume-row mapping, the hat tables, the hat load/unload paths (simple
//! one-JObj hats and the multi-part DObj instantiation) and the m-ex hooks, through recording hooks.
#![allow(static_mut_refs)]

use core::ffi::c_void;
use core::ptr::{null, null_mut};
use melee_chara::stubs as st;
use melee_chara::tu::ftkirby as t;
use melee_chara::tu::ftkirbydata as d;
use melee_types::*;

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
fn log(s: String) {
    unsafe { LOG.push(s) }
}
/// The hooks and the log are process-wide statics: tests take this lock.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn take() -> String {
    unsafe { core::mem::take(&mut LOG).join(",") }
}

struct World {
    f: Box<Fighter>,
    g: Box<HSD_GObj>,
    parts: std::vec::Vec<FighterBone>,
    da: &'static mut ftKb_DatAttrs,
}
impl World {
    fn new() -> World {
        let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
        let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
        let mut parts: std::vec::Vec<FighterBone> = (0..16).map(|_| unsafe { core::mem::zeroed() }).collect();
        f.dat_attrs = da as *mut _ as *mut _;
        f.parts = parts.as_mut_ptr();
        g.user_data = &mut *f as *mut Fighter as *mut _;
        World { f, g, parts, da }
    }
    fn gp(&mut self) -> *mut HSD_GObj {
        &mut *self.g
    }
}

fn install_common_hooks() {
    unsafe {
        t::ext::SKINS_KIRBYROW = Some(|_, c| if c == 9 { 4 } else { -1 });
        t::ext::MEX_COSTUMEVISIDX = Some(|_, c| c);
        t::ext::HSD_OBJALLOC = Some(|_| {
            log("alloc".into());
            Box::leak(Box::new([null_mut::<HSD_DObj>(); 32])).as_mut_ptr() as *mut c_void
        });
        t::ext::HSD_OBJFREE = Some(|_, p| log(format!("free {}", !p.is_null())));
        t::ext::FIGHTER_X2040_ALLOC_DATA = Some(|| null_mut());
        t::ext::FTPARTSPOBJSETDEFAULTCLASS = Some(|| log("setclass".into()));
        t::ext::FTPARTSPOBJCLEARDEFAULTCLASS = Some(|| log("clearclass".into()));
        t::ext::HSD_JOBJLOADJOINT = Some(|j| {
            log("loadjoint".into());
            Box::leak(Box::new(unsafe { core::mem::zeroed::<HSD_JObj>() })) as *mut HSD_JObj
        });
        t::ext::FTPARTS_80075650 = Some(|_, _, _| log("75650".into()));
        t::ext::FTPARTS_8007487C = Some(|_, _, a, _, _| log(format!("7487C {a}")));
        t::ext::HSD_JOBJREMOVEALL = Some(|_| log("jobjremoveall".into()));
    }
}

#[test]
fn costume_row_and_tables() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    // Skin row wins in 0..6, then the visibility index, else 0.
    assert_eq!(t::ftKb_CopyCostumeRow(9), 4);
    assert_eq!(t::ftKb_CopyCostumeRow(3), 3);
    assert_eq!(t::ftKb_CopyCostumeRow(7), 0);
    assert_eq!(t::ftKb_CopyCostumeRow(-1), 0);
    unsafe {
        // Hat tables: the loader/unloader pairs are present for the 33 retail kinds, empty for m-ex kinds.
        for k in 0..33usize {
            assert_eq!(t::ftKb_Init_803C9DD0[k].is_some(), !matches!(k, 11 | 27..=30 | 32), "9DD0[{k}]");
            assert_eq!(t::ftKb_Init_803C9E54[k].is_some(), t::ftKb_Init_803C9DD0[k].is_some(), "9E54[{k}]");
        }
        assert!(t::ftKb_Init_803C9CC8[0].is_some() && t::ftKb_Init_803C9CC8[1].is_some());
        assert!(t::ftKb_Init_803C9CC8[22].is_none() && t::ftKb_Init_803C9CC8[23].is_none());
        for k in 33..t::ftKb_Init_803C9DD0.len() {
            assert!(t::ftKb_Init_803C9DD0[k].is_none() && t::ftKb_Init_803C9E54[k].is_none());
        }
        assert!(t::ftKb_Init_803C9FC8[3] == &raw mut t::ftKb_Init_803C9ED8 as *mut _);
        assert!(t::ftKb_Init_803C9FC8[24] == &raw mut t::ftKb_Init_803C9F98 as *mut _);
        assert_eq!(t::ftKb_Init_803C9FC8.iter().filter(|p| !p.is_null()).count(), 5);
        assert_eq!(t::ftKb_Init_803B7548, [0.0; 10]);
    }
    // ftKirby_CostumeArchive carries its GC layout (size 8, joint @0, matanim @4).
    use melee_core::gc::GcLayout;
    assert_eq!(t::ftKirby_CostumeArchive::GC_SIZE, 8);
    assert_eq!(t::ftKirby_CostumeArchive::gc_field("matanim").unwrap().offset, 4);
}

#[test]
fn init_callbacks() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();
    unsafe {
        t::ext::PLAYER_GETFLAGSAEBIT1 = Some(|_| 1);
        t::ext::PLAYER_GETFLAGSBIT1 = Some(|_| 1);
        t::ext::PLAYER_GETUNK4D = Some(|_| 1);
        t::ext::HSD_RANDI = Some(|n| n - 1);
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("art {} {k}", a as usize)));
        st::FTPARTS_80074A4C = Some(|_, m, v| log(format!("74A4C {m} {v}")));
        // ftKb_SpecialN_800F1BAC (reached from OnDeath): hat reload on demand, change, sfx.
        t::ext::MEX_COSTUMEVISIDX = Some(|_, c| c);
        t::ext::LBDVD_800178E8 = Some(|_, _, _, _, _, _, _, _, _| {});
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx {id:x} {v} {p}")));
    }
    // OnLoad: PUSH_ATTRS, can_multijump, hat flag, four articles from ft_data.x48_items.
    let mut ext_attr = Box::new(ftKb_DatAttrs::zeroed());
    ext_attr.specialn_dk_swings_to_full_charge = 11;
    let mut backup = Box::new(ftKb_DatAttrs::zeroed());
    let mut fd: Box<ftData> = Box::new(unsafe { core::mem::zeroed() });
    let mut items: [*mut c_void; 4] = [10usize as *mut c_void, 20usize as *mut c_void, 30usize as *mut c_void, 40usize as *mut c_void];
    fd.ext_attr = &mut *ext_attr as *mut _ as *mut _;
    fd.x48_items = items.as_mut_ptr();
    w.f.ft_data = &mut *fd;
    w.f.dat_attrs_backup = &mut *backup as *mut _ as *mut _;
    t::ftKb_Init_LoadSpecialAttrs(gp);
    assert_eq!(w.da.specialn_dk_swings_to_full_charge, 11);
    w.da.specialn_dk_swings_to_full_charge = 0;
    t::ftKb_Init_OnLoad(gp);
    assert_eq!(w.f.can_multijump(), 1);
    assert_eq!(w.f.dat_attrs, &mut *backup as *mut _ as *mut _);
    assert_eq!(backup.specialn_dk_swings_to_full_charge, 11);
    assert_eq!(unsafe { w.f.u.kb.hat.x8_b0() }, 1);
    assert_eq!(take(), format!("art 10 {},art 20 {},art 30 {},art 40 {}", It_Kind_Kirby_CBeam, It_Kind_Kirby_Hammer, It_Kind_Unk1, It_Kind_Unk2));

    // OnDeath: hat cleared to Kirby, counters reset, then the kept hat (kind 1, Fox) is put back on.
    w.f.dat_attrs = w.da as *mut _ as *mut _;
    w.f.x619_costume_id = 2;
    unsafe {
        w.f.u.kb.hat.kind = Ft_Kind_Mario;
        w.f.u.kb.x60 = 9;
        w.f.u.kb.x64 = 9;
    }
    // hat slot 1 (ft_80459B88) is empty: the archive loader runs for it (800EED50 -> lbArchive hook).
    unsafe {
        t::ext::EFASYNC_LOADSYNC = Some(|id| log(format!("loadsync {id}")));
        t::ext::LBARCHIVE_80017040 = Some(|_, name, out, rest| unsafe {
            log(format!("arch {} {}", core::ffi::CStr::from_ptr(name as *const _).to_str().unwrap(), rest.len()));
            *(out as *mut *mut c_void) = Box::leak(Box::new(KirbyHatStruct::zeroed())) as *mut KirbyHatStruct as *mut c_void;
        });
        t::ft_80459B88 = core::mem::zeroed();
    }
    t::ftKb_Init_OnDeath(gp);
    let got = take();
    assert_eq!(
        got,
        format!(
            "74A4C 0 0,74A4C 1 0,arch PlKbCpFx.dat 1,loadsync {},alloc,setclass,loadjoint,clearclass,75650,7487C 0,art 0 {},art 0 {}",
            unsafe { d::ftKb_Init_803CB46C[1] },
            It_Kind_Kirby_FoxLaser,
            It_Kind_Kirby_FoxBlaster
        )
    );
    unsafe {
        assert_eq!(w.f.u.kb.hat.kind, 1);
        assert_eq!(w.f.u.kb.hat.x4, 5);
        assert!(!w.f.u.kb.hat.jobj.is_null() && !w.f.u.kb.hat.x14.data.is_null()); // the Fox hat was loaded
        assert_eq!((w.f.u.kb.x60, w.f.u.kb.x64), (0, 0));
        // 800F190C reset the copy-ability vars; Koopa fuel / flame come from the attrs.
        assert_eq!((w.f.u.kb.x68, w.f.u.kb.x6C, w.f.u.kb.xD4, w.f.u.kb.xD8), (9, 9, 1, 3));
        assert_eq!((w.f.u.kb.x8C.x, w.f.u.kb.x8C.y, w.f.u.kb.x8C.z), (1.0, 1.0, 1.0));
    }
    assert!(w.f.death3_cb.is_some() && w.f.death1_cb.is_some());

    // 800EE74C / 800EE7B8 clear their own callback slot after running the cleanups.
    unsafe {
        t::ext::FTKB_SPECIALAIRLW_800F5524 = Some(|_| log("5524".into()));
        t::ext::FTKB_ATTACKDASHAIR_800F22D4 = Some(|_| log("22D4".into()));
        t::ext::FTKB_SPECIALAIRLW_800F5318 = Some(|_| log("5318".into()));
        t::ext::FTKB_SPECIALN_800F9090 = Some(|_| log("9090".into()));
        t::ext::FTKB_SPECIALN_800F5D04 = Some(|_, b| log(format!("5D04 {b}")));
    }
    w.f.u.kb.hat.kind = Ft_Kind_Kirby; // no per-hat cleanup
    t::ftKb_Init_800EE74C(gp);
    assert!(w.f.death3_cb.is_none() && w.f.death1_cb.is_some());
    t::ftKb_Init_800EE7B8(gp);
    assert!(w.f.death1_cb.is_none());
    assert_eq!(take(), "5524,22D4,5318,9090,5D04 false,5524,22D4,5318,9090");

    // Charge flash at full charge, per hat kind.
    unsafe {
        st::FTCOMMON_8007D7FC = Some(|_| {});
        t::ext::FTCO_800BFFD0 = Some(|_, a, b| log(format!("flash {a} {b}")));
    }
    w.da.specialn_dk_swings_to_full_charge = 3;
    w.da.specialn_ss_charge_time = 5.0;
    w.da.specialn_mt_charge_time = 6.0;
    unsafe {
        w.f.u.kb.xBC = 3;
        w.f.u.kb.hat.kind = Ft_Kind_Donkey;
        t::ftKb_Init_UnkMotionStates4(gp);
        w.f.u.kb.xA8 = 5;
        w.f.u.kb.hat.kind = Ft_Kind_Samus;
        t::ftKb_Init_UnkMotionStates4(gp);
        w.f.u.kb.x9C = 6;
        w.f.u.kb.hat.kind = Ft_Kind_Mewtwo;
        t::ftKb_Init_UnkMotionStates4(gp);
        w.f.u.kb.xB4 = 6;
        w.f.u.kb.hat.kind = Ft_Kind_Seak;
        t::ftKb_Init_UnkMotionStates4(gp);
        w.f.u.kb.xB4 = 5;
        t::ftKb_Init_UnkMotionStates4(gp);
    }
    assert_eq!(take(), "flash 58 0,flash 54 0,flash 93 0,flash 87 0");
}

#[test]
fn hat_load_and_unload() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();
    unsafe {
        t::ft_80459B88 = core::mem::zeroed();
    }
    let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
    unsafe { t::ft_80459B88.hats[Ft_Kind_Fox as usize] = &mut *hat };
    // Simple hat (Fox row): load once, no second load while worn; unload frees the pool object.
    t::ftKb_SpecialN_800EFC58(gp);
    assert_eq!(take(), "alloc,setclass,loadjoint,clearclass,75650,7487C 0");
    assert!(!unsafe { w.f.u.kb.hat.jobj }.is_null());
    assert_eq!(w.f.x2225_b2(), 1);
    t::ftKb_SpecialN_800EFC58(gp);
    assert_eq!(take(), "");
    t::ftKb_SpecialN_800EFD08(gp);
    assert_eq!(take(), "jobjremoveall,free true");
    assert!(unsafe { w.f.u.kb.hat.jobj }.is_null() && unsafe { w.f.u.kb.hat.x14.data }.is_null());
    t::ftKb_SpecialN_800EFD08(gp);
    assert_eq!(take(), "");
    // No hat in the archive slot: nothing is loaded.
    t::ftKb_SpecialN_800F0B70(gp); // CLink
    assert_eq!(take(), "");

    // Neutral-B enter dispatch through the tables, or Kirby's own inhale.
    unsafe {
        t::ext::FTKB_SPECIALN_800F5F68 = Some(|_| log("inhale".into()));
        t::ext::FTKB_SPECIALN_800F6070 = Some(|_| log("airinhale".into()));
    }
    w.f.u.kb.hat.kind = 11; // no entry in the table
    t::ftKb_SpecialN_Enter(gp);
    t::ftKb_SpecialAirN_Enter(gp);
    assert_eq!(take(), "inhale,airinhale");

    // 800F10A4 etc. do nothing without the hat; with it they tear down the (absent) DObjs and parts.
    t::ftKb_SpecialN_800F10A4(gp);
    assert_eq!(take(), "");
}


// ---- multi-part hat instantiation (800EF040 / 800EF0E4 / 800EF438 / 800EF69C) ------------------

static mut JOINTS: [*mut HSD_Joint; 2] = [null_mut(); 2];
static mut DESCS: [HSD_DObjDesc; 3] = unsafe { core::mem::zeroed() };
static mut DOBJS: [HSD_DObj; 4] = unsafe { core::mem::zeroed() };
static mut MULTI_PARTS: *mut FighterBone = null_mut();
static mut BASE: *mut Fighter_804D6540_x0_t = null_mut();

#[test]
fn multipart_hat_instantiation() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();
    w.f.x619_costume_id = 1;
    w.f.kind = 2;
    // Bones: 0 and 2 are model bones (flags_b1), 1 is not.
    let mut jobjs: std::vec::Vec<Box<HSD_JObj>> = (0..4).map(|_| Box::new(unsafe { core::mem::zeroed() })).collect();
    for (i, b) in w.parts.iter_mut().take(4).enumerate() {
        b.joint = &mut *jobjs[i];
    }
    unsafe {
        w.parts[0].anon_8.anon_0.set_flags_b1(1);
        w.parts[2].anon_8.anon_0.set_flags_b1(1);
    }
    // Hat joint tree: two joints; joint 0's DObjDesc loads one DObj, joint 1's loads two (the first with a MObj).
    let mut tree: std::vec::Vec<Box<HSD_Joint>> = (0..2).map(|_| Box::new(unsafe { core::mem::zeroed() })).collect();
    let mut mobj: Box<HSD_MObj> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        for i in 0..2 {
            JOINTS[i] = &mut *tree[i];
            (*JOINTS[i]).u.dobjdesc = &raw mut DESCS[i];
        }
        DOBJS = core::mem::zeroed();
        DOBJS[1].next = &raw mut DOBJS[2];
        DOBJS[1].mobj = &mut *mobj;
        // Kind 2's costume rows (arg1 = 3 selects ftKb_Init_803C9ED8): row 1 holds the hat tree.
        t::ftKb_Init_803C9ED8[1].joint = JOINTS[0];
        t::ext::HSD_IDINSERTTOTABLE = Some(|_, key, j| unsafe {
            let idx = if key == JOINTS[0] as *mut c_void { 0 } else { 1 };
            let part = (0..4).position(|i| (*MULTI_PARTS.add(i)).joint == j).unwrap();
            log(format!("id {idx}->bone {part}"));
        });
        t::ext::FTANIM_GETNEXTJOINTINTREE = Some(|joint, idx| unsafe {
            *idx += 1;
            *joint = if *joint == JOINTS[0] { JOINTS[1] } else { null_mut() };
        });
        t::ext::HSD_DOBJLOADDESC = Some(|desc| unsafe {
            if desc == &raw mut DESCS[0] {
                &raw mut DOBJS[0]
            } else {
                &raw mut DOBJS[1]
            }
        });
        t::ext::HSD_JOBJGETDOBJ = Some(|_| null_mut());
        t::ext::HSD_DOBJRESOLVEREFSALL = Some(|d, _| unsafe { log(format!("resolve {}", d.offset_from(&raw mut DOBJS[0]))) });
        t::ext::HSD_JOBJADDDOBJ = Some(|_, d| unsafe { log(format!("add {}", d.offset_from(&raw mut DOBJS[0]))) });
        t::ext::HSDCHANGECLASS_FTMOBJ = Some(|_| log("mobjclass".into()));
        MULTI_PARTS = w.parts.as_mut_ptr();
    }
    let mut data = [null_mut::<HSD_DObj>(); 32];
    w.f.u.kb.hat.x14.data = data.as_mut_ptr();
    let mut indices = [0xEEu8; 16];
    t::ftKb_SpecialN_800EF0E4(gp, 3, indices.as_mut_ptr());
    assert_eq!(
        take(),
        "setclass,id 0->bone 0,id 1->bone 2,resolve 0,add 0,resolve 1,add 1,mobjclass,clearclass"
    );
    unsafe {
        assert_eq!(w.f.u.kb.hat.x14.count, 3);
        assert_eq!((data[0], data[1], data[2], data[3]), (&raw mut DOBJS[0], &raw mut DOBJS[1], &raw mut DOBJS[2], null_mut()));
    }
    assert_eq!((indices[0], indices[1], indices[2], indices[3]), (0, 0xEE, 1, 0xEE));
    let flags: std::vec::Vec<u8> = (0..4).map(|i| unsafe { w.parts[i].anon_8.anon_0.flags_b6() }).collect();
    assert_eq!(flags, [1, 0, 1, 0]);

    // Extra body parts (800EF040): mask 0b110 over a 3-entry part table -> entries 1 and 2.
    let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
    hat.hat_dynamics[1] = 0b110usize as *mut ftDynamics;
    let mut x0: [Fighter_804D6540_x0_t; 3] = unsafe { core::mem::zeroed() };
    let mut ent = Fighter_804D6540_t { x0: x0.as_mut_ptr(), x4: 3 };
    let mut tab: [*mut Fighter_804D6540_t; 4] = [&mut ent; 4];
    unsafe {
        melee_fighter::hooks::FIGHTER_804D6540 = tab.as_mut_ptr();
        BASE = x0.as_mut_ptr();
        t::ext::FTPARTS_800753D4 = Some(|_, p, j| unsafe { log(format!("800753D4 {} {}", p.offset_from(BASE), j == JOINTS[0])) });
        t::ext::FTPARTS_800755E8 = Some(|_, p| unsafe { log(format!("800755E8 {}", p.offset_from(BASE))) });
    }
    t::ftKb_SpecialN_800EF040(gp, 3, &mut *hat);
    assert_eq!(take(), "800753D4 1 true,800753D4 2 true");

    // Unload (800EF69C): bones with flags_b6 lose their DObjs, the pool objects are freed, the mask parts go
    // in descending order.
    let mut p2j = [0u8; 4];
    let mut table = FighterPartsTable { joint_to_part: null_mut(), part_to_joint: p2j.as_mut_ptr(), parts_num: 4 };
    let mut ptabs: [*mut FighterPartsTable; 4] = [&mut table; 4];
    unsafe {
        melee_fighter::hooks::FT_PARTS_TABLE = ptabs.as_mut_ptr();
        t::ext::HSD_DOBJREMOVEALL = Some(|d| log(format!("removeall {}", d.is_null())));
        t::ext::LB_8000CE40 = Some(|j, _| unsafe { log(format!("ce40 {}", (0..4).position(|i| (*MULTI_PARTS.add(i)).joint == j).unwrap())) });
    }
    hat.hat_dynamics[1] = 0b101usize as *mut ftDynamics;
    let mut data2 = [null_mut::<HSD_DObj>(); 32];
    w.f.u.kb.hat.x1C.data = data2.as_mut_ptr();
    t::ftKb_SpecialN_800EF69C(gp, 3, &mut *hat);
    assert_eq!(
        take(),
        "removeall true,ce40 0,removeall true,ce40 2,free true,free true,800755E8 2,800755E8 0"
    );
    assert!(unsafe { w.f.u.kb.hat.x14.data }.is_null());
    let flags: std::vec::Vec<u8> = (0..4).map(|i| unsafe { w.parts[i].anon_8.anon_0.flags_b6() }).collect();
    assert_eq!(flags, [0, 0, 0, 0]);
}

// ---- Kirby-like kinds, bump dust, hat object registration, preload, m-ex hooks -----------------

#[test]
fn misc_hooks_and_flags() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();
    unsafe {
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, v, p| log(format!("sfx {id:x} {v} {p}")));
        t::ext::MEX_INTERNALFORPORTKIND = Some(|k| {
            log(format!("internal {k}"));
            if k == 100 {
                -1
            } else {
                k - 50
            }
        });
        t::ext::MEX_FTBASEKIND = Some(|i| if i == 55 { Ft_Kind_Kirby as i32 } else { 0 });
        t::ext::EFASYNC_SPAWN = Some(|_, _, a, id, _, pos| unsafe { log(format!("spawn {a} {id:x} {} {}", (*pos).x, (*pos).y)) });
    }
    // ftKb_MexKirbyLike: only m-ex kinds; cached once answered; unknown slot answers 0 uncached.
    assert_eq!(t::ftKb_MexKirbyLike(Ft_Kind_Kirby as i32), 0);
    assert_eq!(t::ftKb_MexKirbyLike(100), 0);
    assert_eq!(t::ftKb_MexKirbyLike(105), 1);
    assert_eq!(t::ftKb_MexKirbyLike(105), 1);
    assert_eq!(t::ftKb_MexKirbyLike(106), 0);
    assert_eq!(t::ftKb_MexKirbyLike(1000), 0);
    assert_eq!(take(), "internal 100,internal 105,internal 106");

    // 800F1D24: countdown, bump-dust on the first frame of contact, timer reset to 5 while touching.
    w.f.kind = Ft_Kind_Kirby;
    unsafe { w.f.u.kb.x60 = 2 };
    t::ftKb_SpecialN_800F1D24(gp);
    assert_eq!(unsafe { w.f.u.kb.x60 }, 1);
    t::ftKb_SpecialN_800F1D24(gp);
    t::ftKb_SpecialN_800F1D24(gp);
    assert_eq!(take(), "");
    w.f.coll_data.env_flags = 0x800;
    w.f.coll_data.prev_env_flags = 0;
    w.f.coll_data.ecb.left = Vec2 { x: -3.0, y: 4.0 };
    t::ftKb_SpecialN_800F1D24(gp); // x60 == 0 -> DAC runs, then reset to 5
    assert_eq!(unsafe { w.f.u.kb.x60 }, 5);
    assert_eq!(take(), "spawn 2 49e -3 4");
    w.f.coll_data.prev_env_flags = 0x40; // already touching the wall
    t::ftKb_SpecialN_800F1DAC(gp);
    assert_eq!(take(), "");
    w.f.coll_data.env_flags = 0x10000;
    w.f.coll_data.prev_env_flags = 0;
    w.f.coll_data.ecb.bottom = Vec2 { x: 0.0, y: -1.5 };
    t::ftKb_SpecialN_800F1DAC(gp);
    assert_eq!(take(), "spawn 2 49e 0 -1.5");
    // Non-Kirby kinds skip everything.
    w.f.kind = Ft_Kind_Mario;
    unsafe { w.f.u.kb.x60 = 0 };
    t::ftKb_SpecialN_800F1D24(gp);
    let mut pos = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    t::ftKb_SpecialN_800F1F1C(gp, &mut pos);
    assert_eq!(take(), "");
    w.f.kind = Ft_Kind_Kirby;
    t::ftKb_SpecialN_800F1F1C(gp, &mut pos);
    assert_eq!(take(), "spawn 2 49e 1 2");

    // Held-charge hats (fn_800F1CA0 / 800F1CD8).
    for (k, want) in [(Ft_Kind_Captain, true), (Ft_Kind_Seak, true), (Ft_Kind_Popo, true), (Ft_Kind_Nana, true), (Ft_Kind_Samus, true), (Ft_Kind_Fox, false)] {
        unsafe { w.f.u.kb.hat.kind = k };
        assert_eq!(t::fn_800F1CA0(gp), want);
        assert_eq!(t::ftKb_SpecialN_800F1CD8(gp), want as i32);
    }

    // 800F1BAC with the same hat: only the "keep" jingle.
    w.f.u.kb.hat.kind = Ft_Kind_Fox;
    t::ftKb_SpecialN_800F1BAC(gp, Ft_Kind_Fox as i32, true);
    assert_eq!(take(), "sfx 222f6 127 64");
    assert!(w.f.death3_cb.is_some() && w.f.death1_cb.is_some());

    // Hook/ability cleanup dispatch (800F19AC / 800F1A8C).
    unsafe {
        t::ext::FTKB_SPECIALNIC_80108D04 = Some(|_| log("ic".into()));
        t::ext::FTKB_SPECIALNPE_8010C3F4 = Some(|_| log("pe".into()));
        t::ext::FTKB_SPECIALNLK800FB800 = Some(|_| log("lk0".into()));
        t::ext::FTKB_SPECIALNLK800FB840 = Some(|_| log("lk1".into()));
        t::ext::FTKB_SPECIALNMT_8010709C = Some(|_| log("mt-death".into()));
        t::ext::FTKB_SPECIALNMT_80107130 = Some(|_| log("mt-dmg".into()));
        t::ext::FTKB_SPECIALNSS_800FCD60 = Some(|_| log("ss".into()));
        t::ext::FTKB_SPECIALNSK_8010603C = Some(|_| log("sk".into()));
        t::ext::FTKB_SPECIALNYS_801093B4 = Some(|_| log("ys0".into()));
        t::ext::FTKB_SPECIALNYS_8010A8BC = Some(|_| log("ys1".into()));
        t::ext::FTKB_SPECIALNPR_80100DE0 = Some(|_| log("pr".into()));
        t::ext::FTKB_SPECIALNKP_800FA7D4 = Some(|_| log("kp".into()));
    }
    let kinds = [Ft_Kind_Popo, Ft_Kind_Peach, Ft_Kind_Link, Ft_Kind_CLink, Ft_Kind_Mewtwo, Ft_Kind_Samus, Ft_Kind_Seak, Ft_Kind_Yoshi, Ft_Kind_Donkey];
    for k in kinds {
        unsafe { w.f.u.kb.hat.kind = k };
        t::ftKb_SpecialN_800F19AC(gp);
    }
    assert_eq!(take(), "ic,pe,lk0,lk1,lk0,lk1,mt-death,ss,sk,ys0,ys1,pr");
    w.f.u.kb.hat.kind = Ft_Kind_Mewtwo;
    t::ftKb_SpecialN_800F1A8C(gp);
    assert_eq!(take(), "mt-dmg");
    w.f.u.kb.hat.kind = Ft_Kind_Koopa;
    t::ftKb_Init_UnkMotionStates3(gp);
    w.f.u.kb.hat.kind = Ft_Kind_GKoops;
    t::ftKb_Init_UnkMotionStates3(gp);
    w.f.u.kb.hat.kind = Ft_Kind_Fox;
    t::ftKb_Init_UnkMotionStates3(gp);
    assert_eq!(take(), "kp,kp");

    // 800F16D0: registers the copied ability's articles (hat_dynamics slots as raw article pointers).
    unsafe {
        t::ext::IT_8026B3F8 = Some(|a, k| log(format!("art {} {k}", a as usize)));
        t::ft_80459B88 = core::mem::zeroed();
    }
    let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
    for i in 0..5 {
        hat.hat_dynamics[i] = (100 + i) as *mut ftDynamics;
    }
    unsafe { t::ft_80459B88.hats[0] = &mut *hat };
    t::ftKb_SpecialN_800F16D0(gp, Ft_Kind_Fox as i32);
    assert_eq!(take(), format!("art 100 {},art 101 {}", It_Kind_Kirby_FoxLaser, It_Kind_Kirby_FoxBlaster));
    t::ftKb_SpecialN_800F16D0(gp, Ft_Kind_Luigi as i32); // hats[16] is empty
    t::ftKb_SpecialN_800F16D0(gp, Ft_Kind_Kirby as i32); // not an ability kind
    assert_eq!(take(), "");
}

// ---- preload helpers, table reset, m-ex hat hook ----------------------------------------------

#[test]
fn preload_and_reset() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    unsafe {
        t::ext::LBDVD_800178E8 = Some(|a, name, b, c, dd, e, f, g, h| unsafe {
            log(format!("dvd {a} {} {b}{c}{dd}{e}{f}{g}{h}", core::ffi::CStr::from_ptr(name as *const _).to_str().unwrap()))
        });
        t::ext::EFASYNC_LOADASYNC = Some(|id| log(format!("async {id}")));
    }
    // Donkey (kind 3), all six costumes: archive, six costume files, effect bank.
    t::ftKb_SpecialN_800EEC34(3, 0xFF, 6);
    let got = take();
    assert!(got.starts_with("dvd 2 PlKbCpDk.dat 4401310,dvd 2 PlKbNrCpDk.dat 4401310,"), "{got}");
    assert_eq!(got.matches("dvd").count(), 7);
    assert!(got.ends_with(&format!("async {}", unsafe { d::ftKb_Init_803CB46C[3] })), "{got}");
    // One costume only (row mapped through ftKb_CopyCostumeRow), and a Kirby clone with no archive.
    t::ftKb_SpecialN_800EEC34(3, 2, 6);
    let got = take();
    assert_eq!(got.matches("dvd").count(), 2);
    assert!(got.contains("PlKbBuCpDk.dat"), "{got}");
    t::ftKb_SpecialN_800EEC34(4, 0xFF, 6); // Kirby: no files, but the effect check still runs (255 != -1 is true)
    assert_eq!(take(), "async 255");
    t::ftKb_SpecialN_800EEC34(-1, 0xFF, 6);
    t::ftKb_SpecialN_800EEC34(500, 0xFF, 6);
    assert_eq!(take(), "");
    // The costume count bound is clamped to six.
    t::ftKb_SpecialN_800EEC34(15, 0xFF, 8);
    assert_eq!(take().matches("dvd").count(), 7);

    // ftKb_Init_800EE528: every loaded slot and every archive row cleared.
    unsafe {
        t::ft_80459B88 = core::mem::zeroed();
        t::ft_80459B88.x0 = 1usize as *mut Kirby_Unk;
        for i in 0..125 {
            t::ft_80459B88.hats[i] = (i + 1) as *mut KirbyHatStruct;
        }
        t::ft_80459B88.hats[126] = 9usize as *mut KirbyHatStruct;
        t::ftKb_Init_803C9ED8[2].joint = 1usize as *mut HSD_Joint;
        t::ftKb_Init_803C9F98[5].matanim = 1usize as *mut HSD_MatAnimJoint;
    }
    t::ftKb_Init_800EE528();
    unsafe {
        assert!(t::ft_80459B88.x0.is_null());
        assert!(t::ft_80459B88.hats[..125].iter().all(|p| p.is_null()));
        assert_eq!(t::ft_80459B88.hats[126] as usize, 9); // the C walks only Ft_Kind_Max s32 slots
        assert!(t::ftKb_Init_803C9ED8[2].joint.is_null() && t::ftKb_Init_803C9F98[5].matanim.is_null());
    }

    // ftKb_MexCopyKindHat: copies the base's archive row, takes the callbacks from m-ex (NULL stays NULL).
    unsafe {
        t::ext::MEX_KIRBYCOSTUMES = Some(|_| null_mut());
        t::ext::MEX_KIRBYFUNC = Some(|slot, _| match slot {
            0 => 0x10usize as *mut c_void,
            1 => null_mut(),
            2 => null_mut(),
            _ => 0x30usize as *mut c_void,
        });
        t::ftKb_MexCopyKindHat(50, 3, 27);
        assert_eq!(t::ftKb_Init_803C9FC8[50], t::ftKb_Init_803C9FC8[3]);
        assert!(d::ftKb_Init_803CB3E8[50].is_null());
        assert_eq!(t::ftKb_Init_803C9CC8[100].map(|f| f as usize), Some(0x10));
        assert!(t::ftKb_Init_803C9CC8[101].is_none() && t::ftKb_Init_803C9DD0[50].is_none());
        assert_eq!(t::ftKb_Init_803C9E54[50].map(|f| f as usize), Some(0x30));
        // Costume strings without a runtime archive array are ignored.
        t::ext::MEX_KIRBYCOSTUMES = Some(|_| 1usize as *mut c_void);
        t::ftKb_MexCopyKindHat(51, 4, 27); // kind 4 (Kirby) has no archive row
        assert!(d::ftKb_Init_803CB3E8[51].is_null());
        t::ftKb_MexCopyKindHat(52, 3, 27); // row exists: the costume table is kept
        assert_eq!(d::ftKb_Init_803CB3E8[52] as usize, 1);
    }
}

// ---- small helpers ---------------------------------------------------------------------------

#[test]
fn small_helpers() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();

    // Demo motion file names: kind 11 -> row 0, 14 -> row 3, anything else reads row -(14 - 0) wrapped by the C
    // (offset 0 -> index -14), so only the two real kinds are checked.
    let n = |k: i32| unsafe { core::ffi::CStr::from_ptr(t::ftKb_Init_GetMotionFileString(k) as *const _).to_str().unwrap().to_string() };
    assert_eq!(n(11), "ftDemoVi0501MotionFileKirby");
    assert_eq!(n(14), "ftDemoVi0502MotionFileKirby");

    // Hat jobj, or the gobj itself when there is none (the C's "correctness hack").
    assert_eq!(t::ftKb_Init_UnkMotionStates6(gp) as usize, gp as usize);
    let mut j: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    w.f.u.kb.hat.jobj = &mut *j;
    assert_eq!(t::ftKb_Init_UnkMotionStates6(gp), &mut *j as *mut HSD_JObj);

    // Pichu hat's dynamics words (800EEB00 / 800EEB1C): NULL without a hat.
    unsafe { t::ft_80459B88 = core::mem::zeroed() };
    let mut bones = [ArticleDynamicBones::zeroed()];
    let mut dyn4 = Box::new(ftDynamics::zeroed());
    dyn4.anon_0.ftDynamicBones = bones.as_mut_ptr();
    dyn4.x4 = 77;
    let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
    hat.hat_dynamics[4] = &mut *dyn4;
    let mut out_bones: *mut ArticleDynamicBones = 1usize as *mut _;
    let mut out_x4: i32 = 5;
    t::ftKb_Init_800EEB00(gp, &mut out_bones);
    t::ftKb_Init_800EEB1C(gp, &mut out_x4);
    assert!(out_bones.is_null() && out_x4 == 0);
    unsafe { t::ft_80459B88.hats[Ft_Kind_Pichu as usize] = &mut *hat };
    t::ftKb_Init_800EEB00(gp, &mut out_bones);
    t::ftKb_Init_800EEB1C(gp, &mut out_x4);
    assert_eq!((out_bones, out_x4), (bones.as_mut_ptr(), 77));

    // Hat unload callback by kind.
    unsafe {
        t::ftKb_Init_803C9CC8[2 * 120 + 1] = Some(cb_test);
    }
    t::ftKb_SpecialN_800EEEC4(gp, 120);
    t::ftKb_SpecialN_800EEEC4(gp, 119);
    assert_eq!(take(), "cb");

    // Item pickup / invisible: light items pose the hand by hold kind.
    unsafe {
        st::ITISHEAVY = Some(|_| 0);
        t::ext::ITGETHOLDKIND = Some(|_| 3);
        t::ext::FTANIM_80070FB4 = Some(|_, a, b| log(format!("70FB4 {a} {b}")));
        st::FTANIM_80070C48 = Some(|_, v| log(format!("70C48 {v}")));
        st::FTANIM_80070CC4 = Some(|_, v| log(format!("70CC4 {v}")));
    }
    t::ftKb_Init_OnItemPickup(gp, true);
    t::ftKb_Init_OnItemPickup(gp, false);
    t::ftKb_Init_OnItemInvisible(gp);
    unsafe { st::ITISHEAVY = Some(|_| 1) };
    t::ftKb_Init_OnItemPickup(gp, true);
    t::ftKb_Init_OnItemInvisible(gp);
    assert_eq!(take(), "70FB4 1 1,70C48 1,70FB4 1 1,70CC4 1");

    // 800F1420: write the colour word into the diffuse of each listed DObj's material.
    let mut mats: [HSD_Material; 2] = unsafe { core::mem::zeroed() };
    let mut mobjs: [HSD_MObj; 2] = unsafe { core::mem::zeroed() };
    let mut dobjs: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
    mobjs[0].mat = &mut mats[0];
    mobjs[1].mat = &mut mats[1];
    dobjs[0].mobj = &mut mobjs[0];
    dobjs[1].mobj = &mut mobjs[1];
    let mut data = [&mut dobjs[0] as *mut HSD_DObj, &mut dobjs[1], &mut dobjs[2]];
    w.f.u.kb.hat.x14.data = data.as_mut_ptr();
    let mut idx = [0u8, 2, 1];
    let mut ts = [TempS { x0: 2, x4: idx.as_mut_ptr() }, TempS { x0: 1, x4: unsafe { idx.as_mut_ptr().add(2) } }];
    let mut lookup = FtPartsVisLookup { x0: 2, x4: ts.as_mut_ptr() };
    w.f.x5AC.xC[4] = &mut lookup;
    let colour = u32::from_le_bytes([0x11, 0x22, 0x33, 0x44]);
    let last = t::ftKb_SpecialN_800F1420(gp, &colour);
    assert_eq!(unsafe { last.offset_from(idx.as_mut_ptr().add(2)) }, 1);
    let diffuse = |m: &HSD_Material| unsafe { core::ptr::read_unaligned(&raw const m.diffuse as *const u32) };
    assert_eq!((diffuse(&mats[0]), diffuse(&mats[1])), (colour, colour));

    // The shared hooks for the show/hide and rescale paths log (name, model slot, which list/vis).
    unsafe {
        FBASE = &*w.f as *const Fighter as usize;
        t::ext::FIGHTER_UPDATEMODELSCALE = Some(|_| log("scale".into()));
        t::ext::FTPARTS_80074CA0 = Some(|v, i, l| unsafe { log(format!("74CA0 {i} vis@{} list@{}", v as usize - FBASE, l as usize - FBASE)) });
        t::ext::FTPARTS_80074D7C = Some(|v, i, l| unsafe { log(format!("74D7C {i} vis@{} list@{}", v as usize - FBASE, l as usize - FBASE)) });
        t::ext::FTANIM_800705E0 = Some(|l| unsafe { log(format!("705E0 @{}", l as usize - FBASE)) });
        t::ext::FTANIM_80070458 = Some(|_, l, i, f| unsafe { log(format!("70458 @{} {i} {f}", l as usize - FBASE)) });
    }
    let off = |p: usize| p - unsafe { FBASE };
    let a_x24 = off(unsafe { &raw const w.f.u.kb.hat.x24 } as usize);
    let a_x14 = off(unsafe { &raw const w.f.u.kb.hat.x14 } as usize);
    let a_x1c = off(unsafe { &raw const w.f.u.kb.hat.x1C } as usize);
    let a_5ac = off(&raw const w.f.x5AC as usize);
    let a_203c = off(&raw const w.f.x203C as usize);
    let a_dl = off(&raw const w.f.dobj_list as usize);
    let a_x44 = off(unsafe { &raw const w.f.u.kb.x44 } as usize);
    let fpp: *mut Fighter = &mut *w.f;
    // x14.data null: nothing at all.
    w.f.u.kb.hat.x14.data = null_mut();
    t::ftKb_UnkIntBoolFunc0(fpp, 1, true);
    t::ftKb_Init_UnkCallbackPairs0_0(gp);
    t::ftKb_Init_UnkCallbackPairs0_1(gp, 1, 2.0);
    assert_eq!(take(), "");
    // With a hat jobj the hat's own model is shown; the callback pairs skip (jobj present).
    w.f.u.kb.hat.x14.data = data.as_mut_ptr();
    t::ftKb_UnkIntBoolFunc0(fpp, 1, true);
    t::ftKb_Init_UnkCallbackPairs0_0(gp);
    assert_eq!(take(), format!("74CA0 1 vis@{a_x24} list@{a_x14}"));
    // Without a jobj: animation callbacks run; the hide path depends on the model slot and bone index.
    w.f.u.kb.hat.jobj = null_mut();
    t::ftKb_Init_UnkCallbackPairs0_0(gp);
    t::ftKb_Init_UnkCallbackPairs0_1(gp, 3, 0.5);
    assert_eq!(take(), format!("705E0 @{a_x44},70458 @{a_x44} 3 0.5"));
    w.f.x5F4_arr[0].idx = 4;
    t::ftKb_UnkIntBoolFunc0(fpp, 1, true);
    w.f.x5F4_arr[0].idx = 0;
    t::ftKb_UnkIntBoolFunc0(fpp, 1, true);
    assert_eq!(
        take(),
        format!("74D7C 1 vis@{a_x24} list@{a_x14},74D7C 1 vis@{a_5ac} list@{a_dl},74CA0 1 vis@{a_x24} list@{a_x14}")
    );
    let mut x1c_data = [null_mut::<HSD_DObj>(); 2];
    w.f.u.kb.hat.x1C.data = x1c_data.as_mut_ptr();
    t::ftKb_UnkIntBoolFunc0(fpp, 2, true); // slot 2 with no second model: nothing
    x1c_data[0] = &mut dobjs[0];
    t::ftKb_UnkIntBoolFunc0(fpp, 2, true);
    t::ftKb_UnkIntBoolFunc0(fpp, 2, false);
    t::ftKb_UnkIntBoolFunc0(fpp, 1, false);
    assert_eq!(
        take(),
        format!(
            "74D7C 2 vis@{a_5ac} list@{a_203c},74CA0 2 vis@{a_x24} list@{a_x1c},74D7C 2 vis@{a_x24} list@{a_x1c},74D7C 1 vis@{a_x24} list@{a_x14}"
        )
    );

    // 800F15D8 unloads the Pichu hat (nothing allocated here, no mask), clears the lookup and rescales.
    w.f.u.kb.hat.x14.data = null_mut();
    t::ftKb_SpecialN_800F15D8(gp);
    assert!(w.f.x5AC.xC[4].is_null());
    assert_eq!(take(), "scale");
}

static mut FBASE: usize = 0;

unsafe extern "C" fn cb_test(_g: *mut HSD_GObj) {
    log("cb".into());
}

// ---- mat-anim binding, second DObj set, hat draw, Pichu hat --------------------------------------

#[test]
fn matanim_second_set_and_draw() {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    take();
    install_common_hooks();
    let mut w = World::new();
    let gp = w.gp();
    w.f.x619_costume_id = 0;
    w.f.kind = 1;
    unsafe {
        t::ext::HSD_DOBJADDANIMALL = Some(|d, m, a| log(format!("addanim {} {} {}", d as usize, m as usize, a.is_null())));
        t::ext::FTANIM_GETNEXTMATANIMJOINTINTREE = Some(|j, idx| unsafe {
            *idx += 1;
            *j = (**j).next;
        });
    }
    // 800EF35C: part 0 is skipped (the part table says it is a swapped-out body part), part 1 gets the anim.
    let mut x0: [Fighter_804D6540_x0_t; 1] = unsafe { core::mem::zeroed() };
    let mut ent = Fighter_804D6540_t { x0: x0.as_mut_ptr(), x4: 1 };
    let mut tab: [*mut Fighter_804D6540_t; 4] = [&mut ent; 4];
    unsafe { melee_fighter::hooks::FIGHTER_804D6540 = tab.as_mut_ptr() };
    let mut maj: Box<HSD_MatAnimJoint> = Box::new(unsafe { core::mem::zeroed() });
    maj.matanim = 0x77usize as *mut _;
    unsafe { t::ftKb_Init_803C9ED8[0].matanim = &mut *maj };
    let mut dobjs: [HSD_DObj; 3] = unsafe { core::mem::zeroed() };
    let mut data = [&mut dobjs[0] as *mut HSD_DObj, &mut dobjs[1], &mut dobjs[2]];
    w.f.u.kb.hat.x14.data = data.as_mut_ptr();
    let mut indices = [9u8, 2, 0, 0];
    t::ftKb_SpecialN_800EF35C(gp, 3, indices.as_mut_ptr());
    assert_eq!(take(), format!("addanim {} 119 true", &dobjs[2] as *const _ as usize));
    unsafe { t::ftKb_Init_803C9ED8[0].matanim = null_mut() };

    // 800EF438 without a second set: the first slot of the second list is cleared.
    let mut hat: Box<KirbyHatStruct> = Box::new(KirbyHatStruct::zeroed());
    let mut data2 = [&mut dobjs[0] as *mut HSD_DObj; 2];
    w.f.u.kb.hat.x1C.data = data2.as_mut_ptr();
    t::ftKb_SpecialN_800EF438(gp, &mut *hat);
    assert!(data2[0].is_null() && !data2[1].is_null());
    assert_eq!(take(), "");

    // Draw: only with a hat model and x2225_b2; the hat takes the bone-6 matrix and is flagged user-matrix.
    static mut MTX: [[f32; 4]; 3] = [[0.0; 4]; 3];
    unsafe {
        t::ext::HSD_JOBJGETMTXPTR = Some(|_| (&raw mut MTX) as *mut [f32; 4]);
        t::ext::HSD_JOBJCOPYMTX = Some(|_, m| log(format!("copymtx {}", m == (&raw mut MTX) as *mut [f32; 4])));
        t::ext::HSD_JOBJSETMTXDIRTY = Some(|_| log("dirty".into()));
        t::ext::HSD_JOBJDISPALL = Some(|_, _, r, f| log(format!("disp {r} {f}")));
        t::ext::HSD_GOBJ_80390EB8 = Some(|a| a as u32 + 100);
    }
    let mut jobj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    w.parts[6].joint = &mut *jobj;
    let mut hatj: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
    let mtx: MtxPtr = unsafe { (&raw mut MTX) as *mut [f32; 4] };
    t::ftKb_UnkMtxFunc0(gp, 5, mtx);
    assert_eq!(take(), "");
    w.f.u.kb.hat.jobj = &mut *hatj;
    t::ftKb_UnkMtxFunc0(gp, 5, mtx);
    assert_eq!(take(), "");
    w.f.set_x2225_b2(1);
    t::ftKb_UnkMtxFunc0(gp, 5, mtx);
    assert_eq!(take(), "copymtx true,dirty,disp 105 0");
    assert_eq!(hatj.flags as i32 & (JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT), JOBJ_USER_DEF_MTX | JOBJ_MTX_INDEP_PARENT | JOBJ_MTX_INDEP_SRT);

    // Pichu hat (800F14B4): multi-part load + recolour from the archive's colour words + rescale.
    unsafe {
        t::ext::FIGHTER_UPDATEMODELSCALE = Some(|_| log("scale".into()));
        t::ext::FTPARTS_80074D7C = Some(|_, i, _| log(format!("74D7C {i}")));
        t::ext::FTANIM_80070200 = Some(|_, _, _, _| log("70200".into()));
        t::ext::FTPARTS_800753D4 = Some(|_, _, _| log("753D4".into()));
        t::ext::HSD_DOBJLOADDESC = Some(|_| null_mut());
        t::ext::HSD_IDINSERTTOTABLE = Some(|_, _, _| {});
        t::ext::FTANIM_GETNEXTJOINTINTREE = Some(|j, _| unsafe { *j = null_mut() });
        t::ft_80459B88 = core::mem::zeroed();
    }
    w.f.kind = 0;
    w.f.u.kb.hat.x14.data = null_mut();
    w.f.u.kb.hat.jobj = null_mut();
    // hat_dynamics: [2] none, [3] the vis lookup, [4] raw colour words (+4: diffuse, +8: fighter colour).
    let mut lookup = FtPartsVisLookup { x0: 0, x4: null_mut() };
    let colour_words: [u32; 3] = [0, u32::from_le_bytes([1, 2, 3, 4]), u32::from_le_bytes([5, 6, 7, 8])];
    hat.hat_dynamics[2] = null_mut();
    hat.hat_dynamics[3] = &mut lookup as *mut _ as *mut ftDynamics;
    hat.hat_dynamics[4] = colour_words.as_ptr() as *mut ftDynamics;
    hat.hat_dynamics[1] = null_mut();
    unsafe { t::ft_80459B88.hats[Ft_Kind_Pichu as usize] = &mut *hat };
    let mut p2j = [0u8; 4];
    let mut table = FighterPartsTable { joint_to_part: null_mut(), part_to_joint: p2j.as_mut_ptr(), parts_num: 4 };
    let mut ptabs: [*mut FighterPartsTable; 4] = [&mut table; 4];
    unsafe { melee_fighter::hooks::FT_PARTS_TABLE = ptabs.as_mut_ptr() };
    t::ftKb_SpecialN_800F14B4(gp);
    let got = take();
    assert!(got.ends_with("74D7C 4,scale"), "{got}");
    assert!(got.contains("70200"), "{got}");
    assert_eq!(w.f.x5AC.xC[4], &mut lookup as *mut _);
    assert_eq!(unsafe { core::ptr::read_unaligned(&raw const w.f.x610_color_rgba[1] as *const u32) }, colour_words[2]);
    // Already loaded: nothing happens.
    t::ftKb_SpecialN_800F14B4(gp);
    assert_eq!(take(), "");
}
