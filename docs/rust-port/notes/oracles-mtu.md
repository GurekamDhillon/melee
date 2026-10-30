# Whole-TU ports with state oracles (mtu shards) and efn

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 537-846 -->
## melee_modes::tu: whole-TU ports with a stateful C oracle (shard 465-472)
`rust/melee_modes/src/tu/<stem>.rs` ports every function of one gm `.c` file (gmcameramode,
gmclassic, gmdebugmode, gmevent, gmfixedcamera, gmgiant, gmgover, gmhanyucss: 147 functions,
all checked). Callees of unported TUs are `ext` hooks (`ext::NAME`, settable, unset panics
"TODO") named like the C; data owned by other TUs is a `pub static mut` in `ext`; the block is
generated from the C by `gen/cref/gen_tu.py` (libclang, the native flags of `oracle/build_r10.sh`).
- Oracle: `python rust/melee_modes/gen/cref/run_tu.py <stem>` compiles `gen/cref/tu/<stem>.c`,
  which `#include`s the game's `.c` verbatim (shipping contraction + gwtool lowering), defines a
  recording stub for every callee (generated: trace hash of call order, arguments, and return
  values from a shared xorshift), fills the synthetic game state (generated `fill_<T>` per record,
  bitfields by value, pointers linked by the driver), runs each function on 300-1000 seeded
  states, and writes one line per case (return, state hash, trace hash) to `tests/data/mtu_<stem>.txt`.
  `tests/tu_<stem>.rs` makes the same draws, installs the same recording hooks and compares.
  State hashes cover every field of the records (generated `hash_<T>`: scalars, bitfields by
  value, pointers as region ids, function pointers as table ids), so a wrong store anywhere shows.
- Debugging a mismatch: `rust/melee_modes/gen/cref/dbg.sh <stem> <test> <op:case>` prints the
  first differing trace / state words of one case; `ct.sh <test>` runs one test.
- Caveats found: records the `.c` defines are hand-written with `gc_layout!` (melee_types does not
  generate them); an anonymous union / struct member is compared as raw bytes (bitfield overlays
  such as `MatchPlayerData.x3` are not compared by value: melee_types keeps bitfields in GameCube
  bit order, native C does not); `gmclassic.c` reaches `gm_804908A0` through
  `gmClassicIntroDataBuffer` (adjacent on the console), so the oracle patches the two declarations
  into one object (`specs.py`); `onEnterVs` with `player_init[4]->c_kind == ChKind_None` writes the
  character into the bitfield byte `EventData+0xB` (never in the real level data; the oracle avoids
  it); `gm_8016A404((s32) fnptr)` passes a function address as a 32-bit int (PPC-ism: a native
  callee cannot turn it back into a function; the trace names the function by its low 32 bits).

