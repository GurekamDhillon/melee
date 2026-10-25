//! Native-layout port of `src/sysdolphin/baselib/tev.c` (raw pointers over the generated `HSD_Chan` /
//! `HSD_TevDesc` records): the cached colour channel setup, TEV stage assignment and setup, the
//! TEV register flush, the id -> count tables, the invalidate entry points.
//! `tu::tev` is the descriptor-driven renderer of the platform draw path; this is the C-shaped
//! surface, checked against the C by `tests/tu_tev_native.rs` (gen/mtu).
//!
//! Console semantics of the colour word compares: `CompareRGB` ignores alpha, `CopyRGB` keeps the
//! destination alpha (the C reads the GXColor as a big-endian word; the oracle build patches the masks
//! for the native byte order).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc, dead_code)]
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GXSetChanAmbColor(a0: u32, a1: _GXColor);
    fn GXSetChanCtrl(a0: u32, a1: u8, a2: u32, a3: u32, a4: u32, a5: u32, a6: u32);
    fn GXSetChanMatColor(a0: u32, a1: _GXColor);
    fn GXSetNumChans(a0: u8);
    fn GXSetNumTevStages(a0: u8);
    fn GXSetNumTexGens(a0: u8);
    fn GXSetTevAlphaIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevAlphaOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevColorIn(a0: u32, a1: u32, a2: u32, a3: u32, a4: u32);
    fn GXSetTevColorOp(a0: u32, a1: u32, a2: u32, a3: u32, a4: u8, a5: u32);
    fn GXSetTevColorS10(a0: u32, a1: _GXColorS10);
    fn GXSetTevKAlphaSel(a0: u32, a1: u32);
    fn GXSetTevKColorSel(a0: u32, a1: u32);
    fn GXSetTevOp(a0: u32, a1: u32);
    fn GXSetTevOrder(a0: u32, a1: u32, a2: u32, a3: u32);
    fn GXSetTevSwapMode(a0: u32, a1: u32, a2: u32);
    fn HSD_ObjAllocInit(a0: *mut _HSD_ObjAllocData, a1: u32, a2: u32) = crate::tu::objalloc::HSD_ObjAllocInit(a0, a1 as usize, a2);
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

/// The TU-local `TevReg[i]`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TevRegEntry {
    pub a: GXColorS10,
    pub c: i32,
}
pub static mut TevReg: [TevRegEntry; 4] = [TevRegEntry { a: GXColorS10 { r: 0, g: 0, b: 0, a: 0 }, c: 0 }; 4];

const fn pc(chan: u32, attn_fn: u32) -> HSD_Chan {
    HSD_Chan {
        next: null_mut(),
        chan,
        flags: 0,
        amb_color: GXColor { r: 0, g: 0, b: 0, a: 0 },
        mat_color: GXColor { r: 0, g: 0, b: 0, a: 0 },
        enable: 0,
        amb_src: 0,
        mat_src: 0,
        light_mask: 0,
        diff_fn: 2,
        attn_fn,
        aobj: null_mut(),
    }
}
pub static mut prev_ch: [HSD_Chan; 4] = [pc(0, 0), pc(1, 2), pc(2, 0), pc(3, 2)];
static mut invalid_prev_ch: [HSD_Chan; 4] = [pc(0, 0), pc(1, 2), pc(2, 0), pc(3, 2)];

pub static mut render_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut tevreg_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };
pub static mut chan_alloc_data: HSD_ObjAllocData = unsafe { core::mem::zeroed() };

pub static mut current_tev: i32 = 0;
pub static mut num_tex_gens: i32 = 0;
pub static mut prev_num_chans: i32 = 0;
pub static mut prev_mat_invalid: [i32; 2] = [0; 2];
pub static mut prev_amb_invalid: [i32; 2] = [0; 2];
pub static mut committed_tev_stages: i32 = 0;

const GX_COLOR0: u32 = 0;
const GX_COLOR1: u32 = 1;
const GX_COLOR0A0: u32 = 4;
const GX_COLOR1A1: u32 = 5;
const GX_COLOR_NULL: u32 = 0xFF;

/// `HSD_RenderInitAllocData`.
pub unsafe fn HSD_RenderInitAllocData() {
    ext::HSD_ObjAllocInit(addr_of_mut!(render_alloc_data), 28, 4);
    ext::HSD_ObjAllocInit(addr_of_mut!(tevreg_alloc_data), 20, 4);
    ext::HSD_ObjAllocInit(addr_of_mut!(chan_alloc_data), 48, 4);
}

