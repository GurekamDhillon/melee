# Install a mod from a local ZIP

Download the ZIP yourself. GD does not follow Google Drive links or other links
in Nucleus posts. A fighter ZIP must be a MexManager package containing
`fighter.json`, its fighter DAT, animations, and costumes. The same compatibility
checks used by Nucleus apply; a moveset requiring another clone base or custom
items can still be refused.

## Folder drop

1. Find the running game's mods folder: `MELEE_MODS_DIR` when set, otherwise
   `mods` beside `melee-pc.exe`.
2. Create `_incoming` inside it and copy the ZIP there. Leave the ZIP packed.
3. While offline, open **Mods**, choose **Nucleus** (Z), then select the
   **Incoming** tab with L/R or Tab. It refreshes every two seconds; Z refreshes
   it immediately. Up to 128 regular ZIP files are offered.
4. Select the ZIP and press **A / Space** to install, or **Y** for details and
   the Install action. Wait for **Installed** or a failure message. Details show
   the result and any fighter compatibility limitations.
5. After a successful install, restart the game. The installed mod is enabled
   for that boot and appears in the main Mods list. Pending installs lock that
   list's toggles until restart, protecting the updated enabled list.

The original ZIP remains in `_incoming`; remove or move it yourself when you no
longer want it offered. Installing the identical ZIP again updates the same mod,
even if its ZIP filename changed. Different ZIP bytes create a different mod ID.
The Nucleus **Installed** tab lists Nucleus posts; local mods are managed through
the main Mods list after restart.

## Console or Lua

Open the console with the grave/tilde key and enter:

```lua
= gd.mods_install_zip([[C:/Users/YourName/Downloads/Tails.zip]])
```

Use forward slashes or Lua long brackets for Windows paths. An absolute path is
recommended; relative paths resolve against the game's working directory.
Paths must name a local `.zip`, under 500 characters. URLs and UNC paths are
rejected.

The function returns `accepted, message`. Acceptance means the file worker has
queued it, not that installation succeeded. Check the result separately:

```lua
= gd.mods_install_status().state
= gd.mods_install_status().message
```

`gd.mods_install_status()` returns a table with `state` (`idle`, `working`,
`installed`, or `failed`), `path`, `message`, and `restart`. Only one local
installation can be pending at a time. A partial filesystem failure can require
restart even when the state is `failed`; read the message. The log also records
the result. Calling this API alone starts no Nucleus network workers.

Lua mods call `gd.mods_install_zip(path)` without the console's leading `=`.
They need `gameplay: true`, and the game must be offline, just like other
gameplay-changing APIs. The worker rechecks offline state before publishing each
staged mod.

## Supported packages and limits

- MexManager fighters and custom stages, plain costume DATs, and supported
  stage/effect/menu/audio disc overrides use the existing Nucleus validators,
  staging writers, and conflict rules. Enabling an override turns off enabled
  mods targeting the same disc file.
- A ZIP containing multiple fighter/stage packages, mixed costumes and disc
  overrides, generic `mod.json`/Lua bundles, or unsupported files is refused.
  README and artwork files are ignored for plain costume/override packages.
- ZIP paths are validated before selection. Traversal, absolute/stream paths,
  symlinks, encrypted ZIPs, duplicate names, unsupported compression, and file
  types outside the shared allow-list are refused. Extracted files get generated
  destination paths; archive paths are never used as output paths. Extracted
  payloads must pass their ZIP CRC checks.
- Limits: 160 MiB ZIP, 64 MiB per entry, 200 MiB declared expanded entries, 2,048
  entries. Existing asset-specific limits also apply. Fighters additionally have
  a 200 MiB combined fighter/animation/costume output cap, including nested ZIPs
  and repeated costume references.
- At most 31 custom fighter mods can be installed, counting local, Nucleus, and
  manually added `mexfighter.txt` folders, including disabled ones. Reinstalling
  the same package does not consume an extra slot. To free a local fighter slot,
  close the game and remove its `local-<hash>-fighter` folder; merely disabling it
  does not free a slot.

Installed manifests say `source: "local file"`, with no Nucleus post ID. Mod sync
reports a missing one as **you need this mod** and cannot download it for a peer.
Both players must obtain and install the same ZIP themselves.
