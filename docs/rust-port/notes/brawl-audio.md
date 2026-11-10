# Brawl audio (runtime, from the player's disc)

Envoy needs the player's own Brawl disc. Its music and sound effects are read from that disc at runtime only, the
same way as Petey's model (`melee_modes/src/tu/petey_model.rs`). Nothing from the disc is shipped, committed, bundled or
cached on disk.

Code: `rust/melee_audio/src/brawl/` (`reader.rs` holds the file readers and `mod.rs` the mixer). Test:
`rust/melee_audio/tests/brawl_decode.rs`, which passes with a "skipped" note when the files are absent.

## Where the files come from

The search matches `petey_model.rs`:

1. `MELEE_BRAWL_LOCAL`: a folder that holds `disc/files/...`;
2. otherwise the first `_build/brawl` with a `disc/files` folder, walking up from the working directory and from the
   exe's folder.

Inside it, the code uses `disc/files/sound/smashbros_sound.brsar` and the `disc/files/sound/strm/*.brstm` streams that
the archive names. `MELEE_BRAWL_ABSENT=1` makes the game behave as if the files were missing (for tests).

## Formats (clean room)

Everything was written from the public format descriptions, and no game code was looked at.

- **DSP-ADPCM**: 8-byte frames. Each frame has a header byte (the predictor index in the high nibble, the scale in the
  low nibble) and 14 signed 4-bit deltas. Each sample is `(delta << scale << 11) + c1*h1 + c2*h2 + 1024 >> 11`,
  clamped to s16. Every channel has 8 coefficient pairs plus starting history.
- **RSTM (.brstm)**: the HEAD block points to the stream info, the track table and the channel table.
  - The stream info gives the format, the loop flag, the channel count, the rate, the loop start (in samples), the
    total sample count, the absolute data offset, the block count, the block size and samples per block, and the
    last block's samples and padded size.
  - The channel table gives each channel's ADPCM parameters.
  - Data blocks interleave the channels block by block. The whole stream is decoded once, in a worker thread, to
    interleaved s16. A loop then wraps from the end back to the loop start. A sample-continuous decode needs no
    per-block history.
- **RSAR (.brsar)**: the header lists SYMB, INFO and FILE.
  - SYMB: a string table.
  - INFO: 8-byte references (`u8 is_offset, u8 type, u16, u32 offset` relative to INFO+8) to five tables: sounds,
    banks, players, files and groups. Each table is `u32 count` followed by `count` references.
  - Sound entry: string id, file id, player, 3D params, volume, priority, **type** (1 sequence, 2 stream, 3 wave
    sound), and a detail reference. For a wave sound, the detail's first word is the sound's index inside its RWSD.
  - File entry: sizes, an external name (for streams, e.g. `strm/X18.brstm`), and a table of (group, item) places.
  - Group entry: the absolute offsets of its file data and its wave data, plus an item table. Each item is
    `file id, data offset, data size, wave offset, wave size`, relative to the group.
- **RWSD**: a DATA block (a table of wave sound entries, each with info, track and note references) and a WAVE block
  (a table of wave infos).
  - We play the sound's first note: its wave index and its pitch (f32 at note+0x10).
  - The wave info holds the format (0 PCM8, 1 PCM16, 2 ADPCM), the loop flag, the channels, the 24-bit rate, and the
    loop and end positions. For ADPCM these are nibble addresses: 16 nibbles hold 14 samples.
  - It also holds the channel table: each channel's data offset and ADPCM parameters, relative to the wave's data
    location in the group's wave data.

What the test decodes from a real disc:

- 10348 sounds: 371 streams and 9457 wave sounds.
- The boss stream: stereo, 32 kHz, 73.6 s, with a loop.
- Petey's `snd_se_boss_Bosspackun_00`.
- Every 97th wave sound.

Sequenced sounds (RSEQ + RBNK) are not played. Their notes need a sequencer.

## Playback

`melee_audio::brawl` mixes on top of the game's output. `Sound::sub_frame` calls `brawl::mix_into` after the AX mixer.
While nothing Brawl is playing, that call is one relaxed atomic load and adds nothing. So captures, replays and the
gate's audio hash are unchanged. The sim never reads any of it.

