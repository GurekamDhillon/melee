//! The save files' GameCube image (lbcardgame GC_IMAGE): GmSaveData and NameTagDataBank decode as a byte
//! permutation, and encode is its exact inverse, so card bytes round-trip unchanged.
use melee_runtime::tu::lbcardgame::{gc_decode_into, gc_encode, gc_permutation};
use melee_types::{GmSaveData, NameTagDataBank};

fn roundtrip<T: melee_core::gcdecode::GcDecode>() {
    let perm = gc_permutation::<T>();
    let g = T::GC_BYTES as usize;
    let mapped: usize = perm.iter().filter(|&&k| k != u32::MAX).count();
    assert_eq!(core::mem::size_of::<T>(), g, "native size equals the GameCube size");
    let mut seen = vec![false; g];
    for &k in &perm { if k != u32::MAX { assert!(!seen[k as usize], "image byte {k:#x} decoded twice"); seen[k as usize] = true; } }
    let holes: Vec<usize> = (0..g).filter(|&i| !seen[i]).collect();
    eprintln!("{}: {} of {} image bytes are fields; uncovered (implicit padding, kept from the card): {:x?}", core::any::type_name::<T>(), mapped, g, holes);
    let img: Vec<u8> = (0..g).map(|i| (i as u32).wrapping_mul(2246822519).rotate_left(13) as u8).collect();
    let mut native = vec![0u8; g];
    unsafe { gc_decode_into(&perm, &img, native.as_mut_ptr()) };
    let mut back = img.clone();
    for i in 0..g { if seen[i] { back[i] = 0; } }
    unsafe { gc_encode(&perm, native.as_ptr(), &mut back) };
    for i in 0..g { if seen[i] { assert_eq!(img[i], back[i], "byte {i:#x}"); } }
}

#[test]
fn save_data_image() {
    roundtrip::<GmSaveData>();
}
#[test]
fn name_tag_image() {
    roundtrip::<NameTagDataBank>();
}