## melee_world::tu: whole-TU gr ports with a stateful C oracle (shard 553-560)
`rust/melee_world/src/tu/{grdisplay,grdynamicattr,grfigure1,grfigure2,grfigure3,grfigureget,grflatzone,grfourside}.rs`
port every function of those `src/melee/gr` files (187 incl. the static-inline helpers; 44 constant-return
callbacks are re-exports of `gr::trivial`). Callees of unported TUs are `ext` hooks (unset: panics "TODO"), the
block is generated from the C by `gen/mtu/gen_tu.py` (a copy of melee_modes' `gen/cref` oracle, retargeted).
- Oracle: `bash rust/melee_world/gen/mtu/regen_gr.sh` (or `python gen/mtu/run_tu.py <stem>`) compiles the game's
  `.c` verbatim (shipping contraction + gwtool lowering), with a recording stub per callee, runs every function on
  500-2500 seeded synthetic states per op and writes `tests/data/mtu_<stem>.txt` (return, field-by-field state hash,
  call-trace hash); `tests/tu_<stem>.rs` makes the same draws. The stage drivers are written from one description by
  `gen/mtu/mk_stage_driver.py` + `mk_stage_cfgs.py` (C and Rust halves cannot drift); grdisplay / grdynamicattr
  have hand-written drivers (`gen/mtu/tu/<stem>.c`, `tests/tu_<stem>.rs`).
- Traps found: (1) melee_types keeps `uintptr_t`/`HSD_IDKey` at 4 bytes (GC) but native C has 8: the state walker
  compares such fields as 32 bits; (2) a top-level anonymous union member must be addressed as the record, not as a
  reference to it (generator bug fixed); (3) `x = (u64) fv() << 32 | fv()` has no defined evaluation order: use
  `fv64()`; (4) unions that hold pointers must be hashed by region id (`PIB`), never as raw bytes (addresses differ
  between the two executables), and a pointer slot another view overwrites byte-wise only counts at an exact base;
  (5) bitfield bytes differ between native (LSB-first) and GC order (MSB-first): compared by value (`foursideCrane.x1`);
  (6) div/rem by zero is not rewritten by `lower.py` (the native C traps), so fixtures keep divisors non-zero; the
  port uses `cabi::srem` for the gwtool result (`x % 0 = x`), which the oracle therefore does not exercise.
- GC union aliasing: `grflatzone.c` names one union word through `flatzone.xD0`/`unk.xD0`/`flatzone2.xD0` and
  `flatzone.xD4`/`flatzone2.timer`; with 8-byte native pointers (`flatzone2.xCC`) these stop aliasing. The port keeps
  the GameCube aliasing (uses `flatzone2.xD0` / `.timer` everywhere) and the oracle compiles a copy of the TU with
  the same substitution (`patch` in `gen/mtu/specs.py`). Mutation checks: flipping a fused multiply-add site, a
  boundary constant or a state-machine constant in grdisplay / grflatzone / grfourside fails the test.
- Not compared: `OSReport` diagnostics; `atan2f` (lbtrigf.c) is an `ext` hook in grfourside (ported and checked in
  melee_runtime); the crane/stage init chains of `OnInit` / `setup_gobj` ops compare the callback trace, not the
  union bytes the chain overwrites (each init function has its own exact op).

## melee_world::tu: gr/grt*.c set B (shard 601-608)
`grtkirby grtkoopa grtlink grtluigi grtmario grtmars grtmewtwo grtness` ported by the same
generator as set A (`GRT_SET=b bash rust/melee_world/cref/gen_grt_stages_ref.sh`; test
`tests/tu_grt_stages_b.rs`, 60 seeded cases per function). New fixtures: Generator/AppSRT pair
(mario particle scale), 8-slot yakumono struct (mewtwo), `ypick` biased ftLib positions so the
mario floor-crossing branch is exercised.

## melee_world::tu: gr/grt*.c set C (shard 609-616)
`grtpeach grtpichu grtpikachu grtpurin grtsamus grtseak grtyoshi grtzelda`, by the set-A/B generator
(`GRT_SET=c bash rust/melee_world/cref/gen_grt_stages_ref.sh`; test `tests/tu_grt_stages_c.rs`, 60
seeded cases per function, all 20 functions of each TU incl. the re-exported constant ones). New:
`grtseak.c`'s TARGET_PC creator (`Mex_GrCallbacks` / `Mex_GrTrace`, fixture = a fake m-ex
`StageCallbacks[4]` whose entries record their call; the "grtrace" OSReports carry `%p` args and are
the separate ext `OSReport_grtrace`, traced by format only), `grtpurin.c`'s loop over the
joint table (`lb_8000B1CC` recorder now returns x in 37.5..162.5 so both sides of `x < 130` run) and
its `yakumono_param` reader; the TU-local `YakumonoParam` carries a `gc_layout!` GameCube table.

## melee_items::tu: it/kinds item-kind ports with a C state oracle (shard 657-664)
`rust/melee_items/src/tu/{itarwinglaser,itbat,itbombhei,itbox,itcapsule,itcerebi,itchicorita,itclimbersblizzard}.rs` port
every function of those `src/melee/it/kinds` files (234, plus the static-inline helpers from `itbombhei.static.h`,
`it/inlines.h` and `jobj.h`, which live in the file or in `tu/kinds_inl.rs`) with the C names and signatures; callees of
unported TUs are `ext` hooks (unset: panics "TODO"). The old snake_case forwarders in `itfwd*.rs` / `itkinds*.rs` take
`&mut Item` and are not re-exported here (different signatures).
- Oracle: `bash rust/melee_items/gen/kinds/regen.sh [stem...]` (a copy of melee_world's `gen/mtu`, retargeted; own file
  names so it does not collide with the other melee_items TU oracles): `mk_driver.py` writes `gen/kinds/tu/<stem>.c` and
  `tests/kinds_<stem>.rs` from one description (`specs_cfg.py`: typed `x4_specialAttributes` buffer, fixture tweaks,
  argument hints, extra ops), `run_tu.py` compiles the game's `.c` verbatim (shipping contraction + gwtool lowering)
  and writes `tests/data/kinds_<stem>.txt`. The world is 3 gobjs / items / articles / attributes, 48 jobjs (14-bone
  dynamic bone tables), the common item data and 4 Vec3, redrawn field by field every case (400 per function);
  every callee is a recording stub (call order, arguments, `Vec3*` / `SpawnItem*` pointees by value, random returns,
  `lb_8000B1CC`-style out-parameters written from the shared stream). After each call the whole world, the return
  value, the trace and (once) the `ItemStateTable` arrays are hashed.
- Walker changes vs melee_world: a union whose views all cover the same storage and one of which holds bitfields
  (`UnkFlagStruct`, `flag32`) is compared by bitfield value (native C is LSB-first, melee_types MSB-first); a union
  of differently sized variants (`Item_ItemVars`) stays raw bytes, so a TU's pointer slots / bitfield bytes are hashed
  by value and cleared in the per-TU `ptrs` snippet; stubs are defined as `(name)(...)` so macro-named callees
  (`HSD_JObjSetMtxDirty`) work; only file-scope variables count as extern data; fill / hash bodies are `unsafe`.
