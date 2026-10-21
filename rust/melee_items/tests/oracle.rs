//! melee_items against values computed by the C (tests/c_oracle/oracle.c, compiled natively).
#[path = "c_oracle/expected.rs"]
#[allow(dead_code)]
mod expected;

use melee_core::gc::GcLayout;
use melee_items::hash::{item_word, ItemSum};
use melee_items::item::*;
use melee_items::layout::{itSamusMissileAttributes, Item};

fn f(u: u32) -> f32 {
    f32::from_bits(u)
}

#[test]
fn item_word_matches_c() {
    let mut items = Vec::new();
    for &(kind, msid, p, want) in expected::ITEM_WORD {
        let mut it = Item::zeroed();
        it.kind = kind;
        it.msid = msid;
        (it.pos.x, it.pos.y, it.pos.z) = (f(p[0]), f(p[1]), f(p[2]));
        (it.x40_vel.x, it.x40_vel.y, it.x40_vel.z) = (f(p[3]), f(p[4]), f(p[5]));
        assert_eq!(item_word(&it), want, "kind {kind} msid {msid}");
        items.push(it);
    }
    // the sum is order independent
    let a = ItemSum::of(items.iter());
    let b = ItemSum::of(items.iter().rev());
    assert_eq!(a, b);
    assert_eq!(a.count, expected::ITEM_WORD.len() as u32);
    let s = expected::ITEM_WORD.iter().fold(0u32, |s, e| s.wrapping_add(e.3));
    assert_eq!(a.sum, s);
}

#[test]
fn damage_clamp_matches_c() {
    for &(base, add, want) in expected::DAMAGE {
        let mut it = Item::zeroed();
        it.xC9C = base;
        item_80269ca0(&mut it, add);
        assert_eq!(it.xC9C, want);
    }
}

#[test]
fn decelerate_matches_c() {
    for &(v, d, want) in expected::DECEL_X {
        let mut it = Item::zeroed();
        it.x40_vel.x = f(v);
        decelerate_item_x(&mut it, f(d));
        assert_eq!(it.x40_vel.x.to_bits(), want, "v {v:08X} d {d:08X}");
    }
}

#[test]
fn samus_missile_clamp_matches_c() {
    for &(t, m, want) in expected::CLAMP_TURN {
        let mut sa = itSamusMissileAttributes::zeroed();
        sa.x1C = f(m);
        let mut it = Item::zeroed();
        it.xDD4_itemVar.samusmissile.x8 = f(t);
        its_samus_missile_clamp_turn(&mut it, &sa);
        assert_eq!(unsafe { it.xDD4_itemVar.samusmissile.x8 }.to_bits(), want);
    }
}

#[test]
fn norm_xf74_matches_c() {
    for &(fd, a, want) in expected::NORM_XF74 {
        let mut it = Item::zeroed();
        it.facing_dir = f(fd);
        it.xDD4_itemVar.linkboomerang.xF74 = f(a);
        norm_xf74(&mut it);
        assert_eq!(unsafe { it.xDD4_itemVar.linkboomerang.xF78 }.to_bits(), want);
    }
}

#[test]
fn neutral_hash_fields_at_spec_gc_offsets() {
    // docs/rust-port/neutral-hash.md, "Item sum"
    for (name, off) in [("kind", 0x10), ("msid", 0x24), ("pos", 0x4C), ("x40_vel", 0x40)] {
        assert_eq!(Item::gc_field(name).unwrap().offset, off, "{name}");
    }
    assert_eq!(Item::GC_SIZE, 0xFD0); // clang ppc32 sizeof(Item) = 4048
    assert_eq!(Item::gc_field_at(0x50).unwrap().name, "pos");
}

#[test]
fn bitfield_flags_use_gc_numbering() {
    let mut it = Item::zeroed();
    it_802762bc(&mut it);
    assert_eq!(it.ground_or_air, GA_AIR);
    it.xDC8_word.flags.set_x1F(1);
    // x1F is the last bit (MSB=0 numbering) of the u32 at GC +0xDC8: the word's LSB
    assert_eq!(u32::from_be_bytes(it.xDC8_word.flags.bf_0), 1);
    it_802762bc(&mut it);
    assert_eq!(it.xDC8_word.flags.x1F(), 0);
    check_hit_lag(3.0, &mut it);
    assert_eq!(it.xCBC_hitlagFrames, 3.0);
    assert_eq!(it.xDC8_word.flags.xA(), 1);
    assert_eq!(u32::from_be_bytes(it.xDC8_word.flags.bf_0), 1 << (31 - 10));
    it_80274f28(&mut it, -2, None, None);
    assert_eq!(unsafe { it.xDD0_flag.byte }, 0x04); // b5: bit 5 from the MSB
    assert_eq!(it.xD08, 0xFE);
}

