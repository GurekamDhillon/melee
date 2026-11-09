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
