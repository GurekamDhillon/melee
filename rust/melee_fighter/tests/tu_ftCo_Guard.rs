//! src/tu/ftCo_Guard.rs vs the C (gen/cref/tu/ftCo_Guard.c: the whole TU, TARGET_PC, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Guard`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_Guard as m;
use melee_fighter::types::*;
use std::vec::Vec;
use m::ext::hook as hk;

static mut MV: f32 = 0.0;
static mut ENDF: f32 = 0.0;
static mut PARTS: *mut FighterBone = core::ptr::null_mut();
static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut JO2: *mut HSD_JObj = core::ptr::null_mut();
static mut SKEL: *mut HSD_JObj = core::ptr::null_mut();
static mut CJ: *mut HSD_Joint = core::ptr::null_mut();
static mut CJC: *mut HSD_Joint = core::ptr::null_mut();
static mut JPOSE: *mut HSD_Joint = core::ptr::null_mut();
static mut OWN: *mut HSD_Joint = core::ptr::null_mut();
static mut JTAB: *mut *mut HSD_Joint = core::ptr::null_mut();
static mut FD: *mut ftData = core::ptr::null_mut();
static mut FD8: *mut ftData_x8 = core::ptr::null_mut();
static mut FD20: *mut ftData_x20 = core::ptr::null_mut();
static mut MS: *mut MotionState = core::ptr::null_mut();
static mut COL: [u8; 64] = [0; 64];
static mut FIGA: u32 = 0;
static mut POS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

fn isf(p: &mut Fighter) -> u32 { (p as *mut _ == unsafe { F }) as u32 }
fn figa() -> *mut FigaTree { unsafe { &mut FIGA as *mut u32 as *mut FigaTree } }
fn jo_idx(j: *mut HSD_JObj) -> u32 { unsafe { j.offset_from(JO) as i32 as u32 } }
fn leak<T>(n: usize, z: impl Fn() -> T) -> *mut T {
    Box::leak((0..n).map(|_| z()).collect::<Vec<T>>().into_boxed_slice()).as_mut_ptr()
}
fn tj(mut j: *mut HSD_Joint) {
    unsafe {
        while !j.is_null() {
            let r = &*j;
            t(100); t(b(r.rotation.x)); t(b(r.rotation.y)); t(b(r.rotation.z));
            t(b(r.scale.x)); t(b(r.scale.y)); t(b(r.scale.z));
            t(b(r.position.x)); t(b(r.position.y)); t(b(r.position.z)); t(r.flags);
            tj(r.child);
            t(101);
            j = r.next;
        }
    }
}
fn cbeq(cb: HSD_GObjEvent, f: unsafe extern "C" fn(*mut HSD_GObj)) -> u32 {
    (cb.map(|c| c as usize) == Some(f as usize)) as u32
}

