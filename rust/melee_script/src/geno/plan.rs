//! geno_plan.h and the plan half of geno_define_registry.inc (gn_plan_parse, gn_define_plan, gw_Geno_DefinePlanInt /
//! DefinePlanText): the flat reading of a base "none" define's plan.json (joint tree, parts table, animation bank clips,
//! motion rows, ftData joint fields, hurtboxes). Parsed once per define on first query; boot metadata, never state.
use super::json::{Doc, Ty};
use super::registry::{Env, Registry};

pub const GPL_MAX_JOINTS: usize = 256;
pub const GPL_MAX_OWN_ROWS: usize = 448;
pub const GPL_MAX_PARTS: usize = 56;
pub const GPL_MAX_CLIPS: usize = 1024;
pub const GPL_MAX_MOTIONS: usize = 368;
pub const GPL_MAX_ROWS: usize = 512;
pub const GPL_NONE: u8 = 255;

pub struct Plan {
    pub ok: i32,
    pub joints: i32,
    pub nparts: i32,
    pub joint_to_part: [u8; GPL_MAX_JOINTS],
    pub part_to_joint: [u8; GPL_MAX_PARTS],
    /// NUL-terminated, as the game half keeps the pointers in its rows.
    pub clip_name: Vec<Box<[u8]>>,
    pub clip_symbol: Vec<Box<[u8]>>,
    pub clip_offset: Vec<u32>,
    pub clip_bytes: Vec<u32>,
    pub clip_frames: Vec<i32>,
    pub nmotions: i32,
    pub ft_x8: [i32; 5],
    pub ft_x34: [i32; 1],
    pub ft_x38: [i32; 2],
    pub ft_x44: [i32; 6],
    pub ft_x54: [i32; 5],
    pub ft_x58: [i32; 10],
    pub nhurt: i32,
    pub hurt_joint: [i32; 16],
    pub hurt_height: [i32; 16],
    pub hurt_grab: [i32; 16],
    pub hurt_a: [[f32; 3]; 16],
    pub hurt_b: [[f32; 3]; 16],
    pub hurt_radius: [f32; 16],
    pub nown: i32,
    pub own_clip: Vec<i16>,
    pub row_flags: Vec<u32>,
    pub row_blend: Vec<[u8; 2]>,
    pub has_blend: Vec<u8>,
    pub has_flags: Vec<u8>,
    pub row_clip: Vec<i16>,
    pub motion_clip: Vec<i16>,
}

impl Default for Plan {
    fn default() -> Self {
        Plan {
            ok: 0,
            joints: 0,
            nparts: 0,
            joint_to_part: [GPL_NONE; GPL_MAX_JOINTS],
            part_to_joint: [GPL_NONE; GPL_MAX_PARTS],
            clip_name: Vec::new(),
            clip_symbol: Vec::new(),
            clip_offset: Vec::new(),
            clip_bytes: Vec::new(),
            clip_frames: Vec::new(),
            nmotions: 0,
            ft_x8: [-1; 5],
            ft_x34: [-1; 1],
            ft_x38: [-1; 2],
            ft_x44: [-1; 6],
            ft_x54: [-1; 5],
            ft_x58: [-1; 10],
            nhurt: 0,
            hurt_joint: [0; 16],
            hurt_height: [0; 16],
            hurt_grab: [0; 16],
            hurt_a: [[0.0; 3]; 16],
            hurt_b: [[0.0; 3]; 16],
            hurt_radius: [0.0; 16],
            nown: 0,
            own_clip: vec![0; GPL_MAX_OWN_ROWS],
            row_flags: vec![0; GPL_MAX_ROWS],
            row_blend: vec![[0; 2]; GPL_MAX_ROWS],
            has_blend: vec![0; GPL_MAX_ROWS],
            has_flags: vec![0; GPL_MAX_ROWS],
            row_clip: vec![-1; GPL_MAX_ROWS],
            motion_clip: vec![-1; GPL_MAX_MOTIONS],
        }
    }
}

fn cstr(s: &[u8]) -> Box<[u8]> {
    let mut v = s.to_vec();
    v.push(0);
    v.into_boxed_slice()
}
fn name_of(b: &[u8]) -> &[u8] {
    &b[..b.len() - 1]
}

/// gn_plan_num: an integral number in lo..=hi, else -1.
fn num(d: &Doc, o: i32, key: &str, lo: i64, hi: i64) -> i32 {
    let v = d.get(o, key);
    if v < 0 {
        return -1;
    }
    let n = &d.n[v as usize];
    if n.ty != Ty::Num || n.num < lo as f64 || n.num > hi as f64 || n.num != (n.num as i32) as f64 {
        return -1;
    }
    n.num as i32
}
/// `(unsigned char)d->n[c].num`
fn u8c(x: f64) -> u8 {
    (x as i64) as u8
}