- Fused sites (checked in the IR): `itChicorita` leaf `prev_pos.x += x8 * facing_dir` and Blizzard
  `(x10 - xC) * rand + xC` are `fmadds`; `itBox_UnkMotion8_Phys` has two f64 sites (`mul_add`, indistinguishable after the f32
  store except in rare double-rounding cases, so a mutation of them is not caught). `sinf` / `cosf` are the real
  `melee_runtime::gekko` ports (gekko_fp.c is linked into the oracle); `atan2f` and the variadic `efSync_Spawn` are
  `ext` hooks (fixed-arity typed hook per call shape).
- `it_802E7654` has no `return` in the C; the port returns the spawned gobj (what PPC leaves in r3), the oracle does
  not compare that return value. Ints that the C wraps (`-= 1`, sums of weights) use `wrapping_*` ops. Mutation checks
  (flipping fused sites, constants, comparison boundaries, state ids) fail the tests.

## melee_items::tu: it/kinds itpeach*, itpikachu*, itpippi (shard 769-776)
`rust/melee_items/src/tu/{itpeachparasol,itpeachtoad,itpeachtoadspore,itpeachturnip,itpikachuthunder,itpikachutjoltair,itpikachutjoltground,itpippi}.rs`
port every function (131 incl. statics) with the shard 657-664 oracle (`bash rust/melee_items/gen/kinds/regen.sh <stem>`,
`cargo test -p melee_items --test kinds_<stem>`, 400 seeded states per op). Fused sites: toadspore `speed` and `angle`
(`fmadds`). Notes: the turnip odds table is read by raw offset (the C reads one entry past `x8[length-1]`);
`peachturnip.xDD4.b0` is hashed by value (native bitfield order differs); `it_802BDA40` with state row -1 would index
`it_804D5518[-1]` (UB, unknown sdata neighbour): the port returns 0 and the oracle only asks rows 1 and 2; `bool` is
MSL `int`, so the ports return Rust `bool` and the oracle's stubs for int-returning callees used as booleans return 0/1.

## melee_items::tu: it/kinds set (shard 673-680)
`rust/melee_items/src/tu/{itegg,itentei,itevyoshiegg,itfflower,itfflowerflame,itfire,itflipper,itfoods}.rs`
port every function of those `src/melee/it/kinds` files (220 incl. the static-inline helpers) plus their
`ItemStateTable` data. Callees of other TUs are `ext` hooks (unset: panics "TODO").
- Oracle: `bash rust/melee_items/gen/mtu/go.sh <stem>` (`gen_it.py` = melee_world's `gen_tu.py` retargeted, plus
  `UnkFlagStruct` filled / hashed bit by bit by value; `mk_it_driver.py` + `cfg_it.py` write the C driver and the
  Rust test from one description; `run_it.py` compiles the game's `.c` verbatim with shipping contraction + gwtool
  lowering). One fixture item (gobj, Item, Article + 1 KiB attribute buffer, ItemAttr, 4 joints, bone table, second
  item for spawns, owner, ICD `it_804D6D28`), 150 seeded states per function, state hashed field by field, every
  call traced. Spawn records are hashed inside the `Item_80268B18` stub. Mutation checks: fma sites (foods
  `fmadds`, flipper `sqrtf(vx*vx+vy*vy)`), a boundary compare and a swapped spawn parent each fail the test.
- Variadic `efSync_Spawn`: the compiled copy of the TU counts the arguments (`patch` in specs_it.py), the port
  calls `ext::efSync_Spawn_N(gfx, gobj, &[args])`; pointees of the stack args (`&val`, `&v`) are not compared.
- Traps: (1) `UnkFlagStruct` raw bytes differ between native (LSB-first) and GC bit order; compare by value.
  (2) the flipper's `xDEC_stageAnchor` pointer lives in the item-var union: normalised to its region id before the
  hash. (3) `itFoodsAttributes[]` is indexed with the native 24-byte stride (GC: 16); the port mirrors the native
  struct like the rest of melee_types. (4) `ABS(x)` is `x < 0 ? -x : x`; `-1.0f * y + c` is a (non-discriminating)
  fma site, kept as `fmadds`.

