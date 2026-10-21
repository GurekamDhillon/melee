//! HSD archives and m-ex's `MxDt.dat` tables, offline (data base 0: a relocated pointer word is
//! the data offset it already holds).
//!
//! `Archive` mirrors `tools/mex_port/mex_hsd.py` `Archive` rather than `melee_runtime::dat`
//! (which rejects a header file_size that differs from the file length and relocated words past
//! the data block): the importer has to read the same files the Python packer accepts. Every
//! accessor is bounds-checked and reads 0 out of range (Python would raise).
//! `Tables` / `fighter_claims` are `tools/mex_port/split_modpack.py` line for line.

use std::collections::{BTreeMap, HashSet};

pub const HEADER: usize = 0x20;

#[allow(dead_code)]
pub struct Archive {
    pub data: Vec<u8>,
    pub relocs: Vec<u32>,
    pub reloc_set: HashSet<u32>,
    pub publics: Vec<(String, u32)>,
    pub externs: Vec<(String, u32)>,
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at.checked_add(4)?).map(|s| u32::from_be_bytes(s.try_into().unwrap()))
}

impl Archive {
    pub fn parse(raw: &[u8]) -> Result<Archive, String> {
        if raw.len() < HEADER {
            return Err("shorter than an HSD header".into());
        }
        let g = |o| be32(raw, o).unwrap() as usize;
        let (data_size, nr, np, ne) = (g(4), g(8), g(0xC), g(0x10));
        let o_reloc = HEADER.checked_add(data_size).ok_or("size overflow")?;
        let o_pub = o_reloc.checked_add(nr.checked_mul(4).ok_or("overflow")?).ok_or("overflow")?;
        let o_ext = o_pub.checked_add(np.checked_mul(8).ok_or("overflow")?).ok_or("overflow")?;
        let o_sym = o_ext.checked_add(ne.checked_mul(8).ok_or("overflow")?).ok_or("overflow")?;
        if o_sym > raw.len() {
            return Err(format!("archive tables run past EOF (need {o_sym:#x}, have {:#x})", raw.len()));
        }
        let data = raw[HEADER..o_reloc].to_vec();
        let relocs: Vec<u32> = (0..nr).map(|i| be32(raw, o_reloc + 4 * i).unwrap()).collect();
        let sym = |tab: usize, i: usize| -> Result<(String, u32), String> {
            let off = be32(raw, tab + 8 * i).unwrap();
            let s = o_sym + be32(raw, tab + 8 * i + 4).unwrap() as usize;
            let rest = raw.get(s..).ok_or("symbol offset past EOF")?;
            let end = rest.iter().position(|&c| c == 0).ok_or("unterminated symbol")?;
            Ok((String::from_utf8_lossy(&rest[..end]).into_owned(), off))
        };
        let publics = (0..np).map(|i| sym(o_pub, i)).collect::<Result<_, _>>()?;
        let externs = (0..ne).map(|i| sym(o_ext, i)).collect::<Result<_, _>>()?;
        Ok(Archive { reloc_set: relocs.iter().copied().collect(), data, relocs, publics, externs })
    }

    pub fn public(&self, name: &str) -> Option<u32> {
        self.publics.iter().find(|(s, _)| s == name).map(|(_, o)| *o)
    }
    pub fn u32(&self, off: u32) -> u32 {
        be32(&self.data, off as usize).unwrap_or(0)
    }
    pub fn s32(&self, off: u32) -> i32 {
        self.u32(off) as i32
    }
    pub fn u16(&self, off: u32) -> u16 {
        self.data.get(off as usize..off as usize + 2).map(|s| u16::from_be_bytes([s[0], s[1]])).unwrap_or(0)
    }
    pub fn u8(&self, off: u32) -> u8 {
        self.data.get(off as usize).copied().unwrap_or(0)
    }
    pub fn in_data(&self, off: u32) -> bool {
        off > 0 && (off as usize) < self.data.len()
    }
}

// ---- MxDt.dat ------------------------------------------------------------------------------

