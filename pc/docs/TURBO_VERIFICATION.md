# Gameplay Turbo: offline verification, 2026-10-07

This is the Project M style match rule, not `MELEE_TURBO` / `gw_turbo` (the test
clock). The lane packet and research in the read-only workspace remain the design
sources. No game was launched and no full game build was performed in this lane.

## What already existed

- `pc/platform/gw_matchrules.h`: one rule word, validation, frame count, V1 `0x5fb`.
- `pc/gameworld/script_fighter_interrupt.inc`: hit-confirm windows in game BSS,
  normal IASA first, shared exits second, indicator, multi-hit/repeat guards,
  offline script ownership and release. `script_fighter_interrupt_tests.inc`
  already held an in-game fixture; it was not a standalone native test.
- `src/melee/ft/ftcoll.c`: hit, shield and item hooks. `fighter.c`: free-frame
  ticking, curated SyncTest word and rollback hash. Snapshot storage remains the
  same `ft_intrwin[12]` BSS table; the shared header does not relocate it.
- `gw_net.c` / `gw_netplay.c`: protocol-5 rule exchange, guest host-rule adoption,
  explicit mismatch refusal, supported-word/scene agreement checks. Only a private
  host uses its saved preference; Random Opponent forces the host word to zero.
- `gmfrontend.c`, `gmfrontend_settings.inc`, `gmfrontend_online.inc`: private
  host preference, persisted `turbo_online`, guest-visible room label before
  readiness, and local Versus `turbo_versus` toggle.
- `gw_script_fighter_caps.inc`: `gd.fighter_interrupt(entity,
  {frames=12, exits={"tilt","jump"}, guard=true, restore_jumps=true})`. Its write
  gate requires offline gameplay permission and branches the rewind timeline.
  A modifier needs a fighter entity, granting event/move key, duration, allowed
  exit classes and repeat/jump flags; no modifier system was added.
- Runtime match-rule logging already records the word and source in the game log.

## What this lane changed

The existing state/rule decisions now live in `gw_intrwin_core.h`, compiled by
both the PPC game and the native fixture. Fighter-to-key mapping, engine exit
calls, colour overlay, hitlag gate and native diagnostics remain in the game
adapter. The adapter returns before reading a fighter when Turbo is off.

`engine_gaps_test.c` includes `turbo_rule_test.inc` and `turbo_net_test.inc` so
the read-only workspace runner can execute them through its existing `engine-data`
target. The latter uses the real transport and existing simulated-peer tests;
it never opens UDP sockets. No workspace runner change is needed.

Recordings previously lost Turbo and playback forced it off. `gw_replay_matchrules.h`
now encodes a `GDT1` tag and big-endian rule word after the standard `0x2f8`-byte
Game Start payload. The event-size table advertises the extra eight bytes.
`gw_replay.c` writes/reads it, refuses tagged unsupported words/versions, and logs
the rule; `gw_runtime.c` chooses the recorded rule during playback. Legacy and
untagged upstream recordings stay off regardless of local preferences. This
changes the port recording's Game Start payload length; third-party reader
compatibility still needs verification. Older port playback ignores this suffix
and cannot faithfully play a Turbo recording. Netplay wire format is unchanged:
protocol remains 5; different protocol versions are refused, and different EXE
build hashes are refused before gameplay.

`CREDITS.md` and the in-game Credits table now include UnclePunch's 20XX Turbo
alongside the existing Project M / Project+ credit. Ideas only, no code copied.

The existing `turbo_colanim` preference was read separately on each peer even
though the colour overlay mutates snapshotted fighter state. Turbo now always
uses its existing default overlay id (8), including playback. The setting is
ignored: a cosmetic preference must not steer the native simulation.

## Rule set preserved

V1 opens on fighter hits and shield hits, not throws/grabs; item/projectile
opening is an optional bit and is off in V1. The first connected hit opens
30 free logic frames; later hits of that instance do not refresh or reopen it.
Hitlag cannot consume or use the window. The first Turbo cancel consumes it;
an action change, landing, or expiry closes it. Retail exit entries clear
remaining hitboxes; landing lag remains retail.

No same-move cancel; smash directions are move families. Specials conservatively
cannot cancel into any special. Only jabs/dash attacks can dash-cancel; smashes
cannot crouch/jump/dash-cancel; grounded attacks cannot shield-cancel; aerials
and specials cannot air-dodge-cancel. Walk/turn never consume the V1 window.
Air hits restore air jumps. Optional `NO_MOVE_LOOP` remembers a move across
movement cancels; this is not in V1. No damage, knockback or hitstun scaling.

