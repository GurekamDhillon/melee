//! Petey's real look, loaded at runtime from the owner's own extracted Brawl files (notes/petey-extract.md).
//!
//! Presentation only: nothing here feeds the sim (the boss state lives in `petey.rs`); the draw reads `petey::pose()`
//! and the model is drawn on top of it. When the local data is absent (the usual case) `get()` is `None` and the
//! placeholder capsules are drawn instead.
//!
//! Where: `MELEE_BRAWL_LOCAL` (a folder holding `disc/files/...`), else the first `_build/brawl` found walking up
//! from the working folder or the exe. Nothing from it is ever written anywhere.
//!
//! What is read: the adventure stage archive holding the boss (ARC, LZ77 inside), the BRRES with Petey's body MDL0
//! and the two arm / cage MDL0s, their TEX0 textures (CMPR / RGB / I formats) and their CHR0 animations. Each part
//! is kept as a rig (bones, matrix sources, unlit triangles); the draw poses it per frame from a CHR0 (bone
//! matrices -> skinning -> a fixed light). Which animation and frame is chosen by `petey::draw` from the sim state.
#![allow(clippy::missing_safety_doc)]

use std::path::{Path, PathBuf};

pub struct Model {
    pub body: Rig,
    pub arm_l: Option<Rig>,
    pub arm_r: Option<Rig>,
    /// The boss's enemy param file (hurtbox sets, camera range), when found.
    pub params: Option<Params>,
    /// The arena's blast zones (left, right, top, bottom) from its stage position model's Dead0N / Dead1N bones.
    pub arena_blast: Option<[f32; 4]>,
}

/// One hurtbox record (BrawlBox "HurtBox" layout): offset and stretch in the bone's space, radius, flag word whose
/// top byte is the bone index.
#[derive(Clone, Copy, Debug)]
pub struct Hurt {
    pub off: [f32; 3],
    pub stretch: [f32; 3],
    pub r: f32,
    pub flags: u32,
}

/// What is read from the boss's enemy param file (a relocatable section: header, data, relocations, named roots).
#[derive(Default, Debug)]
pub struct Params {
    pub body: Vec<Hurt>,
    pub arm_l: Vec<Hurt>,
    pub arm_r: Vec<Hurt>,
    /// cameraRangeSet floats, CameraClipSphereData floats
    pub camera_range: Vec<f32>,
    pub camera_clip: Vec<f32>,
}

fn params(d: &[u8]) -> Option<Params> {
    if d.len() < 0x20 || be32(d, 0) as usize != d.len() {
        return None;
    }
    let (ds, rc, r1, r2) = (be32(d, 4) as usize, be32(d, 8) as usize, be32(d, 12) as usize, be32(d, 16) as usize);
    let t = 0x20 + ds + rc * 4;
    let names = t + (r1 + r2) * 8;
    let root = |want: &str| -> Option<usize> {
        (0..r1 + r2).find_map(|i| {
            let (o, n) = (be32(d, t + 8 * i) as usize, be32(d, t + 8 * i + 4) as usize);
            (cstr(d, names + n) == want).then_some(0x20 + o)
        })
    };
    // a set: (data offset of the records, count)
    let set = |want: &str| -> Vec<Hurt> {
        let Some(r) = root(want) else { return vec![] };
        let (p, n) = (0x20 + be32(d, r) as usize, be32(d, r + 4) as usize);
        (0..n.min(16))
            .map(|i| {
                let q = p + 0x20 * i;
                let v = |o: usize| [bef(d, q + o), bef(d, q + o + 4), bef(d, q + o + 8)];
                Hurt { off: v(0), stretch: v(12), r: bef(d, q + 24), flags: be32(d, q + 28) }
            })
            .collect()
    };
    let floats = |want: &str, n: usize| -> Vec<f32> {
        let Some(r) = root(want) else { return vec![] };
        let p = 0x20 + be32(d, r) as usize;
        (0..n).map(|i| bef(d, p + 4 * i)).collect()
    };
    let body = set("HitDataSet");
    if body.is_empty() {
        return None;
    }
    Some(Params {
        body,
        arm_l: set("wpnHitDataSet_ArmLeft"),
        arm_r: set("wpnHitDataSet_ArmRight"),
        camera_range: floats("cameraRangeSet", 8),
        camera_clip: floats("CameraClipSphereData", 5),
    })
}

/// The model was loaded (without loading it: the HUD asks every frame).
pub fn loaded() -> bool {
    LOADED.load(std::sync::atomic::Ordering::Relaxed)
}
static LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Petey's model, loaded once on first use; `None` without local data (or if it failed to parse).
pub fn get() -> Option<&'static Model> {
    static M: std::sync::OnceLock<Option<Model>> = std::sync::OnceLock::new();
    M.get_or_init(|| {
        if std::env::var_os("MELEE_PETEY_NO_MODEL").is_some() {
            return None;
        }
        // load on a helper thread: its allocations stay on the system heap, outside the game arena that rollback
        // snapshots rewrite (the model must survive every load / resimulation)
        let r = std::thread::spawn(load).join().unwrap_or_else(|_| Err("loader panicked".into()));
        match &r {
            Ok(m) => eprintln!("petey: real model loaded ({} animations)", m.body.anims.len()),
            Err(e) => eprintln!("petey: no real model ({e}); placeholder look"),
        }
        LOADED.store(r.is_ok(), std::sync::atomic::Ordering::Relaxed);
        r.ok()
    })
    .as_ref()
}

/// Whether the player's own Brawl files are present (the boss archive under the local Brawl folder). Envoy requires them
/// (the full-game plan, Q3: no Brawl disc, no Envoy); checked once per process, never prints the path.
pub fn brawl_present() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        if std::env::var_os("MELEE_BRAWL_ABSENT").is_some() {
            return false; // test knob: behave as if the files were missing
        }
        local_root().is_some_and(|r| r.join("disc").join("files").join("stage").join("adventure").join(STAGE).is_file())
    })
}

/// Whether the owner's Brawl folder has adventure archive `name` (`MELEE_BRAWL_ABSENT`: as if missing).
pub fn adventure_present(name: &str) -> bool {
    std::env::var_os("MELEE_BRAWL_ABSENT").is_none()
        && local_root().is_some_and(|r| r.join("disc").join("files").join("stage").join("adventure").join(name).is_file())
}

fn local_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("MELEE_BRAWL_LOCAL") {
        return Some(PathBuf::from(p));
    }
    let mut starts = vec![];
    if let Ok(c) = std::env::current_dir() {
        starts.push(c);
    }
    if let Ok(e) = std::env::current_exe() {
        if let Some(d) = e.parent() {
            starts.push(d.to_path_buf());
        }
    }
    for s in starts {
        let mut d: Option<&Path> = Some(&s);
        while let Some(p) = d {
            let c = p.join("_build").join("brawl");
            if c.join("disc").join("files").is_dir() {
                return Some(c);
            }
            d = p.parent();
        }
    }
    None
}

/// The adventure stage archive that carries the boss (found by scanning; see notes/petey-extract.md).
const STAGE: &str = "030101.pac";
/// A BRRES file's fixed header (magic, byte order, version, size, root offset, section count): 16 bytes.
pub const BRRES_HEADER_LEN: usize = 16;

fn load() -> Result<Model, String> {
    let root = local_root().ok_or("no local Brawl folder")?;
    let f = root.join("disc").join("files").join("stage").join("adventure").join(STAGE);
    let raw = std::fs::read(&f).map_err(|e| format!("boss archive not readable ({})", e.kind()))?;
    let mut leaves = vec![];
    walk(&unwrap(raw), 0, &mut leaves);
    let mut tex_at: Vec<(String, usize, usize)> = vec![]; // (name, leaf, offset)
    let mut plt_at: Vec<(String, usize, usize)> = vec![];
    let mut mdls: Vec<(String, Vec<u8>, usize)> = vec![];
    let mut chrs: Vec<(String, Vec<u8>, usize)> = vec![];
    let par = leaves.iter().find_map(|b| params(b));
    for (li, b) in leaves.iter().enumerate() {
        if b.len() < 0x10 || &b[..4] != b"bres" {
            continue;
        }
        let Some(entries) = brres_entries(b) else { continue };
        for (name, off) in entries {
            match b.get(off..off + 4) {
                Some(b"TEX0") => {
                    if !tex_at.iter().any(|(n, _, _)| *n == name) {
                        tex_at.push((name, li, off));
                    }
                }
                Some(b"PLT0") => {
                    if !plt_at.iter().any(|(n, _, _)| *n == name) {
                        plt_at.push((name, li, off));
                    }
                }
                Some(b"CHR0") => {
                    if !chrs.iter().any(|(n, _, _)| *n == name) {
                        chrs.push((name, b.clone(), off));
                    }
                }
                Some(b"MDL0") => {
                    if !mdls.iter().any(|(n, _, _)| *n == name) {
                        mdls.push((name, b.clone(), off));
                    }
                }
                _ => {}
            }
        }
    }
    // every texture once, shared by the parts that use it (the body atlases are on the cage models too)
    let texs: Vec<(String, std::sync::Arc<Tex>)> = tex_at
        .iter()
        .filter_map(|(n, li, o)| {
            let plt = plt_at.iter().find(|(pn, _, _)| pn == n).map(|(_, pl, po)| (leaves[*pl].as_slice(), *po));
            tex0(&leaves[*li], *o, plt).map(|t| (n.clone(), std::sync::Arc::new(t)))
        })
        .collect();
    let rig = |want: &str| -> Option<Rig> {
        let (_, b, off) = mdls.iter().find(|(n, _, _)| n == want)?;
        let mut r = mdl0(b, *off, &texs)?;
        // its animations: CHR0 `<model><Name>`, kept by suffix
        for (n, cb, co) in &chrs {
            if let Some(suf) = n.strip_prefix(want) {
                if !suf.is_empty() && !suf.starts_with("Arm") {
                    r.anims.push((suf.to_string(), chr0(cb, *co)));
                }
            }
        }
        // extent from the first frame of the idle (bind pose without one): size and depth centre stay put while animating
        let mut tris = vec![];
        r.skin(r.anim("Wait1"), 0.0, &mut tris);
        (r.min, r.max) = extent(&tris);
        Some(r)
    };
    let body = rig("EnmBosspackun").ok_or("body model not found")?;
    let arena_blast = leaves.iter().find_map(|b| arena_blast(b));
    Ok(Model { body, arm_l: rig("WpnBosspackunArmL"), arm_r: rig("WpnBosspackunArmR"), params: par, arena_blast })
}

/// A bone's bind translation by name in an MDL0 inside `d`: the bone record whose name offset (+8, relative to the
/// record) points at the name; translation at [off].
pub fn bone_trans(d: &[u8], name: &[u8]) -> Option<[f32; 2]> {
    let mut want = name.to_vec();
    want.push(0);
    let locs: Vec<usize> = d.windows(want.len()).enumerate().filter(|(_, w)| *w == want.as_slice()).map(|(i, _)| i).collect();
    if locs.is_empty() {
        return None;
    }
    (0..d.len().saturating_sub(0x48)).step_by(4).find_map(|s| {
        let no = bei(d, s + 8);
        let t = s as i64 + no;
        if no != 0 && t >= 0 && locs.contains(&(t as usize)) && bef(d, s + 0x20) == 1.0 {
            Some([bef(d, s + 0x38), bef(d, s + 0x3C)])
        } else {
            None
        }
    })
}

/// The stage's blast zones from Dead0N (left, top) and Dead1N (right, bottom), if this leaf holds them.
pub fn arena_blast(d: &[u8]) -> Option<[f32; 4]> {
    if d.len() < 0x10 || &d[..4] != b"bres" {
        return None;
    }
    let a = bone_trans(d, b"Dead0N")?;
    let b = bone_trans(d, b"Dead1N")?;
    let z = [a[0].min(b[0]), a[0].max(b[0]), a[1].max(b[1]), a[1].min(b[1])];
    (z[0] < -50.0 && z[1] > 50.0 && z[2] > 50.0 && z[3] < -50.0).then_some(z)
}

/// The generic form of [`load`] (Subspace enemies, `subspace_model.rs`): the MDL0s named in `wants` from one local
/// archive `rel` (path parts under `disc/files`), each as a rig with the CHR0s named `<anim prefix><Suffix>` kept by
/// suffix and its extent taken in the first frame of `idle` (the bind pose without it). `wants` = (model name,
/// animation prefix). Call it on a helper thread, as `get` does: the rigs must live on the system heap.
pub fn load_rigs(rel: &[&str], wants: &[(&str, &str, &str)]) -> Result<Vec<Option<Rig>>, String> {
    load_rigs_with(rel, wants, true)
}

/// [`load_rigs`] with the root-motion strip optional (`strip` false: the clips keep their travel, for a host that
/// reads it as movement and holds it itself, `vm_boss_model.rs`).
pub fn load_rigs_with(rel: &[&str], wants: &[(&str, &str, &str)], strip: bool) -> Result<Vec<Option<Rig>>, String> {
    let root = local_root().ok_or("no local Brawl folder")?;
    let mut f = root.join("disc").join("files");
    for p in rel {
        f = f.join(p);
    }
    let raw = std::fs::read(&f).map_err(|e| format!("archive not readable ({})", e.kind()))?;
    let mut leaves = vec![];
    walk(&unwrap(raw), 0, &mut leaves);
    let mut tex_at: Vec<(String, usize, usize)> = vec![];
    let mut plt_at: Vec<(String, usize, usize)> = vec![];
    let mut mdls: Vec<(String, usize, usize)> = vec![];
    let mut chrs: Vec<(String, usize, usize)> = vec![];
    let mut viss: Vec<(String, usize, usize)> = vec![];
    for (li, b) in leaves.iter().enumerate() {
        if b.len() < 0x10 || &b[..4] != b"bres" {
            continue;
        }
        let Some(entries) = brres_entries(b) else { continue };
        for (name, off) in entries {
            let list = match b.get(off..off + 4) {
                Some(b"TEX0") => &mut tex_at,
                Some(b"PLT0") => &mut plt_at,
                Some(b"CHR0") => &mut chrs,
                Some(b"VIS0") => &mut viss,
                Some(b"MDL0") => &mut mdls,
                _ => continue,
            };
            if !list.iter().any(|(n, _, _)| *n == name) {
                list.push((name, li, off));
            }
        }
    }
    let texs: Vec<(String, std::sync::Arc<Tex>)> = tex_at
        .iter()
        .filter_map(|(n, li, o)| {
            let plt = plt_at.iter().find(|(pn, _, _)| pn == n).map(|(_, pl, po)| (leaves[*pl].as_slice(), *po));
            tex0(&leaves[*li], *o, plt).map(|t| (n.clone(), std::sync::Arc::new(t)))
        })
        .collect();
    if std::env::var_os("MELEE_MDL_LOG").is_some() {
        let pre = wants.first().map_or("", |w| w.1);
        let names: Vec<&str> = mdls.iter().map(|m| m.0.as_str()).collect();
        let cn: Vec<&str> = chrs.iter().map(|m| m.0.as_str()).filter(|n| !n.starts_with(pre)).collect();
        eprintln!("petey_model: {} MDL0s {:?}; other CHR0s {:?}", rel.join("/"), names, cn);
    }
    Ok(wants
        .iter()
        .map(|(want, prefix, idle)| {
            let (_, li, off) = mdls.iter().find(|(n, _, _)| n == want)?;
            let mut r = mdl0(&leaves[*li], *off, &texs)?;
            for (n, cl, co) in &chrs {
                if let Some(suf) = n.strip_prefix(prefix) {
                    if !suf.is_empty() {
                        r.anims.push((suf.to_string(), chr0(&leaves[*cl], *co)));
                    }
                }
            }
            for (n, vl, vo) in &viss {
                if let Some(suf) = n.strip_prefix(prefix) {
                    if !suf.is_empty() {
                        r.vis.push((suf.to_string(), vis0(&leaves[*vl], *vo)));
                    }
                }
            }
            // the motion files: Brawl keeps one BRRES per motion, in the motion list's order (an animation-only
            // BRRES whose every entry carries the model's name: CHR0, VIS0, CLR0, PAT0, SRT0)
            for b in &leaves {
                if b.len() < 0x10 || &b[..4] != b"bres" {
                    continue;
                }
                let Some(es) = brres_entries(b) else { continue };
                let kind = |o: usize| b.get(o..o + 4);
                let anim_only = !es.is_empty()
                    && es.iter().all(|(n, o)| n.starts_with(prefix) && matches!(kind(*o), Some(b"CHR0" | b"VIS0" | b"CLR0" | b"PAT0" | b"SRT0" | b"SHP0")));
                if anim_only {
                    let of = |k: &[u8]| es.iter().find(|(_, o)| kind(*o) == Some(k)).and_then(|(n, _)| n.strip_prefix(prefix)).map(str::to_string);
                    r.motion_files.push((of(b"CHR0"), of(b"VIS0")));
                }
            }
            if strip {
                r.strip_root_motion();
            }
            let mut tris = vec![];
            r.skin(r.anim(idle), 0.0, &mut tris);
            (r.min, r.max) = extent(&tris);
            Some(r)
        })
        .collect())
}

