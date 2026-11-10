//! GXSetChanAmbColor / GXSetChanMatColor through `GxShim`: the SDK's per-field merge of a channel pair's
//! colour register (GX_COLORn sets RGB, GX_ALPHAn sets A, GX_COLORnAn all four).
use melee_gx::api::GxColor;
use melee_gx::consts::*;
use melee_gx::shim::GxShim;

fn c(r: u8, g: u8, b: u8, a: u8) -> GxColor {
    GxColor { r, g, b, a }
}

#[test]
fn single_channel_writes_keep_the_other_half() {
    let mut s = GxShim::new();
    for mat in [false, true] {
        let set = |s: &mut GxShim, id, col| if mat { s.set_chan_mat_color(id, col) } else { s.set_chan_amb_color(id, col) };
        set(&mut s, GX_COLOR0A0, c(1, 2, 3, 4));
        assert_eq!(s.gx.chan_color_reg(0, mat), 0x0102_0304);
        set(&mut s, GX_ALPHA0, c(9, 9, 9, 200));
        assert_eq!(s.gx.chan_color_reg(0, mat), 0x0102_03C8);
        set(&mut s, GX_COLOR0, c(10, 20, 30, 255));
        assert_eq!(s.gx.chan_color_reg(0, mat), 0x0A14_1EC8);
        set(&mut s, GX_COLOR1A1, c(5, 6, 7, 8));
        set(&mut s, GX_ALPHA1, c(0, 0, 0, 0));
        assert_eq!(s.gx.chan_color_reg(1, mat), 0x0506_0700);
        assert_eq!(s.gx.chan_color_reg(0, mat), 0x0A14_1EC8);
    }
}
