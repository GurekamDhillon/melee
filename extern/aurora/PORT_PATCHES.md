# Port patches to Aurora

This directory is a vendored copy of [encounter/aurora](https://github.com/encounter/aurora)
(MIT — see [`LICENSE`](LICENSE)), taken at upstream commit
`cb0e279` ("Fix imgui texture upload race", updated from `749d6ee7a22bdfab78c8ece9047bca5d79aa72ca`).
Nine files carry changes made for this port. They fall into five groups.

To update: clone upstream, commit the port's changed files onto the old base commit (with LF
line endings), rebase onto the new upstream head, then replace this tree with the result
(`git archive`) and keep this file.

## 1. 32-bit pointer portability

The port builds for 32-bit x86 with `/LARGEADDRESSAWARE`, so guest pointers are host
pointers and `reinterpret_cast<u64>(pointer)` is not valid. Each site is routed
through `uintptr_t`:

- `lib/dolphin/gx/GXExtra.cpp`
- `lib/dolphin/gx/GXFrameBuffer.cpp`
- `lib/dolphin/gx/GXGeometry.cpp`
- `lib/dolphin/gx/GXTexture.cpp` (two sites: texture and TLUT metadata)
- `lib/gx/command_processor.cpp` (four `handle_aurora` sites: texture, TLUT,
  copy destination, copy-tex destroy)

## 2. MSVC narrowing

- `lib/gfx/texture_replacement.cpp` — two `ByteBuffer{size}` constructions need an
  explicit `static_cast<size_t>` (MSVC C2398).

## 3. Indexed vertex arrays and draw merging

`lib/gx/command_processor.cpp`:

- **`attr_stream_offset` / `max_index_for_attr`** — new helpers that compute the
  highest index an indexed attribute actually references across a draw's vertices.
- **`push_gx_draw`** — upload only the referenced extent of an indexed array
  instead of the whole nominal array, and assert against the size actually needed.
- **`draw_prim`** — read the vertex data *before* deciding whether to merge, and
  refuse to merge a draw whose vertices index past the previous draw's uploaded
  array snapshot. A merged draw is folded into the previous draw command and never
  reaches `push_gx_draw`, so it inherits that draw's `arrayStart[]` storage, which
  only covered the first draw's maximum index. Merging then made later vertices
  read past the end of the snapshot — a few vertices landing on garbage while their
  neighbours were correct, which rendered as spikes through otherwise sound
  geometry.
- **`handle_draw_unmerged`** — now takes the vertex data so the above can run.
- **`push_gx_draw`** — when the referenced extent of an indexed array grows past the
  cached snapshot, grow the snapshot **geometrically** (doubling, clamped to the
  array's reported size) instead of re-uploading exactly `needed` bytes. A model
  drawn as many per-triangle indexed draws (Melee's envelope path — Sonic's m-ex
  model, which has no pre-built display list and references its arrays with a
  monotonically increasing max index) otherwise re-uploads the whole array once per
  triangle: O(n²) bytes into the storage buffer, overflowing its 8 MiB frame budget.
  The doubling is applied only to *bounded* arrays (the port's shim reports MEM1
  arrays as `[base, end-of-MEM1)`); an unbounded array (`UINT32_MAX`) is still
  uploaded exactly `needed`, never over-read.

## 4. Memory card

- `lib/dolphin/card.cpp` — `CARDSetBasePath` keeps the supplied base path when Aurora has no
  game name yet (`g_gameName` is only set by Aurora's DVD layer, which the port disables), and
  `CARDInit` resolves it with the game code it is given. Upstream dropped the path in that case
  and fell back to `<userPath>/<region>`, so the port's card folder (`_build/card`,
  `MELEE_CARD_PATH`) was silently replaced by `%AppData%/Melee PC`.
- `lib/dolphin/card.cpp` — `CARDGetStatus` no longer logs an error for `NOFILE`. The game
  scans all 127 directory slots at boot and an empty slot answering `NOFILE` is normal; it
  logged ~126 errors per boot.

The port's earlier card patches (`openFile` returning `NOFILE`, `deleteFile`, `renameFile`)
were dropped at the `cb0e279` update: upstream's card rewrite ("CARD: Fix error handling &
bugs, use temp files") implements all three.

## 5. Input latency (lane beta, B2)

- `lib/gx/command_processor.cpp` — `bytes_equal`, an SSE2 64-bytes-per-iteration equality test,
  replaces `std::memcmp` in `reuse_array_upload` and `revalidate_array`. Those snapshot checks
  compare tens of megabytes a frame in a match and the compiler inlined memcmp 4 bytes at a time;
  the FIFO thread spent ~5.8 ms a frame there, which the game thread waits out in `fifo::drain`
  at the end of every frame. Measured on ACE (MELEE_PROFILE_SAMPLE): 5318 -> 2792 samples,
  `aurora_end_frame` 7.7 -> 6.0 ms. It is memory-bound now; the rest needs fewer comparisons, not
  faster ones.
- `lib/gfx/frame.cpp`, `include/aurora/gfx.h` — `aurora_get_last_present_ns()` and
  `aurora_get_present_count()`: when the render worker last returned from `Present()`, so the
  port can measure input-to-present latency (MELEE_INPUT_PROFILE). The port links against an
  older library too (`/alternatename` fallbacks in `shim_vi.c`).