## melee_items::tu: it TUs with a stateful C oracle (shard 649-656)
`rust/melee_items/src/tu/{itspawn,itmaterial,itzako,it_26B1,it_2725,it_279C}.rs` (271 functions incl. 18 static inlines, all
present; it_3F14 / it_3F2F hold only data tables and have no functions; their tables are not ported). Oracle (generator copy kept in gen/mtu649, since gen/mtu was taken by another shard's variant; the generated gen/mtu/tu files stay where they are): `python
rust/melee_items/gen/mtu649/{gen_tu,mk_driver,run_tu}.py <stem>` (same design as melee_world's gen/mtu: the game `.c` verbatim,
shipping contraction, recording stubs, seeded synthetic state, field-by-field hash) -> `tests/data/mtu_<stem>.txt`,
`tests/tu_<stem>.rs`. Per-TU fixtures live in `gen/mtu/stems.py` / `specs.py`; the shared world is `tests/mtu_w/world.rs`.
New generator features: `xpools` (stem-private record pools), `gen_extra`, `paren_stubs`, `skip_freg`, `post_all`, `noret`.
- Bugs the oracle found in the first-pass ports: invented RENDER_* constants (itmaterial); missed fused multiply-adds
  (itzako product_xyz/sqrt structure, it_26B1 it_8026B344); `M_PI_2` is a double (it_2725); `1 << n` with n >= 32 needs
  `cabi::shl`; C `bool` is int (parameters and pass-through returns are i32); dropped TARGET_PC paths (Mex_ItRecord);
  argument evaluation order (itspawn: HSD_Randf before gm_8016AE80); calls into sibling TUs must go through `ext`;
  header inlines must be local, not `ext`; 27 functions the first pass left to itlib were missing from the TU files.
- Oracle traps: unions with pointers or bitfield views (xDD4_itemVar, xD90) alias differently native vs GC bit order, so
  fixtures normalise them (`post_all`); `HM_LINE` (MTU_DBG builds) prints the gen.h line of each hashed word.
- Not measured: branch coverage (llvm-cov profile versions do not match rustc's); `chars_ref` (melee_chara) does not
  terminate in this checkout, unrelated.
## melee_items::tu: it/kinds set (shard 681-688)
`itfoxblaster itfoxillusion itfoxlaser itfreeze itfreezer itfushigibana itgamewatchbreath itgamewatchchef`
(`rust/melee_items/src/tu/`, 163 functions incl. the static-inline helpers and the TU's `ItemStateTable` /
float / int data tables; header inlines the TUs use are copied into each file). Callees of unported TUs are
`ext` hooks (unset: panic "TODO"); `cosf` / `sinf` are `melee_runtime::gekko` (the oracle links the real
`gekko_fp.c`), `atan2f` is an ext hook.
- Oracle: `python rust/melee_world/gen/mtu/mk_itkinds.py` writes, from the one description in
  `gen/mtu/specs_it.py` (`CFG`), the C driver `gen/mtu/tu/<stem>.c` and the Rust test
  `rust/melee_items/tests/tu_<stem>.rs`; `python rust/melee_world/gen/mtu/run_tu.py <stem>` builds the C
  (shipping contraction + gwtool lowering) and writes `tests/data/mtu_<stem>.txt`. Each case (300 per
  function, seeded) builds a synthetic world (item gobj + Item, owner / reference / parent / spawn-result
  gobjs and Items, a JObj chain, article / attribute / dynamic-bone records, the kind's item-var union
  member) and hashes every field of every record after the call (generated `fill_` / `hash_` walkers,
  `skip_fields` + by-value bitfield views added to `gen_tu.py`), the return value, and the call trace of
  every stub (the `SpawnItem` a spawn stub receives is hashed at the call).
- Traps found: (1) `-1.0f * x` is folded to a negation by LLVM, which flips a NaN's sign; the console
  `fmuls` keeps it (`itfreeze.rs` `mul_m1`); (2) `attrs->entries[i]` on a one-element array type: index
  through a raw pointer; (3) a float local read before init (`it_8028EB88` `ip`) is 0 in the game build;
  (4) `HSD_ASSERT(jobj)` in the jobj.h setters hangs on NULL: the oracle never passes NULL, the port asserts;
  (5) `it_8029CD78` loops forever when the ghost jobj exists (kept as `loop {}`).
- Mutation checks (flip a fused site, a table value, a state constant, a flag, an index) are caught in all
  eight TUs; equivalent mutants only at float-equality boundaries.

## melee_items::tu: it/kinds set (shard 705-712)
`ithinoarashi ithitodeman ithouou itkabigon itkamex itkinoko itkirbycutterbeam itkirbygamewatchchefpan`
(`rust/melee_items/src/tu/`, 133 functions incl. the static-inline helpers and the 8 state tables; the older leaf-style
ports in itkinds*.rs / itfwd*.rs keep their own signatures). State oracle: `python rust/melee_world/gen/mtu/mk_itk.py all <stem>`
generates the C driver (the game `.c` included verbatim, shipping contraction + gwtool lowering, `gekko_fp.c`, `lb/lbtrigf.c`,
`lb/lbvector.c` linked for sinf/cosf/atan2f/lbVector_*), the recording stubs for every other callee, and the Rust test
`tests/tu_<stem>.rs`; 400 seeded cases per function, each hashing every field of Item + the spawned Item + JObjs + attribute
records. Bitfield unions (UnkFlagStruct, flag32) are hashed by value; the item-var union is walked as the TU's variant;
hinoarashi's Vec3-over-flags pun (`it_802D61C8` / `it_802D6310`) is compared as raw bytes. `efSync_Spawn` (variadic) is an
ext taking `&[*const c_void]`, its extras traced by gfx id. Mutation checks (flipping fused sites, boundaries, constants) fail
the tests; `-1.0 * x + z` fused vs unfused is bit-equal, so that site is not distinguishable.
## melee_items::tu: it/kinds set 721-728 (itkyasarinegg itleadead itlgun itlgunbeam itlgunray itlikelike itlinkarrow itlinkbomb)
`rust/melee_items/src/tu/<stem>.rs` ports all 344 functions of those `src/melee/it/kinds` files (static inlines included; header inlines are
local `#[inline]` copies) with the C names and signatures, checked by the shard 657-664 kinds oracle (`bash rust/melee_items/gen/kinds/regen.sh
<stem>`, tests `kinds_<stem>.rs`, 400 seeded cases per function, whole world hashed after every call, call trace hashed).
- Raw-int `bool`s: `bool` is the MSL `int`, so a callback that `return`s a callee's int (`return it_8027C794(gobj);`) must return that
  value unnormalised: those ports return `i32` (state-table slot `Some(f)` instead of `pred_b(f)`), found by the oracle (random stub returns).
- Fused sites (checked in the IR, every one caught by an unfused mutation unless noted): `itlgunbeam` `diff * r + lo` x2; `itlinkarrow`
  `arg4 * ((x8-x4)/arg5) + x4` x2, the `lookup[8] * rand + lookup[0]` terms (cases 0/2/4/6 and 1/3/5, both in `802A81C4` and `Motion4_Anim`),
  `xD4 * cosf/sinf + xC8/xCC` (`Motion2_Phys`, `Logic98_HitShield`); `itlinkbomb` `Motion4_Phys`; `itleadead` `neg_dot` is
  `fmuladd(-ax, bx, -(ay*by))` then `fmuladd(-az, bz, .)` (clang folds the negated products; IR-checked, the branch outcome it feeds is
  not sensitive to the rounding in the random cases, so no mutation of it is caught). Use `melee_core::fp::fmadds` (f64 multiply-add), never `mul_add`.
- Oracle additions: `all3()` in `specs_cfg.py` fills the special-attribute block of all three items (spawned items read `ART[2]`);
  `deep_ecb` spec flag traces the pointee of `itECB*` stub arguments (`TECB` / `tecb`; off for the older TUs so their data files stay valid);
  bitfield words inside an item-var union are zeroed before their bits are set and hashed by value (native C is LSB-first, melee_types MSB-first).
- No `return` in the C (UB, PPC leaves r3): `itKyasarinegg_UnkMotion4_Anim` (returns the `it_802751D8` int), `itLinkArrow_802A81C4`
  (returns the new `x9C`) and the fall-off path of `itLinkArrow_802A850C` (returns 0); the oracle skips those return values (state, trace
  and the defined path of 802A850C are compared). Boundary constants that random stub returns never straddle (for example `dx <= x10`,
  `ad > 3.0` in leadead, `var_f1 < 15.0` in likelike) are matched by reading, not by mutation.
- 274 callee hooks across the eight files are `ext` TODO stubs (unset: panic "TODO: name is not ported").
## melee_items::tu: it/kinds itluigifireball, itmaril, itmariocape, itmariofireball, itmarumine, itmasterhandbullet, itmasterhandlaser, itmatadogas (shard 737-744)
Same layout and oracle (`bash rust/melee_items/gen/kinds/regen.sh <stem>...`) as the 657-664 shard: every function of the
eight files (150 incl. statics, e.g. `calc_dist_2d_accurate`, `sqrtf_store`) in `rust/melee_items/src/tu/<stem>.rs`, 400 seeded cases each.
- Driver additions: `double` returns are hashed (`mk_driver.py`); variadic `efAsync_Spawn` / `efSync_Spawn` are fixed-arity custom stubs;
  `itmasterhandlaser` adds a fake `Fighter` (`parts` -> `FighterBone[8]`) as GO[1]'s user data, a 2-entry `HSD_Joint` chain for `xC8_joint`,
  and `mpCheckMultiple` writes its `Vec3` out-parameter; bool-returning stubs of the bullet return 0/1.
- Fused sites (IR-checked): f32 `fmadds` in `calc_dist_2d_accurate` (luigi / mario fireball) and 4 in `it_802F063C`; f64 Newton steps of
  `sqrtf_accurate` / `sqrtf_store` (`3.0 - g*g*x` as fma; un-fusing them is not caught, the f32 rounding hides it). Mutations of the f32 sites,
  the `M_PI` negation in `itmaril` and the `MTXDegToRad` constant in `itmatadogas` fail the tests.
## melee_items::tu: it/kinds nokonoko ... peachexplode (shard 761-768)
`rust/melee_items/src/tu/{itnokonoko,itoctarock,itoctarockstone,itoldkuri,itoldottosea,itparasol,itpatapata,itpeachexplode}.rs`
port every function of those `src/melee/it/kinds` files (257, statics and the two `*_inline` helpers of itnokonoko included),
checked with the same whole-TU C state oracle as shard 657-664 (`bash rust/melee_items/gen/kinds/regen.sh [stem...]`,
400 seeded cases per function, tests `kinds_<stem>`). New specs live in `specs.py` (`MY_RET` / `MY_EFF`: pass-through `bool`
callees return 0/1, `it_80276308` returns 0/4/8, `ftLib_8008627C` returns an owner gobj or NULL, `Vec3` out-parameters are
written) and `specs_cfg.py` (typed `x4_specialAttributes` with a pointee block at +64, the `it_804D6D40` data, the
ptr slot of `oldottosea.x20` cleared after hashing, bones 14..18 of the dynamic bone table for patapata).
- Fused sites (checked in the IR): parasol `deg_to_rad * fm + ry`, octarockstone `Randf * ((pos.y - y) / 10) + vel.y`,
  oldottosea `pos.x += x14 * facing` and `vel.x += -facing * xC`; octarock / oldkuri / patapata / nokonoko / peachexplode have none.
  Mutations of each fused site, constants and state ids fail the tests.
- `x40_vel.y <= 0.2` (oldkuri, nokonoko) is a double comparison in the C (`0.2` unsuffixed); the port compares as f64.
- `itParasol it_8028B08C(item, statenum)` indexes a 8-entry table by `anim_id`, which is -1 for states 0 and 3 (C reads out
  of bounds); the oracle only draws the states 1, 2, 4..10 that the game uses. `bool` parameters of the C are `int`: ports take `i32`.
- Not in these files (itwhitebea.c): `itOldottosea_UnkMotion8..11`, `it_802E3400/3528/35CC/37A4`; the state table names the former
  through `ext::cb`.
### Shard 745-752: it/kinds itmato, itmball, itmetalb, itmetamon, itmew, itmewtwodisable, itmewtwoshadowball, itmsbomb
`rust/melee_items/src/tu/{itmato,itmball,itmetalb,itmetamon,itmew,itmewtwodisable,itmewtwoshadowball,itmsbomb}.rs` port every
function of those `src/melee/it/kinds` files (171, incl. the static inlines) with the same `gen/kinds` state oracle (168 ops x
400 cases, hashed world + return + trace; the state tables too). The TARGET_PC `ScriptGame_*` branches of itmato are ported
(`ext` hooks). Notes: `Item_ItemVars` bitfield slots (mball b0/b1, msbomb xDD4_b0) are native-LSB-first in C vs MSB-first in
melee_types, so the cfg tweaks draw them by value and the hash clears the slot; variadic `efAsync_Spawn` / `efSync_Spawn` are
fixed-arity custom stubs; `it_802C4D10` has no `return` in the C (port returns 0, oracle skips it). Fused sites (checked with
`fma_sites.py`): msbomb `pos.y += scl*(top-bottom)`; shadowball `x30.y/z += (x40*x50)*sin/cos`, the five `charge*((hi-lo)/max)+lo`
lerps and `0.5*(x18/x1C)+0.5` (`fmadds`), and `x3C += (M_PI/4)*(2*(rand-0.5))` (f64 `fma`). Mutation checks (unfusing the
msbomb and sinf sites, a constant, a comparison boundary) fail the tests; the f64 `x3C` fma and the exact `0.5*q+0.5` lerp are
indistinguishable after the f32 store, as with itbox's f64 sites. Regenerate: `bash rust/melee_items/gen/kinds/regen.sh itmato ...`.

## melee_items::tu: it/kinds set 2 (shard 777-784)
`itporygon2 itrabbitc itraikou itrshell itsamusbomb itsamuschargeshot itsamusgrapple itsamusmissile` ported with the
shard 657-664 generator (`bash rust/melee_items/gen/kinds/regen.sh [stem...]`; tests `tests/kinds_<stem>.rs`, 400
seeded cases per function, every function of each TU incl. the state tables).
- Traps: (1) `bool` parameters are C `int` (`typedef int bool`), so extern fns take `i32`; (2) `typedef`-less local
  records (`itRShell_Attrs`) are plain `#[repr(C)]` in the port; (3) raw-byte comparison of unions that hold a pointer
  or a bitfield differs between the two executables (address bytes, LSB- vs MSB-first): `itrshell` clears the item
  var union / flag byte after hashing the fields by value, `itsamusgrapple` does the same for `Fighter.u` and
  `HitCapsule.owner`/`hit_grabbed_victim_only`; (4) `itSamusChargeshot` item vars `xDEC/xDF0/xDF8/xDFC` are ints, so
  float values are converted when stored; (5) `ftSs_DatAttrs.xA0/xB0/xC0` are ints compared to a float.
- `itsamusgrapple.c` reads two packed bitfields of the hitbox command by byte offset (`*(u16*)(data+2) & 0x3FF`,
  `(flags+3)->hit_grounded`). On the GameCube (MSB-first) those are the `damage` and `hit_grounded` fields; the port
  reads them by name and the oracle compiles a copy of the TU with the same two substitutions (`patch` in
  `gen/kinds/specs.py`). The world is custom (`gen/kinds/specs_cfg_grapple.py`): a 2-6 link chain wired into
  `ItemLink` pool entries, an owner fighter with `ftSs_DatAttrs`, a bone table, the beam attributes and a hitbox
  record; the pool stubs (`GObj_Create`, `HSD_ObjAlloc`, `HSD_JObjLoadJoint`) hand out pool entries.
- Fusion: each contracted `a*b + c` is `fmadds` (f32) or `mul_add` (f64); mutating the sites in missile/bomb/
  chargeshot/rshell/grapple fails the tests except where the product is exact (`1.0f * x`).
## melee_items::tu: it/kinds set 2 (shard 785-792)
`rust/melee_items/src/tu/{itscball,itseakchain,itseakneedleheld,itseakneedlethrown,itseakvanish,itsonans,itspycloak,itsscope}.rs`
port every function of those `src/melee/it/kinds` files (static inlines included) with the C names and signatures, using the
shard 657-664 oracle (`bash rust/melee_items/gen/kinds/regen.sh itscball ...`, tests `tests/kinds_<stem>.rs`, 400 seeded cases
per function). New fixtures: a 16-link `ItemLink` pool + `Fighter` behind `GO[2]` for itseakchain (`ptrargs` in `mk_driver.py`
maps `ItemLink*` / attribute-buffer parameters; pool allocators `GObj_Create` / `HSD_ObjAlloc` / `HSD_JObjLoadJoint` are stubs
returning pool entries; `PSMTX*` stubs have matrix effects). Notes: `ABS(table[HSD_Randi(8)])` evaluates its argument twice
(two draws); the `it_802BAF2C` result is `(int) pointer` and is not compared; float-to-int of `xC[]` in itsscope follows
gwtool's saturating cast. A mutation of the fused comparison `v < -x18 * s - x28` in `it_802BC080` is not caught.
## melee_items::tu: it/kinds set B (shard 793-800)
`rust/melee_items/src/tu/{itsscopebeam,itstar,itstarrod,itstarrodstar,itsuikun,itsword,ittaru,ittarucann}.rs` port every
function of those `src/melee/it/kinds` files (the same pipeline and `ext` convention as shard 657-664; `gen/kinds/splice.py <stem>`
pastes the generated `ext` block into a port). State oracle per file (`bash rust/melee_items/gen/kinds/regen.sh <stem>`, then
`cargo test -p melee_items --test kinds_<stem>`), 400 cases per function plus the state tables.
- MSL `bool` is `int`: a C function that `return`s a callee's int gives the raw value (e.g. 12), the port a Rust `bool`; mk_driver
  now compares `(call) != 0` for functions declared `bool`.
- `ittarucann` / `ittaru` / `itsscopebeam` fused sites (checked by mutation): `+=` of a product, `a*b + c` (`fmadds`); the product stored
  to `temp` first in the `*_Spawned` functions is NOT fused. `8.0f * cosf/sinf + x` and `2.0f * angle + y` are exact either way.
  `itTarucann_UnkMotion7_Phys` returns a `float` (state-table slot is cast); `itTarucann` stubs `sinf`/`cosf` are the real gekko ports.
- Oracle additions: `facing_dir` also drawn as an arbitrary float; a fake `ItemModelDesc`/`HSD_Joint` (`MD`, `JNT`) for the
  `x10_modelDesc->x0_joint` reads; `ftCo_800C92E4`'s `lbColl_80008D30_arg1` pointee is traced.
## melee_items::tu: it/kinds set 2 (shard 809-816)
`itwhitebea itwstar ityaku ityoshiegglay ityoshieggthrow ityoshistar ityoshitongue itzeldadinfire` (165 fns, all with a
state-oracle op each) ported with the shard 657-664 generator (`gen/kinds`; descriptions in `cfg_809.py`, specs in `specs.py`).
New in the oracle: `efAsync_Spawn` modelled as a 6-argument hook (the variadic scale is read only for spawn kind 4);
`__frsqrte`/`sinf`/`cosf` are the real linked ports; callback fixtures (ityaku) are registered functions.
Fused sites (IR): yoshistar `vel.x += accel*dir`, yoshiegglay timer `-= xCA0*x0`, zeldadinfire scale and steering,
whitebea distance. The test catches the yoshistar mutation; the whitebea distance (only compared against thresholds)
and zelda steering (rarely reached) are not discriminated by the current inputs.
`x28_entries` (itwstar) and the yaku `while (true)` hang are handled as noted in the file headers.
## Shard 881-888: pl/player.c plattack.c plbonus.c plbonuslib.c, mn/mn_22EC.c, mp/mpisland.c mplib.c mpcoll.c
No oracle was run for this shard (no C oracle, no cargo test): the ports are checked by construction.
- `melee_fighter::tu::{player,plattack,plbonus,plbonuslib}` and `melee_modes::tu::mn_22EC` are hand ports with the C
  names and signatures (`bool` is MSL `int`, so `bool` parameters that the C compares with `== 1` are `i32`). `player_slots` /
  `ftMapping_list` / `unk_804D6480` are the existing statics (`crate::stale`, `crate::player`) re-exported under the C
  names. `pl_800386D8` / `pl_8003E70C` / `pl_8003E7D4` index past the member they name (into `x504..`, `x848[]`), so
  they use byte-offset pointer reads. Fused sites (`fma_sites.py`): plbonuslib `pl_8003ED0C` (1), `fn_8003F294` (1),
  `fn_8003F654` (3), `pl_8003FAA8` (1); everything else in these files has none.
- `melee_world::tu::{mplib,mpcoll,mpisland}` are generated: `python rust/melee_world/gen/mtu/c2rs.py` translates the
  functions of one C file with libclang (the AST carries every implicit conversion, so the integer / float / pointer
  semantics are the C ones; ints use plain ops because the workspace builds with overflow-checks off (-fwrapv),
  `/ % <<` with variable operands go through `melee_runtime::cabi`, float->int uses the saturating `as`). Contraction
  follows clang's `-ffp-contract=on` rule (a `+`/`-` whose left, else right, operand is a same-type multiply, or the
  negation of one, becomes `fmadds` / `mul_add`); `--check` compares the per-function count with `tools/rust-port/fma_sites.py`
  (all translated functions agree; the hand ports and the `sqrtf` alias are covered by hand), and spot checks against
  the LLVM IR (`-Xclang -disable-llvm-passes`) agreed on operand order and negation. `bash rust/melee_world/gen/mtu/regen_mp.sh`
  regenerates the three files (heads in `gen/mtu/mp/`, goto state machines the tool cannot structure in `gen/mtu/hand/`:
  `mpIsland_8005AE1C`, `mpLib_80053DA4_Floor`, `mpLib_80053ECC_Floor`, `mpLib_80056C54`). Forward / backward / loop-entry
  gotos (`block_8`, `again`, `after0`...) are turned into labeled blocks / loops by the tool.
