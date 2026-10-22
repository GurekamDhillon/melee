//! tu::ftkirbyspecialpikachu vs the C (gen/turef/tu_ftkirbyspecialpikachu_ref.c, shipping contraction; data
//! in tests/data/tu_ftkirbyspecialpikachu.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialpikachu`): the jolt spawn position
//! (4 fused sites per Anim) and the Anim control flow, plus a smoke test of Enter / Coll.
//! One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialpikachu as t;
use melee_fighter::hooks as fh;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        match self.rnd() % 8 {
            0 => 0.0,
            1 => -0.0,
            2 => 1.0,
            3 => -1.0,
            _ => ((self.rnd() % 4001) as i32 - 2000) as f32 / 997.0,
        }
    }
}

static mut EV: std::vec::Vec<u32> = std::vec::Vec::new();
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut P: [u32; 4] = [0; 4];
static mut FRAMES: i32 = 0;

fn ev(v: u32) {
    unsafe { EV.push(v) }
}

fn flush(tag: &str, i: usize, fp: &Fighter) -> String {
    let (p, evs) = unsafe { (P, std::mem::take(&mut EV)) };
    let mut s = format!("{} {} {:08x} {:08x} {:08x} {:08x} {} {} |", tag, i, p[0], p[1], p[2], p[3], fp.cmd_vars[0], fp.cmd_vars[1]);
    for e in evs {
        s += &format!(" {:x}", e);
    }
    unsafe { P = [0; 4] };
    s
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialpikachu.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        t::ext::ITPIKACHUTHUNDERJOLT_SPAWN = Some(|_, p, dir, kind| unsafe {
            P = [(*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits(), dir.to_bits()];
            ev(0x10);
            ev(kind as u32);
            core::ptr::null_mut()
        });
        fh::PLAY_SFX = Some(|_, id, vol, pan| {
            ev(0x20);
            ev(id as u32);
            ev(vol as u32);
            ev(pan as u32);
        });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| unsafe { FRAMES });
        st::FT_8008A2BC = Some(|_| ev(0x30));
        st::FTCO_FALL_ENTER = Some(|_| ev(0x31));
        st::FTCO_80096900 = Some(|_, a, b, c, d, e| {
            ev(0x32);
            ev(a as u32);
            ev(b as u32);
            ev(c as u32);
            ev(d.to_bits());
            ev(e.to_bits());
        });
    }
    let mut r = Rng(0x91CA);
    let mut lines = want.lines();
    for i in 0..2000 {
        da.specialn_pk_spawn_offset.x = r.rf();
        da.specialn_pk_spawn_offset.y = r.rf();
        da.specialn_pc_spawn_offset.x = r.rf();
        da.specialn_pc_spawn_offset.y = r.rf();
        da.specialairn_pk_spawn_offset.x = r.rf();
        da.specialairn_pk_spawn_offset.y = r.rf();
        da.specialairn_pc_spawn_offset.x = r.rf();
        da.specialairn_pc_spawn_offset.y = r.rf();
        da.specialn_pk_itkind = (r.rnd() % 7) as _;
        da.specialn_pc_itkind = (r.rnd() % 7) as _;
        da.specialairn_pk_landing_lag = if r.rnd() % 3 != 0 { r.rf() } else { 0.0 };
        da.specialairn_pc_landing_lag = if r.rnd() % 3 != 0 { r.rf() } else { 0.0 };
        f.x34_scale.y = r.rf();
        f.cur_pos.x = r.rf();
        f.cur_pos.y = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        let k = r.rnd() % 3;
        unsafe {
            f.u.kb.hat.kind = if k == 0 { Ft_Kind_Pikachu } else if k == 1 { Ft_Kind_Pichu } else { Ft_Kind_Kirby };
            FRAMES = (r.rnd() & 1) as i32;
        }
        f.cmd_vars[0] = r.rnd() % 3;
        f.cmd_vars[1] = r.rnd() % 3;
        t::ftKb_PkSpecialN_Anim(gp);
        assert_eq!(flush("G", i, &f), lines.next().unwrap(), "G case {i}");
        f.cmd_vars[0] = r.rnd() % 3;
        f.cmd_vars[1] = r.rnd() % 3;
        t::ftKb_PkSpecialAirN_Anim(gp);
        assert_eq!(flush("H", i, &f), lines.next().unwrap(), "H case {i}");
    }
    assert!(lines.next().is_none());

    // smoke: Enter (Pichu picks the Pc state, cmd vars cleared), the Coll ground <-> air changes.
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, _, _, _| unsafe { LOG.push(format!("ms {ms} {fl:x} {a}")) });
        st::FTANIM_8006EBA4 = Some(|_| unsafe { LOG.push("anim".into()) });
        st::FT_80082708 = Some(|_| 0);
        st::FT_80081D0C = Some(|_| 1);
        st::FTCOMMON_8007D7FC = Some(|_| unsafe { LOG.push("land".into()) });
        f.u.kb.hat.kind = Ft_Kind_Pichu;
    }
    f.cmd_vars = [1, 2, 3, 4];
    t::ftKb_SpecialNPk_800F9FD4(gp);
    assert_eq!(f.cmd_vars, [0, 0, 0, 0]);
    f.cur_anim_frame = 5.0;
    t::ftKb_PkSpecialN_Coll(gp);
    f.self_vel.y = 3.0;
    t::ftKb_PkSpecialAirN_Coll(gp);
    assert_eq!(f.self_vel.y, 0.0);
    let got = unsafe { LOG.join(",") };
    assert_eq!(
        got,
        format!(
            "ms {} 0 0,anim,ms {} {:x} 5,land,ms {} {:x} 5",
            ftKb_MS_PcSpecialN, ftKb_MS_PcSpecialAirN, t::coll_mf, ftKb_MS_PcSpecialN, t::coll_mf
        )
    );
}
