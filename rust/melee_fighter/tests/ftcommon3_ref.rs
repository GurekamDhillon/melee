//! ftcommon3.rs vs the C functions of src/melee/ft/ftcommon.c (gen/cref/ftcommon3_ref.c, built
//! with the shipping build's contraction + gwtool lowering; expected values in
//! tests/data/ftcommon3.txt; regenerate with `python rust/melee_fighter/gen/cref/run.py ftcommon3`).
//! Fill order and the stubbed callees (trace word `TR`) must match the C exactly.
//! One #[test]: ftCommon_PcFloorMisses is a never-reset static in both programs.
#![allow(static_mut_refs, non_snake_case)]

use melee_fighter::ftcommon3 as f3;
use melee_fighter::hooks as h;
use melee_fighter::types::*;

const NF: usize = 6;

struct World {
    f: [Fighter; NF],
    go: [HSD_GObj; NF],
    it: [HSD_GObj; 3],
    jo: [[u64; 32]; 8],
    fd: [ftData; NF],
    da: [ftCo_DatAttrs; NF],
    x8: [ftData_x8; NF],
    pb: [[FighterBone; 8]; NF],
    cd: ftCommonData,
    head: *mut HSD_GObj,
    map_null: bool,
    map_lc: i32,
    itk: [i32; 3],
    itm: [i32; 3],
}

static mut W: *mut World = core::ptr::null_mut();
static mut TR: u32 = 0;

fn w() -> &'static mut World {
    unsafe { &mut *W }
}
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}
fn b(x: f32) -> u32 {
    x.to_bits()
}
fn e<T>(p: *const T) -> u32 {
    let p = p as *const u8;
    if p.is_null() {
        return 0;
    }
    let w = w();
    for i in 0..NF {
        if p == &w.go[i] as *const HSD_GObj as *const u8 {
            return 1 + i as u32;
        }
    }
    for i in 0..3 {
        if p == &w.it[i] as *const HSD_GObj as *const u8 {
            return 10 + i as u32;
        }
    }
    for i in 0..8 {
        if p == w.jo[i].as_ptr() as *const u8 {
            return 30 + i as u32;
        }
    }
    99
}
fn item_ix(g: *mut HSD_GObj) -> Option<usize> {
    let v = e(g);
    if (10..13).contains(&v) { Some((v - 10) as usize) } else { None }
}
fn jo(i: u32) -> *mut HSD_JObj {
    w().jo[i as usize].as_mut_ptr().cast()
}

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
            _ => ((self.rnd() % 2001) as i32 - 1000) as f32 / 7.0,
        }
    }
    fn rit(&mut self) -> *mut HSD_GObj {
        let k = self.rnd() % 4;
        if k < 3 { &mut w().it[k as usize] } else { core::ptr::null_mut() }
    }
}

unsafe extern "C" fn cb_dmg(g: *mut HSD_GObj) {
    t(0x4F0 + e(g));
}
unsafe extern "C" fn cb_death(g: *mut HSD_GObj) {
    t(0x500 + e(g));
}

fn mex3(n: u32, kind: i32, g: *mut HSD_GObj, a: i32) {
    t(0x270 + n * 16 + e(g));
    t(kind as u32);
    t(a as u32);
}
fn mex2(n: u32, kind: i32, g: *mut HSD_GObj) {
    t(0x270 + n * 16 + e(g));
    t(kind as u32);
}

