#!/usr/bin/env python3
"""Validate the shipped Geno mods against docs/specs/geno-v5.schema.json.

    python docs/specs/tools/check_geno_schema.py [--mods DIR] [--negative]

For every <mods>/*/geno.json it validates the document against the schema root, then every file it names
that the schema covers: fx_bindings (-> $defs/fxBindings) and fx/*/*.gfx.json (-> $defs/gfxPackage), and checks
the limits JSON Schema cannot express (file <= 1 MiB, words files exist and have no '..', at most 336 overlays, ...).
Needs the `jsonschema` package. Exit 0 = every file valid.

--negative also checks that the schema rejects a set of known-bad documents (so a passing run means something).
The shipped mods are all `define` fighters (geno 6-10); their `define`, `lua`, `controls`... blocks are out of v5
scope and only the v5 keys they carry are validated (reported per file as `checked` vs `ignored` top-level keys).
"""
import json
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:
    sys.exit("check_geno_schema: pip install jsonschema")

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "docs/specs/geno-v5.schema.json").read_text(encoding="utf-8"))
V = jsonschema.Draft202012Validator


def sub(name):
    s = {"$schema": SCHEMA["$schema"], "$ref": "#/$defs/" + name, "$defs": SCHEMA["$defs"]}
    return V(s)


ROOT_V = V(SCHEMA)
FX_V, BIND_V = sub("gfxPackage"), sub("fxBindings")
COVERED = set(SCHEMA["$defs"]["fighter"]["properties"])


def errors(v, doc):
    return sorted(v.iter_errors(doc), key=lambda e: list(e.absolute_path))


def report(path, errs):
    for e in errs[:8]:
        print("  %s: %s: %s" % (path, "/".join(str(p) for p in e.absolute_path) or "<root>", e.message[:160]))
    return len(errs)


def check_mod(d):
    bad = 0
    gj = d / "geno.json"
    if gj.stat().st_size > 1 << 20:
        print("  %s: larger than 1 MiB (the reader refuses it)" % gj)
        bad += 1
    doc = json.loads(gj.read_text(encoding="utf-8"))
    bad += report(gj.relative_to(ROOT), errors(ROOT_V, doc))
    checked, ignored = set(), set()
    for f in doc.get("fighters", []):
        for k in f:
            (checked if k in COVERED and k != "define" else ignored).add(k)
        if len(f.get("subactions", [])) > 336:
            print("  %s: more than 336 subaction overlays" % gj)
            bad += 1
        for s in f.get("subactions", []):
            if "file" in s:
                p = d / s["file"]
                if ".." in s["file"] or not p.is_file():
                    print("  %s: words file %s missing or refused" % (gj, s["file"]))
                    bad += 1
        fb = f.get("fx_bindings")
        if fb:
            p = d / fb
            if not p.is_file():
                print("  %s: fx_bindings %s missing" % (gj, fb))
                bad += 1
            else:
                bad += report(p.relative_to(ROOT), errors(BIND_V, json.loads(p.read_text(encoding="utf-8"))))
    for p in sorted(d.glob("fx/*/*.gfx.json")):
        bad += report(p.relative_to(ROOT), errors(FX_V, json.loads(p.read_text(encoding="utf-8"))))
    print("%-18s geno=%s fighters=%d  checked: %s  ignored (out of scope): %s" % (
        d.name, doc.get("geno"), len(doc.get("fighters", [])), ",".join(sorted(checked)) or "-", ",".join(sorted(ignored)) or "-"))
    return bad


