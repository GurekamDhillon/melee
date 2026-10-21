//! Code inventory over a set of .dat files: run `analyze` on every m-ex blob, aggregate.

use crate::analyze::{analyze, Class};
use crate::hsd::Archive;
use crate::layout::Registry;
use crate::mexfn;
use std::collections::BTreeMap;
use std::fmt::Write as _;

#[derive(Default, Clone, Debug)]
pub struct Counts {
    pub by_class: BTreeMap<Class, u64>,
    /// Mapped accesses whose offset is the leaf field's first byte with a matching size.
    pub exact: u64,
}
impl Counts {
    pub fn struct_total(&self) -> u64 {
        [Class::Mapped, Class::ArrayElem, Class::TypedNoField, Class::Opaque, Class::Unknown, Class::Indexed].iter().map(|c| self.by_class.get(c).copied().unwrap_or(0)).sum()
    }
    pub fn get(&self, c: Class) -> u64 {
        self.by_class.get(&c).copied().unwrap_or(0)
    }
    pub fn coverage(&self) -> f64 {
        let t = self.struct_total();
        if t == 0 {
            0.0
        } else {
            self.get(Class::Mapped) as f64 / t as f64
        }
    }
    fn add(&mut self, o: &Counts) {
        for (k, v) in &o.by_class {
            *self.by_class.entry(*k).or_default() += v;
        }
        self.exact += o.exact;
    }
}

pub struct BlobSummary {
    pub label: String,
    pub functions: usize,
    pub instructions: usize,
    pub seeded: usize,
    pub data_words: usize,
    pub unreached: usize,
    pub counts: Counts,
}

#[derive(Default)]
pub struct FieldUse {
    pub native: String,
    pub c_type: &'static str,
    pub loads: u64,
    pub stores: u64,
    pub inner: u64,
}

#[derive(Default)]
pub struct CodeReport {
    pub files_scanned: usize,
    pub blobs: Vec<BlobSummary>,
    pub errors: Vec<String>,
    pub total: Counts,
    /// C path -> use.
    pub fields: BTreeMap<String, FieldUse>,
    /// record -> accesses with a typed base but no field (offset, count).
    pub no_field: BTreeMap<String, BTreeMap<i32, u64>>,
    /// Opaque accesses by the untyped field their base pointer was loaded from.
    pub opaque: BTreeMap<String, u64>,
    /// TSV rows of every non-stack access.
    pub tsv: String,
}

impl CodeReport {
    pub fn scan_file(&mut self, reg: &Registry, name: &str, bytes: &[u8]) {
        let Ok(ar) = Archive::parse(bytes) else { return };
        if !ar.publics.iter().any(|(s, _)| s.ends_with("Function")) {
            return;
        }
        self.files_scanned += 1;
        let (blobs, errs) = mexfn::blobs(&ar, name);
        self.errors.extend(errs);
        for b in blobs {
            let r = analyze(reg, &b);
            let mut c = Counts::default();
            for a in &r.accesses {
                *c.by_class.entry(a.class).or_default() += 1;
                if a.class == Class::Stack {
                    continue;
                }
                let (rec, off) = a.record.map(|(n, o)| (n.to_string(), o.to_string())).unwrap_or_default();
                let (cp, np) = a.mapping.as_ref().map(|m| (m.c_path.clone(), m.native_path.clone())).unwrap_or_default();
                let addr = a.addr.map(|x| format!("{x:#010x}")).unwrap_or_default();
                let _ = writeln!(
                    self.tsv,
                    "{}\t{}\t{:#x}\t{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}",
                    r.label, a.func, a.code_off, a.insn, if a.store { "st" } else { "ld" }, a.size, a.disp, a.class, rec, off, cp, np, addr
                );
                if let Some(m) = &a.mapping {
                    if m.at_start && a.class == Class::Mapped {
                        c.exact += 1;
                    }
                    let u = self.fields.entry(m.c_path.clone()).or_default();
                    u.native = m.native_path.clone();
                    u.c_type = m.leaf_type;
                    if a.store {
                        u.stores += 1
                    } else {
                        u.loads += 1
                    }
                    if !m.at_start {
                        u.inner += 1;
                    }
                } else if a.class == Class::Opaque {
                    if let Some((n, o)) = a.record {
                        let origin = reg
                            .find(n)
                            .and_then(|r| reg.resolve(r, o as u32, 4))
                            .map(|m| m.c_path)
                            .unwrap_or_else(|| format!("{n}+{o:#x}"));
                        *self.opaque.entry(origin).or_default() += 1;
                    }
                } else if a.class == Class::TypedNoField {
                    if let Some((n, o)) = a.record {
                        *self.no_field.entry(n.to_string()).or_default().entry(o).or_default() += 1;
                    }
                }
            }
            self.total.add(&c);
            self.blobs.push(BlobSummary { label: r.label, functions: r.functions, instructions: r.instructions, seeded: r.seeded_entries, data_words: r.data_words, unreached: r.unreached_words, counts: c });
        }
    }

