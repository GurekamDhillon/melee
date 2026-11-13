//! TARGET_PC (Rust only): per-move action-script overrides (docs/rust-port/notes/anim-overrides.md, "Script overrides").
//!
//! A mod's `mod.json` maps (fighter, clip) to a loose file holding ONE replacement subaction (action) script, the
//! command stream a motion row's `xC` points at in the Pl file. It pairs with `anim_overrides`: a replaced clip can
//! carry its own hitbox timing, sword trail, sounds, IASA frame. The table is filled before the engine boots
//! (`install`, from `melee::mods`); `Fighter_ChangeMotionState` and the wait-animation switch ask `script_for` for the
//! script pointer of the row they bind. With no entry installed `active()` is false and they run exactly as before.
//!
//! Mechanism: the pointer is substituted where the row is bound, not written into the loaded Pl file's table (what a
//! Geno `subactions` overlay does at spawn). The Geno route needs a `geno.json` define for the fighter (a profile
//! index, a content id, the snapshotted script pool) and the table patch; this one touches nothing of the match state
//! but the command pointer it already holds, and only for rows a mod names.
//!
//! File formats: `.words` = text, one 32-bit word per line (`0x..` hex or decimal, `#` comments; what
//! `python -m tools.geno.script asm --words` writes); any other extension = raw big-endian words.
//!
//! Validation (at load, the boot stops on an error naming the mod and file): the stream is walked command by command
//! with the engine's own word-count table; every command must lie inside the file, the walk must reach an End
//! (opcode 0); loops must balance (no Execute Loop without a Set Loop, nesting <= 4); opcodes the engine has no
//! handler for (> 59), Geno's escape (59, needs a Geno profile), and Subroutine / Return / Goto (5 / 6 / 7: they hold
//! 32-bit addresses into the Pl file, meaningless for a loose file) are refused. One End word is appended after the
//! file's words so a stream that ends on a wait still stops inside the buffer.

use crate::anim_override::Clip;
use crate::types::CmdUnion;
use std::path::PathBuf;

/// Largest accepted script, in words (retail action scripts are well under 200 words).
pub const MAX_WORDS: usize = 4096;
/// Loop nesting the CommandInfo's return slots hold safely (two slots per loop).
const MAX_LOOP_DEPTH: usize = 4;

/// One installed script override.
#[derive(Debug)]
pub struct Entry {
    /// FighterKind (internal kind) whose rows it replaces.
    pub kind: i32,
    pub clip: Clip,
    pub path: PathBuf,
    /// `<mod id>/<file>` for messages.
    pub label: String,
    /// The script words as the engine reads them (big-endian bytes in each u32), End appended. Never reallocated while
    /// installed: fighters hold pointers into it.
    pub words: Box<[u32]>,
}

impl Entry {
    /// The command pointer to bind.
    pub fn ptr(&self) -> *mut CmdUnion {
        self.words.as_ptr() as *mut CmdUnion
    }
}

static mut TABLE: Vec<Entry> = Vec::new();
static mut ACTIVE: bool = false;

/// True when at least one script override is installed.
#[inline]
pub fn active() -> bool {
    unsafe { ACTIVE }
}

/// Replace the whole table (mount order: later entries win). Call before the engine boots (fighters hold pointers into
/// the installed buffers). An empty vector turns overrides off.
pub fn install(v: Vec<Entry>) {
    unsafe {
        let t = &mut *core::ptr::addr_of_mut!(TABLE);
        *t = v;
        ACTIVE = !t.is_empty();
    }
}

pub fn entries() -> &'static [Entry] {
    unsafe { &*core::ptr::addr_of!(TABLE) }
}

/// The entry that replaces a motion row's script (same matching as `anim_override::find`).
pub fn find(owner: i32, row: Option<i32>, sym: Option<&[u8]>) -> Option<&'static Entry> {
    entries().iter().rev().find(|e| {
        e.kind == owner
            && match &e.clip {
                Clip::Row(r) => row == Some(*r),
                Clip::Symbol(s) => sym == Some(s.as_slice()),
            }
    })
}