/// `HSD_RenderGetAllocData`.
pub unsafe fn HSD_RenderGetAllocData() -> *mut HSD_ObjAllocData {
    addr_of_mut!(render_alloc_data)
}

/// `HSD_TevRegGetAllocData`.
pub unsafe fn HSD_TevRegGetAllocData() -> *mut HSD_ObjAllocData {
    addr_of_mut!(tevreg_alloc_data)
}

/// `HSD_ChanGetAllocData`.
pub unsafe fn HSD_ChanGetAllocData() -> *mut HSD_ObjAllocData {
    addr_of_mut!(chan_alloc_data)
}

fn CompareRGB(c0: &GXColor, c1: &GXColor) -> bool {
    c0.r != c1.r || c0.g != c1.g || c0.b != c1.b
}
fn CompareRGBA(c0: &GXColor, c1: &GXColor) -> bool {
    c0.r != c1.r || c0.g != c1.g || c0.b != c1.b || c0.a != c1.a
}
fn CopyRGB(dst: &mut GXColor, src: &GXColor) {
    dst.r = src.r;
    dst.g = src.g;
    dst.b = src.b;
}

/// `HSD_SetupChannel`.
pub unsafe fn HSD_SetupChannel(ch: *mut HSD_Chan) {
    if ch.is_null() || (*ch).chan == GX_COLOR_NULL {
        return;
    }
    let chan = (*ch).chan;
    let idx = (chan & 3) as usize;
    let no = (chan & 1) as usize;
    if (*ch).enable != 0 && (*ch).amb_src == 0 {
        let mut set_amb = false;
        if prev_amb_invalid[no] != 0 {
            prev_amb_invalid[no] = 0;
            ext::GXSetChanAmbColor(no as u32 + 4, (*ch).amb_color);
            prev_ch[no].amb_color = (*ch).amb_color;
        } else if chan == GX_COLOR0A0 || chan == GX_COLOR1A1 {
            if CompareRGBA(&(*ch).amb_color, &prev_ch[no].amb_color) {
                prev_ch[no].amb_color = (*ch).amb_color;
                set_amb = true;
            }
        } else if chan == GX_COLOR0 || chan == GX_COLOR1 {
            if CompareRGB(&(*ch).amb_color, &prev_ch[no].amb_color) {
                CopyRGB(&mut prev_ch[no].amb_color, &(*ch).amb_color);
                set_amb = true;
            }
        } else if (*ch).amb_color.a != prev_ch[no].amb_color.a {
            prev_ch[no].amb_color.a = (*ch).amb_color.a;
            set_amb = true;
        }
        if set_amb {
            ext::GXSetChanAmbColor(chan, (*ch).amb_color);
        }
    }

    if (*ch).mat_src == 0 {
        let mut set_mat = false;
        if prev_mat_invalid[no] != 0 {
            prev_mat_invalid[no] = 0;
            ext::GXSetChanMatColor(no as u32 + 4, (*ch).mat_color);
            prev_ch[no].mat_color = (*ch).mat_color;
        } else if chan == GX_COLOR0A0 || chan == GX_COLOR1A1 {
            if CompareRGBA(&(*ch).mat_color, &prev_ch[no].mat_color) {
                prev_ch[no].mat_color = (*ch).mat_color;
                set_mat = true;
            }
        } else if chan == GX_COLOR0 || chan == GX_COLOR1 {
            if CompareRGB(&(*ch).mat_color, &prev_ch[no].mat_color) {
                CopyRGB(&mut prev_ch[no].mat_color, &(*ch).mat_color);
                set_mat = true;
            }
        } else if (*ch).mat_color.a != prev_ch[no].mat_color.a {
            prev_ch[no].mat_color.a = (*ch).mat_color.a;
            set_mat = true;
        }
        if set_mat {
            ext::GXSetChanMatColor(chan, (*ch).mat_color);
        }
    }

    if (*ch).enable != prev_ch[idx].enable
        || (*ch).amb_src != prev_ch[idx].amb_src
        || (*ch).mat_src != prev_ch[idx].mat_src
        || (*ch).light_mask != prev_ch[idx].light_mask
        || (*ch).diff_fn != prev_ch[idx].diff_fn
        || (*ch).attn_fn != prev_ch[idx].attn_fn
    {
        ext::GXSetChanCtrl(chan, (*ch).enable, (*ch).amb_src, (*ch).mat_src, (*ch).light_mask, (*ch).diff_fn, (*ch).attn_fn);
        prev_ch[idx].enable = (*ch).enable;
        prev_ch[idx].amb_src = (*ch).amb_src;
        prev_ch[idx].mat_src = (*ch).mat_src;
        prev_ch[idx].light_mask = (*ch).light_mask;
        prev_ch[idx].diff_fn = (*ch).diff_fn;
        prev_ch[idx].attn_fn = (*ch).attn_fn;
        if chan == GX_COLOR0A0 || chan == GX_COLOR1A1 {
            prev_ch[idx + 2].enable = (*ch).enable;
            prev_ch[idx + 2].amb_src = (*ch).amb_src;
            prev_ch[idx + 2].mat_src = (*ch).mat_src;
            prev_ch[idx + 2].light_mask = (*ch).light_mask;
            prev_ch[idx + 2].diff_fn = (*ch).diff_fn;
            prev_ch[idx + 2].attn_fn = (*ch).attn_fn;
        }
    }
}

