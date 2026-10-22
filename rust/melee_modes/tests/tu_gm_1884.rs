//! tu::gm_1884 against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_1884_ref.c, data tests/data/tu_gm_1884.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_1884`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_1884 as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

fn btn() -> u64 {
    let r = rnd();
    (if r & 1 != 0 { 1u64 << 36 } else { 0 })
        | (if r & 2 != 0 { 1u64 << 37 } else { 0 })
        | (if r & 4 != 0 { 1u64 << 38 } else { 0 })
        | (if r & 8 != 0 { 1u64 << 39 } else { 0 })
        | (if r & 16 != 0 { 0x100u64 } else { 0 })
}
unsafe fn mkjobj() -> *mut HSD_JObj {
    let j = az_t::<HSD_JObj>();
    (*j).translate.x = rf();
    (*j).translate.y = rf();
    (*j).translate.z = rf();
    j
}
unsafe fn mkgobj() -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = mkjobj() as *mut _;
    g
}

include!("stubs/gm_1884.rs");

unsafe fn mkmodels() -> *mut *mut DynamicModelDesc {
    let t = az(16) as *mut *mut DynamicModelDesc;
    *t = az_t::<DynamicModelDesc>();
    (**t).joint = az(64) as *mut HSD_Joint;
    t
}
fn t_lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_80016DBC");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) = mkmodels() as *mut _;
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}
fn t_lb_80011E24(j: *mut HSD_JObj, o: *mut *mut HSD_JObj, id: i32, end: i32) -> i32 {
    unsafe {
        out("lb_80011E24");
        trp(j as *const u8);
        trp(o as *const u8);
        tri(id as i64);
        tri(end as i64);
        out("\n");
        *o = mkjobj();
        1
    }
}

/// The PPC bitfields of StartMeleeRules in the C's declaration order: (getter, setter).
macro_rules! rule_fields {
    ($m:ident) => {
        $m!(match_kind, set_match_kind); $m!(x0_3, set_x0_3); $m!(timer_enabled, set_timer_enabled);
        $m!(timer_counts_up, set_timer_counts_up); $m!(x1_0, set_x1_0); $m!(x1_1, set_x1_1);
        $m!(x1_2, set_x1_2); $m!(x1_3, set_x1_3); $m!(x1_4, set_x1_4); $m!(x1_5, set_x1_5);
        $m!(timer_shows_hours, set_timer_shows_hours); $m!(friendly_fire, set_friendly_fire);
        $m!(is_stock, set_is_stock); $m!(x2_1, set_x2_1); $m!(x2_2, set_x2_2);
        $m!(single_button, set_single_button); $m!(disable_pausing, set_disable_pausing);
        $m!(x2_5, set_x2_5); $m!(x2_6, set_x2_6); $m!(x2_7, set_x2_7); $m!(x3_0, set_x3_0);
        $m!(x3_1, set_x3_1); $m!(x3_2, set_x3_2); $m!(x3_3, set_x3_3); $m!(x3_4, set_x3_4);
        $m!(x3_5, set_x3_5); $m!(x3_6, set_x3_6); $m!(x3_7, set_x3_7); $m!(x4_0, set_x4_0);
        $m!(is_vs, set_is_vs); $m!(x4_2, set_x4_2); $m!(x4_3, set_x4_3); $m!(x4_4, set_x4_4);
        $m!(x4_5, set_x4_5); $m!(x4_6, set_x4_6); $m!(x4_7, set_x4_7); $m!(x5_0, set_x5_0);
        $m!(x5_1, set_x5_1); $m!(x5_2, set_x5_2); $m!(x5_3, set_x5_3); $m!(x5_4, set_x5_4);
        $m!(x5_5, set_x5_5); $m!(x5_6, set_x5_6); $m!(x5_7, set_x5_7);
    };
}

static mut G_SMD: StartMeleeData = unsafe { core::mem::zeroed() };

