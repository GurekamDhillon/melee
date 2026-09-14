# Dependencies

What the port builds against, where each piece comes from, and its licence.

## Vendored in this repository

| Dependency | Where | Licence |
|---|---|---|
| [encounter/aurora](https://github.com/encounter/aurora) (GC/Wii SDK reimplementation) | `extern/aurora` — base commit `749d6ee7a22bdfab78c8ece9047bca5d79aa72ca`, seven port patches (see [`extern/aurora/PORT_PATCHES.md`](extern/aurora/PORT_PATCHES.md)) | MIT |
| Nintendo Dolphin SDK / MSL / MetroTRK | `extern/dolphin`, `src/MSL`, `src/Runtime`, `src/MetroTRK` (part of the upstream decompilation) | proprietary SDK sources, upstream |

`extern/aurora` is a plain vendored copy, not a git submodule, so a clone contains
the patched Aurora directly.

## Fetched at build time

Aurora's CMake pins and fetches these (see
`extern/aurora/cmake/AuroraDependencyVersions.cmake`):

| Dependency | Pin | Licence |
|---|---|---|
| [encounter/dawn](https://github.com/encounter/dawn) (WebGPU) | commit `1155e0ed531126f33a1279afa029349651ca1c93`, release tag `v20260807.225922` | Apache-2.0 |
| SDL3 | `3.4.10` (prebuilt `encounter/sdl3-build` `v3.4.10`, Windows x86) | zlib |

Transitive, pulled by the Aurora/Dawn CMake: fmt (MIT), freetype (FTL/GPLv2),
libpng (libpng), zlib (zlib), zstd (BSD), xxhash (BSD), imgui (MIT), sqlite3
(public domain), tracy (BSD), googletest (BSD), abseil (Apache-2.0).

## Toolchain (not redistributed)

- clang/LLVM 23 — PowerPC front-end and i686 codegen.
- MSVC Build Tools — linker (`link.exe`) and `vcvarsall`.
- CMake + Ninja — for the Aurora/Dawn build.
- Python 3 (standard library only) — `pc/`-adjacent tooling and the replay harness.

## Derived code

`pc/gameworld/gekko_fp.c` reproduces the Gekko `frsqrte`/`fres` estimate tables
from Dolphin Emulator's `Common/FloatUtils.cpp` (GPL-2.0-or-later). The file
carries an SPDX and attribution header; it is the only GPL-derived source in `pc/`.

## Not dependencies

The workspace's `dusklight/`, `tp/`, `nod/` and `dawn/` checkouts are reference
material only. The build uses none of them — the port's Aurora is vendored under
`extern/aurora`, and the bootstrap build (`pc/build/build_aurora_x86.bat`) points
there too.
