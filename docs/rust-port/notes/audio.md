# Audio (melee_audio and the audio proofs)

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 400-417 -->
## Platform track, night 3 step 4: melee_audio connected (2026-10-21)
- `melee_audio::Sound` = AX + 16 MB ARAM + the synth.c and axdriver.c state; `Sound::boot(fs, banks)`
  is lbAudioAx's audio init (HSD_AudioInitMultiPStream, reverb-std 1.88 on aux A, delay on aux B,
  three banks, stereo, `/audio/us/smash2.sem`, `/audio/us/main.ssm` into bank 0). `sub_frame()` =
  HSD_SynthCallback (8-node sweep, envelope, AXDriverCallback, PStream clock) + the AX mix.
- Sim side: `melee_audio::sim::{install, lbaudioax_800237a8, lbaudioax_80023870, lbaudioax_80024b94,
  virtual_frame, Output}`. `melee golden --audio` installs it and sets the fighter hooks (minimal
  call-site change in rust/melee: Cargo.toml dep, main.rs). The golden's frames never reach a sfx
  call yet, so its CSV is unchanged.
- Audible check: `cargo run -p melee_audio --release --bin sfx_play -- --iso <disc> [--id 0xC1]
  [--bgm menu01.hps] [--wav out.wav] [--scan N]`. `--scan` lists each main.ssm script's handle,
  length and peak headless.
- Floats: axdriver's sqrtf/powf are the game's (melee_runtime gekko::sqrtf, lb_00ce::powf),
  synth's sqrtf_accurate is gekko's; no fused site in either file.
- Left: a C oracle for synth/axdriver (needs the game-TU retarget pipeline), group 1's lbAudioAx
  tables (bank lookup, SSM-per-scene loading, fighter voice limiter 80026510/800265C4), a mixer
  thread, user listening test.


<!-- RUST_PORT.md lines 1329-1342 -->
## Platform track, audio proof (plat-audio, 2026-10-24)
- C side: `MELEE_AUDIO_TRACE=<path>` (shim_ax.c `gw_ax_trace`) logs every game-facing axdriver.c / synth.c call
  (`<sub-frame> OP args`; START with its returned handle, KO*, PAN/VOL/PITCH/MIX, MIXGRP, PAUSE/RESUME, PSTART,
  UAV, SSV); calls made inside the synth callback are not logged. With `MELEE_TURBO=1` + `MELEE_AUDIO_DUMP` the C
  mix is on the virtual 5 ms clock and run-to-run identical (two runs: identical trace and WAV).
- Port side: `melee_audio::replay` + `sfx_replay` bin replay a trace into the Sound (boot, `lb_audio_ax_80028690`
  scene init, `lb_load_announcer` = gm_LoadAnnouncer / nr_vs.ssm, match banks) and `--check` against the C WAV.
- Result (pad_fixed.lua Falcon v Fox FD, 6106 sub-frames = 30.5 s): 1953920 samples, 0 differ, 176/176 voice handles
  equal; `melee_audio/tests/replay.rs` pins the trace + C mix fingerprint (needs the disc).
- Fix found by it: the C runs in sound mode 0 (mono, `OSGetSoundMode` shim); the port's boot forced stereo.
- Not yet: the native sim's own SFX calls (golden --audio) match the C bit for bit only up to the first
  game-driven sound (sub-frame ~2003; the sim does not issue the announcer/entry sounds yet), so the end-to-end
  native capture is not equal; positional/KO/PAN paths unused in this scenario (no trace lines).


<!-- RUST_PORT.md lines 1488-1549 -->
## Platform track, audio round 2 (plat-audio, 2026-10-25): pause and time-up proven against the C mix
- No OS input anywhere: every run below is `melee golden --audio --idle-pads` (headless, virtual DSP clock) against the C build
  on `MELEE_TURBO=1` + `MELEE_AUDIO_DUMP` with Lua pad scripts. `golden` gained `--idle-pads`, `--cpu LEVEL`, `--pause F:T`
  (START after logic frame F, resume T-F frames later, the audio running on the held frames) and `--time-secs N` (the clock).
