//! src/tu/ftcoll.rs vs the C (gen/cref/tu/ftcoll.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftcoll`).
//! Ops 0..9 (ftColl_80079AB0, 80079EA8, 80079C70, 80076528, 800765F0, 80076640, 8007AC68,
//! 8007A06C, 8007699C, 80076CBC); an assertion in the C (`HSD_ASSERTREPORT`) is the line
//! `op k ASSERT`, which the port reproduces as a panic.
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftcoll as m;
use m::ext as e;
use melee_fighter::types::*;

static mut RV: [f32; 8] = [0.0; 8];
static mut RVI: u32 = 0;
fn rv() -> f32 { unsafe { let v = RV[(RVI & 7) as usize]; RVI += 1; v } }

fn hit_rand(h: &mut HitCapsule) {
    *h = HitCapsule::zeroed();
    h.state = rnd() % 3;
    h.x4 = (rnd() & 1) as _;
    h.unk_count = rnd() % 20;
    h.damage = if rnd() & 1 != 0 { rw() } else { (rnd() % 30) as f32 + rg() };
    h.x24 = (rnd() % 200) as _;
    h.x28 = if rnd() & 1 != 0 { 0 } else { (rnd() % 150) as _ };
    h.x2C = (rnd() % 100) as _;
    h.kb_angle = if rnd() % 4 == 0 { 0x16A } else { (rnd() % 400) as _ };
    h.element = (rnd() % 17) as _;
    h.x34 = (rnd() % 20) as i32 - 5;
    h.sfx_severity = (rnd() % 3) as _;
    h.set_x40_b1((rnd() & 1) as _);
    h.hurt_coll_pos.x = rw(); h.hurt_coll_pos.y = rw(); h.hurt_coll_pos.z = rw();
    h.coll_distance = rw();
    if h.damage > 10.0 { h.damage = 10.0; }
}

fn cd_rand() {
    let c = cd();
    c.xF4 = rw(); c.xF8 = rw(); c.x108 = if rnd() & 1 != 0 { 100000.0 } else { rw() };
    c.x110 = rw(); c.x114 = rw(); c.x118 = rw(); c.x11C = rw(); c.x120 = rw();
    c.x6D4 = (rnd() % 300) as i32; c.x6D8 = (rnd() % 300) as i32;
    c.x128 = rw(); c.x714 = rw(); c.x3CC = (rnd() % 10) as i32;
    c.x3D0 = rw(); c.x3D4 = rw(); c.x4C8 = (rnd() % 4) as i32; c.x4D0 = rw(); c.x4D4 = rw();
    c.x7A8 = rw(); c.unk_kb_angle_min = rnd() % 400; c.unk_kb_angle_max = rnd() % 400;
    c.x1A4 = rw();
}

fn fp_rand(f: &mut Fighter, g: *mut HSD_GObj) {
    f.gobj = g;
    f.player_idx = (rnd() % 4) as _;
    f.set_x2225_b7((rnd() & 1) as _); f.set_stamina_dead((rnd() & 1) as _);
    f.dmg.x1830_percent = rw(); f.dmg.x1838_percentTemp = rw();
    f.dmg.x182c_behavior = rw(); f.dmg.x1834 = rw();
    f.dmg.x183C_applied = (rnd() % 20) as i32;
    f.dmg.int_value = (rnd() % 20) as i32;
    f.dmg.x1924 = (rnd() % 20) as i32;
    f.co_attrs.weight = rw();
    f.facing_dir = rg();
    f.cur_pos.x = rw(); f.cur_pos.y = rw();
    f.coll_data.floor.normal.x = rw(); f.coll_data.floor.normal.y = rw();
    f.x2090 = (rnd() % 4) as _; f.x2092 = (rnd() % 3) as _;
    f.ground_or_air = (rnd() & 1) as _;
    f.motion_id = if rnd() & 1 != 0 { ftCo_MS_DamageIce as _ } else { 0 };
    f.x221B.set_x221B_b5((rnd() & 1) as _); f.set_x221C_b4((rnd() & 1) as _); f.set_x221C_b2((rnd() & 1) as _);
    f.lightshield_amount = rw(); f.shield_unk0 = if rnd() & 1 != 0 { 0.0 } else { rw() };
    f.x19A4 = (rnd() % 20) as i32;
    f.x1988 = (rnd() % 4 == 0) as _; f.x198C = 0;
    f.kind = (rnd() % 30) as _;
}

fn cl(x: &mut f32) { if *x > 3.0 { *x = 3.0 } if *x < -3.0 { *x = -3.0 } }