/// Read, parse and validate a script file.
pub fn load(kind: i32, clip: Clip, path: PathBuf, label: String) -> Result<Entry, String> {
    let bytes = std::fs::read(&path).map_err(|er| format!("script override {label}: cannot read {}: {er}", path.display()))?;
    let text = path.extension().is_some_and(|x| x.eq_ignore_ascii_case("words"));
    let words = parse(&bytes, text).and_then(|w| check(&w).map(|_| w)).map_err(|m| format!("script override {label}: {m}"))?;
    let mut be: Vec<u32> = words.iter().map(|w| w.to_be()).collect();
    be.push(0); // End
    Ok(Entry { kind, clip, path, label, words: be.into_boxed_slice() })
}

/// The file's words (host values).
pub fn parse(b: &[u8], text: bool) -> Result<Vec<u32>, String> {
    let mut out = Vec::new();
    if text {
        let s = std::str::from_utf8(b).map_err(|_| "the .words file is not UTF-8 text".to_string())?;
        for (i, line) in s.lines().enumerate() {
            let t = line.split('#').next().unwrap_or("").trim();
            if t.is_empty() {
                continue;
            }
            let v = if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                u32::from_str_radix(h, 16).ok()
            } else if let Some(n) = t.strip_prefix('-') {
                n.parse::<u32>().ok().filter(|&n| n <= 1 << 31).map(|n| n.wrapping_neg())
            } else {
                t.parse::<u32>().ok()
            };
            out.push(v.ok_or_else(|| format!("line {}: \"{t}\" is not a 32-bit word", i + 1))?);
        }
    } else {
        if b.len() % 4 != 0 {
            return Err(format!("{} bytes is not a whole number of 32-bit words", b.len()));
        }
        out.extend(b.chunks_exact(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])));
    }
    if out.is_empty() {
        return Err("the script is empty".into());
    }
    if out.len() > MAX_WORDS {
        return Err(format!("{} words is over the limit of {MAX_WORDS}", out.len()));
    }
    Ok(out)
}

/// Words per command for opcodes 0..=9 (lbcommand) and 10..=58 (`ftAction_803C0870`).
fn cmd_words(op: u32) -> Option<usize> {
    match op {
        0..=4 | 6 | 8 | 9 => Some(1),
        5 | 7 => Some(2),
        10..=58 => Some(crate::tu::ftaction::ftAction_803C0870[(op - 10) as usize] as usize),
        _ => None,
    }
}

