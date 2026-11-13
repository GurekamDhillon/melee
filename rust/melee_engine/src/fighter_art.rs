//! Fighter icons and portraits for the presentation layer (the Envoy picker, the camp), read from the player's own
//! character-select file at boot.
//!
//! [`build`] reads `MnSlChr` from the mounted disc (a mod's copy when one overrides it), decodes it with the engine's
//! archive decoder into host memory and walks the *descriptors* (joints, material animations, texture animations):
//! no game object is created, no game heap is touched and nothing here is read back by the sim, so it cannot change a
//! state hash. The result is a list of RGBA pictures per character kind, handed to the UI host
//! (`melee_modhost::ui::fighter_art`). Nothing derived from the disc is stored anywhere: it is decoded at each boot.
//!
//! What is read (the same rules as `mnCharSel_PcArtIcon` / `mnCharSel_PcArtPortrait`, which do it on live objects):
//! - retail: the menu model's icon joints (`icons[]`: joint, kind) and the first door's portrait joint, whose
//!   texture animation holds every portrait (frame = the icon's HUD index + costume * 30);
//! - an m-ex disc (`mexSelectChr` in the file, a CSS icon table in MxDt.dat): the m-ex icon model, and the m-ex
//!   portrait animation at frame = external id + costume * stride.


use melee_modhost::ui::fighter_art::{Art, Pic};
use melee_types::*;
use std::vec::Vec;

/// HSD track ids of a texture animation.
const TIMG: u8 = 1;
const TCLT: u8 = 10;

/// `fobj.c`'s interpreter over one track, as far as a texture pick needs it (KEY / CON / LIN; a spline returns its
/// start value): the value the object is given by `ReqAnim(frame)`, one `InterpretAnim(0)` and the `StopAnim` flush.
struct Track<'a> {
    ad: &'a [u8],
    pos: usize,
    time: f32,
    op: u8,
    op_intrp: u8,
    flags: u8,
    nb_pack: u32,
    fterm: u16,
    p0: f32,
    p1: f32,
    d0: f32,
    state: u32,
    frac: u8,
    frac_slope: u8,
    out: Option<f32>,
}

