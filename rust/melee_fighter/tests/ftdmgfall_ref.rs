//! Group 3 round 14 vs the C (gen/cref/ftdmgfall_ref.c, cut verbatim, shipping contraction +
//! gwtool lowering; expected values in tests/data/ftdmgfall.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ftdmgfall`): the ftCo_DamageFall.c model-shift
//! leaves and ftparts.c table lookups of src/ftdmgfall.rs. Fill order matches the C driver exactly.
#![allow(static_mut_refs)]

use melee_fighter::ftdmgfall as m;
use melee_fighter::hooks as h;
use melee_fighter::types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rg(&mut self) -> f32 {
        match self.rnd() % 16 {
            0 => 0.0,
            1 => -0.0,
            2 => f32::from_bits(0x7FC0_0000),
            3 => 1.0,
            4 => -1.0,
            _ => ((self.rnd() % 41) as i32 - 20) as f32 / 20.0,
        }
    }
    fn rw(&mut self) -> f32 {
        const K: [f32; 4] = [1.0, 13.0, 1000.0, 100000.0];
        let a = self.rg();
        a * K[(self.rnd() % 4) as usize]
    }
}

static mut TR: u32 = 0;
fn t(v: u32) {
    unsafe { TR = TR.wrapping_mul(31).wrapping_add(v) }
}
static mut GO_PTR: *mut HSD_GObj = core::ptr::null_mut();
static mut SFXA_PTR: *const FtSFXArr = core::ptr::null();

struct Hs(u32);
impl Hs {
    fn m(&mut self, v: u32) {
        self.0 = self.0.wrapping_mul(0x0100_0193) ^ v;
    }
}

fn state_hash(f: &Fighter) -> u32 {
    let d = &f.dmg;
    let mut s = Hs(0x811C_9DC5);
    s.m(d.x18F8 as u32);
    s.m(d.x18fa_model_shift_frames as u32);
    s.m(d.x18FC as u32);
    s.m(d.x18FD as u32);
    s.m(d.x1900.to_bits());
    s.m(d.x1904.to_bits());
    s.m(d.x1908 as u32);
    s.m(d.x190C.is_null() as u32);
    s.0
}