unsafe fn seed_rules() {
    macro_rules! seed {
        ($g:ident, $s:ident) => {
            G_SMD.rules.$s(rnd());
        };
    }
    rule_fields!(seed);
    G_SMD.rules.is_teams = rnd() as u8;
    G_SMD.rules.item_freq = rnd() as i8;
    G_SMD.rules.x20 = ((rnd() as u64) << 32) | rnd() as u64;
}
unsafe fn print_smd() {
    let mut h: u64 = 1469598103934665603;
    macro_rules! fp {
        ($g:ident, $s:ident) => {
            h = h.wrapping_mul(1000003).wrapping_add(G_SMD.rules.$g() as u64);
        };
    }
    rule_fields!(fp);
    out(&format!("rules {:016x} {} {} {:016x}\n", h, G_SMD.rules.is_teams as i32, G_SMD.rules.item_freq as i32, G_SMD.rules.x20));
    dump("players", G_SMD.players.as_ptr() as *const u8, core::mem::size_of_val(&G_SMD.players));
}

unsafe fn state() {
    dump_t("lbl", &t::lbl_80473700);
    dump_t("gm", &t::gm_80473814);
    dump_t("a28", &t::lbl_804D6628);
    dump_t("a2c", &t::lbl_804D662C);
    dump_arena();
    state_end();
}
unsafe fn seed_state() {
    rfill_t(&mut t::lbl_80473700);
    rfill_t(&mut t::gm_80473814);
    let l = &mut t::lbl_80473700;
    l.count = (rnd() % 4) as i32;
    l.mode = (rnd() % 4) as i32;
    for i in 0..25 {
        l.char_data[i] = (rnd() % 1200) as i32;
    }
    l.result_cache[0] = (rnd() % 1200) as i32;
    l.result_cache[1] = (rnd() % 2) as i32;
    {
        let p = &raw mut t::lbl_80473700 as *mut i32;
        *p.add(67) = (rnd() % 1200) as i32;
        *p.add(68) = (rnd() % 2) as i32;
    }
    let g = &mut t::gm_80473814;
    for i in 0..39 {
        g.jobjs[i] = mkjobj();
        g.anim_frames[i] = (rnd() % 40) as i32;
    }
    g.gobj = core::ptr::null_mut();
    g.x00 = (rnd() % 9) as u8;
    g.x01 = (rnd() % 2) as u8;
    g.x03 = (rnd() % 6) as u8;
    g.menu_values[0] = (rnd() % 6) as i32;
    g.menu_values[1] = (rnd() % 30) as i32;
    g.menu_values[2] = (rnd() % 3) as i32;
    g.menu_values[3] = (rnd() % 6) as i32;
    {
        let r = rnd() % 8;
        g.menu_values[4] = if r == 0 {
            0
        } else if r == 1 {
            1
        } else if r == 2 {
            998
        } else if r == 3 {
            999
        } else if r == 4 {
            0x3E7
        } else {
            (rnd() % 1000) as i32
        };
    }
    g.menu_values[5] = (rnd() % 2) as i32;
    g.menu_values[6] = (rnd() % 3) as i32;
    g.text = az_t::<HSD_Text>();
    t::lbl_804D662C = mkmodels();
    t::lbl_804D6628 = az_t::<HSD_Archive>();
}
unsafe fn chain3() -> *mut HSD_JObj {
    let a = mkjobj();
    let b = mkjobj();
    let c = mkjobj();
    (*a).next = b;
    (*b).next = c;
    a
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HOOK_lb_80011E24 = Some(t_lb_80011E24);
        ext::HSD_GObj_JObjKind = 0x2A;
        t::OS_BUS_CLOCK = 162000000;
        size_t::<TrainingModeState>("TrainingModeState");
        size_t::<CssSubStruct>("CssSubStruct");
        size_t::<PlayerInitData>("PlayerInitData");
        size_t::<HSD_Text>("HSD_Text");
        size_t::<t::TrainingItemEntry>("TrainingItemEntry");
        reg("lbl", &t::lbl_80473700, core::mem::size_of::<TrainingModeState>());
        reg("gm", &t::gm_80473814, core::mem::size_of::<CssSubStruct>());
        reg("a28", &t::lbl_804D6628, 8);
        reg("a2c", &t::lbl_804D662C, 8);
        reg("smd", &G_SMD, core::mem::size_of::<StartMeleeData>());
        fnreg("fn_801891F4", t::fn_801891F4_cb as usize);
        fnreg("fn_80188EE8", t::fn_80188EE8 as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);
        dump("items", t::gmTraining_ItemTable.as_ptr() as *const u8, core::mem::size_of_val(&t::gmTraining_ItemTable));
        dump_t("speeds", &t::lbl_803B7C68);
        state_end();

        for c in 0..60usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::gm_80188454((c % 27) as i32)));
            state();
        }
        for c in 0..200usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            arena_reset();
            seed_state();
            out(&format!("r {}\n", t::fn_8018846C()));
            state();
            out(&format!("r {}\n", t::fn_801884F8()));
            state();
        }
        for c in 0..200usize {
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            seed_state();
            t::lbl_80473700.count = (c % 4) as i32;
            t::fn_80188550(((c / 4) % 4) as i32);
            state();
        }
        for c in 0..40usize {
            case(3000 + c);
            rseed(4000 + c as u32);
            arena_reset();
            seed_state();
            t::lbl_80473700.count = (c % 4) as i32;
            t::fn_80188644();
            state();
        }
        for c in 0..300usize {
            case(4000 + c);
            rseed(5000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80188738(if c % 7 == 0 { core::ptr::null_mut() } else { chain3() });
            state();
            t::fn_80188910(if c % 7 == 1 { core::ptr::null_mut() } else { chain3() });
            state();
            t::fn_80188B3C(if c % 7 == 2 { core::ptr::null_mut() } else { chain3() });
            state();
            {
                let r = mkjobj();
                (*r).child = if c % 7 == 3 { core::ptr::null_mut() } else { chain3() };
                t::fn_80188D3C(if c % 7 == 4 { core::ptr::null_mut() } else { r });
            }
            state();
        }
        for c in 0..500usize {
            case(5000 + c);
            rseed(6000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_80473814.jobjs[9] = chain3();
            t::gm_80473814.jobjs[5] = chain3();
            t::gm_80473814.jobjs[19] = chain3();
            (*t::gm_80473814.jobjs[27]).child = chain3();
            if c % 3 == 0 {
                t::gm_80473814.anim_frames[22] = 9 + (c % 4) as i32 * 10 + (rnd() % 3) as i32;
            }
            t::fn_80188EE8(mkgobj());
            state();
        }
        for c in 0..2500usize {
            case(6000 + c);
            rseed(7000 + c as u32);
            arena_reset();
            seed_state();
            if c % 5 == 0 {
                t::gm_80473814.x00 = (c / 5 % 9) as u8;
            }
            t::fn_801891F4();
            state();
        }
        for c in 0..4usize {
            case(9000 + c);
            rseed(10000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_80189B88();
            state();
        }
        for c in 0..40usize {
            case(10000 + c);
            rseed(11000 + c as u32);
            arena_reset();
            seed_state();
            rfill(G_SMD.players.as_mut_ptr() as *mut u8, core::mem::size_of_val(&G_SMD.players));
            seed_rules();
            G_SMD.players[0].slot = (rnd() % 5) as u8;
            t::gm_80189CDC(&raw mut G_SMD);
            print_smd();
            state();
        }
        for c in 0..8usize {
            case(11000 + c);
            rseed(12000 + c as u32);
            arena_reset();
            seed_state();
            out("r");
            trp(t::fn_8018A000() as *const u8);
            out("\n");
            state();
        }
    }
    check("tu_gm_1884");
}