/// Stage props (explore.rs: moving collision): every MDL0 named in `want` from the leaves of one map archive, each
/// with every CHR0 of the archive (by full name) and its root motion kept. Call it on a helper thread.
pub fn prop_rigs(leaves: &[Vec<u8>], want: &[&str]) -> Vec<Option<Rig>> {
    let mut mdls: Vec<(String, usize, usize)> = vec![];
    let mut chrs: Vec<(String, usize, usize)> = vec![];
    for (li, b) in leaves.iter().enumerate() {
        if b.len() < 0x10 || &b[..4] != b"bres" {
            continue;
        }
        let Some(entries) = brres_entries(b) else { continue };
        for (name, off) in entries {
            let list = match b.get(off..off + 4) {
                Some(b"CHR0") => &mut chrs,
                Some(b"MDL0") => &mut mdls,
                _ => continue,
            };
            if !list.iter().any(|(n, _, _)| *n == name) {
                list.push((name, li, off));
            }
        }
    }
    want.iter()
        .map(|w| {
            let (_, li, off) = mdls.iter().find(|(n, _, _)| n == w)?;
            let mut r = mdl0(&leaves[*li], *off, &[])?;
            for (n, cl, co) in &chrs {
                r.anims.push((n.clone(), chr0(&leaves[*cl], *co)));
            }
            Some(r)
        })
        .collect()
}

// ---------------------------------------------------------------- containers

fn be16(d: &[u8], o: usize) -> u16 {
    d.get(o..o + 2).map(|s| u16::from_be_bytes([s[0], s[1]])).unwrap_or(0)
}
fn be32(d: &[u8], o: usize) -> u32 {
    d.get(o..o + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]])).unwrap_or(0)
}
fn bei(d: &[u8], o: usize) -> i64 {
    be32(d, o) as i32 as i64
}
fn bef(d: &[u8], o: usize) -> f32 {
    f32::from_bits(be32(d, o))
}
fn at(base: usize, rel: i64) -> usize {
    (base as i64 + rel).max(0) as usize
}

/// Nintendo LZ77 (types 0x10 and 0x11). Returns the input unchanged if it does not look compressed.
fn unwrap(d: Vec<u8>) -> Vec<u8> {
    if d.len() < 8 || (d[0] != 0x10 && d[0] != 0x11) {
        return d;
    }
    lz(&d).unwrap_or(d)
}

fn lz(d: &[u8]) -> Option<Vec<u8>> {
    let t = d[0];
    let mut n = d[1] as usize | (d[2] as usize) << 8 | (d[3] as usize) << 16;
    let mut p = 4;
    if n == 0 {
        n = u32::from_le_bytes(d.get(4..8)?.try_into().ok()?) as usize;
        p = 8;
    }
    if n > 64 << 20 {
        return None;
    }
    let mut o = Vec::with_capacity(n);
    while o.len() < n {
        let f = *d.get(p)?;
        p += 1;
        for b in 0..8 {
            if o.len() >= n {
                break;
            }
            if f & (0x80 >> b) != 0 {
                let (l, dist);
                if t == 0x10 {
                    let x = (*d.get(p)? as usize) << 8 | *d.get(p + 1)? as usize;
                    p += 2;
                    l = (x >> 12) + 3;
                    dist = (x & 0xfff) + 1;
                } else {
                    let b0 = *d.get(p)? as usize;
                    match b0 >> 4 {
                        0 => {
                            let (b1, b2) = (*d.get(p + 1)? as usize, *d.get(p + 2)? as usize);
                            l = ((b0 & 0xf) << 4 | b1 >> 4) + 0x11;
                            dist = ((b1 & 0xf) << 8 | b2) + 1;
                            p += 3;
                        }
                        1 => {
                            let (b1, b2, b3) = (*d.get(p + 1)? as usize, *d.get(p + 2)? as usize, *d.get(p + 3)? as usize);
                            l = ((b0 & 0xf) << 12 | b1 << 4 | b2 >> 4) + 0x111;
                            dist = ((b2 & 0xf) << 8 | b3) + 1;
                            p += 4;
                        }
                        i => {
                            l = i + 1;
                            dist = ((b0 & 0xf) << 8 | *d.get(p + 1)? as usize) + 1;
                            p += 2;
                        }
                    }
                }
                if dist > o.len() {
                    return None;
                }
                for _ in 0..l {
                    let v = o[o.len() - dist];
                    o.push(v);
                }
            } else {
                o.push(*d.get(p)?);
                p += 1;
            }
        }
    }
    o.truncate(n);
    Some(o)
}

/// Every leaf (non-ARC) file in a (nested, possibly compressed) ARC.
fn walk(d: &[u8], depth: u32, out: &mut Vec<Vec<u8>>) {
    walk_tagged(d, depth, out, &mut vec![], None);
}

/// [`walk`], and with `tags` each leaf's place in the archive tree: the (file type, file index) of every archive
/// entry from the top down to the leaf (Brawl's gfArchive looks files up by that pair).
fn walk_tagged(d: &[u8], depth: u32, out: &mut Vec<Vec<u8>>, at: &mut Vec<(u16, u16)>, mut tags: Option<&mut Vec<Vec<(u16, u16)>>>) {
    if depth > 6 || d.len() < 0x40 || &d[..4] != b"ARC\0" {
        out.push(d.to_vec());
        if let Some(t) = tags {
            t.push(at.clone());
        }
        return;
    }
    let n = be16(d, 6) as usize;
    let mut p = 0x40;
    for _ in 0..n {
        let sz = be32(d, p + 4) as usize;
        let Some(body) = d.get(p + 0x20..p + 0x20 + sz) else { return };
        at.push((be16(d, p), be16(d, p + 2)));
        walk_tagged(&unwrap(body.to_vec()), depth + 1, out, at, tags.as_deref_mut());
        at.pop();
        p = (p + 0x20 + sz + 0x1f) & !0x1f;
    }
}

fn cstr(d: &[u8], o: usize) -> String {
    let e = d.get(o..).and_then(|s| s.iter().position(|&c| c == 0)).map(|e| o + e).unwrap_or(o);
    String::from_utf8_lossy(d.get(o..e).unwrap_or(&[])).into_owned()
}

/// A BRRES index group: (name, absolute data offset).
fn group(d: &[u8], g: usize) -> Vec<(String, usize)> {
    let n = be32(d, g + 4) as usize;
    (1..=n.min(4096))
        .map(|i| {
            let e = g + 8 + 16 * i;
            (cstr(d, at(g, bei(d, e + 8))), at(g, bei(d, e + 12)))
        })
        .collect()
}

/// All second-level entries of a BRRES (every folder's items).
pub fn brres_entries(d: &[u8]) -> Option<Vec<(String, usize)>> {
    let root = be16(d, 0xC) as usize;
    if d.get(root..root + 4)? != b"root" {
        return None;
    }
    let mut v = vec![];
    for (_, f) in group(d, root + 8) {
        v.extend(group(d, f));
    }
    Some(v)
}

// ---------------------------------------------------------------- textures

/// One texture, kept in the GX layout the renderer samples (`fmt`, `levels` mip levels, padded to the size the GX
/// layer may read for an 11-level object). Formats GX decodes itself (I4..RGBA8, CMPR) keep the file's own bytes,
/// mips included; palette formats (CI4/CI8/CI14 + PLT0) are expanded here to RGBA8, one level.
/// Made once on the loader's helper thread; the draw only points GX at it.
pub struct Tex {
    w: u16,
    h: u16,
    fmt: u32,
    levels: u8,
    data: Vec<u8>,
    /// storage whose address names this texture's GX texture object
    obj: Vec<u8>,
}

/// Block size of a GX texture format: (block width, block height, bytes per block).
fn gx_block(fmt: u32) -> Option<(usize, usize, usize)> {
    Some(match fmt {
        0 | 8 => (8, 8, 32),
        1 | 2 | 9 => (8, 4, 32),
        3 | 4 | 5 | 10 => (4, 4, 32),
        6 => (4, 4, 64),
        14 => (8, 8, 32),
        _ => return None,
    })
}

/// Bytes of `levels` mip levels of a w x h texture.
fn gx_size(w: usize, h: usize, fmt: u32, levels: usize) -> usize {
    let Some((bw, bh, bb)) = gx_block(fmt) else { return 0 };
    let (mut w, mut h, mut n) = (w, h, 0);
    for _ in 0..levels {
        n += w.div_ceil(bw) * h.div_ceil(bh) * bb;
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    n
}

fn rgb565(c: u16) -> [u8; 4] {
    let r = ((c >> 11) & 31) as u8;
    let g = ((c >> 5) & 63) as u8;
    let b = (c & 31) as u8;
    [r << 3 | r >> 2, g << 2 | g >> 4, b << 3 | b >> 2, 255]
}

fn rgb5a3(c: u16) -> [u8; 4] {
    if c & 0x8000 != 0 {
        let f = |s: u16| ((s & 31) as u8) << 3 | ((s & 31) as u8) >> 2;
        [f(c >> 10), f(c >> 5), f(c), 255]
    } else {
        let f = |s: u16| ((s & 15) as u8) * 17;
        let a = ((c >> 12) & 7) as u8;
        [f(c >> 8), f(c >> 4), f(c), a << 5 | a << 2 | a >> 1]
    }
}

/// A TEX0 (and, for a palette format, its PLT0 at `plt`) as a GX texture.
fn tex0(d: &[u8], o: usize, plt: Option<(&[u8], usize)>) -> Option<Tex> {
    let (w, h, fmt) = (be16(d, o + 0x1C) as usize, be16(d, o + 0x1E) as usize, be32(d, o + 0x20));
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return None;
    }
    let data = at(o, bei(d, o + 0x10));
    let (bw, bh, bb) = gx_block(fmt)?;
    let obj = vec![0u8; core::mem::size_of::<melee_types::_GXTexObj>().max(32)];
    if !matches!(fmt, 8..=10) {
        let levels = (be32(d, o + 0x24).clamp(1, 11)) as usize;
        let mut v = d.get(data..data + gx_size(w, h, fmt, levels))?.to_vec();
        v.resize(gx_size(w, h, fmt, 11) + 64, 0);
        return Some(Tex { w: w as u16, h: h as u16, fmt, levels: levels as u8, data: v, obj });
    }
    // palette formats: index -> PLT0 colour, re-encoded as RGBA8 tiles
    let (p, po) = plt?;
    let (pfmt, pn) = (be32(p, po + 0x18), be16(p, po + 0x1C) as usize);
    let pd = at(po, bei(p, po + 0x10));
    let pal = |i: usize| -> [u8; 4] {
        if i >= pn {
            return [0; 4];
        }
        let c = be16(p, pd + 2 * i);
        match pfmt {
            0 => {
                let (a, l) = ((c >> 8) as u8, c as u8);
                [l, l, l, a]
            }
            1 => rgb565(c),
            _ => rgb5a3(c),
        }
    };
    let mut px = vec![[0u8; 4]; w * h];
    let (bx, by) = (w.div_ceil(bw), h.div_ceil(bh));
    for ty in 0..by {
        for tx in 0..bx {
            let b = data + (ty * bx + tx) * bb;
            d.get(b..b + bb)?;
            for k in 0..bw * bh {
                let i = match fmt {
                    8 => ((d[b + k / 2] >> if k % 2 == 0 { 4 } else { 0 }) & 15) as usize,
                    9 => d[b + k] as usize,
                    _ => (be16(d, b + 2 * k) & 0x3FFF) as usize,
                };
                let (x, y) = (tx * bw + k % bw, ty * bh + k / bw);
                if x < w && y < h {
                    px[y * w + x] = pal(i);
                }
            }
        }
    }
    let mut v = vec![0u8; gx_size(w, h, 6, 11) + 64];
    let bx = w.div_ceil(4);
    for ty in 0..h.div_ceil(4) {
        for tx in 0..bx {
            let b = (ty * bx + tx) * 64;
            for k in 0..16 {
                let (x, y) = (tx * 4 + k % 4, ty * 4 + k / 4);
                let c = if x < w && y < h { px[y * w + x] } else { [0; 4] };
                v[b + 2 * k] = c[3];
                v[b + 2 * k + 1] = c[0];
                v[b + 32 + 2 * k] = c[1];
                v[b + 32 + 2 * k + 1] = c[2];
            }
        }
    }
    Some(Tex { w: w as u16, h: h as u16, fmt: 6, levels: 1, data: v, obj })
}

// ---------------------------------------------------------------- MDL0 (v9) arrays

struct Arr {
    data: usize,
    comp: u32,
    div: u32,
    stride: usize,
    count: usize,
}

