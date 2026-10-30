# Traps index

The traps found along the way live in the topic note of the work that hit them. Grep `Trap` in
`docs/rust-port/notes/` for the full set; the cross-cutting ones, by file:

- golden.md: recording traps (MSVC `current_zone()` crash at boot, `gd.input(..., 1)` never read in turbo, read-only
  `.git` packs in a fresh lane) and the per-round "Traps found" lists (each cost a bisect).
- fma.md: shipping `-ffp-contract=on` fuses `a*b+c`; `gk_fmadds` is f64-then-f32, not `f32::mul_add`.
- types.md: the C typedef `Vec` shadows `std::vec::Vec` under `use melee_types::*`.
- oracles-mtu.md: `bool` is `int`; `uintptr_t` 4 vs 8 bytes; unions aliasing bitfields differ native vs GC bit order;
  `-1.0f * x` folds to a negation (NaN sign); asserts in `extern "C"` ports abort the test.
- render.md: the game calls GXInit (aurora_initialize does not); `std::cos(float)` in Aurora on x86 MSVC is double cos.
- script.md: an error raised by a Rust callback reaches Lua `pcall` as a userdata; write files with Write/Edit, not heredocs.
- integration.md: C evaluates call arguments right to left on the MSVC target.
