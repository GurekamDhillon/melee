# enemies2

Code-only batch lane: no builds, game runs, or commits.

## Changes

This checkout already contained `like_like`, `octorok`, and `polar_bear` in the
spawn/event tables, along with stage-archive preloading and the stock defeat hook.
This patch completes the roster and corrects the portable Like Like spawn:

- `pc/gameworld/script_game.c`: append `It_Kind_Ottosea` to the roster without
  renumbering existing kinds; expand preload state/reset/index limits to seven.
  Use the stock Topi initializer and synchronize requested facing/model rotation.
- Like Like now uses `it_802DC4BC(0, ...)`, the floor/falling variant, and restores
  the requested position after its buried-enemy initializer subtracts 40 from Y.
  Previously `arg0=1` selected ceiling physics (`it_802DAD18`, positive vertical
  acceleration), which cannot land on FD.
- `pc/platform/gw_script.c`: expose `topi`, extend the defeat-event bounds, and
  accept `gd.hit({enemy=handle}, spec)`. Integer targets still mean fighter ports.
  The table form has the same offline/gameplay/active-match restriction as spawning,
  validates the handle, and forks the LAB rewind timeline.
- The isolated `ScriptGame_EnemyHit` block calculates knockback using
  `it_80270CD8` and loaded item attributes; mirrors `OnTakeDamageThink` damage
  accounting; invokes the item's real `dmg_received` callback. That callback
  chooses damage/death state and calls the existing `Item_ZakoDefeat` /
  `ScriptGame_EnemyDefeated` path. It does not fabricate defeat events. Consumed
  pending damage/knockback are cleared to avoid processing the injected hit twice.
  Missing, removed, or defeated handles, missing sources, and zero damage return false.
- Koopa and its shell transition are unchanged. The shell remains a separate item;
  this patch does not count its transition as a defeat or add it to the enemy roster.

Only those two existing C files are changed. No new translation units, bridge edits,
generated assets, build files, or upstream enemy AI files are needed. Existing enemy
pool state remains in snapshotted game BSS. New code blocks are marked `enemies2`.

### Data and AI audit

| Lua kind | Stock item | Asset source | Stock damage callback |
|---|---|---|---|
| `like_like` | `It_Kind_Likelike` | `GrNSr.dat` itemdata | `it_2725_Logic5_DmgReceived` |
| `octorok` | `It_Kind_Octarock` | language-selected ItCo, including its stone | `it_802E4B00` |
| `polar_bear` | `It_Kind_Whitebea` | `GrIm.dat` itemdata | `it_802E3884` |
| `topi` | **active** `It_Kind_Ottosea`, not `It_Kind_Old_Otto` | language-selected ItCo | `it_2725_Logic8_DmgReceived` |

All four stock callbacks use `Item_ZakoDefeat`. Scripted enemies already bypass
grZakoGenerator bookkeeping on defeat/destruction. Their walking/falling AI uses
ordinary item collision. Polar Bear's `grIceMt_801FA6D8` stage-scroll call is guarded
by the Icicle Mountain stage ID inside the callee. Topi's ice-block choice remains
the stock item-enabled/availability check. Octorok's stone and Topi's ice block use
ordinary item logic. Actual behavior on arbitrary stages still needs runtime checks.

**Topi identity confirmed from the user's NTSC 1.02 disc, read-only:** followed
`itPublicData + 0x8` (monster article table), entry 3 (`Ottosea`), the article's
model descriptor, and the joint/DObj/material/texture chains. Compared SHA-256 of
complete base-level texture storage against the trophy models:

- `ItCo.usd` Ottosea: all **five** textures in `TyToppi.dat`
  (`ToyToppiUsModel_TopN_joint`) match; zero match `TyOtosei.dat`.
- `ItCo.dat` Ottosea: all **five** textures in `TyOtosei.dat`
  (`ToyOttoseiModel_TopN_joint`) match; zero match `TyToppi.dat`.
- Both files have the monster table at data offset `0x4ec8`, the active Ottosea
  article at `0x4c50`, and its model descriptor at `0x4bc8`.
- `it_8027870C` selects `.usd` for US language and `.dat` otherwise. Thus `topi`
  retains the stock localized model, including the Japanese seal appearance.

No disc bytes or extracted assets were written into the worktree.

## Test and launch

Fixture: `pc/tests/enemies2.lua`. It logs
`TEST enemies2 section <n>: PASS|FAIL <detail>` for each check and calls `gd.quit()`
on completion or its 120-second wall-clock watchdog. It pads through memory-card
prompts and claims both human controllers as neutral. No human input is needed.

Checks LAB and FD's **internal** stage ID `Gr_Kind_Last = 0x25`; then, for each
kind, spawns at `(0,20)` facing left, observes live AI/physics and settling onto FD,
injects a 500-damage hit, checks the matching kind/handle defeat, rejects another
hit on the defeated handle, and checks exactly one event. Separate spawns check
explicit removal, stale handles, and absence of removal-triggered defeat events.
The original fighter-port `gd.hit` behavior must remain separate; the test checks
that an enemy hit does not change the source fighter's percent.

**For the integration lane, after its combined build**, from the workspace root
in Git Bash (replace `GW_MELEE` with the integrated checkout):

```bash
export GW_MELEE="$PWD/worktrees/codex-enemies2"
MELEE_SCENE='mode=lab;p1=mario/hu;p2=fox/hu;stage=fd;items=0' \
MELEE_PAD_SCRIPT="$GW_MELEE/pc/tests/enemies2.lua" \
MELEE_SCRIPTS=0 MELEE_TURBO=1 \
bash tools/port/run.sh enemies2 --iso 'C:/iso/Super Smash Bros. Melee (USA) (En,Ja) (v1.02).iso'
```

Use the gameplay launch above, not `run.sh --test` (which selects the native
headless-test entry point). The Lua file may also be loaded via `MELEE_SCRIPT`
by the combined runner, with the same LAB/FD scene and controller claims.
Passing requires the final `complete; failures=0` PASS line and **no** FAIL lines;
game exit alone is not a pass. Check native `script enemy: spawned`, `hit`,
`defeated`, and `queued defeat` lines for diagnostics. Run only once as part of
the coordinating lane's batch, per the task instructions.

## Verification and untested behavior

- `luac -p pc/tests/enemies2.lua`: passed (exit 0).
- `git diff --check`: passed.
- PowerPC syntax check attempted with:

  ```text
  ../../_toolchains/llvm/bin/clang.exe -fsyntax-only -w -DTARGET_PC --target=powerpc-unknown-eabi -nostdinc -Isrc -Isrc/melee -Iinclude -Ilibs/dolphin/include -Ipc -Ipc/gameworld -Isrc/sysdolphin -Isrc/MSL pc/gameworld/script_game.c
  ```

  The sandbox denied executing Clang (`permission denied`). **C syntax is not
  verified.** `gw_script.c` is native Windows code, not a PowerPC TU; it also has
  not been compiler-checked here.
- No build, link, bridge regeneration, executable-string check, game run, or
  commit was performed. The Lua test has not been run against the game, so no
  runtime PASS or red/green result is claimed.
- Untested: render/model appearance, all four FD outcomes, other stages and moving
  platforms, repeated scene transitions, savestate/rewind, capture interactions,
  Octorok projectiles, Polar Bear attacks, and Topi's optional ice-block behavior.
  The injected hit calls stock damage callbacks directly; it does not exercise
  hitbox/hurtbox overlap, attack scaling, or the full collision VFX path. Natural
  fighter attacks and nonlethal hit reactions need integration testing too.