- Traps: (1) `(CollLine*) ((int) groundCollLine + line_offset)` is a PPC-ism (pointer through an `int`); it is a byte offset
  from the base pointer here, and `line - groundCollLine` is `offset_from`. (2) `mpLib_80056C54` passes `sqrt_tmp - 4` as the
  volatile store slot of `sqrtf_store` (stack layout hack): a scratch float here. (3) `HSD_ASSERT` / `__assert` panic;
  `OSReport` diagnostics go to `ext::OSReport(&str)`. (4) The TARGET_PC script hooks (`ScriptGame_Stage*`) are ext hooks
  like every other callee outside the three files (unset: panic "TODO"); `ScriptGame_ModValue` (player.c) falls back to
  the crate's neutral `SCRIPT_MOD_VALUE` hook. (5) `lbVector_Add/Diff`, `PSMTXMultVec`, `sinf`, `cosf`, `sqrtf` and
  `PSVECNormalize` call the existing `melee_runtime` ports.


<!-- RUST_PORT.md lines 910-919 -->
## melee_items::tu: it/kinds set (shard 665-672)
`itclimbersice itclimbersstring itclinkmilk itcoin itcrazyhandbomb itdkinoko itdosei itdrmariopill`: every function
(incl. static inlines) checked by the mtu state oracle (`python rust/melee_items/gen/mtu/run_tu.py <stem>`, driver from
`mk_driver.py`, per-TU fixtures in `gen/mtu/stems.py`, stub effects in `specs.py`). Header inlines (HSD_JObj setters,
spawn helpers) are local ports via the `jobj_inl!`/`spawn_inl!` macros in `tu/mod.rs`; every out-of-TU callee goes through
`ext`. Traps: C `bool` is `int` (pass-through returns are `i32`), `ext` callbacks need `ext::cb::*` (C ABI), itemVar unions
holding pointers or aliasing bitfield members are hashed by field and cleared after each op, `HSD_ASSERT` is an infinite
loop in the C driver so fixtures clear JOBJ_USE_QUATERNION. The double-precision Newton steps of `my_sqrtf` cannot be
told apart from unfused ones through the f32 result.


