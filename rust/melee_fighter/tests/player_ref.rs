//! src/player.rs against the C (gen/cref/player_ref.c, the player.c accessors cut verbatim):
//! the same xorshift-driven random player tables and op sequences, the same tracing stubs for
//! every call out of player.c, every printed line compared (tests/data/player.txt).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::player as pl;
use melee_fighter::stale::PLAYER_SLOTS;
use melee_fighter::types::{pl_x5EC_t, HSD_GObj, StaticPlayer, Vec3};

static mut TR: u32 = 0;
static mut RS: u32 = 0;
static mut GA: HSD_GObj = unsafe { core::mem::zeroed() };
static mut GB: HSD_GObj = unsafe { core::mem::zeroed() };

fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}
fn b(f: f32) -> u32 {
    f.to_bits()
}
fn e(g: *mut HSD_GObj) -> u32 {
    if g.is_null() { 0 } else if g == &raw mut GA { 1 } else if g == &raw mut GB { 2 } else { 3 }
}
fn rnd() -> u32 {
    unsafe {
        RS ^= RS << 13;
        RS ^= RS >> 17;
        RS ^= RS << 5;
        RS
    }
}
fn rf() -> f32 {
    match rnd() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => 1.0,
        3 => -1.0,
        _ => ((rnd() % 2001) as i32 - 1000) as f32 / 7.0,
    }
}
fn rent() -> *mut HSD_GObj {
    match rnd() % 3 {
        0 => core::ptr::null_mut(),
        1 => &raw mut GA,
        _ => &raw mut GB,
    }
}
unsafe extern "C" fn sf(_a: i32) {}

fn install() {
    unsafe {
        h::SCRIPT_MOD_VALUE = Some(|s, f, base| {
            t(0x220 + s as u32 * 2 + f as u32);
            t(b(base));
            base
        });
        h::FTLIB_DISABLE_INPUT = Some(|g| t(0x10 + e(g)));
        h::FTLIB_ENABLE_INPUT = Some(|g| t(0x20 + e(g)));
        h::FTKB_SPECIALN_800EED50 = Some(|a, c| {
            t(0x30);
            t(a as u32);
            t(c as u32)
        });
        h::FTLIB_IS_SLEEPING = Some(|g| {
            t(0x40 + e(g));
            (tr() >> 3) & 1 != 0
        });
        h::FTCO_800D4F24 = Some(|g, i| {
            t(0x50 + e(g));
            t(i as u32)
        });
        h::HSD_GOBJ_FREE = Some(|g| t(0x60 + e(g)));
        h::FTCO_800D4FF4 = Some(|g| t(0x70 + e(g)));
        h::IFSTATUS_802F6E1C = Some(|s| t(0x80 + s as u32));
        h::FTLIB_IS_REBIRTH = Some(|g| {
            t(0x90 + e(g));
            (tr() >> 5) & 1 != 0
        });
        h::FTLIB_80087140 = Some(|g| t(0xA0 + e(g)));
        h::FTLIB_SET_POS = Some(|g, v| {
            t(0xB0 + e(g));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z))
        });
        h::FTDEMO_SET_FACING_DIRECTION = Some(|g, f| {
            t(0xC0 + e(g));
            t(b(f))
        });
        h::FTLIB_800869F8 = Some(|g| {
            t(0xD0 + e(g));
            (tr() % 1000) as i32 as f32 / 7.0
        });
        h::FTLIB_SET_SCALE = Some(|g, f| {
            t(0xE0 + e(g));
            t(b(f))
        });
        h::FT_80087BEC = Some(|g, v| {
            t(0xF0 + e(g));
            t(v as u32)
        });
        h::FT_80087BC0 = Some(|g, v| {
            t(0x100 + e(g));
            t(v as u32)
        });
        h::GM_8016C5C0 = Some(|s| {
            t(0x110 + s as u32);
            (tr() % 97) as i32
        });
        h::FTLIB_SET_PERCENT = Some(|g, v| {
            t(0x120 + e(g));
            t(v as u32)
        });
        h::IFSTATUS_802F6D10 = Some(|s| t(0x130 + s as u32));
        h::GM_8016B168 = Some(|| {
            t(0x140);
            (tr() >> 4) & 1 != 0
        });
        h::IFSTATUS_802F69C0 = Some(|a, c| {
            t(0x150);
            t(a as u32);
            t(c as u32)
        });
        h::IFSTATUS_802F6AF8 = Some(|s| t(0x160 + s as u32));
        h::GM_GET_FRAME_COUNT = Some(|| {
            t(0x170);
            tr()
        });
        h::IFSTATUS_802F6C04 = Some(|s| t(0x180 + s as u32));
        h::FTLIB_80086F4C = Some(|g| {
            t(0x190 + e(g));
            (tr() >> 2) & 1 != 0
        });
        h::FTLIB_GET_NAME_TAG_HEIGHT = Some(|g| {
            t(0x1A0 + e(g));
            (tr() % 777) as i32 as f32 / 3.0
        });
        h::FTLIB_GET_KIND = Some(|g| {
            t(0x1B0 + e(g));
            tr() % 40
        });
        h::FTLIB_GET_LAST_ATTACKER_SLOT = Some(|g| {
            t(0x1C0 + e(g));
            (tr() % 11) as i32
        });
        h::FT_80087AC0 = Some(|g, v| {
            t(0x1D0 + e(g));
            t(v as u32)
        });
        h::FTLIB_80086A4C = Some(|g, f| {
            t(0x1E0 + e(g));
            t(b(f))
        });
        h::FT_80087BAC = Some(|g, v| {
            t(0x1F0 + e(g));
            t(v as u32)
        });
        h::FTLIB_IS_CAMERA_SUBJECT_IN_BOUNDS = Some(|g| {
            t(0x200 + e(g));
            (tr() >> 6) & 1 != 0
        });
        h::FTLIB_GET_CAMERA_SUBJECT_BONE_POS = Some(|g, v| {
            t(0x210 + e(g));
            v.x = (tr() % 100) as i32 as f32;
            v.y = -1.0;
        });
    }
}

