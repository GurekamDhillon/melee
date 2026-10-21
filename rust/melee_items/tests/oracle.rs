//! melee_items against values computed by the C (tests/c_oracle/oracle.c, compiled natively).
#[path = "c_oracle/expected.rs"]
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
    assert_eq!(it.xDC8_word.flags.bf_0, 1);
    it_802762bc(&mut it);
    assert_eq!(it.xDC8_word.flags.x1F(), 0);
    check_hit_lag(3.0, &mut it);
    assert_eq!(it.xCBC_hitlagFrames, 3.0);
    assert_eq!(it.xDC8_word.flags.xA(), 1);
    assert_eq!(it.xDC8_word.flags.bf_0, 1 << (31 - 10));
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