fn arr(d: &[u8], o: usize) -> Arr {
    Arr { data: at(o, bei(d, o + 8)), comp: be32(d, o + 0x18), div: d[o + 0x1C] as u32, stride: d[o + 0x1D] as usize, count: be16(d, o + 0x1E) as usize }
}

fn comp_read(d: &[u8], a: &Arr, i: usize, n: usize) -> [f32; 3] {
    let mut r = [0.0; 3];
    if i >= a.count {
        return r;
    }
    let base = a.data + i * a.stride;
    let sc = 1.0 / (1u32 << a.div.min(31)) as f32;
    for (k, rk) in r.iter_mut().enumerate().take(n) {
        *rk = match a.comp {
            0 => d.get(base + k).copied().unwrap_or(0) as f32 * sc,
            1 => d.get(base + k).copied().unwrap_or(0) as i8 as f32 * sc,
            2 => be16(d, base + 2 * k) as f32 * sc,
            3 => be16(d, base + 2 * k) as i16 as f32 * sc,
            _ => bef(d, base + 4 * k),
        };
    }
    r
}

fn color_read(d: &[u8], o: usize, i: usize) -> [u8; 4] {
    let data = at(o, bei(d, o + 8));
    let fmt = be32(d, o + 0x18);
    let stride = d[o + 0x1C] as usize;
    if i >= be16(d, o + 0x1E) as usize {
        return [255; 4];
    }
    let b = data + i * stride;
    match fmt {
        0 => rgb565(be16(d, b)),
        1 | 2 => [d[b], d[b + 1], d[b + 2], 255],
        3 => {
            let c = be16(d, b);
            let f = |s: u16| ((s & 15) as u8) * 17;
            [f(c >> 12), f(c >> 8), f(c >> 4), f(c)]
        }
        4 => {
            let c = (d[b] as u32) << 16 | (d[b + 1] as u32) << 8 | d[b + 2] as u32;
            let f = |s: u32| ((s & 63) as u8) << 2 | ((s & 63) as u8) >> 4;
            [f(c >> 18), f(c >> 12), f(c >> 6), f(c)]
        }
        _ => [d[b], d[b + 1], d[b + 2], d[b + 3]],
    }
}

type M34 = [[f32; 4]; 3];
const IDENT: M34 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];

fn mul(a: &M34, b: &M34) -> M34 {
    let mut r = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..4 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + if j == 3 { a[i][3] } else { 0.0 };
        }
    }
    r
}

/// T * Rz * Ry * Rx * S (rotation in degrees), the NW4R bone order.
fn srt(sc: [f32; 3], rot: [f32; 3], tr: [f32; 3]) -> M34 {
    let (sx, cx) = rot[0].to_radians().sin_cos();
    let (sy, cy) = rot[1].to_radians().sin_cos();
    let (sz, cz) = rot[2].to_radians().sin_cos();
    let r = [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
        [-sy, sx * cy, cx * cy],
    ];
    let mut m = [[0.0; 4]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = r[i][j] * sc[j];
        }
        m[i][3] = tr[i];
    }
    m
}

fn xf(m: &M34, p: [f32; 3], w: f32) -> [f32; 3] {
    let f = |i: usize| m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2] + m[i][3] * w;
    [f(0), f(1), f(2)]
}

// ---------------------------------------------------------------- CHR0

/// One animated channel.
pub enum Track {
    Fixed(f32),
    /// Hermite keys: (frame, value, tangent).
    Keys(Vec<[f32; 3]>),
    /// One value per frame (linear between them).
    Baked(Vec<f32>),
}

impl Track {
    fn at(&self, f: f32) -> f32 {
        match self {
            Track::Fixed(v) => *v,
            Track::Baked(v) => {
                if v.is_empty() {
                    return 0.0;
                }
                let i = (f.max(0.0) as usize).min(v.len() - 1);
                let j = (i + 1).min(v.len() - 1);
                let t = (f - i as f32).clamp(0.0, 1.0);
                v[i] + (v[j] - v[i]) * t
            }
            Track::Keys(k) => {
                if k.is_empty() {
                    return 0.0;
                }
                if f <= k[0][0] {
                    return k[0][1];
                }
                for w in k.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if f <= b[0] {
                        let span = b[0] - a[0];
                        if span <= 0.0 {
                            return b[1];
                        }
                        let t = (f - a[0]) / span;
                        let (t2, t3) = (t * t, t * t * t);
                        return (2.0 * t3 - 3.0 * t2 + 1.0) * a[1]
                            + (t3 - 2.0 * t2 + t) * span * a[2]
                            + (-2.0 * t3 + 3.0 * t2) * b[1]
                            + (t3 - t2) * span * b[2];
                    }
                }
                k[k.len() - 1][1]
            }
        }
    }
}

/// A CHR0: its length and, per bone name, the nine channels (scale xyz, rotation xyz degrees, translation xyz).
pub struct Anim {
    pub frames: f32,
    pub tracks: Vec<(String, [Option<Track>; 9])>,
}

impl Anim {
    /// The (min, max) of `bone`'s translation per axis over the clip, or None when it has no translation channel.
    pub fn trans_range(&self, bone: &str) -> Option<[(f32, f32); 3]> {
        let (_, v) = self.tracks.iter().find(|(n, _)| n == bone)?;
        let mut r = [(0.0f32, 0.0f32); 3];
        let mut any = false;
        for a in 0..3 {
            if let Some(t) = &v[6 + a] {
                any = true;
                let (mut lo, mut hi) = (f32::MAX, f32::MIN);
                let mut f = 0.0;
                while f <= self.frames {
                    let x = t.at(f);
                    lo = lo.min(x);
                    hi = hi.max(x);
                    f += 1.0;
                }
                r[a] = (lo, hi);
            }
        }
        any.then_some(r)
    }
    /// `bone`'s rotation channels (degrees) at `frame` (None: no such bone track; an axis without a channel reads 0).
    pub fn rot_at(&self, bone: &str, frame: f32) -> Option<[f32; 3]> {
        let (_, v) = self.tracks.iter().find(|(n, _)| n == bone)?;
        Some(core::array::from_fn(|a| v[3 + a].as_ref().map_or(0.0, |t| t.at(frame))))
    }
    /// `bone`'s translation at `frame` (None: no such bone track; an axis without a channel reads 0).
    pub fn trans_at(&self, bone: &str, frame: f32) -> Option<[f32; 3]> {
        let (_, v) = self.tracks.iter().find(|(n, _)| n == bone)?;
        let mut r = [0.0f32; 3];
        for a in 0..3 {
            if let Some(t) = &v[6 + a] {
                r[a] = t.at(frame);
            }
        }
        Some(r)
    }
    /// Root-motion strip (the halberd `transn_policy` 'strip' rule): `bone`'s translation on the chosen axes holds
    /// its frame-0 value for the whole clip, for a body whose travel the game moves itself (Subspace enemies).
    pub fn hold_translation(&mut self, bone: &str, axes: [bool; 3]) {
        if let Some((_, v)) = self.tracks.iter_mut().find(|(n, _)| n == bone) {
            for a in 0..3 {
                if axes[a] {
                    if let Some(t) = &v[6 + a] {
                        let x0 = t.at(0.0);
                        v[6 + a] = Some(Track::Fixed(x0));
                    }
                }
            }
        }
    }
}

fn chr0(d: &[u8], c: usize) -> Anim {
    let ver = be32(d, c + 8);
    let frames = be16(d, c + if ver >= 5 { 0x20 } else { 0x1C }) as usize;
    let data = at(c, bei(d, c + 0x10));
    let mut tracks = vec![];
    for (name, e) in group(d, data) {
        let code = be32(d, e + 4);
        let mut v: [Option<Track>; 9] = Default::default();
        let mut q = e + 8;
        // one channel: a fixed value, or an offset (from the entry) to keyframes in the given format
        let track = |q: usize, fixed: bool, fmt: u32| -> Track {
            if fixed {
                return Track::Fixed(bef(d, q));
            }
            let k = at(e, bei(d, q));
            match fmt {
                1..=3 => {
                    let n = be16(d, k) as usize;
                    let (step, base) = (bef(d, k + 8), bef(d, k + 12));
                    let mut keys = Vec::with_capacity(n.min(4096));
                    for i in 0..n.min(4096) {
                        keys.push(match fmt {
                            1 => {
                                let w = be32(d, k + 16 + 4 * i);
                                let tan = (((w & 0xFFF) as i32) << 20 >> 20) as f32 / 32.0;
                                [(w >> 24) as f32, base + step * ((w >> 12) & 0xFFF) as f32, tan]
                            }
                            2 => {
                                let o = k + 16 + 6 * i;
                                [be16(d, o) as f32 / 32.0, base + step * be16(d, o + 2) as f32, be16(d, o + 4) as i16 as f32 / 256.0]
                            }
                            _ => {
                                // the float-key header is 8 bytes (count, pad, frame scale): no step / base
                                let o = k + 8 + 12 * i;
                                [bef(d, o), bef(d, o + 4), bef(d, o + 8)]
                            }
                        });
                    }
                    Track::Keys(keys)
                }
                // baked rotations: one angle per frame, a byte or a u16 over the full turn (no header)
                4 | 5 => {
                    let (step, base) = (bef(d, k), bef(d, k + 4));
                    Track::Baked(
                        (0..=frames)
                            .map(|i| {
                                let x = if fmt == 4 { d.get(k + 8 + i).copied().unwrap_or(0) as f32 } else { be16(d, k + 8 + 2 * i) as f32 };
                                base + step * x
                            })
                            .collect(),
                    )
                }
                _ => Track::Baked((0..=frames).map(|i| bef(d, k + 4 * i)).collect()),
            }
        };
        if code & 2 == 0 {
            for (part, exists_bit, iso_bit, fixed_sh, fmt_sh, fmt_mask) in
                [(0usize, 22, 4, 13, 25, 3u32), (1, 23, 5, 16, 27, 7), (2, 24, 6, 19, 30, 3)]
            {
                if code & (1 << exists_bit) == 0 {
                    continue;
                }
                let fmt = (code >> fmt_sh) & fmt_mask;
                if code & (1 << iso_bit) != 0 {
                    let fixed = code & (1 << fixed_sh) != 0;
                    for a in 0..3 {
                        v[part * 3 + a] = Some(track(q, fixed, fmt));
                    }
                    q += 4;
                } else {
                    for a in 0..3 {
                        v[part * 3 + a] = Some(track(q, code & (1 << (fixed_sh + a)) != 0, fmt));
                        q += 4;
                    }
                }
            }
        }
        // flags that fix a part without data: identity, scale one, rotation + translation zero
        for a in 0..3 {
            if code & 2 != 0 || code & 8 != 0 {
                v[a].get_or_insert(Track::Fixed(1.0));
            }
            if code & 2 != 0 || code & 4 != 0 {
                v[3 + a].get_or_insert(Track::Fixed(0.0));
                v[6 + a].get_or_insert(Track::Fixed(0.0));
            }
        }
        tracks.push((name, v));
    }
    Anim { frames: frames.max(1) as f32, tracks }
}

// ---------------------------------------------------------------- VIS0

/// One bone's visibility channel: a fixed value, or one bit per frame (most significant bit first).
enum VisTrack {
    Fixed(bool),
    Bits(Vec<u32>),
}

/// A VIS0: per bone name, whether that bone shows at a frame. Brawl binds it with the motion's CHR0 and gives both
/// the same frame (the motion file's anim set: sora_melee 0x8101FA94 builds it, 0x81020084 sets every member's
/// frame); a draw-list entry whose visibility bone is off is not drawn.
pub struct Vis {
    pub frames: u32,
    pub looped: bool,
    tracks: Vec<(String, VisTrack)>,
}

impl Vis {
    /// `bone`'s visibility at `frame` (None: the VIS0 has no channel for it). The frame follows the anim object's
    /// play policy: wrapped when the VIS0 loops, held at its last frame otherwise.
    pub fn at(&self, bone: &str, frame: f32) -> Option<bool> {
        let (_, t) = self.tracks.iter().find(|(n, _)| n == bone)?;
        Some(match t {
            VisTrack::Fixed(v) => *v,
            VisTrack::Bits(w) => {
                let n = self.frames.max(1);
                let f = frame.max(0.0) as u32;
                let f = if self.looped { f % n } else { f.min(n - 1) };
                w.get((f / 32) as usize).is_some_and(|x| x & (0x8000_0000 >> (f % 32)) != 0)
            }
        })
    }
}

fn vis0(d: &[u8], v: usize) -> Vis {
    let ver = be32(d, v + 8);
    // v4 adds a user-data offset before the name: the fields after it move by 4
    let h = if ver >= 4 { 4 } else { 0 };
    let frames = be16(d, v + 0x1C + h) as u32;
    let looped = be32(d, v + 0x20 + h) != 0;
    let data = at(v, bei(d, v + 0x10));
    let words = frames.div_ceil(32).clamp(1, 4096) as usize;
    let mut tracks = vec![];
    for (name, e) in group(d, data) {
        let flags = be32(d, e + 4);
        // flags: bit 1 = a fixed value (bit 0); else the frame bits follow the entry header
        let t = if flags & 2 != 0 { VisTrack::Fixed(flags & 1 != 0) } else { VisTrack::Bits((0..words).map(|i| be32(d, e + 8 + 4 * i)).collect()) };
        tracks.push((name, t));
    }
    Vis { frames, looped, tracks }
}

// ---------------------------------------------------------------- MDL0 -> skinned rig

pub struct Bone {
    pub name: String,
    /// the bone's own index (MDL0 bone header [off]), what param files refer to
    index: u32,
    srt: [[f32; 3]; 3],
    pub parent: Option<usize>,
    inv: M34,
    /// the bone's default visibility (its MDL0 flags' visible bit): what shows without a VIS0 channel
    visible: bool,
}

/// Where a vertex's matrix comes from.
enum Src {
    Ident,
    /// stored in that bone's space: the bone's world
    Bone(usize),
    /// weights x (world x inverse bind)
    Mix(Vec<(usize, f32)>),
}

/// One body part ready to pose: bones, matrix sources and a triangle list in source space.
/// Scratch for drawing a rig (see [`Rig::draw_bufs`]).
pub struct DrawBufs {
    world: Vec<M34>,
    mats: Vec<M34>,
    pub tris: Vec<Vert>,
    /// per batch: hidden by its visibility bone at the skinned frame (filled by [`Rig::skin_vis`]; empty: all shown)
    pub hide: Vec<bool>,
}

/// A posed vertex: model-space position, lit colour (the rasterised colour TEV sees), texture coordinate.
/// (position, colour, texture coordinate, normal (model space; zero: none))
pub type Vert = ([f32; 3], [u8; 4], [f32; 2], [f32; 3]);

