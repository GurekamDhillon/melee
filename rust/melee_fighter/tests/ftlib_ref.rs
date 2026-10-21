//! ftlib.rs vs src/melee/ft/ftlib.c (gen/cref/ftlib_ref.c, built with the shipping build's
//! contraction + gwtool lowering; expected values in tests/data/ftlib.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ftlib`). Six fighters, a random fighter p-link
//! list, one random op per case. Fill order and the stubbed callees (trace word `TR`) must
//! match the C exactly.
#![allow(static_mut_refs, non_upper_case_globals)]

use melee_fighter::ftlib as l;
use melee_fighter::hooks as h;
use melee_fighter::types::*;
use std::collections::BTreeSet;

const NF: usize = 6;
static mut TR: u32 = 0;
static mut G_SLIP: i32 = 0;

fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
fn tr() -> u32 {
    unsafe { TR }
}
fn b(x: f32) -> u32 {
    x.to_bits()
}

struct World {
    f: std::vec::Vec<Fighter>,
    go: std::vec::Vec<HSD_GObj>,
    it: std::vec::Vec<HSD_GObj>,
    camg: HSD_GObj,
    jo: std::vec::Vec<[u64; 32]>,
    fd: std::vec::Vec<ftData>,
    da: std::vec::Vec<ftCo_DatAttrs>,
    sfx: std::vec::Vec<FtSFX>,
    cs: std::vec::Vec<CmSubject>,
    pb: std::vec::Vec<[FighterBone; 8]>,
    cd: ftCommonData,
    plinks: [*mut HSD_GObj; 64],
}
static mut W: *mut World = core::ptr::null_mut();
fn w() -> &'static mut World {
    unsafe { &mut *W }
}

fn e<T>(p: *const T) -> u32 {
    let p = p as usize;
    if p == 0 {
        return 0;
    }
    let w = w();
    for i in 0..NF {
        if p == &w.go[i] as *const _ as usize {
            return 1 + i as u32;
        }
    }
    for i in 0..3 {
        if p == &w.it[i] as *const _ as usize {
            return 10 + i as u32;
        }
    }
    if p == &w.camg as *const _ as usize {
        return 20;
    }
    for i in 0..8 {
        if p == w.jo[i].as_ptr() as usize {
            return 30 + i as u32;
        }
    }
    for i in 0..NF {
        if p == &w.cs[i] as *const _ as usize {
            return 40 + i as u32;
        }
    }
    for i in 0..NF {
        if p == &w.f[i].coll_data as *const _ as usize {
            return 50 + i as u32;
        }
    }
    for i in 0..NF {
        if p == &w.f[i].x20A4 as *const _ as usize {
            return 60 + i as u32;
        }
    }
    99
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
}