fn install() {
    unsafe {
        e::SCRIPTGAME_ECHO_CAPSULE_COUNT = Some(|_| 4);
        e::SCRIPTGAME_ECHO_CAPSULE = Some(|fp, i| unsafe { &mut (*fp).x914[i as usize] as *mut _ });
        e::SCRIPTGAME_ECHO_IS_CAPSULE = Some(|_| 0);
        e::SCRIPTGAME_ECHO_BLOCKED = Some(|_, _| 0);
        e::SCRIPTGAME_ECHO_SAME_GROUP = Some(|a, b| unsafe { ((*a).x4 == (*b).x4) as i32 });
        e::SCRIPTGAME_ECHO_TARGET_ALLOWED = Some(|_, _| 0);
        e::SCRIPTGAME_ECHO_CONNECTED = Some(|_, _| t(40));
        e::SCRIPTGAME_ECHO_PREPARE = Some(|| {});
        e::SCRIPTGAME_ECHO_HIT_INDEX = Some(|fp, h| { t(41); unsafe { h.offset_from((*fp).x914.as_ptr()) as i32 } });
        e::SCRIPTGAME_ECHO_REPORT_CONTEXT = Some(|_, _, _| { t(42); 1 });
        e::SCRIPTGAME_ECHO_CREDIT = Some(|_, _, _, d| { t(43); t(b(d)); sr(4) as i32 });
        e::SCRIPTGAME_ECHO_SOURCE_X = Some(|_, fb| fb);
        e::SCRIPT_GAME_EVENT = Some(|a, _b, c, d, ee| { t(44); t(a as u32); t(c as u32); t(d as u32); t(ee as u32); });
        e::SCRIPTGAME_SKILL_PERFECT_SHIELD = Some(|_, _, _, _| t(45));
        e::SCRIPTGAME_INTR_WIN_HIT = Some(|_, k| { t(46); t(k as u32); });
        e::GENO_COMBO_CONTACT = Some(|_, k| { t(47); t(k as u32); });
        e::SCRIPTGAME_HIT_RULE_CONTACT = Some(|_, _, v, k| { t(48); t(k as u32); t(b(v)); v });
        e::SCRIPTGAME_FIGHTER_DEALT_DAMAGE = Some(|_, d| { t(49); d });
        e::SCRIPTGAME_HIT_RULE_WON = Some(|_, _| t(50));
        e::GENO_HIT_STUN_BONUS = Some(|_, i, _| { t(51); t(i as u32); });
        e::GENO_HIT_FLAGS = Some(|_, _, _| { t(52); if sr(5) { 4 } else { 0 } });
        e::SCRIPTGAME_HIT_RULE_PERCENT_QUEUE = Some(|_, _, d| { t(53); t(b(d)); });
        e::SCRIPTGAME_CRIT_HIT = Some(|_, _, _, _, _| t(54));
        e::SCRIPTGAME_REPORT_HIT_CONTEXT = Some(|_, _, el, _| { t(55); t(el as u32); });
        e::SCRIPTGAME_SKILL_HIT = Some(|_, _, _| t(56));
        e::SCRIPTGAME_HIT_RULE_CONTEXT = Some(|_, _| t(57));
        e::SCRIPT_CLANK_WANTED = Some(|| { t(58); sr(6) as i32 });
        e::SCRIPT_CLANK = Some(|a, b_, _c, _d, ee, f_, g, h, i, j, _k, _l| { t(59); t(a as u32); t(b_ as u32); t(ee as u32); t(f_ as u32); t(g as u32); t(h as u32); t(i as u32); t(j as u32); });
        e::SCRIPTMODS_CLANK = Some(|_, _, x, y| { t(60); t(b(x)); t(b(y)); });
        e::GENO_AUTOLINK = Some(|_, _, _, _, _| { t(61); 0 });
        e::MEX_ENABLED = Some(|_| 0);
        e::PLAYER_GET_DEFENSE_RATIO = Some(|_| { t(70); rv() });
        e::PLAYER_GET_ATTACK_RATIO = Some(|_| { t(71); rv() });
        e::GM_8016B248 = Some(|| { t(72); rv() });
        e::LBCOLL_80008688 = Some(|_, k, _| { t(73); t(k as u32); sr(7) });
        e::LBCOLL_80008820 = Some(|_, k, _| { t(74); t(k as u32); sr(8) });
        e::EFSYNC_SPAWN = Some(|id, _, _, _| { t(75); t(id as u32); });
        e::FTCO_800BFFD0 = Some(|_, a, _| { t(77); t(a as u32); false });
        e::FTCO_80094138 = Some(|_| t(76));
        e::PL_8003E150 = Some(|_, _| t(79));
        e::PLSTALE_UPDATE_STALE_MOVES_FROM_FIGHTER = Some(|_, _| t(80));
        e::PL_8003EB30 = Some(|d, _, _, _, _, _| { t(81); t(b(d)); });
        e::LBCOLL_80008D30 = Some(|h, a| unsafe {
            t(82);
            let x0 = *(a as *const i32);
            (*h).x24 = (x0.wrapping_mul(3)) as _; (*h).x28 = ((x0 as u32) & 3) as _;
            (*h).x2C = ((x0 as u32) % 7) as _; (*h).unk_count = (x0 as u32) % 5;
        });
        e::PL_80038144 = Some(|_, _, i, _, _, gr, _| { t(83); t(i as u32); t(gr as u32); });
        e::IT_GET_OWNER = Some(|_| { t(84); core::ptr::null_mut() });
        e::IT_GET_ATTACK_ID = Some(|_| 0);
        melee_fighter::hooks::PLAY_SFX = Some(|_, id, _, _| { t(78); t(id as u32); });
        melee_fighter::hooks::HSD_RANDI = Some(|n| { t(92); t(n as u32); if n > 0 { (rnd() % n as u32) as i32 } else { 0 } });
    }
}