impl Plan {
    fn clip_index(&self, s: &[u8]) -> Option<usize> {
        self.clip_name.iter().position(|c| name_of(c) == s)
    }

    /// gn_plan_parse.
    pub fn parse(d: &Doc, root: i32, who: &str, log: &mut Vec<String>) -> Option<Plan> {
        let mut pl = Plan::default();
        pl.joints = num(d, root, "joint_count", 1, GPL_MAX_JOINTS as i64);
        let parts = d.get(root, "parts");
        macro_rules! fail { ($($t:tt)*) => {{ log.push(format!($($t)*)); return None; }} }
        if pl.joints < 0 || parts < 0 {
            fail!("geno: plan {who}: no joint_count or parts");
        }
        let a = d.get(parts, "joint_to_part");
        if a < 0 || d.n[a as usize].ty != Ty::Arr || d.n[a as usize].count != pl.joints {
            fail!("geno: plan {who}: joint_to_part is not {} long", pl.joints);
        }
        for (i, c) in d.kids(a).into_iter().enumerate() {
            pl.joint_to_part[i] = u8c(d.n[c as usize].num);
        }
        let a = d.get(parts, "part_to_joint");
        if a < 0 || d.n[a as usize].ty != Ty::Arr || d.n[a as usize].count < 1 || d.n[a as usize].count as usize > GPL_MAX_PARTS {
            fail!("geno: plan {who}: part_to_joint size");
        }
        pl.nparts = d.n[a as usize].count;
        for (i, c) in d.kids(a).into_iter().enumerate() {
            pl.part_to_joint[i] = u8c(d.n[c as usize].num);
        }
        let bank = d.get(root, "bank");
        let clips = if bank >= 0 { d.get(bank, "clips") } else { -1 };
        if clips < 0 || d.n[clips as usize].ty != Ty::Obj || d.n[clips as usize].count as usize > GPL_MAX_CLIPS {
            fail!("geno: plan {who}: no bank.clips (run the converter again)");
        }
        for c in d.kids(clips) {
            let sym = d.get(c, "symbol");
            let off = num(d, c, "offset", 0, 0x7fff_ffff);
            let by = num(d, c, "bytes", 1, 0x7fff_ffff);
            let key = &d.n[c as usize].key;
            if sym < 0 || d.n[sym as usize].ty != Ty::Str || off < 0 || by < 0 || key.len() >= 32 || d.n[sym as usize].str.len() >= 72 {
                fail!("geno: plan {who}: bad clip {}", String::from_utf8_lossy(key));
            }
            pl.clip_name.push(cstr(key));
            pl.clip_symbol.push(cstr(&d.n[sym as usize].str));
            pl.clip_offset.push(off as u32);
            pl.clip_bytes.push(by as u32);
            pl.clip_frames.push(num(d, c, "frames", 1, 100000));
        }
        let rows = d.get(root, "motion_rows");
        if rows < 0 || d.n[rows as usize].ty != Ty::Arr || d.n[rows as usize].count as usize > GPL_MAX_MOTIONS {
            fail!("geno: plan {who}: no motion_rows");
        }
        let rc = d.get(root, "row_clips");
        if d.is(rc, Ty::Arr) {
            for (ri, e) in d.kids(rc).into_iter().enumerate().take(GPL_MAX_ROWS) {
                if d.n[e as usize].ty == Ty::Str {
                    if let Some(i) = pl.clip_index(&d.n[e as usize].str) {
                        pl.row_clip[ri] = i as i16;
                    }
                }
            }
        }
        for c in d.kids(rows) {
            let m = num(d, c, "motion", 0, GPL_MAX_MOTIONS as i64 - 1);
            let cl = d.get(c, "clip");
            if m < 0 {
                fail!("geno: plan {who}: bad motion row");
            }
            if d.is(cl, Ty::Str) {
                match pl.clip_index(&d.n[cl as usize].str) {
                    Some(i) => pl.motion_clip[m as usize] = i as i16,
                    None => fail!("geno: plan {who}: motion {m} names clip {}, not in the bank", d.text(cl)),
                }
            }
            if m + 1 > pl.nmotions {
                pl.nmotions = m + 1;
            }
        }
        let rb = d.get(root, "row_blend");
        if d.is(rb, Ty::Arr) {
            for (ri, e) in d.kids(rb).into_iter().enumerate().take(GPL_MAX_ROWS) {
                if d.n[e as usize].ty == Ty::Arr && d.n[e as usize].count == 2 {
                    let k = d.kids(e);
                    pl.row_blend[ri] = [u8c(d.n[k[0] as usize].num), u8c(d.n[k[1] as usize].num)];
                    pl.has_blend[ri] = 1;
                }
            }
        }
        let rd = d.get(root, "row_flags");
        if d.is(rd, Ty::Arr) {
            for (ri, e) in d.kids(rd).into_iter().enumerate().take(GPL_MAX_ROWS) {
                let n = &d.n[e as usize];
                if n.ty == Ty::Num && n.num >= 0.0 && n.num <= 4294967295.0 {
                    pl.row_flags[ri] = (n.num as u32) & !0x3f;
                    pl.has_flags[ri] = 1;
                }
            }
        }
        let lists: [(&str, usize); 6] = [("x8_part_bytes", 5), ("x34_centre", 1), ("x38_coin", 2), ("x44_ecb", 6), ("x54_gfx", 5), ("x58_ik", 10)];
        let fd = d.get(root, "ftdata");
        for (li, &(key, n)) in lists.iter().enumerate() {
            let mut dst = vec![-1i32; n];
            let arr = if fd >= 0 { d.get(fd, key) } else { -1 };
            if !d.is(arr, Ty::Arr) {
                fail!("geno: plan {who}: ftdata.{key} missing");
            }
            let mut k = 0;
            for e in d.kids(arr) {
                if k >= n {
                    break;
                }
                if d.n[e as usize].ty == Ty::Arr {
                    for q in d.kids(e) {
                        if k >= n {
                            break;
                        }
                        dst[k] = num(d, q, "joint", 0, GPL_MAX_JOINTS as i64 - 1);
                        k += 1;
                    }
                } else {
                    dst[k] = num(d, e, "joint", 0, GPL_MAX_JOINTS as i64 - 1);
                    k += 1;
                }
            }
            if let Some(k) = dst.iter().position(|&v| v < 0) {
                fail!("geno: plan {who}: ftdata.{key} entry {k} has no joint (unresolved role)");
            }
            match li {
                0 => pl.ft_x8.copy_from_slice(&dst),
                1 => pl.ft_x34.copy_from_slice(&dst),
                2 => pl.ft_x38.copy_from_slice(&dst),
                3 => pl.ft_x44.copy_from_slice(&dst),
                4 => pl.ft_x54.copy_from_slice(&dst),
                _ => pl.ft_x58.copy_from_slice(&dst),
            }
        }
        let hb = d.get(root, "hurtboxes");
        if hb < 0 || d.n[hb as usize].ty != Ty::Arr || d.n[hb as usize].count < 1 || d.n[hb as usize].count > 15 {
            fail!("geno: plan {who}: hurtboxes must hold 1..15 capsules");
        }
        for a in d.kids(hb) {
            let n = pl.nhurt as usize;
            pl.nhurt += 1;
            let (hv, gv, av, bv, rv) = (d.get(a, "height"), d.get(a, "grabbable"), d.get(a, "a"), d.get(a, "b"), d.get(a, "radius"));
            pl.hurt_joint[n] = num(d, a, "joint", 0, GPL_MAX_JOINTS as i64 - 1);
            let hs = if hv >= 0 { d.n[hv as usize].str.as_slice() } else { b"" };
            pl.hurt_height[n] = if hv >= 0 && hs == b"high" { 2 } else if hv >= 0 && hs == b"mid" { 1 } else { 0 };
            pl.hurt_grab[n] = if gv >= 0 && d.n[gv as usize].ty == Ty::Bool { d.n[gv as usize].num as i32 } else { 1 };
            if pl.hurt_joint[n] < 0 || av < 0 || bv < 0 || rv < 0 || d.n[av as usize].count != 3 || d.n[bv as usize].count != 3 {
                fail!("geno: plan {who}: hurtbox {n} is malformed");
            }
            for (k, q) in d.kids(av).into_iter().take(3).enumerate() {
                pl.hurt_a[n][k] = d.n[q as usize].num as f32;
            }
            for (k, q) in d.kids(bv).into_iter().take(3).enumerate() {
                pl.hurt_b[n][k] = d.n[q as usize].num as f32;
            }
            pl.hurt_radius[n] = d.n[rv as usize].num as f32;
        }
        pl.ok = 1;
        Some(pl)
    }

