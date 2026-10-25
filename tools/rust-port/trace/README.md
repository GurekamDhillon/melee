# Call trace of the C build

Which game C functions does the C build run in a scenario, and when is each first called?

* `gwtool_trace.sh` is a drop-in for `gwtool.exe` (`GW_GWTOOL`). It adds the LLVM attribute
  `instrument-function-entry-inlined=__cyg_profile_func_enter` to the TU's bitcode; gwtool's O2
  pipeline then inserts the entry hook in every function (inlined ones too, hook placed before inlining).
* `pc/platform/gw_calltrace.c` defines the hook. It is compiled with the shims but is **not** in the
  curated link list, so the normal exe never contains it (`gw_script.c` defaults the per-tick frame
  report `gw_calltrace_set_frame` to a no-op with `/alternatename`).
* `build_trace.sh` builds in a private root (default `_build/agents/trace`) with both of the above.
* Run with `MELEE_CALLTRACE=<raw file>` in addition to the golden recording environment (`record.sh`).
* `calltrace.py` resolves addresses through `melee-pc.map` and classifies the Rust status.

```
bash tools/rust-port/trace/build_trace.sh
MELEE_CALLTRACE=<raw> <golden env of pad_falcon_fox_fd> bash tools/port/run.sh ...
python tools/rust-port/trace/calltrace.py --trace <raw> --map $GW_BUILD_ROOT/melee-pc.map \
   --tus "<workspace>/_build/masstest/files.txt" --out docs/rust-port/trace/pad_falcon_fox_fd.csv --summary
```
Status definitions are in the `calltrace.py` docstring.
