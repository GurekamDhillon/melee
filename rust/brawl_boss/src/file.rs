//! The boss parameter file: header, relocations, exports, imports, the boss record and everything it points at
//! (format spec section 2). All offsets inside the data are data-section offsets (file offset - 0x20).

use crate::error::{be32, get, Error, Result};
use crate::fx::Fx;
use crate::params::Params;
use crate::script::{Arg, Command, Op, Script, Target};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

/// Header size.
pub const HEADER: usize = 0x20;
/// Words in the boss record (`data` export).
pub const RECORD_WORDS: usize = 28;
/// Status table slots: statuses 0-15 plus the common slot 16.
pub const STATUS_SLOTS: usize = 17;
/// The common status slot.
pub const COMMON_SLOT: usize = 16;
/// Timeline categories and threads per category.
pub const CATEGORIES: usize = 3;
pub const THREADS: usize = 3;
/// Longest script decoded.
pub const MAX_SCRIPT_CMDS: usize = 4096;
/// Most motions / rules a table may hold.
pub const MAX_TABLE: usize = 256;
/// Sizes over this are rejected (Petey's file is about 28 KB).
pub const MAX_FILE: usize = 16 << 20;

/// Header (0x20 bytes, big-endian).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub file_size: u32,
    pub data_size: u32,
    pub reloc_count: u32,
    pub root_count: u32,
    pub import_count: u32,
}

impl Header {
    pub fn parse(d: &[u8]) -> Result<Header> {
        let h = Header {
            file_size: be32(d, 0)?,
            data_size: be32(d, 4)?,
            reloc_count: be32(d, 8)?,
            root_count: be32(d, 12)?,
            import_count: be32(d, 16)?,
        };
        if h.file_size as usize > MAX_FILE || h.file_size as usize > d.len() {
            return Err(Error::BadHeader("file size"));
        }
        let tail = (h.reloc_count as u64) * 4 + (h.root_count as u64 + h.import_count as u64) * 8;
        if HEADER as u64 + h.data_size as u64 + tail > h.file_size as u64 {
            return Err(Error::BadHeader("sections exceed the file"));
        }
        Ok(h)
    }
    pub fn strings_at(&self) -> usize {
        HEADER + self.data_size as usize + self.reloc_count as usize * 4 + (self.root_count + self.import_count) as usize * 8
    }
}

/// An exported (root) name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    pub name: String,
    pub at: u32,
}

/// An imported common script: its name and every field that refers to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub name: String,
    pub fields: Vec<u32>,
}

/// A motion table entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Motion {
    pub flags: u32,
    pub name: String,
}
impl Motion {
    /// [L] 0x00020000 = loop.
    pub const FLAG_LOOP: u32 = 0x0002_0000;
    /// [L] 0x00040000 = motion-driven translation.
    pub const FLAG_TRANSN: u32 = 0x0004_0000;
    pub fn looping(&self) -> bool {
        self.flags & Self::FLAG_LOOP != 0
    }
}

/// A 24-byte rule record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub at: u32,
    /// Lower runs first.
    pub priority: i32,
    /// 0x10 = no target, evaluated once; 0x0F = once per present player slot 0-3.
    pub target_mask: u32,
    pub condition: Target,
    pub action: Target,
    /// -1 = none.
    pub target_status: i32,
    pub default_enabled: bool,
    pub lock: bool,
    /// Bytes +20..+24 as stored.
    pub flag_bytes: [u8; 4],
}
impl Rule {
    pub const MASK_NO_TARGET: u32 = 0x10;
    pub fn per_slot(&self) -> bool {
        self.target_mask & 0x0f != 0
    }
}

/// A hurtbox (32 bytes): offset, stretch, radius, flags; bone = flags >> 23.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hurtbox {
    pub offset: [Fx; 3],
    pub stretch: [Fx; 3],
    pub radius: Fx,
    pub flags: u32,
}
impl Hurtbox {
    pub fn bone(&self) -> u32 {
        self.flags >> 23
    }
}