fn install_hooks() {
    unsafe {
        h::SCRIPT_MOD_VALUE = Some(|s, f, base| {
            t(0x100u32.wrapping_add(s as u32));
            t(f as u32);
            base + 1.0
        });
        h::GENO_GROUND_EDGE = Some(|fp, l| t(0x110 + e(fp.gobj) * 2 + l as u32));
        h::SCRIPT_GAME_EVENT = Some(|wh, a, bb, c, d| {
            t(0x120 + wh as u32);
            t(a as u32);
            t(bb as u32);
            t(c as u32);
            t(d as u32);
        });
        h::PC_FLOOR_LOST_REPORT = Some(|_| t(0x130));
        h::MPLIB_8004D164_LINE_COUNT = Some(|| {
            t(0x140);
            if w().map_null { None } else { Some(w().map_lc) }
        });
        h::FT_80084A18 = Some(|g| {
            t(0x150 + e(g));
            (tr() >> 3) & 1 != 0
        });
        h::FTCO_FALL_ENTER = Some(|g| t(0x160 + e(g)));
        h::FT_8008A2BC = Some(|g| t(0x170 + e(g)));
        h::UN_803224DC = Some(|s, x, kb| {
            t(0x180u32.wrapping_add(s as u32));
            t(b(x));
            t(b(kb));
            s & 1 != 0
        });
        h::PL_8003FC44 = Some(|s, u| t(0x190 + s as u32 * 2 + u as u32));
        h::EFLIB_DESTROY_ALL = Some(|g| t(0x1A0 + e(g)));
        h::LBAUDIOAX_80023870 = Some(|a, bb, c, d| {
            t(0x1B0);
            t(a as u32);
            t(bb as u32);
            t(c as u32);
            t(d as u32);
            0
        });
        h::FTPARTS_GET_BONE_INDEX = Some(|fp, i| {
            t(0x1C0u32.wrapping_add(i as u32));
            (fp.player_idx % 8) as i32
        });
        h::HSD_JOBJ_GET_TRANSLATION = Some(|j, v| {
            t(0x1D0 + e(j));
            v.x = e(j) as f32;
            v.y = -((tr() & 7) as f32);
            v.z = 0.5;
        });
        h::HSD_JOBJ_SET_TRANSLATE = Some(|j, v| {
            t(0x1E0 + e(j));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
        });
        h::HSD_JOBJ_MTX_IS_DIRTY = Some(|j| {
            t(0x1F0 + e(j));
            (tr() >> 2) & 1 != 0
        });
        h::HSD_JOBJ_SET_SCALE = Some(|j, v| {
            t(0x200 + e(j));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
        });
        h::HSD_JOBJ_LOAD_JOINT = Some(|j| {
            t(0x210 + e(j));
            jo(tr() % 8)
        });
        h::HSD_JOBJ_ADD_ANIM_ALL = Some(|j, a| {
            t(0x220 + e(j));
            t(e(a));
            t(3);
        });
        h::HSD_JOBJ_REQ_ANIM_ALL = Some(|j, f| {
            t(0x230 + e(j));
            t(b(f));
        });
        h::FTCO_800C5240 = Some(|g| {
            t(0x240 + e(g));
            (tr() >> 1) & 1 != 0
        });
        h::FTCO_800C5500 = Some(|g| t(0x250 + e(g)));
        h::PL_8003EA08 = Some(|s, u| t(0x260 + s as u32 * 2 + u as u32));
        h::MEX_ON_ITEM_DROP_EXT = Some(|k, g, a| mex3(0, k, g, a));
        h::MEX_ON_ITEM_DROP = Some(|k, g, a| mex3(1, k, g, a));
        h::MEX_ON_ITEM_PICKUP2 = Some(|k, g, a| mex3(2, k, g, a));
        h::MEX_ON_ITEM_INVISIBLE = Some(|k, g| mex2(3, k, g));
        h::MEX_ON_ITEM_VISIBLE = Some(|k, g| mex2(4, k, g));
        h::MEX_ON_KNOCKBACK_ENTER = Some(|k, g| mex2(5, k, g));
        h::MEX_ON_KNOCKBACK_EXIT = Some(|k, g| mex2(6, k, g));
        h::MEX_ON_REMOVE_HEAD_ITEM = Some(|k, g| mex2(7, k, g));
        h::MEX_ON_APPLY_HEAD_ITEM = Some(|k, g| mex2(8, k, g));
        h::IT_GET_KIND = Some(|g| {
            let i = item_ix(g);
            t(0x300 + e(g));
            i.map_or(0, |i| w().itk[i])
        });
        h::IT_GET_MOTION_ID = Some(|g| {
            let i = item_ix(g);
            t(0x310 + e(g));
            i.map_or(0, |i| w().itm[i])
        });
        h::IT_8028B08C = Some(|g, s| {
            t(0x320 + e(g));
            t(s as u32);
            (tr() % 7) as i32 - 2
        });
        h::IT_802BDA40 = Some(|g, s| {
            t(0x330 + e(g));
            t(s as u32);
            (tr() % 9) as i32 - 3
        });
        h::IT_PARASOL_TABLE_1 = Some(|ix, g, v| {
            t(0x340 + ix as u32);
            t(e(g));
            t(b(v));
        });
        h::IT_PARASOL_TABLE_3 = Some(|ix, g, v| {
            t(0x350 + ix as u32);
            t(e(g));
            t(b(v));
        });
        h::GM_8016B0B4 = Some(|| {
            t(0x360);
            (tr() >> 2) & 1 != 0
        });
        h::FTCO_DAMAGE_CALC_ANGLE = Some(|fp, kb| {
            t(0x370 + e(fp.gobj));
            t(b(kb));
            if tr() % 8 == 0 { core::f64::consts::PI as f32 } else { (tr() % 700) as f32 / 100.0 }
        });
        h::FTLIB_GETCAMERABONEPOS = Some(|g, v| {
            t(0x380 + e(g));
            v.x = 1.5;
            v.y = -((tr() & 15) as f32);
            v.z = 2.0;
        });
        h::IT_802E5F8C = Some(|g, v, a, bb, ang, kb| {
            t(0x390 + e(g));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
            t(a as u32);
            t(bb as u32);
            t(b(ang));
            t(b(kb));
            (tr() % 100) as i32
        });
        h::PLAYER_GET_UNK98 = Some(|s| {
            t(0x3A0u32.wrapping_add(s as u32));
            (tr() % 1000) as i32
        });
        h::PLAYER_SET_UNK98 = Some(|s, v| {
            t(0x3B0u32.wrapping_add(s as u32));
            t(v as u32);
        });
        h::PLAYER_GET_ENTITY = Some(|s| {
            t(0x3C0u32.wrapping_add(s as u32));
            &mut w().go[((tr() >> 1) % NF as u32) as usize]
        });
        h::MP_LINE_GET_NEXT = Some(|l| {
            t(0x3D0);
            t(l as u32);
            l + (tr() & 1) as i32
        });
        h::MP_LINE_GET_PREV = Some(|l| {
            t(0x3D8);
            t(l as u32);
            l - ((tr() >> 1) & 1) as i32
        });
        h::FTLIB_ISSAMEPLAYER = Some(|a, bb| {
            t(0x3E0 + e(a));
            t(e(bb));
            a == bb || (*((*a).user_data as *const Fighter)).player_idx == (*((*bb).user_data as *const Fighter)).player_idx
        });
        h::PLAYER_8003544C = Some(|s, u| (s as u32).wrapping_add(u as u32) & 1 != 0);
        h::LB_RUMBLE_START = Some(|p, id, a, bb| {
            t(0x600 + p as u32);
            t(id as u32);
            t(a as u32);
            t(bb as u32);
        });
        h::IT_80284FC4 = Some(|g, a, f| {
            t(0x3F0 + e(g));
            t(a as u32);
            t(b(f));
        });
        h::IT_80285024 = Some(|g, a| {
            t(0x400 + e(g));
            t(a as u32);
        });
        h::FTCO_800D105C = Some(|g| t(0x410 + e(g)));
        h::FT_80081C88 = Some(|g, s| {
            t(0x420 + e(g));
            t(b(s));
        });
        h::IT_80294EB0 = Some(|g, a, c| {
            t(0x430 + e(g));
            t(b(a.x));
            t(b(a.y));
            t(b(a.z));
            t(b(c.x));
            t(b(c.y));
            t(b(c.z));
        });
        h::IT_80294E78 = Some(|g, f| {
            t(0x440 + e(g));
            t(b(f));
        });
        h::IT_8026B54C = Some(|g| {
            t(0x450 + e(g));
            (tr() % 600) as f32 - 0.5
        });
        h::ITEM_8026A8EC = Some(|g| t(0x460 + e(g)));
        h::IT_8026BD0C = Some(|g| t(0x470 + e(g)));
        h::ITEM_8026ABD8 = Some(|g, v, f| {
            t(0x480 + e(g));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
            t(b(f));
        });
        h::ITEM_8026AB54 = Some(|g, o, p| {
            t(0x490 + e(g));
            t(e(o));
            t(p as u32);
        });
        h::IT_8026BCF4 = Some(|g| t(0x4A0 + e(g)));
        h::IT_8029A748 = Some(|g, p, part, d| {
            t(0x4B0 + e(g));
            t(b(p.x));
            t(b(p.y));
            t(b(p.z));
            t(part as u32);
            t(b(d));
            if tr() & 1 != 0 { &mut w().it[(tr() % 3) as usize] } else { core::ptr::null_mut() }
        });
        h::IT_8029A8F4 = Some(|g, v| {
            t(0x4C0 + e(g));
            t(b(v.x));
            t(b(v.y));
            t(b(v.z));
        });
        h::IT_8029A89C = Some(|g, f| {
            t(0x4D0 + e(g));
            t(b(f));
        });
        h::PLAY_SFX = Some(|fp, id, vol, pan| {
            t(0x4E0 + e(fp.gobj));
            t(id as u32);
            t(vol as u8 as u32);
            t(pan as u8 as u32);
        });
        h::PLINK_HEAD = Some(|link| if link == HSD_GOBJ_PLINK_FIGHTER { w().head } else { core::ptr::null_mut() });
    }
}