#[test]
fn pkthunder_shift() {
    let mut it = Item::zeroed();
    unsafe {
        for i in 0..16 {
            it.xDD4_itemVar.pkthunder.angles[i] = i as f32;
        }
    }
    pkthunderball_shift_angles(&mut it);
    let a = unsafe { it.xDD4_itemVar.pkthunder.angles };
    assert_eq!(a[0], 0.0);
    for i in 1..16 {
        assert_eq!(a[i], (i - 1) as f32);
    }
}

// ---- round 1 (night 2): helpers.rs against tests/c_oracle/round1.c (contraction on) ----
mod round1 {
    use super::{expected, f};
    use melee_items::helpers::*;
    use melee_items::layout::{
        ftCommonData, itECB, itSeakChain_Attrs, Item, ItemCommonData, ItemLink, Vec as GcVec,
    };

    fn v3(x: u32, y: u32, z: u32) -> GcVec {
        let mut v = GcVec::zeroed();
        (v.x, v.y, v.z) = (f(x), f(y), f(z));
        v
    }
    /// Bit-exact, except for NaN results: x86 SSE gives the sign/payload of whichever operand
    /// the compiler put first (or the default 0xFFC00000), which differs between clang's and
    /// rustc's instruction choice and is not game state (Gekko differs again). A NaN must still
    /// come out where the C gives one.
    fn eq(got: f32, want: u32, what: &str, row: &[u32]) {
        if got.is_nan() && f32::from_bits(want).is_nan() {
            return;
        }
        assert_eq!(got.to_bits(), want, "{what}: {row:08X?}");
    }

    #[test]
    fn products() {
        for r in expected::PRODUCT_XYZ {
            let (a, b) = (v3(r[0], r[1], r[2]), v3(r[3], r[4], r[5]));
            eq(product_xy(&a, &b), r[6], "product_xy", r);
            eq(product_xyz(&a, &b), r[7], "product_xyz", r);
        }
        for r in expected::NEG_DOT {
            let (a, b) = (v3(r[0], r[1], r[2]), v3(r[3], r[4], r[5]));
            eq(neg_dot(&a, &b), r[6], "neg_dot", r);
        }
    }

    #[test]
    fn calc_angle() {
        for r in expected::CALC_ANGLE {
            let mut cd = ftCommonData::zeroed();
            (cd.x144_radians, cd.x148, cd.x14C, cd.x150) = (f(r[2]), f(r[3]), f(r[4]), f(r[5]));
            eq(it_8027b798_calc_angle(r[0] as _, r[1] as i32, &cd, f(r[6])), r[7], "calc_angle", r);
        }
    }

    #[test]
    fn angles() {
        for r in expected::ANGLES {
            let x = f(r[0]);
            let mut t = x;
            clamp_tau(&mut t);
            eq(t, r[1], "clamp_tau", r);
            t = x;
            clamp_pi_tau(&mut t);
            eq(t, r[2], "clamp_pi_tau", r);
            t = x;
            clamp_angle_pi(&mut t);
            eq(t, r[3], "clamp_angle_pi", r);
            let mut it = Item::zeroed();
            it.facing_dir = 1.0;
            norm_xf74_from_angle(&mut it, x);
            eq(unsafe { it.xDD4_itemVar.linkboomerang.xF78 }, r[4], "norm fd=1", r);
            it.facing_dir = -1.0;
            norm_xf74_from_angle(&mut it, x);
            eq(unsafe { it.xDD4_itemVar.linkboomerang.xF78 }, r[5], "norm fd=-1", r);
        }
        for r in expected::DOSEI {
            let mut it = Item::zeroed();
            it.facing_dir = f(r[0]);
            eq(it_dosei_facing_angle(&it, f(r[1])), r[2], "dosei", r);
        }
    }

    #[test]
    fn scalars() {
        for (i, r) in expected::SCALAR3.iter().enumerate() {
            let (x, y, z) = (f(r[0]), f(r[1]), f(r[2]));
            eq(link_boomerang_add_clamp(x, y, z), r[3], "add_clamp", r);
            eq(it_link_lerp(x, y, z), r[4], "lerp", r);
            eq(ty_display_calc_dist_sq(x, y), r[5], "dist_sq", r);
            eq(pika_scale(x, y), r[6], "pika_scale", r);
            eq(octarock_stone_get_x(x, y), r[7], "getX", r);
            eq(fsign_inline(x), r[8], "fsign", r);
            let si = float_sign_int_inline(if i & 3 == 0 { -1.0 } else { x });
            assert_eq!(si as u32, r[9], "fsign_int {r:08X?}");
            eq(inline_fabsf(x), r[10], "fabsf", r);
            eq(lt_zero(x), r[11], "lt_zero", r);
        }
    }