/// The decoded file.
#[derive(Clone, Debug)]
pub struct BossFile {
    pub header: Header,
    data: Vec<u8>,
    relocs: BTreeSet<u32>,
    /// Every relocated pointer's target: where objects start.
    targets: BTreeSet<u32>,
    import_at: BTreeMap<u32, u16>,
    pub roots: Vec<Root>,
    pub imports: Vec<Import>,
    pub record_at: u32,
    pub record: [u32; RECORD_WORDS],
    pub motions: Vec<Motion>,
    /// `[category][thread][motion]`; categories game, effect, sound.
    pub timelines: [[Vec<Target>; THREADS]; CATEGORIES],
    pub status_loop: [Target; STATUS_SLOTS],
    pub status_exit: [Target; STATUS_SLOTS],
    pub status_enter: [Target; STATUS_SLOTS],
    pub rules: Vec<Rule>,
    pub init: Target,
    pub params: Params,
    /// The general block (data 0x00, 44 words) and the tuning block (record word 7, 44 words), raw.
    pub general: [u32; 44],
    pub tuning: [u32; 44],
    pub body_hurtboxes: Vec<Hurtbox>,
    pub cage_hurtboxes: [Vec<Hurtbox>; 2],
    /// `s_wpnAnmCmdDataSet`: the cages' small command arrays (count-terminated, not zero-terminated).
    pub cage_cmd_sets: Vec<Vec<Command>>,
    /// Every script reachable from the tables, rules, init, the weighted lists and other scripts, by offset.
    pub scripts: BTreeMap<u32, Script>,
}

/// Categories.
pub const GAME: usize = 0;
pub const EFFECT: usize = 1;
pub const SOUND: usize = 2;

impl BossFile {
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn is_reloc(&self, field: u32) -> bool {
        self.relocs.contains(&field)
    }
    pub fn reloc_count(&self) -> usize {
        self.relocs.len()
    }
    pub fn word(&self, at: u32) -> Result<u32> {
        be32(&self.data, at as usize)
    }
    pub fn float(&self, at: u32) -> Result<Fx> {
        Ok(Fx::from_f32_bits(self.word(at)?))
    }
    pub fn import_name(&self, i: u16) -> Option<&str> {
        self.imports.get(i as usize).map(|x| x.name.as_str())
    }
    pub fn root(&self, name: &str) -> Option<u32> {
        self.roots.iter().find(|r| r.name == name).map(|r| r.at)
    }
    pub fn script(&self, at: u32) -> Option<&Script> {
        self.scripts.get(&at)
    }
    pub fn tuning_fx(&self, word: usize) -> Fx {
        Fx::from_f32_bits(self.tuning.get(word).copied().unwrap_or(0))
    }
    pub fn general_fx(&self, word: usize) -> Fx {
        Fx::from_f32_bits(self.general.get(word).copied().unwrap_or(0))
    }
    /// Body HP (tuning word 0).
    pub fn body_hp(&self) -> Fx {
        self.tuning_fx(0)
    }
    /// Target-slot weights P1-P4 (tuning words 2-5).
    pub fn slot_weights(&self) -> [Fx; 4] {
        [self.tuning_fx(2), self.tuning_fx(3), self.tuning_fx(4), self.tuning_fx(5)]
    }
    /// Slot repeat factor (tuning +0x74).
    pub fn slot_repeat_factor(&self) -> Fx {
        self.tuning_fx(0x74 / 4)
    }
    /// Param 3056, the bounce factor (general +0x08).
    pub fn bounce_factor(&self) -> Fx {
        self.general_fx(2)
    }
    /// Param 3057, the death threshold (general +0x9C).
    pub fn death_threshold(&self) -> Fx {
        self.general_fx(0x9c / 4)
    }

