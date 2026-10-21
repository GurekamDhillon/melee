//! `smash2.sem`: the sound-effect script table AXDriver_8038DA70 (axdriver.c) loads. Layout, all
//! big-endian u32: `nA, A[nA]`, `nB, B[nB]` (offsets), `nbanks, first_index[nbanks]`,
//! `nscripts, script[nscripts]` (offsets), `nC, C[nC]` (offsets). The C relocates the offsets to
//! pointers into the file buffer; here a script is a word index into [`Sem::words`], and a command
//! stream walks words exactly as the C's `u32* cmd_stream` does.

#[derive(Clone, Debug, Default)]
pub struct Sem {
    /// The whole file as big-endian words (a trailing partial word is zero-padded).
    pub words: Vec<u32>,
    /// AXDriver_804D77B4: per bank, the index of its first script.
    pub bank_first: Vec<u32>,
    /// AXDriver_804D77BC: per script, its first command (word index).
    pub scripts: Vec<usize>,
}

impl Sem {
    pub fn parse(file: &[u8]) -> Result<Sem, String> {
        let mut words = vec![0u32; file.len().div_ceil(4)];
        for (i, w) in words.iter_mut().enumerate() {
            let mut b = [0u8; 4];
            let n = (file.len() - i * 4).min(4);
            b[..n].copy_from_slice(&file[i * 4..i * 4 + n]);
            *w = u32::from_be_bytes(b);
        }
        let get = |at: usize| words.get(at).copied().ok_or_else(|| format!("smash2.sem: truncated at word {at}"));
        let mut at = 0usize;
        let na = get(at)? as usize;
        at += 1 + na;
        let nb = get(at)? as usize;
        at += 1 + nb;
        let nbanks = get(at)? as usize;
        at += 1;
        let bank_first = (0..nbanks).map(|i| get(at + i)).collect::<Result<Vec<_>, _>>()?;
        at += nbanks;
        let ns = get(at)? as usize;
        at += 1;
        let mut scripts = Vec::with_capacity(ns);
        for i in 0..ns {
            let off = get(at + i)? & !3; // the C adds `buf & ~3`: offsets are word-aligned
            if off as usize / 4 >= words.len() {
                return Err(format!("smash2.sem: script {i} at {off:#x} past the end"));
            }
            scripts.push(off as usize / 4);
        }
        Ok(Sem { words, bank_first, scripts })
    }
}
