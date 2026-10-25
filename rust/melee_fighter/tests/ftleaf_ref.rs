//! Port round 9 vs the C (gen/cref/ftleaf_ref.c, cut verbatim, shipping contraction + gwtool
//! lowering; expected values in tests/data/ftleaf.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ftleaf`): ftaction.c skip/flag handlers,
//! fighter.c leaves + UCF pad ring, ftanim.c, ftcoll.c, the ftcommon.c atan2f angles.
//! Fill order matches the C driver exactly (same xorshift, same draw order).
#![allow(static_mut_refs)]

use melee_fighter::types::*;
use melee_fighter::{fighter as fi, ftaction as fa, ftanim as an, ftcoll as co, ftcommon3 as c3, hooks as h};

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            4 => f32::from_bits(self.rnd()),
            5 => -0.609375,
            6 => f32::from_bits(0x7FC0_0000),
            _ => ((self.rnd() % 2001) as i32 - 1000) as f32 / 777.0,
        }
    }
}

type Cmd = fn(*mut HSD_GObj, &mut CommandInfo);
const CMDF: [Cmd; 29] = [
    fa::ftaction_800711dc, fa::ftaction_800715ec, fa::ftaction_8007168c, fa::ftaction_800716f8,
    fa::ftaction_80071774, fa::ftaction_800717c8, fa::ftaction_80071810, fa::ftaction_80071ca4,
    fa::ftaction_80071d30, fa::ftaction_80071f0c, fa::ftaction_800722c8, fa::ftaction_800726c0,
    fa::ftaction_80072e24, fa::ftaction_80072fe0, fa::ftaction_8007309c, fa::ftaction_8007320c,
    fa::ftaction_8007296c, fa::ftaction_800729c4, fa::ftaction_80072a4c, fa::ftaction_80072aac,
    fa::ftaction_80072b04, fa::ftaction_80072b84, fa::ftaction_80072be4, fa::ftaction_80072c5c,
    fa::ftaction_80073108, fa::ftaction_80071908, fa::ftaction_8007192c, fa::ftaction_80071950,
    fa::ftaction_80071974,
];

fn state(f: &Fighter) -> u32 {
    let mut hh: u32 = 0;
    let mut hm = |v: u32| hh = hh.wrapping_mul(31).wrapping_add(v);
    let tf = unsafe { f.anon_2210.anon_0 };
    hm(tf.throw_flags_b0() as u32);
    hm(tf.throw_flags_b1() as u32);
    hm(tf.throw_flags_b2() as u32);
    hm(tf.throw_flags_b3() as u32);
    hm(f.allow_interrupt() as u32);
    hm(f.x2219_b7() as u32);
    hm(f.x221A_b1() as u32);
    for s in f.x8B0.iter() {
        hm(s.x10 as i32 as u32);
        hm(s.x11 as i32 as u32);
    }
    unsafe {
        hm(fi::SPAWN_NUM_COUNTER);
        hm(co::DMG_LOG0_IDX as u32);
        hm(co::DMG_LOG1_IDX as u32);
        for p in fi::UCF_PAD.iter() {
            for i in 0..4 {
                hm(p.x[i] as i32 as u32);
                hm(p.y[i] as i32 as u32);
            }
            hm(p.idx as u32);
            hm(p.counter as u32);
        }
    }
    hh
}