impl<'a> Track<'a> {
    fn byte(&mut self) -> u8 {
        let b = self.ad.get(self.pos).copied().unwrap_or(0);
        self.pos += 1;
        b
    }
    fn float(&mut self, frac: u8) -> f32 {
        if frac == 0 {
            let b = [self.byte(), self.byte(), self.byte(), self.byte()];
            return f32::from_le_bytes(b);
        }
        let denom = (1u32 << (frac & 0x1F)) as f32;
        let n = match frac & 0xE0 {
            0x60 => self.byte() as i8 as f32,
            0x80 => self.byte() as f32,
            0x20 => i16::from_le_bytes([self.byte(), self.byte()]) as f32,
            0x40 => u16::from_le_bytes([self.byte(), self.byte()]) as f32,
            _ => return 0.0,
        };
        n / denom
    }
    fn update(&mut self) {
        match self.op_intrp {
            6 => {
                if self.flags & 0x80 != 0 {
                    self.out = Some(self.p0);
                    self.flags &= 0x7F;
                }
            }
            1 => self.out = Some(if self.time >= self.fterm as f32 { self.p1 } else { self.p0 }),
            2 => {
                if self.flags & 0x20 != 0 {
                    self.flags &= 0xDF;
                    if self.fterm != 0 {
                        self.d0 = (self.p1 - self.p0) / self.fterm as f32;
                    } else {
                        self.d0 = 0.0;
                        self.p0 = self.p1;
                    }
                }
                self.out = Some(self.d0 * self.time + self.p0);
            }
            3 | 4 | 5 => self.out = Some(if self.fterm != 0 { self.p0 } else { self.p1 }),
            _ => {}
        }
    }
    fn launch_key(&mut self) {
        if self.flags & 0x40 != 0 {
            self.op_intrp = self.op;
            self.flags &= !0x40;
            self.flags |= 0x80;
            self.p0 = self.p1;
        }
    }
    fn load_data(&mut self) -> u32 {
        if self.pos >= self.ad.len() {
            return 6;
        }
        self.op_intrp = self.op;
        if self.nb_pack == 0 {
            self.op = self.ad[self.pos] & 0xF;
            let mut d = self.byte();
            let mut n = (((d >> 4) & 7) + 1) as u32;
            let mut shift = 3;
            while d & 0x80 != 0 {
                d = self.byte();
                n = n.wrapping_add(((d & 0x7F) as u32).wrapping_shl(shift));
                shift += 7;
            }
            self.nb_pack = n;
        }
        self.nb_pack -= 1;
        let next = if self.state == 1 { 3 } else { 4 };
        match self.op {
            1 | 2 | 3 => {
                self.p0 = self.p1;
                self.p1 = self.float(self.frac);
                next
            }
            4 => {
                self.p0 = self.p1;
                self.p1 = self.float(self.frac);
                let _ = self.float(self.frac_slope);
                next
            }
            5 => {
                let _ = self.float(self.frac_slope);
                self.state
            }
            6 => {
                self.launch_key();
                self.p1 = self.float(self.frac);
                self.flags |= 0x40;
                next
            }
            _ => 0,
        }
    }
    fn interpret(&mut self, rate: f32) {
        let mut fterm = 0.0f32;
        if self.state == 0 {
            return;
        }
        self.time += rate;
        if self.time < 0.0 {
            return;
        }
        // a bound on the walk: a damaged stream must not spin
        for _ in 0..200_000 {
            match self.state {
                6 => {
                    self.time += fterm;
                    self.launch_key();
                    self.update();
                    return;
                }
                1 | 2 => {
                    let s = self.load_data();
                    self.state = s;
                }
                3 => {
                    if self.flags & 0x80 != 0 {
                        self.update();
                    }
                    if self.pos >= self.ad.len() {
                        self.state = 6;
                    } else {
                        let (mut wait, mut shift) = (0u32, 0u32);
                        loop {
                            let d = self.byte();
                            wait |= ((d & 0x7F) as u32).wrapping_shl(shift);
                            shift += 7;
                            if d & 0x80 == 0 {
                                break;
                            }
                        }
                        self.fterm = wait as u16;
                        self.flags |= 0x20;
                        self.state = 2;
                    }
                }
                4 => {
                    if self.fterm as f32 <= self.time {
                        self.state = 3;
                        fterm = self.fterm as f32;
                        self.time -= self.fterm as f32;
                        continue;
                    }
                    self.update();
                    self.state = 5;
                    return;
                }
                5 => self.state = 4,
                _ => return,
            }
        }
    }
}

/// The value track `desc` gives its object at animation frame `frame` (None: the track never updates it there).
unsafe fn track_value(desc: *const _HSD_FObjDesc, frame: f32) -> Option<f32> {
    let d = &*desc;
    if d.ad.is_null() || d.length == 0 || d.length > (1 << 24) {
        return None;
    }
    let ad = core::slice::from_raw_parts(d.ad as *const u8, d.length as usize);
    let start = d.startframe as i16 as f32;
    let mut t = Track { ad, pos: 0, time: start + frame, op: 0, op_intrp: 0, flags: 0, nb_pack: 0, fterm: 0, p0: 0.0, p1: 0.0, d0: 0.0, state: 1, frac: d.frac_value, frac_slope: d.frac_slope, out: None };
    t.interpret(0.0);
    if t.op_intrp == 6 {
        t.interpret(1.0); // StopAnim flushes a pending key
    }
    t.out
}

/// The image and palette a texture shows: its own, or the pick of its texture animation at `frame`.
unsafe fn tex_at(t: *const _HSD_TObjDesc, anim: *const _HSD_TexAnim, frame: f32) -> (*const HSD_ImageDesc, *const _HSD_TlutDesc) {
    let mut img = (*t).imagedesc as *const HSD_ImageDesc;
    let mut tlut = (*t).tlutdesc as *const _HSD_TlutDesc;
    let mut a = anim;
    while !a.is_null() {
        if (*a).id == (*t).id && !(*a).aobjdesc.is_null() {
            let mut f = (*(*a).aobjdesc).fobjdesc as *const _HSD_FObjDesc;
            let mut n = 0;
            while !f.is_null() && n < 64 {
                if let Some(v) = track_value(f, frame) {
                    let i = v as i32;
                    if (*f).r#type == TIMG && !(*a).imagetbl.is_null() && i >= 0 && i < (*a).n_imagetbl as i32 {
                        let p = *(*a).imagetbl.add(i as usize);
                        if !p.is_null() {
                            img = p;
                        }
                    } else if (*f).r#type == TCLT && !(*a).tluttbl.is_null() && i >= 0 && i < (*a).n_tluttbl as i32 {
                        let p = *(*a).tluttbl.add(i as usize);
                        if !p.is_null() {
                            tlut = p;
                        }
                    }
                }
                f = (*f).next;
                n += 1;
            }
        }
        a = (*a).next;
    }
    (img, tlut)
}

