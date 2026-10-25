//! tu/pobj.rs: the display-list scan that sizes the vertex arrays handed to the GX backend.
use melee_runtime::tu::pobj::{scan_display_list, VtxDesc};

fn desc(attr: u32, attr_type: u32, comp_cnt: u32, comp_type: u32) -> VtxDesc {
    VtxDesc { attr, attr_type, comp_cnt, comp_type, frac: 0, stride: 0, vertex: None }
}

#[test]
fn scan_finds_max_index_per_attribute() {
    // PNMTXIDX direct u8, POS index16, CLR0 direct RGBA8, TEX0 index8.
    let attrs = [desc(0, 1, 0, 0), desc(9, 3, 1, 4), desc(11, 1, 1, 5), desc(13, 2, 1, 3)];
    let mut dl = vec![0x98, 0x00, 0x02]; // GX_TRIANGLESTRIP, 2 vertices
    dl.extend([0x00, 0x01, 0x2C, 1, 2, 3, 4, 0x07]); // pn 0, pos 300, clr, tex 7
    dl.extend([0x03, 0x00, 0x05, 1, 2, 3, 4, 0x40]); // pn 3, pos 5, clr, tex 64
    dl.extend([0; 8]); // NOP padding to the 32-byte list size
    let mut m = scan_display_list(&dl, &attrs);
    m.sort();
    assert_eq!(m, vec![(9, 300), (13, 64)]);
}

#[test]
fn scan_stops_at_a_truncated_list() {
    let attrs = [desc(9, 3, 1, 4)];
    let dl = [0x90, 0x00, 0x03, 0x00, 0x02, 0x00]; // 3 vertices announced, 1.5 present
    assert_eq!(scan_display_list(&dl, &attrs), vec![(9, 2)]);
}