/// A run of triangles drawn with one material. Brawl material state honoured (what Petey and his cages use; a
/// future boss extends this): texture 0 with its wrap modes, the alpha compare from the material's pixel display
/// list (cutouts), the light channel's ambient colour. Every Petey material's TEV reduces to
/// `2 x texture x rasterised colour` (stage 1 passes, stage 2 lerps to C2 by a zero konst), drawn that way.
pub struct Batch {
    /// one past the last vertex of the run (in `Rig::verts` and in a skinned triangle list alike)
    end: u32,
    tex: Option<std::sync::Arc<Tex>>,
    wrap: [u32; 2],
    /// GXSetAlphaCompare (comp0, ref0, op, comp1, ref1)
    alpha: [u8; 5],
    /// light channel 0 ambient (grey level) and whether the channel is lit
    amb: f32,
    lit: bool,
    /// BP 0x41 blend (src factor, dst factor) when the material blends (shadow decals, light shafts); None: opaque
    blend: Option<(u8, u8)>,
    /// the draw-list entry's visibility bone (a bone slot): the object is drawn only while that bone is visible
    vis_bone: Option<usize>,
    /// the material's multi-stage shading (None: no shader read; the single-stage look)
    sh: Option<Box<Shade>>,
    /// light channel 0's material colour when its control takes it from the register, not the vertex (the shaded
    /// look's rasterised colour starts from it: Brawl's TEV stages scale it up again, x2 / x4)
    mat: Option<[u8; 4]>,
}

/// A material's full pixel shading, read from its MDL0 shader and material display lists (the BP registers the Wii
/// would receive) and its texture references: every TEV stage's order, colour / alpha combiners, swap and konstant
/// selections, the TEV colour and konstant registers, the swap tables, every texture map, and per texture
/// coordinate its source (the vertex UV, or the environment map from the normal) and its SRT matrix (Maya mode).
/// Drawn through the GX TEV calls the Melee renderer already takes ([`draw_tris_vis`]); the texture coordinates are
/// made per vertex on the CPU and sent direct.
pub struct Shade {
    stages: u8,
    /// per stage: (texture coordinate, texture map (255: none), GX colour channel)
    order: [[u8; 3]; 16],
    /// per stage: the colour and alpha combiner BP values (hardware layout)
    cenv: [u32; 16],
    aenv: [u32; 16],
    /// per stage: konstant colour / alpha selection
    kc: [u8; 16],
    ka: [u8; 16],
    /// swap tables: (r, g, b, a) channel per table
    swap: [[u8; 4]; 4],
    /// TEV colour registers PREV, REG0-2 (signed 11-bit) and konstant colours K0-K3, each with its written flag
    creg: [([i16; 4], bool); 4],
    kreg: [([u8; 4], bool); 4],
    /// (texture map, texture, wrap s / t)
    maps: Vec<(u8, std::sync::Arc<Tex>, [u32; 2])>,
    /// per texture coordinate
    gens: Vec<Gen>,
}

#[derive(Clone, Copy)]
struct Gen {
    /// from the normal (environment map) instead of the UV
    env: bool,
    /// u' = m0 u + m1 v + m2, v' = m3 u + m4 v + m5
    m: [f32; 6],
    ident: bool,
}

/// BP register file of a display-list run: BP writes under the BP mask register, XF writes kept by address.
struct Regs {
    bp: [u32; 256],
    wr: [bool; 256],
    xf: Vec<(u16, u32)>,
}

impl Regs {
    fn new() -> Self {
        Regs { bp: [0; 256], wr: [false; 256], xf: vec![] }
    }
    /// Run the GX commands in `d[p..end]` (NOP, BP, XF, CP); stops at anything else.
    fn run(&mut self, d: &[u8], mut p: usize, end: usize) {
        let end = end.min(d.len());
        let mut mask = 16777215u32;
        while p < end {
            match d[p] {
                0 => p += 1,
                97 if p + 5 <= end => {
                    let v = be32(d, p + 1);
                    let (r, val) = ((v >> 24) as usize, v & 16777215);
                    if r == 254 {
                        mask = val;
                    } else {
                        self.bp[r] = (self.bp[r] & !mask) | (val & mask);
                        self.wr[r] = true;
                        mask = 16777215;
                    }
                    p += 5;
                }
                16 if p + 5 <= end => {
                    let n = be16(d, p + 1) as usize + 1;
                    let a = be16(d, p + 3);
                    for k in 0..n {
                        if p + 5 + 4 * k + 4 <= end {
                            self.xf.push((a.wrapping_add(k as u16), be32(d, p + 5 + 4 * k)));
                        }
                    }
                    p += 5 + 4 * n;
                }
                8 => p += 6,
                _ => break,
            }
        }
    }
}

/// The Maya-mode texture matrix of an SRT (BrawlLib's MayaMtxSRT): (u, v) -> (u', v').
fn maya(s: [f32; 2], r_deg: f32, t: [f32; 2]) -> [f32; 6] {
    let (sn, cs) = (r_deg.to_radians().sin(), r_deg.to_radians().cos());
    let (sin, cos) = (0.5 * sn - 0.5, -0.5 * cs);
    [s[0] * cs, s[0] * sn, s[0] * (cos - sin - t[0]), -s[1] * sn, s[1] * cs, s[1] * (cos + sin + t[1]) + 1.0]
}

/// The shading of the v9 MDL0 material at `mo` (None: no shader, or a texture map a stage samples is missing).
fn shade_of(d: &[u8], mo: usize, texs: &[(String, std::sync::Arc<Tex>)]) -> Option<Box<Shade>> {
    let so = bei(d, mo + 40);
    if so == 0 {
        return None;
    }
    let sh = at(mo, so);
    let stages = d.get(mo + 22).copied()?.clamp(1, 16);
    let ngen = d.get(mo + 20).copied()?.min(8) as usize;
    let mut r = Regs::new();
    r.run(d, sh + 32, sh + be32(d, sh) as usize);
    // the material's own lists (pixel, TEV colours, konstants, indirect matrices, texture coordinate generation)
    let dl = at(mo, bei(d, mo + 56));
    r.run(d, dl, mo + be32(d, mo) as usize);
    let mut s = Shade {
        stages,
        order: [[255, 255, 6]; 16],
        cenv: [0; 16],
        aenv: [0; 16],
        kc: [0; 16],
        ka: [0; 16],
        swap: [[0, 1, 2, 3]; 4],
        creg: [([0; 4], false); 4],
        kreg: [([0; 4], false); 4],
        maps: vec![],
        gens: vec![],
    };
    for t in 0..4 {
        let (a, b) = (r.bp[246 + 2 * t], r.bp[247 + 2 * t]);
        s.swap[t] = [(a & 3) as u8, ((a >> 2) & 3) as u8, (b & 3) as u8, ((b >> 2) & 3) as u8];
    }
    for st in 0..stages as usize {
        let tref = r.bp[40 + st / 2] >> (12 * (st & 1));
        let chan = match (tref >> 7) & 7 {
            0 => 4, // GX_COLOR0A0
            _ => 6, // GX_COLOR_ZERO: channel 1 (Brawl's second light set, specular) is not sent [L]
        };
        s.order[st] = if tref & 64 != 0 { [((tref >> 3) & 7) as u8, (tref & 7) as u8, chan] } else { [255, 255, chan] };
        s.cenv[st] = r.bp[192 + 2 * st];
        s.aenv[st] = r.bp[193 + 2 * st];
        let ks = r.bp[246 + st / 2] >> (10 * (st & 1));
        s.kc[st] = ((ks >> 4) & 31) as u8;
        s.ka[st] = ((ks >> 9) & 31) as u8;
    }
    let s11 = |v: u32| (((v & 2047) as i32) << 21 >> 21) as i16;
    for k in 0..4 {
        let (lo, hi) = (r.bp[224 + 2 * k], r.bp[225 + 2 * k]);
        if !(r.wr[224 + 2 * k] || r.wr[225 + 2 * k]) {
            continue;
        }
        let (rr, a, b, g) = (lo & 2047, (lo >> 12) & 2047, hi & 2047, (hi >> 12) & 2047);
        if lo & 8388608 != 0 {
            s.kreg[k] = ([rr as u8, g as u8, b as u8, a as u8], true);
        } else {
            s.creg[k] = ([s11(rr), s11(g), s11(b), s11(a)], true);
        }
    }
    // texture references: name, texture map id, wrap; their SRTs and mapping methods
    let nref = be32(d, mo + 44) as usize;
    let e0 = at(mo, bei(d, mo + 48));
    let mref = be32(d, mo + 48);
    let srt = mo + 416 + if mref == 0 { 4 } else { (mref & 15) as usize };
    for k in 0..nref.min(8) {
        let e = e0 + 52 * k;
        let name = cstr(d, at(e, bei(d, e)));
        let map = be32(d, e + 16).min(7) as u8;
        let t = texs.iter().find(|(n, _)| *n == name)?.1.clone();
        s.maps.push((map, t, [be32(d, e + 24).min(2), be32(d, e + 28).min(2)]));
    }
    for g in 0..ngen {
        let x = r.xf.iter().rev().find(|(a, _)| *a == 4160 + g as u16).map(|(_, v)| *v).unwrap_or(5 << 7);
        let row = (x >> 7) & 31;
        let base = srt + 8 + 20 * g;
        let (sc, rot, tr) = ([bef(d, base), bef(d, base + 4)], bef(d, base + 8), [bef(d, base + 12), bef(d, base + 16)]);
        let ident = sc == [1.0, 1.0] && rot == 0.0 && tr == [0.0, 0.0];
        let mode = d.get(srt + 168 + 52 * g + 2).copied().unwrap_or(0);
        s.gens.push(Gen { env: row == 1 || matches!(mode, 1 | 3 | 4), m: maya(sc, rot, tr), ident });
    }
    // every map a stage samples must be loaded
    for st in 0..stages as usize {
        let m = s.order[st][1];
        if m != 255 && !s.maps.iter().any(|(mm, _, _)| *mm == m) {
            return None;
        }
    }
    Some(Box::new(s))
}

/// Texture coordinate `g` of a vertex: its UV or its environment-map coordinate (from the view-space normal `nv`),
/// through the coordinate's SRT matrix.
fn gen_uv(g: &Gen, uv: [f32; 2], nv: [f32; 3]) -> [f32; 2] {
    let (u, v) = if g.env {
        let l = (nv[0] * nv[0] + nv[1] * nv[1] + nv[2] * nv[2]).sqrt().max(1e-6);
        (0.5 + 0.5 * nv[0] / l, 0.5 - 0.5 * nv[1] / l)
    } else {
        (uv[0], uv[1])
    };
    if g.ident {
        [u, v]
    } else {
        let m = &g.m;
        [m[0] * u + m[1] * v + m[2], m[3] * u + m[4] * v + m[5]]
    }
}

/// `MELEE_BRAWL_ONE_STAGE`: draw every material with the old single-stage look (before / after shots).
fn one_stage() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("MELEE_BRAWL_ONE_STAGE").is_some())
}

/// Set GX up for a shaded batch: texture maps, texture coordinate generators (direct, one per coordinate), and
/// every TEV stage. The caller sends per vertex the position, colour 0 and the coordinates.
unsafe fn set_shade(s: &Shade) {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_runtime::tu::lbrefract::ext as gi;
    use melee_runtime::tu::sobjlib::ext as gs;
    use melee_runtime::tu::tobj::ext as gt;
    use melee_types::*;
    for (map, t, wrap) in &s.maps {
        let o = t.obj.as_ptr() as *mut _GXTexObj;
        gs::GXInitTexObj(o, t.data.as_ptr() as *mut _, t.w, t.h, t.fmt, wrap[0], wrap[1], (t.levels > 1) as u8);
        let min = if t.levels > 1 { GX_LIN_MIP_LIN } else { GX_LINEAR };
        gt::GXInitTexObjLOD(o, min as _, GX_LINEAR as _, 0.0, (t.levels - 1) as f32, 0.0, 0, 0, GX_ANISO_1 as _);
        gs::GXLoadTexObj(o, GX_TEXMAP0 as u32 + *map as u32);
    }
    gx::GXSetNumTexGens(s.gens.len() as u8);
    for i in 0..8u32 {
        if (i as usize) < s.gens.len() {
            gs::GXSetTexCoordGen2(GX_TEXCOORD0 as u32 + i, GX_TG_MTX2x4 as _, GX_TG_TEX0 as u32 + i, GX_IDENTITY as _, 0, GX_PTIDENTITY as _);
            gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_TEX0 as u32 + i, GX_TEX_ST as _, GX_F32 as _, 0);
            gx::GXSetVtxDesc(GX_VA_TEX0 as u32 + i, GX_DIRECT as _);
        } else {
            gx::GXSetVtxDesc(GX_VA_TEX0 as u32 + i, GX_NONE as _);
        }
    }
    gx::GXSetNumTevStages(s.stages);
    for st in 0..s.stages as usize {
        let n = st as u32;
        gi::GXSetTevDirect(n);
        let [c, m, ch] = s.order[st];
        let (c, m) = if m == 255 { (GX_TEXCOORD_NULL as u32, GX_TEXMAP_NULL as u32) } else { (c as u32, m as u32) };
        gx::GXSetTevOrder(n, c, m, ch as u32);
        let ce = s.cenv[st];
        gs::GXSetTevColorIn(n, (ce >> 12) & 15, (ce >> 8) & 15, (ce >> 4) & 15, ce & 15);
        let (bias, sub, clamp, scale, dest) = ((ce >> 16) & 3, (ce >> 18) & 1, ((ce >> 19) & 1) as u8, (ce >> 20) & 3, (ce >> 22) & 3);
        if bias == 3 {
            gs::GXSetTevColorOp(n, 8 + 2 * scale + sub, 0, 0, clamp, dest);
        } else {
            gs::GXSetTevColorOp(n, sub, bias, scale, clamp, dest);
        }
        let ae = s.aenv[st];
        gs::GXSetTevAlphaIn(n, (ae >> 13) & 7, (ae >> 10) & 7, (ae >> 7) & 7, (ae >> 4) & 7);
        let (bias, sub, clamp, scale, dest) = ((ae >> 16) & 3, (ae >> 18) & 1, ((ae >> 19) & 1) as u8, (ae >> 20) & 3, (ae >> 22) & 3);
        if bias == 3 {
            gs::GXSetTevAlphaOp(n, 8 + 2 * scale + sub, 0, 0, clamp, dest);
        } else {
            gs::GXSetTevAlphaOp(n, sub, bias, scale, clamp, dest);
        }
        gs::GXSetTevSwapMode(n, ae & 3, (ae >> 2) & 3);
        gs::GXSetTevKColorSel(n, s.kc[st] as u32);
        gs::GXSetTevKAlphaSel(n, s.ka[st] as u32);
    }
    for (t, w) in s.swap.iter().enumerate() {
        gs::GXSetTevSwapModeTable(t as u32, w[0] as u32, w[1] as u32, w[2] as u32, w[3] as u32);
    }
    for (k, (c, on)) in s.creg.iter().enumerate() {
        if *on {
            gs::GXSetTevColorS10(k as u32, _GXColorS10 { r: c[0], g: c[1], b: c[2], a: c[3] });
        }
    }
    for (k, (c, on)) in s.kreg.iter().enumerate() {
        if *on {
            gs::GXSetTevKColor(k as u32, _GXColor { r: c[0], g: c[1], b: c[2], a: c[3] });
        }
    }
}

