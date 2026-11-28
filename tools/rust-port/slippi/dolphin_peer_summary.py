#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Summary of one dolphin_peer.sh set: both sides' desync verdicts, the offline compare, rollbacks, disconnects, and
the network check.

    python -I dolphin_peer_summary.py OUTDIR [--games N] [--tsv]

Our side (ours.log): per game the match's agreed scene, "N final checksums, C compared with the peer: S soft, H HARD
desyncs", stale-checksum lines, the rollback total of the last status line, and any link-down / disconnect line.
Dolphin's side: d_checksums.txt (SLIPPI_CHECKSUM_LOG: every pad packet's (checksum frame, checksum), TX = its own,
RX = ours) split into games where the frame number restarts. Its verdict is recomputed by the ASM rule
(StartEngineLoop.asm: frames <= 84 skipped; low halves as s16 more than 1 apart = Hard, else a differing word = Soft):
the game's own compare is in the ASM and logs nothing, so d_peer.txt's REPORT line (end method 7, winner -2 = ended on
a Hard) is its own record of a Hard. Dolphin's rollbacks: d_peer.txt LOAD lines (savestate loads) per game, and the
.slp's repeated frames (slp_rollbacks.py). Offline: our finalized checksums (ours*.csv) against Dolphin's TX, every
common frame. Network: netwatch.txt must show only loopback TCP peers and loopback-bound UDP endpoints.
"""
import glob
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))


def s16(v):
    v &= 0xFFFF
    return v - 0x10000 if v & 0x8000 else v


def verdict(a, b):
    """'eq', 'soft' or 'hard' for two checksum words, by the ASM rule."""
    if a == b:
        return "eq"
    return "hard" if abs(s16(a) - s16(b)) > 1 else "soft"


def dolphin_games(path):
    """[{'tx': {cf: cs}, 'rx': {cf: cs}}] split where the pad frame restarts (a new game starts at frame 1)."""
    games, cur, last = [], None, None
    if not os.path.exists(path):
        return games
    for ln in open(path, errors="replace"):
        p = ln.split()
        if len(p) < 8 or p[4] != "cf":
            continue
        d, frame, cf, cs = p[0], int(p[3]), int(p[5]), int(p[7], 16)
        if cur is None or (last is not None and frame < last - 60):
            cur = {"tx": {}, "rx": {}}
            games.append(cur)
        last = frame
        if cf > 1:
            cur["tx" if d == "TX" else "rx"][cf] = cs
    return games


def ours_csvs(o):
    out = []
    first = os.path.join(o, "ours.csv")
    if os.path.exists(first):
        out.append(first)
    for g in range(2, 20):
        p = os.path.join(o, f"ours-g{g}.csv")
        if os.path.exists(p):
            out.append(p)
    res = []
    for p in out:
        m = {}
        for ln in open(p).read().split("\n")[1:]:
            if ln:
                f, c = ln.split(",")
                m[int(f)] = int(c, 16)
        res.append(m)
    return res


def peer_log(path):
    games, cur = [], None
    if not os.path.exists(path):
        return games
    for ln in open(path, errors="replace"):
        if ln.startswith("GAME_START"):
            cur = {"loads": 0, "report": None}
            games.append(cur)
        elif ln.startswith("LOAD") and cur is not None:
            cur["loads"] += 1
        elif ln.startswith("REPORT"):
            kv = dict(zip(ln.split()[1::2], ln.split()[2::2]))
            if cur is None:
                cur = {"loads": 0, "report": None}
                games.append(cur)
            cur["report"] = kv
    return games


def ours_games(path):
    txt = open(path, errors="replace").read() if os.path.exists(path) else ""
    lines = txt.splitlines()
    games, cur = [], None
    issues = []
    for ln in lines:
        m = re.search(r'match agreed after [0-9.]+s: "([^"]*)"', ln) or re.search(r"(Slippi match started.*)", ln)
        if m:
            if cur is not None and cur.get("finals") is None and "scene" in cur and not cur["scene"].startswith("Slippi"):
                cur["scene2"] = m.group(1)
                continue
            cur = {"scene": m.group(1), "finals": None, "compared": None, "soft": None, "hard": None, "stale": 0,
                   "rollbacks": None, "maxdepth": None, "slow": None}
            games.append(cur)
            continue
        if cur is None:
            continue
        m = re.search(r"Slippi: (\d+) final checksums, (\d+) compared with the peer: (\d+) soft, (\d+) HARD desyncs; "
                      r"tick work max (\d+) us, (\d+) of (\d+) ticks over 16 ms", ln)
        if m:
            cur.update(finals=int(m.group(1)), compared=int(m.group(2)), soft=int(m.group(3)), hard=int(m.group(4)),
                       slow=int(m.group(6)))
        if "stale remote checksum" in ln:
            cur["stale"] += 1
        m = re.search(r"rollbacks [0-9.]+/s \(total (\d+), max (\d+) frames\)", ln) or             re.search(r"final hashes \(through game frame \d+\), rollbacks (\d+) \(max depth (\d+)", ln)
        if m:
            cur["rollbacks"], cur["maxdepth"] = int(m.group(1)), int(m.group(2))
        m = re.search(r"stalls (\d+), waits", ln)
        if m:
            cur["stalls"] = int(m.group(1))
    for ln in lines:
        if re.search(r"link is down|opponent disconnected|opponent left|taken over \((Disconnected|Failed)\)|"
                     r"could not start|no clean compare|DISCONNECT|desync|DESYNC|panicked|error", ln, re.I) and \
                "0 soft, 0 HARD" not in ln and "MELEE_DESYNC" not in ln:
            issues.append(ln.strip()[:240])
    return games, issues


def slp_rollbacks(path):
    try:
        r = subprocess.run([sys.executable, "-I", os.path.join(HERE, "slp_rollbacks.py"), path], capture_output=True,
                           text=True, timeout=120)
        m = re.search(r"rollbacks (\d+) max_depth (\d+)", r.stdout)
        return (int(m.group(1)), int(m.group(2))) if m else None
    except Exception:
        return None


def netcheck(path, dolphin_pid):
    """(rows, problems, wildcard UDP binds of our build). A TCP peer must be loopback (a connected socket's extra
    "0.0.0.0:0 Bound" row is Windows bookkeeping); every Dolphin UDP endpoint must be bound to 127.0.0.1, so it cannot
    send off the machine. Our build's UDP socket may be bound to 0.0.0.0 (it only sends to the addresses the loopback
    mock gave it, and asks no STUN): listed apart."""
    bad, wild, n = [], set(), 0
    if not os.path.exists(path):
        return None, ["no netwatch.txt"], wild
    for ln in open(path, errors="replace"):
        p = ln.split()
        if len(p) < 5:
            continue
        n += 1
        pid = p[2]
        if p[3] == "TCP":
            remote = p[6] if len(p) > 6 else ""
            state = p[7] if len(p) > 7 else ""
            rip = remote.rsplit(":", 1)[0]
            if rip in ("127.0.0.1", "::1") or (rip in ("0.0.0.0", "::") and state in ("Bound", "Listen")):
                continue
            bad.append(ln.strip())
        elif p[3] == "UDP":
            lip = p[4].rsplit(":", 1)[0]
            if lip in ("127.0.0.1", "::1"):
                continue
            if pid == dolphin_pid:
                bad.append(ln.strip())
            else:
                wild.add(f"pid {pid} UDP {p[4]}")
    return n, bad, wild


def main():
    o = sys.argv[1]
    want = int(sys.argv[sys.argv.index("--games") + 1]) if "--games" in sys.argv else None
    og, issues = ours_games(os.path.join(o, "ours.log"))
    dg = dolphin_games(os.path.join(o, "d_checksums.txt"))
    pg = peer_log(os.path.join(o, "d_peer.txt"))
    oc = ours_csvs(o)
    dslp = sorted(glob.glob(os.path.join(o, "dslp", "*.slp")), key=os.path.getmtime)
    args = open(os.path.join(o, "args.txt")).readline().strip() if os.path.exists(os.path.join(o, "args.txt")) else o
    print(f"== {args}")
    tot = dict(games=0, compared=0, o_soft=0, o_hard=0, o_stale=0, d_soft=0, d_hard=0, off=0, off_bad=0, o_rb=0, d_rb=0)
    n = max(len(og), len(dg), len(pg))
    for i in range(n):
        g = og[i] if i < len(og) else {}
        d = dg[i] if i < len(dg) else {"tx": {}, "rx": {}}
        pl = pg[i] if i < len(pg) else {"loads": None, "report": None}
        # Dolphin's view: our reported checksums (RX) against its own (TX)
        # only up to the frame Dolphin's game ended on (its report), where it stops simulating
        endf = int(rep0["frames"]) if (rep0 := (pl.get("report") or {})).get("frames") else 1 << 30
        dv = [verdict(d["tx"][f], c) for f, c in d["rx"].items() if 84 < f <= endf and f in d["tx"]]
        ds, dh = dv.count("soft"), dv.count("hard")
        # offline: our finalized checksums against Dolphin's own, every common frame
        ours = oc[i] if i < len(oc) else {}
        common = sorted(f for f in set(ours) & set(d["tx"]) if f <= endf)
        offv = [verdict(d["tx"][f], ours[f]) for f in common]
        offbad = [f for f, v in zip(common, offv) if v != "eq"]
        rb = slp_rollbacks(dslp[i]) if i < len(dslp) else None
        rep = pl.get("report") or {}
        print(f"  game {i + 1}: scene \"{g.get('scene', '-')}\"")
        print(f"    ours: {g.get('finals')} finals, {g.get('compared')} compared live, soft {g.get('soft')}, hard {g.get('hard')}, "
              f"stale {g.get('stale')}, rollbacks {g.get('rollbacks')} (max {g.get('maxdepth')}), stalls {g.get('stalls')}, slow ticks {g.get('slow')}")
        print(f"    dolphin: {len(dv)} compared (RX vs TX), soft {ds}, hard {dh}; savestate loads {pl.get('loads')}; "
              f".slp rollbacks {rb[0] if rb else '-'} (max {rb[1] if rb else '-'}); report end_method {rep.get('end_method', '-')} "
              f"winner {rep.get('winner', '-')} frames {rep.get('frames', '-')}")
        print(f"    offline: {len(common) - len(offbad)}/{len(common)} of our finals equal Dolphin's"
              + (f"; first differing frames {offbad[:5]}" if offbad else ""))
        tot["games"] += 1 if g.get("finals") is not None or d["tx"] else 0
        tot["compared"] += len(common)
        tot["off"] += len(common)
        tot["off_bad"] += len(offbad)
        for k, v in (("o_soft", g.get("soft")), ("o_hard", g.get("hard")), ("o_stale", g.get("stale")),
                     ("o_rb", g.get("rollbacks")), ("d_rb", pl.get("loads"))):
            tot[k] += v or 0
        tot["d_soft"] += ds
        tot["d_hard"] += dh
    dpid = ""
    ap = os.path.join(o, "args.txt")
    if os.path.exists(ap):
        m = re.search(r"dolphin pid (\d+)", open(ap).read())
        dpid = m.group(1) if m else ""
    nn, bad, wild = netcheck(os.path.join(o, "netwatch.txt"), dpid)
    disc = [x for x in issues if re.search(r"link is down|disconnect|left|taken over|could not start", x, re.I)]
    print(f"  issues in ours.log ({len(issues)}):")
    for x in issues[:12]:
        print(f"    {x}")
    drive = open(os.path.join(o, "drive.log"), errors="replace").read().strip().splitlines() if os.path.exists(os.path.join(o, "drive.log")) else []
    print(f"  drive: {drive[-1] if drive else '-'}")
    print(f"  network: {nn} socket rows, {len(bad)} not loopback" + ("".join(f"\n    {b}" for b in bad[:5]))
          + (f"; our build's wildcard UDP binds: {', '.join(sorted(wild))}" if wild else ""))
    ok = (want is None or tot["games"] >= want) and tot["o_hard"] == 0 and tot["d_hard"] == 0 and tot["off_bad"] == 0 \
        and not disc and not bad and tot["o_soft"] == 0 and tot["d_soft"] == 0
    row = (f"ROW\tgames {tot['games']}/{want if want else '-'}\tframes compared {tot['compared']}\t"
           f"ours soft {tot['o_soft']} hard {tot['o_hard']} stale {tot['o_stale']}\tdolphin soft {tot['d_soft']} hard {tot['d_hard']}\t"
           f"offline differing {tot['off_bad']}\trollbacks ours {tot['o_rb']} dolphin {tot['d_rb']}\tdisconnects {len(disc)}\t"
           f"non-loopback sockets {len(bad)}\t{'PASS' if ok else 'FAIL'}")
    print(row)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
