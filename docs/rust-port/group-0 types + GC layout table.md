# Port group 0: types + GameCube layout table (`melee_types`)

Status 2026-10-21. Crate: `rust/melee_types`. Commits `c1fd50c1d`, `3e0cb411a` on `rust-port`.

## What is there

| piece | file | how it is made |
|---|---|---|
| every struct / union / enum / typedef / numeric constant of the group-0 headers (and every record they pull in: dolphin, MSL) | `src/gen.rs` (@generated, ~2.5 MB) | `python rust/melee_types/gen/gen_types.py` |
| the header list | `gen/all_types.c` | one `#include` per `src/melee/*/{types,forward}.h` and `src/sysdolphin/baselib/*.h` |
| neutral hash NHV1 over native structs + `StateHash` for `Fighter`, `CpuFighter`, `Item` | `src/neutral_hash.rs` | hand port of `nh_*` in `src/melee/ft/fighter.c` |
| 82 of the 110 `static inline` functions defined in the group headers | `src/inline.rs` | hand port |
| C oracle for the hash words | `gen/nh_oracle.py` -> `tests/data/nh_expected.txt` | extracts the C functions from fighter.c verbatim, compiles them natively with clang |

Counts (generator output): 1576 records, 201 enums, 1229 typedefs, 437 `#define`/`static const`
constants, 17 incomplete types (opaque, pointer-only), 1226 native-size compile-time asserts.

## The generator (`gen/gen_types.py`)