    /// A pointer field: import, relocated pointer, null, or an unresolved value.
    pub fn ptr_field(&self, field: u32) -> Result<Target> {
        if let Some(&i) = self.import_at.get(&field) {
            return Ok(Target::Import(i));
        }
        let v = self.word(field)?;
        Ok(if self.relocs.contains(&field) {
            if v as usize >= self.data.len() {
                return Err(Error::BadOffset { what: "pointer", at: field });
            }
            Target::Local(v)
        } else if v == 0 {
            Target::None
        } else {
            Target::Unresolved(v)
        })
    }
    fn local(&self, field: u32) -> Result<u32> {
        match self.ptr_field(field)? {
            Target::Local(o) => Ok(o),
            _ => Err(Error::NotAPointer { field }),
        }
    }

    /// Decode a boss parameter file.
    pub fn parse(file: &[u8]) -> Result<BossFile> {
        let header = Header::parse(file)?;
        let file = &file[..header.file_size as usize];
        let data = get(file, HEADER, header.data_size as usize)?.to_vec();
        let dsz = data.len() as u32;
        let rel_at = HEADER + data.len();
        let mut relocs = BTreeSet::new();
        for i in 0..header.reloc_count as usize {
            let r = be32(file, rel_at + 4 * i)?;
            if r.checked_add(4).map_or(true, |e| e > dsz) || r % 4 != 0 {
                return Err(Error::BadOffset { what: "relocation", at: r });
            }
            relocs.insert(r);
        }
        let mut targets = BTreeSet::new();
        for &r in &relocs {
            targets.insert(be32(&data, r as usize)?);
        }
        let rec_at = rel_at + 4 * header.reloc_count as usize;
        let str_at = header.strings_at();
        let name = |off: u32| -> Result<String> {
            let s = str_at.checked_add(off as usize).ok_or(Error::BadOffset { what: "string", at: off })?;
            let tail = file.get(s..).ok_or(Error::BadOffset { what: "string", at: off })?;
            let end = tail.iter().position(|&c| c == 0).ok_or(Error::BadOffset { what: "string", at: off })?;
            Ok(String::from_utf8_lossy(&tail[..end]).into_owned())
        };
        let mut roots = Vec::new();
        let mut imports = Vec::new();
        let mut import_at = BTreeMap::new();
        for i in 0..(header.root_count + header.import_count) as usize {
            let at = be32(file, rec_at + 8 * i)?;
            let nm = name(be32(file, rec_at + 8 * i + 4)?)?;
            if i < header.root_count as usize {
                if at >= dsz {
                    return Err(Error::BadOffset { what: "root", at });
                }
                roots.push(Root { name: nm, at });
            } else {
                let idx = imports.len() as u16;
                let mut fields = Vec::new();
                let mut f = at;
                while f != 0xffff_ffff {
                    if f.checked_add(4).map_or(true, |e| e > dsz) || fields.len() > 4096 || import_at.contains_key(&f) {
                        return Err(Error::BadOffset { what: "import chain", at: f });
                    }
                    fields.push(f);
                    import_at.insert(f, idx);
                    f = be32(&data, f as usize)?;
                }
                imports.push(Import { name: nm, fields });
            }
        }
        let mut bf = BossFile {
            header,
            data,
            relocs,
            targets,
            import_at,
            roots,
            imports,
            record_at: 0,
            record: [0; RECORD_WORDS],
            motions: Vec::new(),
            timelines: Default::default(),
            status_loop: [Target::None; STATUS_SLOTS],
            status_exit: [Target::None; STATUS_SLOTS],
            status_enter: [Target::None; STATUS_SLOTS],
            rules: Vec::new(),
            init: Target::None,
            params: Params::default(),
            general: [0; 44],
            tuning: [0; 44],
            body_hurtboxes: Vec::new(),
            cage_hurtboxes: Default::default(),
            cage_cmd_sets: Vec::new(),
            scripts: BTreeMap::new(),
        };
        bf.decode_record()?;
        bf.collect_scripts()?;
        Ok(bf)
    }

