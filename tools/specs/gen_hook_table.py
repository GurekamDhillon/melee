#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 GurekamDhillon
"""Generate docs/specs/hooks-v1.tsv, the mod-c-v1 hook table.

Inputs (all from this repository, never from the disc):
  config/GALE01/symbols.txt       retail GameCube (GALE01 1.02) function addresses; the decomp names are the aliases
  rust/melee_link/linked_keys.txt keys of the Rust link table (cross-TU hooks); a name found there has a call-site seam
  src/**/*.h, src/**/*.c          prototypes, to derive the signature of each hook

Output: tab-separated, '#' lines are comments (skip them), one header row, then one row per hook:
  name      stable mod-facing hook id (never changes once published in a hooks-vN table)
  address   retail address, 0x%08X; THE KEY. Mods register by address, the name is for humans
  signature ABI signature  `ret(arg,...)`  with ABI types void i32 u32 f32 gd_fighter gd_item gd_gobj gd_spawn gd_ptr
  phase     phase family (see mod-c-v1.md section 3)
  alias     current decomp symbol (may be renamed upstream; the address stays)
  site      how an implementation intercepts it: call | proc | state | scene
  link      yes if the alias is a key of rust/melee_link (cross-TU seam exists), else no
  sig_src   decomp (prototype found in src/) | assumed (family default; hook restricted to void pre/post)

Usage: python tools/specs/gen_hook_table.py [--check]   (--check fails if the committed file is stale)
"""
import os
import re
import sys

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
SYMBOLS = os.path.join(ROOT, "config", "GALE01", "symbols.txt")
LINKED = os.path.join(ROOT, "rust", "melee_link", "linked_keys.txt")
OUT = os.path.join(ROOT, "docs", "specs", "hooks-v1.tsv")
TABLE_VERSION = 1

# --- the curated, named part of v1 (alias -> (stable name, phase, site)) -------------------------------------------
# Fighter gobj procs in the order Fighter_Create (src/melee/ft/fighter.c) registers them; slot = HSD_GObj_SetupProc priority.
FIGHTER_PROCS = [
    ("Fighter_8006A1BC", 0x00), ("Fighter_8006A360", 0x01), ("Fighter_8006ABA0", 0x02),
    ("Fighter_Spaghetti_8006AD10", 0x03), ("Fighter_procUpdate", 0x04), ("Fighter_procMap", 0x06),
    ("Fighter_8006C5F4", 0x07), ("Fighter_CallAcessoryCallbacks_8006C624", 0x08), ("Fighter_8006C80C", 0x09),
    ("Fighter_UnkProcessGrab_8006CA5C", 0x0C), ("Fighter_8006CB94", 0x0D), ("Fighter_ProcessHit_8006D1EC", 0x0E),
    ("Fighter_8006D9AC", 0x10), ("Fighter_UnkCallCameraCallback_8006D9EC", 0x12), ("Fighter_8006DA4C", 0x16),
]
CURATED = []  # (alias, name, phase, site)
for a, slot in FIGHTER_PROCS:
    CURATED.append((a, "fighter.proc%02X" % slot, "fighter.proc", "proc"))
CURATED += [
    ("Fighter_ProcessHit_8006D1EC", "hit.fighter_process_hit", "hit", "proc"),
    ("Fighter_TakeDamage_8006CC7C", "damage.fighter_take_damage", "damage", "call"),
    ("Fighter_UnkTakeDamage_8006CC30", "damage.fighter_take_damage_wrap", "damage", "call"),
    ("Item_80269CA0", "damage.item_take_damage", "damage", "call"),
    ("Fighter_UnkProcessDeath_80068354", "fighter.process_death", "fighter.life", "call"),
    ("Fighter_Create", "fighter.create", "fighter.life", "call"),
    ("Item_8026862C", "item.spawn", "item.spawn", "call"),
    ("Item_80268B18", "item.spawn_gobj", "item.spawn", "call"),
    ("Item_80268B5C", "item.spawn_gobj2", "item.spawn", "call"),
    ("Item_80268B9C", "item.spawn_void", "item.spawn", "call"),
    ("Item_802693E4", "item.update0", "item.update", "proc"),
    ("Item_80269978", "item.update5", "item.update", "proc"),
    ("Item_80268E5C", "item.change_state", "item.update", "call"),
    ("Item_802680CC", "item.destroy", "item.update", "call"),
    ("Ground_801C0C2C", "stage.update", "stage.update", "proc"),
    ("Ground_UpdateMapColl", "stage.update_map_coll", "stage.update", "call"),
    ("Ground_OnLoad", "stage.on_load", "stage.update", "call"),
    ("Ground_801C0FB8", "stage.on_start", "stage.update", "call"),
    ("gm_Scene_Vs_OnEnter", "match.vs_enter", "match.start", "scene"),
    ("gm_Scene_Training_OnEnter", "match.training_enter", "match.start", "scene"),
    ("gm_Scene_Vs_OnExit", "match.vs_exit", "match.end", "scene"),
    # logic iteration boundary: gm_801A4D34 calls HSD_GObj_RunProcs() once per logic iteration (NHV1 log point follows it)
    ("HSD_GObj_RunProcs", "frame.run_procs", "frame", "call"),
]
# frame.begin / frame.end are the pre / post of frame.run_procs; they are not separate rows (see the spec, section 3).