/// `HSD_StateSetNumChans`.
pub unsafe fn HSD_StateSetNumChans(num: i32) {
    if prev_num_chans != num {
        ext::GXSetNumChans(num as u8);
    }
}

/// `HSD_SetupChannelAll`.
pub unsafe fn HSD_SetupChannelAll(mut channel: *mut HSD_Chan) {
    let mut num = 0;
    while !channel.is_null() {
        let temp = HSD_Channel2Num((*channel).chan as i32);
        if temp > num {
            num = temp;
        }
        HSD_SetupChannel(channel);
        channel = (*channel).next;
    }
    HSD_StateSetNumChans((num as u8) as i32);
}

/// `HSD_StateRegisterTexGen`.
pub unsafe fn HSD_StateRegisterTexGen(coord: i32) {
    let temp_r3 = HSD_TexCoordID2Num(coord);
    if temp_r3 > num_tex_gens {
        num_tex_gens = temp_r3;
    }
}

/// `HSD_StateSetNumTexGens`.
pub unsafe fn HSD_StateSetNumTexGens() {
    ext::GXSetNumTexGens(num_tex_gens as u8);
    num_tex_gens = 0;
}

/// `HSD_StateInitTev`.
pub unsafe fn HSD_StateInitTev() {
    current_tev = 0;
}

/// `HSD_StateGetCommittedTevStages`.
pub unsafe fn HSD_StateGetCommittedTevStages() -> i32 {
    committed_tev_stages
}

/// `HSD_StateGetNumTevStages`.
pub unsafe fn HSD_StateGetNumTevStages() -> i32 {
    current_tev
}

/// `HSD_StateAssignTev`.
pub unsafe fn HSD_StateAssignTev() -> i32 {
    let i = current_tev;
    current_tev += 1;
    HSD_Index2TevStage(i)
}

/// `HSD_StateSetNumTevStages`.
pub unsafe fn HSD_StateSetNumTevStages() {
    committed_tev_stages = current_tev;
    ext::GXSetNumTevStages(current_tev as u8);
    current_tev = 0;
}

/// `HSD_SetupTevStage`.
pub unsafe fn HSD_SetupTevStage(desc: *mut HSD_TevDesc) {
    ext::GXSetTevOrder((*desc).stage, (*desc).coord, (*desc).map, (*desc).color);
    if (*desc).flags == 0 {
        ext::GXSetTevOp((*desc).stage, (*desc).u.tevconf.clr_op);
        ext::GXSetTevSwapMode((*desc).stage, 0, 0);
        return;
    }
    let c = &(*desc).u.tevconf;
    ext::GXSetTevColorOp((*desc).stage, c.clr_op, c.clr_bias, c.clr_scale, c.clr_clamp, c.clr_out_reg);
    ext::GXSetTevColorIn((*desc).stage, c.clr_a, c.clr_b, c.clr_c, c.clr_d);
    ext::GXSetTevAlphaOp((*desc).stage, c.alpha_op, c.alpha_bias, c.alpha_scale, c.alpha_clamp, c.alpha_out_reg);
    ext::GXSetTevAlphaIn((*desc).stage, c.alpha_a, c.alpha_b, c.alpha_c, c.alpha_d);
    ext::GXSetTevSwapMode((*desc).stage, c.ras_swap, c.tex_swap);
    ext::GXSetTevKColorSel((*desc).stage, c.kcsel);
    ext::GXSetTevKAlphaSel((*desc).stage, c.kasel);
}

