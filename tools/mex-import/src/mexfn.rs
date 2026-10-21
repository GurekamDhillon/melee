//! m-ex code blobs inside .dat files: `ftFunction` (fighters), `grFunction` (stages) and
//! `itFunction` (articles: a count + an array of the same header).
//!
//! `MEXFunction`, data-relative (gw_mex_ftfunction.c, tools/mex_port/dump_ftfunction.py):
//!   +0x00 code  +0x04 instruction_reloc_table (stride 8)  +0x08 its count
//!   +0x0C func_table (stride 8: {replace_this, replace_with})  +0x10 its count
//!   +0x14 code_size  +0x18 debug_symbol_num  +0x1C debug_symbol (stride 12: {start, END, char*})
//! Instruction relocs: word0 = flag << 24 | code offset, word1 = target (code offset, or an
//! absolute 0x80xxxxxx vanilla address). Flags: 0x01 abs32, 0x04 lo16, 0x06 hi16 (ha), 0x0A
//! branch (rel24), 0x1A rel32.

use crate::hsd::Archive;

pub const BLOB_BASE: u32 = 0xC000_0000; // where the analysis relocates a blob (outside MEM1)

/// The vanilla address an out-of-blob branch at BLOB_BASE means.
///
/// m-ex links a blob to sit in MEM1, a rel24 `b`/`bl` away from the game's text; at BLOB_BASE
/// (0xC0000000, 1 GiB away) the relocated displacement no longer fits its 26 bits, so the
/// decoded target is the intended one modulo 2^26 (ip + sext26(f - at) == f mod 2^26). Vanilla
/// text is below 0x84000000, so its low 26 bits name it. Other targets come back unchanged.
pub fn vanilla_target(t: u32) -> u32 {
    if (0xBE00_0000..0xC200_0000).contains(&t) {
        0x8000_0000 | (t & 0x03FF_FFFF)
    } else {
        t
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Fighter,
    Stage,
    Item,
}
impl Owner {
    /// The record an `HSD_GObj.user_data` of this blob's callbacks points to.
    pub fn user_data_record(self) -> &'static str {
        match self {
            Owner::Fighter => "Fighter",
            Owner::Stage => "Ground",
            Owner::Item => "Item",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reloc {
    pub flag: u8,
    pub code_off: u32,
    pub target: u32,
}

#[derive(Clone, Debug)]
pub struct Blob {
    pub label: String,
    pub owner: Owner,
    /// Relocated code (internal targets at BLOB_BASE + offset).
    pub code: Vec<u8>,
    pub relocs: Vec<Reloc>,
    /// (slot or absolute patch address, code offset).
    pub overrides: Vec<(u32, u32)>,
    /// (start, end, name), sorted by start.
    pub symbols: Vec<(u32, u32, String)>,
}

fn cstr(ar: &Archive, p: u32) -> Option<String> {
    if !ar.in_data(p) {
        return None;
    }
    let s = &ar.data[p as usize..];
    let e = s.iter().position(|&c| c == 0)?;
    Some(String::from_utf8_lossy(&s[..e]).into_owned())
}

/// Parse one MEXFunction header at data offset `b`. None when it does not fit the archive.
pub fn parse_at(ar: &Archive, b: u32, owner: Owner, label: String) -> Result<Blob, String> {
    parse_at_base(ar, b, owner, label, BLOB_BASE)
}

/// As [`parse_at`], relocated to run at guest address `base` (an executing host places the blob
/// in MEM1 range, within rel24 reach of the game's text, as the C build's gw_ftfunction_load does).
pub fn parse_at_base(ar: &Archive, b: u32, owner: Owner, label: String, base: u32) -> Result<Blob, String> {
    let r = |o: u32| ar.u32(b + o);
    let (code_off, rel_off, rel_n, fn_off, fn_n, code_size, dbg_n, dbg_off) =
        (r(0), r(4), r(8), r(0xC), r(0x10), r(0x14), r(0x18), r(0x1C));
    let ds = ar.data.len() as u64;
    if !(code_off > 0 && (code_off as u64) < ds) || code_off as u64 + code_size as u64 > ds {
        return Err(format!("code {code_off:#x}+{code_size:#x} does not fit the {ds:#x}-byte data"));
    }
    if rel_off as u64 + rel_n as u64 * 8 > ds || dbg_off as u64 + dbg_n as u64 * 12 > ds || fn_off as u64 + fn_n as u64 * 8 > ds {
        return Err("a table runs past the data section".into());
    }
    let mut code = ar.data[code_off as usize..(code_off + code_size) as usize].to_vec();
    let relocs: Vec<Reloc> = (0..rel_n)
        .map(|i| {
            let w0 = ar.u32(rel_off + i * 8);
            Reloc { flag: (w0 >> 24) as u8, code_off: w0 & 0xFF_FFFF, target: ar.u32(rel_off + i * 8 + 4) }
        })
        .collect();
    for rl in &relocs {
        let o = rl.code_off as usize;
        if o + 4 > code.len() {
            continue;
        }
        let at = base.wrapping_add(rl.code_off);
        let f = if rl.target & 0xF000_0000 == 0x8000_0000 { rl.target } else { base.wrapping_add(rl.target) };
        let w = u32::from_be_bytes(code[o..o + 4].try_into().unwrap());
        let put16 = |code: &mut Vec<u8>, v: u16| code[o..o + 2].copy_from_slice(&v.to_be_bytes());
        match rl.flag {
            0x01 => code[o..o + 4].copy_from_slice(&f.to_be_bytes()),
            0x04 | 0x06 => {
                // dump_ftfunction.py: carry the sign of the low half into the high half (ha)
                let mut v = f;
                if v & 0x8000 != 0 {
                    let high = (v >> 16).wrapping_add(1);
                    v = (high << 16) | (v.wrapping_sub(high << 16) & 0xFFFF);
                }
                put16(&mut code, if rl.flag == 0x04 { v as u16 } else { (v >> 16) as u16 });
            }
            0x0A => code[o..o + 4].copy_from_slice(&(w | (f.wrapping_sub(at) & 0x03FF_FFFC)).to_be_bytes()),
            0x1A => code[o..o + 4].copy_from_slice(&f.wrapping_sub(at).to_be_bytes()),
            _ => {}
        }
    }
    let overrides = (0..fn_n).map(|i| (ar.u32(fn_off + i * 8), ar.u32(fn_off + i * 8 + 4))).collect();
    let mut symbols: Vec<(u32, u32, String)> = (0..dbg_n)
        .map(|i| {
            let e = dbg_off + i * 12;
            (ar.u32(e), ar.u32(e + 4), cstr(ar, ar.u32(e + 8)).unwrap_or_else(|| format!("fn_{:x}", ar.u32(e))))
        })
        .collect();
    symbols.sort();
    Ok(Blob { label, owner, code, relocs, overrides, symbols })
}

/// Every m-ex code blob in one archive (empty when it has none).
pub fn blobs(ar: &Archive, file: &str) -> (Vec<Blob>, Vec<String>) {
    let mut out = vec![];
    let mut errs = vec![];
    for (sym, off) in &ar.publics {
        let owner = match sym.as_str() {
            "ftFunction" => Owner::Fighter,
            "grFunction" => Owner::Stage,
            "itFunction" => Owner::Item,
            _ => continue,
        };
        if owner == Owner::Item {
            let n = ar.u32(*off);
            if n > 256 {
                errs.push(format!("{file}: itFunction count {n} implausible"));
                continue;
            }
            for i in 0..n {
                let at = off + 4 + 4 * i;
                let p = ar.u32(at);
                if p == 0 || !ar.reloc_set.contains(&at) {
                    continue;
                }
                match parse_at(ar, p, owner, format!("{file}:itFunction[{i}]")) {
                    Ok(b) => out.push(b),
                    Err(e) => errs.push(format!("{file}:itFunction[{i}]: {e}")),
                }
            }
        } else {
            match parse_at(ar, *off, owner, format!("{file}:{sym}")) {
                Ok(b) => out.push(b),
                Err(e) => errs.push(format!("{file}:{sym}: {e}")),
            }
        }
    }
    (out, errs)
}
