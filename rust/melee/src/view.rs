//! `melee view-golden`: the native golden match (vs_match.rs) stepped one frame per tick and drawn
//! every frame through the ported HSD draw path (melee_runtime `tu::displayfunc` -> dobj -> mobj /
//! tobj / tev / state -> pobj) into melee_gx's `GxShim`, presented in a winit window.
//!
//! ```text
//! melee view-golden [--iso DISC.iso] [--scenario pad_falcon_fox_fd] [--frames N]
//!                   [--shots 30,60,90] [--shot-dir rust/target/shots] [--hidden]
//! ```
//!
//! What is drawn: the stage's `map_head` model groups (bind pose: stage animation is not run) and
//! each fighter's costume tree (PlXxNr.dat) with the simulation's joint SRTs, matrices from the
//! ported `HSD_JObjSetupMatrix` on a copy of the tree (the sim's own tree is not touched, so
//! drawing cannot change the simulation). Lights are the stage's light set (`map_head + 0x10`).
//!
//! NOT the game's frame: the camera is a fixed approximation of the match camera (cm/camera.c's
//! CObj path is not ported), no HUD, no effects/particles, no fog, no background CObj, no
//! shadows, no zsort. Screenshots (disc art: never commit) go under rust/target/shots/.
use crate::gx_backend::{DataCache, ShimBackend};
use crate::vs_match::VsMatch;
use crate::Sim;
use melee_gx::api::GxColor;
use melee_gx::consts::*;
use melee_gx::shim::GxShim;
use melee_platform::gfx::{write_png, Presenter, EFB_HEIGHT, EFB_WIDTH};
use melee_platform::{run_windowed, Game, PadSystem, WindowConfig};
use melee_runtime::dat::DatArchive;
use melee_runtime::jobj::{JObjHost, JObjId, JObjTree};
use melee_runtime::mtx::{self, Mtx, Vec3};
use melee_runtime::tu::displayfunc::{DispJoint, DispModel, HSD_TRSP_OPA, HSD_TRSP_TEXEDGE, HSD_TRSP_XLU};
use melee_runtime::tu::lobj::stage_light_descs;
use melee_runtime::tu::state::{Dat, HsdRender};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use winit::window::Window;

type Tree = JObjTree<(), ()>;

/// Matrices only: these trees have no RObjs and are never freed here.
struct NoRObj;
impl JObjHost<(), ()> for NoRObj {
    fn ik_hint_bone_length(&mut self, _: &[()]) -> Option<f32> {
        None
    }
    fn robj_update_all(&mut self, _: &mut Tree, _: JObjId) {}
    fn resolve_ik_joint1(&mut self, _: &mut Tree, _: JObjId) {}
    fn resolve_ik_joint2(&mut self, _: &mut Tree, _: JObjId) {}
    fn unref(&mut self, _: &mut Tree, _: JObjId) {}
}

fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}
fn vec3(d: &[u8], at: u32) -> Vec3 {
    Vec3 { x: bef(d, at), y: bef(d, at + 4), z: bef(d, at + 8) }
}

/// `HSD_JObjLoadJoint` for drawing: the tree (child first, then next, INSTANCE children skipped)
/// plus, per JObj, its descriptor. Allocation order matches skeleton.rs's `load_joint_sub`.
fn load_tree(d: &[u8], at: u32, tree: &mut Tree, parent: Option<JObjId>, descs: &mut Vec<u32>, depth: u32) -> Option<JObjId> {
    if at == 0 || depth > 200 || at as usize + 0x40 > d.len() {
        return None;
    }
    let j = tree.alloc();
    descs.push(at);
    let flags = be32(d, at + 4);
    if flags & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
        let c = load_tree(d, be32(d, at + 8), tree, Some(j), descs, depth + 1);
        tree.get_mut(j).child = c;
    }
    let n = load_tree(d, be32(d, at + 0xC), tree, parent, descs, depth + 1);
    let r = vec3(d, at + 0x14);
    let o = tree.get_mut(j);
    o.next = n;
    o.parent = parent;
    o.flags |= flags;
    o.rotate.x = r.x;
    o.rotate.y = r.y;
    o.rotate.z = r.z;
    o.scale = vec3(d, at + 0x20);
    o.translate = vec3(d, at + 0x2C);
    Some(j)
}

/// The descriptors of an already-loaded tree (skeleton.rs's costume tree), by walking the
/// descriptors in the same allocation order.
fn be16(b: &[u8], at: u32) -> u16 {
    u16::from_be_bytes([b[at as usize], b[at as usize + 1]])
}

fn descs_of(d: &[u8], at: u32, depth: u32, out: &mut Vec<u32>) {
    if at == 0 || depth > 200 {
        return;
    }
    out.push(at);
    if be32(d, at + 4) & melee_runtime::jobj::JOBJ_INSTANCE == 0 {
        descs_of(d, be32(d, at + 8), depth + 1, out);
    }
    descs_of(d, be32(d, at + 0xC), depth + 1, out);
}

/// A drawable: DAT data section, a tree and each JObj's descriptor (index = JObjId.0 order).
struct Drawable {
    dat: Arc<Vec<u8>>,
    roots: Vec<JObjId>,
    descs: Vec<u32>,
}