    pub fn summary_text(&self) -> String {
        let mut s = String::new();
        let t = &self.total;
        let _ = writeln!(s, "code inventory: {} files with m-ex code, {} blobs, {} errors", self.files_scanned, self.blobs.len(), self.errors.len());
        let insns: usize = self.blobs.iter().map(|b| b.instructions).sum();
        let data: usize = self.blobs.iter().map(|b| b.data_words).sum();
        let unr: usize = self.blobs.iter().map(|b| b.unreached).sum();
        let _ = writeln!(
            s,
            "  instructions analysed {insns} (+{data} words of in-blob data tables, {unr} unreached words); memory accesses {}",
            t.by_class.values().sum::<u64>()
        );
        for c in [Class::Stack, Class::Global, Class::Mapped, Class::ArrayElem, Class::TypedNoField, Class::Opaque, Class::Unknown, Class::Indexed] {
            let _ = writeln!(s, "    {:<13} {}", format!("{c:?}"), t.get(c));
        }
        let _ = writeln!(
            s,
            "  struct-offset accesses (not stack, not global): {}; mapped to a native field: {} ({:.1}%), {} of them at a field's first byte with its size",
            t.struct_total(),
            t.get(Class::Mapped),
            100.0 * t.coverage(),
            t.exact
        );
        let _ = writeln!(s, "  distinct fields touched: {}", self.fields.len());
        let mut op: Vec<_> = self.opaque.iter().collect();
        op.sort_by(|a, b| b.1.cmp(a.1));
        let top: Vec<String> = op.iter().take(8).map(|(k, v)| format!("{k} {v}")).collect();
        let _ = writeln!(s, "  opaque bases (pointer loaded from an untyped field): {}", top.join(", "));
        for b in &self.blobs {
            let _ = writeln!(
                s,
                "  {:<40} fns {:>4} insns {:>6} seeds {:>3}  struct {:>5} mapped {:>5} ({:>5.1}%)",
                b.label,
                b.functions,
                b.instructions,
                b.seeded,
                b.counts.struct_total(),
                b.counts.get(Class::Mapped),
                100.0 * b.counts.coverage()
            );
        }
        for e in &self.errors {
            let _ = writeln!(s, "  error: {e}");
        }
        s
    }

    pub fn json(&self) -> String {
        // Hand-written JSON (no serde in the workspace lock). Fields sorted by C path.
        let mut o = String::from("{\n");
        let t = &self.total;
        let _ = writeln!(o, "  \"files_scanned\": {},", self.files_scanned);
        let _ = writeln!(o, "  \"blobs\": {},", self.blobs.len());
        let _ = writeln!(o, "  \"struct_accesses\": {},", t.struct_total());
        let _ = writeln!(o, "  \"mapped\": {},", t.get(Class::Mapped));
        let _ = writeln!(o, "  \"mapped_exact\": {},", t.exact);
        let _ = writeln!(o, "  \"coverage\": {:.4},", t.coverage());
        o.push_str("  \"by_class\": {");
        let cls: Vec<String> = t.by_class.iter().map(|(k, v)| format!("\"{k:?}\": {v}")).collect();
        o.push_str(&cls.join(", "));
        o.push_str("},\n  \"per_blob\": [\n");
        let rows: Vec<String> = self
            .blobs
            .iter()
            .map(|b| {
                format!(
                    "    {{\"blob\": {}, \"functions\": {}, \"instructions\": {}, \"data_words\": {}, \"unreached\": {}, \"struct\": {}, \"mapped\": {}}}",
                    crate::report::q(&b.label),
                    b.functions,
                    b.instructions,
                    b.data_words,
                    b.unreached,
                    b.counts.struct_total(),
                    b.counts.get(Class::Mapped)
                )
            })
            .collect();
        o.push_str(&rows.join(",\n"));
        o.push_str("\n  ],\n  \"fields\": {\n");
        let rows: Vec<String> = self
            .fields
            .iter()
            .map(|(k, u)| {
                format!(
                    "    {}: {{\"native\": {}, \"c_type\": {}, \"loads\": {}, \"stores\": {}, \"inside\": {}}}",
                    q(k),
                    q(&u.native),
                    q(u.c_type),
                    u.loads,
                    u.stores,
                    u.inner
                )
            })
            .collect();
        o.push_str(&rows.join(",\n"));
        o.push_str("\n  },\n  \"typed_no_field\": {\n");
        let rows: Vec<String> = self
            .no_field
            .iter()
            .map(|(k, m)| {
                let v: Vec<String> = m.iter().map(|(off, n)| format!("\"{off:#x}\": {n}")).collect();
                format!("    {}: {{{}}}", q(k), v.join(", "))
            })
            .collect();
        o.push_str(&rows.join(",\n"));
        o.push_str("\n  },\n  \"opaque_origins\": {\n");
        let rows: Vec<String> = self.opaque.iter().map(|(k, n)| format!("    {}: {n}", q(k))).collect();
        o.push_str(&rows.join(",\n"));
        o.push_str("\n  },\n  \"errors\": [");
        o.push_str(&self.errors.iter().map(|e| q(e)).collect::<Vec<_>>().join(", "));
        o.push_str("]\n}\n");
        o
    }
}

/// JSON string literal.
pub fn q(s: &str) -> String {
    crate::pack::jstr(s)
}