/// Back to the single-stage state the plain batches and the Melee draws after ours expect.
unsafe fn reset_shade() {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_runtime::tu::sobjlib::ext as gs;
    use melee_types::*;
    gx::GXSetNumTevStages(1);
    gs::GXSetTevSwapMode(GX_TEVSTAGE0 as _, GX_TEV_SWAP0 as _, GX_TEV_SWAP0 as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP0 as _, GX_CH_RED as _, GX_CH_GREEN as _, GX_CH_BLUE as _, GX_CH_ALPHA as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP1 as _, GX_CH_RED as _, GX_CH_RED as _, GX_CH_RED as _, GX_CH_ALPHA as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP2 as _, GX_CH_GREEN as _, GX_CH_GREEN as _, GX_CH_GREEN as _, GX_CH_ALPHA as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP3 as _, GX_CH_BLUE as _, GX_CH_BLUE as _, GX_CH_BLUE as _, GX_CH_ALPHA as _);
    for i in 1..8u32 {
        gx::GXSetVtxDesc(GX_VA_TEX0 as u32 + i, GX_NONE as _);
    }
}

pub struct Rig {
    pub bones: Vec<Bone>,
    srcs: Vec<Src>,
    /// (source, position, normal (zero: none), vertex colour, texture coordinate)
    verts: Vec<(u16, [f32; 3], [f32; 3], [u8; 4], [f32; 2])>,
    batches: Vec<Batch>,
    /// animations by suffix (the CHR0 name after the model name: `Wait1`, `BasketAttackL`, ...)
    pub anims: Vec<(String, Anim)>,
    /// visibility animations by suffix, like `anims`
    pub vis: Vec<(String, Vis)>,
    /// the animation-only BRRES files in archive order (one per motion, in Brawl's motion-list order): their CHR0
    /// and VIS0 suffixes
    pub motion_files: Vec<(Option<String>, Option<String>)>,
    /// Extent in the first frame of `Wait1` (size and depth centre stay fixed while animating; the ground is y 0).
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// What [`Rig::strip_root_motion`] removed (one line per clip with travel), for the load log.
    pub motion_log: String,
}

impl Rig {
    pub fn anim(&self, name: &str) -> Option<&Anim> {
        self.anims.iter().find(|(n, _)| n == name).map(|(_, a)| a)
    }