impl Drawable {
    /// World matrices (ported `HSD_JObjSetupMatrix`) on `tree`, joints in preorder.
    fn disp_joints(&self, tree: &mut Tree) -> Vec<DispJoint> {
        let mut out = Vec::new();
        let mut index = std::collections::HashMap::new();
        for &root in &self.roots {
            let mut order = Vec::new();
            // The root and its `next` chain (walk_tree visits one root and its children).
            let mut x = Some(root);
            while let Some(r) = x {
                tree.walk_tree(Some(r), &mut |j, _| order.push(j));
                x = tree.get(r).next;
            }
            for j in order {
                if index.contains_key(&j.0) {
                    continue;
                }
                tree.setup_matrix(&mut NoRObj, Some(j));
                let o = tree.get(j);
                let parent = o.parent.and_then(|p| index.get(&p.0).copied());
                index.insert(j.0, out.len());
                out.push(DispJoint { desc: self.descs[j.0 as usize], flags: o.flags, mtx: o.mtx, parent });
            }
        }
        out
    }
}

/// The FD stage (`grlast.c`): its map groups and the camera pass (`StageGObj::x11_flags.b012`) each is
/// drawn in. 0 / 2 default 0; 1 and 3 set it to 1 in their on_init; 4-8 to 2 (`grMaterial_801C94D8` users).
const FD_GROUPS: [(u32, u32); 9] = [(0, 0), (1, 1), (2, 0), (3, 1), (4, 2), (5, 2), (6, 2), (7, 2), (8, 2)];
/// `grNLa_803E8010`: { map id, jobj idx for the looping anim, jobj idx of the sub-tree anim (-1 none), its set }.
const FD_BACKDROP: [(u32, u32, i32, i32); 5] = [(4, 1, 2, 0), (5, 1, 2, 4), (6, 1, 2, 6), (7, 1, 2, 8), (8, 1, 2, 10)];

struct Stage {
    dat: Arc<Vec<u8>>,
    anim: crate::stage_anim::StageAnim,
    map_head: u32,
    /// (map id, camera pass) of each drawn group.
    groups: Vec<(u32, u32)>,
    /// `x18` light descriptors of the light group.
    lights: Vec<u32>,
    /// `stage_info.map_plit`'s lights (ftCo_8009F4A4: the set the fighters are lit by); empty = the stage's.
    fighter_lights: Vec<u32>,
    stage_lights: AnimLights,
    plit_lights: Option<AnimLights>,
}