#[test]
fn ftCo_Guard_matches_c() {
    init();
    unsafe {
        PARTS = leak(8, FighterBone::zeroed);
        JO = leak(8, HSD_JObj::zeroed);
        JO2 = leak(8, HSD_JObj::zeroed);
        SKEL = leak(1, HSD_JObj::zeroed);
        CJ = leak(1, HSD_Joint::zeroed);
        CJC = leak(1, HSD_Joint::zeroed);
        JPOSE = leak(1, HSD_Joint::zeroed);
        OWN = leak(3, HSD_Joint::zeroed);
        JTAB = leak(4, || core::ptr::null_mut());
        FD = leak(1, ftData::zeroed);
        FD8 = leak(1, ftData_x8::zeroed);
        FD20 = leak(1, ftData_x20::zeroed);
        MS = leak(ftCo_MS_Guard as usize + 1, MotionState::zeroed);
        let pt = leak(1, FighterPartsTable::zeroed);
        (*pt).parts_num = 8;
        melee_fighter::hooks::FT_PARTS_TABLE = leak(Ft_Kind_Max as usize, || pt);

        hk::ScriptGame_CapsActionAllowed = Some(|p, fl| { t(1); t(isf(p)); t(fl as u32); (sr(6) | sr(7)) as i32 });
        hk::ScriptGame_ModValue = Some(|s, fl, bs| { t(2); t(s as u32); t(fl as u32); t(b(bs)); MV });
        hk::lb_8000D008 = Some(|y, x| { t(3); t(b(y)); t(b(x)); rg() });
        hk::HSD_JObjSetMtxDirtySub = Some(|j| { t(4); t(jo_idx(j)) });
        hk::efLib_SetParamAlpha = Some(|g, a| { t(5); t(is_go(g)); t(a as u32) });
        hk::GenoDefine_RestPoseTree = Some(|p| { t(6); t(isf(p)); if sr(8) { OWN } else { core::ptr::null_mut() } });
        hk::Geno_MotionAnimRow = Some(|p, mo| { t(7); t(isf(p)); t(mo as u32); (SRV[9] % 40) as i32 - 10 });
        hk::ftData_80085E50 = Some(|p, mo| { t(8); t(isf(p)); t(mo as u32); if sr(10) { figa() } else { core::ptr::null_mut() } });
        hk::ftAnim_8006F4C8 = Some(|p, bb, tr| { t(9); t(isf(p)); t(bb as u32); t((tr == figa()) as u32) });
        hk::ftAnim_80070710 = Some(|j, fr| { t(10); t((j == SKEL) as u32); t(b(fr)) });
        hk::ftAnim_8006FB88 = Some(|p, pa, j| { t(11); t(isf(p)); t(pa); t((j == CJC) as u32); t((j == OWN) as u32) });
        hk::HSD_JObjAnimAll = Some(|j| { t(12); t((j == SKEL) as u32) });
        hk::ftAnim_80070108 = Some(|p, pa, a, bb, j| { t(13); t(isf(p)); t(pa); t(b(a)); t(b(bb)); t((j == JPOSE) as u32); tj(j) });
        hk::ftAnim_8006FE9C = Some(|p, pa, a, bb| { t(14); t(isf(p)); t(pa); t(b(a)); t(b(bb)) });
        hk::ftAnim_8006FF74 = Some(|p, pa| { t(15); t(isf(p)); t(pa) });
        hk::ftAnim_80070010 = Some(|p, pa, a, bb, j| { t(16); t(isf(p)); t(pa); t(b(a)); t(b(bb)); t((j == JPOSE) as u32); tj(j) });
        hk::ftAnim_8006FA58 = Some(|p, pa, j| { t(17); t(isf(p)); t(pa); t((j == JPOSE) as u32); tj(j) });
        hk::Player_GetUnk45 = Some(|s| { t(18); t(s as u32); SRV[11] % 4 });
        hk::Fighter_804D650C = Some(|| COL.as_mut_ptr());
        hk::efSync_Spawn = Some(|id, g, j, c| { t(19); t(id as u32); t(is_go(g)); t(jo_idx(j)); t(c as u32) });
        hk::ftColl_8007B1B8 = Some(|g, a, cb| {
            t(20); t(is_go(g)); t(a.x0_bone_id as u32); t(b(a.x4_offset.x)); t(b(a.x4_offset.y)); t(b(a.x4_offset.z)); t(b(a.x10_size));
            t(cbeq(cb, m::ftCo_80092E50));
        });
        hk::ft_PlaySFX = Some(|p, id, a, bb| { t(21); t(isf(p)); t(id as u32); t(a as u32); t(bb as u32) });
        hk::Geno_DefineTemplateSkeleton = Some(|k| { t(22); t(k as u32); sr(12) as i32 });
        hk::Geno_DefineBaseKind = Some(|k| { t(23); t(k as u32); if sr(13) { Ft_Kind_Yoshi as i32 } else if sr(14) { Ft_Kind_Mars as i32 } else { 5 } });
        hk::ftYs_Init_8012BECC = Some(|g| { t(24); t(is_go(g)) });
        hk::ftYs_Shield_8012C1D4 = Some(|g| { t(25); t(is_go(g)) });
        hk::ftYs_Shield_8012C49C = Some(|g| { t(26); t(is_go(g)) });
        hk::ftYs_Shield_8012C600 = Some(|g, bb| { t(27); t(is_go(g)); t(bb as u32) });
        hk::ftYs_Shield_8012C850 = Some(|g| { t(28); t(is_go(g)) });
        hk::ftYs_Shield_8012C914 = Some(|g| { t(29); t(is_go(g)) });
        hk::ftParts_80074B0C = Some(|g, mo, v| { t(30); t(is_go(g)); t(mo as u32); t(v as u32) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| { t(31); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32) });
        hk::ftAnim_8006EBA4 = Some(|g| { t(32); t(is_go(g)) });
        hk::pl_8003E0E8 = Some(|a, bb| { t(33); t(a as u32); t(bb as u32) });
        hk::pl_8003E114 = Some(|a, bb, v| { t(34); t(a as u32); t(bb as u32); t(b(v)) });
        hk::ftCo_80098B20 = Some(|g| { t(35); t(is_go(g)) });
        hk::ftCo_8009515C = Some(|_| { t(36); sr(16) });
        hk::ftCo_8009980C = Some(|_| { t(37); sr(17) });
        hk::ftCo_8009917C = Some(|_| { t(38); sr(18) });
        hk::ftCo_800D8B9C = Some(|_| { t(39); sr(19) });
        hk::ftCo_Catch_CheckInput = Some(|_| { t(40); sr(20) });
        hk::ftCo_800CB024 = Some(|_| { t(41); sr(21) });
        hk::ftCo_8009A080 = Some(|_| { t(42); sr(22) });
        hk::ft_80084F3C = Some(|g| { t(43); t(is_go(g)) });
        hk::ftColl_8007AEE0 = Some(|g| { t(44); t(is_go(g)) });
        hk::ftColl_8007AEF8 = Some(|g| { t(45); t(is_go(g)) });
        hk::ft_800845B4 = Some(|g| { t(46); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(47); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(48); sr(23) });
        hk::ft_8008A2BC = Some(|g| { t(49); t(is_go(g)) });
        hk::ftCo_SpecialS_CheckInput = Some(|_| { t(50); sr(24) });
        hk::ftCo_Attack100_CheckInput = Some(|_| { t(51); sr(25) });
        hk::ftCo_800D6824 = Some(|_| { t(52); sr(26) });
        hk::ftCo_800D68C0 = Some(|_| { t(53); sr(27) });
        hk::ftCo_AttackS4_CheckInput = Some(|_| { t(54); sr(28) });
        hk::ftCo_AttackHi4_CheckInput = Some(|_| { t(55); sr(29) });
        hk::ftCo_AttackLw4_CheckInput = Some(|_| { t(56); sr(30) });
        hk::ftCo_AttackS3_CheckInput = Some(|_| { t(57); sr(31) });
        hk::ftCo_AttackHi3_CheckInput = Some(|_| { t(58); sr(32) });
        hk::ftCo_AttackLw3_CheckInput = Some(|_| { t(59); sr(33) });
        hk::ftCo_Attack1_CheckInput = Some(|_| { t(60); sr(34) });
        hk::ftCo_800C36DC = Some(|g| { t(61); t(is_go(g)) });
        hk::ftAnim_SetAnimRate = Some(|g, v| { t(62); t(is_go(g)); t(b(v)) });
        hk::lbGetJObjEndFrame = Some(|j| { t(63); t(jo_idx(j)); ENDF });
        hk::Replay_UcfVersion = Some(|p| { t(64); t(p as u32); if sr(35) { 84 } else { 83 } });
        hk::ftUcf_PadRaw = Some(|p, y, bk| { t(65); t(p as u32); t(y as u32); t(bk as u32); (SRV[12 + bk as usize] % 200) as i32 - 100 });
        hk::ftColl_CreateReflectHit = Some(|g, r, cb| {
            t(66); t(is_go(g)); t(r.x0_bone_id); t(r.x4_max_damage as u32); t(b(r.x8_offset.x)); t(b(r.x8_offset.y)); t(b(r.x8_offset.z));
            t(b(r.x14_size)); t(b(r.x18_damage_mul)); t(b(r.x1C_speed_mul)); t(r.x20_behavior as u32); t(cbeq(cb, m::ftCo_80093790));
        });
        hk::ftCommon_8007DB24 = Some(|g| { t(67); t(is_go(g)) });
        hk::lb_8000B1CC = Some(|j, p0, p1| {
            t(68); t(jo_idx(j)); t(p0.is_null() as u32); t((p1 == &mut POS as *mut Vec3) as u32);
            (*p1).x = 1.0; (*p1).y = 2.0; (*p1).z = 3.0;
        });
    }
    let kinds = [2u32, Ft_Kind_Yoshi as u32, Ft_Kind_Mars as u32, 40];
    let mask = (JOBJ_MTX_INDEP_SRT | JOBJ_USER_DEF_MTX | JOBJ_MTX_DIRTY) as u32;
    let mut c = Check::new(include_str!("data/tu_ftCo_Guard.txt"));
    for op in 0..54u32 {
        for k in 0..1000u32 {
            case(op, k);
            unsafe {
                for i in 0..8 {
                    *JO.add(i) = HSD_JObj::zeroed();
                    *JO2.add(i) = HSD_JObj::zeroed();
                }
                for i in 0..3 { *OWN.add(i) = HSD_Joint::zeroed(); }
                *JPOSE = HSD_Joint::zeroed();
                POS = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                for i in 0..64 { COL[i] = (rnd() & 0xFF) as u8; }
                for i in 0..8 {
                    let p = &mut *PARTS.add(i);
                    p.joint = JO.add(i);
                    p.x4_jobj2 = if rnd() & 3 != 0 { JO2.add(i) } else { core::ptr::null_mut() };
                    (*JO.add(i)).flags = rnd() & mask;
                    (*JO.add(i)).scale.x = rg();
                    let j2 = &mut *JO2.add(i);
                    j2.rotate.x = rg(); j2.rotate.y = rg(); j2.rotate.z = rg();
                    j2.scale.y = rg(); j2.translate.z = rg();
                }
                for i in 0..3 {
                    let o = &mut *OWN.add(i);
                    o.flags = rnd(); o.rotation.x = rg(); o.scale.z = rg(); o.position.y = rg();
                }
                (*OWN).child = OWN.add(1);
                (*OWN.add(1)).next = OWN.add(2);
                if rnd() & 1 != 0 { (*OWN.add(2)).child = core::ptr::null_mut(); }
                (*JPOSE).flags = rnd(); (*JPOSE).rotation.y = rg();
                *JTAB.add(2) = JPOSE; (*FD20).x0 = (JTAB) as *mut _; (*FD).x20 = FD20;
                (*FD8).x11 = (rnd() % 4) as u8; (*FD).x8 = FD8;
                (*CJ).child = CJC;
                (*MS.add(ftCo_MS_Guard as usize)).anim_id = (rnd() % 50) as i32;
                (*go()).hsd_obj = JO.add(7) as *mut core::ffi::c_void;
                MV = rw(); ENDF = rw();
            }
            let fr = f();
            unsafe {
                fr.ft_data = FD; fr.parts = PARTS; fr.x8AC_animSkeleton = SKEL; fr.x108_costume_joint = CJ;
                fr.x1C_actionStateList = MS;
            }
            fr.kind = kinds[(rnd() % 4) as usize];
            fr.player_idx = (rnd() % 4) as u8; fr.pad_port = (rnd() % 4) as u8; fr.set_is_sub_fighter((rnd() & 1) as u8);
            fr.input.held_buttons[0] = rnd(); fr.input.pressed_buttons = rnd();
            fr.input.triggers[0] = rg();
            fr.input.lstick[0].x = rg(); fr.input.lstick[0].y = rg(); fr.input.lstick[1].x = rg();
            fr.facing_dir = rg();
            fr.active_timer.trigger = (rnd() % 8) as u8; fr.active_timer.lstick.x = (rnd() % 8) as u8;
            fr.active_sticky.lstick.x = (rnd() % 3) as u8;
            fr.shield_health = if rnd() & 3 != 0 { rw() } else { 0.0 };
            fr.lightshield_amount = rg();
            fr.x2E8 = (rnd() % 5) as f32;
            unsafe {
                let gv = &mut fr.mv.co.guard;
                gv.x0 = if rnd() & 1 != 0 { (rnd() % 6) as f32 } else { rg() };
                gv.x4 = if rnd() & 1 != 0 { 0.0 } else { rg() };
                gv.x8 = rw(); gv.xC = (rnd() & 1) as i32; gv.x10 = rg();
                gv.x14 = rg(); gv.x18 = rg(); gv.x1C = (rnd() % 3) as i32;
                gv.x20 = rnd() as i32; gv.x24 = rnd() as i32; gv.x2C = rg();
            }
            fr.x19A4 = (rnd() % 30) as i32; fr.x19B0 = if rnd() & 1 != 0 { 10 } else { 3 };
            fr.specialn_facing_dir = rg();
            fr.set_allow_sdi((rnd() & 1) as u8); fr.ground_or_air = rnd() & 1;
            fr.set_reflecting((rnd() & 1) as u8); fr.x221B.set_x221B_b0((rnd() & 1) as u8);
            fr.set_x221C_b1((rnd() & 1) as u16); fr.set_x221C_b2((rnd() & 1) as u16); fr.set_x221C_b3((rnd() & 1) as u16);
            fr.co_attrs.initial_shield_size = rg();
            fr.cur_anim_frame = rg(); fr.cur_pos.x = rw(); fr.cur_pos.y = rw();
            fr.coll_data.floor.normal.x = rg(); fr.coll_data.floor.normal.y = rg();

            let d = cd();
            d.powershield_input_window = (rnd() % 6) as i32; d.x68 = rw(); d.x44C = rg();
            d.x260_startShieldHealth = rw(); d.x264 = rg(); d.x2D4 = rg(); d.x2D8 = rg();
            d.x2F4 = (rnd() % 200) as f32 + rg(); d.x268 = rg() * 3.0;
            d.analog_shoulder_deadzone = rg(); d.x278 = rg(); d.x2EC = rg(); d.x2F0 = rg();
            d.x28C = rw(); d.x290 = rg(); d.x294 = rg(); d.x298 = rg(); d.x2BC = rg();
            d.x2E4 = rg(); d.x2E8 = rg();
            d.sdi_min_stick_mag = rg(); d.sdi_stick_window = (rnd() % 8) as i32; d.sdi_pos_scale = rg();
            d.x4BC = rg(); d.x4C0 = rw();
            d.x2A4 = rg(); d.x2A8 = rg(); d.x2AC = rg(); d.x2B0 = rg(); d.x2B4 = rg(); d.x2B8 = (rnd() % 10) as i32;

            let g = go();
            let ret: u32 = unsafe {
                match op {
                    0 => m::ftCo_80091A2C(g) as u32,
                    1 => m::ftCo_80091A4C(g) as u32,
                    2 => m::ftCo_80091AD8(g, rnd() as i32) as u32,
                    3 => { m::ftCo_80091B90(g, rnd() as i32); 0 }
                    4 => { m::ftCo_80091B9C(g); 0 }
                    5 => { m::ftCo_80091BC4(f()); 0 }
                    6 => { m::ftCo_80091D58(f()); 0 }
                    7 => { m::ftCo_80091E78(g, rg()); 0 }
                    8 => { let id = (rnd() % 2000) as i32; m::ftCo_80092158(g, id, JO.add((rnd() % 8) as usize)); 0 }
                    9 => { m::ftCo_800921DC(g); 0 }
                    10 => { m::ftCo_800923B4(g); 0 }
                    11 => { m::ftCo_80092450(g); 0 }
                    12 => { m::ftCo_800924C0(g); 0 }
                    13 => m::ftCo_800925A4(g) as u32,
                    14 => { m::ftCo_GuardOn_Anim(g); 0 }
                    15 => { m::ftCo_GuardOn_IASA(g); 0 }
                    16 => { m::ftCo_GuardOn_Phys(g); 0 }
                    17 => { m::ftCo_GuardOn_Coll(g); 0 }
                    18 => { m::ftCo_800928CC(g); 0 }
                    19 => { m::ftCo_80092908(g); 0 }
                    20 => { m::ftCo_Guard_Anim(g); 0 }
                    21 => { m::ftCo_Guard_IASA(g); 0 }
                    22 => { m::ftCo_Guard_Phys(g); 0 }
                    23 => { m::ftCo_Guard_Coll(g); 0 }
                    24 => { m::ftCo_80092BCC(g); 0 }
                    25 => { m::ftCo_80092BE8(g); 0 }
                    26 => { m::ftCo_80092C54(g); 0 }
                    27 => { m::ftCo_GuardOff_Anim(g); 0 }
                    28 => { m::ftCo_GuardOff_IASA(g); 0 }
                    29 => { m::ftCo_GuardOff_Phys(g); 0 }
                    30 => { m::ftCo_GuardOff_Coll(g); 0 }
                    31 => { m::ftCo_80092E50(g); 0 }
                    32 => { let a = (rnd() % 40) as i32; b(m::ftCo_80092ED8(a, rg())) }
                    33 => { m::ftCo_80092F2C(g, rnd() & 1 != 0); 0 }
                    34 => { m::ftCo_80093240(g); 0 }
                    35 => { m::ftCo_800932DC(g); 0 }
                    36 => { m::ftCo_GuardSetOff_Anim(g); 0 }
                    37 => { m::ftCo_GuardSetOff_IASA(g); 0 }
                    38 => { m::ftCo_GuardSetOff_Phys(g); 0 }
                    39 => { m::ftCo_GuardSetOff_Coll(g); 0 }
                    40 => m::ftCo_80093694(g) as u32,
                    41 => { m::ftCo_8009370C(g, Some(m::ftCo_80093790)); 0 }
                    42 => { m::ftCo_80093790(g); 0 }
                    43 => { m::ftCo_80093850(g); 0 }
                    44 => { m::ftCo_8009388C(g); 0 }
                    45 => { m::ftCo_800939B4(g); 0 }
                    46 => { m::ftCo_80093A50(g); 0 }
                    47 => { m::ftCo_80093BC0(g); 0 }
                    48 => { m::ftCo_GuardReflect_Anim(g); 0 }
                    49 => { m::ftCo_GuardReflect_IASA(g); 0 }
                    50 => { m::ftCo_GuardReflect_Phys(g); 0 }
                    51 => { m::ftCo_GuardReflect_Coll(g); 0 }
                    52 => b(m::ftCo_80094098(g, &mut POS.x)),
                    _ => { m::ftCo_80094138(f()); 0 }
                }
            };
            let mut h = Hs::new();
            let fr = f();
            unsafe {
                let gv = &fr.mv.co.guard;
                h.m(b(gv.x0)); h.m(b(gv.x4)); h.m(b(gv.x8)); h.m(gv.xC as u32);
                h.m(b(gv.x10)); h.m(b(gv.x14)); h.m(b(gv.x18)); h.m(gv.x1C as u32);
                h.m(gv.x20 as u32); h.m(gv.x24 as u32); h.m(b(gv.x2C));
            }
            h.m(b(fr.lightshield_amount)); h.m(b(fr.shield_health)); h.m(b(fr.gr_vel)); h.m(b(fr.cur_pos.x)); h.m(b(fr.cur_pos.y));
            h.m(fr.active_timer.trigger as u32); h.m(fr.active_timer.lstick.x as u32);
            h.m(fr.reflecting() as u32); h.m(fr.x221B.x221B_b0() as u32); h.m(fr.x221A_b7() as u32); h.m(fr.x2219_b0() as u32);
            h.m(fr.x221C_b1() as u32); h.m(fr.x221C_b2() as u32); h.m(fr.x221C_b3() as u32);
            h.m(cbeq(fr.hitlag_cb, m::ftCo_80093240)); h.m(cbeq(fr.post_hitlag_cb, m::ftCo_800932DC));
            unsafe {
                for i in 0..8 {
                    let j = &*JO.add(i);
                    h.m(b(j.scale.x)); h.m(b(j.scale.y)); h.m(b(j.scale.z));
                    h.m(b(j.translate.x)); h.m(b(j.translate.y)); h.m(b(j.translate.z));
                }
                h.m(b(POS.x)); h.m(b(POS.y)); h.m(b(POS.z));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 54000);
}
