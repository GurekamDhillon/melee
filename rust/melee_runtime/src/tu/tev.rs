//! `src/sysdolphin/baselib/tev.c`: colour-channel setup with tev.c's colour cache, the TEV stage
//! counter and the texgen count (methods on [`HsdRender`]).
use super::state::gx::*;
use super::state::{GxBackend, HsdChan, HsdRender, Rgba};

/// One cached colour register (`prev_ch[no].amb_color` / `.mat_color`, `prev_*_invalid[no]`).
fn cached_color<G: GxBackend>(g: &mut G, slot: &mut Option<Rgba>, chan: u32, no: u32, c: Rgba, amb: bool) {
    let set = |g: &mut G, id: u32| if amb { g.set_chan_amb_color(id, c) } else { g.set_chan_mat_color(id, c) };
    match slot {
        None => {
            // invalid: load the COLORnAn register (no + 4) and remember all four bytes.
            set(g, no + 4);
            *slot = Some(c);
        }
        Some(p) => {
            if chan == GX_COLOR0A0 || chan == GX_COLOR1A1 {
                if *p != c {
                    *p = c;
                    set(g, chan);
                }
            } else if chan == GX_COLOR0 || chan == 1 {
                if p[..3] != c[..3] {
                    p[..3].copy_from_slice(&c[..3]);
                    set(g, chan);
                }
            } else if c[3] != p[3] {
                p[3] = c[3];
                set(g, chan);
            }
        }
    }
}

impl HsdRender {
    /// `HSD_SetupChannel`.
    pub fn setup_channel<G: GxBackend>(&mut self, g: &mut G, ch: &HsdChan) {
        if ch.chan == GX_COLOR_NULL {
            return;
        }
        let no = ch.chan & 1;
        if ch.enable && ch.amb_src == GX_SRC_REG {
            cached_color(g, &mut self.prev_amb[no as usize], ch.chan, no, ch.amb_color, true);
        }
        if ch.mat_src == GX_SRC_REG {
            cached_color(g, &mut self.prev_mat[no as usize], ch.chan, no, ch.mat_color, false);
        }
        // The control word's cache (`prev_ch[idx]`) only skips identical rewrites; GXSetChanCtrl
        // is idempotent, so writing it every time leaves the same GX state.
        g.set_chan_ctrl(ch.chan, ch.enable, ch.amb_src, ch.mat_src, ch.light_mask, ch.diff_fn, ch.attn_fn);
    }

    /// `HSD_StateSetNumChans`.
    pub fn state_set_num_chans<G: GxBackend>(&mut self, g: &mut G, num: i32) {
        if self.prev_num_chans != num {
            g.set_num_chans(num as u8);
        }
    }

    /// `HSD_StateRegisterTexGen` (`HSD_TexCoordID2Num(coord)` = coord + 1 for GX_TEXCOORD0..7).
    pub fn state_register_tex_gen(&mut self, coord: u32) {
        let n = if coord < 8 { coord + 1 } else { 0 };
        if n > self.num_tex_gens {
            self.num_tex_gens = n;
        }
    }

    /// `HSD_StateSetNumTexGens`.
    pub fn state_set_num_tex_gens<G: GxBackend>(&mut self, g: &mut G) {
        g.set_num_tex_gens(self.num_tex_gens as u8);
        self.num_tex_gens = 0;
    }

    /// `HSD_StateInitTev`.
    pub fn state_init_tev(&mut self) {
        self.current_tev = 0;
    }

    /// `HSD_StateAssignTev` (`HSD_Index2TevStage` is the identity for 0..15).
    pub fn state_assign_tev(&mut self) -> u32 {
        let s = self.current_tev;
        self.current_tev += 1;
        s
    }

    /// `HSD_StateSetNumTevStages`.
    pub fn state_set_num_tev_stages<G: GxBackend>(&mut self, g: &mut G) {
        self.stats.tev_stages += self.current_tev;
        g.set_num_tev_stages(self.current_tev as u8);
        self.current_tev = 0;
    }
}
