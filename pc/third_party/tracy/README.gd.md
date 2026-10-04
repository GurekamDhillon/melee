# Optional development profiler client

Client files only, copied from the already present Aurora CMake Tracy dependency
`_build/ax86m/_deps/tracy-src/public`, 2026-10-03. Upstream:
https://github.com/wolfpld/tracy. The preserved BSD-3-Clause licence is `LICENSE`;
libbacktrace carries its own copyright/licence headers. No server/UI is bundled.

Enable a development build with `GW_PROF_TRACY=1 bash tools/port/build.sh`.
`portlib.sh` defines `GW_PROF_TRACY`, `TRACY_ENABLE`, `TRACY_ON_DEMAND` for
`gw_profiler.c` and `gw_profiler_tracy.cpp`. The latter compiles only
`public/TracyClient.cpp`; both objects are in the curated response file. Existing
Windows SDK libraries provide sockets/debug/system APIs. Default builds compile
an empty client wrapper and perform no Tracy initialization. The flag is in the
object content key, so switching back does rebuild these objects.

`GW_RELEASE_BUILD=1` refuses this flag. Release packaging must reject the embedded
`GD_MELEE_TRACY_DEVELOPMENT_ONLY` marker even for an existing development EXE.
Do not enable Tracy for overhead comparisons: its queues/allocations/worker and
network activity are additional to the bounded builtin profiler.

Native zones call the C client begin/end API using persistent source locations,
numeric details use zone values, counters use plots, frames emit frame marks.
GPU samples supplied by Aurora are report durations plus callback-arrival counter
tracks carrying source render-frame IDs; this adapter does not invent GPU execution
timestamps. The compatible Tracy viewer is obtained
separately. No Tracy/client/game build was run in this source-only pass.