/// MexData root, in declaration order (dump_mxdt.py ROOT_FIELDS).
pub const ROOT_FIELDS: [&str; 14] = [
    "metadata", "menu", "fighter", "fighter_function", "ssm", "music", "effect", "item", "kirby_data",
    "kirby_function", "stage", "stage_desc", "scene", "misc",
];
/// MexMetaData counts after {u8 major, u8 minor, u16 flags} (dump_mxdt.py METADATA_COUNTS).
pub const METADATA_COUNTS: [&str; 14] = [
    "internal_id_count", "external_id_count", "css_icon_count", "internal_stage_count",
    "external_stage_count", "sss_icon_count", "ssm_count", "bgm_count", "effect_count",
    "bootup_scene", "last_major", "last_minor", "trophy_count", "trophy_sd_offset",
];
/// MexData.fighter parallel arrays (dump_mxdt.py FIGHTER_FIELDS).
pub const FIGHTER_FIELDS: [&str; 33] = [
    "names", "pl_file", "insignia_idx", "ft_kind_desc", "costume_info", "costume_file", "ftdemo",
    "anim_filenames", "anim_num", "effect_index", "result_file", "result_scale", "victory_theme",
    "announcer_call", "ssm_files", "costume_pointers", "ft_archives", "walljump", "rst_runtime",
    "item_lookup", "target_test_lookup", "fighter_music", "vi_files", "endclassicfiles",
    "endadventurefiles", "endallstarfiles", "endmoviefiles", "race_to_finish", "demo_params",
    "classic_trophy_id", "adventure_trophy_id", "allstar_trophy_id", "ending_fall_scale",
];

pub const FIRST_NEW_FIGHTER: i32 = 27;
pub const FIRST_NEW_STAGE_EXT: i32 = 33;
pub const FIRST_NEW_STAGE_INT: i32 = 71;
const EFFECT_STRIDE: u32 = 12;
const SSS_STRIDE: u32 = 0x20;
const SSS_EXT_OFF: u32 = 0x1C;
const STAGE_IDS_STRIDE: u32 = 12;
const STAGE_ROW_FILE: u32 = 8;

pub struct Tables {
    pub ar: Archive,
    pub root: BTreeMap<&'static str, u32>,
    pub meta: BTreeMap<&'static str, i32>,
    pub version: (u8, u8),
}

impl Tables {
    pub fn parse(raw: &[u8]) -> Result<Tables, String> {
        let ar = Archive::parse(raw)?;
        let base = ar.public("mexData").ok_or("no public symbol mexData")?;
        let root: BTreeMap<_, _> = ROOT_FIELDS.iter().enumerate().map(|(i, n)| (*n, ar.u32(base + 4 * i as u32))).collect();
        let m = root["metadata"];
        let meta = METADATA_COUNTS.iter().enumerate().map(|(i, n)| (*n, ar.s32(m + 4 + 4 * i as u32))).collect();
        let version = (ar.u8(m), ar.u8(m + 1));
        Ok(Tables { ar, root, meta, version })
    }

    pub fn ptr(&self, p: u32) -> u32 {
        if p > 0 && p as usize + 4 <= self.ar.data.len() {
            self.ar.u32(p)
        } else {
            0
        }
    }

    /// Printable ASCII C string of at most 96 bytes, or None (split_modpack.py Tables.cstr).
    pub fn cstr(&self, p: u32) -> Option<String> {
        if p == 0 || p as usize >= self.ar.data.len() {
            return None;
        }
        let s = &self.ar.data[p as usize..(p as usize + 96).min(self.ar.data.len())];
        let s = &s[..s.iter().position(|&c| c == 0).unwrap_or(s.len())];
        if s.is_empty() || !s.iter().all(|&c| (32..127).contains(&c)) {
            return None;
        }
        Some(String::from_utf8(s.to_vec()).unwrap())
    }

    pub fn ff(&self, name: &str) -> u32 {
        let i = FIGHTER_FIELDS.iter().position(|f| *f == name).expect("fighter field") as u32;
        self.ar.u32(self.root["fighter"] + i * 4)
    }

    /// Internal fighter id -> first external id naming it (ft_kind_desc, 3 bytes per external).
    pub fn ext_of_internal(&self) -> BTreeMap<u8, i32> {
        let mut out = BTreeMap::new();
        let base = self.ff("ft_kind_desc");
        for e in 0..self.meta["external_id_count"].max(0) {
            out.entry(self.ar.u8(base + e as u32 * 3)).or_insert(e);
        }
        out
    }