    /// gw_Geno_DefinePlanInt's switch (the "ok"/no-plan case is the caller's).
    pub fn int(&self, what: i32, i: i32) -> i32 {
        let nc = self.clip_name.len() as i32;
        let r = |n: usize| i >= 0 && (i as usize) < n;
        let iu = i as usize;
        match what {
            0 => self.ok,
            1 => self.joints,
            2 => self.nparts,
            3 => nc,
            4 => self.nmotions,
            10 => if i >= 0 && i < nc { self.clip_offset[iu] as i32 } else { -1 },
            11 => if i >= 0 && i < nc { self.clip_bytes[iu] as i32 } else { -1 },
            12 => if i >= 0 && i < nc { self.clip_frames[iu] } else { -1 },
            13 => if r(GPL_MAX_MOTIONS) { self.motion_clip[iu] as i32 } else { -1 },
            14 => if r(GPL_MAX_ROWS) { self.row_clip[iu] as i32 } else { -1 },
            5 => self.nhurt,
            6 => self.nown,
            15 => if i >= 0 && i < self.nown { self.own_clip[iu] as i32 } else { -1 },
            16 => if r(GPL_MAX_ROWS) && self.has_flags[iu] != 0 { self.row_flags[iu] as i32 } else { 0 },
            17 => if r(GPL_MAX_ROWS) { self.has_flags[iu] as i32 } else { 0 },
            18 => if r(GPL_MAX_ROWS) && self.has_blend[iu] != 0 { self.row_blend[iu][0] as i32 | (self.row_blend[iu][1] as i32) << 8 } else { -1 },
            20 => if r(5) { self.ft_x8[iu] } else { -1 },
            21 => if i == 0 { self.ft_x34[0] } else { -1 },
            22 => if r(2) { self.ft_x38[iu] } else { -1 },
            23 => if r(6) { self.ft_x44[iu] } else { -1 },
            24 => if r(5) { self.ft_x54[iu] } else { -1 },
            25 => if r(10) { self.ft_x58[iu] } else { -1 },
            30 => if i >= 0 && i < self.nhurt { self.hurt_joint[iu] } else { -1 },
            31 => if i >= 0 && i < self.nhurt { self.hurt_height[iu] } else { -1 },
            32 => if i >= 0 && i < self.nhurt { self.hurt_grab[iu] } else { -1 },
            40 => {
                let (h, c) = (i / 8, i % 8);
                if i < 0 || h >= self.nhurt || c > 6 {
                    return 0;
                }
                let h = h as usize;
                let v = if c < 3 { self.hurt_a[h][c as usize] } else if c < 6 { self.hurt_b[h][c as usize - 3] } else { self.hurt_radius[h] };
                v.to_bits() as i32
            }
            _ => -1,
        }
    }