/// `HSD_SetupTevStageAll`.
pub unsafe fn HSD_SetupTevStageAll(desc: *mut HSD_TevDesc) {
    let mut num = 0;
    let mut td = desc;
    while !td.is_null() {
        let tmp = HSD_TevStage2Num((*td).stage as i32);
        if tmp > num {
            num = tmp;
        }
        HSD_SetupTevStage(td);
        td = (*td).next;
    }
    current_tev = num;
    ext::GXSetNumTevStages(current_tev as u8);
    current_tev = 0;
}

/// `HSD_Channel2Num`.
pub unsafe fn HSD_Channel2Num(chan: i32) -> i32 {
    match chan {
        0 => 1,
        1 => 2,
        2 => 1,
        3 => 2,
        4 => 1,
        5 => 2,
        0xFF => 0,
        _ => panic!("tev.c:656 0"),
    }
}

/// `HSD_Index2TevStage`.
pub unsafe fn HSD_Index2TevStage(idx: i32) -> i32 {
    match idx {
        0..=15 => idx,
        _ => panic!("tev.c:709 0"),
    }
}

/// `HSD_TevStage2Index`.
pub unsafe fn HSD_TevStage2Index(stage: i32) -> i32 {
    match stage {
        0..=15 => stage,
        _ => panic!("tev.c:762 0"),
    }
}

/// `HSD_TevStage2Num`.
pub unsafe fn HSD_TevStage2Num(stage: i32) -> i32 {
    match stage {
        0..=15 => stage + 1,
        _ => panic!("tev.c:793 0"),
    }
}

/// `HSD_SetTevRegAll`.
pub unsafe fn HSD_SetTevRegAll() {
    let mut i = 0usize;
    while i < 4 {
        if TevReg[i].c != 0 {
            let reg = match i {
                0 => 1,
                1 => 2,
                2 => 3,
                3 => 0,
                _ => 1,
            };
            ext::GXSetTevColorS10(reg, TevReg[i].a);
            TevReg[i].c = 0;
        }
        i += 1;
    }
}

/// `HSD_TexCoordID2Num`.
pub unsafe fn HSD_TexCoordID2Num(id: i32) -> i32 {
    match id {
        0..=7 => id + 1,
        0xFF => 0,
        _ => panic!("tev.c:1010 0"),
    }
}

/// `ChanUpdateFunc`.
pub unsafe fn ChanUpdateFunc(chan: *mut HSD_Chan, arg1: i32, arg2: *const f32) {
    if !chan.is_null() {
        let v = *arg2 as f64;
        let conv = |x: f64| (crate::cabi::f2u32(x) as u8);
        match arg1 {
            5 => (*chan).mat_color.r = conv(255.0 * v),
            6 => (*chan).mat_color.g = conv(255.0 * v),
            7 => (*chan).mat_color.b = conv(255.0 * v),
            8 => (*chan).mat_color.a = conv(255.0 * v),
            9 => (*chan).amb_color.r = conv(255.0 * v),
            10 => (*chan).amb_color.g = conv(255.0 * v),
            11 => (*chan).amb_color.b = conv(255.0 * v),
            12 => (*chan).amb_color.a = conv(255.0 * v),
            _ => {}
        }
    }
}

/// `_HSD_StateInvalidateColorChannel`.
pub unsafe extern "C" fn _HSD_StateInvalidateColorChannel() {
    core::ptr::copy_nonoverlapping(addr_of_mut!(invalid_prev_ch).cast::<HSD_Chan>(), addr_of_mut!(prev_ch).cast::<HSD_Chan>(), 4);
    prev_amb_invalid[0] = 1;
    prev_amb_invalid[1] = 1;
    prev_mat_invalid[0] = 1;
    prev_mat_invalid[1] = 1;
    prev_num_chans = -1;
}

/// `_HSD_StateInvalidateTevStage`.
pub unsafe extern "C" fn _HSD_StateInvalidateTevStage() {
    current_tev = 0;
}

/// `_HSD_StateInvalidateTevRegister`.
pub unsafe extern "C" fn _HSD_StateInvalidateTevRegister() {
    let mut i = 0;
    while i < 4 {
        TevReg[i].c = 0;
        i += 1;
    }
}

/// `_HSD_StateInvalidateTexCoordGen`.
pub unsafe extern "C" fn _HSD_StateInvalidateTexCoordGen() {
    num_tex_gens = 0;
}
