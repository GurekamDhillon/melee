//! tu::ftkirbyspecialness vs the C (gen/turef/tu_ftkirbyspecialness_ref.c, shipping contraction; data in
//! tests/data/tu_ftkirbyspecialness.txt; regenerate with
//! `python rust/melee_chara/gen/turef/tu_shard321.py tu_ftkirbyspecialness`): the PK Flash spawn height (one
//! fused site), plus a smoke test of the Hold timers and IASA. One #[test]: the hooks are process globals.
#![allow(static_mut_refs)]

use melee_chara::stubs as st;
use melee_chara::tu::ftkirbyspecialness as t;
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

static mut LB: [f32; 3] = [0.0; 3];
static mut FP: [u32; 4] = [0; 4];
static mut SPAWN_OK: bool = false;
static mut SPAWNED: *mut HSD_GObj = core::ptr::null_mut();
static mut LOG: std::vec::Vec<String> = std::vec::Vec::new();

#[test]
fn matches_c_and_smoke() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_ftkirbyspecialness.txt")).unwrap();
    let mut f: Box<Fighter> = Box::new(Fighter::zeroed());
    let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut spawned: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut dummy: Box<[u8; 64]> = Box::new([0; 64]);
    let mut parts: std::vec::Vec<FighterBone> = (0..100).map(|_| unsafe { core::mem::zeroed() }).collect();
    for p in parts.iter_mut() {
        p.joint = dummy.as_mut_ptr() as *mut _;
    }
    let da: &'static mut ftKb_DatAttrs = Box::leak(Box::new(ftKb_DatAttrs::zeroed()));
    f.dat_attrs = da as *mut _ as *mut _;
    f.parts = parts.as_mut_ptr();
    g.user_data = &mut *f as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut *g;
    unsafe {
        SPAWNED = &mut *spawned;
        t::ext::LB_8000B1CC = Some(|_, _, out| unsafe {
            (*out).x = LB[0];
            (*out).y = LB[1];
            (*out).z = LB[2];
        });
        t::ext::IT_802AA8C0 = Some(|_, p, _, dir| unsafe {
            FP = [(*p).x.to_bits(), (*p).y.to_bits(), (*p).z.to_bits(), dir.to_bits()];
            if SPAWN_OK {
                SPAWNED
            } else {
                core::ptr::null_mut()
            }
        });
    }
    let mut r = Rng(0x2E55);
    let mut lines = want.lines();
    for i in 0..3000 {
        unsafe {
            LB = [r.rf(), r.rf(), r.rf()];
        }
        f.x34_scale.y = r.rf();
        f.facing_dir = if r.rnd() & 1 != 0 { 1.0 } else { -1.0 };
        unsafe {
            SPAWN_OK = r.rnd() & 1 != 0;
            f.u.kb.ns_flash_gobj = if r.rnd() & 1 != 0 { SPAWNED } else { core::ptr::null_mut() };
            FP = [0; 4];
        }
        f.death2_cb = None;
        f.take_dmg_cb = None;
        let mut pos = Vec3 { x: 9.0, y: 9.0, z: 9.0 };
        t::ftKb_NsSpecialNStart_Anim_inline(gp, &mut pos);
        let p = unsafe { FP };
        let got = format!(
            "{} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {:08x} {} {}",
            i,
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
            p[0],
            p[1],
            p[2],
            p[3],
            (unsafe { f.u.kb.ns_flash_gobj } == unsafe { SPAWNED }) as i32,
            (f.death2_cb.is_some() && f.take_dmg_cb.is_some()) as i32
        );
        assert_eq!(got, lines.next().unwrap(), "case {i}");
    }
    assert!(lines.next().is_none());

    // smoke: Hold timers count down; with no flash left and both timers out, Hold -> End; the IASA
    // drops the flash after the release delay once B is let go.
    unsafe {
        st::FIGHTER_CHANGEMOTIONSTATE = Some(|_, ms, fl, a, _, _, _| unsafe { LOG.push(format!("ms {ms} {fl:x} {a}")) });
        f.u.kb.ns_flash_gobj = core::ptr::null_mut();
        f.mv.ns.specialn.frames_to_loop_charge_ground = 1;
        f.mv.ns.specialn.frames_to_loop_charge_air = 2;
    }
    f.motion_id = 0;
    f.cur_anim_frame = 7.0;
    t::ftKb_NsSpecialNHold_Anim(gp);
    unsafe {
        assert_eq!(f.mv.ns.specialn.frames_to_loop_charge_ground, 0);
        assert_eq!(f.mv.ns.specialn.frames_to_loop_charge_air, 1);
    }
    t::ftKb_NsSpecialNHold_Anim(gp);
    unsafe {
        f.u.kb.ns_flash_gobj = &mut *spawned;
        f.mv.ns.specialn.charge_release_delay = 1;
    }
    f.input.held_buttons[0] = 0;
    t::ftKb_NsSpecialNHold_IASA(gp);
    assert!(unsafe { f.u.kb.ns_flash_gobj }.is_null());
    assert_eq!(
        unsafe { LOG.join(",") },
        format!(
            "ms {} 0 7,ms {} 0 0",
            ftKb_MS_NsSpecialNHold1, ftKb_MS_NsSpecialNEnd
        )
    );
    drop(parts);
}