    fn table_len(&self, start: u32, stride: u32, ptr_field: u32) -> usize {
        // a table ends where another object starts (some pointer targets the next entry), or where its pointer
        // field stops being relocated
        let mut n = 0usize;
        while n < MAX_TABLE {
            let e = start + n as u32 * stride;
            if n > 0 && self.targets.contains(&e) {
                break;
            }
            if !self.relocs.contains(&(e + ptr_field)) {
                break;
            }
            n += 1;
        }
        n
    }

    fn decode_record(&mut self) -> Result<()> {
        let rec = self.root("data").ok_or(Error::BadHeader("no `data` export"))?;
        self.record_at = rec;
        for i in 0..RECORD_WORDS {
            self.record[i] = self.word(rec + 4 * i as u32)?;
        }
        let w = |i: usize| rec + 4 * i as u32;
        // motions
        let mt = self.local(w(0))?;
        let nm = self.table_len(mt, 8, 4);
        for i in 0..nm as u32 {
            let flags = self.word(mt + 8 * i)?;
            let s = self.local(mt + 8 * i + 4)? as usize;
            let tail = self.data.get(s..).unwrap_or(&[]);
            let end = tail.iter().position(|&c| c == 0).ok_or(Error::BadOffset { what: "motion name", at: s as u32 })?;
            let name = String::from_utf8_lossy(&tail[..end]).into_owned();
            self.motions.push(Motion { flags, name });
        }
        // timelines: words 10..=18
        for c in 0..CATEGORIES {
            for t in 0..THREADS {
                let field = w(10 + 3 * c + t);
                let tab = match self.ptr_field(field)? {
                    Target::Local(o) => o,
                    _ => continue,
                };
                let mut v = Vec::with_capacity(nm);
                for m in 0..nm as u32 {
                    v.push(self.ptr_field(tab + 4 * m)?);
                }
                self.timelines[c][t] = v;
            }
        }
        // status tables: 19 loop, 20 exit, 21 enter
        for (word, which) in [(19usize, 0usize), (20, 1), (21, 2)] {
            let tab = self.local(w(word))?;
            for s in 0..STATUS_SLOTS {
                let t = self.ptr_field(tab + 4 * s as u32)?;
                match which {
                    0 => self.status_loop[s] = t,
                    1 => self.status_exit[s] = t,
                    _ => self.status_enter[s] = t,
                }
            }
        }
        // rules
        let rt = self.local(w(22))?;
        let nr = self.table_len(rt, 4, 0);
        for i in 0..nr as u32 {
            let at = self.local(rt + 4 * i)?;
            let fb = get(&self.data, at as usize + 20, 4)?;
            let flag_bytes = [fb[0], fb[1], fb[2], fb[3]];
            self.rules.push(Rule {
                at,
                priority: self.word(at)? as i32,
                target_mask: self.word(at + 4)?,
                condition: self.ptr_field(at + 8)?,
                action: self.ptr_field(at + 12)?,
                target_status: self.word(at + 16)? as i32,
                default_enabled: flag_bytes[0] != 0,
                lock: flag_bytes[1] != 0,
                flag_bytes,
            });
        }
        self.init = self.ptr_field(w(24))?;
        // general / tuning blocks
        for i in 0..44 {
            self.general[i] = self.word(4 * i as u32).unwrap_or(0);
        }
        if let Ok(tb) = self.local(w(7)) {
            for i in 0..44 {
                self.tuning[i] = self.word(tb + 4 * i as u32).unwrap_or(0);
            }
        }
        // parameter block
        let pb = self.local(w(8))?;
        self.params = Params::decode(self, pb)?;
        // hurtboxes
        let hd = |bf: &BossFile, root: &str| -> Result<Vec<Hurtbox>> {
            let Some(r) = bf.root(root) else { return Ok(Vec::new()) };
            let list = bf.local(r)?;
            let n = bf.word(r + 4)?.min(64);
            let mut v = Vec::new();
            for i in 0..n {
                let h = list + 32 * i;
                let f = |k: u32| bf.float(h + 4 * k);
                v.push(Hurtbox {
                    offset: [f(0)?, f(1)?, f(2)?],
                    stretch: [f(3)?, f(4)?, f(5)?],
                    radius: f(6)?,
                    flags: bf.word(h + 28)?,
                });
            }
            Ok(v)
        };
        self.body_hurtboxes = hd(self, "HitDataSet")?;
        self.cage_hurtboxes = [hd(self, "wpnHitDataSet_ArmLeft")?, hd(self, "wpnHitDataSet_ArmRight")?];
        // the cages' command sets: {ptr, count} pairs until the next object. Their layout is only [M] in the spec,
        // so a set that does not decode is left out rather than failing the file.
        if let Some(r) = self.root("s_wpnAnmCmdDataSet") {
            let n = self.table_len(r, 8, 0);
            for i in 0..n as u32 {
                let set = (|| -> Result<Vec<Command>> {
                    let p = self.local(r + 8 * i)?;
                    let c = self.word(r + 8 * i + 4)?.min(64);
                    (0..c).map(|k| self.decode_cmd(p + 8 * k)).collect()
                })();
                if let Ok(v) = set {
                    self.cage_cmd_sets.push(v);
                }
            }
        }
        Ok(())
    }