STATE_RE = re.compile(r"^ftCo_([A-Za-z0-9]+)_(Anim|IASA|Phys|Coll|Enter)$")
STATE_PHASE = {"Anim": "action.anim", "IASA": "action.iasa", "Phys": "action.phys", "Coll": "action.coll", "Enter": "action.enter"}
STATE_DEFAULT_SIG = {"Anim": "void(gd_fighter)", "IASA": "void(gd_fighter)", "Phys": "void(gd_fighter)",
                     "Coll": "void(gd_fighter)", "Enter": "void(gd_fighter)"}


def read_symbols():
    """alias -> address for every `type:function` line of symbols.txt (first definition wins)."""
    out = {}
    rx = re.compile(r"^(\S+)\s*=\s*\.text:0x([0-9A-Fa-f]+);.*type:function")
    with open(SYMBOLS, encoding="utf-8", errors="replace") as f:
        for line in f:
            m = rx.match(line)
            if m and m.group(1) not in out:
                out[m.group(1)] = int(m.group(2), 16)
    return out


def read_linked():
    out = set()
    with open(LINKED, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                out.add(line.rsplit("::", 1)[-1])
    return out


def load_sources():
    """All src/ .h then .c text (headers first so a prototype wins over a definition)."""
    hs, cs = [], []
    for d, _, files in os.walk(os.path.join(ROOT, "src")):
        for fn in files:
            p = os.path.join(d, fn)
            if fn.endswith(".h"):
                hs.append(p)
            elif fn.endswith(".c"):
                cs.append(p)
    texts = []
    for p in sorted(hs) + sorted(cs):
        with open(p, encoding="utf-8", errors="replace") as f:
            texts.append(f.read())
    return texts


def find_proto(texts, name):
    rx = re.compile(r"(?m)^[ \t]*(?:/\*[^*]*\*/\s*)?((?:(?:static|extern|inline)\s+)*[A-Za-z_][\w \t\*]*?)[ \t\*]\b%s\s*\(([^;{)]*)\)\s*[;{]" % re.escape(name))
    for t in texts:
        m = rx.search(t)
        if m:
            return m.group(1), m.group(2)
    return None


def abi_type(ctype, phase, first):
    t = re.sub(r"\b(const|static|extern|inline|struct|volatile)\b", "", ctype).strip()
    t = re.sub(r"\s+", " ", t)
    ptr = t.endswith("*")
    base = t.rstrip("* ").strip()
    if ptr:
        if base in ("Fighter_GObj", "Fighter"):
            return "gd_fighter"
        if base in ("Item_GObj", "Item"):
            return "gd_item"
        if base == "SpawnItem":
            return "gd_spawn"
        if base == "HSD_GObj":
            return "gd_item" if (phase.startswith("item") and first) else "gd_gobj"
        return "gd_ptr"
    if base == "void":
        return "void"
    if base in ("float", "f32"):
        return "f32"
    if base in ("u8", "u16", "u32", "unsigned int", "unsigned", "size_t"):
        return "u32"
    if base == "double" or base == "f64":
        return "f64"
    return "i32"  # s8 s16 s32 int bool enum_t and the decomp enums


def signature(texts, alias, phase, default):
    pr = find_proto(texts, alias)
    if not pr:
        return default, "assumed"
    ret, args = pr
    ret = ret.strip()
    args = args.strip()
    parts = [] if args in ("", "void") else [a.strip() for a in re.split(r",(?![^()]*\))", args.replace("\n", " "))]
    out = []
    for i, a in enumerate(parts):
        if a == "...":
            return default, "assumed"
        a = re.sub(r"\[[^\]]*\]", "*", a)          # array parameter -> pointer
        a = re.sub(r"\(\*\s*\w*\)\s*\(.*\)", "*", a)  # function pointer parameter
        m = re.match(r"(.*?)(\b\w+)?$", a.strip())
        ty = a
        if m and m.group(2) and (m.group(1).strip() != "") and not a.strip().endswith("*"):
            ty = m.group(1).strip()
            # the whole thing was a bare type like "float" when the group(1) would be empty; handled above
        elif a.strip().endswith("*"):
            ty = a.strip()
        elif m and m.group(2) and m.group(1).strip() == "":
            ty = m.group(2)
        # name stuck to a star: "Fighter* fp" -> m gave ("Fighter* ", "fp"); "Fighter *fp" -> ("Fighter *","fp")
        out.append(abi_type(ty, phase, i == 0))
    return "%s(%s)" % (abi_type(ret, phase, False), ",".join(out)), "decomp"


def main():
    sym = read_symbols()
    linked = read_linked()
    texts = load_sources()
    rows = []
    seen = set()

    def add(name, alias, phase, site, default):
        if alias not in sym:
            sys.exit("symbol %s missing from symbols.txt (curated list is stale)" % alias)
        addr = sym[alias]
        if (addr, name) in seen:
            return
        seen.add((addr, name))
        sig, src = signature(texts, alias, phase, default)
        rows.append((name, addr, sig, phase, alias, site, "yes" if alias in linked else "no", src))

    for alias, name, phase, site in CURATED:
        if alias == "Fighter_ProcessHit_8006D1EC" and name != "hit.fighter_process_hit":
            continue
        add(name, alias, phase, site, "void(gd_fighter)")
    # Fighter_ProcessHit is both slot 0x0E of the fighter procs and the hit entry; one address, one row: keep the hit name
    rows = [r for r in rows if not (r[4] == "Fighter_ProcessHit_8006D1EC" and r[0] == "fighter.proc0E")]
    for alias in sorted(sym):
        m = STATE_RE.match(alias)
        if m:
            kind = m.group(2)
            add("action.%s.%s" % (m.group(1), kind.lower()), alias, STATE_PHASE[kind], "state", STATE_DEFAULT_SIG[kind])

    order = {"frame": 0, "match.start": 1, "match.end": 2, "stage.update": 3, "fighter.life": 4, "fighter.proc": 5,
             "hit": 6, "damage": 7, "item.spawn": 8, "item.update": 9, "action.enter": 10, "action.anim": 11,
             "action.iasa": 12, "action.phys": 13, "action.coll": 14}
    rows.sort(key=lambda r: (order.get(r[3], 99), r[1]))
    names = [r[0] for r in rows]
    assert len(names) == len(set(names)), "duplicate hook name"
    addrs = {}
    for r in rows:
        addrs.setdefault(r[1], []).append(r[0])
    dup = {a: n for a, n in addrs.items() if len(n) > 1}
    assert not dup, "one address two names: %r" % dup

    lines = [
        "# SPDX-License-Identifier: MIT",
        "# Copyright (c) 2026 GurekamDhillon",
        "# mod-c-v1 hook table, version %d. GENERATED by tools/specs/gen_hook_table.py. Do not edit." % TABLE_VERSION,
        "# Key = address (retail GALE01 1.02). Rows are append-only across hooks-vN; a name or address never changes meaning.",
        "# Skip lines starting with '#'. Columns: name address signature phase alias site link sig_src. Spec: docs/specs/mod-c-v1.md",
        "\t".join(["name", "address", "signature", "phase", "alias", "site", "link", "sig_src"]),
    ]
    for n, a, s, p, al, site, lk, src in rows:
        lines.append("\t".join([n, "0x%08X" % a, s, p, al, site, lk, src]))
    text = "\n".join(lines) + "\n"
    if "--check" in sys.argv:
        cur = open(OUT, encoding="utf-8", newline="").read() if os.path.exists(OUT) else ""
        if cur != text:
            sys.exit("%s is stale; rerun tools/specs/gen_hook_table.py" % OUT)
        print("up to date: %d hooks" % len(rows))
        return
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    by = {}
    for r in rows:
        by[r[3]] = by.get(r[3], 0) + 1
    print("wrote %s: %d hooks" % (OUT, len(rows)))
    for k in sorted(by, key=lambda k: order.get(k, 99)):
        print("  %-14s %d" % (k, by[k]))


if __name__ == "__main__":
    main()