#[test]
fn ftColl_matches_c() {
    init();
    install();
    let mut h0 = Box::new(HitCapsule::zeroed());
    let mut log: std::vec::Vec<DmgLogEntry> = (0..3).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut env: std::vec::Vec<[u64; 40]> = std::vec![[0; 40]; 3];
    let mut hurt: std::vec::Vec<FighterHurtCapsule> = (0..3).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut c = Check::new(include_str!("data/tu_ftcoll.txt"));
    for op in 0..10u32 {
        for k in 0..2000u32 {
            case(op, k);
            unsafe {
                for i in 0..8 { RV[i] = rw(); }
                if op >= 8 { for i in 0..8 { cl(&mut RV[i]); } }
                RVI = 0;
            }
            cd_rand();
            if op >= 8 {
                let c_ = cd();
                for x in [&mut c_.xF4, &mut c_.xF8, &mut c_.x110, &mut c_.x114, &mut c_.x118, &mut c_.x11C, &mut c_.x120,
                          &mut c_.x128, &mut c_.x714, &mut c_.x3D0, &mut c_.x3D4, &mut c_.x4D0, &mut c_.x4D4, &mut c_.x1A4] { cl(x); }
            }
            fp_rand(f(), go());
            fp_rand(f2(), go2());
            hit_rand(&mut h0);
            let mut h1 = HitCapsule::zeroed(); hit_rand(&mut h1);
            if op >= 8 {
                for fr in [f(), f2()] { cl(&mut fr.dmg.x182c_behavior); cl(&mut fr.dmg.x1834); cl(&mut fr.co_attrs.weight); }
            }
            for i in 0..4 { hit_rand(&mut f().x914[i]); hit_rand(&mut f2().x914[i]); }
            unsafe {
                melee_fighter::ftcoll::DMG_LOG0_IDX = (rnd() % 2) as i32;
                melee_fighter::ftcoll::DMG_LOG1_IDX = 0;
            }
            let mut h = Hs::new();
            let fp = f() as *mut Fighter; let fq = f2() as *mut Fighter;
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                let mut r = 0f32;
                match op {
                    0 => { let a = rnd() % 20; let (b1, b2, b3, b4) = (rw(), rw(), rw(), rw()); r = m::ftcoll_80079ab0(&*fp, &h0, a, b1, b2, b3, b4); }
                    1 => { let a = rnd() % 20; r = m::ftcoll_80079ea8(&*fp, &h0, a); }
                    2 => { let a = (rnd() % 20) as i32; r = m::ftcoll_80079c70(fp, fq, &mut *h0 as *mut _, a); }
                    3 => { m::ftcoll_80076528(go()); }
                    4 => {
                        let a = rw();
                        let v = if rnd() & 1 != 0 { go() } else { go2() };
                        r = m::ftcoll_800765f0(&*fp, v, a);
                        (*fp).victim_gobj = go2();
                        let a2 = rw();
                        r += m::ftcoll_800765f0(&*fp, go(), a2);
                    }
                    5 => {
                        let mut d = rw(); if d > 100.0 { d = 100.0 } if d < -100.0 { d = -100.0 }
                        if (*fp).dmg.x1834 < -300.0 { (*fp).dmg.x1834 = -300.0 }
                        t(m::ftcoll_80076640(&mut *fp, &mut d) as u32); r = d;
                    }
                    6 => { let a = rnd() % 400; t(m::ftcoll_8007ac68(a) as u32); }
                    7 => {
                        for i in 0..3 {
                            log[i] = core::mem::zeroed();
                            log[i].x0 = if rnd() & 1 != 0 { 1 } else { 3 };
                            log[i].gobj = go2();
                            log[i].kind = (rnd() % 30) as _;
                            hurt[i].capsule.a_pos.x = rw(); hurt[i].capsule.a_pos.y = rw();
                            hurt[i].capsule.b_pos.x = rw(); hurt[i].capsule.b_pos.y = rw();
                            hurt[i].height = (rnd() % 3) as _;
                            log[i].x10.hurt1 = &mut hurt[i] as *mut _;
                            if log[i].x0 == 1 {
                                log[i].xC.hit0 = &mut (*fq).x914[i] as *mut _;
                            } else {
                                let ep = env[i].as_mut_ptr() as *mut i32; *ep = (rnd() % 100) as i32;
                                *ep.add(2) = if rnd() & 1 != 0 { 0x16A } else { (rnd() % 360) as i32 };
                                *ep.add(6) = (rnd() % 17) as i32;
                                log[i].xC.unk_anim0 = ep as *mut _;
                            }
                            log[i].pos.x = rw(); log[i].pos.y = rw();
                            log[i].x20 = rw();
                            log[i].size_of_xC = (rnd() % 20) as _;
                        }
                        let mut out = [0f32; 16];
                        let fl = (rnd() & 1) as i32; let n = 1 + rnd() % 3;
                        m::ftcoll_8007a06c(go(), out.as_mut_ptr() as *mut _, log.as_mut_ptr(), n as usize, fl);
                        for i in 0..9 { h.m(out[i].to_bits()); }
                        h.m((*fp).x221C_b0() as u32); h.m((*fp).x1960_vibrateMult.to_bits());
                    }
                    8 => { let bb = (rnd() % 4) as usize; let a = (rnd() % 4) as usize; t(m::ftcoll_8007699c(fp, &mut (*fp).x914[a], fq, &mut (*fq).x914[bb]) as u32); }
                    9 => { let a = (rnd() % 4) as usize; m::ftcoll_80076cbc(fp, &mut (*fp).x914[a], fq); }
                    _ => {}
                }
                r
            }));
            let r = match res {
                Err(_) => { c.assert_line(op, k); continue; }
                Ok(v) => v,
            };
            let hf = |h: &mut Hs, v: f32| h.m(v.to_bits());
            let (a, bq) = (f(), f2());
            hf(&mut h, r);
            hf(&mut h, a.cur_pos.x); hf(&mut h, a.cur_pos.y); h.m(a.x2092 as u32);
            h.m(a.x221C_b4() as u32); hf(&mut h, a.dmg.x1834); hf(&mut h, a.dmg.x1838_percentTemp); h.m(a.dmg.x183C_applied as u32);
            h.m(a.dmg.int_value as u32); hf(&mut h, a.dmg.x191C); hf(&mut h, a.dmg.facing_dir); h.m(a.dmg.x1914 as u32);
            h.m(a.dmg.x1924 as u32); hf(&mut h, a.dmg.x1928); hf(&mut h, a.dmg.x192c); hf(&mut h, a.x1964);
            h.m(bq.dmg.int_value as u32); hf(&mut h, bq.dmg.x191C); hf(&mut h, bq.dmg.facing_dir); h.m(bq.x19A4 as u32);
            hf(&mut h, bq.specialn_facing_dir); h.m(bq.x19B0 as u32); h.m(bq.x19A0_shieldDamageTaken as u32); hf(&mut h, bq.x1964);
            h.m(bq.dmg.x1840 as u32); hf(&mut h, bq.dmg.x1834); hf(&mut h, bq.dmg.x1838_percentTemp); h.m(bq.dmg.x183C_applied as u32);
            h.m(bq.x221C_b4() as u32);
            let (i0, i1) = unsafe { (melee_fighter::ftcoll::DMG_LOG0_IDX, melee_fighter::ftcoll::DMG_LOG1_IDX) };
            h.m(i0 as u32); h.m(i1 as u32);
            for i in 0..44 { h.m(unsafe { m::ftColl_804D6560[i] } as u8 as u32); }
            for i in 0..(i0.max(0) as usize).min(4) { let l = unsafe { &m::dmg_log0[i] }; h.m(l.x0 as u32); hf(&mut h, l.x20); h.m(l.size_of_xC as u32); }
            for i in 0..(i1.max(0) as usize).min(4) { let l = unsafe { &m::dmg_log1[i] }; h.m(l.x0 as u32); hf(&mut h, l.x20); h.m(l.size_of_xC as u32); }
            c.line2(op, k, h.0);
        }
    }
    assert_eq!(c.done(), 20000);
}
