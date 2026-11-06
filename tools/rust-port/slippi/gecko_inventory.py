#!/usr/bin/env python3
"""Inventory of a Slippi Dolphin Gecko INI (e.g. Data/Sys/GameSettings/GALE01r2.ini).

Lists each Gecko code group ($Name) and, inside it, every code entry with its
Gecko opcode, target address and size. Entries are named by the "#source.asm"
comment the slippi-ssbm-asm generator puts on an entry's first line; lines
without a name inherit the previous one (multi-line 04/C2 runs of one source).

No network, no disc data: it only reads the INI given on the command line.

usage: python -I gecko_inventory.py GALE01r2.ini [--group SUBSTR] [--md]
"""
import re
import sys

OPS = {
    0x00: "write8", 0x02: "write16", 0x04: "write32", 0x06: "string",
    0x08: "slider", 0x20: "if32==", 0x22: "if32!=", 0x28: "if16==",
    0xC0: "exec", 0xC2: "inject", 0xC6: "branch", 0xE0: "endif", 0xF2: "inject?",
}


def parse(path):
    groups, cur, section = [], None, None
    for raw in open(path, encoding="utf-8", errors="replace"):
        line = raw.rstrip("\n")
        if line.startswith("["):
            section = line.strip()
            continue
        if section != "[Gecko]":
            continue
        if line.startswith("$"):
            cur = {"name": line[1:].strip(), "entries": []}
            groups.append(cur)
            continue
        if cur is None:
            continue
        m = re.match(r"^([0-9A-Fa-f]{8}) ([0-9A-Fa-f]{8})\s*(#.*)?$", line)
        if not m:
            continue
        w0, w1, comment = int(m.group(1), 16), int(m.group(2), 16), m.group(3)
        cur.setdefault("pending", 0)
        if cur["pending"] > 0:  # inside a C2/06 payload
            cur["pending"] -= 1
            continue
        op = (w0 >> 24) & 0xFE
        addr = 0x80000000 | (w0 & 0x01FFFFFF)
        name = comment[1:].strip() if comment else None
        if name is None and cur["entries"]:
            name = cur["entries"][-1]["name"]
        ent = {"op": OPS.get(op, "%02X" % op), "addr": addr, "name": name or "?", "lines": 1}
        if op in (0xC0, 0xC2):
            cur["pending"] = w1
            ent["lines"] += w1
        elif op == 0x06:
            cur["pending"] = (w1 + 7) // 8
            ent["lines"] += cur["pending"]
        cur["entries"].append(ent)
    return groups


def main():
    args = sys.argv[1:]
    if not args:
        print(__doc__)
        return 2
    path, flt, md = args[0], None, "--md" in args
    if "--group" in args:
        flt = args[args.index("--group") + 1]
    for g in parse(path):
        if flt and flt.lower() not in g["name"].lower():
            continue
        # one row per named source: its op kinds and addresses
        rows = {}
        for e in g["entries"]:
            r = rows.setdefault(e["name"], {"ops": [], "lines": 0})
            r["ops"].append("%s@%08X" % (e["op"], e["addr"]))
            r["lines"] += e["lines"]
        print(("### " if md else "== ") + g["name"] + " (%d entries)" % len(g["entries"]))
        for name, r in rows.items():
            ops = r["ops"] if len(r["ops"]) <= 4 else r["ops"][:4] + ["... +%d" % (len(r["ops"]) - 4)]
            if md:
                print("| `%s` | %s | %d |" % (name, ", ".join(ops), r["lines"]))
            else:
                print("  %-70s %s" % (name, " ".join(ops)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
