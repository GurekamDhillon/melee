//! The retail `main.dol` as guest memory, and `config/GALE01/symbols.txt` (the DOL referee, notes/referee.md).
//!
//! The DOL is the user's own disc data: it is read at run time from a path the caller supplies (never printed, never
//! committed). The header is 7 text + 11 data sections (file offset, load address, size) and the bss range.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Section {
    pub addr: u32,
    pub text: bool,
    pub bytes: Vec<u8>,
}

pub struct Dol {
    pub sections: Vec<Section>,
    pub bss_addr: u32,
    pub bss_size: u32,
    pub entry: u32,
}

fn be32(b: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

impl Dol {
    pub fn parse(b: &[u8]) -> Result<Dol, String> {
        if b.len() < 0x100 {
            return Err("DOL shorter than its header".into());
        }
        let mut sections = vec![];
        for i in 0..18 {
            let (off, addr, size) = (be32(b, 4 * i) as usize, be32(b, 0x48 + 4 * i), be32(b, 0x90 + 4 * i) as usize);
            if size == 0 {
                continue;
            }
            let bytes = b.get(off..off + size).ok_or("DOL section past the end of the file")?.to_vec();
            sections.push(Section { addr, text: i < 7, bytes });
        }
        Ok(Dol { sections, bss_addr: be32(b, 0xD8), bss_size: be32(b, 0xDC), entry: be32(b, 0xE0) })
    }

    pub fn load(path: &Path) -> Result<Dol, String> {
        let b = std::fs::read(path).map_err(|e| format!("main.dol: {e}"))?;
        Dol::parse(&b)
    }

    /// The DOL to use: `$REFEREE_DOL`, else `orig/GALE01/sys/main.dol` in this checkout or in the workspace's main
    /// `melee` checkout (worktrees have no `orig/`). None when there is none (tests then skip).
    pub fn find() -> Option<PathBuf> {
        if let Some(p) = std::env::var_os("REFEREE_DOL") {
            return Some(PathBuf::from(p));
        }
        let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut cands = vec![here.join("orig/GALE01/sys/main.dol")];
        // <workspace>/worktrees/<name> or <workspace>/wt/<name> -> <workspace>/melee
        if let Some(ws) = here.canonicalize().ok().and_then(|p| p.parent().and_then(|p| p.parent()).map(Path::to_path_buf)) {
            cands.push(ws.join("melee/orig/GALE01/sys/main.dol"));
        }
        cands.into_iter().find(|p| p.is_file())
    }

    /// The section holding `addr`, if any.
    pub fn section_of(&self, addr: u32) -> Option<&Section> {
        self.sections.iter().find(|s| addr >= s.addr && addr < s.addr + s.bytes.len() as u32)
    }

    pub fn word(&self, addr: u32) -> Option<u32> {
        let s = self.section_of(addr)?;
        let o = (addr - s.addr) as usize;
        s.bytes.get(o..o + 4).map(|w| be32(w, 0))
    }

    pub fn in_text(&self, addr: u32) -> bool {
        self.section_of(addr).is_some_and(|s| s.text)
    }
}

#[derive(Clone, Debug)]
pub struct Sym {
    pub name: String,
    pub section: String,
    pub addr: u32,
    pub size: u32,
    pub function: bool,
}

/// `config/GALE01/symbols.txt` of this checkout (source data, not disc data).
pub fn symbols() -> Result<Vec<Sym>, String> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/GALE01/symbols.txt");
    let txt = std::fs::read_to_string(&p).map_err(|e| format!("symbols.txt: {e}"))?;
    let mut v = vec![];
    for l in txt.lines() {
        // name = .text:0x8000D2EC; // type:function size:0xC4 scope:global
        let Some((name, rest)) = l.split_once(" = ") else { continue };
        let Some((loc, attrs)) = rest.split_once(';') else { continue };
        let Some((section, addr)) = loc.split_once(":0x") else { continue };
        let Ok(addr) = u32::from_str_radix(addr.trim(), 16) else { continue };
        let size = attrs.split_whitespace().find_map(|t| t.strip_prefix("size:0x")).and_then(|s| u32::from_str_radix(s, 16).ok()).unwrap_or(0);
        v.push(Sym { name: name.trim().to_string(), section: section.trim().to_string(), addr, size, function: attrs.contains("type:function") });
    }
    v.sort_by_key(|s| s.addr);
    Ok(v)
}
