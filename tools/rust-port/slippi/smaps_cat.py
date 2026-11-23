#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""One line of a process's resident memory by mapping kind, from /proc/PID/smaps (Linux, no root):
    python -I smaps_cat.py PID
Prints `threads=N heap=KB anon=KB gpu=KB memfd=KB so=KB file=KB stack=KB other=KB` (Rss) and `anon_big=KB` (the
anonymous mappings of 64 MB or more: arenas, snapshot buffers). The soak (soak_dolphin_linux.sh) logs it per minute."""
import re
import sys


def main():
    pid = sys.argv[1]
    cats = dict.fromkeys(["heap", "anon", "gpu", "memfd", "so", "file", "stack", "other"], 0)
    big = 0
    name, size = "", 0
    try:
        with open(f"/proc/{pid}/smaps") as f:
            for ln in f:
                if re.match(r"^[0-9a-f]+-[0-9a-f]+ ", ln):
                    p = ln.split(None, 5)
                    a, b = (int(x, 16) for x in p[0].split("-"))
                    size = b - a
                    name = p[5].strip() if len(p) > 5 else ""
                elif ln.startswith("Rss:"):
                    kb = int(ln.split()[1])
                    if name == "[heap]":
                        c = "heap"
                    elif name == "":
                        c = "anon"
                        if size >= 64 << 20:
                            big += kb
                    elif "/dev/nvidia" in name or "/dev/dri" in name or "drm" in name:
                        c = "gpu"
                    elif name.startswith("/memfd:"):
                        c = "memfd"
                    elif ".so" in name:
                        c = "so"
                    elif name.startswith("/"):
                        c = "file"
                    elif name.startswith("[stack"):
                        c = "stack"
                    else:
                        c = "other"
                    cats[c] += kb
        th = next((ln.split()[1] for ln in open(f"/proc/{pid}/status") if ln.startswith("Threads:")), "?")
    except OSError:
        print("gone")
        return
    print(f"threads={th} " + " ".join(f"{k}={v}" for k, v in cats.items()) + f" anon_big={big}")


if __name__ == "__main__":
    main()
