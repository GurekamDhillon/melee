#![allow(non_upper_case_globals)]
//! melee_gx against Aurora itself. The vectors in tests/data were written by oracle/build.py, which
//! links the shipping Aurora build and runs Aurora's own code:
//!   tex_oracle.txt    convert_texture / convert_texture_palette over xorshift inputs (FNV-1a of output)
//!   shader_oracle.txt build_shader_source for 400 random ShaderConfigs (length + FNV-1a of the WGSL)
//!   state_oracle.txt  300 random GX API call sequences after GXInit: the ShaderConfig, pipeline state,
//!                     uniform bytes and WGSL a GX_TRIANGLES draw would get
use melee_gx::api::{Gx, GxColor, GxColorS10, GxLightObj};
use melee_gx::consts::*;
use melee_gx::shader::{self, AttrConfig, ShaderConfig};
use melee_gx::state::*;
use melee_gx::texture;
use std::sync::Arc;

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| (self.next() >> 24) as u8).collect()
    }
}

fn fnv(d: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for &b in d {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn data(name: &str) -> String {
    std::fs::read_to_string(format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn texture_decode_matches_aurora() {
    let mut n = 0;
    for line in data("tex_oracle.txt").lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| f[i].parse::<u32>().unwrap();
        match f.first() {
            Some(&"T") => {
                let (fmt, w, h, mips, seed) = (num(1), num(2), num(3), num(4), num(5));
                let in_len = num(6) as usize;
                assert_eq!(texture::texture_source_size(fmt, w, h, mips), in_len, "{line}");
                let input = Rng(seed).bytes(in_len);
                let out = texture::convert_texture(fmt, w, h, mips, &input);
                assert_eq!(out.data.len(), num(7) as usize, "{line}");
                assert_eq!(out.has_arbitrary_mips as u32, num(8), "{line}");
                assert_eq!(format!("{:016x}", fnv(&out.data)), f[9], "{line}");
                n += 1;
            }
            Some(&"P") => {
                let (fmt, tl, e, w, h, mips, seed) = (num(1), num(2), num(3), num(4), num(5), num(6), num(7));
                let in_len = num(8) as usize;
                let mut r = Rng(seed);
                let input = r.bytes(in_len);
                let tlut = r.bytes(e as usize * 2);
                let out = texture::convert_texture_palette(fmt, w, h, mips, &input, tl, e as u16, &tlut);
                assert_eq!(out.data.len(), num(9) as usize, "{line}");
                assert_eq!(out.has_arbitrary_mips as u32, num(10), "{line}");
                assert_eq!(format!("{:016x}", fnv(&out.data)), f[11], "{line}");
                n += 1;
            }
            _ => {}
        }
    }
    assert_eq!(n, 145);
}

/// Parse oracle.cpp's `print_config` line ("C ..."); returns the config and the normal attachment.
fn parse_config(line: &str) -> (ShaderConfig, u32) {
    let mut it = line.split_whitespace().skip(1).map(|t| t.parse::<u64>().unwrap());
    let mut n = || it.next().unwrap() as u32;
    let mut c = ShaderConfig {
        fog_type: n() as u8,
        vtx_stride: n() as u8,
        line_mode: n() as u8,
        fog_range_enabled: n() != 0,
        pn_palette: n() != 0,
        ..Default::default()
    };
    let na = n();
    for a in c.attrs.iter_mut() {
        *a = AttrConfig {
            attr_type: n() as u8,
            cnt: n() as u8,
            comp_type: n() as u8,
            offset: n() as u8,
            stride: n() as u8,
            frac: n() as u8,
            le: n() != 0,
            nbt3: n() != 0,
        };
    }
    for s in c.tev_swap_table.iter_mut() {
        *s = TevSwap { red: n(), green: n(), blue: n(), alpha: n() };
    }
    c.tev_stage_count = n();
    for i in 0..c.tev_stage_count as usize {
        let s = &mut c.tev_stages[i];
        s.color_pass = TevPass { a: n(), b: n(), c: n(), d: n() };
        s.alpha_pass = TevPass { a: n(), b: n(), c: n(), d: n() };
        s.color_op = TevOp { op: n(), bias: n(), scale: n(), out_reg: n(), clamp: n() != 0 };
        s.alpha_op = TevOp { op: n(), bias: n(), scale: n(), out_reg: n(), clamp: n() != 0 };
        s.kc_sel = n();
        s.ka_sel = n();
        s.tex_coord_id = n();
        s.tex_map_id = n();
        s.channel_id = n();
        s.tev_swap_ras = n();
        s.tev_swap_tex = n();
        s.ind_tex_stage = n();
        s.ind_tex_format = n();
        s.ind_tex_bias_sel = n();
        s.ind_tex_alpha_sel = n();
        s.ind_tex_mtx_id = n();
        s.ind_tex_wrap_s = n();
        s.ind_tex_wrap_t = n();
        s.ind_tex_use_orig_lod = n() != 0;
        s.ind_tex_add_prev = n() != 0;
    }
    for cc in c.color_channels.iter_mut() {
        *cc = ColorChannelConfig { mat_src: n(), amb_src: n(), diff_fn: n(), attn_fn: n(), lighting_enabled: n() != 0 };
    }
    for t in c.tcgs.iter_mut() {
        *t = TcgConfig { ty: n(), src: n(), mtx: n(), post_mtx: n(), normalize: n() != 0, emboss_src: n() };
    }
    c.alpha_compare = AlphaCompare { comp0: n(), ref0: n(), op: n(), comp1: n(), ref1: n() };
    c.num_ind_stages = n();
    for i in 0..c.num_ind_stages as usize {
        c.ind_stages[i] = IndStage { tex_coord_id: n(), tex_map_id: n(), scale_s: n(), scale_t: n() };
    }
    (c, na)
}

#[test]
fn wgsl_matches_aurora() {
    let text = data("shader_oracle.txt");
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    let mut n = 0;
    let mut bad = Vec::new();
    for pair in lines.chunks(2) {
        let (cfg, na) = parse_config(pair[0]);
        let w: Vec<&str> = pair[1].split_whitespace().collect();
        let src = shader::build_shader_source(&cfg, na);
        let info = shader::build_shader_info(&cfg);
        if src.len().to_string() != w[1] || format!("{:016x}", fnv(src.as_bytes())) != w[2] {
            bad.push((n, src.len(), w[1].to_string()));
            if bad.len() == 1 {
                let dir = format!("{}/../target/gx-oracle", env!("CARGO_MANIFEST_DIR"));
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(format!("{dir}/rust_{n:04}.wgsl"), &src);
            }
        }
        assert_eq!(info.uniform_size.to_string(), w[3], "uniform size, config {n}");
        n += 1;
    }
    assert!(bad.is_empty(), "{} of {n} WGSL sources differ (first: {:?}; Rust text in target/gx-oracle)", bad.len(), bad[0]);
    assert_eq!(n, 400);
}

fn hexf(t: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(t, 16).unwrap())
}

/// Replay one "O ..." line through melee_gx's API.
fn apply_op(gx: &mut Gx, line: &str, dummy: &melee_gx::GxData) {
    let f: Vec<&str> = line.split_whitespace().collect();
    let u = |i: usize| f[i].parse::<i64>().unwrap() as u32;
    let b = |i: usize| f[i] != "0";
    let col = |i: usize| GxColor::new(u(i) as u8, u(i + 1) as u8, u(i + 2) as u8, u(i + 3) as u8);
    match f[1] {
        "tevorder" => gx.set_tev_order(u(2), u(3), u(4), u(5)),
        "colorin" => gx.set_tev_color_in(u(2), u(3), u(4), u(5), u(6)),
        "alphain" => gx.set_tev_alpha_in(u(2), u(3), u(4), u(5), u(6)),
        "colorop" => gx.set_tev_color_op(u(2), u(3), u(4), u(5), b(6), u(7)),
        "alphaop" => gx.set_tev_alpha_op(u(2), u(3), u(4), u(5), b(6), u(7)),
        "kcsel" => gx.set_tev_kcolor_sel(u(2), u(3)),
        "kasel" => gx.set_tev_kalpha_sel(u(2), u(3)),
        "swapmode" => gx.set_tev_swap_mode(u(2), u(3), u(4)),
        "swaptable" => gx.set_tev_swap_mode_table(u(2), u(3), u(4), u(5), u(6)),
        "numtev" => gx.set_num_tev_stages(u(2) as u8),
        "numchans" => gx.set_num_chans(u(2) as u8),
        "chanctrl" => gx.set_chan_ctrl(u(2), b(3), u(4), u(5), u(6), u(7), u(8)),
        "ambcolor" => gx.set_chan_amb_color(u(2), col(3)),
        "matcolor" => gx.set_chan_mat_color(u(2), col(3)),
        "tevcolor" => gx.set_tev_color(u(2), col(3)),
        "kcolor" => gx.set_tev_kcolor(u(2), col(3)),
        "tevcolors10" => {
            let s = |i: usize| f[i].parse::<i16>().unwrap();
            gx.set_tev_color_s10(u(2), GxColorS10 { r: s(3), g: s(4), b: s(5), a: s(6) })
        }
        "numtexgens" => gx.set_num_tex_gens(u(2) as u8),
        "texgen" => gx.set_tex_coord_gen2(u(2), u(3), u(4), u(5), b(6), u(7)),
        "vtxdesc" => gx.set_vtx_desc(u(2), u(3)),
        "vtxfmt" => gx.set_vtx_attr_fmt(GX_VTXFMT0, u(2), u(3), u(4), u(5) as u8),
        "alphacomp" => gx.set_alpha_compare(u(2), u(3) as u8, u(4), u(5), u(6) as u8),
        "blend" => gx.set_blend_mode(u(2), u(3), u(4), u(5)),
        "zmode" => gx.set_z_mode(b(2), u(3), b(4)),
        "fog" => gx.set_fog(u(2), hexf(f[3]), hexf(f[4]), hexf(f[5]), hexf(f[6]), col(7)),
        "cull" => gx.set_cull_mode(u(2)),
        "mtx" => {
            let mut m = [[0.0f32; 4]; 3];
            for i in 0..12 {
                m[i / 4][i % 4] = hexf(f[5 + i]);
            }
            match u(2) {
                0 => gx.load_pos_mtx_imm(&m, u(3)),
                1 => gx.load_nrm_mtx_imm(&m, u(3)),
                _ => gx.load_tex_mtx_imm(&m, u(3), u(4)),
            }
        }
        "proj" => {
            let mut m = [[0.0f32; 4]; 4];
            for i in 0..16 {
                m[i / 4][i % 4] = hexf(f[3 + i]);
            }
            gx.set_projection(&m, u(2))
        }
        "viewport" => gx.set_viewport(hexf(f[2]), hexf(f[3]), hexf(f[4]), hexf(f[5]), hexf(f[6]), hexf(f[7])),
        "light" => {
            let v: Vec<f32> = (0..12).map(|i| hexf(f[7 + i])).collect();
            let mut lo = GxLightObj::default();
            Gx::init_light_attn(&mut lo, v[0], v[1], v[2], v[3], v[4], v[5]);
            Gx::init_light_pos(&mut lo, v[6], v[7], v[8]);
            Gx::init_light_dir(&mut lo, v[9], v[10], v[11]);
            Gx::init_light_color(&mut lo, col(3));
            gx.load_light_obj_imm(&lo, u(2))
        }
        "numind" => gx.set_num_ind_stages(u(2) as u8),
        "indorder" => gx.set_ind_tex_order(u(2), u(3), u(4)),
        "indscale" => gx.set_ind_tex_coord_scale(u(2), u(3), u(4)),
        "tevind" => gx.set_tev_indirect(u(2), u(3), u(4), u(5), u(6), u(7), u(8), b(9), b(10), u(11)),
        "indmtx" => {
            let e = f[3].parse::<i8>().unwrap();
            let mut m = [[0.0f32; 3]; 2];
            for i in 0..6 {
                m[i / 3][i % 3] = hexf(f[4 + i]);
            }
            gx.set_ind_tex_mtx(u(2), &m, e)
        }
        "update" => match u(2) {
            0 => gx.set_color_update(b(3)),
            1 => gx.set_alpha_update(b(3)),
            _ => gx.set_dst_alpha(b(3), u(4) as u8),
        },
        "array" => gx.set_array(u(2), Some(dummy), u(3), u(4) as u8, b(5)),
        "curmtx" => gx.set_current_mtx(u(2)),
        "lp" => match u(2) {
            0 => gx.set_line_width(u(3) as u8, u(4)),
            1 => gx.set_point_size(u(3) as u8, u(4)),
            _ => gx.set_tex_coord_scale_manually(u(5), b(6), u(7) as u16, u(8) as u16),
        },
        "texoffsets" => gx.enable_tex_offsets(u(2), b(3), b(4)),
        op => panic!("unknown op {op}"),
    }
}

#[test]
fn gx_state_matches_aurora() {
    let text = data("state_oracle.txt");
    let dummy: melee_gx::GxData = Arc::from(vec![0u8; 4096]);
    let mut gx = Gx::new();
    let mut case = String::new();
    let mut expect_c: Option<(ShaderConfig, u32)> = None;
    let mut checked = 0;
    for line in text.lines() {
        match line.as_bytes().first() {
            Some(b'S') => case = line.to_string(),
            Some(b'O') => apply_op(&mut gx, line, &dummy),
            Some(b'C') => {
                gx.flush();
                gx.drain();
                expect_c = Some(parse_config(line));
            }
            Some(b'P') => {
                let pc = shader::populate_pipeline_config(&gx.state, GX_TRIANGLES, GX_VTXFMT0);
                let (want, _) = expect_c.take().unwrap();
                assert_eq!(pc.shader, want, "ShaderConfig after {case}");
                let f: Vec<&str> = line.split_whitespace().collect();
                let got = format!(
                    "P {} {} {} {} {} {} {} {:08x} {:08x} {:08x} {} {} {} {}",
                    pc.depth_func,
                    pc.cull_mode,
                    pc.blend_mode,
                    pc.blend_fac_src,
                    pc.blend_fac_dst,
                    pc.blend_op,
                    pc.dst_alpha,
                    pc.polygon_offset.to_bits(),
                    pc.polygon_offset_scale.to_bits(),
                    pc.polygon_offset_clamp.to_bits(),
                    pc.depth_compare as u32,
                    pc.depth_update as u32,
                    pc.alpha_update as u32,
                    pc.color_update as u32
                );
                assert_eq!(got, f.join(" "), "pipeline state after {case}");
            }
            Some(b'U') => {
                let pc = shader::populate_pipeline_config(&gx.state, GX_TRIANGLES, GX_VTXFMT0);
                let info = shader::build_shader_info(&pc.shader);
                let ub = shader::fill_uniform(&gx.state, &info, &gx.state.loaded_textures);
                let src = shader::build_shader_source(&pc.shader, u32::MAX);
                let got = format!(
                    "U {} {} {:016x} W {} {:016x}",
                    info.uniform_size,
                    ub.len(),
                    fnv(&ub),
                    src.len(),
                    fnv(src.as_bytes())
                );
                assert_eq!(got, line, "uniform / WGSL after {case}");
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(checked, 301);
}

/// GXSetZTexture shaders (not in Aurora, so no oracle text): every oracle config whose last stage samples a texture,
/// with each Z format and op, must still pass naga's WGSL validation (the frag_depth output, the forced sample).
#[test]
fn ztex_wgsl_validates() {
    use wgpu::naga;
    let text = data("shader_oracle.txt");
    let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    let mut n = 0;
    for pair in lines.chunks(2) {
        let (mut cfg, na) = parse_config(pair[0]);
        if cfg.tev_stage_count == 0 {
            continue;
        }
        let last = cfg.tev_stages[cfg.tev_stage_count as usize - 1];
        if last.tex_map_id == melee_gx::consts::GX_TEXMAP_NULL || last.tex_coord_id == melee_gx::consts::GX_TEXCOORD_NULL {
            continue;
        }
        let ok = |src: &str| naga::front::wgsl::parse_str(src).ok().is_some_and(|m| {
            naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all()).validate(&m).is_ok()
        });
        if !ok(&melee_gx::render::adapt_wgsl(&shader::build_shader_source(&cfg, na))) {
            continue; // the oracle's random configs include ones no game draw makes (invalid without a Z texture too)
        }
        if (GX_TG_BUMP0..=GX_TG_BUMP7).contains(&cfg.tcgs[last.tex_coord_id as usize].ty) {
            continue; // an emboss coordinate needs its source coordinate generated too (not a Z texture's case)
        }
        for (op, fmt) in [(2u32, 2u32), (1, 0), (2, 1)] {
            cfg.ztex = op | (fmt << 2) | (0x10 << 8);
            let src = melee_gx::render::adapt_wgsl(&shader::build_shader_source(&cfg, na));
            assert!(src.contains("frag_depth"), "config {n}: no depth output");
            let module = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| panic!("config {n}: {}", e.emit_to_string(&src)));
            naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                .validate(&module)
                .unwrap_or_else(|e| panic!("config {n}: {e:?}"));
        }
        n += 1;
    }
    assert!(n > 0);
}