    #[test]
    fn rshell() {
        for r in expected::RSHELL {
            let mut at = itRShell_Attrs::zeroed();
            (at.x8, at.xC) = (f(r[2]), f(r[3]));
            let mut it = Item::zeroed();
            it.x40_vel.x = f(r[0]);
            it_rshell_clamp_vel(&mut it, &at);
            eq(it.x40_vel.x, r[6], "clamp_vel", r);
            it.pos.x = f(r[4]);
            it_rshell_um5_accel(&mut it, &at, &v3(r[5], 0, 0));
            eq(unsafe { it.xDD4_itemVar.rshell.xDE0 }, r[7], "accel", r);
            it.x40_vel.x = f(r[0]);
            it.xDD4_itemVar.rshell.xDE0 = f(r[1]);
            let ret = it_rshell_um5_add_vel_and_check(&mut it);
            eq(it.x40_vel.x, r[8], "addvel", r);
            assert_eq!(ret as u32, r[9], "addvel ret {r:08X?}");
            it_rshell_um5c_reverse(&mut it);
            assert_eq!(it.x40_vel.x.to_bits() ^ unsafe { it.xDD4_itemVar.rshell.xDE0 }.to_bits(),
                r[10], "reverse {r:08X?}");
        }
    }

    #[test]
    fn seak_chain() {
        for r in expected::SEAK_CLAMP {
            let mut at = itSeakChain_Attrs::zeroed();
            (at.x10, at.x14) = (f(r[1]), f(r[2]));
            let mut l = ItemLink::zeroed();
            l.vel.x = f(r[0]);
            it_seak_chain_clamp_x10(&mut l, &at);
            eq(l.vel.x, r[3], "x10", r);
            l.vel.x = f(r[0]);
            it_seak_chain_clamp_x14(&mut l, &at);
            eq(l.vel.x, r[4], "x14", r);
        }
    }

    #[test]
    fn ecb_overlap() {
        for r in expected::ECB_OVERLAP {
            let mut a = itECB::zeroed();
            let mut b = itECB::zeroed();
            (a.top, a.bottom, a.right, a.left) = (f(r[2]), f(r[3]), f(r[4]), f(r[5]));
            (b.top, b.bottom, b.right, b.left) = (f(r[6]), f(r[7]), f(r[8]), f(r[9]));
            let got = it_coll_chk_ecb_overlap(f(r[0]), f(r[1]), &a, &b, &v3(r[10], r[11], 0));
            assert_eq!(got as u32, r[12], "{r:08X?}");
        }
    }

    #[test]
    fn lifetimer_and_ceiling() {
        for r in expected::LIFETIMER {
            let mut cd = ItemCommonData::zeroed();
            cd.x4C_float = f(r[1]);
            let mut it = Item::zeroed();
            it.xD44_lifeTimer = f(r[0]);
            let ret = decrease_lifetimer(&mut it);
            eq(it.xD44_lifeTimer, r[2], "dec", r);
            assert_eq!(ret as u32, r[3]);
            it_80275158(&mut it, f(r[0]), &cd);
            eq(it.xD44_lifeTimer, r[4], "life", r);
            eq(it.xD48_halfLifeTimer, r[5], "half", r);
        }
        for r in expected::CEILING {
            let mut it = Item::zeroed();
            it.xC30 = -7;
            it.x378_itemColl.env_flags = r[0] as i32;
            it.x378_itemColl.ceiling.index = r[1] as i32;
            assert_eq!(it_802763e0(&mut it) as u32, r[2]);
            assert_eq!(it.xC30 as u32, r[3]);
        }
    }

    #[test]
    fn store_digits() {
        for r in expected::DIGITS {
            let mut d = [-1i32; 10];
            ty_figupon_store_digits(&mut d, r[0] as i32);
            let want: Vec<i32> = r[1..].iter().map(|&u| u as i32).collect();
            assert_eq!(d.to_vec(), want, "value {}", r[0] as i32);
        }
    }

    #[test]
    fn rshell_attrs_gc_layout() {
        use melee_core::gc::GcLayout;
        assert_eq!(itRShell_Attrs::GC_SIZE, 0x58);
        assert_eq!(itRShell_Attrs::gc_field("x48").unwrap().offset, 0x48);
        assert_eq!(core::mem::size_of::<itRShell_Attrs>(), 0x58);
    }
}