- Pause (pad_pause.lua, C trace `pse`): PAUSE x4 + the pause sound + duck at sub-frame 1110, RESUME at 1510, the indicator's
  sound (menu sound 4) two frames after the resume. 2440 sub-frames, 0 samples differ (`native_pause_resume_mix_equals_c`).
- Time-up (pad_idle_long.lua, 1 minute, Falcon v Fox FD): the clock starts when Go has played out (`go_frame + 40`), callouts
  5..1 on the 60 frame marks before `clock_end`, the sim holds from `clock_end - 1`, then fn_8016C7F0's audio: AXDriverStop
  (music fades), KOALL, lbAudioAx_80024C84, lbAudioAx_80024D50 (the reset twice), and TIME! nine frames after the hold. 12856
  sub-frames identical (`native_timer_callouts_and_time_up_mix_equals_c`); the C then loads sudden death (a new scene).
  The play loop used `frame >= secs*60` for the time-up, 124 frames early; it now uses `audio::clock_end`.
- `Audio::match_leave` is once per match (`in_match`), and undoes the pause duck only when it is on (a normal leave emits no extra RESUME).
- Still open: GAME! (stock-out) call timing and leave order unproven; results/sudden-death scene audio; the sound-options volume
  settings and positional SFX through the live `melee play` loop have no C capture compared yet (positional/KO paths are in
  `melee_audio::posobj` and covered by the trace replays only where a scenario triggers them).

- Round 3 (plat-audio, 2026-10-25): re-verified after merging port/platform-integ: `melee_audio` (14) and the 4 `audio_match` tests pass, all headless. Master volume
  (`MELEE_VOLUME` / audio.cfg, applied as output gain only, so `MELEE_AUDIO_DUMP` stays full scale as in shim_ax.c) is ported in `output.rs`. GAME! cannot be compared yet:
  the native sim has no KO / blast-zone / death path (only the C build can `gd.teleport` a fighter out and `gd.set_stocks 1`), so a stock-out capture needs the
  fighter Dead* states first. Results / sudden-death scene audio and in-game menu sounds depend on the stand-in ui.rs menus.
- Round 4 (plat-audio): merged port/platform-integ again; added `output::tests::master_volume_env_clamps_and_sets_gain` (MELEE_VOLUME parse/clamp like gw_Audio_Volume, gain = pct/100 on the device path); 15 melee_audio tests pass, headless, no OS input. Open items unchanged (GAME! needs sim death path; results/sudden-death/menu audio need the real menus).

