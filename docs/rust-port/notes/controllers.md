# Controllers in the Rust build

Status: 2026-11-04, branch `port/pads`. Our own implementation; Hero's melee-unlocked was used only to
see which devices and features matter (unlocked-survey.md, "Switch Pro / DS4 / DualSense / generic HID").

## Path

Every source ends as a `PADStatus` in `PadSystem::read` (rust/melee_platform/src/pad_read.rs), in this order:

1. Aurora layer: keyboard bindings plus gilrs gamepads (`aurora.rs`, `gamepads.rs`);
2. launcher remaps (`bindings.rs`, `MELEE_PORT_SETTINGS`);
3. **controller profiles** (`profiles.rs`, `socd.rs`; new): a port whose gamepad has a profile is
   rebuilt from its raw state by the profile instead of Aurora's arithmetic;
4. GameCube adapter (a controller in the adapter wins its port);
5. pad script (`MELEE_PAD_SCRIPT`), then the reset combo.

Profiles run before the sim sees anything, so replays, the pad script, rollback and netplay see
plain PAD bytes. SOCD state is per port, input side only. With no profile options set nothing in
step 3 runs and the input is exactly what it was before.

## Devices

| device | how | notes |
|---|---|---|
| GameCube adapter (WUP-028) | `adapter/` (existing port of `gc_adapter.c`), hotplug scanner | Not changed. Its unit and pad_system tests pass. A PhOB or any GC controller in the adapter goes this way. |
| Xbox / XInput | gilrs | family `xinput` (vendor 0x045e or name) |
| Switch Pro | gilrs (SDL mapping db) | `switch_pro` (057e:2009); ZL/ZR are digital, read as full travel plus L/R |
| DS4 / DS5 | gilrs | `ds4` (054c:05c4/09cc/0ba0), `ds5` (054c:0ce6/0df2) |
| PhOB in USB mode | gilrs, by name | `phob`: no deadzone added (the firmware calibrates) |
| box controllers | gilrs, by name (B0XX, Frame1, GP2040, Haybox, hitbox) or a forced profile | `box`: digital stick, ModX/ModY, light/mid shield, SOCD |
| anything else | gilrs | `generic`: no built-in profile, Aurora default |

gilrs was kept rather than adding SDL: it is already the platform's backend and on Windows reads
WGI/XInput and HID pads with the SDL mapping database. Hotplug: gilrs connect/disconnect events
reassign ports in connection order (existing), the profile is chosen again every frame from the
device on the port, and a port's SOCD history resets when its pad goes away.

## Config file and flags

`--controllers <file>` (or `MELEE_CONTROLLERS=<file>`) loads a file; `--controllers default` loads only
the built-in family profiles. `--pad-profile <port>=<name>` forces a profile on a port; `--socd <mode>`
overrides every profile's SOCD mode. Bad lines are reported as `controllers: line N: ...` and skipped.
The format is documented at the top of `profiles.rs`; example:

```ini
[profile mypro]
match = switch_pro          ; xinput switch_pro ds4 ds5 phob box generic
match_name = 8BitDo         ; or a name substring
mode = pad                  ; pad | box
stick_deadzone = 0.10       ; radial, rescaled
cstick_deadzone = 0.10
trigger_deadzone = 0.05
trigger_threshold = 0.85    ; analog L/R past this presses digital L/R; 0 = never
remap = South:B, East:A, Back:none
socd = 2ip_noreact          ; neutral | 2ip | 2ip_noreact | first
box = DpadLeft:Left, LeftShoulder:ModX, C:CUp

[port 2]
profile = mypro
```

Source names: South East West North Start Back Guide LeftShoulder RightShoulder LeftStick RightStick
DpadUp/Down/Left/Right LeftTrigger2 RightTrigger2 C Z. Targets: A B X Y Z L R Start DUp DDown DLeft
DRight none, and in box mode Left Right Up Down CLeft CRight CUp CDown ModX ModY LightShield MidShield.
File profiles win over built-ins for the same device. No UI: GD's Qt launcher will write this file
later; nothing was added to Hero's launcher.

Built-in values (port-side choices): deadzone xinput 0.08, ds4/ds5 0.06, switch_pro 0.12, phob 0,
C-stick at least 0.10 (phob 0); trigger threshold 0.85. Box stick (GC raw units): rim 80, diagonal
56/56, ModX 53 horizontal, (59,25) diagonal, ModY 43 vertical, (25,59) diagonal; light shield 49,
mid shield 94 on R analog. Default box layout: D-pad directions, LB ModX, Back ModY, rest standard.

## Tested

- Unit tests (`cargo test -p melee_platform`): family detection, radial deadzone and rescale,
  analog-to-trigger, Switch Pro digital triggers, remaps, box values, all four SOCD modes,
  config parsing with a bad line, port overrides, determinism of two independent states.
- Game runs only through `MELEE_PAD_SCRIPT` (gate, rematch css). Scripted runs force gamepads off,
  so profiles do not affect them.
- **No real hardware was used.** Nothing here was tried with a Switch Pro, DS4/DS5, PhOB, box or
  adapter; the USB ids and the gilrs mappings are untested on devices.

## Open

- Box mode reads only buttons gilrs names; a box exposing more than ~19 buttons needs raw HID.
- Keyboard as a box (SOCD on keys) is not wired.
- Gamepad rumble is still not implemented (adapter rumble is).
- Profiles do not apply to adapter ports (the adapter already has `gc_maps` remaps).