const KINDS: [u32; 13] = [0, 2, 8, 14, 15, 20, 24, 27, 28, 32, 33, 34, 40];
const IKINDS: [i32; 4] = [12, 13, 103, 5];

#[test]
fn ftcommon3_matches_c() {
    let mut world: Box<World> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        W = &mut *world;
    }
    install_hooks();
    let expected = include_str!("data/ftcommon3.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..34u32 {
        for k in 0..300u32 {
            let w = w();
            for i in 0..NF {
                w.f[i] = Fighter::zeroed();
                w.go[i] = HSD_GObj::zeroed();
                w.fd[i] = ftData::zeroed();
                w.da[i] = ftCo_DatAttrs::zeroed();
                w.x8[i] = ftData_x8::zeroed();
                for j in 0..8 {
                    w.pb[i][j] = FighterBone::zeroed();
                }
            }
            for i in 0..3 {
                w.it[i] = HSD_GObj::zeroed();
            }
            w.cd = ftCommonData::zeroed();
            w.head = core::ptr::null_mut();
            unsafe {
                TR = 0;
                h::P_FT_COMMON_DATA = &w.cd;
            }
            let mut r = Rng(0x5A5A1u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            w.map_null = r.rnd() % 8 == 0;
            w.map_lc = (r.rnd() % 30) as i32;
            for i in 0..3 {
                w.it[i].classifier = if r.rnd() % 4 != 0 { 6 } else { 4 };
                w.itk[i] = IKINDS[(r.rnd() % 4) as usize];
                w.itm[i] = (r.rnd() % 12) as i32;
            }
            for i in 0..NF {
                let go: *mut HSD_GObj = &mut w.go[i];
                let fd: *mut ftData = &mut w.fd[i];
                w.go[i].user_data = (&mut w.f[i] as *mut Fighter).cast();
                w.go[i].classifier = 4;
                w.fd[i].x0 = &mut w.da[i];
                w.fd[i].x8 = &mut w.x8[i];
                w.x8[i].x12 = (r.rnd() % 40) as u8;
                w.da[i].model_scaling = r.rf();
                let pb: *mut FighterBone = w.pb[i].as_mut_ptr();
                for j in 0..8 {
                    w.pb[i][j].joint = jo(r.rnd() % 8);
                }
                let f = &mut w.f[i];
                f.gobj = go;
                f.ft_data = fd;
                f.parts = pb;
                f.kind = KINDS[(r.rnd() % 13) as usize] as _;
                f.x8_spawnNum = (r.rnd() % 8) as i32;
                f.player_idx = (r.rnd() % 4) as u8;
                f.pad_port = (r.rnd() % 4) as u8;
                f.motion_id = (r.rnd() % 340) as _;
                f.facing_dir = r.rf();
                f.gr_vel = r.rf();
                f.self_vel.x = r.rf();
                f.x34_scale.y = r.rf();
                f.cur_pos.x = r.rf();
                f.cur_pos.y = r.rf();
                f.cur_pos.z = r.rf();
                f.xD4_unk_vel.x = r.rf();
                f.xD4_unk_vel.y = r.rf();
                f.xD4_unk_vel.z = r.rf();
                f.x2C4.x = r.rf();
                f.x2C4.y = r.rf();
                f.x6A4_transNOffset.z = r.rf();
                f.ground_or_air = r.rnd() & 1;
                f.ecb_lock = (r.rnd() % 20) as i32;
                f.coll_data.x130_flags = r.rnd();
                f.coll_data.floor.index = (r.rnd() % 6) as i32 - 2;
                f.x1968_jumpsUsed = (r.rnd() % 4) as u8;
                f.x1969_walljumpUsed = (r.rnd() % 2) as u8;
                f.co_attrs.max_jumps = (r.rnd() % 7) as i32;
                f.co_attrs.ground_max_horizontal_velocity = r.rf();
                f.co_attrs.x114.x = r.rf();
                f.co_attrs.x114.y = r.rf();
                f.co_attrs.x114.z = r.rf();
                f.co_attrs.x120.x = r.rf();
                f.co_attrs.x120.y = r.rf();
                f.co_attrs.x120.z = r.rf();
                f.co_attrs.x12C = r.rf();
                f.co_attrs.x130.x = r.rf();
                f.co_attrs.x130.y = r.rf();
                f.co_attrs.x130.z = r.rf();
                f.co_attrs.x13C = r.rf();
                f.frame_speed_mul = r.rf();
                f.dmg.x18A4_knockbackMagnitude = r.rf();
                f.dmg.kb_applied = if r.rnd() & 1 != 0 { r.rf() } else { 0.0 };
                f.dmg.facing_dir_1 = r.rf();
                f.dmg.x1854_collpos.x = r.rf();
                f.dmg.x1854_collpos.y = r.rf();
                f.dmg.x18c4_source_ply = (r.rnd() % 7) as i32;
                f.xF8_playerNudgeVel.x = r.rf();
                f.xF8_playerNudgeVel.y = r.rf();
                f.x1A6C = r.rf();
                f.x1A7C.x = r.rf();
                f.x1A7C.y = r.rf();
                f.x1A7C.z = r.rf();
                f.x20A0_accessory = if r.rnd() & 1 != 0 { jo(r.rnd() % 8) } else { core::ptr::null_mut() };
                f.victim_gobj = if r.rnd() % 4 == 0 {
                    let j = (r.rnd() % NF as u32) as usize;
                    unsafe { (*W).go.as_mut_ptr().add(j) }
                } else {
                    core::ptr::null_mut()
                };
                f.item_gobj = r.rit();
                f.x1978 = r.rit();
                f.x197C = r.rit();
                f.x1980 = r.rit();
                f.x1984_heldItemSpec = r.rit();
                f.x2014 = r.rnd() as i32;
                f.x2018 = (r.rnd() % 2001) as i32 - 1000;
                f.x201C = r.rnd() as i32;
                f.x2020 = r.rnd() as i8;
                f.x2021 = r.rnd() as i8;
                f.x2024 = (r.rnd() % 201) as i32 - 100;
                f.x2144 = r.rnd() as i32;
                f.x2148 = r.rnd() as i32;
                f.take_dmg_cb = if r.rnd() & 1 != 0 { Some(cb_dmg) } else { None };
                f.death1_cb = if r.rnd() & 1 != 0 { Some(cb_death) } else { None };
                let v = r.rnd() & 1;
                unsafe { f.x594.x0.set_x594_b0(v as u8) };
                let v = r.rnd() & 1;
                f.set_is_sub_fighter(v as u8);
                let v = r.rnd() & 1;
                f.set_x2219_b0(v as u8);
                let v = (r.rnd() % 4 == 0) as u8;
                f.set_x2219_b1(v);
                let v = (r.rnd() % 4 == 0) as u8;
                f.set_x2219_b5(v);
                let v = (r.rnd() % 4 == 0) as u16;
                f.set_x221D_b5(v);
                let v = r.rnd() & 1;
                f.set_x221E_b3(v as u8);
                let v = (r.rnd() % 4 == 0) as _;
                f.set_is_sleeping(v);
                let v = r.rnd() & 1;
                f.set_stamina_dead(v as _);
                let v = r.rnd() & 1;
                f.set_x2227_b0(v as _);
            }
            let cd = &mut w.cd;
            cd.x450 = r.rf();
            cd.x454 = r.rf();
            cd.x458 = r.rf();
            cd.x45C = r.rf();
            cd.x460 = r.rf();
            cd.x5C0 = r.rf();
            cd.x6F8 = r.rnd() as i32;
            cd.x700 = (r.rnd() % 2001) as i32 - 1000;
            cd.x704 = r.rf();
            cd.x708 = r.rf();
            cd.x70C = r.rf();
            cd.x710 = r.rf();
            let n = r.rnd() % (NF as u32 + 1);
            let mut used = 0u32;
            let mut prev: *mut HSD_GObj = core::ptr::null_mut();
            for _ in 0..n {
                let mut j = (r.rnd() % NF as u32) as usize;
                while used & (1 << j) != 0 {
                    j = (j + 1) % NF;
                }
                used |= 1 << j;
                let gj: *mut HSD_GObj = &mut w.go[j];
                if prev.is_null() {
                    w.head = gj;
                } else {
                    unsafe { (*prev).next = gj };
                }
                prev = gj;
            }
            let gix = (r.rnd() % NF as u32) as usize;
            let g: *mut HSD_GObj = &mut w.go[gix];
            let fp: *mut Fighter = &mut w.f[gix];
            let gi = r.rit();
            let a = r.rf();
            let ia = (r.rnd() % 16) as i32 - 4;
            let ib = r.rnd() as i32;
            let mut wv = Vec3 { x: r.rf(), y: r.rf(), z: r.rf() };
            if (op == 5 || op == 6) && k & 1 != 0 {
                for i in 0..NF {
                    w.f[i].x2C4.x = 0.0;
                    w.f[i].cur_pos.x = 0.0;
                    w.f[i].xD4_unk_vel.x = 0.0;
                }
                w.f[gix].x2C4.x = r.rf();
                wv.x = -(w.f[gix].x2C4.x * w.f[gix].facing_dir);
            }
            unsafe { TR = r.rnd() };
            let fpm = || unsafe { &mut *fp };
            let mut ret = 0u32;
            match op {
                0 => f3::ftcommon_8007d6a4(fpm()),
                1 => f3::ftcommon_8007d7fc(fpm()),
                2 => f3::ftcommon_8007d92c(g),
                3 => f3::ftcommon_8007db24(g),
                4 => f3::ftcommon_8007db58(g),
                5 => f3::ftcommon_8007dd7c(g, &wv),
                6 => f3::ftcommon_8007dfd0(g, &wv),
                7 => f3::ftcommon_8007e0e4(g),
                8 => f3::ftcommon_8007e358(g),
                9 => f3::ftcommon_8007e3ec(g),
                10 => {
                    fpm().x20A0_accessory = core::ptr::null_mut();
                    f3::set_accessory(fpm(), jo(ib as u32 % 8).cast());
                }
                11 => f3::ftcommon_8007e690(fpm(), jo(ib as u32 % 8).cast()),
                12 => {
                    let o = if ib & 1 != 0 { fpm().x1978 } else { gi };
                    f3::ftcommon_8007e6dc(g, o, ib)
                }
                13 => f3::ftcommon_8007e79c(g, ib),
                14 => f3::ftcommon_8007e7e4(g, ib),
                15 => f3::ftcommon_8007e82c(g),
                16 => {
                    if f3::ft_get_parasol_status(g) == -1 {
                        t(0xBAD);
                    } else {
                        f3::ftcommon_8007e83c(g, (ib as u32 % 7) as i32, if ib & 8 != 0 { 0.0 } else { a });
                    }
                }
                17 => ret = f3::ft_get_parasol_status(g) as u32,
                18 => f3::ftcommon_8007ea90(fpm(), ib),
                19 => f3::ftcommon_8007ec30(ia as u32, ib as u32),
                20 => f3::ftcommon_8007eec8(fpm(), ia, ib),
                21 => f3::ftcommon_8007ef5c(fpm(), ia),
                22 => f3::ftcommon_8007f578(g),
                23 => f3::ftcommon_8007f5cc(g, ia & 3),
                24 => f3::ftcommon_8007f6a4(fpm(), jo(ib as u32 % 8)),
                25 => f3::ftcommon_8007f824(g),
                26 => f3::ftcommon_8007f86c(g),
                27 => f3::ftcommon_8007f8e8(g),
                28 => f3::ftcommon_8007f948(g, gi, ib),
                29 => f3::ftcommon_8007f9b4(g),
                30 => f3::ftcommon_8007fa00(g),
                31 => f3::ftcommon_8007fa58(g, gi),
                32 => f3::ftcommon_8007fc7c(g, a),
                33 => f3::ftcommon_8007fe84(g, gi, ib, a),
                _ => unreachable!(),
            }
            let mut hsh = 0u32;
            let m = |h: u32, v: u32| h.wrapping_mul(31).wrapping_add(v);
            for i in 0..NF {
                let f = &w.f[i];
                hsh = m(hsh, b(f.self_vel.x) ^ b(f.gr_vel).wrapping_mul(3) ^ b(f.dmg.x18A4_knockbackMagnitude).wrapping_mul(7));
                hsh = m(hsh, b(f.xF8_playerNudgeVel.x) ^ b(f.xF8_playerNudgeVel.y).wrapping_mul(5));
                hsh = m(hsh, b(f.x1A7C.x) ^ b(f.x1A7C.y).wrapping_mul(3) ^ b(f.x1A7C.z).wrapping_mul(11));
                hsh = m(
                    hsh,
                    (f.ground_or_air as u32)
                        .wrapping_add((f.ecb_lock as u32).wrapping_mul(4))
                        .wrapping_add(f.coll_data.x130_flags.wrapping_mul(13)),
                );
                hsh = m(hsh, f.x1968_jumpsUsed as u32 + f.x1969_walljumpUsed as u32 * 8);
                hsh = m(hsh, e(f.x20A0_accessory) + e(f.item_gobj) * 100 + e(f.x1978) * 10000);
                hsh = m(hsh, e(f.x197C) + e(f.x1980) * 100 + e(f.x1984_heldItemSpec) * 10000);
                hsh = m(
                    hsh,
                    (f.x2014 as u32)
                        .wrapping_add((f.x2018 as u32).wrapping_mul(3))
                        .wrapping_add((f.x201C as u32).wrapping_mul(5))
                        .wrapping_add((f.x2024 as u32).wrapping_mul(7)),
                );
                hsh = m(hsh, f.x2020 as u8 as u32 + f.x2021 as u8 as u32 * 256);
                hsh = m(hsh, (f.x2144 as u32).wrapping_add((f.x2148 as u32).wrapping_mul(3)));
                hsh = m(hsh, f.x2227_b0() as u32 | (f.x2219_b0() as u32) << 1 | (f.x221E_b3() as u32) << 2);
            }
            let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, hsh, tr());
            let want = lines.next().expect("missing expected line");
            assert_eq!(got, want, "op {op} case {k}");
            checked += 1;
        }
    }
    assert_eq!(checked, 10200);
}