    /// gw_Geno_DefinePlanText: 0 clip name, 1 clip symbol (NUL-terminated), 2 joint_to_part, 3 part_to_joint.
    pub fn text(&self, which: i32, i: i32) -> Option<*const u8> {
        let nc = self.clip_name.len() as i32;
        match which {
            0 if i >= 0 && i < nc => Some(self.clip_name[i as usize].as_ptr()),
            1 if i >= 0 && i < nc => Some(self.clip_symbol[i as usize].as_ptr()),
            2 => Some(self.joint_to_part.as_ptr()),
            3 => Some(self.part_to_joint.as_ptr()),
            _ => None,
        }
    }
}

impl Registry {
    /// gn_define_plan: the parsed plan of a base "none" define (read from the mod's files/ on first use), None when the
    /// kind is not one or the plan is unusable (logged).
    pub fn define_plan(&mut self, kind: i32, env: &dyn Env) -> Option<&Plan> {
        let p = self.definitions.profile_for_kind(kind);
        if p < 0 {
            return None;
        }
        let p = p as usize;
        let mod_id = self.p[p].mod_id.clone();
        let mut log = Vec::new();
        {
            let fm = self.p[p].def.fm.as_deref_mut()?;
            if !fm.plan_tried {
                fm.plan_tried = true;
                let name = fm.plan_file.clone();
                match env.read_mod_file(&mod_id, &format!("files/{name}")).or_else(|| env.read_mod_file(&mod_id, &name)) {
                    None => log.push(format!("geno: plan file {name} not found in the mod's files/")),
                    Some(raw) => {
                        let (d, root) = Doc::parse(&raw);
                        let mut pl = if root >= 0 {
                            Plan::parse(&d, root, &name, &mut log)
                        } else {
                            log.push(format!("geno: plan {name}: {}", if d.err.is_empty() { "bad JSON" } else { d.err }));
                            None
                        };
                        if let Some(pl2) = pl.as_mut() {
                            let mut ok = true;
                            for (k, oc) in fm.own_clip.iter().enumerate() {
                                match pl2.clip_index(oc.as_bytes()) {
                                    Some(i) => pl2.own_clip[k] = i as i16,
                                    None => {
                                        log.push(format!("geno: plan {name}: fighter.rows[{k}] names clip {oc}, not in the bank"));
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            pl2.nown = fm.own_clip.len() as i32;
                            if !ok {
                                pl = None;
                            }
                        }
                        if let Some(pl2) = pl.as_ref() {
                            log.push(format!(
                                "geno: plan {name}: {} joints, {} parts, {} clips, {} motion rows",
                                pl2.joints, pl2.nparts, pl2.clip_name.len(), pl2.nmotions
                            ));
                        }
                        fm.plan = pl.map(Box::new);
                    }
                }
            }
        }
        self.log.extend(log);
        self.p[p].def.fm.as_deref().and_then(|f| f.plan.as_deref())
    }
}