    pub fn effect_file(&self, eid: u8) -> Option<String> {
        if eid == 0xFF {
            return None;
        }
        let t = self.ptr(self.root["effect"]);
        if t == 0 {
            return None;
        }
        self.cstr(self.ptr(t + eid as u32 * EFFECT_STRIDE))
    }

    pub fn ssm_file(&self, sid: u8) -> Option<String> {
        let t = self.ptr(self.root["ssm"]);
        if t == 0 {
            return None;
        }
        self.cstr(self.ptr(t + sid as u32 * 4)).map(|n| format!("audio/us/{n}"))
    }

    /// External stage id -> (internal id, Gr file).
    pub fn stage_file_for_ext(&self, ext: i32) -> (Option<i32>, Option<String>) {
        let ids = self.ptr(self.root["stage"]);
        if ids == 0 || ext < 0 || ext >= self.meta["external_stage_count"] {
            return (None, None);
        }
        let g = self.ar.s32(ids + ext as u32 * STAGE_IDS_STRIDE);
        if g < 0 || g >= self.meta["internal_stage_count"] {
            return (None, None);
        }
        let row = self.ptr(self.root["stage_desc"] + g as u32 * 4);
        (Some(g), if row != 0 { self.cstr(self.ptr(row + STAGE_ROW_FILE)) } else { None })
    }

    pub fn stage_name(&self, g: i32) -> Option<String> {
        let names = self.ptr(self.root["stage"] + 0x10);
        if names == 0 {
            return None;
        }
        self.cstr(self.ptr(names + g as u32 * 4))
    }

    pub fn sss_exts(&self) -> Vec<i32> {
        let tbl = self.ptr(self.root["menu"] + 0x08);
        (0..self.meta["sss_icon_count"].max(0)).map(|i| self.ar.s32(tbl + i as u32 * SSS_STRIDE + SSS_EXT_OFF)).collect()
    }

    /// (display name, claimed disc paths) for internal fighter k (external e).
    /// split_modpack.py fighter_claims. Empty when the row names no Pl file.
    pub fn fighter_claims(&self, k: i32, e: Option<i32>) -> (Option<String>, Vec<String>) {
        let k = k as u32;
        let mut files: Vec<Option<String>> = Vec::new();
        let Some(pl) = self.cstr(self.ptr(self.ff("pl_file") + k * 8)) else {
            return (None, vec![]);
        };
        files.push(Some(pl.clone()));
        files.push(self.cstr(self.ptr(self.ff("anim_filenames") + k * 4)));
        let mut name = None;
        if let Some(e) = e {
            let e = e as u32;
            name = self.cstr(self.ptr(self.ff("names") + e * 4));
            let ncost = self.ar.u8(self.ff("costume_info") + e * 4);
            let cf = self.ptr(self.ff("costume_file") + k * 4);
            if cf != 0 {
                for c in 0..ncost as u32 {
                    files.push(self.cstr(self.ptr(cf + c * 16)));
                }
            }
            for fld in ["result_file", "vi_files", "endclassicfiles", "endadventurefiles", "endallstarfiles", "endmoviefiles"] {
                files.push(self.cstr(self.ptr(self.ff(fld) + e * 4)));
            }
            files.push(self.ssm_file(self.ar.u8(self.ff("ssm_files") + e * 0x10)));
            let tt = self.ar.u16(self.ff("target_test_lookup") + e * 2) as i32;
            if let (Some(g), Some(gr)) = self.stage_file_for_ext(tt) {
                if g >= FIRST_NEW_STAGE_INT {
                    files.push(Some(gr));
                }
            }
        }
        let demo = self.ptr(self.ff("ftdemo") + k * 4);
        for i in 0..4 {
            if demo != 0 {
                if let Some(sym) = self.cstr(self.ptr(demo + i * 4)) {
                    files.push(Some(format!("{sym}.dat")));
                }
            }
        }
        files.push(self.effect_file(self.ar.u8(self.ff("effect_index") + k)));
        let kd = self.root["kirby_data"];
        let cap = self.ptr(kd);
        files.push(if cap != 0 { self.cstr(self.ptr(cap + k * 8)) } else { None });
        let kef = self.ptr(kd + 0x10);
        files.push(if kef != 0 { self.effect_file(self.ar.u8(kef + k)) } else { None });
        let files = files.into_iter().flatten().map(|f| f.trim_start_matches('/').to_string()).collect();
        (Some(name.unwrap_or(pl)), files)
    }
}
