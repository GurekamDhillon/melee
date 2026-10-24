# Shard 945–952: JPEG

Both JPEG TUs are implemented in `rust/melee_runtime/src/tu/hsd_3B34.rs` and
`hsd_3B5C.rs`: 26/26 C definitions, including all static and inline helpers.
No earlier JPEG implementation existed in the crate. All records come from the
single `melee_types` generator, including the ten C-local records added for this port.

| TU | C definitions mapped to identically named Rust functions |
|---|---|
| hsd_3B34.c (16) | jpegLumaAddress, hsd_803B3408, fn_803B376C, hsd_803B3CD8_bit_length, jpeg_run_bit_length, hsd_803B3CD8_write_byte, hsd_803B3CD8_write_bits, jpeg_write_run_payload, jpeg_encode_component, hsd_803B3CD8, hsd_803B46D4, hsd_803B4A2C, hsd_803B4D64, hsd_803B51C8_inline, hsd_803B51C8, hsd_803B5C2C |
| hsd_3B5C.c (10) | hsd_803B5C4C_read, hsd_803B5C4C, hsd_803B5D70, hsd_803B5EA0, fn_803B61B4, jpeg_clamp, jpeg_store_rgb565, fn_803B6820, hsd_803B6BE4_inline, hsd_803B6BE4 |

## Native C verification

`python rust/melee_runtime/oracle/build_shard_jpeg.py` extracts the actual C
definitions, compiles with `-ffp-contract=on`, and applies `oracle/lower.py`.
The retained IR has 9 f32 contraction sites and 26 f64 contraction sites.
Rust uses `fmadds` for the former and f64 `mul_add` for the latter.

`cargo test -p melee_runtime --test oracle_shard_jpeg` passes 3,200 generated
rows. The corpus contains no disc-derived data. Coverage includes:

- Forward and inverse DCT, including full-range i32 inputs, with all 64 words checked.
- Tiled RGB565 conversion in both directions, with all planes and output bytes checked.
- Bit lengths, clamp and RGB565 pixel helpers; all DC/AC Huffman table entries.
- Byte stuffing, payload writes, and exhaustion at each header/output capacity.
- Component coding and decoding, coefficient arrays, bit counters and stream positions.
- Complete encode/decode entry points and both inline implementations; all 451 state
  words, quantization tables, partial metadata copies, encoded bytes, and decoded pixels.

Every C helper is exercised through the real C call chain or directly. Four intentional
mutations (stuffing, forward DCT coefficient, inverse DCT coefficient, scan marker) were
caught, restored, and followed by a passing targeted test.

## Native adaptations and remaining boundaries

Game working records use native little-endian values. The JPEG stream and source
byte-encoded Huffman tables use explicit big-endian conversion. The oracle consolidates
the original linker-adjacent arrays into contiguous storage, and uses a host setjmp
target separate from the 0x118-byte GC jump-buffer layout. These are addressing/storage
adaptations, not changes to the codec algorithms. The encoder's s32 image address becomes
a native pointer. Rust replaces the local setjmp/longjmp path with a private unwind
payload caught only at the corresponding codec entry, preserving partial state.

Two exact-name external boundaries remain in `hsd_3B34::ext`:
`hsd_803B3344` and `hsd_803B3398` from hsd_3B33.c. Their defaults explicitly report TODO;
tests install deterministic output-writer hooks. Complete encoding needs those hooks
until hsd_3B33.c is integrated. Standard memory operations are native Rust.

Invalid Huffman indices/run lengths that would overrun C arrays fail safely; behavior
after the original C's out-of-bounds access is not asserted. Callers must supply valid,
appropriately padded source/destination allocations, as with the C API. The global codec
state remains non-reentrant. This verifies native C equivalence, not game boot, rendered
screens, replay hashes, or a complete JPEG standards implementation.