fn load_stage(file: &[u8], is_fd: bool) -> Result<Stage, String> {
    let a = DatArchive::parse(file).map_err(|e| format!("stage: {e}"))?;
    let head = a.public_offset("map_head").ok_or("no map_head")?;
    let a_plit = a.public_offset("map_plit");
    let dat = Arc::new(a.data);
    let d = &dat[..];
    let table = be32(d, head + 8);
    let want: Option<Vec<u32>> = std::env::var("MELEE_VIEW_GROUPS").ok().map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect());
    let fd_groups: Vec<(u32, u32)> = if is_fd {
        FD_GROUPS.to_vec()
    } else {
        // Any other stage: every map group with a root, all in the default camera pass (no stage-specific callbacks yet).
        (0..be32(d, head + 0xC).min(64)).filter(|g| be32(d, table + g * 0x34) != 0).map(|g| (g, 0)).collect()
    };
    let groups: Vec<(u32, u32)> = fd_groups.iter().copied().filter(|(g, _)| want.as_ref().map_or(true, |w| w.contains(g))).collect();
    let ids: Vec<u32> = groups.iter().map(|g| g.0).collect();
    let mut anim = crate::stage_anim::StageAnim::new(dat.clone(), head, &ids);
    // FD only: StageCallbacks.flags_b0 is the MSB of the table's flags word (PPC bitfield order).
    let light_group = (0..10u32).find(|&i| is_fd && unsafe { melee_world::tu::grlast::grLast_StageCallbacks[i as usize].anon_10.flags } & 0x8000_0000 != 0).unwrap_or(0);
    let mut lights = Vec::new();
    let ll = be32(d, table + light_group * 0x34 + 0x18);
    if ll != 0 {
        for i in 0..16 {
            let l = be32(d, ll + 4 * i);
            if l == 0 {
                break;
            }
            if be32(d, l) != 0 {
                lights.push(be32(d, l));
            }
        }
    }
    let mut fighter_lights = Vec::new();
    if let Some(pl) = a_plit {
        for i in 0..16 {
            let l = be32(d, pl + 4 * i);
            if l == 0 {
                break;
            }
            if be32(d, l) != 0 {
                fighter_lights.push(be32(d, l));
            }
        }
    }
    // grLast_OnInit: stage gobjs 0-3 on_init (grAnime_801C8138 set 0), then backdrop case 1 of grLast_8021B920.
    for &g in &ids {
        if is_fd && g <= 3 {
            anim.start_map_anim(g, 0);
        }
    }
    for &(map, i1, i2, set) in &FD_BACKDROP {
        if !is_fd || !ids.contains(&map) {
            continue;
        }
        let rate = if std::env::var("MELEE_VIEW_RATE0").is_ok() { 0.0 } else { 1.0 };
        if i2 != -1 {
            anim.grdo_anim(map, i2 as u32, 7, set, true, 0.0, rate);
        }
        if std::env::var("MELEE_VIEW_NOSET0").is_ok() && map >= 5 { continue; }
        anim.grdo_anim(map, i1, 7, 0, false, 0.0, rate);
        anim.grdo_clear_loop(map, i1, 7);
    }
    if let Ok(n) = std::env::var("MELEE_VIEW_EXTRASET").map(|v| v.parse::<i32>().unwrap_or(1)) {
        for &(map, i1, _i2, _set) in &FD_BACKDROP {
            if is_fd && ids.contains(&map) {
                anim.grdo_anim(map, i1, 7, n, false, 0.0, 1.0);
                anim.grdo_clear_loop(map, i1, 7);
            }
        }
    }
    if is_fd && ids.contains(&4) {
        anim.grdo_anim(4, 0x10, 7, 0, false, 0.0, 1.0);
    }
    if let (Ok(v), Err(_)) = (std::env::var("MELEE_VIEW_DUMP"), std::env::var("MELEE_VIEW_DUMP_FRAME")) {
        for g in v.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
            anim.dump(g);
        }
    }
    if let Ok(v) = std::env::var("MELEE_VIEW_PODUMP") {
        // "g:j": the pobjs of a joint: flags, vertex descriptor attributes, display list primitives.
        if let Some((g, j)) = v.split_once(':') {
            let (g, j): (u32, usize) = (g.parse().unwrap(), j.parse().unwrap());
            let e = be32(d, table + g * 0x34);
            let mut descs = Vec::new();
            descs_of(d, e, 0, &mut descs);
            let jd = descs[j];
            let mut dob = be32(d, jd + 0x10);
            while dob != 0 {
                let mut po = be32(d, dob + 0xC);
                while po != 0 {
                    let (flags, nd) = (be16(d, po + 0xC), be16(d, po + 0xE));
                    let (vd, dl) = (be32(d, po + 8), be32(d, po + 0x10));
                    let mut attrs = String::new();
                    let mut a = vd;
                    while be32(d, a) != 0xFF {
                        attrs += &format!(" [attr {} type {} cnt {} ctype {} frac {} stride {}]", be32(d, a), be32(d, a + 4), be32(d, a + 8), be32(d, a + 0xC), d[a as usize + 0x10], be16(d, a + 0x12));
                        a += 0x18;
                    }
                    let mut prims = String::new();
                    let end = dl + nd as u32 * 32;
                    let mut p = dl;
                    // stride of one vertex in the DL: indexed (type 2/3) = 1/2 bytes; direct is unsupported here
                    let mut vstride = 0u32;
                    let mut aa = vd;
                    while be32(d, aa) != 0xFF { vstride += match be32(d, aa + 4) { 2 => 1, 3 => 2, _ => 0 }; aa += 0x18; }
                    while p < end && d[p as usize] != 0 {
                        let (op, cnt) = (d[p as usize], be16(d, p + 1));
                        prims += &format!(" {:#x}x{}", op, cnt);
                        p += 3 + cnt as u32 * vstride;
                    }
                    eprintln!("PODUMP g{g} j{j} dobj {dob:#x} pobj {po:#x} flags {flags:#x} ndisp {nd} vstride {vstride}:{attrs} | prims{prims}");
                    po = be32(d, po + 4);
                }
                dob = be32(d, dob + 4);
            }
        }
    }
    if std::env::var("MELEE_VIEW_BRANCHSCAN").is_ok() {
        for g in 4..9u32 {
            let e = be32(d, table + g * 0x34);
            let mut descs = Vec::new();
            descs_of(d, e, 0, &mut descs);
            let fl: Vec<String> = descs.iter().map(|&jd| format!("{:#x}", be32(d, jd + 4))).collect();
            eprintln!("STATICFLAGS g{g}: {}", fl.join(" "));
        }
        for g in 4..10u32 {
            let e = be32(d, table + g * 0x34);
            let ajp = be32(d, table + g * 0x34 + 4);
            let _ = e;
            if ajp == 0 { continue; }
            for an in 0..17u32 {
                let set = be32(d, ajp + 4 * an);
                if set == 0 || set as usize + 0x14 > d.len() { continue; }
                for idx in 0..10u32 {
                    let aj = set + idx * 0x14;
                    if aj as usize + 0x14 > d.len() { break; }
                    let aobj = be32(d, aj + 8);
                    if aobj == 0 || aobj as usize + 16 > d.len() { continue; }
                    let mut fo = be32(d, aobj + 8);
                    let mut line = String::new();
                    while fo != 0 && (fo as usize) + 20 <= d.len() {
                        let (len, ty) = (be32(d, fo + 4), d[fo as usize + 12]);
                        if ty == 12 || ty == 11 {
                            let ad = be32(d, fo + 16) as usize;
                            line += &format!(" ty{} fv{:02x} start{} len{} ad{:02x?}", ty, d[fo as usize + 13], bef(d, fo + 8), len, &d[ad..(ad + len as usize).min(d.len()).min(ad + 12)]);
                        }
                        fo = be32(d, fo);
                    }
                    if !line.is_empty() { eprintln!("BRANCH g{g} set{an} idx{idx} endf {}:{line}", bef(d, aobj + 4)); }
                }
            }
        }
    }
    if let Ok(n) = std::env::var("MELEE_VIEW_PREROLL").map(|v| v.parse::<u32>().unwrap_or(0)) {
        for _ in 0..n {
            anim.step();
        }
    }
    if let Ok(v) = std::env::var("MELEE_VIEW_UNHIDE") {
        for g in v.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
            anim.set_hidden_all(g, 0, false);
        }
    }
    if let Ok(v) = std::env::var("MELEE_VIEW_HIDE") {
        for it in v.split(',') {
            if let Some((m, j)) = it.split_once(':') {
                if let (Ok(m), Ok(j)) = (m.trim().parse::<u32>(), j.trim().parse::<u32>()) {
                    anim.hide(m, j);
                }
            }
        }
    }
    if let Ok(v) = std::env::var("MELEE_VIEW_TEXDUMP") {
        // Dev aid: per joint of the named groups, each DObj's MObj render flags and TObj images.
        for g in v.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
            let e = be32(d, table + g * 0x34);
            let mut descs = Vec::new();
            descs_of(d, e, 0, &mut descs);
            for (ji, &jd) in descs.iter().enumerate() {
                let mut dob = be32(d, jd + 0x10);
                while dob != 0 {
                    let mob = be32(d, dob + 8);
                    if std::env::var("MELEE_VIEW_TEXDUMP2").is_ok() && mob != 0 {
                        let mut t = be32(d, mob + 8);
                        while t != 0 {
                            let fl = |o: u32| f32::from_bits(be32(d, t + o));
                            eprintln!("  TOBJ {t:#x} rot {:.2} {:.2} {:.2} scale {:.3} {:.3} {:.3} trans {:.3} {:.3} {:.3} wrap {} {} rep {} {} flags {:#x} blend {:.2} filt {} tlut {:#x} lod {:#x}", fl(0x10), fl(0x14), fl(0x18), fl(0x1C), fl(0x20), fl(0x24), fl(0x28), fl(0x2C), fl(0x30), be32(d, t + 0x34), be32(d, t + 0x38), d[t as usize + 0x3C], d[t as usize + 0x3D], be32(d, t + 0x40), fl(0x44), be32(d, t + 0x48), be32(d, t + 0x50), be32(d, t + 0x54));
                            let tv = be32(d, t + 0x58);
                            if tv != 0 {
                                eprintln!("    TEV desc {tv:#x}: color {:02x?} alpha {:02x?} konst {:02x?} tev0 {:02x?} tev1 {:02x?} active {:#x} ops {:02x?}", &d[tv as usize + 8..tv as usize + 12], &d[tv as usize + 12..tv as usize + 16], &d[tv as usize + 0x10..tv as usize + 0x14], &d[tv as usize + 0x14..tv as usize + 0x18], &d[tv as usize + 0x18..tv as usize + 0x1C], be32(d, tv + 0x1C), &d[tv as usize..tv as usize + 8]);
                            }
                            t = be32(d, t + 4);
                        }
                    }
                    let mut line = format!("g{g} j{ji} dobj {dob:#x} mobj {mob:#x}");
                    if mob != 0 {
                        line += &format!(" rflags {:#x}", be32(d, mob + 4));
                        let mut t = be32(d, mob + 8);
                        while t != 0 {
                            let img = be32(d, t + 0x4C);
                            line += &format!(" | tobj {t:#x} flags {:#x} blend {:.2}", be32(d, t + 0x40), f32::from_bits(be32(d, t + 0x44)));
                            if img != 0 {
                                line += &format!(" img {}x{} fmt {}", be16(d, img + 4), be16(d, img + 6), be32(d, img + 8));
                            }
                            t = be32(d, t + 4);
                        }
                    }
                    eprintln!("{line}");
                    dob = be32(d, dob + 4);
                }
            }
        }
    }
    let stage_lights = AnimLights::new(d, if ll != 0 { ll } else { 0 }, head);
    let plit_lights = a_plit.map(|pl| AnimLights::new(d, pl, head));
    Ok(Stage { dat, anim, map_head: head, groups, lights, fighter_lights, stage_lights, plit_lights })
}