#[test]
fn ftleaf_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ftleaf.txt")).unwrap();
    let mut lines = text.lines();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut cd: Box<ftCommonData> = Box::new(unsafe { core::mem::zeroed() });
    let mut jo: Box<[HSD_JObj; 8]> = Box::new(unsafe { core::mem::zeroed() });
    let mut bones: Box<[FighterBone; 4]> = Box::new(unsafe { core::mem::zeroed() });
    let mut cmds: Box<[CmdUnion; 16]> = Box::new(unsafe { core::mem::zeroed() });
    let mut hb: Box<HitCapsule> = Box::new(unsafe { core::mem::zeroed() });
    unsafe { h::P_FT_COMMON_DATA = &*cd };
    let jbase: *mut HSD_JObj = jo.as_mut_ptr();
    let jid = |j: *mut HSD_JObj| -> u32 {
        if j.is_null() {
            return 0;
        }
        (0..8).find(|&i| j == unsafe { jbase.add(i) }).map_or(99, |i| 1 + i as u32)
    };
    let mut n = 0;
    for op in 0..51u32 {
        for k in 0..200u32 {
            let mut ret: u32 = 0;
            let mut delta: isize = 0;
            *f = Fighter::zeroed();
            *go = HSD_GObj::zeroed();
            let mut cmd = CommandInfo::zeroed();
            let mut rg = Rng(0x2468Au32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            let fpp: *mut Fighter = &mut *f;
            go.user_data = fpp.cast();
            let gp: *mut HSD_GObj = &mut *go;
            f.gobj = gp;
            {
                let tf = unsafe { &mut f.anon_2210.anon_0 };
                tf.set_throw_flags_b0((rg.rnd() & 1) as u8);
                tf.set_throw_flags_b1((rg.rnd() & 1) as u8);
                tf.set_throw_flags_b2((rg.rnd() & 1) as u8);
                tf.set_throw_flags_b3((rg.rnd() & 1) as u8);
            }
            f.set_allow_interrupt((rg.rnd() & 1) as u8);
            f.set_x2219_b7((rg.rnd() & 1) as u8);
            f.set_x221A_b1((rg.rnd() & 1) as u8);
            for i in 0..5 {
                f.x8B0[i].x10 = rg.rnd() as i8;
                f.x8B0[i].x11 = rg.rnd() as i8;
            }
            f.player_id = (rg.rnd() % 6) as u8;
            f.set_is_sub_fighter((rg.rnd() & 1) as u8);
            f.cur_pos.x = rg.rf();
            f.cur_pos.y = rg.rf();
            f.input.lstick[0].x = rg.rf();
            f.input.lstick[0].y = rg.rf();
            f.input.cstick[0].x = rg.rf();
            f.input.cstick[0].y = rg.rf();
            if rg.rnd() % 3 == 0 {
                f.input.lstick[0].y = -((rg.rnd() % 1000) as f32) / 999.0;
            }
            if rg.rnd() % 3 == 0 {
                f.input.lstick[0].x = ((rg.rnd() % 161) as i32 - 80) as f32 / 80.0;
            }
            f.active_timer.lstick.y = (rg.rnd() % 4) as u8;
            for i in 0..4 {
                bones[i].joint = unsafe { jbase.add(i) };
                bones[i].x4_jobj2 = unsafe { jbase.add(4 + i) };
            }
            f.parts = bones.as_mut_ptr();
            unsafe {
                fi::SPAWN_NUM_COUNTER = if rg.rnd() % 4 == 0 { 0xFFFF_FFFFu32 - (rg.rnd() % 2) } else { rg.rnd() };
                co::DMG_LOG0_IDX = rg.rnd() as i32;
                co::DMG_LOG1_IDX = rg.rnd() as i32;
                for p in fi::UCF_PAD.iter_mut() {
                    for i in 0..4 {
                        p.x[i] = rg.rnd() as i8;
                        p.y[i] = rg.rnd() as i8;
                    }
                    p.idx = if rg.rnd() % 3 == 0 { rg.rnd() as u8 } else { (rg.rnd() % 4) as u8 };
                    p.counter = if rg.rnd() % 2 != 0 { 0 } else { rg.rnd() as u8 };
                }
            }
            cd.x7A8 = rg.rf();
            hb.coll_distance = rg.rf();
            let base: *mut CmdUnion = cmds.as_mut_ptr();
            cmd.anon_8.u = unsafe { base.add((rg.rnd() % 4) as usize) };
            if op < 29 {
                let before = unsafe { cmd.anon_8.u };
                CMDF[op as usize](gp, &mut cmd);
                delta = unsafe { cmd.anon_8.u.offset_from(before) };
            } else {
                match op {
                    29 => {
                        ret = fi::fighter_new_spawn_80068e40();
                        ret = ret.wrapping_mul(31).wrapping_add(fi::fighter_new_spawn_80068e40());
                    }
                    30 => fi::fighter_unk_set_flag_8006cfbc(gp),
                    31 => fi::set_bit(gp),
                    32 => ret = fi::fighter_get_pos_x(&f).to_bits(),
                    33 => ret = fi::fighter_get_pos_y(&f).to_bits(),
                    34 => {
                        let a = (rg.rnd() % 4) as i32;
                        let b = (rg.rnd() % 600) as i32 - 300;
                        let c = (rg.rnd() % 600) as i32 - 300;
                        fi::ftucf_pad_buffer_push(a, b, c);
                    }
                    35 => {
                        let p = (rg.rnd() % 4) as i32;
                        let i = (rg.rnd() % 2) as i32;
                        ret = fi::ftucf_pad_raw(p, i, (rg.rnd() % 9) as i32 - 2) as u32;
                    }
                    36 => ret = fi::ftucf_flick_counter((rg.rnd() % 4) as i32) as u32,
                    37 => ret = fi::ftucf_rim_units(rg.rf()) as u32,
                    38 => {
                        // half the cases: raw y moved by 43..45 against two entries earlier
                        let p = (rg.rnd() % 4) as usize;
                        if rg.rnd() & 1 != 0 {
                            unsafe {
                                let pad = &mut fi::UCF_PAD[p];
                                let i0 = (pad.idx & 3) as usize;
                                pad.y[(i0.wrapping_sub(2)) & 3] = (pad.y[i0] as i32 - 43 - (rg.rnd() % 3) as i32) as i8;
                            }
                        }
                        fi::ftucf_flick_count(&f, p as i32);
                    }
                    39 => {
                        let mut v = Vec3 { x: rg.rf(), y: rg.rf(), z: rg.rf() };
                        an::scale_inline(rg.rf(), &mut v);
                        ret = v.x.to_bits().wrapping_mul(31).wrapping_add(v.y.to_bits()).wrapping_mul(31).wrapping_add(v.z.to_bits());
                    }
                    40 => {
                        let p = (rg.rnd() % 4) as i32;
                        ret = jid(unsafe { an::get_part_joint(&f, p, rg.rnd() & 1 != 0) });
                    }
                    41 => an::ftanim_8007077c(gp),
                    42 => {
                        let p = (rg.rnd() % 5) as i32;
                        an::ftanim_80070fb4(gp, p, rg.rnd() as i32);
                    }
                    43 => co::ftcoll_800765e0(),
                    44 => {
                        let mut d = rg.rf();
                        if rg.rnd() & 1 != 0 {
                            d *= 0.01;
                        }
                        ret = co::get_env_dmg(d) as u32;
                    }
                    45 => ret = co::inline_b1(&hb) as u32,
                    46 => ret = co::get_unk_val(&f, rg.rnd() as i32) as u32,
                    47 => ret = c3::ftco_get_lstick_angle(&f).to_bits(),
                    48 => ret = c3::ftco_get_cstick_angle(&f).to_bits(),
                    49 => ret = c3::ftcommon_8007d9d4(&f).to_bits(),
                    50 => {
                        f.coll_data.floor_skip = rg.rnd() as i32;
                        unsafe { melee_world::tu::mpcoll::mpClearFloorSkip(&mut f.coll_data as *mut _ as _) };
                        ret = f.coll_data.floor_skip as u32;
                    }
                    _ => unreachable!(),
                }
            }
            let got = format!("{} {} {:08X} {:08X} {}", op, k, ret, state(&f), delta);
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            n += 1;
        }
    }
    assert!(lines.next().is_none());
    assert_eq!(n, 10_200);
}