fn install_hooks() {
    unsafe {
        h::PLINK_HEAD = Some(|link| w().plinks[link as usize]);
        h::GM_8016B168 = Some(|| {
            t(0x140);
            (tr() >> 4) & 1 != 0
        });
        h::SLIPPI_CODES = Some(|| {
            t(0x150);
            G_SLIP
        });
        h::HSD_RANDI = Some(|n| {
            t(0x160u32.wrapping_add(n as u32));
            ((tr() >> 7) & 1) as i32
        });
        h::FTPARTS_GET_BONE_INDEX = Some(|fp, i| {
            t(0x170u32.wrapping_add(i as u32));
            (fp.player_id % 8) as i32
        });
        h::FIGHTER_UPDATE_MODEL_SCALE = Some(|g| t(0x180 + e(g)));
        h::LB_8000B1CC = Some(|j, off, out| {
            let ee = e(j);
            t(0x190 + ee);
            out.x = off.x + ee as f32;
            out.y = off.y - ee as f32;
            out.z = off.z;
        });
        h::FTCOMMON_8007E6DC = Some(|g, o, i| {
            t(0x1A0 + e(g));
            t(e(o));
            t(i as u32);
        });
        h::FTCOMMON_8007E82C = Some(|g| t(0x1B0 + e(g)));
        h::FIGHTER_RESET_INPUT_DATA = Some(|g| t(0x1C0 + e(g)));
        h::CAMERA_80030A50 = Some(|| {
            t(0x1D0);
            if (tr() >> 3) & 1 != 0 { &mut w().camg } else { core::ptr::null_mut() }
        });
        h::CAMERA_LOGIC_TO_SCREEN = Some(|s, p| {
            t(0x1E0 + e(s));
            p.x = p.x.wrapping_add(1);
            (tr() >> 2) & 1 != 0
        });
        h::CAMERA_80031154 = Some(|v| {
            t(0x1F0);
            t(b(v.x));
            (tr() >> 6) & 1 != 0
        });
        h::PLAYER_8003544C = Some(|s, u| (s as u32).wrapping_add(u as u32) & 1 != 0);
        h::LB_RUMBLE_START = Some(|p, id, a, bb| {
            t(0x600 + p as u32);
            t(id as u32);
            t(a as u32);
            t(bb as u32);
        });
        h::HSD_PAD_RUMBLE_REMOVE_ID = Some(|p, id| {
            t(0x700 + p as u32);
            t(id as u32);
        });
        h::LB_RUMBLE_STOP = Some(|p| t(0x800 + p as u32));
        h::PLAYER_GET_MORE_FLAGS_BIT4 = Some(|s| {
            t(0x200u32.wrapping_add(s as u32));
            ((tr() >> 5) & 1) as u8
        });
        h::FTDATA_80085560 = Some(|a, bb| {
            t(0x210);
            t(a as u32);
            t(bb as u32);
        });
        h::FTCO_800D4F24 = Some(|g, i| {
            t(0x220 + e(g));
            t(i as u32);
        });
        h::FIGHTER_UNK_PROCESS_DEATH = Some(|g| t(0x230 + e(g)));
        h::FTCOMMON_8007D92C = Some(|g| t(0x240 + e(g)));
        h::GENO_DEFINE_TEMPLATE_SKELETON = Some(|k| {
            t(0x250u32.wrapping_add(k as u32));
            ((tr() >> 1) & 1) as i32
        });
        h::GENO_DEFINE_BASE_KIND = Some(|k| {
            t(0x260u32.wrapping_add(k as u32));
            k % 27
        });
        h::MEX_SSM_FOR_PORT_KIND = Some(|k| {
            t(0x270u32.wrapping_add(k as u32));
            (tr() % 5) as i32 - 1
        });
        h::PL_8003E4A4 = Some(|s, u, _p, a| {
            t(0x280u32.wrapping_add(s as u32 * 2).wrapping_add(u as u32));
            t(a as u32);
        });
        h::FTANIM_IS_FRAMES_REMAINING = Some(|g| {
            t(0x290 + e(g));
            true
        });
        h::FTGW_INIT_8014A7F4 = Some(|g, _| t(0x2A0 + e(g)));
        h::FTGW_INIT_8014A814 = Some(|g, _| t(0x2B0 + e(g)));
        h::FTKB_INIT_800EEB00 = Some(|g, _| t(0x2C0 + e(g)));
        h::FTKB_INIT_800EEB1C = Some(|g, _| t(0x2D0 + e(g)));
        h::IT_80294E78 = Some(|g, f| {
            t(0xA00 + e(g));
            t(b(f));
        });
        h::IT_8029A89C = Some(|g, f| {
            t(0xB00 + e(g));
            t(b(f));
        });
    }
}

const KINDS: [u32; 13] = [0, 2, 8, 14, 15, 20, 24, 27, 28, 32, 33, 34, 40];
const NOPS: u32 = 87;
type G = *mut HSD_GObj;

