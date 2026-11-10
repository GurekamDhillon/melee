# Envoy tag on the same port: feasibility (milestone 0, 2026-11-09)

The owner's idea (full-game plan Q13): a tag swaps the fighter **in place on the same port**, like Zelda / Sheik, with the
benched fighter's % and state kept in Envoy's own data, so ports 3 and 4 stay free for 4-player co-op or extra CPUs.

## 1. How the tag is built today

It is already same-port. `melee_modes::tu::tag_swap` (port/envoy1p) rides the retail Zelda / Sheik transform path:

* at match start `Player_80031AD0` calls `tag_apply_partner`, which maps the partner as the main character's transform
  sub-fighter (`Player_MexSetPartner`); `Fighter_Create` builds it **asleep in entity slot 1 of the same port**;
* the tag (`swap`) is `ftCommon_8007EFC8`: wake the sleeper, copy position / velocity / input / damage state, sleep the
  outgoing one; each character's own % is restored afterwards; stocks are the port's, so the pair shares them.

Ports 3 and 4 are never used by the tag. They show up only in the plan's *local co-op* proposal (P1+P3 / P2+P4) and in
the CPU seats of netplay (`--cpu`). So "free ports 3 and 4" is already true for the tag itself.

What the current shape costs:

* the benched fighter is a whole live (sleeping) fighter object for the match: memory, a GObj, and the snapshot pages;
* it **occupies the port's sub-fighter slot**, which is why Ice Climbers (Nana), Zelda and Sheik (each other) cannot be
  a tag member (`envoy::taggable`, risk R14).

## 2. The prototype: despawn / spawn on the same slot

`MELEE_TAG_INPLACE=1` (process-wide, off by default; nothing changes without it):

* **benched state in plain statics**: `TAG_BENCH_CK[slot]` (character) and `TAG_BENCH_PCT[slot]` (its own %), in
  `melee_fighter::tu::player`; statics are snapshot state, so they roll back.
* **preload**: `Player_80031CB0` (the scene's file preload) also preloads every configured partner's files, so the
  mid-match `Fighter_Create` never touches the disc.
* **swap** (`tag_swap::swap_inplace`, inside the existing 60-frame freeze, at its release): `Fighter_Create` the benched
  character as a sleeping sub-fighter in the free entity index, run the same retail transform swap onto it, set its
  benched %, free the outgoing fighter (`HSD_GObjFree`, as `Player_80031EBC` does), bench the outgoing character and
  its %, and re-enable input (a fighter created after GO starts with input off; `Player_80031848`).
* between tags the port owns **one** fighter object.

## 3. Results

| check | default tag | in-place prototype |
|---|---|---|
| `tag/check.sh` 1a: the 3 scripted tags happen | 3 | 3 |
| 1b: two golden runs identical (determinism) | yes | yes (901 rows) |
| 2: synctest depth 4 across the tags | 0 mismatches | 0 mismatches (15 swaps incl. resimulation) |
| 3: `MELEE_SNAP_CHECK=1` page-by-page snapshot check | pass | pass |
| spawn / despawn cost (wall clock, release) | - | first spawn 2.9 ms (cold caches), then 0.3-0.4 ms; despawn ~0.2 ms |

No hitch: the worst spawn (2.9 ms) is inside a frozen frame of the 60-frame cinematic and well under the 16.7 ms
frame; resimulations repeat it, so the cost also lands in rollback frames (~0.3 ms warm, fine at depth 7).

Two peers (`TAG_INPLACE=1 TIME=120 PERIOD=900 bash tools/rust-port/envoy/m0_np_envoy.sh`, loopback, 30 ms inbound delay and
3 % loss each way, two CPU9 seats, stage shifts every 900 frames): 7503 frames finalised on both peers, 0 with different
hashes, 1210 host rollbacks, 8 stage-switch executions and the in-place tag on both peers, the host's replay equal to the
end. The first spawn took 7.6 ms there (cold, both peers headless on one machine): still inside a frozen frame, but the
number to watch on slow machines; a warm-up `Fighter_Create` + free at match start would remove it.

## 4. Ice Climbers, Zelda, Sheik

* **Outgoing** ICs / Zelda / Sheik: the prototype refuses a swap while the free entity index is taken (Nana, or the
  transform partner). Needed: free both entities of the outgoing character (Popo and Nana; or Zelda and her sleeping
  Sheik) and bench the pair's state (Nana's % / alive flag; which half was active).
* **Incoming** ICs: spawn Popo through the transform swap as now, then `Fighter_Create` Nana into the freed index
  (`b0 = 1`, no transformation), as `Player_80031AD0` does at match start. Nana's CPU follow logic reads the port's
  entity 1; it expects her at match start, so her spawn position and entry state need checking.
* **Incoming** Zelda / Sheik: the tag cannot use the transform swap for them (their sub-fighter slot is their own
  transform). Spawn the character and her partner directly (`Player_80031AD0`-style, both entities), then copy the
  outgoing fighter's state with the same field list `ftCommon_8007EFC8` uses (a small `copy_state(out, in)` helper);
  her own down-B transform then keeps working on the port.
* So all three become possible once the swap is "despawn everything on the port, spawn the incoming character's full
  set, copy state" instead of "spawn into the free index and use the transform swap". The prototype proves the hard
  parts (mid-match `Fighter_Create` from preloaded files, freeing a fighter, rollback of both) are sound.

## 5. Recommendation

* Adopt the in-place shape for 1.0 (behind the flag until milestone 1 lands its synctest depth 7 across tags):
  one fighter object per port, benched state in Envoy data, and the path to ICs / Zelda / Sheik open.
* Next steps: the state-copy helper, both-entity despawn / spawn (ICs, Zelda / Sheik), the Envoy HUD reading the
  benched % from `TAG_BENCH_PCT`, and KO handling while a partner is benched (today the sleeping sub-fighter rides the
  port's rebirth; with in-place the port simply has one fighter).
