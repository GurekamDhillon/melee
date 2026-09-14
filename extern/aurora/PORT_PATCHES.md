# Port patches to Aurora

This directory is a vendored copy of [encounter/aurora](https://github.com/encounter/aurora)
(MIT — see [`LICENSE`](LICENSE)), taken at upstream commit
`749d6ee7a22bdfab78c8ece9047bca5d79aa72ca`. Seven files carry changes made for this
port. They fall into three groups.

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

## 4. Memory card

`lib/card/CardGciFolder.cpp`:

- `openFile` returns `NOFILE` rather than `NOCARD` for a missing save (the game
  read `NOCARD` as "no card inserted").
- `deleteFile` implemented for both overloads.
- `renameFile` moves the file on disk and commits the directory.

Matches the memory-card behaviour the game expects (see `docs/DEVLOG.md` §13.2).
