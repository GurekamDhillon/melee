# YouGame listing draft: the Rust engine as a Patches desktop build

**DRAFT: the user reviews and edits every word before anything is entered on yougame.co.** Nothing here was
uploaded or entered by an agent; no account was used. Status 2026-11-08, branch `port/ygpack`; dual-disc support
added 2026-11-17, branch `port/yg-dualdisc` (see "Discs" below).

## Listing fields

- **Platform:** `desktop_app` (a native Windows build; the game page shows "Desktop app only" in browsers and
  phones, and the desktop app offers only the desktop build for that computer).
- **Kind:** patch (`kind: "patch"`), a version of the existing base game
  `gamecube/super-smash-bros-melee-usa` ("Super Smash Bros. Melee (GameCube)", catalog Q1052131). Reusing that key
  needs no new base-game definition. The public `api/base/search` answer (read 2026-11-08, no sign-in) lists one
  accepted dump: "Super Smash Bros. Melee (USA) (En,Ja) (Rev 2)", SHA-1 `d4e70c064cc714ba8400a849cf299dbd1aa326fc`,
  1,459,978,240 bytes, extensions `iso`, `gcm`. That matches what the engine accepts (GALE01 rev 2, a plain image).
  Suggested `rejected` dumps with reasons (the player sees these): USA Rev 0 / Rev 1 ("This is NTSC 1.00/1.01; this
  version plays only 1.02 (Rev 2)"), PAL / Japan ("This is the European/Japanese release; this version plays only
  the USA disc, revision 2"). Their hashes come from `api/base/dumps?catalog=Q1052131` when the listing is made.
- **Also requires (optional second disc):** one `also_requires` entry, slot `brawl`, optional:

  ```json
  [{ "slot": "brawl", "required": false, "adds": "Brawl-derived characters and Brawl audio",
     "catalog": "Q843131", "key": "wii/super-smash-bros-brawl-usa",
     "name": "Super Smash Bros. Brawl (USA)", "system": "wii",
     "extensions": ["iso", "wbfs", "rvz", "ciso", "gcz", "wia"],
     "accepted": [{ "name": "Super Smash Bros. Brawl (USA, Canada) (Rev 2)",
                    "sha1": "59432f150bf6b871ab378bb5b28e92005e0862f6", "size": 8511160320,
                    "partition_sha1": "ac7bc058ab92fce3f9f4c3f3fd0879e5d6932fb8",
                    "partition_offset": 260046848 }] }]
  ```

  Values from the SDK reference's "Native builds on the player's own discs" (read 2026-11-17). The player links
  their own Brawl disc; Melee plays without it. Confirm the `adds` wording and that Brawl stays optional.
- **Name:** Melee (Rust engine)
  (alternative: "Melee Native (Rust)". Avoid "Super Smash Bros." in our own title; the "Runs on" line already
  names the game.)
- **Short description (one line):** A native, open-source engine for Melee that runs on your own disc.
- **Long description:**

  > A from-scratch native engine for Super Smash Bros. Melee, written in Rust and checked frame by frame against
  > the original game. It is a Windows program, not an emulator: it reads the game's files from your own disc
  > image when it starts, and nothing from the game is included in the download.
  >
  > **Bring your own disc.** You need your own copy of Super Smash Bros. Melee for GameCube (USA, revision 2,
  > also called NTSC 1.02) as a disc image (.iso or .gcm) made from a disc you own. Choose it on this page once;
  > YouGame checks that it is the right version and hands it to the game when you press Play. Other versions
  > (1.00, 1.01, PAL, Japanese) are not supported.
  >
  > Your memory card and settings follow your YouGame account between computers. Replays stay on this computer.
  > Controllers: XInput pads and keyboard. Online play goes over the internet (UDP).
  >
  > This is an independent fan project. It is not affiliated with or endorsed by Nintendo or HAL Laboratory.

- **Credits:** GD (the Rust engine and the GD's Melee port); the doldecomp/melee decompilation project and its
  contributors, whose matching decompilation the engine is ported from; Hero88go (Melee Unlocked, the reference for
  the YouGame bridge protocol; no code used). The user should confirm the names and how each wants to be credited.
- **Licence:** GNU GPL v3.0 or later (the build ships `LICENSE.txt` and `THIRD_PARTY_NOTICES.txt`; source:
  link to the public repository branch, the user picks which).
- **Tags/genre:** fighting, versus, local multiplayer, online.

Wording rules this follows (publish.md "Patches"): the listing never hosts, links to or says where to download the
game; it says only that the file comes from the player's own copy. Do not add any "how to get the ISO" text, ever.

## Open decisions for the user

1. Name and credits wording (above).
2. Whether to list `rejected` dumps (recommended) and which.
3. Brawl disc: **resolved 2026-11-17** — YouGame's dual-disc support (`also_requires` + `{disc:brawl:files}`) is in
   the SDK reference, and the listing and manifest below use it. Still confirm the `adds` wording and optional-vs-required.
4. Whether to set `base_game.plays_with` (only if a full listing of the same build exists: none does; a multi-disc
   build plays online in its own pool, so it cannot use `plays_with`).
5. GameCube adapter: the app runs a Windows build in its sandbox with `controllers` defaulting to `"none"` (no USB);
   XInput still works. A WUP-028 adapter would need `"controllers": "mirror"` and our side of the pad mirror, which
   is not written. Decide whether launch is XInput/keyboard only.
6. Window: standalone own window (no `embed`). The player box needs the pad mirror too.

## Discs: what the app gives a native build

Read from the installed desktop app's launch code (app 0.13) and the public `sdk.md` "Native builds on the player's
own discs" (read 2026-11-17):

- A spec has **one** `disc` boolean and **one** `{disc}` placeholder for `base_game`. With `"disc": true` the app
  substitutes the path of the disc the player chose for this listing (read-only inside the sandbox) and refuses to
  launch without one ("Choose your disc image on the game's page first.").
- Discs live in **one app-wide store**, each with a list of listings it is granted to. A Melee disc the player
  already added for another listing (Melee Unlocked, or the Orca/Dolphin runtime) is offered again: the launcher
  asks "Allow this game to use your disc image on the game's page." and does not ask for the file again.
- **Dual-disc support (2026-11-17).** A `kind: "patch"` listing may name more discs in `also_requires` (one entry
  per slot, at most 4); each entry is required unless it says `"required": false`. The manifest mirrors it in
  `discs` (`{ "<slot>": true | "optional" }`, or `{ "<slot>": {"need": ..., "files": [...]} }` for a partial unpack).
  This needs the **app** to advertise `YouGameMultiDisc/1` (and `YouGameDiscFiles/1`/`/2` for the files folder); an
  older app is told to update. Our build advertises nothing itself.
  - Main disc: `{disc}` / `YOUGAME_DISC_BASE`, the player's own Melee image, readable directly (GameCube is not
    encrypted).
  - Brawl (slot `brawl`): `{disc:brawl}` / `YOUGAME_DISC_BRAWL` is the raw **encrypted** Wii image (not readable
    directly); `{disc:brawl:files}` / `YOUGAME_DISC_BRAWL_FILES` is the unpacked DATA folder (`files/...`, `sys/...`).
    We use the folder: the Brawl audio reader wants `files/sound/smashbros_sound.brsar`, and the Petey model wants
    its files under `files/...`. A full Brawl unpack is ~7 GB / 5,300 files (minutes, ~7.5 GB free); a selective
    `files` list (untested by YouGame as of 2026-11-17) would cut that to the paths we name.
  - An optional slot the player has not linked drops its placeholders, so `--brawl-files {disc:brawl:files}` drops as
    a pair; the game plays Melee only.

## Engine side (this branch)

- `yougame-native.json` (tools/rust-port/yougame/): `{"version":1,"args":["play","--iso","{disc}","--brawl-files","{disc:brawl:files}"],"disc":true,"discs":{"brawl":"optional"},"net":"internet","memoryMb":4096}`.
- Disc check before boot (melee/src/disc.rs `identify`): GALE01, disc 1, revision 2, GameCube magic, not a
  compressed image (RVZ/WIA/CISO/GCZ/WBFS named); the message never contains the path; shown as a message box and as
  the `crash:` stderr line the app shows when the game stops; exit code 4.
- Optional Brawl disc + first-run hook (melee/src/firstrun.rs): YouGame's `{disc:brawl:files}` (`--brawl-files` /
  `YOUGAME_DISC_BRAWL_FILES`) is used as-is (already unpacked; no conversion); a local `--brawl-disc` /
  `MELEE_BRAWL_DISC` still takes a plain .iso, RSBE01 header-checked, output under `<YOUGAME_DATA>\brawl` (the
  conversion itself is still a stub). The Brawl audio reader (melee_audio/src/brawl/reader.rs) accepts both
  `<root>/disc/files/sound` and `<root>/files/sound`.
- Saves (melee_platform/src/yougame_saves.rs): card `.gci` files, settings.cfg, audio.cfg via
  saveFileList/Read/Write/Delete (sdk.md recipe); replays stay in `<YOUGAME_DATA>\replays`.