    fn decode_cmd(&self, at: u32) -> Result<Command> {
        let word = self.word(at)?;
        let n = (word >> 8) & 0xff;
        let mut args = Vec::with_capacity(n as usize);
        if n > 0 {
            let ap = self.local(at + 4)?;
            for i in 0..n {
                let a = ap + 8 * i;
                let ty = self.word(a)?;
                let v = self.word(a + 4)?;
                let mut arg = Arg::from_pair(ty, v);
                if ty == 2 {
                    arg = Arg::Ptr(self.ptr_field(a + 4)?);
                }
                args.push(arg);
            }
        }
        Ok(Command::decode(at, word, args))
    }

    /// Decode the script at `at` (until the all-zero command).
    pub fn decode_script(&self, at: u32) -> Result<Script> {
        let mut cmds = Vec::new();
        let mut p = at;
        loop {
            if cmds.len() >= MAX_SCRIPT_CMDS {
                return Err(Error::BadScript { at });
            }
            let w = self.word(p).map_err(|_| Error::BadScript { at })?;
            let a = self.word(p + 4).map_err(|_| Error::BadScript { at })?;
            if w == 0 && a == 0 {
                break;
            }
            cmds.push(self.decode_cmd(p)?);
            p += 8;
        }
        Ok(Script { at, cmds })
    }

    fn collect_scripts(&mut self) -> Result<()> {
        let mut work: Vec<u32> = Vec::new();
        let mut push = |t: &Target| {
            if let Target::Local(o) = *t {
                work.push(o);
            }
        };
        for c in &self.timelines {
            for t in c {
                t.iter().for_each(&mut push);
            }
        }
        for t in self.status_enter.iter().chain(&self.status_loop).chain(&self.status_exit) {
            push(t);
        }
        for r in &self.rules {
            push(&r.condition);
            push(&r.action);
        }
        push(&self.init);
        for l in self.params.weighted_lists() {
            for e in l {
                push(&e.condition);
            }
        }
        for set in &self.cage_cmd_sets {
            for c in set {
                c.args.iter().for_each(|a| {
                    if let Arg::Ptr(t) = a {
                        push(t)
                    }
                });
            }
        }
        let mut out = BTreeMap::new();
        while let Some(o) = work.pop() {
            if out.contains_key(&o) {
                continue;
            }
            let s = self.decode_script(o)?;
            for c in &s.cmds {
                for a in &c.args {
                    if let Arg::Ptr(Target::Local(t)) = a {
                        work.push(*t);
                    }
                }
                if let Op::Switch { list, count, .. } = &c.op {
                    if let Ok(es) = self.params.weighted(list.id, *count) {
                        for e in es {
                            if let Target::Local(t) = e.condition {
                                work.push(t);
                            }
                        }
                    }
                }
            }
            out.insert(o, s);
        }
        self.scripts = out;
        Ok(())
    }
}
