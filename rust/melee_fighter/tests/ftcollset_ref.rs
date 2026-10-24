//! `tu::ftcoll` (the ftcoll.c capsule setters) and `melee_runtime::tu::script_hit` (the hit-rule
//! metadata table) vs the C (gen/cref/ftcollset_ref.c, the functions cut verbatim, shipping
//! contraction + gwtool lowering; expected values in tests/data/ftcollset.txt; regenerate with
//! `python rust/melee_fighter/gen/cref/run.py ftcollset`). Fill order matches the C driver exactly.
#![allow(static_mut_refs)]

use melee_fighter::tu::ftchangeparam::ext as cp;
use melee_fighter::tu::ftcoll as co;
use melee_fighter::types::*;
use melee_runtime::tu::script_hit as sh;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        let k = self.rnd() % 16;
        if k == 0 {
            return 0.0;
        }
        if k == 1 {
            return -0.0;
        }
        if k == 2 {
            return 1.0;
        }
        ((self.rnd() % 2001) as i32 - 1000) as f32 / 77.0
    }
}

static mut BFF_LOG: u32 = 0;

struct World {
    f: Box<Fighter>,
    go: Box<HSD_GObj>,
    bones: Box<[FighterBone; 8]>,
    jo: Box<[HSD_JObj; 8]>,
    hb: Box<HitCapsule>,
}

impl World {
    fn jid(&self, j: *mut HSD_JObj) -> u32 {
        if j.is_null() {
            return 0;
        }
        let base = self.jo.as_ptr() as *mut HSD_JObj;
        (0..8).find(|&i| j == unsafe { base.add(i) }).map_or(99, |i| 1 + i as u32)
    }

    fn state(&self) -> u32 {
        let f = &*self.f;
        let b2u = |x: f32| x.to_bits();
        let mut h: u32 = 0;
        let mut hm = |v: u32| h = h.wrapping_mul(31).wrapping_add(v);
        let vid = |p: *mut core::ffi::c_void| (!p.is_null()) as u32;
        for i in 0..4 {
            hm(f.x914[i].state as u32);
            for j in 0..12 {
                hm(vid(f.x914[i].victims_1[j].victim.cast()));
                hm(vid(f.x914[i].victims_2[j].victim.cast()));
            }
            hm(f.x914[i].x44 as u32);
            hm(f.x914[i].x45 as u32);
        }
        hm(f.hurt_capsules_len as u32);
        for i in 0..15 {
            let hc = &f.hurt_capsules[i];
            hm(hc.capsule.state as u32);
            hm(hc.capsule.skip_update_pos() as u32);
            hm(hc.capsule.bone_idx as u32);
            hm(self.jid(hc.capsule.bone));
            for v in [hc.capsule.a_offset, hc.capsule.b_offset] {
                hm(b2u(v.x));
                hm(b2u(v.y));
                hm(b2u(v.z));
            }
            hm(b2u(hc.capsule.scale));
            hm(hc.height as u32);
            hm(hc.is_grabbable as u32);
        }
        hm(f.x221A_b5() as u32);
        hm(f.x221A_b6() as u32);
        hm(f.x2219_b3() as u32);
        let a = unsafe { f.anon_221b };
        hm(a.x221B_b0() as u32);
        hm(a.x221B_b1() as u32);
        hm(a.x221B_b2() as u32);
        hm(a.x221B_b3() as u32);
        hm(a.x221B_b4() as u32);
        hm(f.shield_hit_cb.map_or(0, |c| c as usize as u32));
        hm(self.jid(f.shield_hit.bone));
        hm(b2u(f.shield_hit.size));
        hm(b2u(f.shield_hit.offset.x));
        hm(b2u(f.shield_hit.offset.y));
        hm(b2u(f.shield_hit.offset.z));
        hm(f.x1988 as u32);
        hm(f.x198C as u32);
        hm(f.x1990 as u32);
        hm(f.x2218_b4() as u32);
        hm(f.x2218_b5() as u32);
        hm(f.x2218_b6() as u32);
        hm(f.x2218_b7() as u32);
        hm(f.reflecting() as u32);
        hm(f.reflect_hit_cb.map_or(0, |c| c as usize as u32));
        hm(self.jid(f.reflect_hit.bone));
        hm(b2u(f.reflect_hit.size));
        hm(b2u(f.reflect_hit.offset.x));
        hm(b2u(f.reflect_hit.offset.y));
        hm(b2u(f.reflect_hit.offset.z));
        hm(f.ReflectAttr.x1A30_maxDamage as u32);
        hm(b2u(f.ReflectAttr.x1A34_damageMul));
        hm(b2u(f.ReflectAttr.x1A38_speedMul));
        hm(self.jid(f.absorb_hit.bone));
        hm(b2u(f.absorb_hit.size));
        hm(b2u(f.absorb_hit.offset.x));
        hm(b2u(f.absorb_hit.offset.y));
        hm(b2u(f.absorb_hit.offset.z));
        hm(unsafe { BFF_LOG });
        hm(self.hb.unk_count);
        hm(b2u(self.hb.damage));
        unsafe {
            let m = sh::script_hit_metadata(&*self.hb as *const HitCapsule as *mut HitCapsule, 0);
            hm(sh::script_hit_meta_count as u32);
            hm(!m.is_null() as u32);
            if !m.is_null() {
                hm(b2u((*m).original_damage));
                hm((*m).original_count);
                hm(b2u((*m).damage_scale));
            }
        }
        h
    }
}

