# Draft PR for Hero88go/melee-unlocked: core v1.1 (not sent)

Branch `gd/core-v1` (local clone `_build/agents/corev1/wt`): 4359db5 (core v1), then f7d55cf (v1.1). Neither is
pushed. Send this only after the user decides; see memory "melee-unlocked plan" (a surprise PR once it has an edge).

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
- No change to the native game, its rollback or Slippi.

**Tested** with an independent game library written from the header alone: a Rust engine, outside this tree.
- Battlefield VS: 59.9 fps, and its mix byte-identical through `--audio-dump`.
- Five more stages: hashes identical to the library's own build.
- Its savestates under the state test: 50 rollbacks with 0 mismatches.
- His card: create, read, write, rename and delete through his card calls, and the game's save made through them.
- His controllers via `--script`.
- v1/v1.1 in both directions.

**Not done here**
- His rollback does not call `state_*` yet.
- The AppContainer mapper's 24 MB range is smaller than that library (39.8 MB). It also sets up no implicit TLS.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01DxRt2NWEXSebmoGPzdqDtU