## Platform track, audio round 5 (plat-audio, 2026-10-25): menu / title / results / sudden death replayed against the C mix
- No OS input, headless (`MELEE_TURBO=1`, virtual DSP clock, Lua / text pad scripts). The C trace (`MELEE_AUDIO_TRACE`) gained the
  ops a scene-spanning capture needs: `BANKREQ path bank entry` / `BANKDONE entry bank` (HSD_SynthSFXLoad request and its
  completion), `BANKUNL` / `BANKDFL` (unload, deflag), `DSTOP` / `DPAUSE` / `DRESUME` (AXDriverStop/Pause/Resume), `SHDR` / `SDATA`
  (the HPS start's deferred DevCom completions: header, first block). `MELEE_AX_DBG=<lo>-<hi>` prints the C mixer's per-voice
  state (and every AXSetVoiceVe / VeDelta) for those sub-frames on stderr.
- Port side: `replay` applies them (`Sound::defer_stream` is turned on by a trace that has SHDR: PSTART then waits for the
  recorded completions); `Sound::boot_engine` / `replay::boot_bare` = the audio engine with nothing loaded, so a trace brings
  its own banks; `sfx_replay --bare`.
- What it found: AXDriverPause right after a stream start (opening.hps) runs between the start and the first-block completion, so
  the C starts the voice from volume 0 with a ramp (flags 14 at the first-data callback); the port completed the start
  synchronously and was +-1..3 off for 8 sub-frames. With the completions replayed in place: 0 samples differ.
- Proven sample for sample (`melee_audio/tests/replay.rs`, needs the disc): opening movie + title + attract demo (16412 sub-frames,
  opening.hps, per-scene fighter banks, nr_vs, greatbay.hps); CSS cursor / A / B / START clicks (5650); Falcon v Fox FD stock-out
  with GAME!, the KO and the results screen with its victory theme (5780, 18 handles equal); one-minute time match through TIME!
  into the sudden-death scene (18233, 29 handles equal). Two C runs of the KO capture give identical trace and WAV.
- Fixed: `melee/tests/audio_match.rs` shared one temp dir between its three falcon_fox scenarios and failed ~1 run in 3 when run in
  parallel; each scenario has its own dir now (8 parallel runs pass).
- Still open: the native sim cannot issue the GAME! / KO / results calls itself (no death path), so those are proven for the audio
  engine given the C's calls, not for the sim's call timing; the Atlas main menu / SSS / trophy screens were not driven (the
  title Start lands in the Atlas menu scene; navigating it by pad script was not worked out); volume settings are output gain
  only (the C dump is full scale by design); nothing was listened to.

## Platform track, audio round 6 (plat-audio, 2026-10-25): retail main menu, stream block refills chunk by chunk
- Reached the retail main menu headless: `MELEE_ATLAS=0`, `MELEE_TURBO=1 MELEE_UNATTENDED=1`, Lua pad script: A past the memory
  card and movie, START at the title, A through the unlock notices until `GS_FRONTEND`, stick up/down on the menu, A into `GS_MENU`
  (1P / VS), B back. (Run `tools/port/run.sh` WITHOUT `--test` for this; `--test` runs the self-test suite instead. Export
  `GW_MELEE=<worktree>` too or build.sh builds the shared `melee/` checkout.)
- Mismatch found: 31 sub-frames differed (max 9267) where one HPS stream replaced another (s_info1 -> s_info2 ...). The new
  stream's first master-clock tick refills the next block into the slot the OLD stream is still fading out of. The C moves that block
  as DEVCOM_BUF_SIZE (16 KB) ARQ chunks, one per DVD completion, so the old voice kept its data for ~7 sub-frames; the port wrote the
  block at once. New C trace ops: `HHDR` (HSD_SynthPStreamHakoHeaderCallback), `HDONE` (HSD_SynthResetStreamCounters), `DCHUNK n`
  (each 0x23 DevCom chunk's ARQ post, devcom.c). The replay applies the header at HHDR, each chunk at its DCHUNK, the counters at
  HDONE (`Sound::refill_chunked` / `defer_refill`; traces without these ops keep the synchronous refill).
- Pinned: `melee_audio/tests/replay.rs::frontend_menu_streams_mix_equals_c` (4483 sub-frames, 17 handles, 0 samples differ, needs
  the disc). melee_audio (15 unit, 2 oracle, 8 replay) and melee audio_match (4) pass. Two C runs gave identical WAVs.
- Still open: the older four captures were not re-recorded with the new ops (they match with the synchronous refill); the SSS, trophy
  and Atlas menus were not driven; nothing was listened to. Debug aid: `MELEE_REPLAY_DBG=lo-hi` prints the port's per-voice mixer
  state like the C's `MELEE_AX_DBG`.


<!-- RUST_PORT.md lines 1667-1675 -->
## Platform track, audio round 7 (plat-audio, 2026-10-25): driving past the main menu (no new capture)
- Run recipe gotchas for headless menu captures: pass `MELEE_PAD_SCRIPT` / `MELEE_AUDIO_*` as `E:/...` paths (an MSYS `/e/...`
  path silently loads no script and the run idles to the timeout), and seed a fresh sandbox with a copy of an existing
  `runs/<x>/card` (a sandbox without a memory card never leaves the card notices).
- From `GS_FRONTEND` the A button goes Versus -> Melee -> the Lua-hosted character select (the kit UI, scene name stays
  `GS_FRONTEND`). Scripted A / stick down / START presses (3 or 8 frames, up to 200 frames of waiting) did not change that screen
  in turbo (screenshots identical), so the SSS and the match start were not reached this way. Next step if wanted: an in-engine
  scripted-input hook for the kit UI, or `MELEE_SCENE` presets for the retail SSS. No audio fixture was added this round; the
  six pinned captures still match the C mix sample for sample. Nothing was listened to.
