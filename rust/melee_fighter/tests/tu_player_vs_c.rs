//! src/tu/player.rs (the C-named whole-TU port) against the C (gen/cref/player_ref.c, the player.c accessors cut verbatim):
//! the same xorshift-driven random player tables and op sequences, the same tracing stubs for
//! every call out of player.c, every printed line compared (tests/data/player.txt).
#![allow(static_mut_refs)]

use melee_fighter::hooks as h;
use melee_fighter::player as pl;
use melee_fighter::tu::player as m;
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
unsafe extern "C" fn cb230(a: i32, c: i32) { t(0x230); t(a as u32); t(c as u32) }
unsafe extern "C" fn sf(_a: i32) {}

fn install() {
    unsafe {
        h::SCRIPT_MOD_VALUE = Some(|s, f, base| {
            t(0x220 + s as u32 * 2 + f as u32);
            t(b(base));
            base
        });
        m::ext::hook::ftLib_DisableInput = Some(|g| t(0x10 + e(g)));
        m::ext::hook::ftLib_EnableInput = Some(|g| t(0x20 + e(g)));
        m::ext::hook::ftKb_SpecialN_800EED50 = Some(|a, c| {
            t(0x30);
            t(a as u32);
            t(c as u32)
        });
        m::ext::hook::ftLib_IsSleeping = Some(|g| {
            t(0x40 + e(g));
            (tr() >> 3) & 1 != 0
        });
        m::ext::hook::ftCo_800D4F24 = Some(|g, i| {
            t(0x50 + e(g));
            t(i as u32)
        });
        m::ext::hook::HSD_GObjFree = Some(|g| t(0x60 + e(g)));
        m::ext::hook::ftCo_800D4FF4 = Some(|g| t(0x70 + e(g)));
        m::ext::hook::GenoDefine_InitKinds = Some(|| t(0x240));
        m::ext::hook::ftData_800855C8 = Some(|k, c| {
            t(0x250 + k as u32);
            t(c as u32)
        });
        m::ext::hook::ifStatus_802F6E1C = Some(|s| t(0x80 + s as u32));
        m::ext::hook::ftLib_IsRebirth = Some(|g| {
            t(0x90 + e(g));
            (tr() >> 5) & 1 != 0
        });
        m::ext::hook::ftLib_80087140 = Some(|g| t(0xA0 + e(g)));
        m::ext::hook::ftLib_SetPos = Some(|g, v| {
            t(0xB0 + e(g));
            t(b((*v).x));
            t(b((*v).y));
            t(b((*v).z))
        });
        m::ext::hook::ftDemo_SetFacingDirection = Some(|g, f| {
            t(0xC0 + e(g));
            t(b(f))
        });
        m::ext::hook::ftLib_800869F8 = Some(|g| {
            t(0xD0 + e(g));
            (tr() % 1000) as i32 as f32 / 7.0
        });
        m::ext::hook::ftLib_SetScale = Some(|g, f| {
            t(0xE0 + e(g));
            t(b(f))
        });
        m::ext::hook::ft_80087BEC = Some(|g, v| {
            t(0xF0 + e(g));
            t(v as u32)
        });
        m::ext::hook::ft_80087BC0 = Some(|g, v| {
            t(0x100 + e(g));
            t(v as u32)
        });
        m::ext::hook::gm_8016C5C0 = Some(|s| {
            t(0x110 + s as u32);
            (tr() % 97) as i32
        });
        m::ext::hook::ftLib_SetPercent = Some(|g, v| {
            t(0x120 + e(g));
            t(v as u32)
        });
        m::ext::hook::ifStatus_802F6D10 = Some(|s| t(0x130 + s as u32));
        m::ext::hook::gm_8016B168 = Some(|| {
            t(0x140);
            (tr() >> 4) & 1 != 0
        });
        m::ext::hook::ifStatus_802F69C0 = Some(|a, c| {
            t(0x150);
            t(a as u32);
            t(c as u32)
        });
        m::ext::hook::ifStatus_802F6AF8 = Some(|s| t(0x160 + s as u32));
        m::ext::hook::gm_GetFrameCount = Some(|| {
            t(0x170);
            tr()
        });
        m::ext::hook::ifStatus_802F6C04 = Some(|s| t(0x180 + s as u32));
        m::ext::hook::ftLib_80086F4C = Some(|g| {
            t(0x190 + e(g));
            (tr() >> 2) & 1 != 0
        });
        m::ext::hook::ftLib_GetNameTagHeight = Some(|g| {
            t(0x1A0 + e(g));
            (tr() % 777) as i32 as f32 / 3.0
        });
        m::ext::hook::ftLib_GetKind = Some(|g| {
            t(0x1B0 + e(g));
            tr() % 40
        });
        m::ext::hook::ftLib_GetLastAttackerSlot = Some(|g| {
            t(0x1C0 + e(g));
            (tr() % 11) as i32
        });
        m::ext::hook::ft_80087AC0 = Some(|g, v| {
            t(0x1D0 + e(g));
            t(v as u32)
        });
        m::ext::hook::ftLib_80086A4C = Some(|g, f| {
            t(0x1E0 + e(g));
            t(b(f))
        });
        m::ext::hook::ft_80087BAC = Some(|g, v| {
            t(0x1F0 + e(g));
            t(v as u32)
        });
        m::ext::hook::ftLib_IsCameraSubjectInBounds = Some(|g| {
            t(0x200 + e(g));
            (tr() >> 6) & 1 != 0
        });
        m::ext::hook::ftLib_GetCameraSubjectBonePos = Some(|g, v| {
            t(0x210 + e(g));
            (*v).x = (tr() % 100) as i32 as f32;
            (*v).y = -1.0;
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

const NOPS: u32 = 147;

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
                0 => m::Player_80031790(slot),
                1 => m::Player_80031848(slot),
                2 => m::Player_80031900(),
                3 => ret = m::Player_800319C4(slot, rnd() % 3 != 0) as u32,
                4 => {
                    let a = rnd() as i32;
                    m::Player_80031DA8(a, rnd() as i32)
                }
                5 => m::Player_80031DC8(cb230),




                6 => m::Player_80031EBC(slot),
                7 => m::Player_80031FB0(slot, (rnd() % 2) as i32),
                8 => m::Player_80032070(slot, rnd() % 2 != 0),
                9 => ret = m::Player_8003219C(slot) as u32,
                10 => ret = m::Player_8003221C(slot) as u32,
                11 => ret = m::Player_GetPlayerState(slot) as u32,
                12 => ret = m::Player_GetPlayerCharacter(slot),
                13 => m::Player_SetPlayerCharacter(slot, rnd() % 40),
                14 => ret = m::Player_8003248C(slot, (rnd() % 3) as i32),
                15 => {
                    let c = rnd() % 40;
                    ret = m::Player_800325C8(c, (rnd() % 3) as i32) as i32 as u32
                }
                16 => ret = m::Player_80032610(slot, (rnd() % 3) as i32) as i32 as u32,
                17 => m::Player_LoadPlayerCoords(slot, &mut v),
                18 => m::Player_80032768(slot, &mut v),
                19 => m::Player_80032828(slot, (rnd() % 2) as i32, &mut v),
                20 => m::Player_800328D4(slot, &mut v),
                21 => m::Player_80032A04(slot, &mut v),
                22 => m::Player_SetPlayerAndEntityFacingDirection(slot, rf()),
                23 => ret = b(m::Player_80032BB0(slot)),
                24 => m::Player_SetScale(slot, rf()),
                25 => m::Player_GetSpawnPlatformPos(slot, &mut v),
                26 => m::Player_SetSpawnPlatformPos(slot, &mut v),
                27 => m::Player_GetSomePos(slot, &mut v),
                28 => m::Player_SetSomePos(slot, &mut v),
                29 => ret = m::Player_80032F30(slot) as u32,
                30 => m::Player_80032FA4(slot, rnd() as i32),
                31 => ret = b(m::Player_GetFacingDirection(slot)),
                32 => m::Player_SetFacingDirection(slot, rf()),
                33 => {
                    let c = rnd() % 3 != 0;
                    m::Player_SetFacingDirectionConditional(slot, c, rf())
                }
                34 => ret = m::Player_GetCostumeId(slot),
                35 => m::Player_SetCostumeId(slot, rnd() as i32),
                36 => ret = m::Player_GetControllerIndex(slot) as u32,
                37 => m::Player_SetControllerIndex(slot, rnd() as i8),
                38 => ret = m::Player_GetTeam(slot) as u32,
                39 => m::Player_SetTeam(slot, rnd() as i8),
                40 => ret = m::Player_GetPlayerId(slot) as u32,
                41 => m::Player_SetPlayerId(slot, rnd() as i32),
                42 => ret = m::Player_GetCpuLevel(slot) as u32,
                43 => m::Player_SetPlayerAndEntityCpuLevel(slot, rnd() as i32),
                44 => ret = m::Player_GetCpuType(slot) as u32,
                45 => m::Player_SetPlayerAndEntityCpuType(slot, rnd() as i32),
                46 => ret = m::Player_GetHandicap(slot) as u32,
                47 => m::Player_SetHandicap(slot, rnd() as i8),
                48 => ret = b(m::Player_GetUnk50(slot)),
                49 => ret = b(m::Player_GetAttackRatio(slot)),
                50 => m::Player_SetAttackRatio(slot, rf()),
                51 => ret = b(m::Player_GetDefenseRatio(slot)),
                52 => m::Player_SetDefenseRatio(slot, rf()),
                53 => ret = b(m::Player_GetModelScale(slot)),
                54 => m::Player_SetModelScale(slot, rf()),
                55 => ret = m::Player_80033BB8(slot) as u32,
                56 => ret = m::Player_GetP1Stock() as u32,
                57 => m::Player_LoseStock(slot),
                58 => ret = m::Player_GetCoins(slot) as u32,
                59 => m::Player_SetCoins(slot, rnd() as i32),
                60 => ret = m::Player_GetTotalCoins(slot) as u32,
                61 => m::Player_SetTotalCoins(slot, rnd() as i32),
                62 => ret = m::Player_GetUnk98(slot) as u32,
                63 => m::Player_SetUnk98(slot, rnd() as i32),
                64 => ret = m::Player_GetUnk9C(slot) as u32,
                65 => m::Player_SetUnk9C(slot, rnd() as i32),
                66 => ret = e(m::Player_GetEntity(slot)),
                67 => ret = e(m::Player_GetEntityAtIndex(slot, (rnd() % 2) as i32)),
                68 => {
                    let a = (rnd() % 2) as i32;
                    m::Player_SwapTransformedStates(slot, a, (rnd() % 2) as i32)
                }
                69 => ret = m::Player_GetDamage(slot) as u32,
                70 => m::Player_SetHUDDamage(slot, rnd() as i32),
                71 => {
                    let a = (rnd() % 2) as i32;
                    m::Player_SetHPByIndex(slot, a, rnd() as i32)
                }
                72 => ret = m::Player_GetOtherStamina(slot) as u32,
                73 => m::Player_SetOtherStamina(slot, rnd() as i32),
                74 => ret = m::Player_GetRemainingHP(slot) as u32,
                75 => ret = m::Player_GetRemainingHPByIndex(slot, (rnd() % 2) as i32) as u32,
                76 => ret = m::Player_GetMoreFlagsBit2(slot) as u32,
                77 => m::Player_SetMoreFlagsBit2(slot, rnd() as u8),
                78 => ret = m::Player_GetMoreFlagsBit3(slot) as u32,
                79 => m::Player_SetMoreFlagsBit3(slot, rnd() as u8),
                80 => ret = m::Player_GetMoreFlagsBit4(slot) as u32,
                81 => m::Player_SetMoreFlagsBit4(slot, rnd() as u8),
                82 => ret = m::Player_GetMoreFlagsBit5(slot) as u32,
                83 => m::Player_SetMoreFlagsBit5(slot, rnd() as u8),
                84 => ret = m::Player_GetMoreFlagsBit6(slot) as u32,
                85 => m::Player_SetMoreFlagsBit6(slot, rnd() as u8),
                86 => ret = m::Player_GetFlagsAEBit0(slot) as u32,
                87 => m::Player_SetFlagsAEBit0(slot, rnd() as u8),
                88 => ret = m::Player_GetFalls(slot) as u32,
                89 => ret = m::Player_GetFallsByIndex(slot, (rnd() % 2) as i32) as u32,
                90 => m::Player_SetFalls(slot, rnd() as i32),
                91 => {
                    let a = (rnd() % 2) as i32;
                    m::Player_SetFallsByIndex(slot, a, rnd() as i32)
                }
                92 => ret = m::Player_GetKOsByPlayerIndex(slot, (rnd() % 6) as i32) as u32,
                93 => {
                    let c = rnd() % 3 == 0;
                    m::Player_UpdateKOsBySlot(slot, c, (rnd() % 6) as i32)
                }
                94 => ret = m::Player_GetMatchFrameCount(slot),
                95 => m::Player_UpdateMatchFrameCount(slot, rnd() % 3 == 0),
                96 => ret = m::Player_GetSelfDestructs(slot),
                97 => {
                    let x = if rnd() % 3 == 0 { rnd() } else { rnd() % 0x10002 };
                    m::Player_SetSelfDestructs(slot, x)
                }
                98 => m::Player_IncSelfDestructs(slot, (rnd() % 3 == 0) as i32),
                99 => ret = m::Player_800353BC(slot) as u32,
                100 => ret = m::Player_8003544C(slot, rnd() % 3 == 0) as u32,
                101 => m::Player_SetFlagsBit0(slot, (rnd() % 3) as i32),
                102 => ret = m::Player_GetNametagSlotID(slot) as u32,
                103 => m::Player_SetNametagSlotID(slot, rnd() as i32),
                104 => ret = m::Player_GetFlagsBit1(slot) as u32,
                105 => m::Player_SetFlagsBit1(slot),
                106 => m::Player_UnsetFlagsBit1(slot),
                107 => ret = m::Player_GetFlagsBit3(slot) as u32,
                108 => m::Player_SetFlagsBit3(slot, rnd() as u8),
                109 => ret = m::Player_GetFlagsBit4(slot) as u32,
                110 => ret = m::Player_GetFlagsBit5(slot) as u32,
                111 => m::Player_SetFlagsBit5(slot, rnd() as u8),
                112 => ret = m::Player_GetFlagsBit6(slot) as u32,
                113 => m::Player_SetFlagsBit6(slot, rnd() as u8),
                114 => ret = m::Player_GetFlagsBit7(slot) as u32,
                115 => m::Player_SetFlagsBit7(slot, rnd() as u8),
                116 => ret = m::Player_GetMoreFlagsBit0(slot) as u32,
                117 => ret = m::Player_GetMoreFlagsBit1(slot) as u32,
                118 => m::Player_SetMoreFlagsBit1(slot, rnd() as u8),
                119 => ret = m::Player_GetUnk4D(slot) as u32,
                120 => m::Player_SetUnk4D(slot, rnd() as i8),
                121 => ret = m::Player_GetFlagsAEBit1(slot) as u32,
                122 => m::Player_SetFlagsAEBit1(slot, rnd() as u8),
                123 => ret = m::Player_GetUnk4C(slot) as u32,
                124 => m::Player_SetUnk4C(slot, rnd() as u8),
                125 => ret = m::Player_80036058(slot) as u32,
                126 => ret = b(m::Player_800360D8(slot)),
                127 => m::Player_SetStructFunc(slot, if rnd() % 2 != 0 { sf as *mut core::ffi::c_void } else { core::ptr::null_mut() }),
                128 => ret = (m::Player_GetUnk6A8Ptr(slot) as *mut pl_x5EC_t as usize - base) as u32,
                129 => ret = (m::Player_GetStaleMoveTableIndexPtr2(slot) as usize - base) as u32,
                130 => ret = m::Player_80036394(slot),
                131 => ret = m::Player_80036428(slot) as u32,
                132 => m::Player_SetUnk45(slot, rnd() as i32),
                133 => ret = m::Player_GetUnk45(slot),
                134 => m::Player_UpdateJoystickCountByIndex(slot, (rnd() % 2) as i32),
                135 => ret = m::Player_GetJoystickCountByIndex(slot, (rnd() % 2) as i32) as u32,
                136 => m::Player_800366DC(slot, rnd() as i32),
                137 => m::Player_80036790(slot, rf()),
                138 => m::Player_80036844(slot, rnd() as i32),
                139 => ret = m::Player_800368F8(slot) as u32,
                140 => m::Player_80036978(slot, &mut v),
                141 => {
                    let c = (rnd() % 40) as i32;
                    m::Player_MexSetMapping(c, (rnd() % 300) as i32)
                }
                142 => {
                    let c = (rnd() % 40) as i32;
                    m::Player_MexSetPartner(c, (rnd() % 300) as i32)
                }
                143 => m::Player_ForgetEntities(),
                144 => {
                    let c = rnd() % 40;
                    m::Player_80031CB0(c, rnd() as u8)
                }
                145 => ret = (m::Player_GetStaleMoveTableIndexPtr(slot) as usize - base) as u32,
                146 => m::Player_InitOrResetPlayer(slot),
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
    let want = include_str!("data/player2.txt");
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
