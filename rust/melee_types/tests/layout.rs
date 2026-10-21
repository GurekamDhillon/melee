//! GameCube offsets from the generated tables against docs/rust-port/neutral-hash.md (offsets
//! written down independently, from the C headers' comments) and bitfield round-trips.
use melee_core::gc::GcLayout;
use melee_types::*;

fn off<T: GcLayout>(n: &str) -> u32 {
    T::gc_field(n).unwrap_or_else(|| panic!("{} has no {n}", T::GC_NAME)).offset
}

#[test]
fn fighter_offsets_match_spec() {
    assert_eq!(off::<Fighter>("motion_id"), 0x10);
    assert_eq!(off::<Fighter>("facing_dir"), 0x2C);
    assert_eq!(off::<Fighter>("self_vel"), 0x80);
    assert_eq!(off::<Fighter>("x8c_kb_vel"), 0x8C);
    assert_eq!(off::<Fighter>("cur_pos"), 0xB0);
    assert_eq!(off::<Fighter>("ground_or_air"), 0xE0);
    assert_eq!(off::<Fighter>("gr_vel"), 0xEC);
    assert_eq!(off::<Fighter>("x3E4_fighterCmdScript") + off::<CommandInfo>("frame_count"), 0x3E8);
    assert_eq!(off::<Fighter>("x67F"), 0x67F);
    assert_eq!(off::<Fighter>("x680"), 0x680);
    assert_eq!(off::<Fighter>("x1968_jumpsUsed"), 0x1968);
    assert_eq!(off::<Fighter>("shield_health"), 0x1998);
    assert_eq!(off::<Fighter>("cpu"), 0x1A88);
    assert_eq!(off::<Fighter>("dmg") + off::<dmg>("x1830_percent"), 0x1830);
    assert_eq!(off::<Fighter>("dmg") + off::<dmg>("x195c_hitlag_frames"), 0x195C);
    let b = Fighter::gc_field("x2219_b5").unwrap();
    assert_eq!(b.offset * 8 + b.bits.unwrap().0 as u32, 0x2219 * 8 + 5);
}

#[test]
fn cpu_and_item_offsets_match_spec() {
    assert_eq!(off::<CpuFighter>("lstick"), 0x4);
    assert_eq!(off::<CpuFighter>("cstick"), 0x6);
    assert_eq!(off::<CpuFighter>("kind"), 0xC);
    assert_eq!(off::<CpuFighter>("x7C"), 0x7C);
    assert_eq!(off::<CpuFighter>("xEC"), 0xEC);
    assert_eq!(off::<CpuFighter>("command_duration"), 0x44C);
    assert_eq!(off::<CpuFighter>("csP"), 0x450);
    assert_eq!(off::<CpuFighter>("buffer"), 0x454);
    let b = CpuFighter::gc_field("xF8_b0").unwrap();
    assert_eq!((b.offset, b.bits.unwrap()), (0xF8, (0, 1)));
    assert_eq!(off::<Item>("kind"), 0x10);
    assert_eq!(off::<Item>("msid"), 0x24);
    assert_eq!(off::<Item>("x40_vel"), 0x40);
    assert_eq!(off::<Item>("pos"), 0x4C);
    // TARGET_PC appends one word to Item (it/types.h ASSERT_SIZE 0xFD0).
    assert_eq!(Item::GC_SIZE, 0xFD0);
}

/// Bitfield bytes hold the GameCube byte image: xF8_b0 (MSB of +0xF8) is 0x80 there.
#[test]
fn bitfields_are_gc_msb_first() {
    let mut c: CpuFighter = unsafe { core::mem::zeroed() };
    c.set_xF8_b0(1);
    assert_eq!(c.bf_f8, [0x80, 0, 0, 0]);
    c.set_xF8_b12(3);
    assert_eq!(c.bf_f8, [0xE0, 0, 0, 0]);
    c.set_xFB_b7(1);
    assert_eq!(c.bf_f8, [0xE0, 0, 0, 1]);
    assert_eq!((c.xF8_b0(), c.xF8_b12(), c.xF8_b34(), c.xFB_b7()), (1, 3, 0, 1));
    c.set_xF8_b12(0);
    assert_eq!(c.bf_f8[0], 0x80);
}

#[test]
fn registry_is_complete_and_sane() {
    assert!(GC_RECORDS.len() > 1500);
    for (name, size, align, fields) in GC_RECORDS {
        assert!(align.is_power_of_two(), "{name}");
        for f in *fields {
            let end = f.offset + if f.bits.is_some() { 0 } else { f.size };
            assert!(end <= *size, "{name}.{} past end", f.name);
        }
    }
    assert_eq!(<HSD_JObj as GcLayout>::GC_SIZE, 0x88); // jobj.h ASSERT_SIZE(struct HSD_JObj, 0x88)
}