#[test]
fn ftlib_matches_c() {
    let mut world = Box::new(World {
        f: (0..NF).map(|_| Fighter::zeroed()).collect(),
        go: (0..NF).map(|_| HSD_GObj::zeroed()).collect(),
        it: (0..3).map(|_| HSD_GObj::zeroed()).collect(),
        camg: HSD_GObj::zeroed(),
        jo: vec![[0u64; 32]; 8],
        fd: (0..NF).map(|_| ftData::zeroed()).collect(),
        da: (0..NF).map(|_| ftCo_DatAttrs::zeroed()).collect(),
        sfx: (0..NF).map(|_| FtSFX::zeroed()).collect(),
        cs: (0..NF).map(|_| CmSubject::zeroed()).collect(),
        pb: (0..NF).map(|_| [FighterBone::zeroed(); 8]).collect(),
        cd: ftCommonData::zeroed(),
        plinks: [core::ptr::null_mut(); 64],
    });
    unsafe {
        W = &mut *world;
        h::P_FT_COMMON_DATA = &w().cd;
    }
    install_hooks();
    let expected = include_str!("data/ftlib.txt");
    let mut lines = expected.lines();
    let mut seen = BTreeSet::new();
    let mut checked = 0;
    for op in 0..NOPS {
        for k in 0..120u32 {
            let w = w();
            let (mut r, mut o1, mut o2) = (0u32, 0u32, 0u32);
            let mut t_h: i32 = 0;
            let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let mut sv = S32Vec2 { x: 0, y: 0 };
            for i in 0..NF {
                w.f[i] = Fighter::zeroed();
                w.go[i] = HSD_GObj::zeroed();
                w.fd[i] = ftData::zeroed();
                w.da[i] = ftCo_DatAttrs::zeroed();
                w.sfx[i] = FtSFX::zeroed();
                w.cs[i] = CmSubject::zeroed();
                w.pb[i] = [FighterBone::zeroed(); 8];
            }
            for i in 0..3 {
                w.it[i] = HSD_GObj::zeroed();
            }
            w.cd = ftCommonData::zeroed();
            w.plinks = [core::ptr::null_mut(); 64];
            unsafe { TR = 0 };
            let mut rg = Rng(0x13579u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            unsafe { G_SLIP = (rg.rnd() % 3) as i32 };
            for i in 0..NF {
                let gp: G = &mut w.go[i];
                let fpp: *mut Fighter = &mut w.f[i];
                w.go[i].user_data = fpp as *mut _;
                w.go[i].classifier = if rg.rnd() % 4 != 0 { 4 } else { 6 };
                let fdp: *mut ftData = &mut w.fd[i];
                w.fd[i].x0 = &mut w.da[i];
                w.fd[i].x4C_sfx = &mut w.sfx[i];
                let pbp = w.pb[i].as_mut_ptr();
                for j in 0..8 {
                    let q = (rg.rnd() % 8) as usize;
                    w.pb[i][j].joint = w.jo[q].as_mut_ptr() as *mut HSD_JObj;
                }
                let csp: *mut CmSubject = &mut w.cs[i];
                let f = &mut w.f[i];
                f.gobj = gp;
                f.ft_data = fdp;
                f.parts = pbp;
                f.x890_cameraBox = csp;
                f.kind = KINDS[(rg.rnd() % 13) as usize];
                f.x8_spawnNum = (rg.rnd() % 8) as i32;
                f.player_id = (rg.rnd() % 4) as u8;
                f.x618_player_id = (rg.rnd() % 4) as u8;
                f.x619_costume_id = (rg.rnd() % 4) as u8;
                f.x61A_controller_index = (rg.rnd() % 4) as u8;
                f.team = (rg.rnd() % 3) as u8;
                f.motion_id = (rg.rnd() % 340) as i32;
                f.facing_dir = rg.rf();
                f.x34_scale.x = rg.rf();
                f.x34_scale.y = rg.rf();
                f.cur_pos.x = rg.rf();
                f.cur_pos.y = rg.rf();
                f.cur_pos.z = rg.rf();
                f.prev_pos.x = rg.rf();
                f.prev_pos.y = rg.rf();
                f.prev_pos.z = rg.rf();
                f.pos_delta.x = rg.rf();
                f.pos_delta.y = rg.rf();
                f.pos_delta.z = rg.rf();
                f.ground_or_air = rg.rnd() & 1;
                f.xF8_playerNudgeVel.x = rg.rf();
                f.input.lstick[0].x = rg.rf();
                f.input.lstick[0].y = rg.rf();
                f.coll_data.ecb.top.y = rg.rf();
                f.coll_data.ecb.bottom.y = rg.rf();
                f.co_attrs.model_scaling = rg.rf();
                f.co_attrs.name_tag_height = rg.rf();
                f.co_attrs.camera_zoom_target_bone = (rg.rnd() % 8) as i32;
                f.co_attrs.x170.x = rg.rf();
                f.co_attrs.x170.y = rg.rf();
                f.co_attrs.x170.z = rg.rf();
                f.co_attrs.x12C = rg.rf();
                f.co_attrs.x13C = rg.rf();
                let nth = rg.rf();
                let czb = (rg.rnd() % 8) as i32;
                let sfx = if rg.rnd() & 1 != 0 { (rg.rnd() % 12000) as i32 } else { 5000 + (rg.rnd() % 5000) as i32 };
                f.dmg.x1830_percent = rg.rf();
                f.dmg.x18A4_knockbackMagnitude = rg.rf();
                f.dmg.x18B8 = if rg.rnd() & 1 != 0 { rg.rf() } else { 0.0 };
                f.dmg.x18BC = if rg.rnd() & 1 != 0 { rg.rf() } else { 0.0 };
                f.dmg.x18c4_source_ply = (rg.rnd() % 7) as i32;
                f.dmg.x18CC = rg.rnd() as i32;
                f.dmg.x18D0 = rg.rnd() as i32;
                f.dmg.x1958 = rg.rf();
                let itp = w.it.as_mut_ptr();
                let mut ri = || if rg.rnd() & 1 != 0 { unsafe { itp.add((rg.rnd() % 3) as usize) } } else { core::ptr::null_mut() };
                f.item_gobj = ri();
                f.x1978 = ri();
                f.x197C = ri();
                f.x1980 = ri();
                f.x1984_heldItemSpec = ri();
                f.x2024 = (rg.rnd() % 201) as i32 - 100;
                f.x209A = (rg.rnd() % 3) as u16;
                f.smash_attrs.state = rg.rnd() % 4;
                f.x2188.x = rg.rnd() as i32;
                f.x2188.y = rg.rnd() as i32;
                let tz = rg.rf();
                let bx = rg.rf();
                let by = rg.rf();
                let bz = rg.rf();
                let px = rg.rf();
                f.set_invisible((rg.rnd() & 1) as _);
                f.set_is_sandbag((rg.rnd() & 1) as _);
                f.set_is_sub_fighter((rg.rnd() & 1) as _);
                f.set_x221C_b6((rg.rnd() & 1) as _);
                f.set_x221D_b4((rg.rnd() & 1) as _);
                f.set_x221E_b1((rg.rnd() & 1) as _);
                f.set_x221E_b2((rg.rnd() & 1) as _);
                f.set_x221E_b3((rg.rnd() & 1) as _);
                f.set_x221E_b4((rg.rnd() & 1) as _);
                f.set_x221E_b5((rg.rnd() & 1) as _);
                f.set_x221F_b0((rg.rnd() & 1) as _);
                f.set_x221F_b1((rg.rnd() & 1) as _);
                f.set_x221F_b2((rg.rnd() & 1) as _);
                f.set_x221F_b3((rg.rnd() % 4 == 0) as _);
                f.set_x221F_b5((rg.rnd() & 1) as _);
                f.set_x2220_b7((rg.rnd() % 4 == 0) as _);
                f.set_x2224_b2((rg.rnd() & 1) as _);
                f.set_x2226_b4((rg.rnd() & 1) as _);
                f.set_x2229_b3((rg.rnd() % 4 == 0) as _);
                w.da[i].name_tag_height = nth;
                w.da[i].camera_zoom_target_bone = czb;
                w.sfx[i].x34 = sfx;
                w.cs[i].target_ext.v.z = tz;
                w.cs[i].bone_pos = Vec3 { x: bx, y: by, z: bz };
                w.cs[i].pos.x = px;
            }
            w.cd.x708 = rg.rf();
            w.cd.x70C = rg.rf();
            w.cd.x710 = rg.rf();
            for i in 0..4 {
                let c = &mut w.cd.x6DC_colorsByPlayer[i];
                c.r = rg.rnd() as u8;
                c.g = rg.rnd() as u8;
                c.b = rg.rnd() as u8;
                c.a = rg.rnd() as u8;
            }
            w.cd.x6D8[0] = rg.rnd() as i32;
            let n = rg.rnd() % (NF as u32 + 1);
            let mut used = 0u32;
            let mut prev: G = core::ptr::null_mut();
            for _ in 0..n {
                let mut j = (rg.rnd() % NF as u32) as usize;
                while used & (1 << j) != 0 {
                    j = (j + 1) % NF;
                }
                used |= 1 << j;
                let gj: G = &mut w.go[j];
                if prev.is_null() {
                    w.plinks[HSD_GOBJ_PLINK_FIGHTER as usize] = gj;
                } else {
                    unsafe { (*prev).next = gj };
                }
                prev = gj;
            }
            let g: G = &mut w.go[(rg.rnd() % NF as u32) as usize];
            let g2: G = {
                let q = rg.rnd() % 10;
                if (q as usize) < NF {
                    &mut w.go[q as usize]
                } else if q < 9 {
                    &mut w.it[q as usize - NF]
                } else {
                    core::ptr::null_mut()
                }
            };
            let a = rg.rf();
            let ia = (rg.rnd() % 16) as i32 - 4;
            let ib = rg.rnd() as i32;
            let wv = Vec3 { x: rg.rf(), y: rg.rf(), z: rg.rf() };
            let (q1, q2, q3) = (rg.rnd(), rg.rnd(), rg.rnd());
            let gn: G = if q1 & 3 != 0 { g } else { core::ptr::null_mut() };
            let gi: G = if q2 & 1 != 0 { &mut w.it[(q3 % 3) as usize] } else { g2 };
            let bu = |x: bool| x as u32;
            match op {
                0 => r = l::ftlib_800860c4() as u32,
                1 => r = bu(l::is_master_hand_present()),
                2 => r = bu(l::is_crazy_hand_present()),
                3 => r = e(l::ftlib_80086198(gn)),
                4 => r = e(l::ftlib_8008627c(&wv, gn)),
                5 => r = e(l::ftlib_80086368(&wv, gn, if q2 & 1 != 0 { -1.0 } else if q2 & 2 != 0 { 1.0 } else { a })),
                6 => r = b(l::ftlib_800864a8(&wv, gn)),
                7 => r = b(l::ftlib_800865c0(g)),
                8 => r = l::ftlib_800865cc(g) as u32,
                9 => {
                    let (mut x, mut y) = (0.0, 0.0);
                    l::ftlib_800865d8(g, &mut x, &mut y);
                    o1 = b(x);
                    o2 = b(y);
                }
                10 => r = e(l::ftlib_800865f0(g)),
                11 => r = e(l::ftlib_80086630(g, (q1 % 8) as i32)),
                12 => l::ftlib_80086644(g, &mut v),
                13 => l::ftlib_80086664(g, &wv),
                14 => l::ftlib_80086684(g, &mut v),
                15 => l::set_scale(g, a),
                16 => l::ftlib_800866dc(g, &mut v),
                17 => l::ftlib_80086724(g, g2),
                18 => l::ftlib_80086764(g),
                19 => r = e(l::get_item(g)),
                20 => r = bu(l::ftlib_800867a0(g, gi)),
                21 => r = e(l::ftlib_800867cc(g)),
                22 => r = bu(l::ftlib_800867d8(g)),
                23 => l::ftlib_800867e8(g),
                24 => l::ftlib_80086824(),
                25 => l::ftlib_8008688c(g),
                26 => l::ftlib_800868a4(),
                27 => r = bu(l::ftlib_800868d4(g, gi)),
                28 => r = bu(l::ftlib_80086960(g2)),
                29 => r = e(l::ftlib_80086984(g)),
                30 => l::ftlib_80086990(g, &mut v),
                31 => r = b(l::ftlib_800869d4(g)),
                32 => r = b(l::ftlib_800869f8(g)),
                33 => r = b(l::ftlib_80086a0c(g)),
                34 => r = bu(l::ftlib_80086a18(g)),
                35 => l::ftlib_80086a4c(g, a),
                36 => r = bu(l::ftlib_80086a58(g, &mut sv)),
                37 => r = bu(l::ftlib_80086a8c(g)),
                38 => l::update_logic_off_camera(g),
                39 => r = bu(l::ftlib_80086b64(g)),
                40 => r = e(l::ftlib_80086b74(g)),
                41 => r = b(l::ftlib_80086b80(g)),
                42 => l::ftlib_80086b90(g, &mut v),
                43 => r = bu(l::ftlib_80086bb4(g)),
                44 => r = l::ftlib_80086be0(g) as u32,
                45 => l::ftlib_80086bec(g, &mut v),
                46 => r = l::get_motion_id(g) as u32,
                47 => l::ftlib_80086c18(g, ia, ib),
                48 => l::ftlib_80086c9c(ia, ib),
                49 => l::ftlib_80086d40(g, ia, ib),
                50 => l::ftlib_80086dc4(ia, ib),
                51 => l::ftlib_80086e68(g),
                52 => r = l::ftlib_80086eb4(g) as u32,
                53 => r = bu(l::ftlib_80086ec0(g)),
                54 => r = bu(l::ftlib_80086ed0(g)),
                55 => r = bu(l::ftlib_80086f4c(g)),
                56 => r = b(l::ftlib_80086f80(g)),
                57 => r = bu(l::ftlib_80086fa8(g)),
                58 => {
                    let o: G = if q2 & 3 != 0 { &mut w.go[(q3 % NF as u32) as usize] } else { core::ptr::null_mut() };
                    r = bu(l::ftlib_80086fd4(gn, o))
                }
                59 => r = bu(l::ftlib_8008701c(g)),
                60 => l::ftlib_8008702c(ia),
                61 => l::ftlib_80087050(ia),
                62 => r = bu(l::ftlib_80087074(g, &mut v)),
                63 => {
                    let mut val = 0x55;
                    r = bu(l::ftlib_800870bc(g, &mut val));
                    o1 = val as u32;
                }
                64 => l::ftlib_800870f0(g, ia * 37),
                65 => r = l::ftlib_80087120(g) as u32,
                66 => l::ftlib_80087140(g),
                67 => r = bu(l::ftlib_80087284(g)),
                68 => r = l::get_kind(g),
                69 => r = e(l::ftlib_800872b0(g)),
                70 => r = bu(l::ftlib_800872bc(g)),
                71 => r = l::ftlib_80087300(g) as u32,
                72 => r = l::ftlib_8008730c(g) as u32,
                73 => r = l::ftlib_8008731c(g) as u32,
                74 | 75 => {
                    if op == 75 {
                        w.f[(q1 % NF as u32) as usize].motion_id = (q2 % 14) as i32;
                    }
                    r = bu(l::ftlib_8008732c(g))
                        | bu(l::ftlib_80087354(g)) << 1
                        | bu(l::ftlib_8008737c(g)) << 2
                        | bu(l::ftlib_800873a4(g)) << 3
                        | bu(l::ftlib_800873cc(g)) << 4;
                    if op == 74 {
                        r |= bu(l::ftlib_800873f4(g)) << 5;
                    }
                }
                76 => r = e(l::ftlib_8008741c(q1 % 9)),
                77 => r = b(l::ftlib_80087454(g)),
                78 => r = l::ftlib_80087460(g),
                79 => r = l::ftlib_8008746c(g) as u32,
                80 => r = l::ftlib_800874bc(g) as u32,
                81 => l::ftlib_800874cc(g, core::ptr::null_mut(), ib),
                82 => l::ftlib_800876b4(g),
                83 => r = bu(l::ftlib_800876d4(g)),
                84 => r = (l::ftlib_800876f4(g) as u32) ^ (l::ftlib_80087700(g) as u32),
                85 => {
                    l::ftlib_8008770c(g, core::ptr::null_mut());
                    l::ftlib_80087744(g, core::ptr::null_mut());
                }
                86 => r = bu(l::ftlib_800877d4(g)),
                _ => unreachable!(),
            }
            for i in 0..NF {
                let f = &w.f[i];
                let x = b(f.cur_pos.x) ^ b(f.cur_pos.y).wrapping_mul(3) ^ b(f.cur_pos.z).wrapping_mul(7);
                t_h = t_h.wrapping_mul(31).wrapping_add(x as i32);
                let x = b(f.x34_scale.y) ^ b(f.dmg.x1958).wrapping_mul(5) ^ b(f.dmg.x1830_percent).wrapping_mul(11);
                t_h = t_h.wrapping_mul(31).wrapping_add(x as i32);
                let x = (f.x221D_b4() as i32 | (f.x221F_b0() as i32) << 1).wrapping_add(f.x2188.x);
                t_h = t_h.wrapping_mul(31).wrapping_add(x);
            }
            let got = format!(
                "{} {} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X} {:08X}",
                op, k, r, o1, o2, sv.x as u32, b(v.x), b(v.y), b(v.z), t_h as u32, tr()
            );
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            seen.insert(op);
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "expected file has extra lines");
    assert_eq!(seen.len(), NOPS as usize);
    assert_eq!(checked, 10440);
}