- **Music** (`play_music(name)`): a looping stream on the music channel, one at a time, resampled to 32 kHz.
  - Starting it stops the game's `.hps` stream and holds off new `.hps` starts until the music stops
    (`Sound::pstream_start_ch_param` checks `brawl::music_on()`).
  - `stop_music(frames)` fades the music out.
  - Gain: `MELEE_BRAWL_MUSIC_VOL` (percent, default 60), then the master volume.
- **SFX** (`play_sfx(name)`): a one-shot, with up to 16 at once.
  - Gain: the sound's own archive volume times `MELEE_BRAWL_SFX_VOL` (default 70).
- Names: a sound's archive name (`snd_bgm_X18_BOSSBATTLE`, `snd_se_boss_Bosspackun_03`), `#<id>` for a sound
  index, or a bare stream file stem (`X18`) for music.
- Files load on a `brawl-audio-load` thread, so a request starts a few milliseconds later. A newer request or a stop
  cancels a load still in flight.
- **Missing files**: every call is a silent no-op. `play_*` returns false, and the reason goes to stderr once per
  request.

## Petey's fight

Petey's fight (`MELEE_ENVOY_BOSS=petey`) plays **`snd_bgm_X18_BOSSBATTLE`** (`strm/X18.brstm`). This is Brawl's
"Boss Battle Song 1", the theme the Subspace Emissary bosses share. The archive has no Petey-specific stream.

Other stream names that mention bosses:

- `snd_bgm_F03_BOSSMEDLEY`: a Kirby boss medley, a stage track.
- The `snd_bgm_Y01_ADV01`..`Y30_ADV30` adventure tracks, named only by number.

The constant is `melee_modes::tu::petey::BOSS_MUSIC`.

How it is driven: `petey::tick` bumps `petey::HEARTBEAT` while the boss is live. `engine_audio::petey_music` starts the music on the first beat. After 2 s with no beat (the match scene has ended), it fades the
music out over 30 frames. The next scene's own music then plays again.

## Mod API (wasm host imports, module `env`)

| import | signature | does |
|---|---|---|
| `gd_brawl_present` | `() -> i32` | 1 if the Brawl sound files are present. **UI only**: never feed it into sim state. |
| `gd_brawl_music` | `(ptr: u32, len: u32) -> i32` | Loops the named stream as music (the name is UTF-8, at most 64 bytes). Always returns 0. |
| `gd_brawl_music_stop` | `(frames: i32) -> i32` | Fades the Brawl music out over `frames` (0..600; 0 = at once). Always returns 0. |
| `gd_brawl_sfx` | `(ptr: u32, len: u32) -> i32` | Plays the named wave sound once. Always returns 0. |

The play calls always return 0, so a mod's logic cannot tell whether the disc is present, and a replay or run record
stays the same with or without the disc. These calls are not aware of rollback: a resimulated frame would play an SFX
again. Envoy is offline, so call them only from offline paths.

Example (Rust mod):

```rust
extern "C" { fn gd_brawl_sfx(p: *const u8, n: u32) -> i32; }
let n = "snd_se_boss_Bosspackun_03";
unsafe { gd_brawl_sfx(n.as_ptr(), n.len() as u32) };
```

`petey_music` runs on both audio paths: from `Audio::tick` in live play, and from `engine_audio::dsp_frame` in the
headless golden. The first build had only the golden call, so live play stayed silent. Each state is logged once to
stderr, prefixed `brawl audio:`:

- the archive found or not found;
- the theme requested or skipped;
- the music started, with its channels, rate, length and loop;
- the music fading out.

The Brawl mix is added before the master volume, so `MELEE_VOLUME` scales it exactly like the game's own sound.

Verified with a headless golden of Envoy fight 5 (Petey), captured with `--audio-wav`, once with the Brawl files and
once with `MELEE_BRAWL_ABSENT=1`. With the files, the boss theme replaces the stage music (per-second RMS difference
2.2k-5k). The sim hash CSVs of the two runs are identical.
