# Data-layout audit (2026-11-05, branch port/audit-layout)

The failure class: the game reads big-endian GameCube data (archives, attribute tables, saves) and relies on the MWCC/PPC byte
layout of its structs (4-byte pointers, its padding, `bool` = MSL `int`); the port uses native little-endian layouts with
8-byte pointers. Past bugs of this class: big-endian padding tables in card saves, shape-anim POBJ vertices, Shift-JIS text, the
bool ABI (Ice Mountain), attribute tables. This audit checks the whole class mechanically, fixes what it found, and leaves the
checker in `tools/rust-port/layout/`.

## The checker

**Part A, record layout against the C** (`python tools/rust-port/layout/c_layout_check.py [--json OUT] [--verbose]`).
Every melee_types record (1982) against the C build's layout of the CURRENT headers and TU sources: libclang with the shipping
pipeline's flags (`_build/masstest/pipe_win.sh`: ppc32-none-eabi, -DTARGET_PC, -fno-short-enums, -fsigned-char,
-mlong-double-64). Each record is flattened to leaves, (GC bit offset, element count) -> scalar kind (width, signedness, float,
pointer), and each bitfield to (bit, width); those are compared with gen.rs's native field types at their GC offsets and its
GC_FIELDS bitfields. Field names are not compared (upstream renames them); nested and anonymous records are checked inside their
parents. Result: 1344 records match, 69 differ (all triaged in the table: none puts a different value into disc, hash or save
bytes), 194 have no C record of that name (port-only records in pc/ files the probe does not parse, such as Geno, script and AX
records, or records upstream renamed).