Needs Windows Python, LLVM at `C:\Program Files\LLVM` and `pip install --user libclang==18.1.1`
(only the Python bindings; LLVM's own libclang.dll is loaded). Run from the worktree root. Two
parses of `gen/all_types.c`:

* **ppc32-none-eabi -DTARGET_PC**, the flags of `_build/masstest/pipe_win.sh`: every GameCube
  offset, size, alignment and PPC bit position. This is the layout of the running PC build's guest
  memory too (TARGET_PC fields are appended, so retail offsets do not move; `Item` is 0xFD0, not
  0xFCC; `HSD_CObjDesc` gains 2 tail fields; `HSD_TObj.tlut_no` is wider). Every record size is
  cross-checked against `clang -Xclang -fdump-record-layouts-complete` (written to
  `gen/ppc32_layouts.txt`, gitignored: 0 mismatches).
* **x86_64-pc-windows-msvc -DTARGET_PC**: native C sizes. Every generated record with no
  bitfield and no `size_t` (deep) gets `const _: () = assert!(size_of::<T>() == N)` against
  it, so the Rust type mapping is checked by the compiler for 1226 records.

### Mapping rules (what later groups must know)

* Structs/unions: `#[repr(C, align(<native C align>))]`, `#[derive(Clone, Copy)]`, fields in C
  order. Packed records (any field off its alignment, e.g. `TmUnkMenuData`) are
  `#[repr(C, packed)]`.
* `long` = i32, `long double` = f64, MSL `bool` (`typedef int bool`) = i32, `size_t` = usize
  (so records holding a size_t differ from the native C build; they have no size assert),
  `va_list` = `*mut c_void`, `s8/s16/s32/s64` aliases are emitted, `u8..f64` are Rust's.
* Pointers: `*mut T` (`*const T` when the pointee is const). Function pointers:
  `Option<unsafe extern "C" fn(..)>`; function typedefs are fn-type aliases.
* Enums: `pub type Name = i32|u32;` plus `pub const Enumerator: Name = v;` (C enums hold
  values outside their enumerator set; a Rust enum would be UB).
* Anonymous records are named `Parent_field`, or `Parent_anon_<line>` when no field names them
  (line = line in the C header; stable while the header does not move).
* C11 anonymous members become a field `anon_<gcoff>`; their leaf fields are ALSO listed in the
  parent's `GC_FIELDS` with parent-relative offsets, so `Fighter::gc_field("x2219_b5")` works the
  way C spells it.
* Rust keywords get `r#` (`r#type`); `_` fields become `_x<gcoff>`; repeated names get `_<off>`.
* **The C typedef `Vec` shadows `std::vec::Vec` on `use melee_types::*`.** Write
  `std::vec::Vec` in code that glob-imports this crate.

### Bitfields

Each run of adjacent bitfields is one field `bf_<gc start offset>: [u8; n]` holding the GameCube
bytes in GameCube order (MSB first), with generated `name()` / `set_name(v)` accessors
(sign-extending for signed types). So the bytes are the GC image whatever the host endianness, and
a DAT/snapshot loader can copy them straight in. `GC_FIELDS` records them as
`GcField::bitfield(name, unit offset, unit size, first bit from the unit's MSB, width)`.

### The table (`melee_core::gc::GcLayout`)

Every record implements `GcLayout` (C name, GC size, GC align, `GC_FIELDS` in declaration
order, nested records referenced by their C type spelling). `melee_types::GC_RECORDS` lists all of
them for tools (DAT loader, netplay/Slippi serialisers, the future m-ex offset map). The table is
top-level per record: resolve `Fighter+0x1A88+0x4` by `Fighter::gc_field_at(0x1A88)` -> `cpu`
(`CpuFighter`) then `CpuFighter::gc_field_at(4)`. A per-field `GcField` names the C field, not
the Rust identifier (they differ only for keywords / `_` / anon members).

## Tests (`cargo test -p melee_types`: 10 tests, all pass)

* `tests/neutral_hash.rs`
  * `words_match_c_oracle`: nh_CpuFlags, nh_Fighter (CPU and non-CPU), item word, 8 cases,
    against the C functions compiled from fighter.c.
  * `game_combine_matches_goldens`: recomputes the `hash` column of all 4 golden CSVs
    (10804 rows) from their seed/items/itemsum/s0..s11 columns. Exact on every row.
* `tests/layout.rs`: Fighter/CpuFighter/Item GC offsets against docs/rust-port/neutral-hash.md,
  bitfield bytes are GC MSB-first, every field fits its record.
* `src/inline.rs` unit tests: JObj dirty rules, refcount wrap, fabsf_bitwise, objalloc flag bit.

## Not done / stubbed / blockers

* **28 header inlines not ported**, each needs a function another group owns: `HSD_MtxColMag`
  (MSL `sqrtf` = `__frsqrte` + 3 Newton steps; needs a bit-exact frsqrte table in melee_core/runtime),
  `vec_normalize_check` (PSVECNormalize), `atan2f_check`, `randi` (HSD_Randi),
  `HSD_AObjGet*Frame`, `HSD_PadGetNml*`, `HSD_VIGet*` (globals), `HSD_IDGetData`,
  `HSD_CObjGetViewingMtxPtrDirect`, `HSD_JObjRef/RefThis`, `hsdDelete`, `HSD_WObjUnref/ClearFlags`,
  `HSD_PerfCountMtxLoad`, `HSD_SisLib_BytePtr`, `HSD_JObjSetupMatrix`, `HSD_JObjGetMtxPtr`,
  `HSD_JObjCopyMtx`, `HSD_JObjSetMtxDirtyOutOfLine(Leaf)` (= `JObjDirty::set_mtx_dirty`).
* Function prototypes in the headers are not bindings: their bodies are other groups' TUs.
* `static` data in `*.static.h` (devcom, synth, axdriver buffers) is not emitted; it belongs to the
  TU that includes it.
* Function-like macros (`HSD_OBJ(o)`, `GET_COBJ`, `FTKB_IS_KIRBY`...) and non-numeric `#define`s are
  not emitted; port them in the group that uses them.
* No `.s` files belong to group 0 (inventory.md: the 5 `.s` files are Runtime/TRK/GX, all dead or
  replaced).
* No big-endian DAT decoder yet: the table is there; the loader (walk `GC_FIELDS`, byte-swap
  scalars, relocate pointers, copy `bf_*` bytes as-is) is runtime work.
* c2rust raw output was not used for this group: libclang over the headers gives the same types
  with exact layouts and no nightly-only extern types. (The headers have no TUs of their own.)
* `Fighter` / `Item` `StateHash` stream the neutral-hash fields only (not the whole struct), and
  `Fighter`'s leaves out stocks and the CPU check (other groups' functions).
