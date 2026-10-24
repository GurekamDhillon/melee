use melee_core::gc::GcLayout;
use melee_types::objheap;

#[test]
fn object_heap_addresses_keep_native_pointer_bits_and_gc_offsets() {
    let mut heap = objheap::zeroed();
    let allocation = Box::new([0u8; 64]);
    let address = allocation.as_ptr() as usize;
    heap.top = address as _;
    heap.curr = (address + 16) as _;
    assert_eq!(heap.top as usize, address);
    assert_eq!(heap.curr as usize, address + 16);
    assert_eq!(objheap::GC_SIZE, 16);
    assert_eq!(objheap::GC_FIELDS[0].offset, 0);
    assert_eq!(objheap::GC_FIELDS[1].offset, 4);
}
