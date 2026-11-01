//! Native `tev.c`, including channel caches, TEV descriptors and allocation accessors.
//! Existing render methods remain the implementation; C-named entry points delegate to them.
#![allow(non_snake_case)]
use super::state::{gxcolor, rgba, GxBackend, HsdChan, HsdRender};
use melee_types::{GXColor, HSD_ObjAllocData, HSD_TevDesc};

pub(crate) fn initial_channels() -> [HsdChan; 4] {
    core::array::from_fn(|i| {
        let mut c = HsdChan::zeroed();
        c.chan = i as u32; c.diff_fn = 2; c.attn_fn = if i & 1 != 0 { 2 } else { 0 }; c
    })
}
/// C's GC word mask ignores alpha, not the first native byte.
pub fn CompareRGB(a: &GXColor, b: &GXColor) -> bool { melee_core::ct!("CompareRGB"); a.r != b.r || a.g != b.g || a.b != b.b }
pub fn CompareRGBA(a: &GXColor, b: &GXColor) -> bool { melee_core::ct!("CompareRGBA"); rgba(*a) != rgba(*b) }
pub fn CopyRGB(dst: &mut GXColor, src: &GXColor) { melee_core::ct!("CopyRGB"); dst.r = src.r; dst.g = src.g; dst.b = src.b; }

fn cached_color<G: GxBackend>(g: &mut G, cached: &mut GXColor, invalid: &mut bool,
                              chan: u32, color: GXColor, ambient: bool) {
    let mut id = chan;
    let changed = if *invalid {
        *invalid = false; *cached = color; id = (chan & 1) + 4; true
    } else if chan == 4 || chan == 5 {
        let changed = CompareRGBA(&color, cached);
        if changed { *cached = color; } changed
    } else if chan == 0 || chan == 1 {
        let changed = CompareRGB(&color, cached);
        if changed { CopyRGB(cached, &color); } changed
    } else {
        let changed = color.a != cached.a;
        if changed { cached.a = color.a; } changed
    };
    if changed {
        if ambient { g.set_chan_amb_color(id, rgba(color)); }
        else { g.set_chan_mat_color(id, rgba(color)); }
    }
}

impl HsdRender {
    pub fn setup_channel<G: GxBackend>(&mut self, g: &mut G, ch: &HsdChan) {
        if ch.chan == 255 { return; }
        let no = (ch.chan & 1) as usize;
        let idx = (ch.chan & 3) as usize;
        if ch.enable != 0 && ch.amb_src == 0 {
            cached_color(g, &mut self.prev_ch[no].amb_color, &mut self.prev_amb_invalid[no], ch.chan, ch.amb_color, true);
        }
        if ch.mat_src == 0 {
            cached_color(g, &mut self.prev_ch[no].mat_color, &mut self.prev_mat_invalid[no], ch.chan, ch.mat_color, false);
        }
        let prev = self.prev_ch[idx];
        if (ch.enable, ch.amb_src, ch.mat_src, ch.light_mask, ch.diff_fn, ch.attn_fn) !=
           (prev.enable, prev.amb_src, prev.mat_src, prev.light_mask, prev.diff_fn, prev.attn_fn) {
            g.set_chan_ctrl(ch.chan, ch.enable != 0, ch.amb_src, ch.mat_src, ch.light_mask, ch.diff_fn, ch.attn_fn);
            let copy = |p: &mut HsdChan| {
                p.enable = ch.enable; p.amb_src = ch.amb_src; p.mat_src = ch.mat_src;
                p.light_mask = ch.light_mask; p.diff_fn = ch.diff_fn; p.attn_fn = ch.attn_fn;
            };
            copy(&mut self.prev_ch[idx]);
            if ch.chan == 4 || ch.chan == 5 { copy(&mut self.prev_ch[idx + 2]); }
        }
    }
    pub fn state_set_num_chans<G: GxBackend>(&mut self, g: &mut G, num: i32) {
        // The C does not assign prev_num_chans here.
        if self.prev_num_chans != num { g.set_num_chans(num as u8); }
    }
    pub fn state_register_tex_gen(&mut self, coord: u32) {
        let n = HSD_TexCoordID2Num(coord as i32) as u32;
        if n > self.num_tex_gens { self.num_tex_gens = n; }
    }
    pub fn state_set_num_tex_gens<G: GxBackend>(&mut self, g: &mut G) {
        g.set_num_tex_gens(self.num_tex_gens as u8); self.num_tex_gens = 0;
    }
    pub fn state_init_tev(&mut self) { self.current_tev = 0; }
    pub fn state_assign_tev(&mut self) -> u32 {
        let s = self.current_tev; self.current_tev = self.current_tev.wrapping_add(1);
        HSD_Index2TevStage(s as i32) as u32
    }
    pub fn state_set_num_tev_stages<G: GxBackend>(&mut self, g: &mut G) {
        self.committed_tev_stages = self.current_tev;
        self.stats.tev_stages = self.stats.tev_stages.wrapping_add(self.current_tev);
        g.set_num_tev_stages(self.current_tev as u8); self.current_tev = 0;
    }
    pub fn setup_tev_stage<G: GxBackend>(&mut self, g: &mut G, desc: &HSD_TevDesc) {
        g.set_tev_order(desc.stage, desc.coord, desc.map, desc.color);
        let c = unsafe { desc.u.tevconf };
        if desc.flags == 0 {
            g.set_tev_op(desc.stage, c.clr_op); g.set_tev_swap_mode(desc.stage, 0, 0); return;
        }
        g.set_tev_color_op(desc.stage, c.clr_op, c.clr_bias, c.clr_scale, c.clr_clamp != 0, c.clr_out_reg);
        g.set_tev_color_in(desc.stage, c.clr_a, c.clr_b, c.clr_c, c.clr_d);
        g.set_tev_alpha_op(desc.stage, c.alpha_op, c.alpha_bias, c.alpha_scale, c.alpha_clamp != 0, c.alpha_out_reg);
        g.set_tev_alpha_in(desc.stage, c.alpha_a, c.alpha_b, c.alpha_c, c.alpha_d);
        g.set_tev_swap_mode(desc.stage, c.ras_swap, c.tex_swap);
        g.set_tev_kcolor_sel(desc.stage, c.kcsel); g.set_tev_kalpha_sel(desc.stage, c.kasel);
    }
    pub fn set_tev_reg_all<G: GxBackend>(&mut self, g: &mut G) {
        for (i, (color, dirty)) in self.tev_regs.iter_mut().enumerate() {
            if *dirty != 0 { g.set_tev_color_s10(((i + 1) & 3) as u32, *color); *dirty = 0; }
        }
    }
    pub fn invalidate_color_channel(&mut self) {
        self.prev_ch = initial_channels();
        self.prev_amb_invalid = [true; 2]; self.prev_mat_invalid = [true; 2]; self.prev_num_chans = -1;
    }
    pub fn invalidate_tev_stage(&mut self) { self.current_tev = 0; }
    pub fn invalidate_tev_register(&mut self) { for (_, dirty) in &mut self.tev_regs { *dirty = 0; } }
    pub fn invalidate_tex_coord_gen(&mut self) { self.num_tex_gens = 0; }
}