    /// Bone slot by its MDL0 index (param files name bones by index).
    pub fn bone_by_index(&self, i: u32) -> Option<usize> {
        self.bones.iter().position(|b| b.index == i)
    }
    /// Bone slot whose name ends with `suffix` (arm bones carry the model name as a prefix).
    pub fn bone_named(&self, suffix: &str) -> Option<usize> {
        self.bones.iter().position(|b| b.name.ends_with(suffix))
    }
    /// Brawl clips carry the body's travel on the top bones' translation (TopN / TransN): Brawl's own code consumes it
    /// as movement. A game that moves the body itself (the Subspace enemies' sim velocities) draws that travel twice:
    /// the body slides off its collision position through a walk / charge / turn and snaps back when the clip loops.
    /// Same rule as the other ports (`ports/halberd/anim/transn_policy.json` 'strip', `ports/ir/tools/
    /// convert_ultimate_anim.py strip_root`): the horizontal translation (x, z) of TopN / TransN (within two levels of
    /// the root) holds its frame-0 value for the whole clip. y stays (hops and bobs are the look; the game's y is the
    /// ground). Applied by [`load_rigs`] to every rig it loads; Petey's own [`load`] does not (his hit / land points
    /// are read from the posed bones, so his clip travel is gameplay there).
    pub fn strip_root_motion(&mut self) {
        // the travel carriers only (TopN, TransN, near the root): RotN / XRotN translations are body sway (leans,
        // wind-ups) and stay part of the look
        let tops: Vec<String> = self
            .bone_depths()
            .into_iter()
            .filter(|(n, d)| *d <= 2 && (n.ends_with("TransN") || n.ends_with("TopN")))
            .map(|(n, _)| n)
            .collect();
        let mut log = String::new();
        for (clip, a) in self.anims.iter_mut() {
            for b in &tops {
                if let Some(r) = a.trans_range(b) {
                    let (dx, dy, dz) = (r[0].1 - r[0].0, r[1].1 - r[1].0, r[2].1 - r[2].0);
                    if dx > 0.5 || dz > 0.5 {
                        log.push_str(&format!("
  root motion stripped: {clip} {b} x {dx:.1} z {dz:.1} (y {dy:.1} kept)"));
                    }
                    a.hold_translation(b, [true, false, true]);
                }
            }
        }
        self.motion_log = log;
    }
    /// Bone names with their depth below the root (0 = root), in skeleton order.
    pub fn bone_depths(&self) -> Vec<(String, usize)> {
        let mut d: Vec<usize> = Vec::with_capacity(self.bones.len());
        for b in &self.bones {
            let x = match b.parent {
                Some(p) if p < d.len() => d[p] + 1,
                _ => 0,
            };
            d.push(x);
        }
        self.bones.iter().zip(d).map(|(b, x)| (b.name.clone(), x)).collect()
    }
    /// A bone's own MDL0 index.
    pub fn bone_index(&self, slot: usize) -> u32 {
        self.bones.get(slot).map(|b| b.index).unwrap_or(u32::MAX)
    }
    pub fn bone_name(&self, slot: usize) -> &str {
        self.bones.get(slot).map(|b| b.name.as_str()).unwrap_or("")
    }
    /// A point given in a bone's local space, in model space at `frame` of `anim`.
    pub fn bone_point(&self, anim: Option<&Anim>, frame: f32, slot: usize, local: [f32; 3]) -> [f32; 3] {
        let w = self.pose_bones(anim, frame);
        w.get(slot).map(|m| xf(m, local, 1.0)).unwrap_or(local)
    }

    /// Each bone's world matrix at `frame` of `anim` (bind pose where a bone has no channel).
    fn pose_bones(&self, anim: Option<&Anim>, frame: f32) -> Vec<M34> {
        let mut world: Vec<M34> = Vec::with_capacity(self.bones.len());
        self.pose_bones_into(anim, frame, &mut world);
        world
    }

    /// [`Self::pose_bones`] into a caller's buffer (no allocation once it holds `bones` matrices).
    fn pose_bones_into(&self, anim: Option<&Anim>, frame: f32, world: &mut Vec<M34>) {
        self.pose_bones_part(anim, frame, None, world);
    }

    /// [`Self::pose_bones_into`] with a second animated part: the bones marked in `part.2` take `part.0` at frame
    /// `part.1` (Brawl's partial motion channel: Porky's bust plays its own motion over the legs' motion).
    fn pose_bones_part(&self, anim: Option<&Anim>, frame: f32, part: Option<(&Anim, f32, &[bool])>, world: &mut Vec<M34>) {
        world.clear();
        for (bk, b) in self.bones.iter().enumerate() {
            let mut s = b.srt;
            let (anim, frame) = match part {
                Some((pa, pf, mask)) if mask.get(bk).copied().unwrap_or(false) => (Some(pa), pf),
                _ => (anim, frame),
            };
            if let Some((_, v)) = anim.and_then(|a| a.tracks.iter().find(|(n, _)| *n == b.name)) {
                for (k, t) in v.iter().enumerate() {
                    if let Some(t) = t {
                        s[k / 3][k % 3] = t.at(frame);
                    }
                }
            }
            let local = srt(s[0], s[1], s[2]);
            world.push(match b.parent {
                Some(p) if p < world.len() => mul(&world[p], &local),
                _ => local,
            });
        }
    }

    /// Pose the bones for `frame` of `anim` into `b.world` (no allocation once `b` came from [`Self::draw_bufs`]).
    pub fn pose_into(&self, anim: Option<&Anim>, frame: f32, b: &mut DrawBufs) {
        self.pose_bones_into(anim, frame, &mut b.world);
    }
    /// Buffers for [`Self::pose_keyed`]: the bone matrices plus one tag entry. Make them off the game thread.
    pub fn sim_bufs(&self) -> DrawBufs {
        let mut b = self.draw_bufs();
        b.world = Vec::with_capacity(self.bones.len() + 1);
        b
    }

    /// [`Self::pose_into`] unless `b` already holds the pose tagged `key` (the caller's (clip, frame)). The tag is an
    /// entry after the bones, in the same memory as the pose, so a rollback that restores one restores the other.
    pub fn pose_keyed(&self, anim: Option<&Anim>, frame: f32, key: [u32; 2], b: &mut DrawBufs) {
        let n = self.bones.len();
        if b.world.len() == n + 1 && b.world[n][0][0].to_bits() == key[0] && b.world[n][0][1].to_bits() == key[1] {
            return;
        }
        self.pose_bones_into(anim, frame, &mut b.world);
        b.world.push([[f32::from_bits(key[0]), f32::from_bits(key[1]), 0.0, 0.0], [0.0; 4], [0.0; 4]]);
    }

    /// A point in bone `slot`'s local space, in model space, from bones posed by [`Self::pose_into`].
    pub fn posed_point(b: &DrawBufs, slot: usize, local: [f32; 3]) -> [f32; 3] {
        b.world.get(slot).map(|m| xf(m, local, 1.0)).unwrap_or(local)
    }

    /// The rotation part (rows) of bone `slot`'s posed model-space matrix; None before a pose or past the bones.
    pub fn posed_rot(b: &DrawBufs, slot: usize) -> Option<[[f32; 3]; 3]> {
        b.world.get(slot).map(|m| core::array::from_fn(|i| [m[i][0], m[i][1], m[i][2]]))
    }

    /// Buffers sized for [`Self::skin_with`]: (bone matrices, skin matrices, vertices). Make them off the game thread
    /// (a helper thread) so that drawing never allocates inside the game scope, which rollback rewrites.
    pub fn draw_bufs(&self) -> DrawBufs {
        DrawBufs {
            world: Vec::with_capacity(self.bones.len()),
            mats: Vec::with_capacity(self.srcs.len()),
            tris: Vec::with_capacity(self.verts.len()),
            hide: Vec::with_capacity(self.batches.len()),
        }
    }

    /// Skin the triangle list for `frame` of `anim` into `out` (model space, lit colour).
    pub fn skin(&self, anim: Option<&Anim>, frame: f32, out: &mut Vec<Vert>) {
        let mut b = DrawBufs { world: Vec::new(), mats: Vec::new(), tris: core::mem::take(out), hide: Vec::new() };
        self.skin_with(anim, frame, &mut b);
        *out = b.tris;
    }

    pub fn vis_named(&self, name: &str) -> Option<&Vis> {
        self.vis.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// [`Self::skin_with`] plus the draw list's visibility: each batch whose visibility bone is off is marked in
    /// `b.hide` ([`draw_tris_vis`] skips it). A bone's visibility is the last of `vis` (a VIS0 and its frame) with a
    /// channel for it, else the bone's MDL0 default (nw4r: the AnmObjVis result, else the node's own flag).
    pub fn skin_vis(&self, anim: Option<&Anim>, frame: f32, vis: &[(&Vis, f32)], b: &mut DrawBufs) {
        self.skin_with(anim, frame, b);
        b.hide.clear();
        for bt in &self.batches {
            let shown = bt
                .vis_bone
                .and_then(|s| self.bones.get(s))
                .is_none_or(|bone| vis.iter().rev().find_map(|(v, f)| v.at(&bone.name, *f)).unwrap_or(bone.visible));
            b.hide.push(!shown);
        }
    }

    /// [`Self::skin`] into `b.tris`, with no allocation when `b` came from [`Self::draw_bufs`].
    pub fn skin_with(&self, anim: Option<&Anim>, frame: f32, b: &mut DrawBufs) {
        self.skin_part(anim, frame, None, b);
    }

    /// The bones in the subtree of the first bone whose name ends with `suffix` (a second animated part's bones).
    pub fn subtree_mask(&self, suffix: &str) -> Option<Vec<bool>> {
        let r = self.bone_named(suffix)?;
        let mut m = vec![false; self.bones.len()];
        m[r] = true;
        for k in r + 1..self.bones.len() {
            if let Some(p) = self.bones[k].parent {
                if p < k && m[p] {
                    m[k] = true;
                }
            }
        }
        Some(m)
    }

    /// [`Self::skin_vis`] with a second animated part (see [`Self::pose_bones_part`]).
    pub fn skin_vis_part(&self, anim: Option<&Anim>, frame: f32, part: Option<(&Anim, f32, &[bool])>, vis: &[(&Vis, f32)], b: &mut DrawBufs) {
        self.skin_part(anim, frame, part, b);
        b.hide.clear();
        for bt in &self.batches {
            let shown = bt
                .vis_bone
                .and_then(|s| self.bones.get(s))
                .is_none_or(|bone| vis.iter().rev().find_map(|(v, f)| v.at(&bone.name, *f)).unwrap_or(bone.visible));
            b.hide.push(!shown);
        }
    }

    /// [`Self::pose_keyed`] with a second animated part; `key` must name both clips and frames.
    pub fn pose_keyed_part(&self, anim: Option<&Anim>, frame: f32, part: Option<(&Anim, f32, &[bool])>, key: [u32; 2], b: &mut DrawBufs) {
        let n = self.bones.len();
        if b.world.len() == n + 1 && b.world[n][0][0].to_bits() == key[0] && b.world[n][0][1].to_bits() == key[1] {
            return;
        }
        self.pose_bones_part(anim, frame, part, &mut b.world);
        b.world.push([[f32::from_bits(key[0]), f32::from_bits(key[1]), 0.0, 0.0], [0.0; 4], [0.0; 4]]);
    }

    fn skin_part(&self, anim: Option<&Anim>, frame: f32, part: Option<(&Anim, f32, &[bool])>, b: &mut DrawBufs) {
        let DrawBufs { world, mats, tris: out, .. } = b;
        self.pose_bones_part(anim, frame, part, world);
        let world = &*world;
        mats.clear();
        mats.extend(self
            .srcs
            .iter()
            .map(|s| match s {
                Src::Ident => IDENT,
                Src::Bone(i) => world.get(*i).copied().unwrap_or(IDENT),
                Src::Mix(ws) => {
                    let mut acc = [[0.0f32; 4]; 3];
                    for (bi, w) in ws {
                        if let (Some(bw), Some(b)) = (world.get(*bi), self.bones.get(*bi)) {
                            let m = mul(bw, &b.inv);
                            for i in 0..3 {
                                for j in 0..4 {
                                    acc[i][j] += w * m[i][j];
                                }
                            }
                        }
                    }
                    acc
                }
            }));
        let light = {
            let l = [0.35f32, 0.55, 0.75];
            let n = (l[0] * l[0] + l[1] * l[1] + l[2] * l[2]).sqrt();
            [l[0] / n, l[1] / n, l[2] / n]
        };
        out.clear();
        let mut bi = 0;
        for (vi, (si, p, n, c, uv)) in self.verts.iter().enumerate() {
            while bi + 1 < self.batches.len() && vi as u32 >= self.batches[bi].end {
                bi += 1;
            }
            let m = &mats[*si as usize];
            let mut c = *c;
            // light channel 0: vertex material colour x clamp(ambient + diffuse) (one fixed key light)
            if let Some(bt) = self.batches.get(bi) {
                if let (Some(mc), true) = (bt.mat, bt.sh.is_some() && !one_stage()) {
                    c = mc;
                }
                if bt.lit {
                    let mut s = bt.amb;
                    if *n != [0.0; 3] {
                        let n = xf(m, *n, 0.0);
                        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                        s += 0.5 * ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / l).max(0.0);
                    }
                    let s = s.min(1.0);
                    for ck in c.iter_mut().take(3) {
                        *ck = (*ck as f32 * s) as u8;
                    }
                }
            }
            out.push((xf(m, *p, 1.0), c, *uv, if *n == [0.0; 3] { *n } else { xf(m, *n, 0.0) }));
        }
    }
}

/// The material state Petey's materials use (see [`Batch`]), from a v9 MDL0 material header at `mo`.
fn material(d: &[u8], mo: usize, texs: &[(String, std::sync::Arc<Tex>)], bt: &mut Batch) {
    bt.sh = shade_of(d, mo, texs);
    if std::env::var_os("MELEE_MDL_LOG").is_some() {
        if let Some(s) = &bt.sh {
            let st: Vec<String> = (0..s.stages as usize)
                .map(|k| format!("[o {:?} c {:06x} a {:06x} k {} {}]", s.order[k], s.cenv[k], s.aenv[k], s.kc[k], s.ka[k]))
                .collect();
            eprintln!(
                "mdl0: shade {} stages {} gens {} maps {:?} swap {:?} creg {:?} kreg {:?} {}",
                cstr(d, at(mo, bei(d, mo + 8))),
                s.stages,
                s.gens.iter().map(|g| (g.env, g.ident)).collect::<Vec<_>>().len(),
                s.maps.iter().map(|m| m.0).collect::<Vec<_>>(),
                s.swap,
                s.creg,
                s.kreg,
                st.join(" ")
            );
        }
    }
    // texture 0 of the material's texture references (name, then wrap S / T)
    if std::env::var_os("MELEE_MDL_LOG").is_some() {
        let n = be32(d, mo + 0x2C) as usize;
        let e0 = at(mo, bei(d, mo + 0x30));
        let names: Vec<String> = (0..n.min(8)).map(|k| cstr(d, at(e0 + 0x34 * k, bei(d, e0 + 0x34 * k)))).collect();
        eprintln!("mdl0: material {} texrefs {:?} found {:?}", cstr(d, at(mo, bei(d, mo + 8))), names, names.iter().map(|nm| texs.iter().any(|(t, _)| t == nm)).collect::<Vec<_>>());
    }
    if be32(d, mo + 0x2C) > 0 {
        let e = at(mo, bei(d, mo + 0x30));
        let name = cstr(d, at(e, bei(d, e)));
        bt.tex = texs.iter().find(|(n, _)| *n == name).map(|(_, t)| t.clone());
        bt.wrap = [be32(d, e + 0x18).min(2), be32(d, e + 0x1C).min(2)];
    }
    // the pixel display list: BP 0xF3 is the alpha compare (ref0 0-7, ref1 8-15, comp0 16-18, comp1 19-21, op 22-23)
    let dl = at(mo, bei(d, mo + 0x38));
    let end = mo + be32(d, mo) as usize;
    let mut p = dl;
    while p + 5 <= end && d.get(p) == Some(&0x61) {
        let v = be32(d, p + 1);
        if v >> 24 == 0x41 && v & 1 != 0 && v & 0x800 == 0 {
            bt.blend = Some((((v >> 8) & 7) as u8, ((v >> 5) & 7) as u8));
        }
        if v >> 24 == 0xF3 {
            bt.alpha = [((v >> 16) & 7) as u8, v as u8, ((v >> 22) & 3) as u8, ((v >> 19) & 7) as u8, (v >> 8) as u8];
        }
        p += 5;
    }
    // light channel 0 (flags, material colour, ambient colour, colour control): lit when the control enables it
    if d[mo + 0x15] > 0 {
        let lc = mo + 0x3EC;
        let amb = &d[lc + 8..lc + 11];
        bt.amb = (amb[0] as f32 + amb[1] as f32 + amb[2] as f32) / (3.0 * 255.0);
        bt.lit = be32(d, lc + 12) & 2 != 0;
        if be32(d, lc + 12) & 1 == 0 {
            bt.mat = Some([d[lc + 4], d[lc + 5], d[lc + 6], d[lc + 7]]);
        }
    }
}

fn mdl0(d: &[u8], m: usize, texs: &[(String, std::sync::Arc<Tex>)]) -> Option<Rig> {
    mdl0_any(d, m, texs, true)
}

/// A bone-only rig (no geometry needed) from the first MDL0 of BRRES `d`, with every CHR0 of `d`: a stage gimmick's
/// locator model (explore.rs: placement paths).
pub fn locator_rig(d: &[u8]) -> Option<Rig> {
    let entries = brres_entries(d)?;
    let (_, m) = entries.iter().find(|(_, o)| d.get(*o..*o + 4) == Some(b"MDL0"))?;
    let mut r = mdl0_any(d, *m, &[], false)?;
    for (n, o) in entries.iter().filter(|(_, o)| d.get(*o..*o + 4) == Some(b"CHR0")) {
        r.anims.push((n.clone(), chr0(d, *o)));
    }
    Some(r)
}

fn mdl0_any(d: &[u8], m: usize, texs: &[(String, std::sync::Arc<Tex>)], need_tris: bool) -> Option<Rig> {
    if be32(d, m + 8) != 9 {
        return None; // only the v9 layout is read
    }
    let sec = |i: usize| -> Vec<(String, usize)> {
        let s = bei(d, m + 0x10 + 4 * i);
        if s == 0 { vec![] } else { group(d, at(m, s)) }
    };
    let (defs, verts, norms, cols, uvs, mats, objs) = (sec(0), sec(2), sec(3), sec(4), sec(5), sec(6), sec(8));
    // bones in file order (parents come first); node id -> matrix source
    let mut bones: Vec<Bone> = vec![];
    let mut offs: Vec<usize> = vec![];
    let mut srcs: Vec<Src> = vec![Src::Ident];
    let mut node_src: Vec<(u32, u16)> = vec![];
    let mut bone_of_node: Vec<(u32, usize)> = vec![];
    for (name, b) in &sec(1) {
        let b = *b;
        let rd = |o: usize| [bef(d, b + o), bef(d, b + o + 4), bef(d, b + o + 8)];
        let par = bei(d, b + 0x5C);
        let parent = if par == 0 { None } else { offs.iter().position(|o| *o == at(b, par)) };
        let mut inv = [[0.0; 4]; 3];
        for (i, row) in inv.iter_mut().enumerate() {
            for (j, x) in row.iter_mut().enumerate() {
                *x = bef(d, b + 0xA0 + 16 * i + 4 * j);
            }
        }
        let bi = bones.len();
        bones.push(Bone { name: name.clone(), index: be32(d, b + 0x0C), srt: [rd(0x20), rd(0x2C), rd(0x38)], parent, inv, visible: be32(d, b + 0x14) & 0x100 != 0 });
        offs.push(b);
        let id = be32(d, b + 0x10);
        srcs.push(Src::Bone(bi));
        node_src.push((id, (srcs.len() - 1) as u16));
        bone_of_node.push((id, bi));
    }
    // MELEE_MDL_VTX_LOG: each bone's rest world from its SRT chain against the file's stored world (bone header
    // [off]) and inverse: where the chain and the file disagree, the bone's flags change how it composes
    if std::env::var_os("MELEE_MDL_VTX_LOG").is_some() && bones.len() > 20 {
        let mut w: Vec<M34> = vec![];
        for (k, bn) in bones.iter().enumerate() {
            let l = srt(bn.srt[0], bn.srt[1], bn.srt[2]);
            let m = match bn.parent { Some(p) if p < w.len() => mul(&w[p], &l), _ => l };
            let b = offs[k];
            let mut st = [[0.0f32; 4]; 3];
            for (i, row) in st.iter_mut().enumerate() {
                for (j, x) in row.iter_mut().enumerate() {
                    *x = bef(d, b + 112 + 16 * i + 4 * j); // [off] the stored world
                }
            }
            let dmax = (0..3).flat_map(|i| (0..4).map(move |j| (i, j))).map(|(i, j)| (m[i][j] - st[i][j]).abs()).fold(0.0f32, f32::max);
            let id = mul(&m, &bn.inv);
            let imax = (0..3).flat_map(|i| (0..3).map(move |j| (i, j))).map(|(i, j)| (id[i][j] - if i == j { 1.0 } else { 0.0 }).abs()).fold(0.0f32, f32::max);
            eprintln!(
                "mdl0 bone: {k} {} flags {} srt {:?} chain-vs-stored {dmax:.3} chain*inv-vs-I {imax:.3} stored row1 {:?}",
                bn.name, be32(d, b + 20), bn.srt, st[1]
            );
            w.push(m);
        }
    }
    if let Some((_, o)) = defs.iter().find(|(n, _)| n == "NodeMix") {
        let mut p = *o;
        loop {
            match d.get(p) {
                Some(5) => p += 5,
                Some(3) => {
                    let (id, n) = (be16(d, p + 1) as u32, d[p + 3] as usize);
                    let mut ws = vec![];
                    for k in 0..n {
                        let (bn, w) = (be16(d, p + 4 + 6 * k) as u32, bef(d, p + 6 + 6 * k));
                        if let Some((_, bi)) = bone_of_node.iter().find(|(i, _)| *i == bn) {
                            ws.push((*bi, w));
                        }
                    }
                    srcs.push(Src::Mix(ws));
                    node_src.push((id, (srcs.len() - 1) as u16));
                    p += 4 + 6 * n;
                }
                _ => break,
            }
        }
    }
    let src_of = |id: u32| node_src.iter().rev().find(|(i, _)| *i == id).map(|(_, s)| *s).unwrap_or(0);
    // object index -> (material header offset, visibility bone index), from DrawOpa / DrawXlu
    let mut obj_mat: Vec<(usize, usize, u32)> = vec![];
    for (n, o) in &defs {
        if !n.starts_with("Draw") {
            continue;
        }
        let mut p = *o;
        while d.get(p) == Some(&4) {
            let (mi, oi, vb) = (be16(d, p + 1) as usize, be16(d, p + 3) as usize, be16(d, p + 5) as u32);
            if let Some((_, mo)) = mats.iter().find(|(_, mo)| be32(d, mo + 0xC) as usize == mi) {
                obj_mat.push((oi, *mo, vb));
            }
            p += 8;
        }
    }
    let mut tris = vec![];
    let mut batches: Vec<Batch> = vec![];
    for (oi, (_, o)) in objs.iter().enumerate() {
        let o = *o;
        let idx_of = |id: i16, set: &Vec<(String, usize)>| -> Option<usize> {
            if id < 0 { None } else { set.iter().map(|x| x.1).find(|a| be32(d, a + 0x10) as usize == id as usize) }
        };
        let id = |k: usize| be16(d, o + 0x48 + 2 * k) as i16;
        let pos = idx_of(id(0), &verts).map(|a| arr(d, a));
        let nrm = idx_of(id(1), &norms).map(|a| arr(d, a));
        let clr = idx_of(id(2), &cols);
        let uv = idx_of(id(4), &uvs).map(|a| arr(d, a));
        let Some(pos) = pos else { continue };
        let mut bt = Batch { end: 0, tex: None, wrap: [1, 1], alpha: [7, 0, 0, 7, 0], amb: 1.0, lit: false, blend: None, vis_bone: None, sh: None, mat: None };
        if let Some((_, mo, vb)) = obj_mat.iter().find(|(i, _, _)| *i == oi) {
            material(d, *mo, texs, &mut bt);
            bt.vis_bone = bones.iter().position(|b| b.index == *vb);
        }
        let (lo, hi) = (be32(d, o + 0xC), be32(d, o + 0x10));
        // per-vertex attribute byte sizes in display-list order
        let mut lay: Vec<(u8, usize)> = vec![]; // (attr: 0 skip, 1 pos, 2 nrm, 3 clr0, 4 tex0), size
        if lo & 1 != 0 {
            lay.push((0, 1));
        }
        for k in 0..8 {
            if lo & (2 << k) != 0 {
                lay.push((0, 1));
            }
        }
        let sz = |v: u32| -> Option<usize> {
            match v & 3 {
                0 => Some(0),
                2 => Some(1),
                3 => Some(2),
                _ => None, // direct data: not handled
            }
        };
        for (attr, sh) in [(1u8, 9), (2, 11), (3, 13), (0, 15)] {
            let s = sz(lo >> sh)?;
            if s > 0 {
                lay.push((attr, s));
            }
        }
        for k in 0..8 {
            let s = sz(hi >> (2 * k))?;
            if s > 0 {
                lay.push((if k == 0 { 4 } else { 0 }, s));
            }
        }
        let vsize: usize = lay.iter().map(|l| l.1).sum();
        let obj_node = bei(d, o + 8);
        let obj_src = if obj_node >= 0 {
            bone_of_node.iter().find(|(i, _)| *i == obj_node as u32).map(|(_, bi)| (*bi + 1) as u16)
        } else {
            None
        };
        let mut slots = [0u32; 10];
        let dl = at(o + 0x24, bei(d, o + 0x2C));
        let dl_end = dl + be32(d, o + 0x28) as usize;
        let mut p = dl;
        let vert = |p: usize, slots: &[u32; 10]| -> (u16, [f32; 3], [f32; 3], [u8; 4], [f32; 2]) {
            let mut q = p;
            let (mut pi, mut ni, mut ci, mut ti) = (0, usize::MAX, usize::MAX, usize::MAX);
            let mtx_idx = if lo & 1 != 0 { d.get(q).copied().unwrap_or(0) as usize / 3 } else { 0 };
            let si = obj_src.unwrap_or_else(|| src_of(slots[mtx_idx.min(9)]));
            for (attr, s) in &lay {
                let v = if *s == 1 { d.get(q).copied().unwrap_or(0) as usize } else { be16(d, q) as usize };
                match attr {
                    1 => pi = v,
                    2 => ni = v,
                    3 => ci = v,
                    4 => ti = v,
                    _ => {}
                }
                q += s;
            }
            let st = match &uv {
                Some(u) if ti != usize::MAX => comp_read(d, u, ti, 2),
                _ => [0.0; 3],
            };
            // the channel's material colour comes from the vertex (colour control 0x703); white without one
            let c = match clr {
                Some(co) if ci != usize::MAX => color_read(d, co, ci),
                _ => [255; 4],
            };
            let n = match &nrm {
                Some(na) if ni != usize::MAX => comp_read(d, na, ni, 3),
                _ => [0.0; 3],
            };
            (si, comp_read(d, &pos, pi, 3), n, c, [st[0], st[1]])
        };
        while p < dl_end {
            let op = d.get(p).copied().unwrap_or(0);
            match op {
                0x00 => p += 1,
                0x08 => p += 6,
                0x10 => {
                    let n = ((be32(d, p + 1) >> 16) & 0xF) as usize + 1;
                    p += 5 + 4 * n;
                }
                0x20 => {
                    let v = be32(d, p + 1);
                    let slot = ((v & 0xFFF) / 12) as usize; // low 12 bits: XF address (top 4: size - 1)
                    if slot < 10 {
                        slots[slot] = v >> 16;
                    }
                    p += 5;
                }
                0x28 | 0x30 | 0x38 => p += 5,
                0x80..=0xBF => {
                    let n = be16(d, p + 1) as usize;
                    let vs: Vec<_> = (0..n).map(|i| vert(p + 3 + i * vsize, &slots)).collect();
                    p += 3 + n * vsize;
                    let mut emit = |a: usize, b: usize, c: usize| {
                        for i in [a, b, c] {
                            tris.push(vs[i]);
                        }
                    };
                    match op & 0xF8 {
                        0x80 => {
                            for q in (0..n / 4).map(|k| k * 4) {
                                emit(q, q + 1, q + 2);
                                emit(q, q + 2, q + 3);
                            }
                        }
                        0x90 => (0..n / 3).for_each(|k| emit(3 * k, 3 * k + 1, 3 * k + 2)),
                        0x98 => (2..n).for_each(|k| if k % 2 == 0 { emit(k - 2, k - 1, k) } else { emit(k - 1, k - 2, k) }),
                        0xA0 => (2..n).for_each(|k| emit(0, k - 1, k)),
                        _ => {}
                    }
                }
                _ => break,
            }
        }
        // MELEE_MDL_VTX_LOG: per object, its bind node, the matrix-index mode and which sources its vertices take
        // (bone / mix / identity, with the bone names and a sample position): the vertex-level check of the skinning
        if std::env::var_os("MELEE_MDL_VTX_LOG").is_some() {
            let start = batches.last().map_or(0, |b: &Batch| b.end as usize);
            let mut used: Vec<(u16, usize, [f32; 3])> = vec![];
            for v in &tris[start..] {
                match used.iter_mut().find(|u| u.0 == v.0) {
                    Some(u) => u.1 += 1,
                    None => used.push((v.0, 1, v.1)),
                }
            }
            let desc: Vec<String> = used
                .iter()
                .map(|(si, n, p0)| {
                    let k = match srcs.get(*si as usize) {
                        Some(Src::Ident) => "ident".to_string(),
                        Some(Src::Bone(b)) => format!("bone {}", bones[*b].name),
                        Some(Src::Mix(ws)) => format!("mix {:?}", ws.iter().map(|(b, w)| (bones[*b].name.as_str(), *w)).collect::<Vec<_>>()),
                        None => "?".to_string(),
                    };
                    format!("src {si} {k} x{n} first ({:.1} {:.1} {:.1})", p0[0], p0[1], p0[2])
                })
                .collect();
            eprintln!("mdl0 vtx: object {} node {} mtxidx {} -> {}", objs[oi].0, obj_node, lo & 1, desc.join("; "));
        }
        bt.end = tris.len() as u32;
        batches.push(bt);
    }
    if std::env::var_os("MELEE_MDL_VTX_LOG").is_some() {
        let ids: Vec<String> = node_src.iter().map(|(id, s)| format!("{id}->{s}")).collect();
        eprintln!("mdl0 vtx: node ids {}", ids.join(" "));
        if let Some((_, o)) = defs.iter().find(|(n, _)| n == "NodeMix") {
            let raw: Vec<String> = d[*o..(*o + 96).min(d.len())].iter().map(|b| b.to_string()).collect();
            eprintln!("mdl0 vtx: NodeMix head {}", raw.join(" "));
        }
    }
    if tris.is_empty() && need_tris {
        return None;
    }
    Some(Rig { bones, srcs, verts: tris, batches, anims: vec![], vis: vec![], motion_files: vec![], min: [0.0; 3], max: [0.0; 3], motion_log: String::new() })
}

/// Extent of a posed triangle list; y from the 1st..99th percentile (a few far-off vertices must not set the feet
/// or the scale).
pub fn extent(tris: &[Vert]) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for (p, ..) in tris {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    if !tris.is_empty() {
        let mut ys: Vec<f32> = tris.iter().map(|(p, ..)| p[1]).collect();
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
        mn[1] = ys[ys.len() / 100];
        mx[1] = ys[ys.len() - 1 - ys.len() / 100];
    }
    (mn, mx)
}

// ---------------------------------------------------------------- draw

/// Draw a rig's posed triangle list (`tris` from [`Rig::skin_with`]): model x scale, yawed, with model-space y `foot`
/// at `at` (world). One run per material (see [`Batch`]): texture 0 sampled per pixel, `2 x texture x vertex colour`,
/// the material's alpha compare; opaque, back faces kept. `flash`: solid white.
/// `org`: the model-space ground height and depth centre that land on `at` (x stays: the model's x is its own).
/// Allocates nothing of its own: the texture objects live in the model, made on the loader's helper thread.
/// Every texture `rig`'s batches draw with: (texels, width, height, GX format, mip levels), as `draw_tris` hands
/// them to `GXInitTexObj` (for an upload ahead of the first draw).
pub fn rig_textures(rig: &Rig, mut f: impl FnMut(*const u8, u16, u16, u32, u32)) {
    for bt in &rig.batches {
        if let Some(t) = &bt.tex {
            f(t.data.as_ptr(), t.w, t.h, t.fmt, t.levels as u32);
        }
    }
}

/// `draw_tris` draws one triangle per material batch only (set around a call; presentation only).
pub static mut SAMPLE: bool = false;

/// A static model's triangles as GX display lists, one per material batch, placed at `at` (no scale or turn): the
/// bytes `draw_tris`'s immediate-mode loop would write for it (GX_DRAW_TRIANGLES on VTXFMT0, then per vertex the
/// position as 3 BE f32, RGBA8 with alpha 255, and for a textured batch the texture coordinate as 2 BE f32). Built
/// once off the game thread (a section's 10-20 k triangles cost ~1 ms a draw one GX call at a time).
pub fn build_lists(rig: &Rig, tris: &[Vert], at: [f32; 2]) -> Vec<Vec<u8>> {
    let mut out = Vec::with_capacity(rig.batches.len());
    let mut start = 0usize;
    for bt in &rig.batches {
        let end = (bt.end as usize).min(tris.len());
        let run = &tris[start.min(end)..end];
        start = end;
        let textured = bt.tex.is_some();
        // a shaded batch: every texture coordinate (an environment map from the model-space normal: a static model
        // does not turn, and the side camera looks down the z axis)
        let gens: &[Gen] = bt.sh.as_ref().filter(|_| !one_stage()).map_or(&[], |s| &s.gens[..]);
        let mut l = Vec::with_capacity(run.len() * (16 + 8 * gens.len().max(textured as usize)) + 3 * (run.len() / 60000 + 1) + 32);
        for chunk in run.chunks(3 * 20000) {
            l.push((melee_types::GX_TRIANGLES | melee_types::GX_VTXFMT0) as u8);
            l.extend_from_slice(&(chunk.len() as u16).to_be_bytes());
            for (p, col, uv, n) in chunk {
                for v in [at[0] + p[0], at[1] + p[1], p[2]] {
                    l.extend_from_slice(&v.to_bits().to_be_bytes());
                }
                l.extend_from_slice(&[col[0], col[1], col[2], 255]);
                if bt.sh.is_some() && !one_stage() {
                    for g in gens {
                        let q = gen_uv(g, *uv, *n);
                        l.extend_from_slice(&q[0].to_bits().to_be_bytes());
                        l.extend_from_slice(&q[1].to_bits().to_be_bytes());
                    }
                } else if textured {
                    l.extend_from_slice(&uv[0].to_bits().to_be_bytes());
                    l.extend_from_slice(&uv[1].to_bits().to_be_bytes());
                }
            }
        }
        // GX display lists are whole 32-byte blocks (NOPs after the last command)
        l.resize(l.len().next_multiple_of(32), 0);
        out.push(l);
    }
    out
}

/// `draw_tris` with a model's prebuilt display lists (`build_lists`, same placement): one GX call per batch.
pub unsafe fn draw_tris_lists(rig: &Rig, tris: &[Vert], at: [f32; 3], lists: &[Vec<u8>]) {
    LISTS = Some(core::mem::transmute::<&[Vec<u8>], &'static [Vec<u8>]>(lists));
    draw_tris(rig, tris, [0.0, 0.0], at, 1.0, 0.0, false);
    LISTS = None;
}
/// set by `draw_tris_lists` around one `draw_tris` call
static mut LISTS: Option<&'static [Vec<u8>]> = None;

pub unsafe fn draw_tris(rig: &Rig, tris: &[Vert], org: [f32; 2], at: [f32; 3], scale: f32, yaw: f32, flash: bool) {
    let (s, c) = (yaw.sin(), yaw.cos());
    draw_tris_m(rig, tris, org, at, scale, [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]], flash)
}

/// [`draw_tris`] with a full model rotation `m` (row-major, applied to the model-space point) in place of the yaw.
pub unsafe fn draw_tris_m(rig: &Rig, tris: &[Vert], org: [f32; 2], at: [f32; 3], scale: f32, m: [[f32; 3]; 3], flash: bool) {
    draw_tris_vis(rig, tris, &[], org, at, scale, m, flash)
}

/// [`draw_tris_m`] skipping the batches `hide` marks (from [`Rig::skin_vis`]; empty: every batch is drawn).
#[allow(clippy::too_many_arguments)]
pub unsafe fn draw_tris_vis(rig: &Rig, tris: &[Vert], hide: &[bool], org: [f32; 2], at: [f32; 3], scale: f32, m: [[f32; 3]; 3], flash: bool) {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_runtime::tu::lbrefract::ext as gi;
    use melee_runtime::tu::sobjlib::ext as gs;
    use melee_runtime::tu::tobj::ext as gt;
    use melee_types::*;
    if tris.is_empty() {
        return;
    }
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
    gx::GXSetColorUpdate(1);
    gx::GXSetAlphaUpdate(0);
    gx::GXSetBlendMode(GX_BM_NONE as _, GX_BL_ONE as _, GX_BL_ZERO as _, GX_LO_NOOP as _);
    gx::GXSetZMode(1, GX_LEQUAL as _, 1);
    gx::GXSetZCompLoc(0);
    gi::GXSetNumIndStages(0);
    gi::GXSetTevDirect(GX_TEVSTAGE0 as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP0 as _, GX_CH_RED as _, GX_CH_GREEN as _, GX_CH_BLUE as _, GX_CH_ALPHA as _);
    gs::GXSetTevSwapMode(GX_TEVSTAGE0 as _, GX_TEV_SWAP0 as _, GX_TEV_SWAP0 as _);
    gx::GXSetNumTevStages(1);
    gx::GXSetNumChans(1);
    gx::GXSetChanCtrl(GX_COLOR0A0 as _, 0, GX_SRC_REG as _, GX_SRC_VTX as _, 0, GX_DF_NONE as _, GX_AF_NONE as _);
    gx::GXSetCullMode(GX_CULL_NONE as _);
    gx::GXClearVtxDesc();
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_POS as _, GX_POS_XYZ as _, GX_F32 as _, 0);
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_CLR0 as _, GX_CLR_RGBA as _, GX_RGBA8 as _, 0);
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_TEX0 as _, GX_TEX_ST as _, GX_F32 as _, 0);
    gx::GXSetVtxDesc(GX_VA_POS as _, GX_DIRECT as _);
    gx::GXSetVtxDesc(GX_VA_CLR0 as _, GX_DIRECT as _);
    let cobj = melee_runtime::tu::cobj::HSD_CObjGetCurrent();
    if cobj.is_null() {
        return;
    }
    melee_runtime::tu::pobj::native::mx::GXLoadPosMtxImm((&mut (*cobj).view_mtx) as *mut Mtx as _, 0);
    gx::GXSetCurrentMtx(0);
    // the view's rotation (environment maps take the view-space normal)
    let vm = (*cobj).view_mtx;
    let mut shaded_last = false;
    let mut start = 0usize;
    for (bi, bt) in rig.batches.iter().enumerate() {
        let k = bi;
        let end = (bt.end as usize).min(tris.len());
        let run = &tris[start.min(end)..end];
        start = end;
        if run.is_empty() || hide.get(k).copied().unwrap_or(false) {
            continue;
        }
        let shade = bt.sh.as_deref().filter(|_| !flash && !one_stage());
        let textured = !flash && bt.tex.is_some();
        // blending materials (Brawl stage shadows, light shafts) blend and keep the depth buffer as it is
        match bt.blend {
            Some((sf, df)) if !flash => {
                gx::GXSetBlendMode(GX_BM_BLEND as _, sf as _, df as _, GX_LO_NOOP as _);
                gx::GXSetZMode(1, GX_LEQUAL as _, 0);
            }
            _ => {
                gx::GXSetBlendMode(GX_BM_NONE as _, GX_BL_ONE as _, GX_BL_ZERO as _, GX_LO_NOOP as _);
                gx::GXSetZMode(1, GX_LEQUAL as _, 1);
            }
        }
        if shaded_last && shade.is_none() {
            reset_shade();
        }
        shaded_last = shade.is_some();
        if let Some(sh) = shade {
            set_shade(sh);
            let a = bt.alpha;
            gx::GXSetAlphaCompare(a[0] as _, a[1], a[2] as _, a[3] as _, a[4]);
        } else if let (Some(t), false) = (&bt.tex, flash) {
            let o = t.obj.as_ptr() as *mut _GXTexObj;
            gs::GXInitTexObj(o, t.data.as_ptr() as *mut _, t.w, t.h, t.fmt, bt.wrap[0], bt.wrap[1], (t.levels > 1) as u8);
            let min = if t.levels > 1 { GX_LIN_MIP_LIN } else { GX_LINEAR };
            gt::GXInitTexObjLOD(o, min as _, GX_LINEAR as _, 0.0, (t.levels - 1) as f32, 0.0, 0, 0, GX_ANISO_1 as _);
            gs::GXLoadTexObj(o, GX_TEXMAP0 as _);
            gx::GXSetNumTexGens(1);
            gs::GXSetTexCoordGen2(GX_TEXCOORD0 as _, GX_TG_MTX2x4 as _, GX_TG_TEX0 as _, GX_IDENTITY as _, 0, GX_PTIDENTITY as _);
            gx::GXSetVtxDesc(GX_VA_TEX0 as _, GX_DIRECT as _);
            gx::GXSetTevOrder(GX_TEVSTAGE0 as _, GX_TEXCOORD0 as _, GX_TEXMAP0 as _, GX_COLOR0A0 as _);
            gs::GXSetTevColorIn(GX_TEVSTAGE0 as _, GX_CC_ZERO as _, GX_CC_TEXC as _, GX_CC_RASC as _, GX_CC_ZERO as _);
            gs::GXSetTevColorOp(GX_TEVSTAGE0 as _, GX_TEV_ADD as _, GX_TB_ZERO as _, GX_CS_SCALE_2 as _, 1, GX_TEVPREV as _);
            gs::GXSetTevAlphaIn(GX_TEVSTAGE0 as _, GX_CA_ZERO as _, GX_CA_TEXA as _, GX_CA_RASA as _, GX_CA_ZERO as _);
            gs::GXSetTevAlphaOp(GX_TEVSTAGE0 as _, GX_TEV_ADD as _, GX_TB_ZERO as _, GX_CS_SCALE_1 as _, 1, GX_TEVPREV as _);
            let a = bt.alpha;
            gx::GXSetAlphaCompare(a[0] as _, a[1], a[2] as _, a[3] as _, a[4]);
        } else {
            gx::GXSetNumTexGens(0);
            gx::GXSetVtxDesc(GX_VA_TEX0 as _, GX_NONE as _);
            gx::GXSetTevOrder(GX_TEVSTAGE0 as _, GX_TEXCOORD_NULL as _, GX_TEXMAP_NULL as _, GX_COLOR0A0 as _);
            gx::GXSetTevOp(GX_TEVSTAGE0 as _, GX_PASSCLR as _);
            gx::GXSetAlphaCompare(GX_ALWAYS as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
        }
        // GX vertex counts are u16: send in chunks of whole triangles
        // SAMPLE: one triangle per batch (keeps the batch's texture resident in the renderer's cache; the caller
        // draws it where the camera does not look)
        let run = if SAMPLE { &run[..3.min(run.len())] } else { run };
        if let (Some(ls), false) = (LISTS, SAMPLE) {
            if let Some(l) = ls.get(bi).filter(|l| !l.is_empty()) {
                melee_runtime::tu::pobj::ext::GXCallDisplayList(l.as_ptr() as *mut _, l.len() as u32);
                continue;
            }
        }
        for chunk in run.chunks(3 * 20000) {
            gx::GXBegin(GX_TRIANGLES as _, GX_VTXFMT0 as _, chunk.len() as _);
            for (p, col, uv, n) in chunk {
                let (x, y, z) = (p[0] * scale, (p[1] - org[0]) * scale, (p[2] - org[1]) * scale);
                gx::GXPosition3f32(
                    at[0] + m[0][0] * x + m[0][1] * y + m[0][2] * z,
                    at[1] + m[1][0] * x + m[1][1] * y + m[1][2] * z,
                    at[2] + m[2][0] * x + m[2][1] * y + m[2][2] * z,
                );
                if flash {
                    gx::GXColor4u8(255, 255, 255, 255);
                } else {
                    gx::GXColor4u8(col[0], col[1], col[2], 255);
                }
                if let Some(sh) = shade {
                    let w = [
                        m[0][0] * n[0] + m[0][1] * n[1] + m[0][2] * n[2],
                        m[1][0] * n[0] + m[1][1] * n[1] + m[1][2] * n[2],
                        m[2][0] * n[0] + m[2][1] * n[1] + m[2][2] * n[2],
                    ];
                    let nv = [
                        vm[0][0] * w[0] + vm[0][1] * w[1] + vm[0][2] * w[2],
                        vm[1][0] * w[0] + vm[1][1] * w[1] + vm[1][2] * w[2],
                        vm[2][0] * w[0] + vm[2][1] * w[1] + vm[2][2] * w[2],
                    ];
                    for g in &sh.gens {
                        let q = gen_uv(g, *uv, nv);
                        gs::GXTexCoord2f32(q[0], q[1]);
                    }
                } else if textured {
                    gs::GXTexCoord2f32(uv[0], uv[1]);
                }
            }
        }
    }
    if shaded_last {
        reset_shade();
    }
    gx::GXSetAlphaCompare(GX_ALWAYS as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
}

#[cfg(test)]
mod tests {
    /// With the owner's local data present (skipped otherwise): every clip decodes so that the arm models' attach
    /// bones follow the body's frame for frame, and the feet stay on y 0 in every grounded clip.
    #[test]
    fn arms_follow_body_and_feet_on_ground() {
        let Ok(m) = super::load() else { return };
        let b = &m.body;
        let bi = |r: &super::Rig, n: &str| r.bones.iter().position(|x| x.name.ends_with(n)).unwrap();
        let mut t = vec![];
        for (an, a) in &b.anims {
            for f in 0..a.frames as i32 {
                let w = b.pose_bones(Some(a), f as f32);
                for (r, n) in [(m.arm_l.as_ref().unwrap(), "Attach01N"), (m.arm_r.as_ref().unwrap(), "Attach02N")] {
                    let wa = r.pose_bones(r.anim(an), f as f32);
                    let (x, y) = (w[bi(b, n)], wa[bi(r, n)]);
                    for i in 0..3 {
                        for j in 0..4 {
                            assert!((x[i][j] - y[i][j]).abs() < 0.5, "{an} f{f} {n}: arm attach off the body");
                        }
                    }
                }
                if an != "JumpAttack" {
                    b.skin(Some(a), f as f32, &mut t);
                    let lo = super::extent(&t).0[1];
                    assert!((-2.0..6.0).contains(&lo), "{an} f{f}: lowest point {lo}");
                }
            }
        }
    }
}

#[cfg(test)]
mod probe {
    /// Local-only probe (needs the owner's files; prints derived numbers only).
    #[test]
    #[ignore]
    fn petey_probe() {
        let m = super::load().expect("local data");
        let p = m.params.as_ref().expect("params");
        eprintln!("params {:?}", p);
        for (nm, r) in [("body", Some(&m.body)), ("armL", m.arm_l.as_ref()), ("armR", m.arm_r.as_ref())] {
            let r = r.unwrap();
            eprintln!("{nm} extent {:?} {:?}", r.min, r.max);
            for i in 0..r.bones.len() {
                eprintln!("  {nm} slot {i} idx {} {}", r.bones[i].index, r.bones[i].name);
            }
            for (an, a) in &r.anims {
                eprintln!("  {nm} anim {an} frames {}", a.frames);
            }
        }
        let w = m.arm_l.as_ref().unwrap();
        for b in ["XRotN", "LWeakN", "LIronNa", "PeachBodyM"] {
            let s = w.bone_named(b).unwrap();
            for f in [0.0, 40.0, 80.0, 120.0] {
                eprintln!("armL {b} BasketAttackL f{f} {:?}", w.bone_point(w.anim("BasketAttackL"), f, s, [0.0; 3]));
            }
        }
    }
}

// ---------------------------------------------------------------- shared with other Brawl importers (explore.rs)

/// Every leaf file of the adventure stage archive `name` (`stage/adventure/<name>`), from the owner's local files.
/// `None` without them. Nothing is written anywhere and the path is never printed.
pub fn pac_leaves(name: &str) -> Option<Vec<Vec<u8>>> {
    let root = local_root()?;
    let raw = std::fs::read(root.join("disc").join("files").join("stage").join("adventure").join(name)).ok()?;
    let mut leaves = vec![];
    walk(&unwrap(raw), 0, &mut leaves);
    Some(leaves)
}

/// [`pac_leaves`] with each leaf's (file type, file index) path through the archive tree.
pub fn pac_leaves_tagged(name: &str) -> Option<(Vec<Vec<u8>>, Vec<Vec<(u16, u16)>>)> {
    let root = local_root()?;
    let raw = std::fs::read(root.join("disc").join("files").join("stage").join("adventure").join(name)).ok()?;
    let (mut leaves, mut tags) = (vec![], vec![]);
    walk_tagged(&unwrap(raw), 0, &mut leaves, &mut vec![], Some(&mut tags));
    Some((leaves, tags))
}

/// The MDL0s named in `want` from `leaves` (textures resolved across every BRRES of the archive), each posed once in
/// its bind pose: (name, rig, triangle list in model space). For static scenery (stage maps, doors).
pub fn static_models(leaves: &[Vec<u8>], want: &[&str]) -> Vec<(String, Rig, Vec<Vert>)> {
    let mut tex_at: Vec<(String, usize, usize)> = vec![];
    let mut plt_at: Vec<(String, usize, usize)> = vec![];
    let mut mdls: Vec<(String, usize, usize)> = vec![];
    for (li, b) in leaves.iter().enumerate() {
        if b.len() < 0x10 || &b[..4] != b"bres" {
            continue;
        }
        let Some(entries) = brres_entries(b) else { continue };
        for (name, off) in entries {
            let list = match b.get(off..off + 4) {
                Some(b"TEX0") => &mut tex_at,
                Some(b"PLT0") => &mut plt_at,
                Some(b"MDL0") if want.contains(&name.as_str()) => &mut mdls,
                _ => continue,
            };
            if !list.iter().any(|(n, _, _)| *n == name) {
                list.push((name, li, off));
            }
        }
    }
    let texs: Vec<(String, std::sync::Arc<Tex>)> = tex_at
        .iter()
        .filter_map(|(n, li, o)| {
            let plt = plt_at.iter().find(|(pn, _, _)| pn == n).map(|(_, pl, po)| (leaves[*pl].as_slice(), *po));
            tex0(&leaves[*li], *o, plt).map(|t| (n.clone(), std::sync::Arc::new(t)))
        })
        .collect();
    let mut out = vec![];
    for w in want {
        let Some((_, li, off)) = mdls.iter().find(|(n, _, _)| n == w) else { continue };
        let Some(mut r) = mdl0(&leaves[*li], *off, &texs) else { continue };
        let mut tris = vec![];
        r.skin(None, 0.0, &mut tris);
        (r.min, r.max) = extent(&tris);
        out.push((w.to_string(), r, tris));
    }
    out
}

/// A black veil over the whole view with alpha `a` (door transitions). Drawn in view space just past the near plane.
pub unsafe fn draw_fade(a: u8) {
    use melee_runtime::tu::displayfunc_native::ext as gx;
    use melee_runtime::tu::lbrefract::ext as gi;
    use melee_runtime::tu::sobjlib::ext as gs;
    use melee_types::*;
    if a == 0 {
        return;
    }
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
    gx::GXSetColorUpdate(1);
    gx::GXSetAlphaUpdate(0);
    gx::GXSetBlendMode(GX_BM_BLEND as _, GX_BL_SRCALPHA as _, GX_BL_INVSRCALPHA as _, GX_LO_NOOP as _);
    gx::GXSetZMode(0, GX_ALWAYS as _, 0);
    gi::GXSetNumIndStages(0);
    gi::GXSetTevDirect(GX_TEVSTAGE0 as _);
    gs::GXSetTevSwapModeTable(GX_TEV_SWAP0 as _, GX_CH_RED as _, GX_CH_GREEN as _, GX_CH_BLUE as _, GX_CH_ALPHA as _);
    gs::GXSetTevSwapMode(GX_TEVSTAGE0 as _, GX_TEV_SWAP0 as _, GX_TEV_SWAP0 as _);
    gx::GXSetNumTevStages(1);
    gx::GXSetNumTexGens(0);
    gx::GXSetNumChans(1);
    gx::GXSetChanCtrl(GX_COLOR0A0 as _, 0, GX_SRC_REG as _, GX_SRC_VTX as _, 0, GX_DF_NONE as _, GX_AF_NONE as _);
    gx::GXSetCullMode(GX_CULL_NONE as _);
    gx::GXSetTevOrder(GX_TEVSTAGE0 as _, GX_TEXCOORD_NULL as _, GX_TEXMAP_NULL as _, GX_COLOR0A0 as _);
    gx::GXSetTevOp(GX_TEVSTAGE0 as _, GX_PASSCLR as _);
    gx::GXSetAlphaCompare(GX_ALWAYS as _, 0, GX_AOP_AND as _, GX_ALWAYS as _, 0);
    gx::GXClearVtxDesc();
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_POS as _, GX_POS_XYZ as _, GX_F32 as _, 0);
    gx::GXSetVtxAttrFmt(GX_VTXFMT0 as _, GX_VA_CLR0 as _, GX_CLR_RGBA as _, GX_RGBA8 as _, 0);
    gx::GXSetVtxDesc(GX_VA_POS as _, GX_DIRECT as _);
    gx::GXSetVtxDesc(GX_VA_CLR0 as _, GX_DIRECT as _);
    let mut id: Mtx = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
    melee_runtime::tu::pobj::native::mx::GXLoadPosMtxImm((&mut id) as *mut Mtx as _, 0);
    gx::GXSetCurrentMtx(0);
    let (z, e) = (-10.0f32, 1000.0f32);
    gx::GXBegin(GX_TRIANGLES as _, GX_VTXFMT0 as _, 6);
    for (x, y) in [(-e, -e), (e, -e), (e, e), (-e, -e), (e, e), (-e, e)] {
        gx::GXPosition3f32(x, y, z);
        gx::GXColor4u8(0, 0, 0, a);
    }
    gx::GXSetZMode(1, GX_LEQUAL as _, 1);
    gx::GXSetBlendMode(GX_BM_NONE as _, GX_BL_ONE as _, GX_BL_ZERO as _, GX_LO_NOOP as _);
    melee_runtime::tu::state_native::HSD_StateInvalidate(-1);
}



