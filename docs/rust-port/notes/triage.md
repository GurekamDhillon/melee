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
- Oracle suite (per-binary runner with oracle_tests.sh's flags, `-Zmerge-functions=disabled -Clink-arg=/OPT:NOICF`,
  shared target `_build/cargo-shared/oracle`, tests serial): NOT RUN on port/mrg2. The workspace `--tests` build was
  stopped by the harness for critically low system memory three minutes in (one rustc at 7.6 GB with 6 jobs beside
  other sessions); it is to be started again only on request, with `CARGO_BUILD_JOBS=2`. Reference before the
  types regen (port/regen1): 1037 targets, 5 no-compile, 583 pass / 449 fail; 142 fail with /OPT:NOICF
  (melee_fighter 39, melee_world 28, melee_runtime 24, melee_items 17, melee_chara 15, melee_modes 14, five
  singles). tu_shadow's op 17 (HSD_ViewingRectAddRect) trace agreement is therefore unverified too; its C oracle
  now links the real PSVECScale (mtx_pc.c copy in lbspecs_shadow.py) and the test data was regenerated.