NEGATIVE = [
    ("no version", {"fighters": []}),
    ("fighter without attach or define", {"geno": 5, "fighters": [{"name": "x"}]}),
    ("unknown attribute", {"geno": 5, "fighters": [{"attach": "kirby", "attributes": {"gravityy": 1}}]}),
    ("max_jumps 251", {"geno": 5, "fighters": [{"attach": "kirby", "attributes": {"max_jumps": 251}}]}),
    ("scale 3", {"geno": 5, "fighters": [{"attach": "kirby", "scale": 3}]}),
    ("unknown hook", {"geno": 5, "fighters": [{"attach": "kirby", "hooks": {"on_frame": ["geno.nope"]}}]}),
    ("nine hooks", {"geno": 5, "fighters": [{"attach": "kirby", "hooks": {"on_frame": ["geno.log"] * 9}}]}),
    ("bad target", {"geno": 5, "fighters": [{"attach": "kirby", "on_land": [{"from": "x", "to": 3}]}]}),
    ("49 states", {"geno": 5, "fighters": [{"attach": "kirby", "states": [{"name": "s"}] * 49}]}),
    ("17 articles", {"geno": 5, "fighters": [{"attach": "kirby", "articles": [{"name": "a"}] * 17}]}),
    ("unknown glide key", {"geno": 5, "fighters": [{"attach": "kirby", "glide": {"nope": 1}}]}),
    ("overlay without words", {"geno": 5, "fighters": [{"attach": "kirby", "subactions": [{"index": 3}]}]}),
    ("overlay index 1024", {"geno": 5, "fighters": [{"attach": "kirby", "subactions": [{"index": 1024, "words": [0]}]}]}),
]
POSITIVE = [
    ("minimal attach", {"geno": 1, "fighters": [{"attach": "kirby"}]}),
    ("docs example 7", {"geno": 1, "fighters": [{"attach": "kirby", "name": "Kirby (Brawl numbers)",
                                                  "attributes": {"gravity": 0.085, "terminal_velocity": 1.6, "max_jumps": 6},
                                                  "jumps": {"max": 6, "air_vy": [1.8, 1.55, 1.35, 1.2, 1.1]},
                                                  "hooks": {"on_init": ["geno.log:1"], "on_frame": ["geno.jumps.to_var:0"], "on_action": []}}]}),
    ("docs example 15.5", {"geno": 1, "fighters": [{"attach": "kirby", "subactions": [
        {"index": 87, "words": ["0xEC220001", 5, "0x08000014"]}, {"index": 88, "file": "geno/mk_fair.txt"}],
        "special_attributes": [{"index": 13, "float": 16.0}, {"offset": "0x2C", "int": 10}],
        "on_land": [{"from": "special:4", "to": "special:6"}, {"from": 66, "to": 43, "keep_frame": False}],
        "hooks": {"on_land": ["geno.count_frames:5"]}}]}),
    ("docs example 16.1", {"geno": 2, "fighters": [{"attach": "kirby", "states": [
        {"name": "GlideStart", "behavior": "geno.glide.start", "subaction": 57},
        {"name": "Plain", "behavior": "geno.air", "subaction": 18, "like": "motion:65", "flags": "0x55", "move_id": 13,
         "next": "geno:Glide", "land": "helpless", "landing_lag": 12, "anim": "next", "iasa": "like", "phys": "auto", "coll": "air"}],
        "glide": {"hold_frames": 16, "w07": 2.2, "end_buttons": 14}, "tornado": {"tap_cooldown": 10}, "drill": {"angle_max": 75, "speed": 2.2}}]}),
    ("docs example 19.2", {"geno": 5, "fighters": [{"attach": "kirby", "articles": [
        {"name": "Fire", "model": {"file": "GnSoraFire.dat", "symbol": "fire_joint"}, "scale": 1.0, "spin": 0, "lifetime": 60,
         "spawn": [8, 8], "velocity": [2.5, 0], "gravity": 0, "max_fall": 0, "accel": 0, "max_speed": 0,
         "homing": {"turn": 0, "range": 0, "delay": 0}, "max_live": 4,
         "despawn": {"hit": True, "shield": True, "stage": True, "clank": True},
         "hitboxes": [{"damage": 7, "size": 4, "offset": [0, 0, 0], "angle": 45, "kbg": 50, "wkb": 0, "bkb": 30, "element": 1,
                       "shield_damage": 0, "sfx_severity": 1, "sfx_kind": 0, "start": 1, "end": 0,
                       "hits": {"ground": True, "air": True, "reflect": True, "absorb": True, "counter": True}}]}]}]}),
]


def main():
    mods = ROOT / "pc/geno/mods"
    if "--mods" in sys.argv:
        mods = Path(sys.argv[sys.argv.index("--mods") + 1])
    bad = 0
    n = 0
    for d in sorted(p for p in mods.iterdir() if (p / "geno.json").is_file()):
        n += 1
        bad += check_mod(d)
    print("%d mod(s) with geno.json, %d error(s)" % (n, bad))
    for name, doc in POSITIVE:
        e = errors(ROOT_V, doc)
        if e:
            bad += report("positive: " + name, e)
    print("%d positive documents (the spec's own examples) valid" % len(POSITIVE))
    if "--negative" in sys.argv:
        for name, doc in NEGATIVE:
            if not errors(ROOT_V, doc):
                print("  negative document accepted: %s" % name)
                bad += 1
        print("%d negative documents rejected" % len(NEGATIVE))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