/// A joint tree in the order `lb_80011E24` numbers it (a joint, its children, then its siblings), with each
/// joint's material animation (the parallel tree; null without one) and the size of its subtree.
struct Tree {
    joints: Vec<(*const HSD_Joint, *const HSD_MatAnimJoint, usize)>,
}

impl Tree {
    unsafe fn new(root: *const HSD_Joint, anim: *const HSD_MatAnimJoint) -> Tree {
        let mut t = Tree { joints: Vec::new() };
        t.walk(root, anim, 0);
        t
    }
    unsafe fn walk(&mut self, mut j: *const HSD_Joint, mut a: *const HSD_MatAnimJoint, depth: u32) {
        while !j.is_null() && depth < 64 && self.joints.len() < 4096 {
            let at = self.joints.len();
            self.joints.push((j, a, 1));
            self.walk((*j).child, if a.is_null() { a } else { (*a).child }, depth + 1);
            self.joints[at].2 = self.joints.len() - at;
            j = (*j).next;
            a = if a.is_null() { a } else { (*a).next };
        }
    }
    /// The largest texture joint `id` and its children show at `frame` (`mnCharSel_PcBestTex`); `over` replaces
    /// the joint's own material animation list (m-ex's portrait animation).
    unsafe fn best(&self, id: usize, frame: f32, over: *const _HSD_MatAnim) -> Option<Pic> {
        let &(_, _, size) = self.joints.get(id)?;
        let mut best: Option<(i32, *const HSD_ImageDesc, *const _HSD_TlutDesc)> = None;
        for (k, &(j, a, _)) in self.joints[id..id + size].iter().enumerate() {
            // a joint that carries a spline or particles has no meshes
            if (*j).flags & 0x4020 != 0 {
                continue;
            }
            let mut d = (*j).u.dobjdesc as *const HSD_DObjDesc;
            let mut m: *const _HSD_MatAnim = if k == 0 && !over.is_null() {
                over
            } else if a.is_null() {
                core::ptr::null()
            } else {
                (*a).matanim
            };
            let mut n = 0;
            while !d.is_null() && n < 256 {
                let mo = (*d).mobjdesc;
                let mut t = if mo.is_null() { core::ptr::null() } else { (*mo).texdesc as *const _HSD_TObjDesc };
                let ta = if m.is_null() { core::ptr::null() } else { (*m).texanim as *const _HSD_TexAnim };
                let mut tn = 0;
                while !t.is_null() && tn < 16 {
                    let (img, tlut) = tex_at(t, ta, frame);
                    if !img.is_null() && !(*img).image_ptr.is_null() {
                        let area = (*img).width as i32 * (*img).height as i32 * if (*img).format >= 4 { 4 } else { 1 };
                        if best.map_or(true, |b| area > b.0) {
                            best = Some((area, img, tlut));
                        }
                    }
                    t = (*t).next;
                    tn += 1;
                }
                d = (*d).next;
                m = if m.is_null() { m } else { (*m).next };
                n += 1;
            }
        }
        let (_, img, tlut) = best?;
        pic(img, tlut)
    }
}

/// Decode one image (and its palette) to RGBA.
unsafe fn pic(img: *const HSD_ImageDesc, tlut: *const _HSD_TlutDesc) -> Option<Pic> {
    use melee_gx::texture as tx;
    let i = &*img;
    let (w, h, fmt) = (i.width as u32, i.height as u32, i.format as u32);
    if w == 0 || h == 0 || w > 1024 || h > 1024 {
        return None;
    }
    let n = tx::texture_source_size(fmt, w, h, 1);
    if n == 0 {
        return None;
    }
    let data = core::slice::from_raw_parts(i.image_ptr as *const u8, n);
    let rgba = if matches!(fmt, 8 | 9 | 10) {
        if tlut.is_null() || (*tlut).lut.is_null() || (*tlut).fmt as u32 > 2 {
            return None;
        }
        let e = (*tlut).n_entries;
        let pal = core::slice::from_raw_parts((*tlut).lut as *const u8, tx::tlut_source_size(e));
        tx::convert_texture_palette(fmt, w, h, 1, data, (*tlut).fmt as u32, e, pal).data
    } else {
        tx::convert_texture(fmt, w, h, 1, data).data
    };
    (rgba.len() == (w * h * 4) as usize).then(|| Pic { w, h, rgba })
}

