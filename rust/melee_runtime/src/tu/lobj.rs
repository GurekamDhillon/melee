//! `src/sysdolphin/baselib/lobj.c`, the GX half the display path needs: the active light list
//! with its masks (`HSD_LObjSetupInit` -> `setup_diffuse_lightobj` / `setup_infinite_lightobj` /
//! `setup_point_lightobj`) and `HSD_LObjSetup`. Lights come from `HSD_LightDesc`s (the stage's
//! light set); light animation (`HSD_LObjAnim`) is not applied, and spot lights are set up as
//! points (no `GXInitLightSpot` cone): none of the stages wired use them.
use super::state::gx::*;
use super::state::{ActiveLight, Dat, GxBackend, HsdRender, LightLoad};
use crate::mtx::{self, Mtx, Vec3};

pub const LOBJ_AMBIENT: u32 = 0;
pub const LOBJ_INFINITE: u32 = 1;
pub const LOBJ_POINT: u32 = 2;
pub const LOBJ_SPOT: u32 = 3;
pub const LOBJ_DIFFUSE: u32 = 1 << 2;
pub const LOBJ_SPECULAR: u32 = 1 << 3;
pub const LOBJ_ALPHA: u32 = 1 << 4;
pub const LOBJ_HIDDEN: u32 = 1 << 5;

/// `HSD_LObjGetPosition` / `GetInterest` of a descriptor (`HSD_WObjDesc.pos` at +4).
fn wobj_pos(d: &Dat, w: Option<u32>, default: Vec3) -> Vec3 {
    match w {
        Some(w) => Vec3 { x: d.f32_(w + 4), y: d.f32_(w + 8), z: d.f32_(w + 0xC) },
        None => default,
    }
}

impl HsdRender {
    /// `HSD_LObjSetupInit` over `descs` (each an `HSD_LightDesc` offset in `d`) under the viewing
    /// matrix `vmtx`: assigns GX_LIGHT0.. in order, loads every diffuse light, rebuilds the masks.
    pub fn lobj_setup_init<G: GxBackend>(&mut self, g: &mut G, d: &Dat, descs: &[u32], vmtx: &Mtx) {
        self.lights.clear();
        self.lightmask_diffuse = 0;
        self.lightmask_specular = 0;
        self.lightmask_alpha = 0;
        self.lightmask_attnfunc = 0;
        let mut n = 0u32;
        for &ld in descs {
            let flags = d.u16_(ld + 8) as u32;
            let color = d.color(ld + 0xC);
            if flags & LOBJ_HIDDEN != 0 {
                continue;
            }
            let ty = flags & 3;
            if ty == LOBJ_AMBIENT {
                self.lights.push(ActiveLight { flags, color, id: 0 });
                continue;
            }
            if n >= 8 {
                continue;
            }
            let id = 1u32 << n;
            n += 1;
            let pos = wobj_pos(d, d.ptr(ld + 0x10), Vec3 { x: 0.0, y: 0.0, z: 1.0 });
            let mut l = LightLoad { color, pos: [0.0; 3], dir: [0.0, 0.0, -1.0], attn: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] };
            if flags & LOBJ_DIFFUSE != 0 {
                // setup_diffuse_lightobj
                self.lightmask_diffuse |= id;
                if ty != LOBJ_INFINITE {
                    self.lightmask_attnfunc |= id;
                }
                if flags & LOBJ_ALPHA != 0 {
                    self.lightmask_alpha |= id;
                }
            }
            match ty {
                LOBJ_INFINITE => {
                    // setup_infinite_lightobj: the position pushed 2^20 out, into view space.
                    let p = Vec3 { x: pos.x * 1048576.0, y: pos.y * 1048576.0, z: pos.z * 1048576.0 };
                    let p = mtx::mult_vec(vmtx, &p);
                    l.pos = [p.x, p.y, p.z];
                }
                _ => {
                    // setup_point_lightobj (attenuation from the desc's union is not modelled:
                    // k0 = 1, no distance falloff).
                    let p = mtx::mult_vec(vmtx, &pos);
                    l.pos = [p.x, p.y, p.z];
                }
            }
            if flags & LOBJ_DIFFUSE != 0 {
                g.load_light(id, &l);
            }
            self.lights.push(ActiveLight { flags, color, id });
        }
    }

    /// `HSD_LObjSetup`: the diffuse light is loaded once by [`Self::lobj_setup_init`]; with no
    /// colour change (the C passes `lobj->color`) and no specular light object, nothing is left.
    pub fn lobj_setup<G: GxBackend>(&mut self, _g: &mut G, _l: &ActiveLight) {}
}

/// The stage light set (gr/ground.c `Ground_801C466C`): the first model group
/// (`map_head->unk8[i]`, 0x34 bytes) with a `x18` light set, a NULL-terminated `LightList*[]` of
/// `{ HSD_LightDesc* desc; HSD_LightAnim** anims; }`. APPROXIMATION: the C picks the group whose
/// stage callback has `flags_b0` and applies `Ground_801C20E0`'s per-light overrides (DIFFUSE /
/// SPECULAR / 0x400 bits from the stage's override table); neither is modelled here.
pub fn stage_light_descs(d: &Dat, map_head: u32) -> Vec<u32> {
    let mut v = Vec::new();
    let Some(groups) = d.ptr(map_head + 8) else { return v };
    let count = d.u32_(map_head + 0xC).min(64);
    let Some(list) = (0..count).find_map(|i| d.ptr(groups + i * 0x34 + 0x18)) else { return v };
    for i in 0..16 {
        let Some(ll) = d.ptr(list + 4 * i) else { break };
        if let Some(desc) = d.ptr(ll) {
            v.push(desc);
        }
    }
    v
}

#[allow(dead_code)]
fn unused(_: Mtx) -> u32 {
    GX_COLOR0
}
