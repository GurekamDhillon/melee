//! tu::ftkirbyspecialn vs the C (gen/turef/tu_ftkirbyspecialn_ref.c, shipping contraction; data in
//! tests/data/tu_ftkirbyspecialn.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialn`), plus a smoke test of the
//! Inhale / Eat states through recording hooks. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialn as t;
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

static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();
static mut EV: std::vec::Vec<u32> = std::vec::Vec::new();
static mut POS: [u32; 3] = [0; 3];
static mut ITEM_RET: f64 = 0.0;
static mut FIGHTER_RET: f32 = 0.0;

fn ev(v: u32) {
    unsafe { EV.push(v) }
}

fn flush(tag: &str, i: usize, fp: &mut Fighter) -> String {
    let (p, evs) = unsafe { (POS, std::mem::take(&mut EV)) };
    let mut s = format!("{} {} {:08x} {:08x} {:08x} {:x} |", tag, i, p[0], p[1], p[2], fp.x1A6A);
    for e in evs {
        s += &format!(" {:x}", e);
    }
    fp.x1A6A = 0;
    s
}

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialn.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut tg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut vg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    g.user_data = &mut *f as *mut Fighter as *mut _;
    f.gobj = &mut *g;
    f.target_item_gobj = &mut *tg;
    f.victim_gobj = &mut *vg;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| {
            ev(0x100);
            ev(ms as u32);
            ev(fl);
        });
        t::ext::IT_802F23AC = Some(|_, p| unsafe {
            POS = [(*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits()];
            ITEM_RET
        });
        t::ext::IT_802F2810 = Some(|_| ev(1));
        t::ext::FTCO_800BD19C = Some(|_, p| unsafe {
            POS = [(*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits()];
            FIGHTER_RET
        });
        t::ext::FTCO_800BD620 = Some(|_| ev(2));
    }
    let mut r = Rng(0x5EED5);
    let mut lines = want.lines();
    for i in 0..1500 {
        da.specialn_x_offset_inhaled = r.rf();
        da.specialn_y_offset_inhaled = r.rf();
        da.specialn_inhale_velocity = r.rf() * 4.0;
        da.specialn_spit_out_release_angle = r.rf() * 3.0;
        da.specialn_swallow_star_vertical_velocity = r.rf();
        da.specialn_swallow_star_gravity = r.rf();
        f.cur_pos.x = r.rf();
        f.cur_pos.y = r.rf();
        f.cur_pos.z = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        f.ground_or_air = (r.rnd() & 1) as _;
        let d = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        let mut v = Vec3 { x: 7.0, y: 7.0, z: 7.0 };
        let ret = t::ftKb_SpecialN_800F58D8(gp, &mut v, d);
        let got = format!("A {} {:08x} {:08x} {:08x} {:08x}", i, ret.to_bits(), v.x.to_bits(), v.y.to_bits(), v.z.to_bits());
        assert_eq!(got, lines.next().unwrap(), "A case {i}");
        let mut o = Vec3 { x: 7.0, y: 7.0, z: 7.0 };
        t::ftKb_SpecialN_800F5B5C(gp, &mut o);
        let got = format!("B {} {:08x} {:08x} {:08x}", i, o.x.to_bits(), o.y.to_bits(), o.z.to_bits());
        assert_eq!(got, lines.next().unwrap(), "B case {i}");
        unsafe {
            ITEM_RET = r.rf() as f64 * 3.0;
            FIGHTER_RET = r.rf() * 3.0;
        }
        t::ftKb_SpecialN_800F5DE8(gp);
        assert_eq!(flush("C", i, &mut f), lines.next().unwrap(), "C case {i}");
        t::ftKb_SpecialN_800F5EA8(gp);
        assert_eq!(flush("D", i, &mut f), lines.next().unwrap(), "D case {i}");
        t::ftKb_SpecialNCapture0_Anim(gp);
        assert_eq!(flush("E", i, &mut f), lines.next().unwrap(), "E case {i}");
        t::ftKb_SpecialNCapture_Anim(gp);
        assert_eq!(flush("F", i, &mut f), lines.next().unwrap(), "F case {i}");
    }
    assert!(lines.next().is_none());

    // smoke: the Wait IASA eating the held item, the Loop IASA timer, the C-signature accessors.
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, _, _, _, _| unsafe { LOG.push(format!("ms {ms} {fl:x}")) });
        st::FTANIM_8006EBA4 = Some(|_| unsafe { LOG.push("anim".into()) });
        st::FTANIM_ISFRAMESREMAINING = Some(|_| 0);
        st::FT_8008A2BC = Some(|_| unsafe { LOG.push("wait".into()) });
        st::FTCO_FALL_ENTER = Some(|_| unsafe { LOG.push("fall".into()) });
    }
    // EatWait, holding an item, A pressed: ItemEat -> state 0x170 with flags 2.
    unsafe { f.u.kb.set_xF4_b0(1) };
    f.input.pressed_buttons = 0x200;
    t::ftKb_EatWait_IASA(gp);
    // Loop IASA: counts xE4 down, then B released -> End.
    unsafe { f.u.kb.xE4 = 1 };
    t::ftKb_SpecialNLoop_IASA(gp);
    assert_eq!(unsafe { f.u.kb.xE4 }, 0);
    f.input.held_buttons[0] = 0;
    t::ftKb_SpecialNLoop_IASA(gp);
    t::ftKb_SpecialAirNLoop_IASA(gp);
    t::ftKb_SpecialNEnd_Anim(gp);
    t::ftKb_SpecialAirNEnd_Anim(gp);
    t::ftKb_EatWait_Anim(gp);
    let got = unsafe { LOG.join(",") };
    assert_eq!(
        got,
        format!(
            "ms 368 2,anim,ms {} 0,ms {} 0,wait,fall",
            ftKb_MS_SpecialNEnd, ftKb_MS_SpecialAirNEnd
        )
    );
    assert_eq!(f.x2222_b2(), 1);
    assert_eq!(t::ftKb_SpecialN_800F597C(gp), 0);
    f.facing_dir = 1.0;
    assert_eq!(t::ftKb_SpecialN_800F5A60(gp), -1.0);
}