/// A light set (`lb_80011AC4` over a `LightList*[]`) with its `HSD_LObjAnim` state: the `VIS` / `LITC_*`
/// tracks of each light's `anims[0]` (positions stay as authored).
struct AnimLights {
    lights: Vec<AnimLight>,
    globals: melee_runtime::aobj::AObjGlobals,
}

struct AnimLight {
    desc: u32,
    flags: u32,
    color: [u8; 4],
    aobj: Option<melee_runtime::aobj::HsdAObj<()>>,
}

impl AnimLights {
    /// `list` is the offset of a NULL-terminated `LightList*[]`; `shadow_tbl` the stage's `Ground_801C43C4` table
    /// (`map_head + 0x20`: { anim, flag } pairs) deciding which light animations loop.
    fn new(d: &[u8], list: u32, head: u32) -> Self {
        let mut lights = Vec::new();
        let (tbl, tn) = (be32(d, head + 0x20), be32(d, head + 0x24));
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
    fn step(&mut self) {
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

    fn ins(&self) -> Vec<melee_runtime::tu::lobj::LightIn> {
        self.lights.iter().map(|l| melee_runtime::tu::lobj::LightIn { desc: l.desc, flags: l.flags, color: l.color }).collect()
    }
}

struct FighterModel {
    dat: Arc<Vec<u8>>,
    descs: Vec<u32>,
}

/// What it takes to draw a `VsMatch`: the stage and costume models, the match camera and the HSD render state.
pub struct MatchRender {
    cm: crate::cm_drive::CmDrive,
    pose: crate::cm_drive::CamPose,
    stage: Stage,
    hud: crate::hud_view::Hud,
    fighters: Vec<FighterModel>,
    cache: DataCache,
    render: HsdRender,
}

pub struct View {
    sim: VsMatch,
    sim_alive: bool,
    mr: MatchRender,
    shim: GxShim,
    presenter: Option<Presenter>,
    frame: u64,
    frames: Option<u64>,
    shots: Vec<u64>,
    shot_dir: PathBuf,
    quit: bool,
    pub failed: Option<String>,
}

/// The view matrix and projection of a CObj pose (`HSD_CObjGetViewingMtx`, MTXPerspective).
fn camera(p: &crate::cm_drive::CamPose) -> (Mtx, [[f32; 4]; 4]) {
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

impl MatchRender {
    pub fn new(disc: &mut crate::disc::Disc, stage_file: &str, names: &[&str], sim: &mut VsMatch) -> Result<Self, String> {
        let stage_bytes = disc.read(stage_file)?;
        let stage = load_stage(&stage_bytes, stage_file.ends_with("GrNLa.dat") || stage_file.ends_with("GrNLa.usd"))?;
        let cm = {
            let a = DatArchive::parse(&stage_bytes).map_err(|e| format!("{stage_file}: {e}"))?;
            let gp = crate::stage::ground_param(&a).ok_or("stage dat: no grGroundParam")?;
            let mut points = crate::stage::spawn_points(&stage_bytes)?;
            let sc = bef(&a.data, gp);
            if sc != 1.0 && sc > 0.001 {
                for p in &mut points {
                    for c in &mut p.pos {
                        *c *= sc;
                    }
                }
            }
            let (cs, fixed_zoom, smooth, tilt) = crate::cm_drive::stage_cam(&a.data, gp, &points)?;
            let ins: Vec<_> = sim.players.iter_mut().filter_map(|p| p.camera_subject_in()).collect();
            if std::env::var("MELEE_VIEW_CAMLOG").is_ok() {
                eprintln!("RCAMSTAGE {:?} fixed_zoom {fixed_zoom} smooth {smooth}", cs);
                for i in &ins {
                    eprintln!("RCAMSUBJ {:?}", i);
                }
            }
            let d = crate::cm_drive::CmDrive::new(cs, fixed_zoom, smooth, tilt, sim.blast_top(), &ins);
            d
        };
        let mut fighters = Vec::new();
        for (slot, n) in names.iter().enumerate() {
            let (nr, sym) = crate::fighter_data::costume_for_slot(n, slot).ok_or("no costume")?;
            let a = DatArchive::parse(&disc.read(nr)?).map_err(|e| format!("{nr}: {e}"))?;
            let at = a.public_offset(sym).ok_or(format!("{nr}: no {sym}"))?;
            let mut descs = Vec::new();
            descs_of(&a.data, at, 0, &mut descs);
            fighters.push(FighterModel { dat: Arc::new(a.data), descs });
        }
        let hud = crate::hud_view::Hud::new(crate::hud_view::HudFile::load(&disc.read("IfAll.usd")?)?, names)?;
        let pose = cm.pose();
        Ok(Self { cm, pose, stage, hud, fighters, cache: DataCache::default(), render: HsdRender::new() })
    }
}

impl MatchRender {
    /// Draw the stage and fighters into the shim's EFB (no copy-out: the caller adds UI, then ends the frame).
    pub fn draw(&mut self, shim: &mut GxShim, sim: &VsMatch) {
        shim.set_copy_clear(GxColor::new(0, 0, 0, 255), 0xFF_FFFF);
        shim.set_viewport(0.0, 0.0, EFB_WIDTH as f32, EFB_HEIGHT as f32, 0.0, 1.0);
        shim.set_scissor(0, 0, EFB_WIDTH, EFB_HEIGHT);
        let sd = self.stage.dat.clone();
        let (vmtx, proj) = camera(&self.pose);
        shim.set_projection(&proj, GX_PERSPECTIVE);
        // Joint matrices and animated materials of this frame.
        let mut stage_joints: Vec<(u32, u32, Vec<DispJoint>)> = Vec::new();
        for (g, pass) in self.stage.groups.clone() {
            stage_joints.push((g, pass, self.stage.anim.disp_joints(g)));
        }
        let over = self.stage.anim.overrides();
        let mut fighter_models: Vec<(Arc<Vec<u8>>, Vec<DispJoint>)> = Vec::new();
        for (p, f) in sim.players.iter().zip(&self.fighters) {
            // ftDrawCommon_80080A18: `!fighter->invisible && !fighter->x221E_b5` gates the body (Entry sets
            // `invisible`; the entry beam / trophy accessory stands in for it).
            if p.fp.invisible() != 0 || p.fp.x221E_b5() != 0 {
                continue;
            }
            let mut t = p.skel.tree.clone();
            let d = Drawable { dat: f.dat.clone(), roots: vec![p.skel.root], descs: f.descs.clone() };
            fighter_models.push((f.dat.clone(), d.disp_joints(&mut t)));
        }
        let r = &mut self.render;
        r.invalidate_all();
        let mut g = ShimBackend { shim, cache: &mut self.cache };
        r.lobj_setup_init_with(&mut g, &Dat(&sd), &self.stage.stage_lights.ins(), &vmtx);
        let stage_dms: Vec<(u32, DispModel)> = stage_joints.iter().map(|(gid, pass, j)| (*pass, DispModel::new(&sd, j.clone()).with_mat_anim(&over))).collect();
        let fighter_dms: Vec<DispModel> = fighter_models.iter().map(|(d, j)| DispModel::new(d, j.clone())).collect();
        // fn_800301D0's gx-link schedule: the codes (OPA = 0, XLU = 1, TEXEDGE = 2) run as the outer loop.
        let mask_of = |code: u32| 1u32 << code;
        let mut stage_pass = |r: &mut HsdRender, g: &mut ShimBackend, pass: u32, codes: &[u32]| {
            for &c in codes {
                for (gp, m) in &stage_dms {
                    if *gp == pass {
                        r.clear_vtx_desc(g);
                        r.jobj_disp_all(g, m, &vmtx, mask_of(c), 0);
                    }
                }
            }
        };
        stage_pass(r, &mut g, 2, &[0, 1, 2]);
        stage_pass(r, &mut g, 1, &[0, 1, 2]);
        stage_pass(r, &mut g, 0, &[0, 1]);
        let own_lights = !self.stage.fighter_lights.is_empty() && std::env::var_os("MELEE_VIEW_NOPLIT").is_none();
        let plit_ins = self.stage.plit_lights.as_ref().map(|p| p.ins()).unwrap_or_default();
        if own_lights {
            r.lobj_setup_init_with(&mut g, &Dat(&sd), &plit_ins, &vmtx);
        }
        for c in [0u32, 1] {
            for m in &fighter_dms {
                r.clear_vtx_desc(&mut g);
                r.jobj_disp_all(&mut g, m, &vmtx, mask_of(c), 0);
            }
        }
        if own_lights {
            r.lobj_setup_init_with(&mut g, &Dat(&sd), &self.stage.stage_lights.ins(), &vmtx);
        }
        stage_pass(r, &mut g, 0, &[2]);
        if own_lights {
            r.lobj_setup_init_with(&mut g, &Dat(&sd), &plit_ins, &vmtx);
        }
        for m in &fighter_dms {
            r.clear_vtx_desc(&mut g);
            r.jobj_disp_all(&mut g, m, &vmtx, mask_of(2), 0);
        }
        if std::env::var_os("MELEE_VIEW_NOPS").is_none() {
            // efLib_render_callback: gx link 7 draws particle links 0, 2 and the stage's (3), link 8 draws link 1.
            let mut ep = 0u64;
            for (links, sw) in [(0x1 | 0x4 | 0x8u32, 1u32), (0x1 | 0x4 | 0x8, 2), (0x2, 1), (0x2, 2)] {
                crate::ps_view::disp_particles(g.shim, g.cache, &sim.ps, links, sw, &vmtx, &mut ep);
            }
            r.invalidate_all();
        }
        if std::env::var_os("MELEE_VIEW_NOHUD").is_none() {
            self.hud.draw(r, &mut g, camera);
        }
    }
}

impl View {
    pub fn draw_frame(&mut self) {
        self.mr.draw(&mut self.shim, &self.sim);
        self.shim.copy_disp(true);
    }
}

impl MatchRender {
    /// One match frame's render-side update, after `sim.step()`: the stage animation and the match camera.
    pub fn step(&mut self, sim: &mut VsMatch) {
        self.stage.anim.step();
        if std::env::var_os("MELEE_VIEW_NOLIGHTANIM").is_none() {
            self.stage.stage_lights.step();
            if let Some(p) = self.stage.plit_lights.as_mut() {
                p.step();
            }
        }
        if let Ok(v) = std::env::var("MELEE_VIEW_ONLY") {
            // "g:j,j,j": per frame, hide every joint of group g except the listed ones.
            if let Some((g, js)) = v.split_once(':') {
                let g: u32 = g.parse().unwrap_or(0);
                let keep: Vec<u32> = js.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                for j in 0..self.stage.anim.jobj_count(g) as u32 {
                    if !keep.contains(&j) { self.stage.anim.hide(g, j); }
                }
            }
        }
        if let Ok(v) = std::env::var("MELEE_VIEW_FORCEVIS") {
            for g in v.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
                self.stage.anim.set_hidden_all(g, 0, false);
            }
        }
        if std::env::var("MELEE_VIEW_HIDLOG").is_ok() && sim.frame % 20 == 0 {
            eprintln!("HID frame {} {}", sim.frame, self.stage.anim.hidden_summary());
        }
        if let (Ok(v), Ok(f)) = (std::env::var("MELEE_VIEW_DUMP"), std::env::var("MELEE_VIEW_DUMP_FRAME")) {
            if f.parse::<u64>().ok() == Some(sim.frame as u64) {
                for g in v.split(',').filter_map(|x| x.trim().parse::<u32>().ok()) {
                    self.stage.anim.dump(g);
                }
            }
        }
        if std::env::var("MELEE_VIEW_PSLOG").is_ok() && sim.frame % 100 == 0 {
            let mut by: std::collections::BTreeMap<(usize, u8, u8, u32), u32> = Default::default();
            for l in 0..melee_runtime::particle::NUM_LINKS {
                for id in sim.ps.particles(l) {
                    let p = sim.ps.part(id);
                    *by.entry((l, p.bank, p.tex_group, p.kind)).or_default() += 1;
                }
            }
            for l in 0..melee_runtime::particle::NUM_LINKS {
                for id in sim.ps.particles(l).into_iter().take(4) {
                    let p = sim.ps.part(id);
                    eprintln!("  PSP link {l} bank {} tg {} pose {} kind {:#x} size {:.3} pos {:.1} {:.1} {:.1} prim {:?} cnt {} rot {:.2} appsrt {:?} aCmp {} {} {}", p.bank, p.tex_group, p.pose_num, p.kind, p.size, p.pos.x, p.pos.y, p.pos.z, p.prim_col, p.prim_col_count, p.rotate, p.appsrt, p.a_cmp_mode, p.a_cmp_param1, p.a_cmp_param2);
                }
            }
            eprintln!("PS frame {} gens {} parts {} | (link,bank,texgrp,kind)=n {:?}", sim.frame, sim.ps.num_gens, sim.ps.num_parts, by);
        }
        if std::env::var("MELEE_VIEW_FPLOG").is_ok() && sim.frame % 5 == 0 && sim.frame < 120 {
            for (i, p) in sim.players.iter().enumerate() {
                let f = &p.fp;
                eprintln!("FP frame {} p{} ms {} pos {:.1} {:.1} invisible {} x221E_b5 {} x2219_b1 {} ", sim.frame, i, f.motion_id, f.cur_pos.x, f.cur_pos.y, f.invisible(), f.x221E_b5(), f.x2219_b1());
            }
        }
        if std::env::var("MELEE_VIEW_POSLOG").is_ok() && sim.frame % 10 == 0 {
            let p = &sim.players;
            eprintln!("POS frame {} p1 {:.2} {:.2} p2 {:.2} {:.2}", sim.frame, p[0].fp.cur_pos.x, p[0].fp.cur_pos.y, p[1].fp.cur_pos.x, p[1].fp.cur_pos.y);
        }
        let ins: Vec<_> = sim.players.iter_mut().filter_map(|p| p.camera_subject_in()).collect();
        let snaps: Vec<_> = sim.players.iter_mut().filter(|p| !p.follower).map(|p| p.cam_snap.take()).collect();
        self.cm.step(&ins, &snaps);
        let dmg: Vec<i32> = sim.players.iter().map(|p| p.fp.dmg.x1830_percent as i32).collect();
        self.hud.step(&dmg);
        self.pose = self.cm.pose();
        {
            // ifnametag.c fn_802FCC44: the head position (+ the fighter's name_tag_height - 2.5), projected by the match camera.
            let (vm, pj) = camera(&self.pose);
            let tags: Vec<Option<(f32, f32)>> = sim
                .players
                .iter()
                .filter(|p| !p.follower)
                .map(|p| {
                    let f = &p.fp;
                    let w = [f.cur_pos.x, f.cur_pos.y + f.co_attrs.name_tag_height - 2.5, f.cur_pos.z, 1.0];
                    let v = [
                        vm[0][0] * w[0] + vm[0][1] * w[1] + vm[0][2] * w[2] + vm[0][3],
                        vm[1][0] * w[0] + vm[1][1] * w[1] + vm[1][2] * w[2] + vm[1][3],
                        vm[2][0] * w[0] + vm[2][1] * w[1] + vm[2][2] * w[2] + vm[2][3],
                        1.0,
                    ];
                    let c: Vec<f32> = (0..4).map(|i| pj[i][0] * v[0] + pj[i][1] * v[1] + pj[i][2] * v[2] + pj[i][3] * v[3]).collect();
                    if c[3] <= 0.0 {
                        return None;
                    }
                    Some((640.0 * (c[0] / c[3] * 0.5 + 0.5), 480.0 * (0.5 - c[1] / c[3] * 0.5)))
                })
                .collect();
            self.hud.set_nametags(&tags);
        }
        if std::env::var("MELEE_VIEW_CAMLOG").is_ok() {
            let p = &self.pose;
            eprintln!("RCAM {} {:.5} {:.5} {:.5} {:.5} {:.5} {:.5} {:.4}", sim.frame, p.eye[0], p.eye[1], p.eye[2], p.interest[0], p.interest[1], p.interest[2], p.fov);
        }
    }
}

impl Game for View {
    fn tick(&mut self, _pads: &mut PadSystem) {}
    fn present(&mut self, window: &Arc<Window>, _alpha: f64) {
        if self.presenter.is_none() {
            match Presenter::new(window.clone()) {
                Ok(p) => self.presenter = Some(p),
                Err(e) => {
                    self.failed = Some(e);
                    self.quit = true;
                    return;
                }
            }
        }
        // One sim frame per presented frame (the golden's frame index = self.frame).
        if self.sim_alive {
            self.sim_alive = self.sim.step();
            self.mr.step(&mut self.sim);
            if !self.sim_alive {
                eprintln!("view-golden: simulation stopped after frame {}: {:?}", self.sim.frame, self.sim.stopped);
            }
        }
        // MELEE_VIEW_SKIP: only frames named in --shots are drawn (the sim still steps every frame).
        if std::env::var("MELEE_VIEW_SKIP").is_ok() && !self.shots.contains(&self.frame) {
            self.frame += 1;
            if self.frames.is_some_and(|n| self.frame >= n) {
                self.quit = true;
            }
            return;
        }
        self.draw_frame();
        let frame = self.shim.take_frame();
        let p = self.presenter.as_mut().unwrap();
        p.present(&self.shim.gx.state, frame.as_deref());
        self.shim.log.clear();
        if self.frame == 0 {
            let s = self.mr.render.stats;
            eprintln!(
                "view-golden: first frame {} draws, {} pobjs ({} envelope, {} shape-anim skipped), {} shader errors",
                frame.as_ref().map_or(0, |f| f.len()),
                s.pobjs,
                s.envelope_pobjs,
                s.shape_anim_skipped,
                p.renderer.shader_errors.len()
            );
            if let Some(e) = p.renderer.shader_errors.first() {
                eprintln!("{e}");
            }
        }
        if self.shots.contains(&self.frame) {
            let px = p.screenshot();
            let path = self.shot_dir.join(format!("native_{:04}.png", self.frame));
            match write_png(&path, p.renderer.width, p.renderer.height, &px) {
                Ok(()) => eprintln!("view-golden: frame {} (sim frame {}) -> {}", self.frame, self.sim.frame, path.display()),
                Err(e) => eprintln!("view-golden: screenshot: {e}"),
            }
        }
        self.frame += 1;
        if self.frames.is_some_and(|n| self.frame >= n) {
            self.quit = true;
        }
    }
    fn resized(&mut self, w: u32, h: u32) {
        if let Some(p) = self.presenter.as_mut() {
            p.resize(w, h);
        }
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
}

pub fn view_golden(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee view-golden --iso DISC.iso [--frames N] [--shots 30,60] [--shot-dir DIR]");
        return ExitCode::from(2);
    };
    let scenario = get("--scenario").unwrap_or_else(|| "pad_falcon_fox_fd".into());
    let (stage_file, names): (&str, &[&str]) = match scenario.as_str() {
        "pad_falcon_fox_fd" => ("GrNLa.dat", &["falcon", "fox"]),
        s => {
            eprintln!("view-golden: scenario {s:?} not wired");
            return ExitCode::from(2);
        }
    };
    let built = (|| -> Result<View, String> {
        let mut disc = crate::disc::Disc::open(&iso)?;
        // Dev aid: MELEE_DUMP_FILES=a.dat,b.dat MELEE_DUMP_DIR=<scratch dir> writes disc files out for
        // offline inspection (never commit them).
        if let (Ok(names), Ok(dir)) = (std::env::var("MELEE_DUMP_FILES"), std::env::var("MELEE_DUMP_DIR")) {
            for n in names.split(',') {
                let b = disc.read(n.trim())?;
                std::fs::write(std::path::Path::new(&dir).join(n.trim()), b).map_err(|e| e.to_string())?;
            }
            return Err("dumped".into());
        }
        let mut sim = VsMatch::new(&mut disc, stage_file, names, 12345, 3)?;
        let mr = MatchRender::new(&mut disc, stage_file, names, &mut sim)?;
        let shot_dir = PathBuf::from(get("--shot-dir").unwrap_or_else(|| "target/shots".into()));
        std::fs::create_dir_all(&shot_dir).map_err(|e| format!("{}: {e}", shot_dir.display()))?;
        Ok(View {
            sim,
            sim_alive: true,
            mr,
            shim: GxShim::new(),
            presenter: None,
            frame: 0,
            frames: get("--frames").and_then(|v| v.parse().ok()),
            shots: get("--shots").map_or(vec![30, 60, 90], |s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()),
            shot_dir,
            quit: false,
            failed: None,
        })
    })();
    let view = match built {
        Ok(v) => v,
        Err(e) => {
            eprintln!("view-golden: {e}");
            return ExitCode::FAILURE;
        }
    };
    let cfg = WindowConfig { title: format!("melee view-golden - {scenario}"), width: 1280, height: 960, active: std::env::var_os("MELEE_PAD_SCRIPT").is_none() };
    match run_windowed(view, PadSystem::headless(), cfg) {
        Ok((ticks, _)) => {
            eprintln!("view-golden: closed after {ticks} ticks");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("view-golden: event loop: {e}");
            ExitCode::FAILURE
        }
    }
}
