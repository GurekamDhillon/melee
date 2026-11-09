# YouGame listing draft: the Rust engine as a Patches desktop build

**DRAFT: the user reviews and edits every word before anything is entered on yougame.co.** Nothing here was
uploaded or entered by an agent; no account was used. Status 2026-11-08, branch `port/ygpack`.

## Listing fields

- **Kind:** patch (`kind: "patch"`), a version of the existing base game
  `gamecube/super-smash-bros-melee-usa` ("Super Smash Bros. Melee (GameCube)", catalog Q1052131). Reusing that key
  needs no new base-game definition. The public `api/base/search` answer (read 2026-11-08, no sign-in) lists one
  accepted dump: "Super Smash Bros. Melee (USA) (En,Ja) (Rev 2)", SHA-1 `d4e70c064cc714ba8400a849cf299dbd1aa326fc`,
  1,459,978,240 bytes, extensions `iso`, `gcm`. That matches what the engine accepts (GALE01 rev 2, a plain image).
  Suggested `rejected` dumps with reasons (the player sees these): USA Rev 0 / Rev 1 ("This is NTSC 1.00/1.01; this
  version plays only 1.02 (Rev 2)"), PAL / Japan ("This is the European/Japanese release; this version plays only
  the USA disc, revision 2"). Their hashes come from `api/base/dumps?catalog=Q1052131` when the listing is made.
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
3. The Brawl disc (below): no supported route today; ask YouGame or drop it from this listing.
4. Whether to set `base_game.plays_with` (only if a full listing of the same build exists: none does).
5. GameCube adapter: inside the sandbox the app gives a `"controllers": "none"` build no USB; XInput still works
   (the app's comment on DEFAULT_LAUNCH_SPEC). A WUP-028 adapter would need `"controllers": "mirror"` and our side
   of the pad mirror, which is not written. Decide whether launch is XInput/keyboard only.
6. Window: standalone own window (no `embed`). The player box needs the pad mirror too.

## Discs: what the app gives a native build (verified in the app, needs YouGame confirmation where marked)

Read from the installed desktop app's own launch code (its `yougame-native.json` reader and launcher, app 0.13;
the public docs refer to `docs/MELEE_UNLOCKED.md`, which is not public):

- A spec has **one** `disc` boolean and **one** `{disc}` placeholder. With `"disc": true` the app substitutes the
  path of the disc the player chose for this listing (read-only inside the sandbox) and refuses to launch without
  one ("Choose your disc image on the game's page first.").
- Discs live in **one app-wide store**, each with a list of listings it is granted to. A Melee disc the player
  already added for another listing (Melee Unlocked, or the Orca/Dolphin runtime) is offered again: the launcher
  asks "Allow this game to use your disc image on the game's page." and does not ask for the file again. So the
  Melee disc already known to the app is reused by our spec's `"disc": true` with no re-supply. (Verified in the
  app's code; that the site's page offers the stored copy for a new patch listing of the same base-game key
  **needs YouGame confirmation**, sdk.md hints at it: "the player has not let this listing use the copy another
  patch got".)
- The disc must match the listing's `base_game` accepted dumps, so a listing is tied to exactly one base game.
  **A second, optional disc (Brawl, RSBE01) cannot be requested**: no second placeholder, no second base game per
  listing, and no doc mentions one. A Brawl disc the player attached for a Project+ listing on Orca is granted to
  that listing only, and our spec has no way to name it. **Needs YouGame confirmation / a feature request.**
- Closest supported routes, in order:
  1. Ask YouGame for an optional second disc in the spec (for example `"discs": {"brawl": {"base": "wii/super-smash-bros-brawl-usa", "optional": true}}` with a `{disc:brawl}` placeholder). The engine side is ready: `--brawl-disc PATH` / `MELEE_BRAWL_DISC`.
  2. A `prepare` step (sdk.md "Game files made from the player's copy") is browser-side and for one base game;
     Brawl is 7.9 GB, past the browser path; not usable.
  3. Ship Brawl-derived characters only in a separate listing whose base game is Brawl: its data folder is its
     own (`YOUGAME_DATA` is per listing), so it cannot feed this listing. Not usable.
  Until one exists the YouGame build plays Melee only; the first-run hook simply reports "no Brawl disc".

## Engine side (this branch)

- `yougame-native.json` (tools/rust-port/yougame/): `{"version":1,"args":["play","--iso","{disc}"],"disc":true,"controllers":"none","net":"internet","memoryMb":4096}`.
- Disc check before boot (melee/src/disc.rs `identify`): GALE01, disc 1, revision 2, GameCube magic, not a
  compressed image (RVZ/WIA/CISO/GCZ/WBFS named); the message never contains the path; shown as a message box and as
  the `crash:` stderr line the app shows when the game stops; exit code 4.
- Optional Brawl disc + first-run hook (melee/src/firstrun.rs): `--brawl-disc` / `MELEE_BRAWL_DISC`, RSBE01 header
  check, output under `<YOUGAME_DATA>\brawl`, progress lines; the conversion itself is an empty stub.
- Saves (melee_platform/src/yougame_saves.rs): card `.gci` files, settings.cfg, audio.cfg via
  saveFileList/Read/Write/Delete (sdk.md recipe); replays stay in `<YOUGAME_DATA>\replays`.
