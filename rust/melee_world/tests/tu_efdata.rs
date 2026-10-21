use melee_world::tu::efdata::EfData;

#[test]
fn efdata_starts_zeroed() {
    let d = EfData::default();
    assert_eq!((d.anim_count, d.effect_count, d.load_kind), (0, 0, 0));
    let bytes: [u8; core::mem::size_of::<melee_types::HSD_ObjAllocData>()] = unsafe { core::mem::transmute_copy(&d.alloc_data) };
    assert!(bytes.iter().all(|&b| b == 0));
}