**Part B, every field over the whole disc** (`bash tools/rust-port/layout/run_walk.sh [OUT] [file-substring...]`; needs
GW_ISO_VANILLA; the output names disc files and values and stays in `build/layout/`, never committed). The walker loads every
archive on the vanilla disc through the game's own path (`lbFile_LoadInto` -> `lbArchive_InitializeDAT` -> the engine's
`host_parse`), takes every public root the root table types (`dat_types.rs`, `extattr.rs` for each fighter's `ext_attr`, and the
`ftdata` / `itco` / PlCo slot 0 / `dyntemplate` customs) and walks the native graph:
- every scalar is compared bit for bit with the big-endian word at its GC offset, which is what the C build reads (it keeps an
  archive as the file's relocated big-endian bytes in game memory);
- every pointer is checked against the relocation table (relocated <-> non-null; pointers into the raw image are recognised)
  and followed;
- arrays run to the count the generated decode uses (arrays.tsv), unions use the member the hand decoder picks (the HSD_Joint,
  RObjDesc and PObjDesc tags), and bitfield bytes are compared raw.

Native offsets come from `offset_of!` on the compiled structs (a generated table, build output only).
`layout_walk --identical` lists the records whose native layout is the GC layout. `LAYOUT_WATCH=Rec.field` prints every value
of one field. Result: 861 archives, 1945 roots, 2,492,741 scalars, 1,165,202 pointers (366,081 followed), 145 record types,
**0 mismatches**. Negative control: shifting one GC offset in the table (HSD_Joint.scale) gives 220 mismatches over 2 files.

Part A shows that the GC offsets and widths the decoder uses are the C compiler's. Part B shows that the native image holds the
C's values at those offsets. So for every typed root on the disc, the Rust reads what the C reads.

Part B does not cover these hand-typed roots, which have their own tests or are covered by the goldens:
- the effect banks (`efdat`, `ptclcmd`, `ptcltex`: particle.c relocation, see dat_custom);
- Kirby hats (`kirbyhat`, whose slots are rebuilt);
- the stages' item, light and model pointer tables (`ptrtable_*`);
- `sis` text;
- the raw byte readers (`rawptrs`, `rawwords`, `rawhalves`: 517 roots, read big-endian on purpose);
- the 4217 publics the root table does not type (if the game ever asks for one, the engine stops and names the symbol).

The `void*` attribute blocks (items' `x4_specialAttributes`, fighters' `dat_attrs`) are word-swapped copies, so their typed
views were checked statically. Every view type the Rust casts them to (162 `sa::<T>` / cast sites) either has the GC layout
natively (`--identical`), or is decoded by hand in dat_custom (ARTICLE_TYPED, FoodsRow, KinokoAnim, ...), or is read by GC
offset (`swi`/`swf`). The four that are none of these were read by hand: `itEvYoshiEgg_DatAttrs` (only `x0` is read, at offset 0),
`itSword_UnkArticle1` (read by GC offset), `it_2728_DatAttrs` (runtime only) and `_ftSamusAttributes` (= `ftSs_DatAttrs`, typed
by extattr and walked in part B).

## Found and fixed

1. **Peach's Castle flag dynamics read big-endian bytes as floats.** `dynamicsdata_flag3/4/6` are
   `DynamicsTemplate { params*, count, pos }`: lb_80011710 reads `params` as `count` 15-float `lb_00F9_UnkDesc1Inner` records.
   The root table decoded them as `DynamicsDesc` -> `DynamicsData`, whose union `PolymorphicDesc` decodes as raw bytes, so
   `lb_80011710` copied byte-swapped floats into the flags' dynamics. They are now `Custom("dyntemplate")`
   (`dat_custom::dyn_template`), which decodes `count` native records. Part B found it once it knew that the fighter path
   (`fix_dynamics`) uses the same record.
2. **Corneria, `grCorneria_801DFC2C`: `memzero(&gp->u.corneria.xC8, 0x18)`.** 0x18 GameCube bytes are xC8 and xCC (4-byte
   pointers there), xD0, base_x, base_y and offset_x. The port cleared 0x18 native bytes, which end after base_x because the
   pointers are 8 bytes natively, so base_y and offset_x kept their values. Each field is now cleared by name.
3. Pokemon Stadium, `memzero(text, 0x18)` (the GC `HSD_ImageDesc`): the port now clears the whole native desc. This was harmless
   before, because lb_800121FC sets every field.

No shared C bug was found, so nothing was mirrored into the C build and no golden changed (GATE OK against the old goldens).

## Runtime structs (step 2)

- **`memset`/`memzero`/`memcpy` over struct ranges.** Every decomp site with a literal size or an `offsetof` range was matched to
  its Rust port (grep of `src/**/*.c`, 275 calls).
  - The `offsetof`-range sites (the gmmain_lib save clears, ifmagnify, the camera quake) use native `offset_of!`, which is correct.
  - The literal-size sites: corneria and grpstadium were fixed (above). gm_16A2's `0x1C0` was checked: the native layout ends
    the range at the same field. `gm_8017C984` 0x74 and fn_80168A6C 0x20 have native equivalents. Toy's `0x25A` is a byte buffer
    by design, and lbcardnew's are byte arrays.
- **Pointer-offset arithmetic in Rust** (`(p as *mut u8).add(0x..)`, 13 sites): each lands on a record whose native layout is
  the GC layout (`--identical`: MatchEnd, Fighter_x2D0_t, ...) or on raw or word-swapped data.
- **Arrays indexed past a field** (52 annotated sites) were checked against `--identical`. One is latent: `Fighter.x1670` (the
  dynamics hit entries) is declared `[1]`, and the C allows 11 entries running into `filler_x1674`. Native entries are 48 bytes,
  so that room holds 9. Vanilla data never has more than 1 (`LAYOUT_WATCH=ftDynamics.x4` over every Pl*.dat: max 1), but a mod
  with 10 or 11 would overwrite `x1828`. This is left open, because fixing it changes the Fighter native layout.
- **Card saves.** `tests/save_image.rs` (the GC image permutation) passes. The `tu_lbcardgame` oracle fails at op 10 case 30
  because of the native `gc_stage()` call in `dont_inline_helper`, not because of a layout difference; it fails on the base
  commit too.
- **Hash and snapshot.** NHV1 (`melee_types::neutral_hash`) hashes native fields by name and is checked against the C on every
  golden row; snapshots are native only. The hash-relevant records that differ in part A (MatchPlayerData, StaleMoveTable,
  Fighter sub-records) differ only by upstream renames and retypes of the same bytes, and those records have the GC layout
  natively.

## Bool, char and enum at ABI boundaries (step 3)

- The shipping Rust exe links no game C. Its `extern "C"` blocks are libc and Win32 (strcpy, memcmp, localtime, winhttp, gdi32),
  and wasm mods cross only as i32/i64/f32/f64. No bool, char or enum crosses a C boundary.
- In data, `bool` is MSL `int` (4 bytes) in every record (part A checks the widths, and gen.rs has no `_Bool` field). `char` is
  signed (`-fsigned-char`, Rust i8), and enums are 4 bytes (`-fno-short-enums`). The signedness differences part A lists
  (Item.hold_kind, SpawnItem.hold_kind, Element_803F9628, DrMario's cape_kind: a C `u32` enum against Rust `enum_t`/`i32`) hold
  values below 2^31.

## Not done / open

- `melee_types` cannot be regenerated on the current tree. `gen_types.py` stops on the upstream sync: removed mndiagram
  records, `matstate_t`, `Ground_801C34AC_entry`, the folded `*.static.h` headers, `psdisp_Tlut` and the retyped
  `ftDynamics.x10`. Part A is the substitute check, because it reads the current headers. A full regeneration means carrying
  upstream's renames through the Rust code (follow-up 1 of upstream-sync.md).
- `Fighter.x1670` capacity for modded fighters (above).
- Part B does not walk the hand-typed custom roots listed above.

## Per-record table

Columns: A = part A, against the C layout; B = walked over the disc in part B; native = GC layout = `layout_walk --identical`
(the struct's native bytes are the GameCube bytes, so code that uses GC offsets on it is exact). Regenerate with
`python tools/rust-port/layout/make_table.py LA.json build/layout/walk.txt build/layout/identical.txt`.

| record | A: vs C layout | B: disc walk | native = GC layout | note |
|---|---|---|---|---|
| `ARQRequest` | no C record of that name |  | no |  |
| `AXFX_BUFFERUPDATE` | no C record of that name |  | no |  |
| `AXFX_CHORUS` | no C record of that name |  | no |  |
| `AXFX_CHORUS_SRCINFO` | no C record of that name |  | no |  |
| `AXFX_CHORUS_WORK` | no C record of that name |  | no |  |
| `AXFX_DELAY` | no C record of that name |  | no |  |
| `AXFX_REVERBHI` | no C record of that name |  | no |  |
| `AXFX_REVERBSTD` | no C record of that name |  | no |  |
| `AXFX_REVHI_DELAYLINE` | no C record of that name |  | no |  |
| `AXFX_REVHI_WORK` | no C record of that name |  | no |  |
| `AXFX_REVSTD_DELAYLINE` | no C record of that name |  | no |  |
| `AXFX_REVSTD_WORK` | no C record of that name |  | no |  |
| `AX_AUX_DATA` | no C record of that name |  | no |  |
| `AbsorbDesc` | match | walked, 0 mismatch | yes |  |
| `Allocator` | differs |  | no | native-only (lbmemory arena record; Rust keeps native pointers where C keeps u32 addresses) |
| `AnimBundle` | match |  | no |  |
| `AnimLoopSettings` | match |  | yes |  |
| `Anon_anon_102` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_1073` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_109` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_11` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_110` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_111` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_129` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_1290` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_13` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_130` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_131` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_131_area` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_131_asset` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_131_model_owner` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_133` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_1366` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_13_2` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_15` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_150` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_16` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_17` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_18` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_18_2` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_1941` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_197` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_2215` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_2244` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_226` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_2375` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_24` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_26` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_2809` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_2840` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_29` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_294` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_29_2` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_29_3` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_29_4` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_30` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_300` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_309` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3161` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_319` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_320` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_321` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_333` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_3331` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3356` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3373` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_339` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3412` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_345` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3471` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_35` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3510` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_355` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3582` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_35_2` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_36` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3601` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3622` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3672` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_3681` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_3684` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_37` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_378` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_38` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_39` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_40` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_407` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_41` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_44` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_45` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_49` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_5` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_545` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_6` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_612` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_64` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_68` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_69` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_7` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_72` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_76` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_766` | anonymous (checked inside its parent) |  | no |  |
| `Anon_anon_77` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_798` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_8` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_86` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_87` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_87_2` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_87_3` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_949` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_95` | anonymous (checked inside its parent) |  | yes |  |
| `Anon_anon_961` | anonymous (checked inside its parent) |  | yes |  |
| `Article` | match | walked, 0 mismatch | no |  |
| `ArticleDynamicBones` | no C record of that name | walked, 0 mismatch | no |  |
| `BgFlashUserData` | match |  | no |  |
| `BobOmbRain` | match |  | no |  |
| `BoneDynamicsDesc` | match | walked, 0 mismatch | no |  |
| `BracketEntry` | match |  | no |  |
| `BracketEntrySlot` | match |  | no |  |
| `BracketSrcEntry` | no C record of that name |  | yes |  |
| `CARDControl` | match |  | no |  |
| `CARDDecParam` | match |  | no |  |
| `CARDDir` | match |  | yes |  |
| `CARDDirCheck` | match |  | yes |  |
| `CARDFileInfo` | match |  | yes |  |
| `CARDID` | match |  | yes |  |
| `CARDStat` | match |  | yes |  |
| `CSSCharModel` | match |  | no |  |
| `CSSCursorData` | match |  | no |  |
| `CSSData` | match |  | no |  |
| `CSSDoor` | match |  | yes |  |
| `CSSDoorsData` | match |  | yes |  |
| `CSSDoorsData2` | match |  | no |  |
| `CSSDoorsMisc` | match |  | no |  |
| `CSSIcon` | match |  | yes |  |
| `CSSIconsData` | match |  | yes |  |
| `CSSKOStar` | match |  | no |  |
| `CSSModeInfo` | match |  | yes |  |
| `CSSTag` | match |  | no |  |
| `CSSTagData` | match |  | no |  |
| `Camera` | differs |  | no | upstream retyped a union member (x368 orbit); runtime struct, same bytes |
| `CameraBlurData` | match |  | no |  |
| `CameraBounds` | match |  | yes |  |
| `CameraBoxFlags` | match |  | yes |  |
| `CameraBoxFlags_anon_30` | anonymous (checked inside its parent) |  | yes |  |
| `CameraDebugMode` | match |  | yes |  |
| `CameraInputs` | match |  | yes |  |
| `CameraKindData` | no C record of that name |  | no |  |
| `CameraKindParams` | no C record of that name |  | yes |  |
| `CameraModeCallbacks` | match |  | no |  |
| `CameraQuake` | match |  | yes |  |
| `CameraTransformState` | match |  | yes |  |
| `CameraUnkGlobals` | match |  | yes |  |
| `CameraVsData` | match |  | no |  |
| `CameraVsData_anon_819` | anonymous (checked inside its parent) |  | no |  |
| `CameraVsData_anon_819_anon_820` | anonymous (checked inside its parent) |  | no |  |
| `Camera_x2D0` | match |  | no |  |
| `Camera_x344` | anonymous (checked inside its parent) |  | no |  |
| `Camera_x35C` | anonymous (checked inside its parent) |  | no |  |
| `Camera_x35C_bits` | anonymous (checked inside its parent) |  | yes |  |
| `Camera_x378` | anonymous (checked inside its parent) |  | yes |  |
| `Camera_x37C` | anonymous (checked inside its parent) |  | yes |  |
| `CardActiveRequest` | match |  | no |  |
| `CardBlockHeader` | differs |  | yes | flexible array member (checker cannot size it); same layout |
| `CardClearArgs` | match |  | no |  |
| `CardCmd` | match |  | no |  |
| `CardCmdBuf` | match |  | no |  |
| `CardCmd_anon_193` | anonymous (checked inside its parent) |  | no |  |
| `CardCreateArgs` | match |  | no |  |
| `CardCreateReqArgs` | match |  | no |  |
| `CardFileReqArgs` | match |  | no |  |
| `CardGetStatusArgs` | match |  | yes |  |
| `CardHeaderArgs` | match |  | no |  |
| `CardHeaderReqArgs` | match |  | no |  |
| `CardIconInfo` | match |  | yes |  |
| `CardOpenReqArgs` | match |  | yes |  |
| `CardReadArgs` | match |  | no |  |
| `CardReadHeaderArgs` | match |  | no |  |
| `CardRequest` | match |  | no |  |
| `CardRequest_anon_258` | anonymous (checked inside its parent) |  | no |  |
| `CardSectorArgs` | match |  | no |  |
| `CardState` | match |  | no |  |
| `CardStatusReqArgs` | match |  | no |  |
| `CardVerifyArgs` | match |  | no |  |
| `CardWriteArgs` | match |  | no |  |
| `ChallengerData` | match |  | yes |  |
| `CharScaleEntry` | no C record of that name |  | yes |  |
| `CmSubject` | match |  | no |  |
| `CmSubjectExtents` | match |  | yes |  |
| `CmdUnion` | match |  | yes |  |
| `CollData` | match |  | no |  |
| `CollJoint` | match |  | no |  |
| `CollLine` | match |  | no |  |
| `CollVtx` | match |  | yes |  |
| `ColorOverlay` | match |  | no |  |
| `ColorOverlay_UnkInner` | match |  | yes |  |
| `ColorOverlay_x28_colanim` | anonymous (checked inside its parent) |  | no |  |
| `ColorOverlay_x8_t` | match |  | yes |  |
| `ColorOverlay_x8_t_light_rot1` | anonymous (checked inside its parent) |  | yes |  |
| `ColorOverlay_x8_t_light_rot2` | anonymous (checked inside its parent) |  | yes |  |
| `ColorOverlay_x8_t_unk` | anonymous (checked inside its parent) |  | yes |  |
| `CommandInfo` | differs |  | no | TARGET_PC keeps both union spellings (upstream named it x8); same bytes |
| `CommandInfo_anon_1008` | anonymous (checked inside its parent) |  | no |  |
| `Command_00` | match |  | yes |  |
| `Command_02` | match |  | yes |  |
| `Command_03` | match |  | yes |  |
| `Command_04` | match |  | yes |  |
| `Command_05` | differs |  | yes | STREAM_WORD_TYPES pin: jump target kept as the big-endian u32 word of the raw image (by design) |
| `Command_07` | differs |  | yes | STREAM_WORD_TYPES pin (as Command_05) |
| `Command_09` | match |  | yes |  |
| `CostumeTObjList` | match |  | no |  |
| `CountEntry` | match |  | yes |  |
| `CpuFighter` | differs |  | no | upstream retyped u32 -> pointer; runtime struct |
| `CrowdConfig` | match |  | yes |  |
| `CrowdSFX_UnkStruct` | match |  | yes |  |
| `CssSubStruct` | match |  | no |  |
| `DBInterface` | no C record of that name |  | no |  |
| `DEMODMPad` | no C record of that name |  | yes |  |
| `DEMOWinPadInfo` | no C record of that name |  | yes |  |
| `DObjList` | match |  | no |  |
| `DTKTrack` | no C record of that name |  | no |  |
| `DVDBB2` | match |  | no |  |
| `DVDCommandBlock` | match |  | no |  |
| `DVDDir` | match |  | yes |  |
| `DVDDirEntry` | match |  | no |  |
| `DVDDiskID` | match |  | yes |  |
| `DVDDriveInfo` | match |  | yes |  |
| `DVDFileInfo` | match |  | no |  |
| `DamageLogEntry` | match |  | no |  |
| `DebugFontGlyph` | match |  | yes |  |
| `DebugGameOverData` | match |  | yes |  |
| `DemoStatData` | no C record of that name |  | yes |  |
| `DevText` | match |  | no |  |
| `Diagram` | match |  | no |  |
| `Diagram2` | match |  | no |  |
| `Diagram3` | match |  | no |  |
| `DigitInit` | match |  | yes |  |
| `DispBar` | match |  | yes |  |
| `DmgLogEntry` | match |  | no |  |
| `DmgLogEntry_anon_2030` | anonymous (checked inside its parent) |  | no |  |
| `DmgLogEntry_anon_2035` | anonymous (checked inside its parent) |  | no |  |
| `DynamicBoneTable` | match |  | no |  |
| `DynamicModelDesc` | match | walked, 0 mismatch | no |  |
| `DynamicsData` | match |  | no |  |
| `DynamicsDesc` | match | walked, 0 mismatch | no |  |
| `ECBSource` | match |  | no |  |
| `ECBSource_anon_186` | anonymous (checked inside its parent) |  | no |  |
| `ECBSource_anon_186_anon_187` | anonymous (checked inside its parent) |  | no |  |
| `ECBSource_anon_186_anon_191` | anonymous (checked inside its parent) |  | yes |  |
| `EF_DAT_Entry` | match |  | no |  |
| `EF_Effect` | match |  | no |  |
| `EF_EffectDesc` | match |  | no |  |
| `EF_ParamEntry` | match |  | no |  |
| `EF_QueuedEffect` | match |  | no |  |
| `EF_QueuedEffect_ObjAlloc` | match |  | yes |  |
| `EchoRecord` | no C record of that name |  | no |  |
| `EchoRing` | no C record of that name |  | no |  |
| `EchoState` | no C record of that name |  | no |  |
| `Element_803F9628` | differs |  | no | signedness of a static table field retyped upstream (s32 vs u32); values < 2^31 |
| `Element_803F9628_Obj_14` | match |  | yes |  |
| `Element_803F9628_x12` | match |  | yes |  |
| `EventData` | match |  | yes |  |
| `EventPriority` | match |  | no |  |
| `FFlowerAttr` | match |  | yes |  |
| `FallCommon` | match | walked, 0 mismatch | yes |  |
| `FigaTrack` | differs |  | no | upstream named the pad byte dummy0 (set in the data, unused); Rust keeps it as padding, unread |
| `FigaTree` | match |  | no |  |
| `Fighter` | differs |  | no | upstream renames/retypes (x5A4/x5A8 u32 vs ptr, CommandInfo union); runtime struct, hash fields unchanged |
| `FighterBone` | match |  | no |  |
| `FighterBone_anon_910` | anonymous (checked inside its parent) |  | yes |  |
| `FighterBone_anon_910_anon_911` | anonymous (checked inside its parent) |  | yes |  |
| `FighterBone_anon_910_anon_929` | anonymous (checked inside its parent) |  | yes |  |
| `FighterBone_anon_935` | anonymous (checked inside its parent) |  | yes |  |
| `FighterBone_anon_935_anon_936` | anonymous (checked inside its parent) |  | yes |  |
| `FighterData` | match |  | yes |  |
| `FighterData_x7C` | anonymous (checked inside its parent) |  | yes |  |
| `FighterHurtCapsule` | match |  | no |  |
| `FighterPartsTable` | match |  | no |  |
| `Fighter_804D64FC_t` | match |  | no |  |
| `Fighter_804D6518_t` | match |  | yes |  |
| `Fighter_804D651C_t` | match |  | yes |  |
| `Fighter_804D6520_t` | match |  | yes |  |
| `Fighter_804D6524_t` | match |  | yes |  |
| `Fighter_804D653C_t` | match | walked, 0 mismatch | no |  |
| `Fighter_804D6540_t` | match |  | no |  |
| `Fighter_804D6540_x0_t` | match |  | yes |  |
| `Fighter_AbsorbAttr` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_CostumeStrings` | match |  | no |  |
| `Fighter_DemoStrings` | match |  | no |  |
| `Fighter_FighterVars` | match |  | no |  |
| `Fighter_MotionVars` | differs |  | no | MasterHand pads split differently upstream; same size |
| `Fighter_ReflectAttr` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_ShakeTable_t` | match |  | no |  |
| `Fighter_WaitAnimData` | match | walked, 0 mismatch | no |  |
| `Fighter_anon_1364` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1364_anon_1365` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1364_anon_1365_x596_bits` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1364_anon_1379` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1705` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1705_anon_1707` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_anon_1746` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_input` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_x1614_t` | match |  | no |  |
| `Fighter_x1670_t` | match |  | no |  |
| `Fighter_x1A88_xFC_t` | match |  | yes |  |
| `Fighter_x20B0_t` | match |  | yes |  |
| `Fighter_x2D0_t` | match |  | yes |  |
| `Fighter_x59C_t` | match |  | yes |  |
| `Fighter_x5F4_arr` | anonymous (checked inside its parent) |  | yes |  |
| `Fighter_x8B0_t` | match |  | yes |  |
| `FlyCursor` | match |  | no |  |
| `FoxBlasterAttr` | match |  | yes |  |
| `FoxIllusionAttr` | match |  | yes |  |
| `FoxLaserAttr` | match |  | yes |  |
| `Frame` | no C record of that name |  | no |  |
| `FrontendItem` | match |  | no |  |
| `FrontendScreen` | match |  | no |  |
| `FtInputTimers` | match |  | yes |  |
| `FtPartsDesc` | match | walked, 0 mismatch | no |  |
| `FtPartsVis` | match |  | no |  |
| `FtPartsVisLookup` | match |  | no |  |
| `FtSFX` | match | walked, 0 mismatch | no |  |
| `GObjFuncs` | match |  | no |  |
| `GXFifoObj` | match |  | yes |  |
| `GameCache` | match |  | yes |  |
| `GameMode` | match |  | no |  |
| `GameModeState` | match |  | no |  |
| `GamePrefs` | match |  | yes |  |
| `GameRules` | match |  | yes |  |
| `GameScene` | match |  | no |  |
| `GameSceneInfo` | match |  | no |  |
| `GbAnimMotion` | no C record of that name |  | yes |  |
| `GbAnimMotionP` | no C record of that name |  | yes |  |
| `GbCape` | no C record of that name |  | yes |  |
| `GbCapeIn` | no C record of that name |  | yes |  |
| `GbCapeP` | no C record of that name |  | yes |  |
| `GbDrill` | no C record of that name |  | yes |  |
| `GbDrillIn` | no C record of that name |  | yes |  |
| `GbDrillP` | no C record of that name |  | yes |  |
| `GbGlide` | no C record of that name |  | yes |  |
| `GbGlideEntry` | no C record of that name |  | yes |  |
| `GbGlideIn` | no C record of that name |  | yes |  |
| `GbGlideP` | no C record of that name |  | yes |  |
| `GbMultiDash` | no C record of that name |  | yes |  |
| `GbMultiDashIn` | no C record of that name |  | yes |  |
| `GbMultiDashP` | no C record of that name |  | yes |  |
| `GbTornado` | no C record of that name |  | yes |  |
| `GbTornadoIn` | no C record of that name |  | yes |  |
| `GbTornadoP` | no C record of that name |  | yes |  |
| `GenoArtVars` | no C record of that name |  | yes |  |
| `GenoBhvDef` | no C record of that name |  | no |  |
| `GenoBhvParamDef` | no C record of that name |  | no |  |
| `GenoCbDef` | no C record of that name |  | no |  |
| `GenoCheck` | no C record of that name |  | yes |  |
| `GenoComboBlock` | no C record of that name |  | yes |  |
| `GenoCond` | no C record of that name |  | yes |  |
| `GenoCoverField` | no C record of that name |  | no |  |
| `GenoFamilyItem` | no C record of that name |  | no |  |
| `GenoItemVars` | no C record of that name |  | yes |  |
| `GenoLuaBlock` | no C record of that name |  | yes |  |
| `GenoLuaIo` | no C record of that name |  | yes |  |
| `GenoPalPObj` | match |  | no |  |
| `GenoState` | no C record of that name |  | yes |  |
| `GenoWord` | match |  | yes |  |
| `GlyphEntry` | match |  | no |  |
| `GlyphVariantEntry` | match |  | no |  |
| `GmCardData` | match |  | yes |  |
| `GmSaveData` | match |  | yes |  |
| `GmStats` | match |  | yes |  |
| `GrJoint` | match | walked, 0 mismatch | yes |  |
| `Ground` | differs |  | no | grCastle vars retyped upstream (pointer vs u32 at +0xC4); runtime union |
| `GroundItemData` | match |  | no |  |
| `GroundParam` | match | walked, 0 mismatch | no |  |
| `GroundShadowEntry` | match | walked, 0 mismatch | no |  |
| `GroundVars` | differs |  | no | as Ground |
| `GroundVars_carnull` | anonymous (checked inside its parent) |  | no |  |
| `GroundVars_flatzone` | match |  | yes |  |
| `GroundVars_flatzone2` | match |  | no |  |
| `GroundVars_flatzone3` | match |  | yes |  |
| `GroundVars_izumi` | match |  | no |  |
| `GroundVars_izumi2` | match |  | no |  |
| `GroundVars_izumi3` | match |  | no |  |
| `GroundVars_unk` | match |  | no |  |
| `Ground_x10_flags` | anonymous (checked inside its parent) |  | yes |  |
| `Ground_x11_flags` | anonymous (checked inside its parent) |  | yes |  |
| `GwProfDetailStats` | no C record of that name |  | yes |  |
| `GwProfStats` | no C record of that name |  | yes |  |
| `HRMeta` | no C record of that name |  | no |  |
| `HRPort` | no C record of that name |  | yes |  |
| `HRResult` | no C record of that name |  | yes |  |
| `HRRule` | no C record of that name |  | yes |  |
| `HSD_AObj` | match |  | no |  |
| `HSD_AObjDesc` | differs | walked, 0 mismatch | no | obj_id is a POINTER_KEY_FIELDS usize (the native joint the key names); disc walk checks it |
| `HSD_AllocEntry` | match |  | no |  |
| `HSD_AnimJoint` | match | walked, 0 mismatch | no |  |
| `HSD_Archive` | match |  | no |  |
| `HSD_ArchiveExternInfo` | match |  | yes |  |
| `HSD_ArchiveHeader` | match |  | yes |  |
| `HSD_ArchivePublicInfo` | match |  | yes |  |
| `HSD_ArchiveRelocationInfo` | match |  | yes |  |
| `HSD_ByteCodeExpDesc` | match |  | no |  |
| `HSD_CObj` | match |  | no |  |
| `HSD_CObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_CObjInfo` | match |  | no |  |
| `HSD_CObj_projection_param` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_CObj_projection_param_frustum` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_CObj_projection_param_ortho` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_CObj_projection_param_perspective` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_CObj_u` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_CameraAnim` | match | walked, 0 mismatch | no |  |
| `HSD_CameraDescCommon` | match |  | no |  |
| `HSD_CameraDescFrustum` | match |  | no |  |
| `HSD_CameraDescPerspective` | match | walked, 0 mismatch | no |  |
| `HSD_Chan` | match |  | no |  |
| `HSD_DObj` | match |  | no |  |
| `HSD_DObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_DObjInfo` | match |  | no |  |
| `HSD_DevCom` | no C record of that name |  | no |  |
| `HSD_Envelope` | match |  | no |  |
| `HSD_EnvelopeDesc` | match | walked, 0 mismatch | no |  |
| `HSD_Exp` | match |  | no |  |
| `HSD_ExpDesc` | match | walked, 0 mismatch | no |  |
| `HSD_Exp_expr` | anonymous (checked inside its parent) |  | no |  |
| `HSD_FObj` | match |  | no |  |
| `HSD_Fog` | match |  | no |  |
| `HSD_FogAdj` | match |  | no |  |
| `HSD_FogAdjDesc` | match |  | yes |  |
| `HSD_FogAdjInfo` | match |  | no |  |
| `HSD_FogDesc` | match | walked, 0 mismatch | no |  |
| `HSD_FogInfo` | match |  | no |  |
| `HSD_GObj` | match |  | no |  |
| `HSD_GObjProc` | match |  | no |  |
| `HSD_Generator` | match |  | no |  |
| `HSD_Generator_aux` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_Hash` | match |  | no |  |
| `HSD_HashEntry` | match |  | no |  |
| `HSD_IKHint` | match |  | yes |  |
| `HSD_IKHintDesc` | match |  | yes |  |
| `HSD_ImageDesc` | match | walked, 0 mismatch | no |  |
| `HSD_JObj` | match |  | no |  |
| `HSD_JObjInfo` | match |  | no |  |
| `HSD_JObj_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_Joint` | match | walked, 0 mismatch | no |  |
| `HSD_Joint_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_LObj` | match |  | no |  |
| `HSD_LObjInfo` | match |  | no |  |
| `HSD_LObj_u` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_LeakChecker` | match |  | no |  |
| `HSD_LightAnim` | match | walked, 0 mismatch | no |  |
| `HSD_LightAttn` | match |  | yes |  |
| `HSD_LightDesc` | match | walked, 0 mismatch | no |  |
| `HSD_LightDesc_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_LightPoint` | match |  | yes |  |
| `HSD_LightPointDesc` | match |  | yes |  |
| `HSD_LightSpot` | match |  | yes |  |
| `HSD_LightSpotDesc` | match |  | yes |  |
| `HSD_MObj` | match |  | no |  |
| `HSD_MObjInfo` | match |  | no |  |
| `HSD_MatAnimJoint` | match | walked, 0 mismatch | no |  |
| `HSD_Material` | match | walked, 0 mismatch | yes |  |
| `HSD_MaterialState` | match |  | yes |  |
| `HSD_Obj` | match |  | no |  |
| `HSD_ObjAllocUnk` | match |  | yes |  |
| `HSD_ObjAllocUnk5` | match |  | yes |  |
| `HSD_ObjData` | match |  | yes |  |
| `HSD_PEDesc` | match | walked, 0 mismatch | yes |  |
| `HSD_PObj` | match |  | no |  |
| `HSD_PObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_PObjDesc_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_PObjInfo` | match |  | no |  |
| `HSD_PObjUnion` | match |  | no |  |
| `HSD_PadData` | match |  | yes |  |
| `HSD_PadRumbleListData` | match |  | no |  |
| `HSD_PadStatus` | match |  | yes |  |
| `HSD_Particle` | match |  | no |  |
| `HSD_RObj` | match |  | no |  |
| `HSD_RObjAnimJoint` | match |  | no |  |
| `HSD_RObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_RObjDesc_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_RObj_u` | anonymous (checked inside its parent) |  | no |  |
| `HSD_Rumble` | match |  | yes |  |
| `HSD_RumbleData` | match |  | no |  |
| `HSD_Rvalue` | match |  | no |  |
| `HSD_RvalueList` | match |  | no |  |
| `HSD_SM` | match |  | no |  |
| `HSD_SObj` | match |  | no |  |
| `HSD_SObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_SObjDesc2` | match |  | no |  |
| `HSD_SObj_anon_43` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_SObj_anon_43_anon_45` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_SObj_anon_52` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_SObj_anon_52_anon_54` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_Shadow` | match |  | no |  |
| `HSD_ShapeAnim` | match | walked, 0 mismatch | no |  |
| `HSD_ShapeAnimDObj` | match | walked, 0 mismatch | no |  |
| `HSD_ShapeAnimJoint` | match | walked, 0 mismatch | no |  |
| `HSD_ShapeSet` | match |  | no |  |
| `HSD_ShapeSetDesc` | match | walked, 0 mismatch | no |  |
| `HSD_ShapeSet_blend` | anonymous (checked inside its parent) |  | no |  |
| `HSD_Spline` | match | walked, 0 mismatch | no |  |
| `HSD_SynthSFXGroup` | match |  | yes |  |
| `HSD_SynthSFXNode` | match |  | no |  |
| `HSD_SynthSFXNode_user_vol` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_SynthSFXNode_user_vol_anon_37` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_SynthVoiceEntry` | differs |  | no | checker artifact: the record is extracted under a renamed tag (synth.c `foo`) |
| `HSD_TExp` | match |  | no |  |
| `HSD_TExpDag` | match |  | no |  |
| `HSD_TExpRes` | match |  | yes |  |
| `HSD_TExpRes_reg` | anonymous (checked inside its parent) |  | yes |  |
| `HSD_TObj` | match |  | no |  |
| `HSD_Text` | match |  | no |  |
| `HSD_ViewingRect` | match |  | yes |  |
| `HSD_VtxDescList` | match | walked, 0 mismatch | no |  |
| `HSD_WObj` | match |  | no |  |
| `HSD_WObjAnim` | match | walked, 0 mismatch | no |  |
| `HSD_WObjDesc` | match | walked, 0 mismatch | no |  |
| `HSD_WObjInfo` | match |  | no |  |
| `HSD_psAppSRT` | match |  | no |  |
| `Handle` | match |  | no |  |
| `Heap` | match |  | no |  |
| `HeartContainerAttr` | match |  | yes |  |
| `HeartContainerAttr_x4` | anonymous (checked inside its parent) |  | yes |  |
| `HeartContainerAttr_x4_bits` | anonymous (checked inside its parent) |  | yes |  |
| `HitCapsule` | match |  | no |  |
| `HitCapsule_anon_83` | anonymous (checked inside its parent) |  | no |  |
| `HitResult` | match |  | no |  |
| `HitVictim` | match |  | no |  |
| `HudIndex` | match |  | no |  |
| `HurtCapsule` | match |  | no |  |
| `IKState` | match |  | no |  |
| `IfDamageFlags` | match |  | yes |  |
| `IfDamageState` | match |  | no |  |
| `IfStockData` | no C record of that name |  | yes |  |
| `IfStockDataOffset` | no C record of that name |  | yes |  |
| `IfStockStealAnim` | match |  | yes |  |
| `IfStockUserData` | differs |  | yes | upstream retyped the HUD stock records (ifstock.c); runtime only |
| `IntVec2` | match |  | yes |  |
| `IntVec3` | match |  | yes |  |
| `IntrWin` | no C record of that name |  | yes |  |
| `IntrWinAttack` | no C record of that name |  | yes |  |
| `ItCapsuleAttr` | match |  | yes |  |
| `ItHurtBoneDesc` | match | walked, 0 mismatch | yes |  |
| `ItHurtBoneList` | match | walked, 0 mismatch | no |  |
| `ItLGunAttr` | match |  | yes |  |
| `ItLGunBeamAttr` | match |  | yes |  |
| `ItLGunRayAttr` | match |  | yes |  |
| `Item` | differs |  | no | upstream retypes (hold_kind enum u32 vs enum_t, xC78 vec int/float); runtime struct |
| `ItemAttr` | differs | walked, 0 mismatch | yes | x8 f32 (C) vs i32 (Rust): no C or Rust reader; disc bits compared equal |
| `ItemCommonData` | match | walked, 0 mismatch | yes |  |
| `ItemDynamics` | match |  | no |  |
| `ItemHitbox` | match |  | no |  |
| `ItemLink` | match |  | no |  |
| `ItemLogicTable` | match |  | no |  |
| `ItemModStruct` | match |  | yes |  |
| `ItemModelDesc` | match | walked, 0 mismatch | no |  |
| `ItemPickTable` | match |  | no |  |
| `ItemStateArray` | match | walked, 0 mismatch | no |  |
| `ItemStateDesc` | match | walked, 0 mismatch | no |  |
| `ItemStateTable` | match |  | no |  |
| `Item_DynamicBones` | match |  | no |  |
| `Item_FtTrack` | match |  | no |  |
| `Item_ItemVars` | differs |  | no | Fox blaster vars retyped upstream (pointer vs s32); runtime |
| `Item_anon_477` | anonymous (checked inside its parent) |  | yes |  |
| `Item_anon_485` | anonymous (checked inside its parent) |  | yes |  |
| `Item_r13_Data` | match |  | no |  |
| `Item_xDCC_flag` | anonymous (checked inside its parent) |  | yes |  |
| `JpegByteBuffer` | match |  | yes |  |
| `JpegComment` | match |  | yes |  |
| `JpegEncodeTables` | match |  | yes |  |
| `JpegHuffAc` | match |  | yes |  |
| `JpegHuffDc` | match |  | yes |  |
| `JpegMetadata` | match |  | yes |  |
| `JpegOutput` | match |  | no |  |
| `JpegQuantTables` | match |  | yes |  |
| `JpegState` | match |  | yes |  |
| `JpegWork` | match |  | yes |  |
| `JpegWorkData` | match |  | yes |  |
| `KinokoAnim` | no C record of that name |  | no |  |
| `KinokoAttrs` | differs |  | yes | upstream: x8 is HSD_AnimJoint*[2]; Rust reads them through KinokoAnim (dat_custom it_80293660 path) |
| `KirbyHatStruct` | match |  | no |  |
| `Kirby_Unk` | no C record of that name |  | no |  |
| `LBMgr` | match |  | no |  |
| `LabPart` | no C record of that name |  | no |  |
| `Last_GroundVars` | match |  | no |  |
| `LbAudioAxBankMask` | anonymous (checked inside its parent) |  | yes |  |
| `LbAudioAxHeapData` | anonymous (checked inside its parent) |  | no |  |
| `LbCardEntry` | match |  | no |  |
| `LbCollColorPair` | match |  | no |  |
| `LbMcSnapMemSnapIconData` | match |  | no |  |
| `LbShadow` | match |  | no |  |
| `Lbl804799B8_t` | match |  | yes |  |
| `Lbl804799D8_t` | match |  | no |  |
| `Lbl804799D8_t_x2A` | anonymous (checked inside its parent) |  | yes |  |
| `LightList` | match | walked, 0 mismatch | no |  |
| `LightOverrideEntry` | no C record of that name | walked, 0 mismatch | no |  |
| `MTXStack` | match |  | no |  |
| `MapCollData` | match | walked, 0 mismatch | no |  |
| `MapJoint` | match | walked, 0 mismatch | yes |  |
| `MapLine` | match | walked, 0 mismatch | yes |  |
| `MapLineRange` | match | walked, 0 mismatch | yes |  |
| `Map_Chikuwa` | match |  | yes |  |
| `Map_GroundVars` | match |  | no |  |
| `Map_VanishDesc` | match |  | yes |  |
| `Map_VanishEntry` | match |  | no |  |
| `MatchEnd` | differs |  | yes | upstream: player_standings kills is u16[6] (Rust u16[4] + x18 + x1A); same bytes; native layout identical |
| `MatchExitInfo` | differs |  | yes | as MatchEnd |
| `MatchPlayerData` | differs |  | yes | as MatchEnd |
| `MatchPlayerData_anon_543` | anonymous (checked inside its parent) |  | yes |  |
| `MatchPlayerData_anon_543_anon_544` | anonymous (checked inside its parent) |  | yes |  |
| `MatchTeamData` | match |  | yes |  |
| `MaximTomatoSpecialAttr` | match |  | yes |  |
| `MccPacketBuffer` | match |  | yes |  |
| `MemEntry` | match |  | no |  |
| `Menu` | match |  | no |  |
| `MenuEnterData` | match |  | yes |  |
| `MenuExitData` | match |  | yes |  |
| `MenuKindData` | match |  | no |  |
| `MewVars` | match |  | yes |  |
| `MexSelectChr` | match |  | no |  |
| `MnCountData` | match |  | no |  |
| `MnDataDelData` | no C record of that name |  | yes |  |
| `MnDataDelGObjUserData` | no C record of that name |  | no |  |
| `MnDiagram2RowLayout` | match |  | yes |  |
| `MnEventData` | match |  | no |  |
| `MnInfoData` | match |  | no |  |
| `MnSelectChrDataTable` | match | walked, 0 mismatch | no |  |
| `MnSelectChrModels` | match | walked, 0 mismatch | no |  |
| `ModelNamesDesc` | match |  | no |  |
| `MotionState` | match |  | no |  |
| `MotionState_anon_994` | anonymous (checked inside its parent) |  | yes |  |
| `MotionState_anon_994_anon_997` | anonymous (checked inside its parent) |  | yes |  |
| `MotionState_anon_994_anon_997_anon_999` | anonymous (checked inside its parent) |  | yes |  |
| `NameData` | match |  | no |  |
| `NameNewEntry` | match |  | no |  |
| `NameTagData` | match |  | yes |  |
| `NameTagDataBank` | match |  | yes |  |
| `OSAlarm` | match |  | no |  |
| `OSBootInfo_s` | match |  | no |  |
| `OSCalendarTime` | match |  | yes |  |
| `OSCond` | match |  | no |  |
| `OSContext` | match |  | yes |  |
| `OSFontHeader` | match |  | yes |  |
| `OSImportInfo` | match |  | yes |  |
| `OSMessageQueue` | match |  | no |  |
| `OSModuleHeader` | match |  | no |  |
| `OSModuleInfo` | match |  | no |  |
| `OSModuleLink` | match |  | no |  |
| `OSModuleQueue` | match |  | no |  |
| `OSMutex` | match |  | no |  |
| `OSMutexLink` | match |  | no |  |
| `OSMutexQueue` | match |  | no |  |
| `OSRel` | match |  | yes |  |
| `OSResetFunctionInfo` | match |  | no |  |
| `OSResetFunctionQueue` | match |  | no |  |
| `OSSectionInfo` | match |  | yes |  |
| `OSSram` | match |  | yes |  |
| `OSSramEx` | match |  | yes |  |
| `OSStopwatch` | match |  | no |  |
| `OSThread` | match |  | no |  |
| `OSThreadLink` | match |  | no |  |
| `OSThreadQueue` | match |  | no |  |
| `PADStatus` | match |  | yes |  |
| `PPCWGPipe` | match |  | yes |  |
| `PPC_DMA_L_t` | no C record of that name |  | yes |  |
| `PPC_DMA_L_u` | no C record of that name |  | yes |  |
| `PPC_DMA_U_t` | no C record of that name |  | yes |  |
| `PPC_DMA_U_u` | no C record of that name |  | yes |  |
| `PPC_GQR_t` | no C record of that name |  | yes |  |
| `PPC_GQR_u` | no C record of that name |  | yes |  |
| `PackedS16x4` | no C record of that name |  | yes |  |
| `PadLibData` | match |  | no |  |
| `PartEnvelope` | no C record of that name |  | yes |  |
| `PartVertex` | no C record of that name |  | yes |  |
| `ParticleConsoleState` | match |  | no |  |
| `ParticleLogEntry` | match |  | yes |  |
| `ParticleUsbMessages` | match |  | yes |  |
| `PerfDispItem` | match |  | no |  |
| `PerfDispItem_content` | anonymous (checked inside its parent) |  | yes |  |
| `PerfDispItem_content_bars` | anonymous (checked inside its parent) |  | yes |  |
| `PerfDispItem_content_gradient` | anonymous (checked inside its parent) |  | yes |  |
| `PerfEvent` | no C record of that name |  | no |  |
| `PerfLabelLine` | match |  | no |  |
| `PerfSample` | no C record of that name |  | yes |  |
| `Placeholder_8016AE38_flags_2` | match |  | yes |  |
| `Placeholder_8016AE50_flags` | match |  | yes |  |
| `Placeholder_8016AE50_ret_val` | match |  | yes |  |
| `PlayerInitData` | match |  | yes |  |
| `PokemonSelectionState` | match |  | yes |  |
| `PolymorphicDesc` | match |  | no |  |
| `PosArray` | match |  | yes |  |
| `PosArrayFull` | match |  | yes |  |
| `PreloadCache` | match |  | no |  |
| `PreloadCacheSceneEntry` | match |  | yes |  |
| `PreloadEntry` | match |  | no |  |
| `PreloadedGameModeState` | match |  | yes |  |
| `Quaternion` | match | walked, 0 mismatch | yes |  |
| `Randall` | match |  | no |  |
| `RandomItemSpawner` | match |  | no |  |
| `RankEntry` | match |  | yes |  |
| `ReflectDesc` | match | walked, 0 mismatch | yes |  |
| `ResultsCharacterData` | no C record of that name |  | yes |  |
| `ResultsData` | match |  | no |  |
| `ResultsDisplayData` | no C record of that name |  | no |  |
| `ResultsDisplayLayout` | no C record of that name |  | no |  |
| `ResultsMatchInfo` | differs |  | yes | as MatchEnd |
| `ResultsPlayerConfig` | no C record of that name |  | no |  |
| `ResultsPlayerData` | match |  | no |  |
| `ResultsRenderFuncs` | no C record of that name |  | no |  |
| `RouteEntry` | match |  | no |  |
| `RouteEntryFlags` | match |  | yes |  |
| `RumbleCommand` | match |  | yes |  |
| `RumbleInfo` | match |  | no |  |
| `S16Vec` | match |  | yes |  |
| `S32Vec` | match |  | yes |  |
| `S32Vec2` | match |  | yes |  |
| `S8Vec2` | match |  | yes |  |
| `S8Vec3` | match |  | yes |  |
| `SIControl` | match |  | no |  |
| `SIPacket` | match |  | no |  |
| `SIS` | no C record of that name |  | no |  |
| `SSSData` | match |  | no |  |
| `STRUCT_DEMOWIN` | no C record of that name |  | no |  |
| `STRUCT_DSP_TASK` | match |  | no |  |
| `STRUCT_LISTBOX` | no C record of that name |  | no |  |
| `STRUCT_LISTBOX_ITEM` | no C record of that name |  | no |  |
| `STRUCT_MENU` | no C record of that name |  | no |  |
| `STRUCT_MENU_ITEM` | no C record of that name |  | no |  |
| `S_UNK_YOSHI1` | match |  | no |  |
| `S_UNK_YOSHI2` | match |  | no |  |
| `SceneCameraDesc` | match | walked, 0 mismatch | no |  |
| `SceneDesc` | match | walked, 0 mismatch | no |  |
| `SceneFogDesc` | match | walked, 0 mismatch | no |  |
| `ScopeBeamAttrs` | match |  | yes |  |
| `ScopeBeamFloats` | match |  | yes |  |
| `ScriptArmorCore` | no C record of that name |  | yes |  |
| `ScriptArmorCoreEvent` | no C record of that name |  | yes |  |
| `ScriptArmorCoreSlot` | no C record of that name |  | yes |  |
| `ScriptBench` | no C record of that name |  | no |  |
| `ScriptBenchEntity` | no C record of that name |  | no |  |
| `ScriptBenchOps` | no C record of that name |  | no |  |
| `ScriptBuild` | no C record of that name |  | yes |  |
| `ScriptCameraParams` | no C record of that name |  | yes |  |
| `ScriptCapsMovement` | no C record of that name |  | yes |  |
| `ScriptCapsStatus` | no C record of that name |  | no |  |
| `ScriptCpuCtl` | no C record of that name |  | yes |  |
| `ScriptCritCfg` | no C record of that name |  | no |  |
| `ScriptCritLaunch` | no C record of that name |  | no |  |
| `ScriptCritPending` | no C record of that name |  | no |  |
| `ScriptEchoRule` | no C record of that name |  | no |  |
| `ScriptFighterMod` | no C record of that name |  | yes |  |
| `ScriptMeshInstance` | no C record of that name |  | yes |  |
| `ScriptMeshInstance_line` | anonymous (checked inside its parent) |  | yes |  |
| `ScriptOnePTemplate` | no C record of that name |  | yes |  |
| `ScriptShock` | no C record of that name |  | no |  |
| `ScriptSimState` | no C record of that name |  | yes |  |
| `ScriptSkillRec` | no C record of that name |  | no |  |
| `ScriptSkillRow` | no C record of that name |  | yes |  |
| `ScriptStageArchive` | match |  | no |  |
| `ScriptStageEnemy` | match |  | no |  |
| `ScriptStageLine` | match |  | yes |  |
| `ScriptStageModel` | match |  | no |  |
| `ScriptStageSlot` | no C record of that name |  | no |  |
| `ScriptStageTarget` | match |  | no |  |
| `ScrollVars` | match |  | no |  |
| `SfxLoadStreamNode` | match |  | no |  |
| `ShieldDesc` | match |  | yes |  |
| `ShyGuys` | match |  | yes |  |
| `SisBlock` | match |  | no |  |
| `SisFontData` | no C record of that name |  | no |  |
| `SlotMarkers` | no C record of that name |  | no |  |
| `SmCtx` | no C record of that name |  | yes |  |
| `SmEvent` | no C record of that name |  | yes |  |
| `SmFx` | no C record of that name |  | yes |  |
| `SmPlayer` | no C record of that name |  | yes |  |
| `SmProgram` | no C record of that name |  | yes |  |
| `SmStatus` | no C record of that name |  | yes |  |
| `SmWorld` | no C record of that name |  | yes |  |
| `SmallerHitCapsule` | match |  | yes |  |
| `SoundParams` | match |  | no |  |
| `SpawnItem` | differs |  | no | hold_kind enum signedness (u32 vs enum_t) |
| `SramControl` | match |  | no |  |
| `StCarry` | no C record of that name |  | yes |  |
| `StDatLine` | no C record of that name |  | yes |  |
| `StDatModels` | no C record of that name |  | yes |  |
| `StDatView` | no C record of that name |  | no |  |
| `StOwned` | no C record of that name |  | no |  |
| `StOwnedEntry` | no C record of that name |  | no |  |
| `StageBlastZone` | match |  | yes |  |
| `StageCallbacks` | match |  | no |  |
| `StageCallbacks_anon_135` | anonymous (checked inside its parent) |  | yes |  |
| `StageCallbacks_anon_135_anon_137` | anonymous (checked inside its parent) |  | yes |  |
| `StageCameraInfo` | match |  | yes |  |
| `StageData` | match |  | no |  |
| `StageIdPair` | match |  | yes |  |
| `StageInfo` | match |  | no |  |
| `StageInfo_unk8C` | anonymous (checked inside its parent) |  | yes |  |
| `StageParam` | match | walked, 0 mismatch | yes |  |
| `StageSelUserData` | no C record of that name |  | yes |  |
| `StaleMoveTable` | differs |  | yes | upstream: xCBC is a union {s32; bitfields}; Rust keeps the bytes; native layout identical, hash unchanged |
| `StaleMoveTable_StaleMoves` | anonymous (checked inside its parent) |  | yes |  |
| `StartMeleeData` | match |  | no |  |
| `StartMeleeRules` | match |  | no |  |
| `StartMeleeRules_anon_250` | anonymous (checked inside its parent) |  | yes |  |
| `StaticModelDesc` | match | walked, 0 mismatch | no |  |
| `Struct2070` | match |  | yes |  |
| `Struct2070_anon_1036` | anonymous (checked inside its parent) |  | yes |  |
| `Struct2074` | differs |  | yes | upstream retyped int/float vectors; runtime fighter state, same bytes |
| `Struct2074_anon_1061` | anonymous (checked inside its parent) |  | yes |  |
| `Struct2074_anon_1061_anon_1063` | anonymous (checked inside its parent) |  | yes |  |
| `SurfaceData` | match |  | yes |  |
| `SwordAttrs` | match | walked, 0 mismatch | yes |  |
| `SzDef` | no C record of that name |  | yes |  |
| `SzEntity` | no C record of that name |  | yes |  |
| `SzEvent` | no C record of that name |  | yes |  |
| `SzEvent_zone` | anonymous (checked inside its parent) |  | yes |  |
| `SzInput` | no C record of that name |  | yes |  |
| `SzState` | no C record of that name |  | yes |  |
| `THPAlignedHuffmanTabPtr` | match |  | no |  |
| `THPAlignedSamplePtr` | match |  | no |  |
| `THPAlignedWidth` | match |  | yes |  |
| `THPDecComp` | match |  | no |  |
| `THPDec_8032FD40_Data` | match |  | yes |  |
| `THPFileInfoDCTCompUView` | match |  | yes |  |
| `THPFileInfoDCTCompVView` | match |  | yes |  |
| `THPFileInfoDCTCompYView` | match |  | yes |  |
| `THPFrameBuffer` | differs |  | yes | flexible array member (checker cannot size it) |
| `THPInitWork` | match |  | no |  |
| `THPLCSizeEntry` | match |  | yes |  |
| `THPLCWork` | match |  | no |  |
| `THPQuantizationPtr` | match |  | no |  |
| `THPVideoDecodeHeader` | match |  | yes |  |
| `TechAiObservation` | match |  | yes |  |
| `TechAiPolicy` | match |  | yes |  |
| `TempS` | match |  | no |  |
| `TetherAttributes` | match |  | yes |  |
| `TextGlyphTexture` | match |  | yes |  |
| `TextKerning` | no C record of that name |  | yes |  |
| `TmAnimFrameTable` | match |  | yes |  |
| `TmAnimFrames` | match |  | yes |  |
| `TmAnimTimers` | match |  | yes |  |
| `TmAnimTimers_x1D` | anonymous (checked inside its parent) |  | yes |  |
| `TmBoxArrays` | no C record of that name |  | no |  |
| `TmData` | match |  | no |  |
| `TmSettingTable` | match |  | yes |  |
| `TmUnkMenuData` | match |  | yes |  |
| `TmVsData` | match |  | yes |  |
| `Toy` | match |  | yes |  |
| `Toy26B8` | match |  | no |  |
| `Toy26B8_anon_185` | anonymous (checked inside its parent) |  | no |  |
| `Toy6E68` | differs |  | no | NATIVE_FIELD_TYPES pin (native pointers in a pad) |
| `ToyAnimState` | match |  | no |  |
| `ToyCameraControl` | match |  | no |  |
| `ToyDataJObj` | match |  | no |  |
| `ToyDataX8` | match |  | no |  |
| `ToyED8Data` | match |  | no |  |
| `ToyEntry` | match |  | yes |  |
| `ToyEntryData` | match |  | no |  |
| `ToyEntry_anon_46` | anonymous (checked inside its parent) |  | yes |  |
| `ToyGlobalsS_` | match |  | no |  |
| `ToyJObjNode` | match |  | no |  |
| `ToyListEntry` | match |  | no |  |
| `ToyModelFile` | match | walked, 0 mismatch | yes |  |
| `ToyNameData` | match | walked, 0 mismatch | yes |  |
| `ToyPanelLabelData` | match |  | no |  |
| `ToyParamEditor` | match |  | no |  |
| `ToySubStructS_` | match |  | yes |  |
| `ToyTable` | match |  | yes |  |
| `ToyTransitionObj` | match |  | yes |  |
| `ToyUnkJObjData` | match |  | no |  |
| `TrainingModeState` | differs |  | yes | upstream grew char_data 25 -> 27 into the pad; same size |
| `TrophyData` | match | walked, 0 mismatch | yes |  |
| `Ty25Entry` | match |  | no |  |
| `TyArchiveData` | match |  | no |  |
| `TyCameraData_` | match |  | no |  |
| `TyCleanupObj` | match |  | no |  |
| `TyDisplayData` | differs |  | no | NATIVE_FIELD_TYPES pin |
| `TyDspArchNames` | match |  | no |  |
| `TyDspArchiveHolder` | match |  | no |  |
| `TyDspBaseData` | match |  | yes |  |
| `TyDspBgData` | match |  | no |  |
| `TyDspConfig` | match |  | no |  |
| `TyDspEntry` | match | walked, 0 mismatch | yes |  |
| `TyDspGrid` | match |  | yes |  |
| `TyDspNameTables` | match |  | no |  |
| `TyDspPos` | match |  | yes |  |
| `TyFiguponData` | differs |  | no | NATIVE_FIELD_TYPES pin |
| `TyFiguponDigitInit` | match |  | yes |  |
| `TyFiguponDigitInit_s` | anonymous (checked inside its parent) |  | yes |  |
| `TyFiguponED4` | match |  | no |  |
| `TyFiguponInner` | match |  | yes |  |
| `TyFiguponUD` | match |  | yes |  |
| `TyGObjX8_` | match |  | no |  |
| `TyLightFile` | no C record of that name |  | no |  |
| `TyLightIndexEntry` | no C record of that name |  | yes |  |
| `TyLightSymbolEntry` | no C record of that name |  | no |  |
| `TyListArg` | match |  | no |  |
| `TyListData` | match |  | no |  |
| `TyListGobjEntry` | match |  | no |  |
| `TyListRow` | match |  | no |  |
| `TyListState` | match |  | no |  |
| `TyListWaitData` | match |  | yes |  |
| `TyModeState` | match |  | yes |  |
| `TySortElem` | match |  | yes |  |
| `TySortElemInt` | match |  | yes |  |
| `TySortRow` | match |  | yes |  |
| `TyViewData` | match |  | no |  |
| `U32Pair` | no C record of that name |  | yes |  |
| `U8Vec2` | match |  | yes |  |
| `U8Vec4` | match |  | yes |  |
| `UNK_SAMUS_S1` | match |  | no |  |
| `Unk1PData` | match |  | no |  |
| `Unk1PData_x24` | match |  | yes |  |
| `Unk1PData_xC` | match |  | yes |  |
| `Unk803BACC8` | match |  | no |  |
| `Unk80433380` | match |  | no |  |
| `Unk80433380_0` | match |  | yes |  |
| `Unk80433380_48` | match |  | yes |  |
| `UnkAdventureData` | match |  | no |  |
| `UnkAllstarData` | match |  | no |  |
| `UnkArchiveStruct` | match |  | no |  |
| `UnkArrElem` | match |  | yes |  |
| `UnkCostumeList` | match |  | no |  |
| `UnkCostumeStruct` | match |  | no |  |
| `UnkFlagStruct` | match |  | yes |  |
| `UnkFlagStruct_anon_27` | anonymous (checked inside its parent) |  | yes |  |
| `UnkFloat6_Camera` | match | walked, 0 mismatch | yes |  |
| `UnkMultimanData` | match |  | yes |  |
| `UnkPlBonusBits` | no C record of that name |  | yes |  |
| `UnkResultPlayerData` | match |  | yes |  |
| `UnkSelections` | match |  | yes |  |
| `UnkStageDat` | match | walked, 0 mismatch | no |  |
| `UnkStageDatInternal` | no C record of that name |  | yes |  |
| `UnkStageDat_x0_entry` | match | walked, 0 mismatch | no |  |
| `UnkStageDat_x8_t` | match | walked, 0 mismatch | no |  |
| `UnknownClassTypeE` | match |  | no |  |
| `Vec` | match | walked, 0 mismatch | yes |  |
| `Vec2` | match | walked, 0 mismatch | yes |  |
| `Vi0502Data` | match |  | yes |  |
| `ViCharaDesc` | match |  | yes |  |
| `VsApproachData` | match |  | yes |  |
| `VsModeData` | match |  | no |  |
| `VsSceneController` | differs |  | no | as MatchEnd |
| `VsSceneFighter` | match |  | yes |  |
| `VsSceneFighter_anon_483` | anonymous (checked inside its parent) |  | yes |  |
| `VsSceneFighter_anon_483_anon_484` | anonymous (checked inside its parent) |  | yes |  |
| `VsSceneState` | differs |  | no | as MatchEnd |
| `WaitStruct` | match | walked, 0 mismatch | no |  |
| `WaitStruct_u` | anonymous (checked inside its parent) |  | no |  |
| `WaitStruct_u_i` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `WaitStruct_u_p` | anonymous (checked inside its parent) |  | no |  |
| `WarnCmnData` | no C record of that name |  | yes |  |
| `_AXPB` | no C record of that name |  | yes |  |
| `_AXPBADDR` | no C record of that name |  | yes |  |
| `_AXPBADPCM` | no C record of that name |  | yes |  |
| `_AXPBADPCMLOOP` | no C record of that name |  | yes |  |
| `_AXPBDPOP` | no C record of that name |  | yes |  |
| `_AXPBFIR` | no C record of that name |  | yes |  |
| `_AXPBITD` | no C record of that name |  | yes |  |
| `_AXPBITDBUFFER` | no C record of that name |  | yes |  |
| `_AXPBMIX` | no C record of that name |  | yes |  |
| `_AXPBSRC` | no C record of that name |  | yes |  |
| `_AXPBU` | no C record of that name |  | yes |  |
| `_AXPBUPDATE` | no C record of that name |  | yes |  |
| `_AXPBVE` | no C record of that name |  | yes |  |
| `_AXPROFILE` | no C record of that name |  | yes |  |
| `_AXSPB` | no C record of that name |  | yes |  |
| `_AXVPB` | no C record of that name |  | no |  |
| `_DispItem` | match |  | no |  |
| `_DispItem_content` | anonymous (checked inside its parent) |  | yes |  |
| `_ECBFlagStruct` | match |  | yes |  |
| `_EFB` | match |  | yes |  |
| `_FtSFXArr` | match | walked, 0 mismatch | no |  |
| `_GXColor` | match | walked, 0 mismatch | yes |  |
| `_GXColorS10` | match |  | yes |  |
| `_GXFogAdjTable` | match |  | yes |  |
| `_GXLightObj` | match |  | yes |  |
| `_GXRenderModeObj` | match |  | yes |  |
| `_GXTexObj` | match |  | yes |  |
| `_GXTexRegion` | match |  | yes |  |
| `_GXTlutObj` | match |  | yes |  |
| `_GXTlutRegion` | match |  | yes |  |
| `_GXVtxAttrFmtList` | match |  | yes |  |
| `_GXVtxDescList` | match |  | yes |  |
| `_HSD_ChanAnim` | match |  | no |  |
| `_HSD_Class` | match |  | no |  |
| `_HSD_ClassInfo` | match |  | no |  |
| `_HSD_ClassInfoHead` | match |  | no |  |
| `_HSD_DList` | match |  | no |  |
| `_HSD_FObjDesc` | match | walked, 0 mismatch | no |  |
| `_HSD_FreeList` | match |  | no |  |
| `_HSD_GObjLibInitDataType` | match |  | no |  |
| `_HSD_HashClass` | match |  | no |  |
| `_HSD_HashClassInfo` | match |  | no |  |
| `_HSD_IDTable` | match |  | no |  |
| `_HSD_MObjDesc` | match | walked, 0 mismatch | no |  |
| `_HSD_MatAnim` | match | walked, 0 mismatch | no |  |
| `_HSD_MemReport` | match |  | yes |  |
| `_HSD_MemoryEntry` | match |  | no |  |
| `_HSD_ObjAllocData` | match |  | no |  |
| `_HSD_ObjAllocLink` | match |  | no |  |
| `_HSD_ObjInfo` | match |  | no |  |
| `_HSD_PSCmdList` | match |  | yes |  |
| `_HSD_PSFormGroup` | match |  | no |  |
| `_HSD_PSTexGroup` | match |  | no |  |
| `_HSD_PerfStat` | match |  | yes |  |
| `_HSD_RectF32` | match |  | yes |  |
| `_HSD_RectS16` | match | walked, 0 mismatch | yes |  |
| `_HSD_RenderAnim` | match |  | no |  |
| `_HSD_SList` | match |  | no |  |
| `_HSD_TEArg` | match |  | no |  |
| `_HSD_TECnst` | match |  | no |  |
| `_HSD_TECommon` | match |  | no |  |
| `_HSD_TETev` | match |  | no |  |
| `_HSD_TExpTevDesc` | match |  | no |  |
| `_HSD_TObjDesc` | match | walked, 0 mismatch | no |  |
| `_HSD_TObjInfo` | match |  | no |  |
| `_HSD_TObjTev` | match |  | yes |  |
| `_HSD_TObjTevDesc` | match | walked, 0 mismatch | yes |  |
| `_HSD_TevConf` | match |  | yes |  |
| `_HSD_TevDesc` | match |  | no |  |
| `_HSD_TevDesc_u` | anonymous (checked inside its parent) |  | yes |  |
| `_HSD_TevDesc_u_tevop` | anonymous (checked inside its parent) |  | yes |  |
| `_HSD_TevRegAnim` | match |  | no |  |
| `_HSD_TexAnim` | match | walked, 0 mismatch | no |  |
| `_HSD_TexLODDesc` | match | walked, 0 mismatch | yes |  |
| `_HSD_Tlut` | match | walked, 0 mismatch | no |  |
| `_HSD_TlutDesc` | match | walked, 0 mismatch | no |  |
| `_HSD_VIInfo` | match |  | no |  |
| `_HSD_VIStatus` | match |  | yes |  |
| `_IDEntry` | match |  | no |  |
| `_IO_FILE` | match |  | no |  |
| `_MCCPacket` | match |  | yes |  |
| `_MainMenuData` | match |  | no |  |
| `_MainMenuPanelData` | match |  | yes |  |
| `_MarsAttributes` | match | walked, 0 mismatch | yes |  |
| `_MenuFlow` | match |  | no |  |
| `_MenuInputState` | match |  | yes |  |
| `_MenuRulesPlusData` | match |  | no |  |
| `_MenuRulesPlusData_rule_values` | anonymous (checked inside its parent) |  | yes |  |
| `_MenuRulesPlusData_rule_values_anon_23` | anonymous (checked inside its parent) |  | yes |  |
| `_PSUserFunc` | match |  | no |  |
| `_Scissor` | match | walked, 0 mismatch | yes |  |
| `_SmashAttr` | match |  | yes |  |
| `_StaticPlayer` | differs |  | no | as StaleMoveTable |
| `_StaticPlayer_flags` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_flagsAE` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_more_flags` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_player_poses` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_player_poses_byVecName` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_staminas` | anonymous (checked inside its parent) |  | yes |  |
| `_StaticPlayer_staminas_byName` | anonymous (checked inside its parent) |  | yes |  |
| `_THPComponent` | match |  | yes |  |
| `_THPFileInfo` | match |  | no |  |
| `_THPHuffmanTab` | match |  | no |  |
| `_ThrowFlags` | match |  | yes |  |
| `_ThrowFlags_anon_730` | anonymous (checked inside its parent) |  | yes |  |
| `_ThrowFlags_anon_730_anon_731` | anonymous (checked inside its parent) |  | yes |  |
| `_Toy_804A26B8_t` | match |  | no |  |
| `_XFB` | match |  | no |  |
| `__InStrCtrl` | match |  | no |  |
| `__OutStrCtrl` | match |  | no |  |
| `__file_modes` | match |  | yes |  |
| `__file_state` | match |  | yes |  |
| `_auxCone` | match |  | yes |  |
| `_auxDisc` | match |  | yes |  |
| `_auxLine` | match |  | yes |  |
| `_auxRect` | match |  | yes |  |
| `_auxSphere` | match |  | yes |  |
| `_auxTornado` | match |  | yes |  |
| `_callbackArg` | match |  | no |  |
| `_current` | match |  | yes |  |
| `_ftCrazyHandAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes_SpecialHi` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes_SpecialLw` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes_SpecialN` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes_SpecialS` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `_ftDonkeyAttributes_cargo_hold` | anonymous (checked inside its parent) | walked, 0 mismatch | yes |  |
| `_ftDrMarioAttributes` | differs |  | yes | cape_kind ItemKind (u32) vs u32/s32 spelling; disc values compared equal |
| `_ftECB` | match |  | yes |  |
| `_ftGameWatchAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftKoopaAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftKoopaVars` | match |  | yes |  |
| `_ftLuigiAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftMewtwoAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftPichuAttributes` | match |  | yes |  |
| `_ftPikachuAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftPurinAttributes` | differs | walked, 0 mismatch | no | upstream: xE8/xEC are floats; Rust has pointers there. Typed by extattr; disc walk equal (pointer slots null both sides) |
| `_ftSamusAttributes` | differs | walked, 0 mismatch | no | upstream: xD0 is s32; Rust pointer. ext_attr decoded typed (extattr); disc walk equal |
| `_ftSeakAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftYoshiAttributes` | match | walked, 0 mismatch | yes |  |
| `_ftZakoboyAttributes` | match |  | yes |  |
| `_itECB` | match | walked, 0 mismatch | yes |  |
| `_m2c_stack_ftCo_800A3554` | match |  | yes |  |
| `_m2c_stack_itSamusmissile_UnkMotion0_Phys` | match |  | yes |  |
| `_m2c_stack_it_802B64FC` | match |  | yes |  |
| `_objheap` | match |  | no |  |
| `_unk_gobj_struct` | match |  | no |  |
| `_unk_gobj_struct_anon_54` | anonymous (checked inside its parent) |  | yes |  |
| `_unk_gobj_struct_anon_54_anon_56` | anonymous (checked inside its parent) |  | yes |  |
| `datetime` | match |  | yes |  |
| `dmg` | differs |  | no | as StaleMoveTable + x1894 retyped |
| `drawdone` | match |  | no |  |
| `flag32` | match |  | yes |  |
| `flag32_flags` | anonymous (checked inside its parent) |  | yes |  |
| `footstep_fx_0` | match |  | yes |  |
| `ftCaptainSpecialHiVars` | match |  | yes |  |
| `ftCaptainSpecialLwVars` | match |  | yes |  |
| `ftCaptainSpecialSVars` | match |  | yes |  |
| `ftCaptain_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftCaptain_FighterVars` | match |  | yes |  |
| `ftCaptain_MotionVars` | match |  | yes |  |
| `ftCo_803C6594_t` | match |  | no |  |
| `ftCo_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftCo_DatAttrs_xBC_t` | match | walked, 0 mismatch | yes |  |
| `ftCollisionBox` | match | walked, 0 mismatch | yes |  |
| `ftCommonData` | differs | walked, 0 mismatch | yes | upstream: sub_colors[5] where Rust has x6DC[4] + x6EC; same bytes (PlCo walk: slot 0) |
| `ftCommon_MotionVars` | differs |  | no | runtime union, pointer vs u32 |
| `ftCommon_MotionVars_aircatch` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_aircatchhit` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_attack1` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_attack100` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_attackdash` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_attacklw3` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_barrel` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_bury` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_buryjump` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_capturedamage` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_capturekirby` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_capturekoopa` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_captureleadead` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_capturelikelike` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_capturewait` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_cargokneebend` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_cargoturn` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_catch_` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_cliff` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_cliffjump` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_common` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_damage` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_damageice` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_damageicejump` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_dash` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_downdamage` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_downreflect` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_downspot` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_downwait` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_entry` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_escape` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_escapeair` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_fall` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_fallaerial` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_fallspecial` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_fighterthrow` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_guard` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_hammerkneebend` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_hammerlanding` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_itemget` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_itemthrow` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_itemthrow4` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_jump` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_jumpaerial` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_kneebend` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_landing` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_lift` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_mushroom` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_parasol_open` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_pass` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_passivewall` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_rebound` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_run` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_runbrake` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_shouldered` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_squat` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_swing` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_throw_` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_thrown` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_thrownkirby` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_thrownkirby_anon_296` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_thrownkirby_anon_296_anon_298` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_turn` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_turnrun` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_unk_800D2890` | anonymous (checked inside its parent) |  | no |  |
| `ftCommon_MotionVars_unk_800D331C` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_unk_800D34E0` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_unk_800D3680` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_unk_deadleft` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_unk_deadup` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_walk` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_warpstar` | anonymous (checked inside its parent) |  | yes |  |
| `ftCommon_MotionVars_yoshiegg` | anonymous (checked inside its parent) |  | no |  |
| `ftCrazyHand_BackCrushVars` | match |  | yes |  |
| `ftCrazyHand_DamageVars` | match |  | yes |  |
| `ftData` | match | walked, 0 mismatch | no |  |
| `ftData_80085FD4_ret` | match |  | no |  |
| `ftData_UnkCountStruct` | match |  | no |  |
| `ftData_UnkModelStruct` | match |  | no |  |
| `ftData_x1C` | match | walked, 0 mismatch | no |  |
| `ftData_x20` | match | walked, 0 mismatch | no |  |
| `ftData_x30` | match | walked, 0 mismatch | no |  |
| `ftData_x34` | match | walked, 0 mismatch | yes |  |
| `ftData_x38` | no C record of that name | walked, 0 mismatch | yes |  |
| `ftData_x44_t` | match | walked, 0 mismatch | yes |  |
| `ftData_x58_t` | match | walked, 0 mismatch | yes |  |
| `ftData_x8` | match | walked, 0 mismatch | no |  |
| `ftData_x8_x8` | match | walked, 0 mismatch | no |  |
| `ftDeviceUnk3` | match |  | no |  |
| `ftDeviceUnk4` | match |  | no |  |
| `ftDeviceUnk5` | match |  | no |  |
| `ftDonkey_FighterVars` | match |  | yes |  |
| `ftDonkey_MotionVars` | match |  | yes |  |
| `ftDonkey_SpecialLwVars` | match |  | yes |  |
| `ftDonkey_SpecialNVars` | match |  | yes |  |
| `ftDonkey_State5Vars` | match |  | yes |  |
| `ftDonkey_State7Vars` | match |  | yes |  |
| `ftDonkey_State8Vars` | match |  | yes |  |
| `ftDynamics` | match | walked, 0 mismatch | no |  |
| `ftDynamics_anon_1979` | anonymous (checked inside its parent) | walked, 0 mismatch | no |  |
| `ftFoxAppealS` | match |  | yes |  |
| `ftFoxSpecialHi` | match |  | yes |  |
| `ftFoxSpecialLw` | match |  | yes |  |
| `ftFoxSpecialN` | match |  | yes |  |
| `ftFoxSpecialS` | match |  | no |  |
| `ftFox_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftFox_FighterVars` | match |  | no |  |
| `ftFox_MotionVars` | match |  | no |  |
| `ftGameWatchChef` | match |  | yes |  |
| `ftGameWatchJudge` | match |  | yes |  |
| `ftGameWatch_Attack11Vars` | match |  | yes |  |
| `ftGameWatch_FighterVars` | match |  | no |  |
| `ftGameWatch_MotionVars` | match |  | yes |  |
| `ftGameWatch_SpecialLwVars` | match |  | yes |  |
| `ftGameWatch_SpecialNVars` | match |  | yes |  |
| `ftHurtboxInit` | match | walked, 0 mismatch | yes |  |
| `ftIceClimberAttributes` | match | walked, 0 mismatch | yes |  |
| `ftKb_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftKb_FighterVars` | match |  | no |  |
| `ftKb_Hat` | match |  | no |  |
| `ftKb_MotionVars` | match |  | yes |  |
| `ftKb_SpecialHiVars` | match |  | yes |  |
| `ftKb_SpecialHiVars_x10` | anonymous (checked inside its parent) |  | yes |  |
| `ftKb_SpecialHiVars_x8` | anonymous (checked inside its parent) |  | yes |  |
| `ftKb_SpecialLWVars` | match |  | yes |  |
| `ftKb_SpecialNKp_Vars` | match |  | yes |  |
| `ftKb_SpecialNMs_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftKb_SpecialNPe_Vars` | match |  | yes |  |
| `ftKirby_CopyName` | match |  | no |  |
| `ftKoopa_FighterVars` | match |  | yes |  |
| `ftKoopa_MotionVars` | match |  | no |  |
| `ftKoopa_SpecialNVars` | match |  | yes |  |
| `ftKoopa_SpecialSVars` | match |  | yes |  |
| `ftKoopa_State1Vars` | match |  | no |  |
| `ftLk_AttackAirVars` | match |  | yes |  |
| `ftLk_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftLk_FighterVars` | match |  | no |  |
| `ftLk_MotionVars` | match |  | yes |  |
| `ftLk_SpecialNVars` | match |  | yes |  |
| `ftLk_SpecialN_Vec3Group` | match |  | yes |  |
| `ftLuigiSpecialLw` | match |  | yes |  |
| `ftLuigiSpecialS` | match |  | yes |  |
| `ftLuigi_FighterVars` | match |  | yes |  |
| `ftLuigi_MotionVars` | match |  | yes |  |
| `ftMarioSpecialLw` | match |  | yes |  |
| `ftMarioSpecialS` | match |  | yes |  |
| `ftMario_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftMario_FighterVars` | match |  | no |  |
| `ftMario_MotionVars` | match |  | yes |  |
| `ftMario_SpecialHi_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftMario_SpecialLw_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftMario_SpecialLw_ECB` | match |  | yes |  |
| `ftMario_SpecialS_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftMars_FighterVars` | match |  | yes |  |
| `ftMars_MotionVars` | match |  | yes |  |
| `ftMars_SpecialLwVars` | match |  | yes |  |
| `ftMars_SpecialNVars` | match |  | yes |  |
| `ftMars_SpecialSVars` | match |  | yes |  |
| `ftMars_Unk0MotionVars` | match |  | yes |  |
| `ftMasterHand_Damage_0` | differs |  | no | pads split differently upstream; same size |
| `ftMasterHand_FingerBeamVars` | differs |  | no | as Damage_0 |
| `ftMasterHand_GrabVars` | differs |  | no | as Damage_0 |
| `ftMasterHand_MotionVars` | differs |  | no | as Damage_0 |
| `ftMasterHand_SpecialAttrs` | match | walked, 0 mismatch | yes |  |
| `ftMasterHand_Unk0Vars` | match |  | no |  |
| `ftMasterHand_Unk13Vars` | match |  | yes |  |
| `ftMasterHand_Unk4Vars` | match |  | yes |  |
| `ftMasterhand_FighterVars` | match |  | no |  |
| `ftMewtwoSpecialHi` | match |  | yes |  |
| `ftMewtwoSpecialN` | match |  | yes |  |
| `ftMewtwoSpecialS` | match |  | yes |  |
| `ftMewtwo_FighterVars` | match |  | no |  |
| `ftMewtwo_MotionVars` | match |  | yes |  |
| `ftNessAttributes` | match | walked, 0 mismatch | yes |  |
| `ftNess_FighterVars` | match |  | no |  |
| `ftNess_MotionVars` | match |  | yes |  |
| `ftNess_SpecialHiVars` | match |  | yes |  |
| `ftNess_SpecialLwVars` | match |  | yes |  |
| `ftNess_SpecialNVars` | match |  | yes |  |
| `ftNess_YoyoVars` | match |  | yes |  |
| `ftPe_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftPe_FloatAttackVars` | match |  | yes |  |
| `ftPe_ItemChance` | match | walked, 0 mismatch | yes |  |
| `ftPe_MotionVars` | match |  | yes |  |
| `ftPe_SpecialHiVars` | match |  | yes |  |
| `ftPe_SpecialNVars` | match |  | yes |  |
| `ftPe_SpecialSVars` | match |  | yes |  |
| `ftPeach_FighterVars` | match |  | no |  |
| `ftPichu_FighterVars` | match |  | yes |  |
| `ftPikachu_FighterVars` | match |  | yes |  |
| `ftPikachu_MotionVars` | match |  | no |  |
| `ftPikachu_SpecialHiVars` | match |  | yes |  |
| `ftPikachu_SpecialLwVars` | match |  | no |  |
| `ftPikachu_State2Vars` | match |  | yes |  |
| `ftPikachu_State3Vars` | match |  | yes |  |
| `ftPopo_FighterVars` | match |  | no |  |
| `ftPp_MotionVars` | match |  | no |  |
| `ftPp_MotionVars_speciallw` | anonymous (checked inside its parent) |  | yes |  |
| `ftPp_MotionVars_unk_80123954` | anonymous (checked inside its parent) |  | yes |  |
| `ftPp_SpecialSVars` | match |  | no |  |
| `ftPp_SpecialSVars_x8_t` | match |  | no |  |
| `ftPurin_FighterVars` | match |  | no |  |
| `ftPurin_MotionVars` | match |  | yes |  |
| `ftPurin_SpecialHiVars` | match |  | yes |  |
| `ftPurin_SpecialNVars` | match |  | yes |  |
| `ftSamus_FighterVars` | match |  | no |  |
| `ftSamus_GrappleVars` | match |  | yes |  |
| `ftSamus_MotionVars` | match |  | yes |  |
| `ftSamus_State2Vars` | match |  | yes |  |
| `ftSamus_State3Vars` | match |  | yes |  |
| `ftSamus_State5Vars` | match |  | yes |  |
| `ftSamus_State6Vars` | match |  | yes |  |
| `ftSamus_State7Vars` | match |  | yes |  |
| `ftSandbag_FighterVars` | match |  | yes |  |
| `ftSeakSpecialHi` | match |  | yes |  |
| `ftSeak_FighterVars` | match |  | no |  |
| `ftSeak_MotionVars` | match |  | yes |  |
| `ftSeak_SpecialNVars` | match |  | yes |  |
| `ftSeak_SpecialSVars` | match |  | yes |  |
| `ftYoshi_FighterVars` | match |  | no |  |
| `ftYoshi_GuardVars` | differs |  | yes | runtime, pointer vs u32 |
| `ftYoshi_MotionVars` | differs |  | no | runtime |
| `ftYoshi_SpecialHiVars` | match |  | yes |  |
| `ftYoshi_SpecialNVars` | match |  | yes |  |
| `ftYoshi_SpecialSVars` | match |  | yes |  |
| `ftYs_DatAttrs` | match |  | no |  |
| `ftZakoBoy_FighterVars` | match |  | yes |  |
| `ftZelda_DatAttrs` | match | walked, 0 mismatch | yes |  |
| `ftZelda_FighterVars` | match |  | no |  |
| `ftZelda_MotionVars` | match |  | yes |  |
| `ftZelda_SpecialHiVars` | match |  | yes |  |
| `ftZelda_SpecialNVars` | match |  | yes |  |
| `ftZelda_SpecialSVars` | match |  | yes |  |
| `ft_800898B4_t` | match |  | yes |  |
| `ft_80459B88_t` | differs |  | no | upstream: copies[k+1] == Rust hats[k] (trap 2 of upstream-sync.md); same bytes |
| `gmMainLib_8046B0F0_t` | match |  | yes |  |
| `gmPlayerData` | match |  | yes |  |
| `gmScriptEventDefault` | match |  | yes |  |
| `gm_801677C0_s` | match |  | no |  |
| `gm_8017DB6C_arg0_t` | match |  | yes |  |
| `gm_8019ECAC_OnEnter_t` | match |  | yes |  |
| `gm_803DDEC8Struct` | match |  | no |  |
| `gm_803DE650_t` | match |  | yes |  |
| `gm_803DF94C_t` | match |  | no |  |
| `gm_80479D58_t` | match |  | no |  |
| `gm_8049E548_t` | match |  | yes |  |
| `gm_DbPauseInputHandlers` | match |  | no |  |
| `gmm_retval_ED98` | match |  | yes |  |
| `gmm_retval_EDB0` | match |  | yes |  |
| `gmm_retval_EDBC` | match |  | yes |  |
| `gmm_x0` | match |  | no |  |
| `gmm_x0_44_t` | match |  | yes |  |
| `gmm_x0_528_t` | match |  | yes |  |
| `gmm_x0_584_t` | match |  | yes |  |
| `gmm_x0_vsdata` | match |  | yes |  |
| `gmm_x0_vsmodes` | match |  | no |  |
| `gmm_x1868_1A8_t` | match |  | yes |  |
| `grArwing_GroundVars` | match |  | yes |  |
| `grBigBlueRoute_GroundVars` | match |  | no |  |
| `grBigBlueRoute_GroundVars2` | differs |  | no | runtime ground vars retyped upstream |
| `grBigBlueRoute_Track` | match |  | no |  |
| `grBigBlue_CarLane` | match |  | yes |  |
| `grBigBlue_CarLane_anon_1386` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_CarLane_anon_1386_anon_1388` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_CarLane_anon_1386_anon_1394` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_CarVars` | match |  | no |  |
| `grBigBlue_GroundData` | match |  | yes |  |
| `grBigBlue_GroundVars` | match |  | no |  |
| `grBigBlue_GroundVars_anon_1430` | anonymous (checked inside its parent) |  | no |  |
| `grBigBlue_GroundVars_anon_1430_anon_1431` | anonymous (checked inside its parent) |  | no |  |
| `grBigBlue_GroundVars_anon_1430_anon_1431_anon_1432` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_GroundVars_anon_1430_anon_1431_anon_1432_anon_1434` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_GroundVars_anon_1430_anon_1431_anon_1432_anon_1440` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_GroundVars_anon_1430_anon_1431_anon_1432_anon_1444` | anonymous (checked inside its parent) |  | yes |  |
| `grBigBlue_ManagerVars` | match |  | no |  |
| `grBigBlue_PlatformVars` | match |  | yes |  |
| `grBigBlue_RoadVars` | match |  | yes |  |
| `grCastle_GroundVars` | match |  | no |  |
| `grCastle_GroundVars10` | match |  | no |  |
| `grCastle_GroundVars11` | match |  | yes |  |
| `grCastle_GroundVars11_xC4` | anonymous (checked inside its parent) |  | yes |  |
| `grCastle_GroundVars12` | no C record of that name |  | yes |  |
| `grCastle_GroundVars2` | no C record of that name |  | no |  |
| `grCastle_GroundVars3` | differs |  | no | runtime ground vars retyped upstream |
| `grCastle_GroundVars4` | no C record of that name |  | yes |  |
| `grCastle_GroundVars5` | match |  | no |  |
| `grCastle_GroundVars6` | match |  | yes |  |
| `grCastle_GroundVars7` | match |  | no |  |
| `grCastle_GroundVars8` | match |  | no |  |
| `grCastle_GroundVars9` | no C record of that name |  | no |  |
| `grCastle_Platform` | match |  | no |  |
| `grCorneria_GroundVars` | match |  | no |  |
| `grCorneria_GroundVars2` | match |  | no |  |
| `grCorneria_GroundVars2_xC4` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars2_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_offset_y` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_offset_y_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_xC4` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_xC6` | anonymous (checked inside its parent) |  | yes |  |
| `grCorneria_GroundVars_xC6_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grDynamicAttr_UnkStruct` | match |  | no |  |
| `grFigureGet_GroundVars` | match |  | no |  |
| `grFourside_CraneVars` | match |  | yes |  |
| `grFourside_CraneVars_x1` | anonymous (checked inside its parent) |  | yes |  |
| `grFourside_GroundVars` | match |  | no |  |
| `grFourside_GroundVars2` | match |  | yes |  |
| `grFourside_UfoVars` | match |  | no |  |
| `grGarden_GroundVars` | match |  | yes |  |
| `grGarden_GroundVars2` | match |  | no |  |
| `grGreatBay_GroundVars` | match |  | no |  |
| `grGreatBay_GroundVars2` | match |  | no |  |
| `grGreatBay_GroundVars2_x12` | anonymous (checked inside its parent) |  | yes |  |
| `grGreatBay_GroundVars3` | match |  | no |  |
| `grGreatBay_GroundVars4` | match |  | no |  |
| `grGreatBay_GroundVars_xC5` | anonymous (checked inside its parent) |  | yes |  |
| `grGreens_BlockVars` | match |  | no |  |
| `grGreens_GroundVars` | match |  | no |  |
| `grGreens_GroundVars2` | match |  | yes |  |
| `grGreens_GroundVars_x0_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grGreens_GroundVars_x0_flags_anon_1172` | anonymous (checked inside its parent) |  | yes |  |
| `grHomeRun_GroundVars` | match |  | no |  |
| `grHomeRun_GroundVars2` | match |  | yes |  |
| `grHomeRun_GroundVars_xE8_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grIceMt_BG_GroundVars` | match |  | yes |  |
| `grIceMt_FA364_State` | match |  | yes |  |
| `grIceMt_GObj10_GroundVars` | match |  | yes |  |
| `grIceMt_GObj1_GroundVars` | match |  | no |  |
| `grIceMt_GObj9_GObj10_UnderUpperIdPair` | match |  | yes |  |
| `grIceMt_GObj9_GObj10_x0` | match |  | yes |  |
| `grIceMt_GObj9_GroundVars` | match |  | yes |  |
| `grIceMt_GroundVars` | match |  | yes |  |
| `grInishie1_Block` | match |  | no |  |
| `grInishie1_GroundVars` | match |  | no |  |
| `grInishie1_GroundVars2` | match |  | no |  |
| `grInishie1_GroundVars3` | match |  | no |  |
| `grInishie2_GroundVars` | match |  | yes |  |
| `grInishie2_GroundVars2` | match |  | no |  |
| `grInishie2_GroundVars3` | match |  | yes |  |
| `grInishie2_GroundVars3_xC8_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grInishie2_GroundVars_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grKinokoRoute_GroundVars` | match |  | no |  |
| `grKinokoRoute_GroundVars2` | match |  | yes |  |
| `grKinokoRoute_GroundVars_Entry` | match |  | no |  |
| `grKongo_GroundVars` | match |  | no |  |
| `grKongo_GroundVars2` | match |  | no |  |
| `grKongo_GroundVars3` | match |  | no |  |
| `grKongo_GroundVars_u` | anonymous (checked inside its parent) |  | no |  |
| `grKongo_GroundVars_u_taru` | anonymous (checked inside its parent) |  | no |  |
| `grKraid_GroundVars` | match |  | yes |  |
| `grKraid_GroundVars2` | match |  | no |  |
| `grMuteCity_GroundVars` | match |  | no |  |
| `grMuteCity_GroundVars2` | match |  | no |  |
| `grMuteCity_GroundVars2_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grMuteCity_GroundVars_xD0_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grNBa_BG_Vars` | match |  | yes |  |
| `grOldKongo_GroundVars` | match |  | no |  |
| `grOldPupupu_GroundVars` | match |  | yes |  |
| `grOldPupupu_GroundVars2` | match |  | yes |  |
| `grOldYoshi_Cloud` | match |  | no |  |
| `grOldYoshi_Cloud_GroundVars` | match |  | no |  |
| `grOldYoshi_Guest_GroundVars` | match |  | yes |  |
| `grOnett_AwningData` | match |  | no |  |
| `grOnett_Building_GroundVars` | match |  | yes |  |
| `grOnett_Car_GroundVars` | match |  | no |  |
| `grOnett_GroundVars` | match |  | no |  |
| `grPura_GroundVars` | match |  | yes |  |
| `grPura_GroundVars2` | match |  | no |  |
| `grPura_GroundVars3` | match |  | no |  |
| `grPushOn_GroundVars` | match |  | no |  |
| `grRCruise_Entry` | match |  | no |  |
| `grRCruise_GroundVars` | match |  | no |  |
| `grRCruise_GroundVars2` | match |  | no |  |
| `grRCruise_GroundVars_xC4` | anonymous (checked inside its parent) |  | yes |  |
| `grRCruise_SubEntry` | match |  | no |  |
| `grShrineroute_GroundVars` | match |  | no |  |
| `grShrineroute_GroundVars2` | match |  | no |  |
| `grShrineroute_GroundVars3` | match |  | no |  |
| `grShrineroute_GroundVars_platforms` | anonymous (checked inside its parent) |  | no |  |
| `grSmashTaunt_GroundVars` | match |  | no |  |
| `grStadium_Display` | match |  | no |  |
| `grStadium_GroundVars` | match |  | no |  |
| `grStadium_type9_GroundVars` | match |  | no |  |
| `grStarFox_GroundVars` | match |  | no |  |
| `grStarFox_GroundVars_xC4` | anonymous (checked inside its parent) |  | yes |  |
| `grStarFox_GroundVars_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars` | match |  | no |  |
| `grVenom_GroundVars2` | match |  | no |  |
| `grVenom_GroundVars2_xE0_state` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars2_xE0_state_anon_570` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars2_xE0_state_xE0_state_pad` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_516` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_516_xC4_flags` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_529` | anonymous (checked inside its parent) |  | no |  |
| `grVenom_GroundVars_anon_533` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_537` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_541` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_GroundVars_anon_545` | anonymous (checked inside its parent) |  | yes |  |
| `grVenom_Platform_GroundVars` | match |  | no |  |
| `grYorster_GroundVars` | match |  | no |  |
| `grYorster_TrackElement` | match |  | no |  |
| `grZakoGenerator_Config` | match |  | no |  |
| `grZakoGenerator_Data` | match |  | no |  |
| `grZakoGenerator_Entry` | match |  | no |  |
| `grZakoGenerator_Spawn` | match |  | yes |  |
| `grZakoGenerator_SpawnDesc` | match |  | yes |  |
| `grZebesRoute_LightData` | match |  | yes |  |
| `grZebes_GroundVars` | differs |  | yes | upstream shrank it to 0xC (stored_jobj); runtime union member, Rust keeps 0x18 (inside the union) |
| `grZebes_GroundVars2` | match |  | yes |  |
| `grZebes_GroundVars3` | match |  | no |  |
| `grZebes_GroundVars4` | match |  | yes |  |
| `grZebes_GroundVars5` | match |  | yes |  |
| `hsd_804D0F60_t` | match |  | no |  |
| `ifMagnify` | differs |  | no | upstream: GXColor records where Rust has s32 words; runtime HUD state |
| `ifMagnifyPlayer` | match |  | no |  |
| `ifMagnifyPlayer_state` | anonymous (checked inside its parent) |  | yes |  |
| `ifStock_804A1378` | differs |  | no | upstream retyped the stock HUD table (0x204 + per-player records); runtime only |
| `ifStock_804A1378_per_player` | match |  | no |  |
| `ifStock_804A1378_x204` | no C record of that name |  | yes |  |
| `ifStock_804A1774` | match |  | no |  |
| `ifStock_804A1ACC` | match |  | no |  |
| `itArwingLaser_ItemVars` | match |  | no |  |
| `itBatAttributes` | match |  | yes |  |
| `itBombHeiAttributes` | match |  | yes |  |
| `itBombHei_ItemVars` | match |  | yes |  |
| `itBoxAttributes` | match |  | yes |  |
| `itBox_ItemVars` | match |  | no |  |
| `itCLinkMilk_ItemVars` | match |  | no |  |
| `itCapsule_ItemVars` | match |  | yes |  |
| `itChainSegment` | match |  | yes |  |
| `itChicoritaAttr` | match |  | yes |  |
| `itChicoritaLeafAttr` | match |  | yes |  |
| `itChicorita_ItemVars` | match |  | yes |  |
| `itClimbersBlizzardAttributes` | match |  | yes |  |
| `itClimbersBlizzard_ItemVars` | match |  | yes |  |
| `itClimbersIceAttributes` | match |  | yes |  |
| `itClimbersIce_ItemVars` | match |  | no |  |
| `itClimbersStringAttributes` | match |  | no |  |
| `itClimbersString_ItemVars` | match |  | no |  |
| `itCoinAttributes` | match |  | yes |  |
| `itCoin_ItemVars` | match |  | no |  |
| `itCrazyHandBombAttributes` | match |  | yes |  |
| `itDoseiAttributes` | match |  | yes |  |
| `itDosei_ItemVars` | match |  | yes |  |
| `itDrMarioPillAttributes` | match |  | yes |  |
| `itDrMarioPill_ItemVars` | match |  | no |  |
| `itEgg_ItemVars` | match |  | yes |  |
| `itEvYoshiEgg_DatAttrs` | match |  | no |  |
| `itEvYoshiEgg_ItemVars` | match |  | yes |  |
| `itFFlowerFlame_ItemVars` | match |  | yes |  |
| `itFFlower_ItemVars` | match |  | yes |  |
| `itFireAttributes` | match |  | yes |  |
| `itFlashAttributes` | match |  | yes |  |
| `itFlashExplAttributes` | match |  | yes |  |
| `itFlipper_DatAttrs` | match |  | yes |  |
| `itFlipper_ItemVars` | match |  | no |  |
| `itFoodsAttributes` | differs |  | no | flexible entries[] upstream; Rust reads the rows through FoodsRow (dat_custom) |
| `itFoods_ItemVars` | match |  | yes |  |
| `itFoxBlaster_ItemVars` | differs |  | no | runtime item vars retyped (pointers) |
| `itFoxIllusion_ItemVars` | match |  | no |  |
| `itFoxLaser_ItemVars` | match |  | yes |  |
| `itFreeze_ItemVars` | match |  | no |  |
| `itFreezerAttributes` | match |  | yes |  |
| `itFreezer_ItemVars` | match |  | yes |  |
| `itFushigibanaAttributes` | match |  | yes |  |
| `itFushigibana_ItemVars` | match |  | yes |  |
| `itGShell_HurtInit` | match |  | yes |  |
| `itGShell_ItemVars` | match |  | yes |  |
| `itGamewatch_ItemVars` | match |  | no |  |
| `itGamewatchchefAttrEntry` | match |  | yes |  |
| `itGamewatchchefAttributes` | differs |  | no | entries[5] upstream, Rust entries[1] indexed past (as the old C); word-swapped block, all 4-byte words |
| `itGamewatchchef_ItemVars` | match |  | no |  |
| `itGamewatchparachuteAttributes` | no C record of that name |  | no |  |
| `itGamewatchrescue_ItemVars` | match |  | no |  |
| `itGreatFoxLaser_Attrs` | match |  | yes |  |
| `itGreatFoxLaser_ItemVars` | match |  | no |  |
| `itHammerheadAttributes` | match |  | yes |  |
| `itHarisen_DatAttrs` | match |  | yes |  |
| `itHassam_ItemVars` | match |  | yes |  |
| `itHeart_ItemVars` | match |  | yes |  |
| `itHeart_ItemVars_xDD8` | anonymous (checked inside its parent) |  | yes |  |
| `itHeart_ItemVars_xDD8_anon_254` | anonymous (checked inside its parent) |  | yes |  |
| `itHeiho_ItemVars` | match |  | no |  |
| `itHinoarashiAttributes` | match |  | yes |  |
| `itHinoarashi_ItemVars` | match |  | yes |  |
| `itHitodemanAttributes` | match |  | yes |  |
| `itHitodeman_ItemVars` | match |  | no |  |
| `itHououAttr` | match |  | yes |  |
| `itHouou_ItemVars` | match |  | yes |  |
| `itKabigonAttributes` | match |  | yes |  |
| `itKabigon_ItemVars` | match |  | yes |  |
| `itKamexAttributes` | match |  | yes |  |
| `itKinoko_ItemVars` | match |  | yes |  |
| `itKirby2F23_ItemVars` | match |  | yes |  |
| `itKirbyCutterBeamAttributes` | match |  | yes |  |
| `itKirbyCutterBeam_ItemVars` | match |  | yes |  |
| `itKirbyHammer_ItemVars` | match |  | yes |  |
| `itKlap_ItemVars` | match |  | no |  |
| `itKoopaFlame_ItemVars` | match |  | yes |  |
| `itKusudamaAttributes` | match |  | yes |  |
| `itKusudama_ItemVars` | match |  | yes |  |
| `itKyasarinAttributes` | match |  | no |  |
| `itKyasarinEggAttributes` | match |  | no |  |
| `itKyasarinEgg_ItemVars` | match |  | yes |  |
| `itKyasarin_ItemVars` | match |  | no |  |
| `itLGunBeam_ItemVars` | match |  | yes |  |
| `itLGun_ItemVars` | match |  | yes |  |
| `itLeadeadAttr_x0` | no C record of that name |  | yes |  |
| `itLeadeadAttributes` | differs |  | no | upstream named the last pad byte x1F; Rust pad |
| `itLeadead_ItemVars` | match |  | no |  |
| `itLikelikeAttributes` | match |  | no |  |
| `itLikelikeAttributes_x0` | anonymous (checked inside its parent) |  | no |  |
| `itLikelike_ItemVars` | match |  | no |  |
| `itLinkArrowAttributes` | match |  | no |  |
| `itLinkArrow_ItemVars` | match |  | no |  |
| `itLinkBombAttributes` | match |  | yes |  |
| `itLinkBomb_ItemVars` | match |  | no |  |
| `itLinkBoomerangAttributes` | match |  | no |  |
| `itLinkBoomerang_ItemVars` | match |  | no |  |
| `itLinkBow_ItemVars` | match |  | no |  |
| `itLinkHookshotAttributes` | match |  | no |  |
| `itLinkHookshot_ItemVars` | match |  | no |  |
| `itLipstickAttributes` | match |  | yes |  |
| `itLipstickSporeAttributes` | match |  | yes |  |
| `itLipstickSpore_ItemVars` | match |  | yes |  |
| `itLizardonAttributes` | match |  | yes |  |
| `itLizardon_ItemVars` | match |  | yes |  |
| `itLuckyAttributes` | match |  | yes |  |
| `itLuckyEggAttributes` | match |  | yes |  |
| `itLucky_ItemVars` | match |  | yes |  |
| `itLugiaAttributes` | match |  | yes |  |
| `itLugia_ItemVars` | match |  | yes |  |
| `itMBallAttributes` | match |  | yes |  |
| `itMBall_ItemVars` | match |  | yes |  |
| `itMDisableAttributes` | match |  | yes |  |
| `itMDisable_ItemVars` | match |  | no |  |
| `itMarilAttributes` | match |  | yes |  |
| `itMaril_ItemVars` | match |  | yes |  |
| `itMaril_ItemVars_x60` | anonymous (checked inside its parent) |  | yes |  |
| `itMaril_ItemVars_x60_anon_1175` | anonymous (checked inside its parent) |  | yes |  |
| `itMasterHandBulletAttributes` | match |  | yes |  |
| `itMasterHandBullet_ItemVars` | match |  | yes |  |
| `itMasterHandLaserAttributes` | match |  | yes |  |
| `itMasterHandLaser_ItemVars` | match |  | no |  |
| `itMatadogasAttributes` | match |  | yes |  |
| `itMatadogas_ItemVars` | match |  | yes |  |
| `itMato_ItemVars` | match |  | no |  |
| `itMewtwoShadowball_DatAttrs` | match |  | yes |  |
| `itMewtwoShadowball_ItemVars` | match |  | no |  |
| `itMsBomb_ItemVars` | match |  | yes |  |
| `itNessPKFirepillarAttributes` | match |  | yes |  |
| `itNessYoyo_ItemVars` | match |  | no |  |
| `itNessbat_ItemVars` | match |  | no |  |
| `itNesspkthundertrail_ItemVars` | match |  | no |  |
| `itNokoNoko_DatAttrs` | match |  | no |  |
| `itNokoNoko_DatAttrs2` | match |  | yes |  |
| `itNokoNoko_ItemVars` | match |  | yes |  |
| `itOctarockAttributes` | match |  | no |  |
| `itOctarock_ItemVars` | match |  | yes |  |
| `itOldkuri_ItemVars` | match |  | yes |  |
| `itOldottoseaAttributes` | match |  | no |  |
| `itOldottoseaAttributes_anon_1640` | anonymous (checked inside its parent) |  | yes |  |
| `itOldottosea_ItemVars` | match |  | no |  |
| `itPKFlushExplode_ItemVars` | match |  | no |  |
| `itPKFlush_ItemVars` | match |  | no |  |
| `itPKThunderAttributes` | match |  | yes |  |
| `itPKThunder_ItemVars` | match |  | no |  |
| `itPatapataAttributes` | match |  | no |  |
| `itPatapataDatAttrs` | match |  | yes |  |
| `itPatapata_ItemVars` | match |  | yes |  |
| `itPeachToadSporeAttributes` | match |  | yes |  |
| `itPeachTurnipAttributes` | match |  | yes |  |
| `itPeachTurnipAttributes_x8` | anonymous (checked inside its parent) |  | yes |  |
| `itPeachTurnip_ItemVars` | match |  | no |  |
| `itPickup` | match | walked, 0 mismatch | yes |  |
| `itPikachutJoltAir_ItemVars` | match |  | no |  |
| `itPikachutJoltGroundAttributes` | match |  | yes |  |
| `itPikachutJoltGround_ItemVars` | match |  | no |  |
| `itPikachuthunderAttributes` | match |  | yes |  |
| `itPikachuthunder_ItemVars` | match |  | no |  |
| `itPokemonAttributes` | no C record of that name |  | yes |  |
| `itPokemonSpawn_DatAttrs` | match |  | yes |  |
| `itPokemonSpawn_ItemVars` | match |  | yes |  |
| `itPokemon_ItemVars` | match |  | yes |  |
| `itPokemon_ItemVars_anon_862` | anonymous (checked inside its parent) |  | yes |  |
| `itRShell_ItemVars` | match |  | yes |  |
| `itRay_ItemVars` | match |  | yes |  |
| `itRshellAttributes` | no C record of that name |  | yes |  |
| `itSScopeAttributes` | differs |  | yes | upstream named x0; Rust pad, unread |
| `itSamusBombAttributes` | match |  | yes |  |
| `itSamusBomb_ItemVars` | match |  | no |  |
| `itSamusChargeShot_Attributes` | match |  | yes |  |
| `itSamusChargeshot_ItemVars` | match |  | no |  |
| `itSamusGrappleAttributes` | match |  | no |  |
| `itSamusGrapple_Hitbox` | match |  | yes |  |
| `itSamusGrapple_HitboxData` | match |  | yes |  |
| `itSamusGrapple_ItemVars` | match |  | no |  |
| `itSamusMissileAttributes` | match |  | yes |  |
| `itSamusMissile_ItemVars` | match |  | no |  |
| `itScopeBeam_ItemVars` | match |  | yes |  |
| `itSeakChain_Attrs` | match |  | no |  |
| `itSeakChain_ItemVars` | match |  | no |  |
| `itSeakNeedleHeld_ItemVars` | match |  | no |  |
| `itSeakNeedleThrownAttributes` | match |  | yes |  |
| `itSeakNeedleThrown_ItemVars` | match |  | yes |  |
| `itSonans_ItemVars` | match |  | yes |  |
| `itStarRodStar_ItemVars` | match |  | yes |  |
| `itStar_ItemVars` | match |  | yes |  |
| `itSword_ItemVars` | match |  | yes |  |
| `itSword_UnkArticle1` | no C record of that name |  | no |  |
| `itSword_UnkBytes` | match |  | yes |  |
| `itTargetVars` | match |  | no |  |
| `itTaruAttributes` | match |  | yes |  |
| `itTaruCann_DatAttrs` | match |  | yes |  |
| `itTaru_ItemVars` | match |  | yes |  |
| `itTarucann_ItemVars` | match |  | no |  |
| `itThunderAttributes` | match |  | yes |  |
| `itThunderPokemonAttributes` | match |  | yes |  |
| `itThunder_ItemVars` | match |  | yes |  |
| `itTincleAttributes` | differs |  | yes | x0 is itSpecialAttrsHead* upstream, f32 in Rust; no reader of x0 (ittincle.c) |
| `itTincle_ItemVars` | match |  | no |  |
| `itTomatoVars` | match |  | yes |  |
| `itTomato_ItemVars` | match |  | yes |  |
| `itToolsAttributes` | differs |  | yes | flexible motions[] upstream (Rust motions[1] indexed past, ittools.rs) |
| `itToolsMotionAttrs` | match |  | yes |  |
| `itTools_ItemVars` | match |  | yes |  |
| `itTosakinto_Attrs` | match |  | yes |  |
| `itTosakinto_ItemVars` | match |  | yes |  |
| `itUnk2_DatAttrs` | match |  | yes |  |
| `itUnk2_ItemVars` | match |  | yes |  |
| `itUnk4_ItemVars` | match |  | no |  |
| `itUnkAttributes` | differs |  | yes | flexible array upstream |
| `itUnknownAttributes` | match |  | no |  |
| `itUnknownAttributes_x0` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknownAttributes_x18` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknownAttributes_x1C` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknownAttributes_x20` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknown_ItemVars` | match |  | yes |  |
| `itUnknown_ItemVars_x64` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknown_ItemVars_x68` | anonymous (checked inside its parent) |  | yes |  |
| `itUnknown_ItemVars_x78` | anonymous (checked inside its parent) |  | yes |  |
| `itWhispyAppleAttributes` | differs |  | yes | x0 pointer upstream, pad in Rust; unread |
| `itWhispyApple_ItemVars` | match |  | yes |  |
| `itWhiteBeaAttributes` | match |  | no |  |
| `itWhiteBeaAttributes_anon_1476` | anonymous (checked inside its parent) |  | yes |  |
| `itWhiteBea_ItemVars` | match |  | no |  |
| `itWstarAttrEntry` | match |  | no |  |
| `itWstarAttributes` | differs |  | no | flexible x28_entries[] upstream (itwstar.rs indexes past) |
| `itWstar_ItemVars` | match |  | yes |  |
| `itYaku_ItemVars` | match |  | no |  |
| `itYaku_ItemVars_anon_1452` | anonymous (checked inside its parent) |  | no |  |
| `itYoshiEggLay_DatAttrs` | match |  | yes |  |
| `itYoshiEggLay_ItemVars` | match |  | yes |  |
| `itYoshiEggThrowAttributes` | match |  | yes |  |
| `itYoyoAttributes` | match |  | no |  |
| `itZako_DatAttrs` | match |  | yes |  |
| `itZako_ItemVars` | match |  | no |  |
| `itZeldaDinFireExplodeAttributes` | match |  | yes |  |
| `itZeldaDinFireExplode_ItemVars` | match |  | no |  |
| `itZeldaDinFire_ItemVars` | match |  | no |  |
| `it_266F_ItemVars` | match |  | no |  |
| `it_2728_DatAttrs` | match |  | no |  |
| `it_2E5A_Attrs` | match |  | no |  |
| `it_2E5A_ItemVars` | match |  | no |  |
| `it_2E5A_SubVars` | match |  | no |  |
| `it_2E5A_TierEntry` | match |  | no |  |
| `it_8026C47C_arg0_t` | match |  | yes |  |
| `it_8026F3D4_arg1_t` | match |  | no |  |
| `it_802E5FXX_struct` | match |  | no |  |
| `it_804D6D20_t` | match | walked, 0 mismatch | no |  |
| `it_804D6D40_t` | match | walked, 0 mismatch | yes |  |
| `it_create_hitbox_0` | match |  | yes |  |
| `it_create_hitbox_4` | match |  | yes |  |
| `itkireihanaAttributes` | match |  | yes |  |
| `itkireihana_ItemVars` | match |  | yes |  |
| `itsonansAttributes` | match |  | yes |  |
| `jmp_buf` | match |  | yes |  |
| `lbAudioAx_UserData` | match |  | no |  |
| `lbCardNew_SnapshotEntry` | match |  | yes |  |
| `lbColl_80008D30_arg1` | match | walked, 0 mismatch | yes |  |
| `lbColl_8000A10C_arg0_t` | match |  | yes |  |
| `lbDvd_803B72C0_t` | no C record of that name |  | no |  |
| `lbHeap_HeapDesc` | match |  | yes |  |
| `lbHeap_HeapOffsetView` | differs |  | no | removed upstream; Rust-only view |
| `lbHeap_HeapState` | match |  | no |  |
| `lbRefract_CallbackData` | match |  | no |  |
| `lbRefract_Data` | match |  | no |  |
| `lbRefract_DataLayout` | match |  | no |  |
| `lbRefract_State` | match |  | no |  |
| `lb_00F9_UnkDesc0` | match |  | no |  |
| `lb_00F9_UnkDesc1` | match |  | yes |  |
| `lb_00F9_UnkDesc1Inner` | match | walked, 0 mismatch | yes |  |
| `lb_8000FD18_t` | match |  | yes |  |
| `lb_80011A50_t` | match |  | no |  |
| `lb_80014638_arg0_t` | match |  | yes |  |
| `lb_80014638_arg1_t` | match |  | yes |  |
| `lb_804329F0_t` | match |  | no |  |
| `lb_80432A68_38_t` | match |  | yes |  |
| `lb_804D63A0_t` | match |  | no |  |
| `lb_804D63A8_t` | match |  | no |  |
| `lb_Collider` | match |  | yes |  |
| `lbl_803BC918_t` | match |  | yes |  |
| `lbl_803D9D20_t` | match |  | yes |  |
| `lbl_803D9DAC_t` | no C record of that name |  | yes |  |
| `lbl_803D9DD0_t` | no C record of that name |  | no |  |
| `lbl_803D9F0C_t` | match |  | yes |  |
| `lbl_803DA0D0_t` | match |  | yes |  |
| `lbl_803DA2E0_t` | match |  | yes |  |
| `lbl_803FDDE4_t` | match |  | no |  |
| `lbl_803FDDE4_t_symbols` | anonymous (checked inside its parent) |  | no |  |
| `lbl_803FDDE4_t_values` | anonymous (checked inside its parent) |  | yes |  |
| `lbl_804335B8_t` | match |  | no |  |
| `lbl_8046B378_t` | match |  | no |  |
| `lbl_8046B488_t` | match |  | no |  |
| `lbl_8046B668_t` | match |  | yes |  |
| `lbl_8046B6A0_FighterMatchInfoFlags` | match |  | yes |  |
| `lbl_8046E3AC_t` | no C record of that name |  | yes |  |
| `mex_heap_desc` | match |  | yes |  |
| `mnDiagram2_SortEntry` | match |  | yes |  |
| `mnDiagram2_SortEntry_anon_15` | anonymous (checked inside its parent) |  | yes |  |
| `mnDiagram2_SortEntry_anon_19` | anonymous (checked inside its parent) |  | yes |  |
| `mnDiagram2_SortEntry_anon_9` | anonymous (checked inside its parent) |  | yes |  |
| `mnDiagram3_DataTable` | no C record of that name |  | yes |  |
| `mnDiagram3_PosTable` | no C record of that name |  | yes |  |
| `mnDiagram3_StatTable` | no C record of that name |  | yes |  |
| `mnDiagram_AnimData` | match |  | no |  |
| `mnDiagram_AnimTable` | differs |  | yes | upstream retyped mndiagram (pc hacks dropped); runtime menu |
| `mnDiagram_Assets` | differs |  | no | as mnDiagram_AnimTable |
| `mnDiagram_CleanupData` | match |  | no |  |
| `mnDiagram_MainOverlay` | match |  | no |  |
| `mnDiagram_PopupAnimTableHead` | match |  | yes |  |
| `mnDiagram_PopupData` | match |  | no |  |
| `mnGallery_804D6C88_userdata` | match |  | no |  |
| `mnGallery_child_userdata` | match |  | no |  |
| `mnSnap_804A0B90_t` | match |  | yes |  |
| `mnStageSel_804D6C98_t` | no C record of that name |  | no |  |
| `mn_802307F8_t` | match |  | no |  |
| `mn_802307F8_t_anon_18` | anonymous (checked inside its parent) |  | no |  |
| `mn_802307F8_t_anon_18_anon_20` | anonymous (checked inside its parent) |  | no |  |
| `mn_802307F8_t_anon_18_anon_20_x34` | anonymous (checked inside its parent) |  | no |  |
| `mn_80231634_t` | match |  | yes |  |
| `mn_803ED1D0_t` | differs |  | yes | static menu table, pad vs field split |
| `mpCollisionBox` | match |  | yes |  |
| `mpIsland_80458E88_t` | match |  | no |  |
| `mpIsland_Palette` | match |  | yes |  |
| `mpIsland_PaletteEntry` | match |  | yes |  |
| `mp_UnkStruct0` | no C record of that name |  | no |  |
| `mp_UnkStruct3` | no C record of that name |  | yes |  |
| `mpisland` | no C record of that name |  | yes |  |
| `part_anim` | match |  | yes |  |
| `perf` | match |  | yes |  |
| `plActionStats` | match |  | yes |  |
| `plActionStats_anon_65` | anonymous (checked inside its parent) |  | yes |  |
| `plActionStats_anon_65_anon_67` | anonymous (checked inside its parent) |  | yes |  |
| `plAllocInfo` | match |  | yes |  |
| `plAllocInfo2` | match |  | yes |  |
| `plAllocInfo2_anon_34` | anonymous (checked inside its parent) |  | yes |  |
| `plAllocInfo_anon_14` | anonymous (checked inside its parent) |  | yes |  |
| `plAttackStats` | match |  | yes |  |
| `pl_800386E8_arg0_t` | match |  | yes |  |
| `pl_804D6470_t` | match | walked, 0 mismatch | yes |  |
| `pl_StaleMoveTableExt_t` | differs |  | yes | as StaleMoveTable |
| `pl_StaleMoveTableExt_t_xDD0` | anonymous (checked inside its parent) |  | yes |  |
| `pl_StaleMoveTableExt_t_xDD0_anon_239` | anonymous (checked inside its parent) |  | yes |  |
| `pl_StaleMoveTableExt_t_xDD1` | anonymous (checked inside its parent) |  | yes |  |
| `pl_StaleMoveTableExt_t_xDD1_anon_252` | anonymous (checked inside its parent) |  | yes |  |
| `pl_x5EC_t` | match |  | yes |  |
| `pl_x5EC_t_x10` | anonymous (checked inside its parent) |  | yes |  |
| `pl_x8C0_t` | match |  | yes |  |
| `pl_x8C0_t_x4` | anonymous (checked inside its parent) |  | yes |  |
| `pl_x8D8_t` | match |  | yes |  |
| `pl_x8D8_t_x4` | anonymous (checked inside its parent) |  | yes |  |
| `psdisp_Mtx` | match |  | yes |  |
| `psdisp_ParticleSortBucket` | match |  | no |  |
| `psdisp_Tlut` | match |  | yes |  |
| `pseudo_random_sfx_0` | match |  | yes |  |
| `pseudo_random_sfx_1` | match |  | yes |  |
| `r13_ItemTable` | match |  | no |  |
| `script_caps_armor_record` | no C record of that name |  | no |  |
| `script_caps_armour` | no C record of that name |  | no |  |
| `script_caps_effect` | no C record of that name |  | no |  |
| `sdata_ItemGXLink` | match |  | no |  |
| `set_airborne_state` | match |  | yes |  |
| `set_article_vis` | match |  | yes |  |
| `set_cmd_var` | match |  | yes |  |
| `set_dobj_flags` | match |  | yes |  |
| `set_fighter_vis` | match |  | yes |  |
| `set_hitbox_damage` | match |  | yes |  |
| `set_hitbox_scale` | match |  | yes |  |
| `set_hitbox_x42_b57` | match |  | yes |  |
| `set_hurt_state` | match |  | yes |  |
| `set_jab_combo` | match |  | yes |  |
| `set_jab_rapid` | match |  | yes |  |
| `set_tex_anim` | match |  | yes |  |
| `set_throw_flags` | match |  | yes |  |
| `set_throw_hitbox_0` | match |  | yes |  |
| `set_throw_hitbox_1` | match |  | yes |  |
| `set_throw_hitbox_2` | match |  | yes |  |
| `sisLib_803A7664_t` | no C record of that name |  | no |  |
| `sislib_UnkAlloc3` | match |  | no |  |
| `smash_charge_0` | match |  | yes |  |
| `smash_charge_1` | match |  | yes |  |
| `sound_effect_0` | match |  | yes |  |
| `sound_effect_1` | match |  | yes |  |
| `sound_effect_2` | match |  | yes |  |
| `soundtest_user_data` | match |  | no |  |
| `spawn_gfx_0` | match |  | yes |  |
| `spawn_gfx_1` | match |  | yes |  |
| `spawn_gfx_2` | match |  | yes |  |
| `spawn_gfx_3` | match |  | yes |  |
| `spawn_gfx_4` | match |  | yes |  |
| `spawn_hitbox_0` | match |  | yes |  |
| `spawn_hitbox_1` | match |  | yes |  |
| `spawn_hitbox_2` | match |  | yes |  |
| `spawn_hitbox_3` | match |  | yes |  |
| `spawn_hitbox_4` | match |  | yes |  |
| `spawn_hitbox_5` | match |  | yes |  |
| `spawn_hitbox_skip` | no C record of that name |  | yes |  |
| `stage_sfx_0` | match |  | yes |  |
| `stage_sfx_1` | match |  | yes |  |
| `stage_sfx_2` | match |  | yes |  |
| `stage_sfx_3` | match |  | yes |  |
| `stagelistinfo` | no C record of that name |  | no |  |
| `strings` | match |  | no |  |
| `tyDispData` | match |  | no |  |
| `tyLightData` | match |  | no |  |
| `tyUnkStruct` | match |  | no |  |
| `un_80301C64_t` | match |  | no |  |
| `un_80304138_objalloc_t` | match |  | no |  |
| `un_80304138_objalloc_t_x8` | match |  | no |  |
| `un_804A1F48_t` | match |  | no |  |
| `un_804D6E68_t` | match |  | yes |  |
| `un_804D6EF4_t` | match |  | no |  |
| `unk0` | match |  | yes |  |
| `unk1` | match |  | yes |  |
| `unk10` | match |  | yes |  |
| `unk11` | match |  | yes |  |
| `unk12` | match |  | yes |  |
| `unk13` | match |  | yes |  |
| `unk14` | match |  | yes |  |
| `unk15` | match |  | yes |  |
| `unk16` | match |  | yes |  |
| `unk17` | match |  | yes |  |
| `unk18` | match |  | yes |  |
| `unk19` | match |  | yes |  |
| `unk20` | match |  | yes |  |
| `unk21` | match |  | yes |  |
| `unk27` | match |  | yes |  |
| `unk3` | match |  | yes |  |
| `unk31` | match |  | yes |  |
| `unk32` | match |  | yes |  |
| `unk33` | match |  | yes |  |
| `unk4` | match |  | yes |  |
| `unk5` | match |  | yes |  |
| `unk6` | match |  | yes |  |
| `unk8` | match |  | yes |  |
| `unk9` | match |  | yes |  |
| `unk_fx_0` | differs |  | yes | gfx_id is one 16-bit bitfield upstream, two 8-bit ones in Rust; same bits |
| `wind_fx_0` | match |  | yes |  |
| `wind_fx_1` | match |  | yes |  |
| `wind_fx_2` | match |  | yes |  |
| `wind_fx_3` | match |  | yes |  |
| `x1C_struct` | match |  | yes |  |
| `xB6C_t` | match |  | no |  |
