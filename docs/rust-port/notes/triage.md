# First-divergence triage of the failing coverage scenarios (2026-11-02)

Tool: `tools/rust-port/golden/coverage/triage.py`. Result: `docs/rust-port/golden/coverage/triage.tsv` (one row per failing wired
scenario: first frame, first function, its Rust TU and file / C file, first field, crash or divergence, hash columns, C trace cache
path, the sequence difference with both sides' names, early hash-invisible differences, notes). Run: 26 failing wired scenarios of
`run_rust.sh --all` (53 pass, 26 fail, 79 not wired, 10 no golden), engine at the commit this branch was cut from (f7d0d025d).

## How it works
```
python tools/rust-port/golden/coverage/triage.py --run-all -j 2        # or --results R.tsv (run_rust.sh --out) or NAME...
python tools/rust-port/golden/coverage/triage.py --summary-only --md clusters.md
```
Per scenario: (1) Rust engine run (MELEE_ENGINE_PRE_DRAWS calibrated like run_rust.sh), `compare_golden.py` for the first divergent
frame and hash columns, engine stop (panic) vs divergence; (2) C run of the call-trace build (`build_trace_reuse.sh` into
`_build/agents/triage`) with `MELEE_CALLTRACE_SEQ` over frames N-60..N+1 and a `MELEE_SNAP_DUMP` of frame N; both are CACHED in
`<root>/ctcache/<C build commit from build-provenance.json>/<scenario>/` (`c_<lo>-<hi>.seq`, `.snap`, `meta.json`, plus the map), so a
re-run skips the C side (a wider window, 400 frames, is only recorded when 60 show no difference). `meta.json` records whether the C
hash CSV of the trace build equals the golden (it did for all 26); (3) Rust run with the call trace (`build/cargo-ct`, feature
`melee_core/calltrace`) over the same window; (4) `seqdiff.py --anchor Fighter_8006A1BC`; (5) `melee diff --frame N --snap` (the
engine diff works for any wired scenario with `--snap`) and the first hashed (`*`) field; (6) git grep for the Rust TU file and C file.

Noise handling (all learned from the traces themselves, listed in the TSV note column):
- names whose per-chunk call count differs, or that sit at different places, in chunks of known-good frames (frame N-20 and earlier;
  about 700-900 names, C-side inlining, frame-boundary render work) are dropped from the comparison;
- a remaining difference more than 20 frames before the hash divergence (the hash kept matching) is recorded in
  `early_hash_invisible` as `C name/Rust name@frame`, its pair is learned as noise and the search goes on. The recurring one is
  `gm_8016b248` (C) = `player_getdefenseratio` (Rust), a naming/inlining pair, not a bug;
- a difference where one sequence just ends is the window edge, not a divergence.

Limits: the trace holds function ENTRIES only, no arguments; argument/value differences show up in `first_field`
(the first field the golden hash reads, from `melee diff`) not in the call sequence. In "different call (C X / Rust Y)" X is the
call C makes at the first differing position and Y what the engine calls there; either side may be a rename or an inlined helper, so the
TU / file columns locate X (Rust TU where it exists in `rust/*/src`, else `C:<file stem>` with the C file in `c_file`). Crashes have
no Rust trace (the engine dies before writing it): the function is the one enclosing the panic site. Scenarios with no sequence
difference are clustered by their first differing field. Always check the first frame against the hash frame: a difference 10-20 frames
earlier is a state difference the hash only shows later (typical for item / hitbox / collision functions).
`first_field` for 'not hashed' means no hashed field was in the first 500 differences (the hash covers hidden state s0/s2/itemsum).

## Clusters (ranked by scenario count)
### Clusters by first function

| first function | n | TU / C file | kinds | scenarios (first frame) |
|---|---|---|---|---|
| `hsd_texpsimplify` | 3 | C:texpdag / src/sysdolphin/baselib/texpdag.c | divergence | stage_11_rainbowcruise (177), stage_14_temple (2477), items_hi_yoshistory (2805) |
| `lbcoll_copyhitcapsule` | 3 | C:lbcollision / src/melee/lb/lbcollision.c | divergence | cpulv9_marth_sheik (1798), char_zelda (2446), stage_19_mushroomkingdom (2662) |
| `(value only) Fighter cur_pos.x` | 2 | - / - | divergence | stage_22_venom (2777), char_captain (3502) |
| `it_802753bc` | 2 | C:it_2725 / src/melee/it/it_2725.c | divergence | cpu_ffa4_levels (120), char_mewtwo (2116) |
| `lb_8000b1cc` | 2 | C:lb_00B0 / src/melee/lb/lb_00B0.c | divergence | char_kirby (755), stage_20_mushroomkingdom2 (803) |
| `(value only) Fighter cur_pos.y` | 1 | - / - | divergence | char_yoshi (742) |
| `gm_8016b248` | 1 | C:gmvs / src/melee/gm/gmvs.c | divergence | char_younglink (1099) |
| `ground_801c3fa4` | 1 | C:ground / src/melee/gr/ground.c | divergence | stage_07_corneria (955) |
| `hsd_8039dad4` | 1 | C:generator / src/sysdolphin/baselib/generator.c | divergence | stage_02_fountain (3275) |
| `hsd_aobjremove` | 1 | C:aobj / src/sysdolphin/baselib/aobj.c | divergence | char_ness (1795) |
| `hsd_fobjremoveall` | 1 | C:fobj / src/sysdolphin/baselib/fobj.c | divergence | stage_13_greatbay (1623) |
| `lbcoll_80008688` | 1 | C:lbcollision / src/melee/lb/lbcollision.c | divergence | char_mario (455) |
| `lbcopyjobjsrt` | 1 | C:lb_00B0 / src/melee/lb/lb_00B0.c | divergence | sheik_cpu_vs_zelda_cpu (442) |
| `lbvector_diff` | 1 | C:lbvector / src/melee/lb/lbvector.c | divergence | stage_25_icemt (1472) |
| `mpcoll_8004ca6c` | 1 | C:mpcoll / src/melee/mp/mpcoll.c | divergence | cpulv5_puff_link (673) |
| `mplib_8004ed5c` | 1 | C:mplib / src/melee/mp/mplib.c | divergence | char_bowser (791) |
| `player_checkslot` | 1 | player / src/melee/pl/player.c | crash | items_hi_stadium (3529) |
| `resolveenvelope` | 1 | pobj / src/sysdolphin/baselib/pobj.c | crash | stage_08_yoshistory (448) |
| `scriptgame_stageslotprocbegin` | 1 | - / - | divergence | char_roy (787) |

### Clusters by TU

| TU | n | TU / C file | kinds | scenarios (first frame) |
|---|---|---|---|---|
| `C:lbcollision` | 4 | C:lbcollision / src/melee/lb/lbcollision.c | divergence | char_mario (455), cpulv9_marth_sheik (1798), char_zelda (2446), stage_19_mushroomkingdom (2662) |
| `C:lb_00B0` | 3 | C:lb_00B0 / src/melee/lb/lb_00B0.c | divergence | sheik_cpu_vs_zelda_cpu (442), char_kirby (755), stage_20_mushroomkingdom2 (803) |
| `C:texpdag` | 3 | C:texpdag / src/sysdolphin/baselib/texpdag.c | divergence | stage_11_rainbowcruise (177), stage_14_temple (2477), items_hi_yoshistory (2805) |
| `(value only) Fighter cur_pos.x` | 2 | - / - | divergence | stage_22_venom (2777), char_captain (3502) |
| `C:it_2725` | 2 | C:it_2725 / src/melee/it/it_2725.c | divergence | cpu_ffa4_levels (120), char_mewtwo (2116) |
| `(none: no call difference found)` | 1 | - / - | divergence | char_roy (787) |
| `(value only) Fighter cur_pos.y` | 1 | - / - | divergence | char_yoshi (742) |
| `C:aobj` | 1 | C:aobj / src/sysdolphin/baselib/aobj.c | divergence | char_ness (1795) |
| `C:fobj` | 1 | C:fobj / src/sysdolphin/baselib/fobj.c | divergence | stage_13_greatbay (1623) |
| `C:generator` | 1 | C:generator / src/sysdolphin/baselib/generator.c | divergence | stage_02_fountain (3275) |
| `C:gmvs` | 1 | C:gmvs / src/melee/gm/gmvs.c | divergence | char_younglink (1099) |
| `C:ground` | 1 | C:ground / src/melee/gr/ground.c | divergence | stage_07_corneria (955) |
| `C:lbvector` | 1 | C:lbvector / src/melee/lb/lbvector.c | divergence | stage_25_icemt (1472) |
| `C:mpcoll` | 1 | C:mpcoll / src/melee/mp/mpcoll.c | divergence | cpulv5_puff_link (673) |
| `C:mplib` | 1 | C:mplib / src/melee/mp/mplib.c | divergence | char_bowser (791) |
| `player` | 1 | player / src/melee/pl/player.c | crash | items_hi_stadium (3529) |
| `pobj` | 1 | pobj / src/sysdolphin/baselib/pobj.c | crash | stage_08_yoshistory (448) |


## Reading the clusters
- Collision / hitbox (`lbcoll_copyhitcapsule` vs `scriptgame_echoiscapsule`, `lbcoll_80008688`, `mpcoll_8004ca6c`, `mplib_8004ed5c`): 7
  scenarios (cpulv9_marth_sheik, char_zelda, stage_19, char_mario, cpulv5_puff_link, items_hi_yoshistory, char_bowser). Highest value: one fix may clear several.
- Item update (`it_802753bc` vs `it_8027346c`, 10 frames before the item msid / fighter hidden-state divergence): char_mewtwo, cpu_ffa4_levels.
- Texture / particle plumbing (`hsd_texpsimplify` vs `hsd_texpref`, `hsd_aobjremove`, `hsd_fobjremoveall`, `hsd_8039dad4`): 6
  scenarios; the first-differing call is the render-side texture reg-alloc or generator path running one frame apart; the hash
  divergence follows through RNG draws or item msid.
- `lb_8000b1cc` vs `ftlib_80086fd4`, `lbcopyjobjsrt`: char_kirby, stage_20, sheik_cpu_vs_zelda_cpu (joint / animation copy; bone dynamics
  differ in `melee diff`).
- Crashes: items_hi_stadium (`player_checkslot`, garbage player_id 192), stage_08_yoshistory (`pobj.rs:204` resolveenvelope assert).
- Value-only (no call difference, 3 scenarios: venom, captain, yoshi): fighter position `cur_pos`, probably a float (FMA) site.

Re-run after fixes: the C side is cached per C build commit, so only the Rust runs repeat (about 10 minutes for all 26).


## Oracle regen on Windows (port/regen1, 2026-11-08)

Suite through a per-binary runner (oracle flags, one test thread): 1037 test targets, 5 do not compile
(tu_geno_game, tu_geno_game_g, tu_script_game, tu_script_game_t, tu_shadow), 583 pass, 449 fail. 307 of the 449
were `fnsym::require_unmerged` on every harness: MSVC link.exe folds identical COMDATs in a release link (/OPT:ICF),
after rustc, so `-Zmerge-functions=disabled` alone does not keep the twins apart on Windows. oracle_tests.sh now adds
`-Clink-arg=/OPT:NOICF` there (checked on tuo_gmmultiman: guard gone, 37 of 2060 cases differ in gm_801B6428).

**Blocker: the C side cannot be regenerated against the current headers yet.** The upstream sync 363720e12 (116
commits, incl. #3583 "name all anonymous types") renamed record members (`HitCapsule.x134.owner`,
`UnkFlagStruct.x0.b0`, ...) and removed headers (`axdriver.static.h`), but rust/melee_types/src/gen.rs was not
regenerated (`gen_types.py --check` stops on "missing TU records in src/melee/mn/mndiagram.c"). So:
- tuoracle.py (tu_geno_game*, tu_script_game*) fails at once: all_types.c includes the removed header.
- gen/mtu/run_it.py itfoods: "rust_names: no melee_types block for CommandInfo".
- gen/kinds itsamusgrapple: the C driver builds after fixture renames (`.x134.owner`, `.x0.bN`, `(void*)` casts,
  `xD0` int), but the generated Rust walker then names C members melee_types does not have (no field `x134`, `x0`,
  `xDAA`): no compile. Its one-line hand edit stays until melee_types is regenerated.
- oracle/tuo gmmultiman regenerates but then 2060 of 2060 cases differ (old data: 37): the C walk and the stale
  melee_types disagree, so the old data is kept.
- The generators also rewrite the port's `// BEGIN GENERATED EXT` block and drop the hand-wired fallbacks
  (`=> crate::tu::...`): after any regen, restore `src/`.
Order of work: regenerate melee_types for the post-sync headers (and fix the port's uses of the renamed members),
then regen group B, the mtu/kinds hash set, gmmultiman, itfoods, grapple and the NaN-class tests (lower.py's fptoi
select only reaches the data then).

- gwtool: gwtool.cpp already had the `fcmp uno` select; rebuilt into _build/agents/regen1/gwtool (fptosi -> select
  on fcmp uno checked on a sample module; the shared _build/gwtool/gwtool.exe still lacks it and was not replaced:
  every game TU must be rebuilt after it). No TU oracle uses gwtool.exe: they all apply lower.py.
- log_oracle: re-recorded with the user name injected as "Testy" (oracle/log_oracle.c wraps GetUserNameA; the test
  uses the same name).
- tu_shadow (open decision): regeneration changes no data. The C oracle stubs PSVECScale (traced, the C mtx is not
  linked), while the port calls `mtx_pc::raw::PSVECScale` directly since d323c49bb, so the generated test still sets
  a hook that no longer exists (compile error). With that line dropped, ops 0-16 agree and op 17
  (HSD_ViewingRectAddRect) agrees on return and state but differs in the trace. Pick one: link the real PSVECScale
  into the C oracle, or call it through `ext` again in the port.

Expected-fail by design (listed here, not gated; native C cannot reproduce the GC layout these depend on):
oracle_ftyoshispecialn, tu_ftCo_YoshiEgg, kinds_itsword, kinds_itmariofireball / kinds_itluigifireball (unfused square).


## melee_types regenerated for the upstream sync (port/mrg2, 2026-11-08)

`gen_types.py` follows the post-sync headers again (`--check` passes, 2051 records, every layout cross-checked against
the ppc32 dump). What the sync (363720e12, incl. #3583 "name all anonymous types") changed for the generator:
- the baselib / mn / gm / lb `.static.h` files are gone: their records live in the TUs. `write_probe` extracts them
  from the .c (devcom.c HSD_DevCom; synth.c HSD_SynthSFXGroup/Node and the three `static struct` tables; lbheap.c
  Heap/lbHeap_HeapState/HeapDesc; lbmemory.c LBMgr/Allocator; state.c matstate; mndiagram3.c, mndatadel.c (+ its
  WarnCommonJoints enum), gmtoulib.c, mnstagesel.c StageSelUserData; grzebes.c grZe_AcidState). Hoisted records come
  from their new headers (mncharsel.h, gr/types.h Ground_801C34AC_entry, lbmemory.h HSD_AllocEntry); dolphin/ax.h and
  axfx.h are included directly (they came through axdriver.static.h).
- `rust/melee_types/gen/port_legacy.h` holds the records upstream DROPPED that the port still reads, in the decomp's
  pre-sync words, so the one generator still produces their native type, GC layout and decoder: sislib's SIS /
  TextKerning / sisLib_803A7664_t views (sislib.c reads `u8*` now), the item attribute views (itPokemonAttributes,
  itRshellAttributes, KinokoAnim, itSword_UnkArticle1, SisFontData, the 5-float itUnkAttributes as
  `itUnkAttributes_legacy`), mndiagram.c's asset / anim-table layouts (`*_legacy`), WarnCmnData, the mnstagesel model
  table, StageListInfo (its closing brace sits inside `#if TARGET_PC`, so no verbatim extent). A record that only
  moved keeps its name; a record whose shape changed and that the port mirrors keeps the OLD shape under `_legacy`.
- `arrays.tsv` keys renamed with the headers (UnkStageDat.unk0 -> count, UnkStageDat_x8_t.anims/matanims/shapeanims,
  Ground_801C34AC_entry.pairs). The generator now refuses a rule whose `Record.field` no longer exists
  (`check_field_rules`): the three anim-table keys had silently stopped decoding the NULL-terminated tables, and the
  goldens segfaulted on a joint instance whose target was never registered. ftDynamics.x10 is the `s32` bone-set word
  upstream types it as, so the FigaTree*** post-decode hack is gone; ftData_x20.x0 is one joint (`x0->child`), not a
  table of 3.
- Carrying the port (rust/*/src): a compiler-driven rename pass (scratch tools: `build_map.py` diffs the old and new
  gen.rs by GC offset / bit position into a rename map, `fixanon.py` applies it at the exact spots `cargo check`
  reports, round after round) did ~5500 member/type renames; the type changes were done by hand: ssize_t render /
  line callbacks (isize; adapters narrow to the runtime ports that still take i32), `u8*` image pointers, the
  Handle / HSD_AllocEntry split in lbmemory, HSD_Text's state stack, HSD_AObj.jobj, the x4_specialAttributes /
  ftData_Item unions, CollVtx, the castle GroundVars variants merged into castle3, mpIsland, ifStock_804A157C,
  Camera x35C/x368 unions, Fighter x5A4/x5A8 as usize. Every workspace library compiles.
- `OPAQUE_FIELDS` (gen_types.py): a pointer the port reads as raw big-endian bytes stays `d.ptr::<c_void>(..)`
  even when upstream typed it (Article.x4_specialAttributes, ItemStateDesc.xC_script, the vertex / TLUT / image /
  EF data pointers, TmBoxArrays.box2..4): a typed decode there pulled a native copy out of game memory and the
  item scripts diverged (itemsum golden). `ftDynamics_x0.ftDynamicBones` decodes by `extent` (every row of the
  table in the DAT), not by `dynamicsNum`: Puff's dynamics walk reads rows beyond the count
  (ftCo_8009DC54 -> lb_8000FD48) and the hero replays crashed on them with the `count` rule.
- Verification on port/mrg2: `gate.sh` GATE OK (4/4 goldens 1801/1801/3601/3601, synctest 0 mismatches, audio
  mix hash equal, baselines 144, lints, snapguard under baseline); the replay sets against the m5 baselines:
  hero 392/428, local 125/125, own 917/917, no row changed.
- Oracle data regenerated with the existing generators against the new headers (each run rewrites the
  `BEGIN GENERATED EXT` blocks in src/; `git checkout -- rust/*/src` after every batch, so src/ is exactly the
  compile-carried port): runtime mtu (shadow with the real PSVECScale linked into the C oracle, archive,
  lbcardgame, particle, wobj + the 32 that ran unchanged), lbtu (4), kinds (all 82 minus the stale-by-design
  three; the spec strings in gen/kinds/specs_cfg*.py / cfg_809.py carry the renames: UnkFlagStruct.x0.bN,
  itMaril x60.x0_s, itYaku x4_u, HitCapsule.x134, itSpecialAttrsHead for the itLeadead / itLikelike / itTincle
  heads, ItemSpecialAttributes / ftData_ExtAttr union casts, efAsync_Spawn's EF_QueuedEffect** queue head),
  tuoracle group B (tu_geno_game*, tu_script_game*), modes tuo (8), chara oracle (10 of 14), items mtu toy.
  `port_legacy.h` takes `PORT_LEGACY_ITEMS_ONLY` so a kinds oracle pulls only the item views.
- Generators that still do not run against the post-sync headers (their tests keep the committed data, which the
  suite below checks): melee_world mtu / if / gr_oracle / tuo specs (member names: UnkStageDat unk0/unk4/unk8,
  `cmd->u` patch texts, soundtest x10 union, textlib; specs_if.py has a pre-existing `setup_c` repeated keyword
  SyntaxError), melee_fighter regen scripts (19 cref state/stub headers), chara oracle ftsamusspeciallw1 /
  ftsamusspecialn / ftyoshispecialhi / ftyoshispeciallw, runtime lbarq (its `handle->node` patch is gone with
  lbArqHandle), lbtu lbsnap / lbspdisplay / lbvector, items mtu run_tu stems (mk_driver must rewrite world.h first;
  the walker now descends the newly named unions and asks for pick_fn / pick_HSD_DObj pools the per-item
  harness does not have) and run_it stems incl. tu_itfoods (same pick_fn), item (`xDAA_byte` patch text).
  Stale by design and left at their 2bcd8c9ee data: oracle_ftyoshispecialn, tu_ftCo_YoshiEgg, kinds_itsword,
  kinds_itmariofireball, kinds_itluigifireball.
- Oracle suite on port/mrg2 (per-binary runner with oracle_tests.sh's flags `-Zmerge-functions=disabled
  -Clink-arg=/OPT:NOICF`, shared target `_build/cargo-shared/oracle`, 2 build jobs, tests serial; a first attempt
  with 6 jobs was stopped by the harness for low system memory). Before (port/regen1): 1037 targets, 5 no-compile,
  583 pass / 449 fail (142 with /OPT:NOICF). After: 1037 targets, **478 do not compile**, 559 ran: 439 pass /
  120 fail. 38 fixed (tu_toy, tu_lbcardgame, tu_lbbgflash, tu_sobjlib, lbtu_lbsnap, 8 ftCo_Item*/CliffJump, 6
  modes, 9 world...), 16 flipped from pass to fail, 64 of the 120 also failed at regen1.
  - The 478 no-compile targets (melee_items 190, melee_fighter 133, melee_chara 53, melee_modes 50, melee_world 42,
    melee_runtime 8, melee_types 1, melee_net 1) are the tests' own Rust: hand-written test bodies and the generated
    glue (tests/mtu/*_gen.rs, kinds_gen, cref) still spell the pre-sync members (`anon_N`, `hsd_obj`, `x0_f32`,
    `itLeadeadAttr_x0`...). The rename pass was run over src/ only; on melee_runtime's 8 the map fixes 12 of 56
    errors, the rest are bitfield accessors on renamed unions, dropped statics and retyped fields, i.e. the same
    hand work as src/. The glue of a test whose generator runs is better regenerated than renamed.
  - The new state-hash failures at `op 0 case 0` (runtime tu_jobj/mobj/pobj/tobj/cobj/class/fog/lbcollision/
    lbrefract/state_native, world tu_grfigure1-3/grflatzone, modes tuo_gmmultiman) are tests whose C data WAS
    regenerated here and whose Rust glue compiles: the walker now fills the retyped fields differently
    (HSD_AObj.jobj is a pooled HSD_JObj* and gets `pick_HSD_JObj()`, ids are pointer-sized and hash through
    `pi()`), and one side disagrees from the first op; MTU_DBG=<op>:<case> on both halves is the next step. The
    ones whose data was NOT regenerated (world tu_gr* cref/mtu, tu_cm, fighter ftCo_Lift/ItemScopeStart/YoshiEgg,
    modes tu_gm_1A36/gmgiant/lb_020A/lb_0219, tu_lbarq, tu_wobj) carry C hashes of the pre-sync native layouts.
  - tu_shadow does not compile (1 error in its glue), so op 17 (HSD_ViewingRectAddRect) is unverified; its C
    oracle now links the real PSVECScale (mtx_pc.c copy in lbspecs_shadow.py) and its data was regenerated.
  - Still failing (first panic line):

  | target | first panic line |
  |---|---|
  | `melee_chara__oracle_ftyoshispecialn` | melee_chara\tests\oracle_ftyoshispecialn.rs:731:9:: assertion `left == right` failed: sizeof(Fighter) |
  | `melee_chara__tu_ftcrazyhandrockcrush0` | melee_chara\tests\tu_ftcrazyhandrockcrush0.rs:73:13:: assertion `left == right` failed: phys line 11 |
  | `melee_chara__tu_ftkirbyspecialyoshi` | melee_chara\tests\tu_ftkirbyspecialyoshi.rs:191:9:: assertion `left == right` failed |
  | `melee_chara__tu_ftkirbyyoshiegg` | melee_chara\tests\tu_ftkirbyyoshiegg.rs:70:9:: assertion `left == right` failed: case 14 |
  | `melee_chara__tu_ftmasterhandbackdisappear` | melee_chara\tests\tu_ftmasterhandbackdisappear.rs:31:13:: assertion `left == right` failed: phys line 11 |
  | `melee_chara__tu_ftmasterhanddamage0` | melee_chara\tests\tu_ftmasterhanddamage0.rs:105:5:: assertion failed: take().contains("stop 1,stop 12,stop 13,capwait 3000") |
  | `melee_chara__tu_ftmasterhandfingerbeam` | melee_chara\tests\tu_ftmasterhandfingerbeam.rs:77:5:: assertion `left == right` failed |
  | `melee_chara__tu_ftmasterhandfingergun` | melee_chara\tests\tu_ftmasterhandfingergun.rs:146:17:: assertion `left == right` failed: phys line 2 |
  | `melee_chara__tu_ftmasterhandpoke` | melee_chara\tests\tu_ftmasterhandpoke.rs:75:13:: assertion `left == right` failed: phys line 2 |
  | `melee_data__player_oracle` | melee_data\tests\player_oracle.rs:459:75:: called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "The system cannot find the file specified." } |
  | `melee_fighter__ftdmgfall_ref` | melee_fighter\tests\ftdmgfall_ref.rs:175:13:: assertion `left == right` failed: op 1 case 1 |
  | `melee_fighter__tu_ftCo_Barrel` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 9 case 21 |
  | `melee_fighter__tu_ftCo_BarrelWait` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 6 case 1 |
  | `melee_fighter__tu_ftCo_CaptureKirby` | melee_fighter\tests\shard121\mod.rs:139:9:: assertion `left == right` failed: case 13 |
  | `melee_fighter__tu_ftCo_FlyReflect` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 0 case 9 |
  | `melee_fighter__tu_ftCo_ItemParasolOpen` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 0 case 92 |
  | `melee_fighter__tu_ftCo_ItemScopeStart` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 5 case 3 |
  | `melee_fighter__tu_ftCo_KinokoGiantStart` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 1 case 0 |
  | `melee_fighter__tu_ftCo_KinokoSmallStart` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 1 case 0 |
  | `melee_fighter__tu_ftCo_LandingAir` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 0 case 58 |
  | `melee_fighter__tu_ftCo_Lift` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 13 case 16 |
  | `melee_fighter__tu_ftCo_WarpStar` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 8 case 174 |
  | `melee_fighter__tu_ftCo_YoshiEgg` | melee_fighter\tests\tu_ftco_harness\mod.rs:85:9:: assertion `left == right` failed: op 2 case 0 |
  | `melee_fighter__tu_ft_0899` | melee_fighter\tests\tu_ft_0899.rs:147:9:: assertion `left == right` failed: line 498 |
  | `melee_fighter__tu_ftafterimage` | melee_fighter\tests\tu_ftafterimage.rs:50:13:: assertion `left == right` failed: vertex 14 of C 3 2 c1105604 3f066666 3f6432ca bf91bda5 340322a13f24c7fc c25fcccd 42c66666 40e2e148 beb5b574 3f41205c be |
  | `melee_fighter__tu_ftbosslib` | melee_fighter\tests\tu_ftbosslib.rs:19:9:: assertion `left == right` failed: BE40 c3704c1c c1c59f23 80000000 440c1bad 44b32376 439af915 444c8000 bfc79a99 449c1532 450e1518 44d37c0f |
  | `melee_fighter__tu_ftcamera` | melee_fighter\tests\tu_ftcamera.rs:87:13:: assertion `left == right` failed: case 2 727 |
  | `melee_fighter__tu_ftwalkcommon` | melee_fighter\tests\tu_ftwalkcommon.rs:136:9:: assertion `left == right` failed: line 1 |
  | `melee_fptrace__melee_fptrace` | E:/Projects/Melee Workspace/_build/cargo-shared/oracle/release/deps/melee_fptrace-87b9bad05a793adf.exe: error while loading shared libraries: std-f6a1efc0b26a278e.dll: cannot open shared object file: |
  | `melee_items__oracle` | melee_items\tests\oracle.rs:145:9:: assertion `left == right` failed: pika_scale: [7F800000, 7FC00000, 3F7FFFFF, 7FC00000, 7FC00000, 7FC00000, 00000000, 7FC00000, 3F800000, 00000001, 7F800000, 3F80000 |
  | `melee_link__hooks` | melee_link\tests\hooks.rs:29:5:: new unlinked todo hooks: ["ftCommon_CalcSelfAccel_Drift melee_modes::tu::geno_game::ext::hook::ftCommon_CalcSelfAccel_Drift", "ftCommon_Fall melee_modes::tu::geno_game |
  | `melee_modes__oracle` | melee_modes\tests\oracle.rs:161:5:: mismatches [("calcPan", (1500, 29)), ("toFeet", (1500, 1)), ("toMeters", (1500, 3))] |
  | `melee_modes__oracle_r10` | melee_modes\tests\oracle_r10.rs:184:5:: mismatches [("fn_80025E38", (600, 13))] |
  | `melee_modes__oracle_r4` | melee_modes\tests\oracle_r4.rs:251:5:: mismatches [("fn_800253D8", (250, 18)), ("fn_800256BC", (250, 17)), ("fn_800259EC", (250, 10)), ("fn_80025A98", (250, 11)), ("fn_80025B44", (250, 17)), ("fn_8002 |
  | `melee_modes__tu_gm_1A36` | melee_modes\tests\tu_gm_1A36.rs:137:17:: assertion `left == right` failed: function gm_801A3E88 case 0 |
  | `melee_modes__tu_gmgiant` | melee_modes\tests\mtu_harness\mod.rs:184:9:: assertion `left == right` failed: op 4 case 0 |
  | `melee_modes__tu_gmregclear` | melee_modes\src\oracle.rs:1386:9:: 852 mismatching cases |
  | `melee_modes__tu_gmregenddisp` | melee_modes\src\oracle.rs:1386:9:: 112 mismatching cases |
  | `melee_modes__tu_lb_020A` | melee_modes\tests\mtu_harness\mod.rs:184:9:: assertion `left == right` failed: op 3 case 0 |
  | `melee_modes__tu_lb_0219` | melee_modes\tests\mtu_harness\mod.rs:184:9:: assertion `left == right` failed: op 3 case 0 |
  | `melee_modes__tuo_gmmultiman` | melee_modes\tests\tuo_common\mod.rs:635:9:: 38 of 2060 cases differ (expected 2060 rows); per fn {"gm_801B6428": 38}; first: [ |
  | `melee_runtime__lbtu_lbtrigf` | melee_runtime\tests\lbtu_harness\mod.rs:213:9:: assertion `left == right` failed: op 2 case 81 |
  | `melee_runtime__lbtu_lbvector` | melee_runtime\tests\lbtu_harness\mod.rs:213:9:: assertion `left == right` failed: op 0 case 3 |
  | `melee_runtime__melee_runtime` | melee_runtime\src\cabi.rs:112:9:: assertion `left == right` failed |
  | `melee_runtime__oracle_bytecode` | melee_runtime\tests\oracle_bytecode.rs:65:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_cobj` | melee_runtime\tests\oracle_cobj.rs:267:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_fog` | melee_runtime\tests\oracle_fog.rs:205:9:: assertion `left == right` failed: line 67 op 9 |
  | `melee_runtime__oracle_generator` | melee_runtime\tests\oracle_generator.rs:469:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_hsd` | melee_runtime\tests\oracle_hsd.rs:168:5:: 5008 of 69000 checks differ {"MatToQuat": 333, "MtxGetRotation": 170, "MtxQuat": 1499, "MtxSRTQuat": 1498, "MtxSRTQuat ps": 1349, "QuatLib_8037EB28": 43, "Qua |
  | `melee_runtime__oracle_ik` | melee_runtime\tests\oracle_ik.rs:209:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_lobj` | melee_runtime\tests\oracle_lobj.rs:275:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_lobj_complete` | melee_runtime\src\tu\lobj.rs:1081:9:: TODO: HSD_AObjInterpretAnim is not ported |
  | `melee_runtime__oracle_mobj` | melee_runtime\tests\oracle_mobj.rs:106:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_robj` | melee_runtime\tests\oracle_robj.rs:290:5:: assertion `left == right` failed |
  | `melee_runtime__oracle_tobj` | melee_runtime\tests\oracle_tobj.rs:118:5:: assertion `left == right` failed |
  | `melee_runtime__pure_cobj_world` | melee_runtime\tests\pure_common\mod.rs:56:5:: 84 differ, first: [("op15 15 446226336d1ea13e", "op15 15 5a842b6981aa7a9f"), ("op15 28 bdba6dfac256a050", "op15 28 2bc37adfd6d7440b"), ("op15 41 09bde5fc6 |
  | `melee_runtime__pure_mtx_pc` | melee_runtime\tests\pure_common\mod.rs:56:5:: 385 differ, first: [("quat 0 0334f9d02e35ce30", "quat 0 f19882be3beabb1a"), ("quat 1 bcaf7ae5c11684a6", "quat 1 e81acc8013923011"), ("quat 3 0fea5c2a2091b |
  | `melee_runtime__pure_mtx_world` | melee_runtime\tests\pure_common\mod.rs:56:5:: 1 differ, first: [("op0 133 8654f6b141573b45", "op0 133 da610befe62a1d45")] |
  | `melee_runtime__tu_class` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_cobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_fog` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 1 case 0 |
  | `melee_runtime__tu_jobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_lbarq` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_lbcollision` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 102 |
  | `melee_runtime__tu_lbheap` | melee_runtime\tests\lbtest\mod.rs:58:13:: tu_lbheap: first difference at line 1 |
  | `melee_runtime__tu_lbrefract` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_mobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_object` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_pobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_state_native` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 4 case 2 |
  | `melee_runtime__tu_tev_native` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_tobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_runtime__tu_wobj` | melee_runtime\tests\mtu_harness\mod.rs:289:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__gro_grbattle` | melee_world\tests\gro_grbattle.rs:523:9:: 38 of 38 functions differ from the C: ["grBattle_GObj0_Callback0", "grBattle_GObj0_Callback1", "grBattle_GObj0_Callback2", "grBattle_GObj0_Callback3", "grBatt |
  | `melee_world__ground_vs_c` | melee_world\tests\ground_vs_c.rs:28:17:: assertion `left == right` failed: line 12: B 3f9bb374 3e0707e0 4026a154 | bd3bd1cd 3edfc786 00000000 |
  | `melee_world__leaves_r2_vs_c` | melee_world\tests\leaves_r2_vs_c.rs:107:5:: assertion failed: diff > 0 |
  | `melee_world__mplib_check_vs_c` | melee_world\tests\mplib_check_vs_c.rs:216:17:: assertion `left == right` failed: ref line 6277: R mpCheckFloor 1 bf4bd7ba 40f90aea 00000000 13 28613 bf61bee6 3ef173f1 00000000 0 10440 b8 100a8 102f4 1 |
  | `melee_world__mplib_geom_vs_c` | melee_world\tests\mplib_geom_vs_c.rs:146:17:: assertion `left == right` failed: ref line 662: E 14 bfbb851f 3f31a232 3f9e3d45 404b4f4e |
  | `melee_world__textlib_r6_vs_c` | melee_world\tests\textlib_r6_vs_c.rs:116:17:: assertion `left == right` failed: line 73925: M bea0cc20 43b769f2 7fa6a876 ffaf9962 | ffef9962 ffef9962 ffef9962 |
  | `melee_world__tu_cm` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 7 case 64 |
  | `melee_world__tu_grfigure1` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grfigure2` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grfigure3` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grfigureget` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grflatzone` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grgarden` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grgreatbay` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grgreens` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grheal` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grhomerun` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_gricemt` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grinishie1` | melee_world\tests\mtu_harness_cref\mod.rs:204:9:: assertion `left == right` failed: op 0 case 0 |
  | `melee_world__tu_grmutecity` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_groldkongo` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_groldpupupu` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_groldyoshi` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_gronett` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grpstadium` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grpura` | melee_world\tests\mtu_harness\mod.rs:251:13:: assertion `left == right` failed: op 1 case 0 |
  | `melee_world__tu_grt_stages` | melee_world\tests\tu_grt_stages.rs:2816:9:: assertion `left == right` failed: grtdrmario: line 0 differs (rerun with GRT_VERBOSE=1 for the Rust trace) |
  | `melee_world__tu_grt_stages_b` | melee_world\tests\tu_grt_stages_b.rs:2861:9:: assertion `left == right` failed: grtkirby: line 0 differs (rerun with GRT_VERBOSE=1 for the Rust trace) |
  | `melee_world__tu_grt_stages_c` | melee_world\tests\tu_grt_stages_c.rs:2898:9:: assertion `left == right` failed: grtpeach: line 0 differs (rerun with GRT_VERBOSE=1 for the Rust trace) |
  | `mex-import__ppc_oracle` | E:\Projects\Melee Workspace\worktrees\mrg2\tools\mex-import\tests\ppc_oracle.rs:157:5:: 4 of 6000 cases differ: |
  | `mex-import__remap` | E:\Projects\Melee Workspace\worktrees\mrg2\tools\mex-import\tests\remap.rs:79:13:: assertion `left == right` failed: Fighter +0x230b size 4 (round 0) |