/// Read the character-select art of every fighter the select screen lists. Empty when the file is not there or does
/// not parse. Call it with the game arena off (harness code): everything here lives on the system heap.
pub fn build() -> Vec<Art> {
    let t0 = std::time::Instant::now();
    if !crate::host::Host::is_booted() {
        return Vec::new();
    }
    let disc = &mut crate::host::Host::get().disc;
    let Some(mut file) = ["MnSlChr.usd", "MnSlChr.dat"].iter().find_map(|n| disc.read(n).ok()) else {
        eprintln!("fighter art: no MnSlChr on the disc");
        return Vec::new();
    };
    if file.len() < 0x40 || u32::from_be_bytes(file[0..4].try_into().unwrap()) as usize != file.len() {
        eprintln!("fighter art: MnSlChr has an inconsistent header");
        return Vec::new();
    }
    let mut out: Vec<Art> = Vec::new();
    unsafe {
        let dec = crate::archive::decode(file.as_mut_ptr(), file.len());
        let tbl = dec.native_root("MnSelectChrDataTable").unwrap_or(core::ptr::null_mut()) as *const MnSelectChrDataTable;
        if tbl.is_null() {
            eprintln!("fighter art: MnSlChr has no MnSelectChrDataTable");
            return Vec::new();
        }
        let menu = Tree::new((*tbl).models.menu.joint, (*tbl).models.menu.matanim_joint);
        let (rows, costume_joint) = melee_modes::tu::mncharsel::retail_art_rows();
        let mex = dec.native_root("mexSelectChr").unwrap_or(core::ptr::null_mut()) as *const MexSelectChr;
        let n = crate::mex_host::CssIconCount();
        let mtab = crate::mex_host::CssIconTable() as *const CSSIcon;
        if !mex.is_null() && n > 0 && !mtab.is_null() {
            let icons = Tree::new((*mex).icon_joint, (*mex).icon_matanim);
            let end = if !(*mex).csp_matanim.is_null() && !(*(*mex).csp_matanim).texanim.is_null() && !(*(*(*mex).csp_matanim).texanim).aobjdesc.is_null() {
                (*(*(*(*mex).csp_matanim).texanim).aobjdesc).end_frame as i32
            } else {
                0
            };
            for i in 0..n as usize {
                let r = &*mtab.add(i);
                let ext = r.char_kind as i32;
                let ck = crate::mex_host::ExtToPortCKind(ext);
                if ck < 0 || out.iter().any(|a| a.ck == ck) {
                    continue;
                }
                let icon = icons.best(r.joint_id_vs as usize, 0.0, core::ptr::null());
                // costume 0's frame is the external id (mnCharSel_MexCspFrame); past the animation: no portrait
                let portrait = if end > 0 && ext > end { None } else { menu.best(costume_joint as usize, ext as f32, (*mex).csp_matanim) };
                out.push(Art { ck, icon, portrait });
            }
        } else {
            for &(ck, hud, joint) in rows.iter() {
                let icon = menu.best(joint as usize, 0.0, core::ptr::null());
                let portrait = menu.best(costume_joint as usize, hud as f32, core::ptr::null());
                out.push(Art { ck: ck as i32, icon, portrait });
            }
        }
        drop(dec);
    }
    let (ni, np) = (out.iter().filter(|a| a.icon.is_some()).count(), out.iter().filter(|a| a.portrait.is_some()).count());
    eprintln!("fighter art: {} fighters, {ni} icons, {np} portraits from MnSlChr in {:.1} ms", out.len(), t0.elapsed().as_secs_f64() * 1e3);
    out
}

/// `build`, handed to the UI host.
pub fn build_into_ui() {
    melee_modhost::ui::fighter_art::set(build());
}