#[test]
fn ftdmgfall_matches_c() {
    unsafe {
        h::FT_80081DD4 = Some(|g| { t(1); t((g == GO_PTR) as u32) });
        h::PLAY_SFX = Some(|_, id, vol, pan| { t(2); t(id as u32); t(vol as u8 as u32); t(pan as u8 as u32) });
        h::FT_80084DB0 = Some(|g| { t(3); t((g == GO_PTR) as u32) });
        h::FT_800889F4 = Some(|_, a| { t(4); t((a == SFXA_PTR) as u32) });
    }
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut go: Box<HSD_GObj> = Box::new(HSD_GObj::zeroed());
    let mut cd: Box<ftCommonData> = Box::new(ftCommonData::zeroed());
    let mut sfxa: Box<FtSFXArr> = Box::new(FtSFXArr::zeroed());
    let mut sh: Box<[[Vec2; 8]; 3]> = Box::new([[Vec2 { x: 0.0, y: 0.0 }; 8]; 3]);
    let mut t6530: Box<[*mut Vec2; 6]> = Box::new([core::ptr::null_mut(); 6]);
    let mut j2p: Box<[[u8; 64]; 4]> = Box::new([[0; 64]; 4]);
    let mut p2j: Box<[[u8; 64]; 4]> = Box::new([[0; 64]; 4]);
    let mut pt: Box<[FighterPartsTable; 4]> = Box::new([FighterPartsTable::zeroed(); 4]);
    let mut ptp: Box<[*mut FighterPartsTable; 4]> = Box::new([core::ptr::null_mut(); 4]);
    let mut e40: Box<[[Fighter_804D6540_x0_t; 6]; 4]> = Box::new([[Fighter_804D6540_x0_t::zeroed(); 6]; 4]);
    let mut t40: Box<[Fighter_804D6540_t; 4]> = Box::new([Fighter_804D6540_t::zeroed(); 4]);
    let mut t40p: Box<[*mut Fighter_804D6540_t; 4]> = Box::new([core::ptr::null_mut(); 4]);
    let gp: *mut HSD_GObj = &mut *go;
    let sfxp: *mut FtSFXArr = &mut *sfxa;
    go.user_data = &mut *f as *mut Fighter as *mut core::ffi::c_void;
    unsafe {
        GO_PTR = gp;
        SFXA_PTR = sfxp;
        h::P_FT_COMMON_DATA = &*cd;
        h::FIGHTER_804D6530 = t6530.as_mut_ptr();
        h::FT_PARTS_TABLE = ptp.as_mut_ptr();
        h::FIGHTER_804D6540 = t40p.as_mut_ptr();
    }
    let expected = include_str!("data/ftdmgfall.txt");
    let mut lines = expected.lines();
    let mut checked = 0;
    for op in 0..10u32 {
        for k in 0..1500u32 {
            *f = Fighter::zeroed();
            *cd = ftCommonData::zeroed();
            unsafe { TR = 0 };
            let mut r = Rng(0x9059u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            for i in 0..3 {
                for j in 0..8 {
                    sh[i][j].x = r.rg();
                    sh[i][j].y = r.rg();
                }
                t6530[i * 2] = sh[i].as_mut_ptr();
                t6530[i * 2 + 1] = r.rnd() as usize as *mut Vec2;
            }
            for i in 0..4 {
                for j in 0..64 {
                    j2p[i][j] = if r.rnd() % 5 == 0 { 0xFF } else { (r.rnd() % 64) as u8 };
                    p2j[i][j] = r.rnd() as u8;
                }
                pt[i].joint_to_part = j2p[i].as_mut_ptr();
                pt[i].part_to_joint = p2j[i].as_mut_ptr();
                pt[i].parts_num = r.rnd() % 65;
                ptp[i] = if r.rnd() % 4 == 0 { core::ptr::null_mut() } else { &mut pt[i] };
                for j in 0..6 {
                    e40[i][j].x0 = (r.rnd() % 8) as u8;
                    e40[i][j].x1 = r.rnd() as u8;
                }
                t40[i].x0 = e40[i].as_mut_ptr();
                t40[i].x4 = (r.rnd() % 7) as i32;
                t40p[i] = if r.rnd() % 4 == 0 { core::ptr::null_mut() } else { &mut t40[i] };
            }
            cd.x168 = r.rw();
            cd.x16C = r.rw();
            cd.x198 = r.rw();
            cd.x19C = r.rw();
            cd.x1A0 = r.rw();
            f.kind = (r.rnd() % 4) as _;
            f.motion_id = if r.rnd() % 4 == 0 { ftCo_MS_DamageIce as _ } else { (r.rnd() % 400) as _ };
            f.facing_dir = r.rg();
            f.coll_data.floor.normal.x = r.rg();
            f.coll_data.floor.normal.y = r.rg();
            f.dmg.x18F8 = (r.rnd() % 3) as u8;
            f.dmg.x18fa_model_shift_frames = if r.rnd() & 1 != 0 { 0 } else { r.rnd() as u16 };
            f.dmg.x18FC = (r.rnd() % 8) as u8;
            f.dmg.x18FD = r.rnd() as u8;
            f.dmg.x1900 = r.rg();
            f.dmg.x1904 = r.rg();
            f.dmg.x1908 = if r.rnd() & 1 != 0 { -1 } else { (r.rnd() % 0x200) as _ };
            f.dmg.x190C = if r.rnd() & 1 != 0 { core::ptr::null_mut() } else { sfxp as *mut core::ffi::c_void };
            let mul = r.rw();
            let elem = (r.rnd() % 14) as i32;
            let dmg = (r.rnd() % 120) as i32 - 20;
            let msid = if r.rnd() & 1 != 0 { ftCo_MS_Squat as i32 - 1 + (r.rnd() % 4) as i32 } else { (r.rnd() % 400) as i32 };
            let ga = (r.rnd() % 3) as i32;
            let part = (r.rnd() % 64) as i32;
            let kind = (r.rnd() % 4) as i32;
            let (to, from, joint) = (r.rnd() % 4, r.rnd() % 4, r.rnd() % 80);
            let fr: &mut Fighter = unsafe { &mut *((*gp).user_data as *mut Fighter) };
            let ret: u32 = match op {
                0 => { m::ftco_80090574(gp); 0 }
                1 => m::calc_shift(mul, msid, dmg) as u32,
                2 => { m::ftco_80090594(fr, elem, dmg, msid, ga, mul); 0 }
                3 => match m::ftco_80090690(fr) {
                    None => 0xFFFF_FFFF,
                    Some(s) => s.x.to_bits().wrapping_mul(31).wrapping_add(s.y.to_bits()) ^ 1,
                },
                4 => { m::ftco_80090718(fr); 0 }
                5 => { melee_fighter::tu::ftCo_DamageFall::ftCo_DamageFall_Anim(gp); 0 }
                6 => { melee_fighter::tu::ftCo_DamageFall::ftCo_DamageFall_Phys(gp); 0 }
                7 => if !ptp[fr.kind as usize].is_null() { m::ftparts_get_bone_index(fr, part) as u32 } else { 0xEEEE_EEEE },
                8 => m::ftparts_remap(to, from, joint) as u32,
                9 => m::ftparts_8007506c(kind, part % 9),
                _ => unreachable!(),
            };
            let got = format!("{} {} {:08X} {:08X} {:08X}", op, k, ret, state_hash(&f), unsafe { TR });
            let want = lines.next().expect("ftdmgfall.txt too short");
            assert_eq!(got, want, "op {op} case {k}");
            checked += 1;
        }
    }
    assert!(lines.next().is_none(), "ftdmgfall.txt has extra lines");
    assert_eq!(checked, 15000);
}
