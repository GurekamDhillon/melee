//! Render oracle/scene.inc's test scene through melee_gx (GX API -> display lists -> FIFO decode ->
//! WGSL -> wgpu, offscreen) and compare with Aurora's own screenshot of the same scene
//! (tests/data/render_aurora.png, written by oracle/build.py with Aurora on D3D11).
//!
//! Needs a GPU adapter with push constants. Without one the test prints why and returns (so
//! `cargo test --workspace` stays usable on GPU-less machines); MELEE_GX_REQUIRE_GPU=1 makes that a
//! failure.
#![allow(non_upper_case_globals)]
use melee_gx::api::{Gx, GxColor, GxLightObj};
use melee_gx::consts::*;
use melee_gx::render::Renderer;
use melee_gx::state::TexObj;
use melee_gx::GxData;
use std::sync::Arc;

/// The same bytes oracle/scene.inc builds (keep the two in step).
struct Scene {
    tex: GxData,
    pos: GxData,
    dl_a: Vec<u8>,
    dl_b: Vec<u8>,
}

const POS: [[i16; 2]; 13] = [
    [40, 40], [300, 40], [300, 220], [40, 220],
    [340, 40], [600, 40], [340, 220], [600, 220],
    [170, 350], [60, 260], [280, 260], [280, 440], [60, 440],
];
const CLR: [[u8; 4]; 13] = [
    [255, 255, 255, 255], [255, 64, 64, 255], [64, 255, 64, 255], [64, 64, 255, 255],
    [255, 255, 0, 255], [0, 255, 255, 255], [255, 0, 255, 255], [128, 128, 128, 255],
    [255, 255, 255, 255], [255, 128, 0, 255], [0, 128, 255, 255], [128, 255, 0, 255], [255, 0, 128, 255],
];
const UV: [[f32; 2]; 13] = [
    [0.0, 0.0], [2.0, 0.0], [2.0, 1.5], [0.0, 1.5],
    [0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0],
    [0.5, 0.5], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
];
const B: [[f32; 6]; 6] = [
    [360.0, 260.0, -40.0, 0.0, 0.0, 1.0],
    [600.0, 260.0, -40.0, 0.6, 0.0, 0.8],
    [600.0, 440.0, -40.0, 0.6, 0.6, 0.53],
    [360.0, 260.0, -40.0, 0.0, 0.0, 1.0],
    [600.0, 440.0, -40.0, 0.6, 0.6, 0.53],
    [360.0, 440.0, -40.0, -0.7, 0.2, 0.69],
];

fn build_scene() -> Scene {
    let mut rng = 0x1234567u32;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        rng
    };
    let tex: Vec<u8> = (0..16 * 16 * 2).map(|_| (next() >> 24) as u8).collect();
    let mut pos = vec![0u8; 32 * 6];
    for (i, v) in POS.iter().enumerate() {
        for (k, c) in [v[0] * 4, v[1] * 4, -50 * 4].iter().enumerate() {
            pos[i * 6 + k * 2..i * 6 + k * 2 + 2].copy_from_slice(&c.to_be_bytes());
        }
    }
    let mut a = Vec::new();
    let vtx = |a: &mut Vec<u8>, i: usize| {
        a.push(i as u8);
        a.extend_from_slice(&CLR[i]);
        a.extend_from_slice(&UV[i][0].to_be_bytes());
        a.extend_from_slice(&UV[i][1].to_be_bytes());
    };
    a.push(0x80);
    a.extend_from_slice(&4u16.to_be_bytes());
    (0..4).for_each(|i| vtx(&mut a, i));
    a.push(0x98);
    a.extend_from_slice(&4u16.to_be_bytes());
    (4..8).for_each(|i| vtx(&mut a, i));
    a.push(0xA0);
    a.extend_from_slice(&5u16.to_be_bytes());
    (8..13).for_each(|i| vtx(&mut a, i));
    a.resize(a.len().div_ceil(32) * 32, 0);
    let mut b = vec![0x91];
    b.extend_from_slice(&6u16.to_be_bytes());
    for v in &B {
        for f in v {
            b.extend_from_slice(&f.to_be_bytes());
        }
    }
    b.resize(b.len().div_ceil(32) * 32, 0);
    Scene { tex: Arc::from(tex), pos: Arc::from(pos), dl_a: a, dl_b: b }
}

