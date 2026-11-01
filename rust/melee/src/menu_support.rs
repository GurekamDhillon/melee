//! What the retail menu / title stand-ins (retail/) still use of the old model viewer: the CObj pose and its view / projection
//! matrices, and the light set with its `HSD_LObjAnim` tracks.
use melee_runtime::mtx::{self, Mtx, Vec3};

fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}

fn be16(b: &[u8], at: u32) -> u16 {
    u16::from_be_bytes([b[at as usize], b[at as usize + 1]])
}

/// The CObj pose `Camera_8002AF68` leaves (what the draw uses).
#[derive(Clone, Copy, Debug)]
pub struct CamPose {
    pub eye: [f32; 3],
    pub interest: [f32; 3],
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

/// A light set (`lb_80011AC4` over a `LightList*[]`) with its `HSD_LObjAnim` state: the `VIS` / `LITC_*`
/// tracks of each light's `anims[0]` (positions stay as authored).
pub(crate) struct AnimLights {
    pub(crate) lights: Vec<AnimLight>,
    globals: melee_runtime::aobj::AObjGlobals,
}

pub(crate) struct AnimLight {
    pub(crate) desc: u32,
    pub(crate) flags: u32,
    pub(crate) color: [u8; 4],
    aobj: Option<melee_runtime::aobj::HsdAObj<()>>,
}

impl AnimLights {
    /// `list` is the offset of a NULL-terminated `LightList*[]`; `shadow_tbl` the stage's `Ground_801C43C4` table
    /// (`map_head + 0x20`: { anim, flag } pairs) deciding which light animations loop.
    fn new(d: &[u8], list: u32, head: u32) -> Self {
        Self::new_with_table(d, list, be32(d, head + 0x20), be32(d, head + 0x24))
    }

    /// A scene's light set (no stage loop table: no light animation loops).
    pub(crate) fn new_scene(d: &[u8], list: u32) -> Self {
        Self::new_with_table(d, list, 0, 0)
    }

    fn new_with_table(d: &[u8], list: u32, tbl: u32, tn: u32) -> Self {
        let mut lights = Vec::new();
        for i in 0..16 {
            let ll = if list == 0 { 0 } else { be32(d, list + 4 * i) };
            if ll == 0 {
                break;
            }
            let desc = be32(d, ll);
            if desc == 0 {
                continue;
            }
            let anims = be32(d, ll + 4);
            let anim0 = if anims != 0 { be32(d, anims) } else { 0 };
            let mut aobj = None;
            if anim0 != 0 {
                if let Some(ad) = crate::stage_anim::load_aobj_desc(d, be32(d, anim0 + 4)) {
                    let mut a = melee_runtime::aobj::HsdAObj::<()>::load_desc(&ad, |_| None);
                    // HSD_LObjReqAnimAll(.., 0) + HSD_AObjSetRate(1.0) (HSD_ForeachAnim passes the AObj and the float).
                    a.req_anim(0.0);
                    a.set_rate(1.0);
                    let looped = (0..tn).any(|k| be32(d, tbl + 8 * k) == anim0 && d[(tbl + 8 * k + 4) as usize] & 0x80 != 0);
                    if looped {
                        a.set_flags(melee_runtime::aobj::AOBJ_LOOP);
                    }
                    aobj = Some(a);
                }
            }
            lights.push(AnimLight { desc, flags: be16(d, desc + 8) as u32, color: [d[desc as usize + 0xC], d[desc as usize + 0xD], d[desc as usize + 0xE], d[desc as usize + 0xF]], aobj });
        }
        if std::env::var_os("MELEE_VIEW_LIGHTLOG").is_some() {
            for l in &lights {
                eprintln!("LIGHT desc {:#x} flags {:#x} color {:02x?} aobj {:?}", l.desc, l.flags, l.color, l.aobj.as_ref().map(|a| (a.end_frame, a.fobj.iter().map(|f| f.obj_type).collect::<Vec<_>>())));
            }
        }
        let mut s = Self { lights, globals: Default::default() };
        s.step(); // HSD_LObjAnimAll at creation
        s
    }

    /// One `HSD_LObjAnimAll` (`LObjUpdateFunc` over the tracks).
    pub(crate) fn step(&mut self) {
        use melee_runtime::lobj::track;
        for l in &mut self.lights {
            if let Some(a) = l.aobj.as_mut() {
                let (flags, color) = (&mut l.flags, &mut l.color);
                let mut f = |ty: u8, fv: f32| {
                    let u8_of = |v: f32| ((255.0 * v as f64) as u32) as u8;
                    match ty as u32 {
                        track::VIS => {
                            if (fv as f64) < 0.5 {
                                *flags &= !(melee_runtime::tu::lobj::LOBJ_HIDDEN);
                            } else {
                                *flags |= melee_runtime::tu::lobj::LOBJ_HIDDEN;
                            }
                        }
                        track::LITC_R => color[0] = u8_of(fv),
                        track::LITC_G => color[1] = u8_of(fv),
                        track::LITC_B => color[2] = u8_of(fv),
                        track::LITC_A => color[3] = u8_of(fv),
                        _ => {}
                    }
                };
                a.interpret_anim(&mut self.globals, &mut Some(&mut f));
            }
        }
    }

    pub(crate) fn ins(&self) -> Vec<melee_runtime::tu::lobj::LightIn> {
        self.lights.iter().map(|l| melee_runtime::tu::lobj::LightIn { desc: l.desc, flags: l.flags, color: l.color }).collect()
    }
}

/// The view matrix and projection of a CObj pose (`HSD_CObjGetViewingMtx`, MTXPerspective).
pub(crate) fn camera(p: &CamPose) -> (Mtx, [[f32; 4]; 4]) {
    let eye = Vec3 { x: p.eye[0], y: p.eye[1], z: p.eye[2] };
    let at = Vec3 { x: p.interest[0], y: p.interest[1], z: p.interest[2] };
    let f = mtx::vec_normalize(&mtx::vec_sub(&eye, &at));
    let up = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let cross = |a: &Vec3, b: &Vec3| Vec3 { x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x };
    let dot = |a: &Vec3, b: &Vec3| a.x * b.x + a.y * b.y + a.z * b.z;
    let l = mtx::vec_normalize(&cross(&up, &f));
    let u = cross(&f, &l);
    let view = [
        [l.x, l.y, l.z, -dot(&l, &eye)],
        [u.x, u.y, u.z, -dot(&u, &eye)],
        [f.x, f.y, f.z, -dot(&f, &eye)],
    ];
    let (n, fz) = (p.near, p.far);
    let c = 1.0 / (p.fov.to_radians() * 0.5).tan();
    let s = 1.0 / (fz - n);
    let proj = [
        [c / p.aspect, 0.0, 0.0, 0.0],
        [0.0, c, 0.0, 0.0],
        [0.0, 0.0, -n * s, -fz * n * s],
        [0.0, 0.0, -1.0, 0.0],
    ];
    (view, proj)
}

