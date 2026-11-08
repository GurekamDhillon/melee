# Draft PR for Hero88go/melee-unlocked: core v1.1 (not sent)

Branch `gd/core-v1` (local clone `_build/agents/corev1/wt`): 4359db5 (core v1), f7d55cf (v1.1), then 1fca66c (the
sandboxed mapper: sized by the image, implicit TLS). None is pushed. Send this only after the user decides; see
memory "melee-unlocked plan" (a surprise PR once it has an edge). The state test in the sandbox needs this
branch's `arena::static_sections` fallback (the library finds its own image without the loader).

---

**Title:** A frozen core v1 boundary for game libraries, with sound and savestates (v1.1)

**Summary**

- `port/runtime/abi/mu_core_v1.h` is a small, frozen game/host table (ABI 1, entry `mu_game_entry_core_v1`): the
  calls a game library needs from the Source Port (pads, retrace, GX FIFO in console format, disc, card, alarms,
  MEM1). The full `MuHostApi` stays the native game's and keeps changing freely.
- v1.1 only appends, and each side tells the other's revision from `struct_size`:
  - host: `audio_out(lr, frames, rate)`, mixed 32 kHz stereo into `audio_push_native`;
  - game: `state_save / state_load / state_drop / state_hash / state_frame / state_input`, keyed savestates called
    between frames.
- `MELEE_COREV1_STATE_TEST=K` drives the state calls as a rollback would. `MELEE_COREV1_HOST_V1=1` offers v1 only.
- The sandboxed loader (`map_game_image`, inside an AppContainer) sizes the image by its `SizeOfImage` against
  the 64 MB range held from startup, not a fixed 24 MB, and sets up implicit TLS as the loader does for a DLL: a
  slot written to the image's index, a block per thread (template and zero fill), and thread attach/detach for
  its TLS callbacks and entry point through a TLS callback of the host's own.
- No change to the native game, its rollback or Slippi.

**Tested** with an independent game library written from the header alone: the Rust port of Melee,
branch `rust-port` of [github.com/GurekamDhillon/melee](https://github.com/GurekamDhillon/melee/tree/rust-port).
- Battlefield VS: 59.9 fps, and its mix byte-identical through `--audio-dump`.
- Five more stages: hashes identical to the library's own build.
- Its savestates under the state test: 50 rollbacks with 0 mismatches.
- His card: create, read, write, rename and delete through his card calls, and the game's save made through them.
- His controllers via `--script`.
- v1/v1.1 in both directions.
- Inside an AppContainer (the library is 40 MB with 360 bytes of TLS): Battlefield VS, the state test and v1/v1.1
  in both directions give the same hashes as outside it, and the same mix; 60 fps.

**Not done here**
- His rollback does not call `state_*` yet.
