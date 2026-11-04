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

/// The word tables the C indexes into their padding (Target Test best times by selkind, the 51 event scores) are
/// big-endian words in the image, as the C build's GameCube memory holds them, not byte copies of native words.
#[test]
fn save_word_tables_big_endian() {
    use melee_runtime::tu::lbcardgame::{save_permutation, save_word_tables};
    let perm = save_permutation();
    let g = <GmSaveData as melee_core::gcdecode::GcDecode>::GC_BYTES as usize;
    let mut seen = vec![false; g];
    for &k in &perm { if k != u32::MAX { assert!(!seen[k as usize], "image byte {k:#x} decoded twice"); seen[k as usize] = true; } }
    let mut native = vec![0u8; core::mem::size_of::<GmSaveData>()];
    let sd = native.as_mut_ptr() as *mut GmSaveData;
    unsafe {
        // gmMainLib_8015D450(selkind 24) and gmMainLib_8015CF70(50, ..): past the declared s32[4]
        *(core::ptr::addr_of_mut!((*sd).unk_30.x4C) as *mut i32).add(24) = 0x0102_0304;
        *(core::ptr::addr_of_mut!((*sd).x1A70) as *mut i32).add(50) = 0x0A0B_0C0D;
        *(core::ptr::addr_of_mut!((*sd).unk_30.x4C) as *mut i32).add(1) = 0x1122_3344;
    }
    let mut img = vec![0u8; g];
    unsafe { gc_encode(&perm, native.as_ptr(), &mut img) };
    let [(_, tt, _), (_, ev, _)] = save_word_tables();
    assert_eq!(&img[tt + 4..tt + 8], &[0x11, 0x22, 0x33, 0x44]);
    assert_eq!(&img[tt + 96..tt + 100], &[1, 2, 3, 4]);
    assert_eq!(&img[ev + 200..ev + 204], &[0x0A, 0x0B, 0x0C, 0x0D]);
    let mut back = vec![0u8; native.len()];
    unsafe { gc_decode_into(&perm, &img, back.as_mut_ptr()) };
    assert_eq!(back, native);
}
