# Petey Piranha: extracting the boss from your own Brawl disc (owner-run)

Who runs this: **you, the owner, on your own Brawl dump** (decision P9, 2026-11-09). Agents never open, read or
list a Brawl disc or anything extracted from it; they only read the `[?]` numbers you type back into the chat or
into `petey-numbers.txt` (step 5). Nothing from these steps is ever committed: the output folder is git-ignored
(`/local/brawl/` in `.gitignore`), and a `git status` that shows any of it means stop and take it out of the index.

What the game does with it later: read the files from that folder at runtime and draw the real Petey in place of
the placeholder shapes (`MELEE_ENVOY_BOSS=petey`, notes/petey-boss.md). Until that converter exists (plan
milestone P1 in notes/brawl-boss-petey.md) the folder is only the source; the boss plays with original stand-in
shapes.

## 0. What you need

| tool | why | where |
|---|---|---|
| your Brawl disc image (RSBE01, NTSC-U, `.iso` or `.rvz`) | the source | your own dump |
| Dolphin (5.0-2xxxx or newer) | unpacks the Wii disc filesystem | dolphin-emu.org |
| BrawlCrate (v0.4x or newer, Windows) | opens `.pac` / `.brres`; exports models, animations, textures, params | github.com/soopercool101/BrawlCrate |
| PowerShell (built in) | hashes and the listing | |

Our own `--brawl-disc` first-run path (`rust/melee/src/firstrun.rs` on `port/ygpack`) only checks the disc header
today and writes an empty marker; it extracts nothing, so it does not cover this. Use Dolphin + BrawlCrate.

## 1. Make the output folder

In PowerShell, from the repository root (`E:\Projects\Melee Workspace\melee`, or any worktree):

```powershell
New-Item -ItemType Directory -Force local\brawl\petey\disc, local\brawl\petey\export | Out-Null
git check-ignore -v local\brawl\petey\disc   # must print the .gitignore line; if it prints nothing, stop
```

## 2. Take the boss files off the disc (Dolphin)

1. Dolphin game list: right-click Brawl, **Properties**, tab **Filesystem**.
2. Expand the data partition. Petey's files sit in the Subspace boss / enemy area under `pf/`. The exact names are
   **[?]** (agents may not look). His internal name is believed to be **BossPackun**, so search, in this order:
   - folders or files under `pf/` whose name contains `packun` (any case), e.g. `...bosspackun...pac`;
   - the cage props: names containing `cage` or `kago` near the same files (they may also be inside his model);
   - a separate motion file if there is one (`...Motion...pac`, `...MotionEtc...pac`).
   Do **not** take `Packun` files under `pf/item/` (the Piranha Plant item) or any stage file.
3. Right-click each match, **Extract File...**, save it into `local\brawl\petey\disc\` with its original name. If
   you cannot tell which is which, extract the whole folder that holds them (right-click the folder, **Extract
   Files...**) into `local\brawl\petey\disc\` and trim later.

Write down the in-disc paths you took (step 5 records them); they replace the `[?]` above.

## 3. Export what we need (BrawlCrate)

Open each `.pac` from `local\brawl\petey\disc\` in BrawlCrate. Inside, an ARC tree holds one or more BRRES
(model data, motion data, texture data):

| item | BrawlCrate node | export into `local\brawl\petey\export\` |
|---|---|---|
| model + skeleton | `3DModels(NW4R)` > MDL0 (Petey; the cages if they are separate MDL0s) | right-click, Export: `petey.dae` (`cage.dae` / `cage_l.dae`, `cage_r.dae`) |
| textures | `Textures(NW4R)` > every TEX0 | right-click the folder, Export All: PNG |
| bone animations | `AnmChr(NW4R)` > every CHR0 | right-click the folder, Export All, raw `.chr0` (names kept) |
| material / visibility anims | `AnmTexSrt`, `AnmTexPat`, `AnmClr`, `AnmVis` if present | Export All, raw |
| params | any `Params` / `MiscData` / unknown data node next to the model | right-click, Export Raw: `.bin` named after the node |
| sounds | skip for now (plan P6; they live in `smashbros_sound.brsar`, a separate step later) | |

Also save BrawlCrate's names: expand the model's `Bones` and the CHR0 folder and type (or paste) the bone names
and animation names into `local\brawl\petey\listing.txt`. Names only, no data.

## 4. Hashes and a listing (verification)

```powershell
cd local\brawl\petey
Get-ChildItem -Recurse -File disc, export |
  Get-FileHash -Algorithm SHA256 |
  ForEach-Object { "{0}  {1,10}  {2}" -f $_.Hash.Substring(0,16), (Get-Item $_.Path).Length, (Resolve-Path -Relative $_.Path) } |
  Set-Content -Encoding utf8 manifest.txt
Get-Content manifest.txt
git status --short --ignored . | Select-Object -First 3   # every line must start with "!!" (ignored)
```

Check:
- `disc\` holds the `.pac` file(s) you took, each well over 100 KB (0 bytes means the extract failed);
- `export\` has at least one `.dae`, a set of `.png` and a set of `.chr0`;
- the `git status` lines all start with `!!`.

Running the same command later must print the same `manifest.txt`: that is how you, and later the converter, know
the cache still matches. You may paste the manifest lines (hash prefix, size, file name) into the chat; they are
not game data.

## 5. Record what you found

Create `local\brawl\petey\petey-numbers.txt` (ignored; this file you may paste to the chat):

```
disc paths taken:   (e.g. pf/.../xxx.pac)
model bones:        (count, root bone name, the two hand bones that hold the cages)
animations:         (count; names of the wait, the 5-6 attacks, damage / stun, death)
cage props:         (separate MDL0, or part of the main model)
```

## 6. The [?] numbers you can read off

These replace the guesses in `rust/melee_modes/src/tu/petey.rs` (`HP_BASE`, `CAGE_HP`, the `ATTACKS` table) and the
`[?]` marks of notes/brawl-boss-petey.md. Two ways; B needs no file tools.

**A. From a params node (BrawlCrate).** If a `Params` / `MiscData` node exists, BrawlCrate shows it as rows of
floats / ints. Boss HP is one number per difficulty: look for a run of 5 increasing values (Easy to Intense). Note
each candidate with its offset and mark it "params, unconfirmed".

**B. By observation (console or Dolphin; Boss Battles, or his Subspace stage).** One character, one move, no items:

| number | how | write down |
|---|---|---|
| Petey HP | Boss Battles, Normal: count Mario forward smashes (fresh, uncharged) to defeat him; repeat on Easy and Hard | hits x the move's damage, per difficulty |
| cage HP | the same move on one cage until it breaks (watch that Petey's own bar does not move) | hits per cage, per difficulty |
| do cages heal | stop hitting a damaged cage for 30 s, then count again | yes / no |
| speed-up | time between attacks above and below half HP | two times (s) |
| attack timings | per attack: wind-up, active, recovery before he moves again | 3 numbers each: cage slam, cage sweep, belly flop, spin, bite |
| stun | which hit stuns him (head only? any strong hit?) and for how long | where, seconds |

Paste the filled table (numbers only) to the chat or into `petey-numbers.txt`; an agent moves them into the data
table and clears the `[?]` marks.

## Never

- Never `git add -f` anything under `local/brawl/`; never copy it into `rust/`, `docs/`, `pc/` or a mod folder.
- Never send the extracted files anywhere (LAN, the pub remote, YouGame): the hashes and the numbers are enough.
- Agents: never open, list or hash these folders yourselves; ask the owner.