fn init_slot(p: &mut StaticPlayer) {
    p.player_state = (rnd() % 4) as i32;
    p.ckind = rnd() % 40;
    p.pkind = rnd() % 4;
    p.transformed[0] = (rnd() % 2) as u8;
    p.transformed[1] = (rnd() % 2) as u8;
    for i in 0..4 {
        let x = rf();
        let y = rf();
        let z = rf();
        unsafe { p.player_poses.byIndex[i] = Vec3 { x, y, z } };
    }
    p.facing_direction = rf();
    p.costume_id = rnd() as u8;
    p.unk45 = rnd() as u8;
    p.controller_index = rnd() as i8;
    p.team = (rnd() % 3) as u8;
    p.player_id = rnd() as u8;
    p.cpu_level = rnd() as u8;
    p.cpu_type = rnd() as u8;
    p.handicap = rnd() as u8;
    p.unk4C = rnd() as i8;
    p.unk4D = rnd() as i8;
    p.unk4E = rnd() as i8;
    p.unk50 = rf();
    p.attack_ratio = rf();
    p.defense_ratio = rf();
    p.model_scale = rf();
    for i in 0..4 {
        unsafe { p.staminas.byIndex[i] = rnd() as i16 };
    }
    for i in 0..2 {
        p.falls[i] = rnd() as i32;
    }
    for i in 0..6 {
        p.kos_by_player[i] = if rnd() % 4 == 0 { u32::MAX } else { rnd() };
    }
    p.match_frame_count = if rnd() % 3 == 0 { u32::MAX } else { rnd() };
    p.self_destructs = if rnd() % 4 == 0 { 0xFFFF } else { rnd() as u16 };
    p.stocks = ((rnd() % 7) as i32 - 2) as i8;
    p.current_coins = rnd() as i32;
    p.total_coins = rnd() as i32;
    p.unk98 = rnd() as i32;
    p.unk9C = rnd() as i32;
    p.joystick_direction_input_count[0] = rnd() as i32;
    p.joystick_direction_input_count[1] = rnd() as i32;
    p.nametag_slot_id = rnd() as i32;
    let f = &mut p.flags;
    f.set_b0((rnd() & 1) as u8);
    f.set_b1((rnd() & 1) as u8);
    f.set_b2((rnd() & 1) as u8);
    f.set_b3((rnd() & 1) as u8);
    f.set_b4((rnd() & 1) as u8);
    f.set_is_metal((rnd() & 1) as u8);
    f.set_b6((rnd() & 1) as u8);
    f.set_b7((rnd() & 1) as u8);
    let m = &mut p.more_flags;
    m.set_b0((rnd() & 1) as u8);
    m.set_b1((rnd() & 1) as u8);
    m.set_b2((rnd() & 1) as u8);
    m.set_b3((rnd() & 1) as u8);
    m.set_b4((rnd() & 1) as u8);
    m.set_b5((rnd() & 1) as u8);
    m.set_b6((rnd() & 3) as u8);
    p.flagsAE.set_b0((rnd() & 1) as u8);
    p.flagsAE.set_b1((rnd() & 1) as u8);
    p.player_entity[0] = rent();
    p.player_entity[1] = rent();
    p.struct_func = if rnd() & 1 != 0 { Some(sf) } else { None };
}