/// oracle/scene.inc `scene_draw`, call for call.
fn scene_draw(gx: &mut Gx, s: &Scene) {
    gx.set_copy_clear(GxColor::new(32, 48, 64, 255), 0xFF_FFFF);
    gx.set_viewport(0.0, 0.0, 640.0, 480.0, 0.0, 1.0);
    gx.set_scissor(0, 0, 640, 480);
    let mut proj = [[0.0f32; 4]; 4];
    proj[0][0] = 2.0 / 640.0;
    proj[0][3] = -1.0;
    proj[1][1] = -2.0 / 480.0;
    proj[1][3] = 1.0;
    proj[2][2] = -1.0 / 100.0;
    proj[2][3] = -1.0;
    proj[3][3] = 1.0;
    gx.set_projection(&proj, GX_ORTHOGRAPHIC);
    let ident = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
    gx.load_pos_mtx_imm(&ident, GX_PNMTX0);
    gx.load_nrm_mtx_imm(&ident, GX_PNMTX0);
    gx.set_current_mtx(GX_PNMTX0);
    gx.set_cull_mode(GX_CULL_NONE);
    gx.set_z_mode(true, GX_LEQUAL, true);
    gx.set_alpha_compare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
    gx.set_fog(GX_FOG_NONE, 0.0, 1.0, 0.1, 1.0, GxColor::new(0, 0, 0, 0));
    gx.set_num_ind_stages(0);

    let mut tex = TexObj::default();
    gx.init_tex_obj(&mut tex, Some(s.tex.clone()), 16, 16, GX_TF_RGB5A3, GX_REPEAT, GX_REPEAT, false);
    Gx::init_tex_obj_lod(&mut tex, GX_NEAR, GX_NEAR, 0.0, 0.0, 0.0, false, false, GX_ANISO_1);
    gx.load_tex_obj(&mut tex, GX_TEXMAP0);
    gx.clear_vtx_desc();
    gx.set_vtx_desc(GX_VA_POS, GX_INDEX8);
    gx.set_vtx_desc(GX_VA_CLR0, GX_DIRECT);
    gx.set_vtx_desc(GX_VA_TEX0, GX_DIRECT);
    gx.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_S16, 2);
    gx.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    gx.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
    gx.set_array(GX_VA_POS, Some(&s.pos), s.pos.len() as u32, 6, false);
    gx.set_num_chans(1);
    gx.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
    gx.set_num_tex_gens(1);
    gx.set_tex_coord_gen(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY);
    gx.set_num_tev_stages(2);
    gx.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    gx.set_tev_color_in(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_TEXC, GX_CC_RASC, GX_CC_ZERO);
    gx.set_tev_alpha_in(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_RASA, GX_CA_ZERO);
    gx.set_tev_color_op(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
    gx.set_tev_alpha_op(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
    gx.set_tev_order(GX_TEVSTAGE1, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR_NULL);
    gx.set_tev_kcolor(GX_KCOLOR0, GxColor::new(60, 20, 0, 255));
    gx.set_tev_kcolor_sel(GX_TEVSTAGE1, GX_TEV_KCSEL_K0);
    gx.set_tev_color_in(GX_TEVSTAGE1, GX_CC_ZERO, GX_CC_KONST, GX_CC_HALF, GX_CC_CPREV);
    gx.set_tev_alpha_in(GX_TEVSTAGE1, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_APREV);
    gx.set_tev_color_op(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
    gx.set_tev_alpha_op(GX_TEVSTAGE1, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
    gx.set_blend_mode(GX_BM_NONE, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
    gx.call_display_list(&s.dl_a);

    let mut light = GxLightObj::default();
    Gx::init_light_pos(&mut light, 480.0, 240.0, 60.0);
    Gx::init_light_color(&mut light, GxColor::new(255, 240, 200, 255));
    Gx::init_light_attn(&mut light, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    gx.load_light_obj_imm(&light, GX_LIGHT0);
    gx.clear_vtx_desc();
    gx.set_vtx_desc(GX_VA_POS, GX_DIRECT);
    gx.set_vtx_desc(GX_VA_NRM, GX_DIRECT);
    gx.set_vtx_attr_fmt(GX_VTXFMT1, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    gx.set_vtx_attr_fmt(GX_VTXFMT1, GX_VA_NRM, GX_NRM_XYZ, GX_F32, 0);
    gx.set_num_chans(1);
    gx.set_chan_ctrl(GX_COLOR0A0, true, GX_SRC_REG, GX_SRC_REG, GX_LIGHT0, GX_DF_CLAMP, GX_AF_NONE);
    gx.set_chan_amb_color(GX_COLOR0A0, GxColor::new(40, 40, 60, 255));
    gx.set_chan_mat_color(GX_COLOR0A0, GxColor::new(200, 100, 50, 160));
    gx.set_num_tex_gens(0);
    gx.set_num_tev_stages(1);
    gx.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
    gx.set_tev_op(GX_TEVSTAGE0, GX_PASSCLR);
    gx.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
    gx.call_display_list(&s.dl_b);
}

fn read_png(path: &str) -> (u32, u32, Vec<u8>) {
    let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap()));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        t => panic!("unexpected png color type {t:?}"),
    };
    (info.width, info.height, rgba)
}

fn write_png(path: &str, w: u32, h: u32, rgba: &[u8]) {
    let f = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(rgba).unwrap();
}

#[test]
fn scene_matches_aurora_screenshot() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let (aw, ah, aurora) = read_png(&format!("{dir}/tests/data/render_aurora.png"));
    assert_eq!((aw, ah), (640, 480));
    let mut r = match Renderer::new(640, 480) {
        Ok(r) => r,
        Err(e) if std::env::var("MELEE_GX_REQUIRE_GPU").as_deref() != Ok("1") => {
            eprintln!("SKIPPED: no GPU renderer ({e})");
            return;
        }
        Err(e) => panic!("no GPU renderer: {e}"),
    };
    let scene = build_scene();
    let mut gx = Gx::new();
    scene_draw(&mut gx, &scene);
    gx.draw_done();
    let draws = gx.state.take_draws();
    assert_eq!(draws.len(), 4, "quads + strip + fan + lit triangles");
    assert!(gx.state.warnings.is_empty(), "{:?}", gx.state.warnings);
    let ours = r.render(&gx.state, &draws);
    assert!(r.shader_errors.is_empty(), "{}", r.shader_errors.join("\n\n"));
    let out = format!("{dir}/../target/gx-oracle");
    let _ = std::fs::create_dir_all(&out);
    let opaque: Vec<u8> = ours.chunks(4).flat_map(|p| [p[0], p[1], p[2], 255]).collect();
    write_png(&format!("{out}/render_rust.png"), 640, 480, &opaque);

    // Compare RGB (the screenshot's alpha is the swapchain's, not the EFB's).
    let mut max = 0i32;
    let mut differ = 0usize;
    let mut diff_img = vec![0u8; ours.len()];
    for p in 0..(640 * 480) {
        let mut m = 0;
        for c in 0..3 {
            m = m.max((ours[p * 4 + c] as i32 - aurora[p * 4 + c] as i32).abs());
        }
        if m > 0 {
            differ += 1;
        }
        max = max.max(m);
        let v = (m * 32).min(255) as u8;
        diff_img[p * 4..p * 4 + 4].copy_from_slice(&[v, v, v, 255]);
    }
    write_png(&format!("{out}/render_diff.png"), 640, 480, &diff_img);
    eprintln!(
        "adapter {} ({:?}): {differ} of {} pixels differ, max channel difference {max}",
        r.adapter_info.name,
        r.adapter_info.backend,
        640 * 480
    );
    // Same rasterisation rules on any D3D/Vulkan device; per-vertex lighting may round differently.
    assert!(max <= 2, "max channel difference {max} (see target/gx-oracle/render_*.png)");
    assert!(differ <= 640 * 480 / 100, "{differ} pixels differ");
}

// ---- the Envoy effects layer (melee_gx::fx, docs/rust-port/notes/envoy-fx.md)

fn fx_renderer() -> Option<Renderer> {
    match Renderer::new(640, 480) {
        Ok(r) => Some(r),
        Err(e) if std::env::var("MELEE_GX_REQUIRE_GPU").as_deref() != Ok("1") => {
            eprintln!("SKIPPED: no GPU renderer ({e})");
            None
        }
        Err(e) => panic!("no GPU renderer: {e}"),
    }
}

/// A small closed mesh (an octahedron) standing in for a drive model.
fn octa() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let v = [[0.5, 0.0, 0.0], [-0.5, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, -0.5, 0.0], [0.0, 0.0, 0.5], [0.0, 0.0, -0.5]];
    let f = [[0, 2, 4], [2, 1, 4], [1, 3, 4], [3, 0, 4], [2, 0, 5], [1, 2, 5], [3, 1, 5], [0, 3, 5]];
    let (mut p, mut n, mut b, mut g) = (vec![], vec![], vec![], vec![]);
    for t in f {
        for i in t {
            let q: [f32; 3] = v[i];
            p.push(q);
            n.push([q[0] * 2.0, q[1] * 2.0, q[2] * 2.0]);
            b.push([0.8, 0.2, 0.2]);
            g.push([1.0, 0.6, 0.3]);
        }
    }
    (p, n, b, g)
}

fn scene_frame(r: &mut Renderer) {
    let scene = build_scene();
    let mut gx = Gx::new();
    scene_draw(&mut gx, &scene);
    gx.draw_done();
    let draws = gx.state.take_draws();
    r.render(&gx.state, &draws);
}

fn fx_inputs(r: &mut Renderer, posts: &[melee_gx::fx::Post], draws: usize) {
    use melee_gx::fx::*;
    // the scene's camera: identity view, an orthographic box over the 640x480 picture
    let cam = Camera { view: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]], proj: [1.0, 2.0 / 640.0, -1.0, -2.0 / 480.0, 1.0, -0.001, -0.5], vp: [0.0, 0.0, 640.0, 480.0, 0.0, 1.0] };
    let mut f = FxFrame { intensity: 1.0, cam: Some(cam), ..Default::default() };
    for i in 0..draws {
        f.draws.push(WorldDraw { model: 0, mat: [MAT_GLOW, MAT_FOIL, MAT_GLASS, MAT_DISSOLVE][i % 4] | MAT_TRAIL | MAT_OUTLINE, at: [80.0 + i as f32 * 40.0, 400.0 - (i % 3) as f32 * 30.0, -50.0], yaw: i as f32, scale: 36.0, alpha: if i % 4 == 2 { 0.3 } else { 1.0 }, p: [1.0, 0.7, 2.0, if i % 4 == 3 { 0.3 } else { 1.0 }], rgba: 0xFF80_40FF });
    }
    f.posts.extend_from_slice(posts);
    r.set_fx(&mut f);
}