<!-- RUST_PORT.md lines 1245-1259 -->
## melee_world::tu::efn: ef/eflib.c, efasync.c, efdata.c via c2rs + state oracle (pre-melee_world)
- `bash rust/melee_world/gen/mtu/regen_ef.sh` regenerates `src/tu/efn/{eflib,efasync,efdata}.rs` (c2rs `--all-extern`: every
  function is `extern "C"` so a test can store any of them as a callback; then `gen/rewire/rewire.py --skip=lbArchive_80017040`).
  c2rs learned: variadic definitions (`args: &[*const c_void]`, `crate::tu::va::Va` for `va_list`; `va_arg` steps it like the
  `__va_arg` shim), goto-into-a-later-case-label inside a switch (flag + block after the match), `&arr[i]` as a wrapping
  pointer offset (C forms out-of-range addresses before checking them), a NULL `%s` in OSReport, pointer args of `%08X`.
- Oracle: `python rust/melee_world/gen/mtu/mk_ef.py all` (`dbg_ef.sh <op>:<case>` bisects): the three TUs as one native C TU
  (shipping contraction, `efSync_Spawn` call sites counted by a patch macro), 78 ops x 400 cases (every eflib/efasync function,
  Dispatch over all 144 case ids with the argument pointers each case reads, the fused `0.04f * x + 0.3f` site of case 0x3E8
  discriminated). Fighter is filled/hashed by the 14 fields ef reads (the full record has bitfield overlays the walker cannot map).
  Traps: asserts inside `extern "C"` ports abort the test (no unwinding), so the fixture never reaches an assert (GObj_Create
  never NULL, EffectCount < 64); `any` Dispatch ids avoid the case ids because their argument kinds are pointers of the wrong type.
- Camera stubs (Camera_Init/Create/ForgetGameCamera/RefreshViewingMtx/8002A4AC/8002F3AC/PCMainCObj/SetQuakeScale/LogicToScreen) of
  world, modes, items and fighter now default to `tu::cmcamera`.