fn dump(out: &mut Vec<String>, slot: usize) {
    let p = unsafe { &PLAYER_SLOTS[slot] };
    let mut s = format!("S{} {:x} {:x} {:x} {:x} {:x}", slot, p.player_state as u32, p.ckind, p.pkind, p.transformed[0], p.transformed[1]);
    let poses = unsafe { p.player_poses.byIndex };
    for v in poses {
        s += &format!(" {:x} {:x} {:x}", b(v.x), b(v.y), b(v.z));
    }
    s += &format!(
        " {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
        b(p.facing_direction), p.costume_id, p.unk45, p.controller_index as i32 as u32, p.team, p.player_id,
        p.cpu_level, p.cpu_type, p.handicap, p.unk4C as i32 as u32, p.unk4D as i32 as u32, p.unk4E as i32 as u32
    );
    s += &format!(" {:x} {:x} {:x} {:x}", b(p.unk50), b(p.attack_ratio), b(p.defense_ratio), b(p.model_scale));
    for x in unsafe { p.staminas.byIndex } {
        s += &format!(" {:x}", x as i32 as u32);
    }
    s += &format!(" {:x} {:x}", p.falls[0] as u32, p.falls[1] as u32);
    for x in p.kos_by_player {
        s += &format!(" {:x}", x);
    }
    s += &format!(
        " {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
        p.match_frame_count, p.self_destructs, p.stocks as i32 as u32, p.current_coins as u32, p.total_coins as u32,
        p.unk98 as u32, p.unk9C as u32, p.joystick_direction_input_count[0] as u32,
        p.joystick_direction_input_count[1] as u32, p.nametag_slot_id as u32
    );
    let (f, m, a) = (&p.flags, &p.more_flags, &p.flagsAE);
    s += &format!(
        " f{}{}{}{}{}{}{}{} m{}{}{}{}{}{}{} a{}{}",
        f.b0(), f.b1(), f.b2(), f.b3(), f.b4(), f.is_metal(), f.b6(), f.b7(),
        m.b0(), m.b1(), m.b2(), m.b3(), m.b4(), m.b5(), m.b6(), a.b0(), a.b1()
    );
    s += &format!(" e{:x}{:x} {}", e(p.player_entity[0]), e(p.player_entity[1]), p.struct_func.is_some() as i32);
    out.push(s);
}

const NOPS: u32 = 144;