/// With the layer compiled in, the picture is byte for byte the plain one whenever it is disarmed, or armed with
/// nothing to draw; armed with a post effect it changes the edges and never the centre.
#[test]
fn fx_disarmed_is_the_plain_frame() {
    use melee_gx::fx::*;
    let Some(mut r) = fx_renderer() else { return };
    let (w, h) = (800u32, 600u32);
    scene_frame(&mut r);
    let plain = r.read_composed_size(w, h, &[]);
    let efb_plain = r.read_color();
    // armed, nothing queued
    let (p, n, b, g) = octa();
    r.fx_arm(&[FxMesh { pos: &p, nrm: &n, base: &b, glow: &g }]);
    assert!(r.fx_armed());
    fx_inputs(&mut r, &[], 0);
    scene_frame(&mut r);
    assert!(r.read_color() == efb_plain, "armed with nothing: the EFB changed");
    assert!(r.read_composed_size(w, h, &[]) == plain, "armed with nothing: the window changed");
    // armed with a heat-haze and a vignette: the edges change, the centre does not
    for _ in 0..8 {
        fx_inputs(&mut r, &[Post { id: POST_VIGNETTE, strength: 1.0, x: 0.0, y: 0.0, rgba: 0xFF4A_1CFF }, Post { id: POST_HEAT_HAZE, strength: 1.0, x: 0.0, y: 0.0, rgba: 0xFF4A_1CFF }], 0);
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    scene_frame(&mut r);
    let hot = r.read_composed_size(w, h, &[]);
    assert!(hot != plain, "the post effects drew nothing");
    let px = |img: &[u8], x: u32, y: u32| img[((y * w + x) * 4) as usize..((y * w + x) * 4 + 3) as usize].to_vec();
    for y in (h * 3 / 10..h * 7 / 10).step_by(7) {
        for x in (w * 2 / 5..w * 3 / 5).step_by(7) {
            assert_eq!(px(&hot, x, y), px(&plain, x, y), "the centre changed at ({x}, {y})");
        }
    }
    // drives drawn: the EFB changes; then disarmed: byte for byte the plain frame again
    fx_inputs(&mut r, &[], 8);
    scene_frame(&mut r);
    assert!(r.read_color() != efb_plain, "the world layer drew nothing");
    r.fx_disarm();
    fx_inputs(&mut r, &[Post { id: POST_FLASH, strength: 1.0, x: 0.0, y: 0.0, rgba: 0xFFFF_FFFF }], 8);
    scene_frame(&mut r);
    assert!(r.read_color() == efb_plain, "disarmed: the EFB changed");
    assert!(r.read_composed_size(w, h, &[]) == plain, "disarmed: the window changed");
    // the menu tilt: a UI batch at ui_tilt [0, 0] is the untilted one; tilted it moves; back at rest it is again
    let mut atlas = melee_gx::ui::Atlas::new();
    let wq = atlas.white.uv();
    let mut ui = melee_gx::ui::UiFrame::default();
    for i in 0..6 {
        ui.quads.push(melee_gx::ui::UiQuad { rect: [40.0 + i as f32 * 90.0, 60.0 + i as f32 * 50.0, 80.0, 40.0], uv: wq, clip: melee_gx::ui::NO_CLIP, rgba: 0xFF40_80FF, ..Default::default() });
    }
    r.set_ui(Some(&ui), &mut atlas);
    let flat = r.read_composed_size(w, h, &[]);
    r.ui_tilt = [0.3, -0.2];
    assert!(r.read_composed_size(w, h, &[]) != flat, "the tilt moved nothing");
    r.ui_tilt = [0.0, 0.0];
    assert!(r.read_composed_size(w, h, &[]) == flat, "at rest the UI is not the untilted one");
    r.set_ui(None, &mut atlas);
    assert!(r.read_composed_size(w, h, &[]) == plain, "no UI: the window changed");
    let dir = env!("CARGO_MANIFEST_DIR");
    let out = format!("{dir}/../target/gx-oracle");
    let _ = std::fs::create_dir_all(&out);
    write_png(&format!("{out}/fx_hot.png"), w, h, &hot.chunks(4).flat_map(|p| [p[0], p[1], p[2], 255]).collect::<Vec<u8>>());
}

/// The cost of each pass (ms per frame, GPU waited): `cargo test -p melee_gx --release --test render fx_cost -- --ignored --nocapture`.
#[test]
#[ignore]
fn fx_cost() {
    use melee_gx::fx::*;
    let Some(mut r) = fx_renderer() else { return };
    let time = |r: &mut Renderer, f: &mut dyn FnMut(&mut Renderer)| {
        for _ in 0..10 {
            f(r);
        }
        let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
        let t = std::time::Instant::now();
        let n = 200;
        for _ in 0..n {
            f(r);
            let _ = r.device.poll(wgpu::PollType::wait_indefinitely());
        }
        t.elapsed().as_secs_f64() * 1e3 / n as f64
    };
    let tex = |r: &Renderer, w: u32, h: u32| {
        r.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    };
    let (p, n, b, g) = octa();
    for (scale, (ow, oh)) in [(2u32, (1920u32, 1080u32)), (3, (2560, 1440))] {
        r.fx_disarm();
        r.set_target(scale, 4.0 / 3.0);
        let target = tex(&r, ow, oh);
        let view = target.create_view(&Default::default());
        let fmt = wgpu::TextureFormat::Rgba8Unorm;
        scene_frame(&mut r);
        let blit = time(&mut r, &mut |r| r.blit_to(&view, fmt, ow, oh));
        let draw = time(&mut r, &mut |r| scene_frame(r));
        r.fx_arm(&[FxMesh { pos: &p, nrm: &n, base: &b, glow: &g }]);
        for _ in 0..6 {
            fx_inputs(&mut r, &[Post { id: POST_EMBER, strength: 1.0, x: 0.0, y: 0.0, rgba: 0xFF4A_1CFF }, Post { id: POST_HEAT_HAZE, strength: 1.0, x: 0.0, y: 0.0, rgba: 0xFF4A_1CFF }, Post { id: POST_SHOCKWAVE, strength: 1.0, x: 320.0, y: 100.0, rgba: 0xFFFF_FFFF }], 12);
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        let draw_fx = time(&mut r, &mut |r| scene_frame(r));
        let blit_fx = time(&mut r, &mut |r| r.blit_to(&view, fmt, ow, oh));
        eprintln!(
            "fx cost at internal {}x{} -> output {ow}x{oh}: blit {blit:.3} ms, post blit {blit_fx:.3} ms (+{:.3}); frame {draw:.3} ms, with 12 drives + bloom {draw_fx:.3} ms (+{:.3})",
            r.width, r.height, blit_fx - blit, draw_fx - draw
        );
    }
}

/// The sticker effects on the real sticker face (rust/mods/envoy_ui/art/face_white.png), at three moments: plain,
/// foil (Epic) and holo (Unique). Writes fx_stickers_<k>.png to MELEE_FX_SHOT_DIR (default target/gx-oracle).
/// `cargo test -p melee_gx --release --test render fx_sticker_preview -- --ignored`
#[test]
#[ignore]
fn fx_sticker_preview() {
    use melee_gx::ui::*;
    let Some(mut r) = fx_renderer() else { return };
    let dir = env!("CARGO_MANIFEST_DIR");
    let (fw, fh, face) = read_png(&format!("{dir}/../mods/envoy_ui/art/face_white.png"));
    let mut atlas = Atlas::new();
    let tile = atlas.alloc(fw as u16, fh as u16).unwrap();
    atlas.put(tile, &face);
    let mut frame = UiFrame::default();
    let sticker = |q: &mut Vec<UiQuad>, x: f32, y: f32, w: f32, h: f32, fx: [u32; 2]| {
        let (a, b) = (14.0f32, 14.0f32);
        let xs = [x, x + a, x + w - b, x + w];
        let ys = [y, y + a, y + h - b, y + h];
        let us = [0.0, a / 48.0, 1.0 - b / 48.0, 1.0];
        for j in 0..3 {
            for i in 0..3 {
                let (tw, th) = (tile.w as f32, tile.h as f32);
                let uv = tile.sub_uv(us[i] * tw, us[j] * th, (us[i + 1] - us[i]) * tw, (us[j + 1] - us[j]) * th);
                q.push(UiQuad { rect: [xs[i], ys[j], xs[i + 1] - xs[i], ys[j + 1] - ys[j]], uv, clip: NO_CLIP, rgba: 0xFFFF_FFFF, fx, ..Default::default() });
            }
        }
    };
    sticker(&mut frame.quads, 60.0, 60.0, 520.0, 90.0, [FX_NONE, 0]);
    sticker(&mut frame.quads, 60.0, 190.0, 520.0, 90.0, [FX_FOIL, 0xC9BF_F5FF]);
    sticker(&mut frame.quads, 60.0, 320.0, 520.0, 90.0, [FX_HOLO, 0xFFFF_FFFF]);
    scene_frame(&mut r);
    let out = std::env::var("MELEE_FX_SHOT_DIR").unwrap_or(format!("{dir}/../target/gx-oracle"));
    let _ = std::fs::create_dir_all(&out);
    for (k, inten) in [(0, 1.0f32), (1, 1.0), (2, 1.0), (3, 0.5), (4, 0.0)] {
        r.fx_intensity = inten;
        r.set_ui(Some(&frame), &mut atlas);
        let px = r.read_composed_size(640, 480, &[]);
        write_png(&format!("{out}/fx_stickers_{k}_{}.png", ["full", "full", "full", "subtle", "off"][k]), 640, 480, &px.chunks(4).flat_map(|p| [p[0], p[1], p[2], 255]).collect::<Vec<u8>>());
        std::thread::sleep(std::time::Duration::from_millis(700));
    }
}
