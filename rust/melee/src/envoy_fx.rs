//! The presenter's half of the Envoy effects layer (melee_gx::fx, docs/rust-port/notes/envoy-fx.md): each present,
//! the drives the UI mod queued (melee_modes::tu::drive_draw), the fight camera the fighter pass recorded and the
//! queued post effects become one `FxFrame` for the renderer. Arms the layer when Envoy needs it and disarms it when
//! nothing does; with Envoy off nothing here creates or draws anything.

use melee_gx::fx::{FxFrame, FxMesh, WorldDraw, MAT_BODY, MAT_GLASS, MAT_TRAIL};
use melee_modes::tu::drive_draw;

/// Reused across presents (steady state allocates nothing).
#[derive(Default)]
pub struct EnvoyFx {
    frame: FxFrame,
    tris: Vec<drive_draw::Tri>,
    idle: u32,
    demo_n: u32,
}

impl EnvoyFx {
    pub fn present(&mut self, r: &mut melee_gx::render::Renderer, netplay: bool) {
        let posts = melee_gx::fx::posts_pending() || std::env::var_os("MELEE_FX_DEMO").is_some();
        let want = melee_modes::tu::envoy::armed() != 0 || drive_draw::shown() > 0 || posts;
        if !want {
            // a second of nothing: disarm (every pass off; objects kept for the next run)
            if r.fx_armed() {
                self.idle += 1;
                if self.idle > 60 {
                    r.fx_disarm();
                }
            }
            if !r.fx_armed() {
                return;
            }
        } else {
            self.idle = 0;
        }
        if !r.fx_armed() {
            let ms = drive_draw::models();
            let meshes: Vec<FxMesh<'_>> = ms.iter().map(|m| FxMesh { pos: &m.pos, nrm: &m.nrm, base: &m.base, glow: &m.glow }).collect();
            let t = std::time::Instant::now();
            r.fx_arm(&meshes);
            eprintln!("play: envoy fx: armed ({:.1} ms to build the effect pipelines)", t.elapsed().as_secs_f64() * 1e3);
        }
        let f = &mut self.frame;
        f.intensity = melee_gx::fx::intensity_from_env(netplay);
        r.fx_intensity = f.intensity;
        f.cam = drive_draw::camera().map(|c| melee_gx::fx::Camera { view: c.view, proj: c.proj, vp: c.vp });
        f.draws.clear();
        let glass_a = melee_gx::fx::glass_params().0;
        let draws = &mut f.draws;
        drive_draw::world_frame(
            |model, at, yaw, scale, alpha, mat, p, rgba| {
                // the old import's glass shell: the glass material at the material json's opacity
                let (mat, alpha) = if model as usize == drive_draw::M_GLASS && mat & 0x7F == MAT_BODY { (MAT_GLASS | (mat & MAT_TRAIL), alpha.min(glass_a)) } else { (mat, alpha) };
                draws.push(WorldDraw { model, mat, at, yaw, scale, alpha, p, rgba });
            },
            &mut self.tris,
        );
        f.tris.clear();
        f.tris.extend_from_slice(&self.tris);
        melee_gx::fx::take_posts(&mut f.posts);
        demo(&mut f.posts, &mut self.demo_n);
        r.set_fx(f);
    }
}

/// `MELEE_FX_DEMO` = comma list of warm | hot | blazing | cashout | break: the screenshot harness's stand-in posts
/// (each Heat level held; a cash-out / boss break every 90 presents, off the centre). Test aid only.
fn demo(posts: &mut Vec<melee_gx::fx::Post>, n: &mut u32) {
    use melee_gx::fx::*;
    let Ok(v) = std::env::var("MELEE_FX_DEMO") else { return };
    *n += 1;
    let mut add = |id, s, x, y, c| {
        if posts.len() < MAX_POST {
            posts.push(Post { id, strength: s, x, y, rgba: c });
        }
    };
    for w in v.split(',') {
        match w.trim() {
            "warm" => add(POST_VIGNETTE, 0.5, 0.0, 0.0, 0xFFBE_2EFF),
            "hot" => {
                add(POST_VIGNETTE, 0.7, 0.0, 0.0, 0xFF8A_1FFF);
                add(POST_EMBER, 0.6, 0.0, 0.0, 0xFF8A_1FFF);
            }
            "blazing" => {
                add(POST_EMBER, 0.8, 0.0, 0.0, 0xFF4A_1CFF);
                add(POST_HEAT_HAZE, 1.0, 0.0, 0.0, 0xFF4A_1CFF);
            }
            "cashout" if *n % 90 == 1 => {
                add(POST_RADIAL_WAVE, 1.0, 140.0, 330.0, 0xFFD8_4AFF);
                add(POST_FLASH, 0.8, 0.0, 0.0, 0xFFF4_D0FF);
            }
            "break" if *n % 90 == 45 => {
                add(POST_SHOCKWAVE, 1.0, 470.0, 120.0, 0xFF4D_6DFF);
                add(POST_FLASH, 1.0, 0.0, 0.0, 0xFFFF_FFFF);
            }
            _ => {}
        }
    }
}
