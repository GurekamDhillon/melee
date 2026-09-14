# Port build tooling

Windows-side scripts that produce `melee-pc.exe` from this tree plus the platform shims in
`pc/platform`. They are provided as they are used, and still assume the maintainer's workspace
layout:

- workspace root `C:\gdm` (WSL: `/mnt/c/gdm`)
- build directory `C:\gdm\_build`, with objects under `_build\masstest\out` and `_build\masstest\shimobj`
- Aurora + Dawn + SDL3 built into `C:\gdm\_build\ax86m`

`build_aurora_x86.bat` bootstraps Dawn and SDL3 into `_build\ax86` from the vendored Aurora
(`extern/aurora`); `build_aurora_melee.bat` then builds the port's Aurora into `ax86m` reusing them.
No other checkout is required. Third-party versions and licences: [`../DEPENDENCIES.md`](../DEPENDENCIES.md).

`ax86` is a persistent build directory, so if it was ever configured against a different Aurora
checkout CMake refuses the stale cache with "does not match the source ... used to generate cache".
Delete `_build\ax86\CMakeCache.txt` and `_build\ax86\CMakeFiles\` — keep `_deps`, which `ax86m`
reuses — and re-run. A fresh clone never sees this.

## Pipeline

1. Every game translation unit is compiled by `masstest/pipe_wsl.sh` (or `pipe_win.sh` under Git
   Bash): clang's PowerPC front-end emits LLVM IR, `gwtool.exe` byte-swaps every access and emits
   x86 COFF. `masstest/files.txt` is the manifest.
2. The native shims in `pc/platform` compile directly with clang (i686).
3. `build_melee_pc.bat` links everything with `melee_link_libs.rsp` (the object list is generated
   by the link step) and produces `melee-pc.exe`.

The full command reference, including the Aurora build and the run recipes, is
[`../docs/PORT_DEV_QUICKREF.md`](../docs/PORT_DEV_QUICKREF.md). `masstest/mapsym.sh` resolves a
crash RVA to a symbol via the link map.