pub fn HSD_Channel2Num(chan: i32) -> i32 {
    melee_core::ct!("HSD_Channel2Num");
    match chan { 0..=5 => (chan & 1) + 1, 255 => 0, _ => panic!("HSD_ASSERT tev.c:656") }
}
pub fn HSD_Index2TevStage(idx: i32) -> i32 { melee_core::ct!("HSD_Index2TevStage"); assert!((0..16).contains(&idx), "HSD_ASSERT tev.c:709"); idx }
pub fn HSD_TevStage2Index(stage: i32) -> i32 { melee_core::ct!("HSD_TevStage2Index"); assert!((0..16).contains(&stage), "HSD_ASSERT tev.c:762"); stage }
pub fn HSD_TevStage2Num(stage: i32) -> i32 { melee_core::ct!("HSD_TevStage2Num"); assert!((0..16).contains(&stage), "HSD_ASSERT tev.c:793"); stage + 1 }
pub fn HSD_TexCoordID2Num(id: i32) -> i32 {
    melee_core::ct!("HSD_TexCoordID2Num");
    match id { 0..=7 => id + 1, 255 => 0, _ => panic!("HSD_ASSERT tev.c:1010") }
}
pub fn ChanUpdateFunc(chan: Option<&mut HsdChan>, track: i32, value: f32) {
    melee_core::ct!("ChanUpdateFunc");
    if let Some(ch) = chan {
        let color = match track { 5..=8 => &mut ch.mat_color, 9..=12 => &mut ch.amb_color, _ => return };
        let mut c = rgba(*color);
        c[((track - 5) & 3) as usize] = crate::cabi::f2u8(255.0f64 * value as f64);
        *color = gxcolor(c);
    }
}
pub fn HSD_RenderInitAllocData(r: &mut HsdRender) {
    melee_core::ct!("HSD_RenderInitAllocData");
    unsafe {
        ext::HSD_ObjAllocInit(&mut r.render_alloc_data, 28, 4);
        ext::HSD_ObjAllocInit(&mut r.tevreg_alloc_data, 20, 4);
        ext::HSD_ObjAllocInit(&mut r.chan_alloc_data, 48, 4);
    }
}
pub fn HSD_RenderGetAllocData(r: &mut HsdRender) -> *mut HSD_ObjAllocData { melee_core::ct!("HSD_RenderGetAllocData"); &mut r.render_alloc_data }
pub fn HSD_TevRegGetAllocData(r: &mut HsdRender) -> *mut HSD_ObjAllocData { melee_core::ct!("HSD_TevRegGetAllocData"); &mut r.tevreg_alloc_data }
pub fn HSD_ChanGetAllocData(r: &mut HsdRender) -> *mut HSD_ObjAllocData { melee_core::ct!("HSD_ChanGetAllocData"); &mut r.chan_alloc_data }
pub fn HSD_SetupChannel<G: GxBackend>(r: &mut HsdRender, g: &mut G, ch: Option<&HsdChan>) {
    melee_core::ct!("HSD_SetupChannel");
    if let Some(ch) = ch { r.setup_channel(g, ch); }
}
pub unsafe fn HSD_SetupChannelAll<G: GxBackend>(r: &mut HsdRender, g: &mut G, mut ch: *const HsdChan) {
    melee_core::ct!("HSD_SetupChannelAll");
    let mut num = 0;
    while !ch.is_null() {
        num = num.max(HSD_Channel2Num((*ch).chan as i32)); r.setup_channel(g, &*ch); ch = (*ch).next;
    }
    r.state_set_num_chans(g, num as u8 as i32);
}
pub unsafe fn HSD_SetupTevStageAll<G: GxBackend>(r: &mut HsdRender, g: &mut G, mut d: *const HSD_TevDesc) {
    melee_core::ct!("HSD_SetupTevStageAll");
    let mut n = 0;
    while !d.is_null() {
        n = n.max(HSD_TevStage2Num((*d).stage as i32)); r.setup_tev_stage(g, &*d); d = (*d).next;
    }
    r.current_tev = n as u32;
    // Unlike StateSetNumTevStages, this entry point does not update committed_tev_stages.
    g.set_num_tev_stages(r.current_tev as u8); r.current_tev = 0;
}
pub fn HSD_StateSetNumChans<G: GxBackend>(r: &mut HsdRender, g: &mut G, n: i32) { melee_core::ct!("HSD_StateSetNumChans"); r.state_set_num_chans(g, n); }
pub fn HSD_StateRegisterTexGen(r: &mut HsdRender, n: i32) { melee_core::ct!("HSD_StateRegisterTexGen"); r.state_register_tex_gen(n as u32); }
pub fn HSD_StateSetNumTexGens<G: GxBackend>(r: &mut HsdRender, g: &mut G) { melee_core::ct!("HSD_StateSetNumTexGens"); r.state_set_num_tex_gens(g); }
pub fn HSD_StateInitTev(r: &mut HsdRender) { melee_core::ct!("HSD_StateInitTev"); r.state_init_tev(); }
pub fn HSD_StateGetNumTevStages(r: &HsdRender) -> i32 { melee_core::ct!("HSD_StateGetNumTevStages"); r.current_tev as i32 }
pub fn HSD_StateGetCommittedTevStages(r: &HsdRender) -> i32 { melee_core::ct!("HSD_StateGetCommittedTevStages"); r.committed_tev_stages as i32 }
pub fn HSD_StateAssignTev(r: &mut HsdRender) -> i32 { melee_core::ct!("HSD_StateAssignTev"); r.state_assign_tev() as i32 }
pub fn HSD_StateSetNumTevStages<G: GxBackend>(r: &mut HsdRender, g: &mut G) { melee_core::ct!("HSD_StateSetNumTevStages"); r.state_set_num_tev_stages(g); }
pub fn HSD_SetupTevStage<G: GxBackend>(r: &mut HsdRender, g: &mut G, d: &HSD_TevDesc) { melee_core::ct!("HSD_SetupTevStage"); r.setup_tev_stage(g, d); }
pub fn HSD_SetTevRegAll<G: GxBackend>(r: &mut HsdRender, g: &mut G) { melee_core::ct!("HSD_SetTevRegAll"); r.set_tev_reg_all(g); }
pub fn _HSD_StateInvalidateColorChannel(r: &mut HsdRender) { melee_core::ct!("_HSD_StateInvalidateColorChannel"); r.invalidate_color_channel(); }
pub fn _HSD_StateInvalidateTevStage(r: &mut HsdRender) { melee_core::ct!("_HSD_StateInvalidateTevStage"); r.invalidate_tev_stage(); }
pub fn _HSD_StateInvalidateTevRegister(r: &mut HsdRender) { melee_core::ct!("_HSD_StateInvalidateTevRegister"); r.invalidate_tev_register(); }
pub fn _HSD_StateInvalidateTexCoordGen(r: &mut HsdRender) { melee_core::ct!("_HSD_StateInvalidateTexCoordGen"); r.invalidate_tex_coord_gen(); }

pub mod ext {
    use melee_types::HSD_ObjAllocData;
    pub static mut OBJ_ALLOC_INIT: Option<unsafe fn(*mut HSD_ObjAllocData, usize, u32)> = None;
    pub static mut TEV_COLOR_S10: Option<fn(u32, [i16; 4])> = None;
    pub unsafe fn HSD_ObjAllocInit(data: *mut HSD_ObjAllocData, size: usize, align: u32) { unsafe { crate::tu::objalloc::HSD_ObjAllocInit(data as _, size as _, align as _); } }
    pub fn GXSetTevColorS10(reg: u32, color: [i16; 4]) {
        unsafe { (TEV_COLOR_S10.expect("TODO: GXSetTevColorS10"))(reg, color); }
    }
}
