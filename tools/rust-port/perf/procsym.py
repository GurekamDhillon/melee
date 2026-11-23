#!/usr/bin/env python3
"""Name the "slow proc" events of a frame profile (MELEE_PROFILE=1 .jsonl) from the executable's PDB (Windows, dbghelp).

  python procsym.py PROFILE.jsonl EXE [TICKS e.g. 4650-4670,5160-5170]

Each event's `value` is the GObj callback's offset in the executable (melee_runtime gobj.rs `slow_proc`). The PDB is
looked up next to EXE (cargo keeps melee.pdb in the target dir; pass that melee.exe)."""
import ctypes
import json
import os
import sys
from ctypes import wintypes as w


def names(exe, rvas):
    dh = ctypes.WinDLL("dbghelp.dll")
    h = wintypes_handle = ctypes.c_void_p(0x5EED)
    dh.SymSetOptions(0x2 | 0x4)  # UNDNAME | DEFERRED_LOADS
    dh.SymInitializeW(h, ctypes.c_wchar_p(os.path.dirname(os.path.abspath(exe))), False)
    dh.SymLoadModuleExW.restype = ctypes.c_uint64
    base = dh.SymLoadModuleExW(h, None, ctypes.c_wchar_p(os.path.abspath(exe)), None, ctypes.c_uint64(0x10000000),
                               0x10000000, None, 0)
    out = {}
    for r in rvas:
        buf = ctypes.create_string_buffer(88 + 1024)
        ctypes.memmove(buf, (88).to_bytes(4, "little"), 4)
        ctypes.memmove(ctypes.addressof(buf) + 80, (1000).to_bytes(4, "little"), 4)
        disp = ctypes.c_uint64(0)
        ok = dh.SymFromAddr(h, ctypes.c_uint64(base + r), ctypes.byref(disp), buf)
        if ok:
            n = int.from_bytes(buf.raw[76:80], "little")
            out[r] = buf.raw[84:84 + min(n, 1000)].decode("utf8", "replace")
        else:
            out[r] = hex(r)
    return out


def main():
    prof, exe = sys.argv[1], sys.argv[2]
    want = None
    if len(sys.argv) > 3:
        want = set()
        for part in sys.argv[3].split(","):
            a, _, b = part.partition("-")
            want.update(range(int(a), int(b or a) + 1))
    ev = []
    for l in open(prof):
        e = json.loads(l)
        if e.get("type") == "event" and e.get("text", "").startswith("slow proc") and (want is None or e["tick"] in want):
            ev.append(e)
    nm = names(exe, sorted({e["value"] for e in ev}))
    for e in ev:
        print(f"tick {e['tick']:>6} {e['ms']:6.2f} ms  {e['text']:28}  {nm[e['value']]}")


if __name__ == "__main__":
    main()