#[test]
fn player_matches_c() {
    install();
    let mut out = Vec::new();
    for k in 0..200u32 {
        unsafe {
            RS = 0x5EED1u32.wrapping_add(k.wrapping_mul(2654435761));
            TR = 0;
            pl::FT_MAPPING_LIST = pl::FT_MAPPING_INIT;
            for i in 0..6 {
                init_slot(&mut PLAYER_SLOTS[i]);
            }
        }
        for step in 0..60 {
            let slot = (rnd() % 6) as i32;
            let op = rnd() % NOPS;
            let mut ret: u32 = 0;
            let x = rf();
            let y = rf();
            let z = rf();
            let mut v = Vec3 { x, y, z };
            let base = unsafe { &raw const PLAYER_SLOTS[slot as usize] } as usize;
            match op {
                0 => pl::player_80031790(slot),
                1 => pl::player_80031848(slot),
                2 => pl::player_80031900(),
                3 => ret = pl::player_800319c4(slot, (rnd() % 3) as i32) as u32,
                4 => {
                    let a = rnd() as i32;
                    pl::player_80031da8(a, rnd() as i32)
                }
                5 => pl::player_80031dc8(|a, c| {
                    t(0x230);
                    t(a as u32);
                    t(c as u32)
                }),
                6 => pl::player_80031ebc(slot),
                7 => pl::player_80031fb0(slot, (rnd() % 2) as i32),
                8 => pl::player_80032070(slot, (rnd() % 2) as i32),
                9 => ret = pl::player_8003219c(slot) as u32,
                10 => ret = pl::player_8003221c(slot) as u32,
                11 => ret = pl::get_player_state(slot) as u32,
                12 => ret = pl::get_player_character(slot),
                13 => pl::set_player_character(slot, rnd() % 40),
                14 => ret = pl::player_8003248c(slot, (rnd() % 3) as i32),
                15 => {
                    let c = rnd() % 40;
                    ret = pl::player_800325c8(c, (rnd() % 3) as i32) as i32 as u32
                }
                16 => ret = pl::player_80032610(slot, (rnd() % 3) as i32) as i32 as u32,
                17 => v = pl::load_player_coords(slot),
                18 => pl::player_80032768(slot, &v),
                19 => pl::player_80032828(slot, (rnd() % 2) as i32, &v),
                20 => pl::player_800328d4(slot, &v),
                21 => pl::player_80032a04(slot, &v),
                22 => pl::set_player_and_entity_facing_direction(slot, rf()),
                23 => ret = b(pl::player_80032bb0(slot)),
                24 => pl::set_scale(slot, rf()),
                25 => v = pl::get_spawn_platform_pos(slot),
                26 => pl::set_spawn_platform_pos(slot, &v),
                27 => v = pl::get_some_pos(slot),
                28 => pl::set_some_pos(slot, &v),
                29 => ret = pl::player_80032f30(slot) as u32,
                30 => pl::player_80032fa4(slot, rnd() as i32),
                31 => ret = b(pl::get_facing_direction(slot)),
                32 => pl::set_facing_direction(slot, rf()),
                33 => {
                    let c = (rnd() % 3) as i32;
                    pl::set_facing_direction_conditional(slot, c, rf())
                }
                34 => ret = pl::get_costume_id(slot),
                35 => pl::set_costume_id(slot, rnd() as i32),
                36 => ret = pl::get_controller_index(slot) as u32,
                37 => pl::set_controller_index(slot, rnd() as i8),
                38 => ret = pl::get_team(slot) as u32,
                39 => pl::set_team(slot, rnd() as i8),
                40 => ret = pl::get_player_id(slot) as u32,
                41 => pl::set_player_id(slot, rnd() as i32),
                42 => ret = pl::get_cpu_level(slot) as u32,
                43 => pl::set_player_and_entity_cpu_level(slot, rnd() as i32),
                44 => ret = pl::get_cpu_type(slot) as u32,
                45 => pl::set_player_and_entity_cpu_type(slot, rnd() as i32),
                46 => ret = pl::get_handicap(slot) as u32,
                47 => pl::set_handicap(slot, rnd() as i8),
                48 => ret = b(pl::get_unk50(slot)),
                49 => ret = b(pl::get_attack_ratio(slot)),
                50 => pl::set_attack_ratio(slot, rf()),
                51 => ret = b(pl::get_defense_ratio(slot)),
                52 => pl::set_defense_ratio(slot, rf()),
                53 => ret = b(pl::get_model_scale(slot)),
                54 => pl::set_model_scale(slot, rf()),
                55 => ret = pl::player_80033bb8(slot) as u32,
                56 => ret = pl::get_p1_stock() as u32,
                57 => pl::lose_stock(slot),
                58 => ret = pl::get_coins(slot) as u32,
                59 => pl::set_coins(slot, rnd() as i32),
                60 => ret = pl::get_total_coins(slot) as u32,
                61 => pl::set_total_coins(slot, rnd() as i32),
                62 => ret = pl::get_unk98(slot) as u32,
                63 => pl::set_unk98(slot, rnd() as i32),
                64 => ret = pl::get_unk9c(slot) as u32,
                65 => pl::set_unk9c(slot, rnd() as i32),
                66 => ret = e(pl::get_entity(slot)),
                67 => ret = e(pl::get_entity_at_index(slot, (rnd() % 2) as i32)),
                68 => {
                    let a = (rnd() % 2) as i32;
                    pl::swap_transformed_states(slot, a, (rnd() % 2) as i32)
                }
                69 => ret = pl::get_damage(slot) as u32,
                70 => pl::set_hud_damage(slot, rnd() as i32),
                71 => {
                    let a = (rnd() % 2) as i32;
                    pl::set_hp_by_index(slot, a, rnd() as i32)
                }
                72 => ret = pl::get_other_stamina(slot) as u32,
                73 => pl::set_other_stamina(slot, rnd() as i32),
                74 => ret = pl::get_remaining_hp(slot) as u32,
                75 => ret = pl::get_remaining_hp_by_index(slot, (rnd() % 2) as i32) as u32,
                76 => ret = pl::get_more_flags_bit2(slot) as u32,
                77 => pl::set_more_flags_bit2(slot, rnd() as u8),
                78 => ret = pl::get_more_flags_bit3(slot) as u32,
                79 => pl::set_more_flags_bit3(slot, rnd() as u8),
                80 => ret = pl::get_more_flags_bit4(slot) as u32,
                81 => pl::set_more_flags_bit4(slot, rnd() as u8),
                82 => ret = pl::get_more_flags_bit5(slot) as u32,
                83 => pl::set_more_flags_bit5(slot, rnd() as u8),
                84 => ret = pl::get_more_flags_bit6(slot) as u32,
                85 => pl::set_more_flags_bit6(slot, rnd() as u8),
                86 => ret = pl::get_flags_ae_bit0(slot) as u32,
                87 => pl::set_flags_ae_bit0(slot, rnd() as u8),
                88 => ret = pl::get_falls(slot) as u32,
                89 => ret = pl::get_falls_by_index(slot, (rnd() % 2) as i32) as u32,
                90 => pl::set_falls(slot, rnd() as i32),
                91 => {
                    let a = (rnd() % 2) as i32;
                    pl::set_falls_by_index(slot, a, rnd() as i32)
                }
                92 => ret = pl::get_kos_by_player_index(slot, (rnd() % 6) as i32) as u32,
                93 => {
                    let c = (rnd() % 3 == 0) as i32;
                    pl::update_kos_by_slot(slot, c, (rnd() % 6) as i32)
                }
                94 => ret = pl::get_match_frame_count(slot),
                95 => pl::update_match_frame_count(slot, (rnd() % 3 == 0) as i32),
                96 => ret = pl::get_self_destructs(slot),
                97 => {
                    let x = if rnd() % 3 == 0 { rnd() } else { rnd() % 0x10002 };
                    pl::set_self_destructs(slot, x)
                }
                98 => pl::inc_self_destructs(slot, (rnd() % 3 == 0) as i32),
                99 => ret = pl::player_800353bc(slot) as u32,
                100 => ret = pl::player_8003544c(slot, (rnd() % 3 == 0) as i32) as u32,
                101 => pl::set_flags_bit0(slot, (rnd() % 3) as i32),
                102 => ret = pl::get_nametag_slot_id(slot) as u32,
                103 => pl::set_nametag_slot_id(slot, rnd() as i32),
                104 => ret = pl::get_flags_bit1(slot) as u32,
                105 => pl::set_flags_bit1(slot),
                106 => pl::unset_flags_bit1(slot),
                107 => ret = pl::get_flags_bit3(slot) as u32,
                108 => pl::set_flags_bit3(slot, rnd() as u8),
                109 => ret = pl::get_flags_bit4(slot) as u32,
                110 => ret = pl::get_flags_bit5(slot) as u32,
                111 => pl::set_flags_bit5(slot, rnd() as u8),
                112 => ret = pl::get_flags_bit6(slot) as u32,
                113 => pl::set_flags_bit6(slot, rnd() as u8),
                114 => ret = pl::get_flags_bit7(slot) as u32,
                115 => pl::set_flags_bit7(slot, rnd() as u8),
                116 => ret = pl::get_more_flags_bit0(slot) as u32,
                117 => ret = pl::get_more_flags_bit1(slot) as u32,
                118 => pl::set_more_flags_bit1(slot, rnd() as u8),
                119 => ret = pl::get_unk4d(slot) as u32,
                120 => pl::set_unk4d(slot, rnd() as i8),
                121 => ret = pl::get_flags_ae_bit1(slot) as u32,
                122 => pl::set_flags_ae_bit1(slot, rnd() as u8),
                123 => ret = pl::get_unk4c(slot) as u32,
                124 => pl::set_unk4c(slot, rnd() as u8),
                125 => ret = pl::player_80036058(slot) as u32,
                126 => ret = b(pl::player_800360d8(slot)),
                127 => pl::set_struct_func(slot, if rnd() % 2 != 0 { Some(sf) } else { None }),
                128 => ret = (pl::get_unk6a8_ptr(slot) as *mut pl_x5EC_t as usize - base) as u32,
                129 => ret = (pl::get_stale_move_table_index_ptr2(slot) as usize - base) as u32,
                130 => ret = pl::player_80036394(slot),
                131 => ret = pl::player_80036428(slot) as u32,
                132 => pl::set_unk45(slot, rnd() as i32),
                133 => ret = pl::get_unk45(slot),
                134 => pl::update_joystick_count_by_index(slot, (rnd() % 2) as i32),
                135 => ret = pl::get_joystick_count_by_index(slot, (rnd() % 2) as i32) as u32,
                136 => pl::player_800366dc(slot, rnd() as i32),
                137 => pl::player_80036790(slot, rf()),
                138 => pl::player_80036844(slot, rnd() as i32),
                139 => ret = pl::player_800368f8(slot) as u32,
                140 => pl::player_80036978(slot, &mut v),
                141 => {
                    let c = (rnd() % 40) as i32;
                    pl::mex_set_mapping(c, (rnd() % 300) as i32)
                }
                142 => {
                    let c = (rnd() % 40) as i32;
                    pl::mex_set_partner(c, (rnd() % 300) as i32)
                }
                143 => pl::forget_entities(),
                _ => unreachable!(),
            }
            out.push(format!("{} {} {} {} {:x} {:x} {:x} {:x} {:x}", k, step, op, slot, ret, b(v.x), b(v.y), b(v.z), tr()));
            dump(&mut out, slot as usize);
        }
        for i in 0..6 {
            dump(&mut out, i);
        }
        for i in 0..40 {
            let r = unsafe { pl::FT_MAPPING_LIST[i] };
            out.push(format!(
                "M{} {:x} {:x} {:x}",
                i, r.internal_id as i32 as u32, r.extra_internal_id as i32 as u32, r.has_transformation as i32 as u32
            ));
        }
    }
    let want = include_str!("data/player.txt");
    let want: Vec<&str> = want.lines().collect();
    assert_eq!(want.len(), out.len(), "line count");
    let mut ops_seen = [0u32; NOPS as usize];
    for (i, (w, g)) in want.iter().zip(&out).enumerate() {
        assert_eq!(*w, g.as_str(), "line {}", i + 1);
        if !g.starts_with('S') && !g.starts_with('M') {
            ops_seen[g.split(' ').nth(2).unwrap().parse::<usize>().unwrap()] += 1;
        }
    }
    assert!(ops_seen.iter().all(|&n| n > 0), "every op exercised: {ops_seen:?}");
}
