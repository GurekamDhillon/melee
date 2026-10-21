//! GameCube behaviours that a native layout does not reproduce by itself.
//!
//! Each item here is a place where the C code depends on the GameCube's *representation* (byte
//! order, 32-bit pointers, 0x80xxxxxx addresses), so a plain native translation would change
//! gameplay. The mechanical modules (`crate::c2r`) and hand ports both call these.

/// `HitCapsule::hit_grabbed_victim_only` (lb/types.h +0x134) is a `u8 :1` bitfield in a union
/// with `HSD_GObj* owner`. PPC allocates bitfields MSB first, so the flag *is* bit 31 of the
/// owner's GameCube address, and every valid heap pointer (0x80xxxxxx) reads as 1. Native
/// pointers have that bit clear, so this encodes the GameCube MSB in the native word:
///
/// * a native word `w` with bit 63 clear and `w != 0` is a pointer whose GC MSB is 1;
/// * `w == 0` is NULL (GC word 0, flag 0);
/// * bit 63 set marks "GC MSB cleared" (the pointer bits below it are kept, as the GC keeps the
///   low 31 bits); clearing it again restores the pointer exactly as on the GameCube.
///
/// Setting the flag on NULL gives the word 1 (the GC word would be 0x80000000: a garbage
/// non-null pointer either way).
#[inline]
pub fn gc_ptr_msb_flag(w: u64) -> u8 {
    (w != 0 && w >> 63 == 0) as u8
}

/// Store `v & 1` into the GC MSB of a pointer-or-flag union word (see [`gc_ptr_msb_flag`]).
#[inline]
pub fn gc_ptr_msb_flag_set(w: &mut u64, v: u8) {
    const TAG: u64 = 1 << 63;
    if v & 1 != 0 {
        if *w & TAG != 0 {
            *w &= !TAG;
        }
        if *w == 0 {
            *w = 1;
        }
    } else if *w != 0 && *w & TAG == 0 {
        *w = if *w == 1 { 0 } else { *w | TAG };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Model of the GameCube word: owner address (or 0) with the flag as its MSB.
    fn gc_flag(word: u32) -> u8 {
        (word >> 31) as u8
    }

    #[test]
    fn ptr_msb_flag_matches_gamecube_word() {
        // valid pointer: GC 0x80123450 -> flag 1
        let p = 0x0000_7ff6_1234_5670u64;
        let mut w = p;
        assert_eq!(gc_ptr_msb_flag(w), gc_flag(0x8012_3450));
        // clear: GC 0x00123450 -> 0; set again: pointer restored
        gc_ptr_msb_flag_set(&mut w, 0);
        assert_eq!(gc_ptr_msb_flag(w), gc_flag(0x0012_3450));
        gc_ptr_msb_flag_set(&mut w, 1);
        assert_eq!(w, p);
        // NULL: 0 -> flag 0; set -> nonzero, flag 1; clear -> NULL again
        let mut z = 0u64;
        assert_eq!(gc_ptr_msb_flag(z), 0);
        gc_ptr_msb_flag_set(&mut z, 1);
        assert_eq!(gc_ptr_msb_flag(z), gc_flag(0x8000_0000));
        gc_ptr_msb_flag_set(&mut z, 0);
        assert_eq!(z, 0);
        // only bit 0 of the stored value counts (C truncates to the 1-bit field)
        let mut q = p;
        gc_ptr_msb_flag_set(&mut q, 2);
        assert_eq!(gc_ptr_msb_flag(q), 0);
    }
}