#[cfg(test)]
mod water_probe {
    #[test]
    fn water_material() {
        let Some(leaves) = super::pac_leaves("080001.pac") else { return };
        for b in leaves.iter().filter(|b| b.len() >= 16 && &b[..4] == b"bres") {
            for (n, o) in super::brres_entries(b).unwrap_or_default() {
                let k = b.get(o..o + 4).map(|x| String::from_utf8_lossy(x).into_owned()).unwrap_or_default();
                if k != "MDL0" && k != "TEX0" && k != "PLT0" { println!("PROBE entry {k} {n}"); }
            }
        }
        for (n, r, t) in super::static_models(&leaves, &["080vmizu00", "080vAdvJungle01"]) {
            println!("PROBE model {n}: {} tris, {} batches", t.len() / 3, r.batches.len());
            for (i, bt) in r.batches.iter().enumerate() {
                let s0 = if i == 0 { 0 } else { r.batches[i - 1].end as usize }; let (lo, hi) = super::extent(&t[s0.min(t.len())..(bt.end as usize).min(t.len())]); println!("PROBE  ext {i} {:?} {:?}", lo, hi);
                println!("PROBE  batch {i} end {} tex {:?} wrap {:?} alpha {:?} blend {:?} amb {} lit {}", bt.end, bt.tex.as_ref().map(|x| (x.w, x.h, x.fmt)), bt.wrap, bt.alpha, bt.blend, bt.amb, bt.lit);
            }
        }
    }
}