## Reproduce without reading disc configuration

Run in Git Bash **from this worktree** (there must be no `.env` in this worktree):

```bash
test ! -f .env || exit 1
export GW_MELEE="$(pwd -W)"
export GW_ROOT="$GW_MELEE"                 # skips the workspace .env
export GW_BUILD_ROOT="$GW_MELEE/.native-tests"
export GW_CLANG="$(cd ../../_toolchains/llvm/bin && pwd -W)/clang.exe"
export GW_DAWN_INCLUDE="$GW_MELEE/extern/aurora/include"
bash ../../tools/port/native_test.sh engine-data
```

All outputs stay under the ignored `.native-tests/` in this worktree. The
compiler is read from the workspace; the runner and workspace are not edited.

Result: **PASS** data fixtures, rule-word validation, hit opening and consumption,
multi-hit guard, every exclusion, rule variants, jump restore, snapshot replay
before a hit and mid-window across consumption/refusal/expiry, and replay suffix
roundtrip/legacy-off/truncated-tag/unknown-version/unsupported-word cases.
Simulated transport tests pass for standard handshake, host-rule adoption,
explicit agreement, on/off mismatch refusal and a near-miss rule word.
Existing Windows CRT deprecation warnings are nonfatal (12 warnings).

Turbo **off has no gameplay behavior change**: the adapter's existing early
return is preserved, and the native fixture checks all move keys and hit kinds
against a nonzero sentinel window: every byte and the fighter jump count remain
identical. The off hash and original retail input callback path are unchanged.
This is source/native evidence; a full game-frame parity run was not performed.
Temporarily disabling `NO_SELF` made the fixture fail at its self-cancel assertion;
restoring it returned the suite to PASS.

PowerPC syntax checks passed for `script_game.c`, `fighter.c`, `ftcoll.c`, and
`gmfrontend.c` using:

```bash
"$GW_CLANG" --target=powerpc-unknown-eabi -fsyntax-only -w -DTARGET_PC -nostdinc \
  -Isrc -Isrc/melee -Iinclude -Ilibs/dolphin/include -Ipc -Ipc/gameworld \
  -Isrc/sysdolphin -Isrc/MSL <file>
```

Native Windows syntax checks passed for `gw_net.c`, `gw_netplay.c`,
`gw_rollback.c`, `gw_replay.c`, `gw_runtime.c` and `gw_script.c`. Syntax checks
do not prove a full link, bridge regeneration or game execution.

## Owner and friend play check (not performed here)

1. Local Versus > Rules: enable Turbo (Versus). Whiff jab/tilt/smash/aerial:
   expect no early cancel. Hit a stationary opponent and cancel into a different
   move after hitlag; verify the flash and `turbo: window open` / `turbo: cancel`
   logs. Same-move input must be refused by the widened exit list.
2. Check shield-hit opening, all restricted exits, exhausted air-jump restoration,
   multi-hit attacks, normal landing lag and remaining hitboxes after cancellation.
3. Offline script: open the same window with `gd.fighter_interrupt`. Verify
   ownership release, rewind and refusal during netplay. CPU pad inputs must
   actually request a permitted cancel; this rule does not add an AI strategy.
4. Online > Turbo preference ON, host a private room, friend joins: confirm
   Turbo is shown before either player readies. The guest adopts the host word.
   For an explicit disagreement test, set a scripted guest's
   `MELEE_NETPLAY_TURBO=off` against a Turbo host: expect handshake refusal.
   Random Opponent must remain standard with the host preference ON.
5. Use existing reactive pad bots with Turbo ON for long combat; force rollbacks
   across open/use/expiry and compare full snapshots/SyncTest and logs. Require
   zero desyncs; compare an OFF control. The quick-reference already reports a
   separate historical heavy-combat SyncTest mismatch with Turbo OFF as well.
6. Record a Turbo match, play it back on the same build with the local setting
   OFF, and compare state traces. Test an old recording with the setting ON:
   it must remain OFF. Check third-party Slippi readers against the extended event.

No zero-desync soak, `gd.rewind_test`, visual verification, two-client private
lobby session, public matchmaking session, full recording/playback parity, or
full linked EXE verification is claimed by this lane.