/// Walk the stream; answers the number of commands up to and including the End.
pub fn check(w: &[u32]) -> Result<usize, String> {
    let (mut i, mut n, mut depth) = (0usize, 0usize, 0usize);
    while i < w.len() {
        let op = w[i] >> 26;
        let at = format!("word {i} (0x{:08X}, opcode {op})", w[i]);
        match op {
            0 => return Ok(n + 1),
            5..=7 => return Err(format!("{at}: subroutine / return / goto hold addresses into the fighter file and cannot be used in a loose script")),
            59 => return Err(format!("{at}: the Geno escape (opcode 59) needs a Geno fighter define; use a geno.json subactions overlay")),
            3 => {
                depth += 1;
                if depth > MAX_LOOP_DEPTH {
                    return Err(format!("{at}: loops nest deeper than {MAX_LOOP_DEPTH}"));
                }
            }
            4 => {
                if depth == 0 {
                    return Err(format!("{at}: execute loop without a set loop"));
                }
                depth -= 1;
            }
            _ => {}
        }
        let len = cmd_words(op).ok_or_else(|| format!("{at}: no such command"))?;
        if i + len > w.len() {
            return Err(format!("{at}: the command needs {len} words, the script ends after {}", w.len() - i));
        }
        i += len;
        n += 1;
    }
    Err(format!("the script runs off its end after {} words without an End (opcode 0, word 0x00000000)", w.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// marth-test's retuned forward-tilt shape, synthetic values (no game data): continuation, waits, a 5-word command,
    /// a 7-word one, loops, IASA (op 0x2B? any 1-word), End.
    fn sample() -> Vec<u32> {
        vec![
            0xD000_0003, // op 52, 1 word
            0x0800_0007, // wait_until 7
            0x2C00_0000, 0, 0, 0, 0, // op 11 hitbox (5 words)
            0x0C00_0002, // set loop 2
            0x0400_0001, // wait 1
            0x1000_0000, // execute loop
            0x0800_0028, // wait_until 40
            0x0000_0000, // end
        ]
    }

    #[test]
    fn parses_text_and_binary() {
        let t = b"0xD0000003\n# comment\n  8 # trailing\n-1\n\n0x0\n";
        assert_eq!(parse(t, true).unwrap(), vec![0xD000_0003, 8, 0xFFFF_FFFF, 0]);
        assert!(parse(b"0xZZ\n", true).unwrap_err().contains("line 1"));
        assert!(parse(b"\n# only\n", true).unwrap_err().contains("empty"));
        assert_eq!(parse(&[0xD0, 0, 0, 3, 0, 0, 0, 0], false).unwrap(), vec![0xD000_0003, 0]);
        assert!(parse(&[1, 2, 3], false).unwrap_err().contains("whole number"));
        let big = "0\n".repeat(MAX_WORDS + 1);
        assert!(parse(big.as_bytes(), true).unwrap_err().contains("over the limit"));
    }

    #[test]
    fn walks_a_good_script() {
        assert_eq!(check(&sample()).unwrap(), 8);
    }

    #[test]
    fn refuses_bad_scripts() {
        // no End
        let mut w = sample();
        w.pop();
        assert!(check(&w).unwrap_err().contains("without an End"));
        // a 5-word command cut short at the end of the file
        assert!(check(&[0x2C00_0000, 0, 0]).unwrap_err().contains("needs 5 words"));
        // goto / subroutine / return
        assert!(check(&[0x1C00_0000, 0x8000_0000, 0]).unwrap_err().contains("goto"));
        assert!(check(&[0x1400_0000, 0x8000_0000, 0]).unwrap_err().contains("subroutine"));
        // Geno escape, unknown opcode
        assert!(check(&[0xEC00_0000, 0]).unwrap_err().contains("opcode 59"));
        assert!(check(&[0xF000_0000, 0]).unwrap_err().contains("no such command"));
        // unbalanced loops
        assert!(check(&[0x1000_0000, 0]).unwrap_err().contains("without a set loop"));
        assert!(check(&[0x0C00_0001; 5]).unwrap_err().contains("nest deeper"));
    }

    #[test]
    fn load_appends_end_and_stores_big_endian() {
        let d = std::env::temp_dir().join(format!("script-override-test-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("ftilt.words");
        std::fs::write(&f, "0x08000007\n0x0\n").unwrap();
        let e = load(9, Clip::Row(55), f.clone(), "m/ftilt.words".into()).unwrap();
        assert_eq!(&*e.words, &[0x0800_0007u32.to_be(), 0, 0]);
        // errors name the mod/file
        std::fs::write(&f, "0x08000007\n").unwrap();
        let er = load(9, Clip::Row(55), f.clone(), "m/ftilt.words".into()).unwrap_err();
        assert!(er.starts_with("script override m/ftilt.words: ") && er.contains("without an End"), "{er}");
        let er = load(9, Clip::Row(55), d.join("missing.bin"), "m/missing.bin".into()).unwrap_err();
        assert!(er.contains("m/missing.bin") && er.contains("cannot read"), "{er}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn find_later_wins() {
        let e = |clip, label: &str| Entry { kind: 9, clip, path: PathBuf::new(), label: label.into(), words: vec![0].into() };
        install(vec![e(Clip::Row(55), "a"), e(Clip::Row(55), "b"), e(Clip::Symbol(b"S".to_vec()), "s")]);
        assert!(active());
        assert_eq!(find(9, Some(55), None).unwrap().label, "b");
        assert_eq!(find(9, Some(3), Some(b"S")).unwrap().label, "s");
        assert!(find(9, Some(56), None).is_none());
        assert!(find(10, Some(55), None).is_none());
        install(Vec::new());
        assert!(!active());
    }
}
