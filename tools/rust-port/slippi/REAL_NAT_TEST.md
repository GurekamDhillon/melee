# Real NAT test: our engine across two networks

Every online test so far ran on one machine. The server saw the same external IP for both players, so the game
traffic used LAN addresses and never crossed a NAT. This test runs one side on another network, so the real hole
punch gets used.

**This uses the real network:** Slippi's production matchmaking (`mm.slippi.gg:43113`), then a direct UDP connection
between the two networks. Direct mode only. The engine refuses ranked.

- **Machine A:** the Windows workstation (Git Bash), account 1 (`_build/dolphin-netplay/slippi-acct-1/user.json`).
- **Machine B:** the Linux laptop on a phone hotspot, account 2 (`slippi-acct-2/user.json`).

Copy each `user.json` to the machine that uses it. Never paste it into chat, print it or commit it. Delete the copies
afterwards.

## 1. Build on machine B (Linux)

**A native Linux build of the Rust port has never been run.** Section 4 lists what is likely to break.

System packages (Debian/Ubuntu names; Fedora names in brackets):

```
sudo apt install build-essential pkg-config git python3 curl \
  libasound2-dev libudev-dev \
  libvulkan1 mesa-vulkan-drivers vulkan-tools \
  libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev libwayland-dev
# [gcc make pkgconf-pkg-config alsa-lib-devel systemd-devel vulkan-loader mesa-vulkan-drivers
#  libX11-devel libXcursor-devel libXrandr-devel libXi-devel libxkbcommon-devel wayland-devel]
curl https://sh.rustup.rs -sSf | sh   # stable toolchain
```

What needs each package:
- `cpal` (audio) needs `libasound2-dev`.
- `gilrs` (gamepads) needs `libudev-dev`.
- `ring`, for the `real-net` TLS, needs a C compiler.
- `wgpu` needs a Vulkan driver at run time. Check with `vulkaninfo --summary`.
- `winit` needs the X11/Wayland libraries.

Then, from `rust/` in a checkout of branch `port/slippi7`:

```
export CARGO_TARGET_DIR="$PWD/../build/cargo"
cargo build --release -p melee --features real-net
ls ../build/cargo/release/melee        # the binary: melee (no .exe)
```

Disc image: put the vanilla NTSC 1.02 image on the laptop and `export GW_ISO_VANILLA=/path/to/melee.iso`. Never commit
it.

Smoke test before going online. This is offline, with no network:

```
MELEE_VOLUME=3 ./build/cargo/release/melee slippi-replay --help 2>&1 | head -2   # prints the usage line
```

If you have an `engine.trace` and `engine.log` from any earlier run on A, `melee slippi-replay --trace T --log L --out x.csv`
proves the sim boots and runs on Linux.

## 2. Run (both machines within about a minute of each other)

From the checkout root:

```
# machine A (Windows, Git Bash), account 1, searching for account 2's code
bash tools/rust-port/slippi/real_nat_side.sh ~/nat/outA ~/nat/acct1.json CODE#2

# machine B (Linux, bash), account 2, searching for account 1's code
GW_ISO_VANILLA=/path/to/melee.iso bash tools/rust-port/slippi/real_nat_side.sh ~/nat/outB ~/nat/acct2.json CODE#1
```

The script picks `melee` or `melee.exe` itself (`EXE=` overrides it). Each side plays about 40 s of scripted random
pads (`FRAMES=2400`), then exits. Exit code 0 means no desync on that side's live check. Other options:
- `SEED`: pad seed.
- `CHAR`: character id. The default is Fox.
- `PORT`: UDP port. The default is random. No port forwarding is needed.
- `-- --lag-ms 40 --loss-pct 5`: extra degradation on top of the real link.

## 3. Compare and report

Copy `outB/engine.csv` to A, then run:

```
bash tools/rust-port/slippi/real_nat_side.sh --compare ~/nat/outA/engine.csv ~/nat/outB/engine.csv
```

The expected result is `differing 0` with about 2400 common frames. Send back both `engine.log` files and the
compare line. The logs hold no account fields, but they do hold IP addresses, so share them privately. Then delete
the `user.json` copies.

If it does not connect:
- **Stops after `matched` with no `connected`.** The punch failed. Phone carriers often use symmetric/CGNAT NAT, and
  Slippi Dolphin fails the same way there. Try the laptop on another network, such as home Wi-Fi at a friend's or a
  different hotspot.
- **`server error: ...`.** The message comes from the matchmaking server. Check that each side searches for the
  *other* account's code.

## 4. Likely Linux trouble spots (checked from Windows only, not built)

A `cargo check --target x86_64-unknown-linux-gnu` from Windows stopped at the C build scripts (`alsa-sys`,
`libudev-sys`, `ring`), so the Rust code itself was not type-checked for Linux. What a read of the code shows:

1. **`melee_engine::cow` had no non-Windows build.** `pub use imp::*` was `cfg(windows)` only, but `gcm.rs`,
   `snapshot.rs` and `melee/src/main.rs` call `cow::active()` and its siblings unconditionally. Fixed on this branch
   with an inert `cfg(not(windows))` stub. `active()` is false and `protect()` returns an error.
2. **The rollback snapshot base** (`snapshot.rs`, around `cow::protect(...).expect("arena copy-on-write")`) panics if
   that path is reached off Windows. `arena.rs` has `cfg(not(windows))` fallbacks, and the snapshot path should
   follow `arena::active()`. If netplay panics there, take the full-copy path when `cow::active()` is false.
3. **Win32-only code that is gated but has no Linux equivalent:**
   - the crash handler (`melee_engine/src/crash.rs`, `MELEE_CRASH_TRACE`);
   - the guard allocator and the arena-misuse backtraces (`melee/src/main.rs`, `VirtualAlloc`,
     `RtlCaptureStackBackTrace`);
   - the corev1 `user32` calls;
   - DXGI frame statistics (`melee_platform/src/gfx.rs`, VRR detection);
   - `pagewatch.rs` and `lowmem.rs`.

   Each has a `cfg(not(windows))` side, but check that it compiles: these branches have never been built.
4. **`windows` / `windows-sys`** are `[target.'cfg(windows)'.dependencies]` in `melee_platform` and `melee_data`. Any
   `use windows...` outside a `cfg(windows)` item breaks the Linux build. Grep for it if `cargo build` complains.
5. **Paths.** `melee/src/diff.rs` defaults `GW_WORKSPACE` to `E:/Projects/Melee Workspace`. It is not used by
   `slippi-netplay`, but set `GW_WORKSPACE` if a tool needs it. The crash log looks for `USERPROFILE` and then `HOME`.
6. **Memory.** The engine reserves a large arena (the game's decoded archives, several GB of address space). On a
   laptop with little RAM, keep other programs closed. If allocation fails, look at the `arena.rs` non-Windows path
   (mmap and overcommit).
7. **Graphics.** `slippi-netplay` opens a wgpu surface. With no working Vulkan (`vulkaninfo` fails), try
   `WGPU_BACKEND=gl`.
8. **Audio.** cpal on ALSA. With no sound device, it should only log. `MELEE_VOLUME=3` is set by the script.
9. **Firewall.** Allow outbound UDP. `ufw` defaults are fine. The game uses one UDP port (`PORT`) both for the
   matchmaking connection and for the game.
