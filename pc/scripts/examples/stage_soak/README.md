In-game fixture for mid-match stage switching (heap refusals and leaks). Mount it, start a LAB match on Final Destination with `p1=fox;p2=marth/cpu0` (add `cpus=idle`), then in the console:

    soakpre demo          preload the five stages in the demo's order (fd, bf, ys, dl, fod)
    soak 200 20261004     200 seeded pseudo-random switches (immediate back-and-forth and consecutive large stages included; every 50th lands on FD)
    soakpath fod,fd,bf    an explicit path
    soaktrack / soakdump end   allocation tracking, then heapdump_<n>.txt in the run folder (gd.stage_slots(2) / gd.stage_slots(true))

The log ends with `soak DONE n=200 ok=200 refused=0 ...` and prints the free heap at each 50th switch (FD checkpoints). Acceptance: refused=0 and the FD heap numbers stay within a few hundred bytes of the first lap. Heap refusals are counted apart from preflight retries (a fighter captured/dead/respawning).