#[test]
fn ftcollset_matches_c() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ftcollset.txt")).unwrap();
    let mut lines = text.lines();
    let mut w = World {
        f: Box::new(Fighter::zeroed()),
        go: Box::new(HSD_GObj::zeroed()),
        bones: Box::new(unsafe { core::mem::zeroed() }),
        jo: Box::new(unsafe { core::mem::zeroed() }),
        hb: Box::new(unsafe { core::mem::zeroed() }),
    };
    let mut d6524: Box<Fighter_804D6524_t> = Box::new(unsafe { core::mem::zeroed() });
    unsafe {
        co::ext::FTCO_800BFFD0 = Some(|_fp, a, b| {
            BFF_LOG = BFF_LOG.wrapping_mul(31).wrapping_add(a as u32);
            BFF_LOG = BFF_LOG.wrapping_mul(31).wrapping_add(b as u32);
            false
        });
    }
    let mut n = 0;
    for op in 0..11u32 {
        for k in 0..400u32 {
            *w.f = Fighter::zeroed();
            *w.go = HSD_GObj::zeroed();
            *w.hb = unsafe { core::mem::zeroed() };
            unsafe {
                core::ptr::write_bytes(sh::script_hit_meta.as_mut_ptr(), 0, 1024);
                sh::script_hit_meta_count = 0;
                BFF_LOG = 0;
                melee_fighter::stale::DB_LEVEL = melee_fighter::stale::DBL_KIND_DEBUG_ROM;
            }
            let mut rg = Rng(0x51F3u32.wrapping_add(op.wrapping_mul(7919)).wrapping_add(k.wrapping_mul(104729)));
            let gp: *mut HSD_GObj = &mut *w.go;
            let fpp: *mut Fighter = &mut *w.f;
            w.go.user_data = fpp.cast();
            w.f.gobj = gp;
            let jbase = w.jo.as_mut_ptr();
            for i in 0..8 {
                w.bones[i].joint = unsafe { jbase.add(i) };
                w.bones[i].x4_jobj2 = unsafe { jbase.add((i + 1) & 7) };
            }
            w.f.parts = w.bones.as_mut_ptr();
            w.f.kind = (rg.rnd() % 40) as _;
            w.f.x34_scale.y = if rg.rnd() % 3 == 0 { 1.0 } else { rg.rf() };
            if w.f.x34_scale.y == 0.0 {
                w.f.x34_scale.y = 1.0;
            }
            w.f.x2068_attackID = (rg.rnd() % 20) as i32;
            w.f.x206C_attack_instance = rg.rnd() as u16;
            w.f.smash_attrs.state = if rg.rnd() % 3 == 0 { SmashState_Release as _ } else { 0 };
            w.f.smash_attrs.x2120_damageMul = rg.rf();
            w.f.smash_attrs.x2118_frames = rg.rf();
            w.f.smash_attrs.x211C_holdFrame = rg.rf();
            d6524.x4 = rg.rf();
            unsafe { cp::Fighter_804D6524 = &*d6524 };
            for i in 0..4 {
                w.f.x914[i].state = (rg.rnd() % 2) as _;
                for j in 0..12 {
                    w.f.x914[i].victims_1[j].victim = if rg.rnd() & 1 != 0 { unsafe { jbase.add(j & 7) }.cast() } else { core::ptr::null_mut() };
                    w.f.x914[i].victims_2[j].victim = if rg.rnd() & 1 != 0 { unsafe { jbase.add(j & 7) }.cast() } else { core::ptr::null_mut() };
                }
                w.f.x914[i].x44 = rg.rnd() as _;
                w.f.x914[i].x45 = rg.rnd() as _;
            }
            w.f.hurt_capsules_len = (rg.rnd() % 16) as u8;
            for i in 0..15 {
                let hc = &mut w.f.hurt_capsules[i];
                hc.capsule.state = (rg.rnd() % 3) as _;
                hc.capsule.set_skip_update_pos((rg.rnd() & 1) as u8);
                hc.capsule.bone_idx = (rg.rnd() % 8) as i32;
                hc.capsule.scale = rg.rf();
            }
            w.f.set_x221A_b5((rg.rnd() & 1) as u8);
            w.f.set_x2219_b3((rg.rnd() & 1) as u8);
            w.f.set_x221A_b6((rg.rnd() & 1) as u8);
            w.f.x1988 = (rg.rnd() % 4) as _;
            w.f.x198C = rg.rnd() as i32;
            w.f.x1990 = (rg.rnd() % 50) as i32;
            let mut sd: ShieldDesc = unsafe { core::mem::zeroed() };
            sd.bone = (rg.rnd() % 8) as i32;
            sd.pos.x = rg.rf();
            sd.pos.y = rg.rf();
            sd.pos.z = rg.rf();
            sd.radius = rg.rf();
            let mut ad: AbsorbDesc = unsafe { core::mem::zeroed() };
            ad.x0_bone_id = (rg.rnd() % 8) as i32;
            ad.x4_offset.x = rg.rf();
            ad.x4_offset.y = rg.rf();
            ad.x4_offset.z = rg.rf();
            ad.x10_size = rg.rf();
            let mut rd: ReflectDesc = unsafe { core::mem::zeroed() };
            rd.x0_bone_id = rg.rnd() % 8;
            rd.x4_max_damage = (rg.rnd() % 100) as i32;
            rd.x8_offset.x = rg.rf();
            rd.x8_offset.y = rg.rf();
            rd.x8_offset.z = rg.rf();
            rd.x14_size = rg.rf();
            rd.x18_damage_mul = rg.rf();
            rd.x1C_speed_mul = rg.rf();
            rd.x20_behavior = (rg.rnd() % 4) as u8;
            let mut hi: ftHurtboxInit = unsafe { core::mem::zeroed() };
            hi.bone_idx = rg.rnd() % 8;
            hi.height = (rg.rnd() % 3) as _;
            hi.is_grabbable = rg.rnd() & 1;
            hi.a_offset.x = rg.rf();
            hi.a_offset.y = rg.rf();
            hi.a_offset.z = rg.rf();
            hi.b_offset.x = rg.rf();
            hi.b_offset.y = rg.rf();
            hi.b_offset.z = rg.rf();
            hi.scale = rg.rf();
            let cb: HSD_GObjEvent = if rg.rnd() & 1 != 0 {
                Some(unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(*mut HSD_GObj)>(0x1000 + (rg.rnd() % 64) as usize) })
            } else {
                None
            };
            w.hb.unk_count = rg.rnd();
            w.hb.damage = rg.rf();
            w.hb.element = rg.rnd() % 15;
            if rg.rnd() & 1 != 0 {
                unsafe {
                    let m = sh::script_hit_metadata(&mut *w.hb, 1);
                    (*m).damage_scale = if rg.rnd() & 1 != 0 { 1.0 } else { rg.rf() };
                    (*m).original_damage = rg.rf();
                }
            }
            let hbp: *mut HitCapsule = &mut *w.hb;
            match op {
                0 => co::ftColl_8007ABD0(hbp, rg.rnd() % 200, gp),
                1 => co::ftColl_8007AFF8(gp),
                2 => co::ftColl_8007B064(gp, (rg.rnd() % 4) as i32),
                3 => co::ftColl_8007B0C0(gp, rg.rnd() % 3),
                4 => {
                    let mut bone = if rg.rnd() % 4 == 0 {
                        9
                    } else {
                        let n = if w.f.hurt_capsules_len != 0 { w.f.hurt_capsules_len as u32 } else { 1 };
                        w.f.hurt_capsules[(rg.rnd() % n) as usize].capsule.bone_idx
                    };
                    if w.f.hurt_capsules_len == 0 || bone == 9 {
                        w.f.hurt_capsules_len = 1;
                        w.f.hurt_capsules[0].capsule.bone_idx = 3;
                        bone = 3;
                    }
                    co::ftColl_8007B128(gp, bone, rg.rnd() % 3);
                }
                5 => co::ftColl_8007B1B8(gp, &sd, cb),
                6 => co::ftColl_8007B62C(gp, (rg.rnd() % 4) as i32),
                7 => co::ftColl_8007B760(gp, (rg.rnd() % 60) as i32),
                8 => co::ftColl_CreateAbsorbHit(gp, &ad),
                9 => co::ftColl_CreateReflectHit(gp, &rd, cb),
                10 => {
                    let idx = (rg.rnd() % 15) as usize;
                    let hp: *mut FighterHurtCapsule = &mut w.f.hurt_capsules[idx];
                    co::ftColl_HurtboxInit(fpp, hp, &hi)
                }
                _ => unreachable!(),
            }
            let ret = {
                let f = &*w.f;
                let a = unsafe { f.anon_221b };
                let b = [
                    f.x221A_b5(), f.x221A_b6(), f.x2219_b3(), a.x221B_b0(), a.x221B_b1(), a.x221B_b2(), a.x221B_b3(),
                    a.x221B_b4(), f.x2218_b4(), f.x2218_b5(), f.x2218_b6(), f.x2218_b7(),
                ];
                let mut r = 0u32;
                for v in b {
                    r = r.wrapping_mul(31).wrapping_add(v as u32);
                }
                r.wrapping_mul(31).wrapping_add(f.reflecting() as u32)
            };
            let got = format!("{} {} {:08X} {:08X}", op, k, ret, w.state());
            let want = lines.next().expect("expected file too short");
            assert_eq!(got, want, "op {op} case {k}");
            n += 1;
        }
    }
    assert!(lines.next().is_none());
    assert_eq!(n, 4400);
}
