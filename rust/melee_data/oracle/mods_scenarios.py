"""Random mods-folder scenarios for the mods oracle. Output: one script per scenario, separated by a
line "=== name". Fixtures are built by MD/MK lines, so C and Rust build the identical tree.
usage: mods_scenarios.py COUNT"""
import random, sys, json

count = int(sys.argv[1]) if len(sys.argv) > 1 else 60
r = random.Random(0x6d6f6473)
H = lambda b: (b if isinstance(b, bytes) else b.encode()).hex() or "-"

POOL = ["ace-base", "ace-wolf", "ace-fox", "akaneia", "zz", "Alpha", "alpha2", "stage-ps", "stage-ps2", "skin-fd",
        "script-x", "legacy", "m-ex", "t1", "t2", "t3", "MixedCase", "a.b", "dash-ed_1"]
KINDS = ["base", "fighter", "stage", "misc", "script", "skin", "bogus", "", "Base"]
DISC = ["PlWf.dat", "plwf.dat", "audio/us/wolf.ssm", "Audio/US/Wolf.ssm", "GrPs.dat", "GrPs1.usd", "GrIz.dat", "GrNLa.dat",
        "MxDt.dat", "x/y/z.bin", "notes.txt", "data.json", "script.lua", "GrOt.dat", "grst.dat", "GrI1.dat", "Pl.dat", "GrPsx.dat"]


def text_file(kind):
    base = "line one\nline two\n\nlast"
    return base.replace("\n", "\r\n") if kind else base


def mod_json(rr, mid, kind, req, con):
    c = rr.random()
    if c < 0.15:
        return None
    if c < 0.25:
        return b'{"name": "Broken", "kind": "' + (kind or "x").encode() + b'" ,,}'
    if c < 0.30:
        return b"not json at all"
    d = {}
    if rr.random() < 0.8:
        d["name"] = rr.choice([mid.title(), "Wolf é中", 'Quote"d \\ back', "x" * 200, "tab\there"])
    if rr.random() < 0.6:
        d["version"] = rr.choice(["1.0.0", "2", "v" * 40, "0.1-beta"])
    if kind is not None:
        d["kind"] = kind
    if rr.random() < 0.4:
        d["pack"] = rr.choice(["ace", "akaneia", "p" * 50])
    if rr.random() < 0.4:
        d["description"] = rr.choice(["A mod", "d" * 300, "multi\nline"])
    if rr.random() < 0.2:
        d["autostart"] = "mode=vs;at=match;stage=ext:31"
    if rr.random() < 0.3:
        d["hash"] = rr.choice(["abcdef0123", "h" * 100])
    if rr.random() < 0.3:
        d["source"] = rr.choice(["nucleus", "local file", "somewhere-else-long"])
    if rr.random() < 0.3:
        d["nucleus"] = {"id": rr.choice([0, 5, 12345, "x"]), "extra": [1, {"a": 2}], "name": "n"}
    if req:
        d["requires"] = req
    if con:
        d["conflicts"] = con
    if rr.random() < 0.15:
        d["engine"] = rr.choice([{"pobj_palette": 1}, {"pobj_palette": 2}, {"future_thing": 1}, {"pobj_palette": True}, {"pobj_palette": 0, "other": 0}])
    if rr.random() < 0.2:
        d["menus"] = [{"id": "m1", "parent": "main", "label": "L"}]
    if rr.random() < 0.15:
        d["id"] = rr.choice([mid, mid.upper(), "other-id"])
    if rr.random() < 0.2:
        d["unknown"] = rr.choice([[1, 2, [3]], {"a": {"b": [1, "x"]}}, 5, True, None, "s"])
    raw = json.dumps(d, indent=rr.choice([None, 2]), ensure_ascii=rr.random() < 0.5)
    b = raw.encode()
    if rr.random() < 0.1:
        b = b"\xef\xbb\xbf" + b
    if rr.random() < 0.1:
        b = b.replace(b'"', b'"', 1).replace(b"\\u00e9", b"\\u00E9")
    if rr.random() < 0.05:
        b = b.replace(b"\\n", b"\\u0041\\r\\b\\f\\/\\u00e9\\ud83d")
    return b


def build(name):
    rr = random.Random(r.getrandbits(32))
    out = [f"=== {name}"]
    n = rr.randint(2, 9)
    ids = rr.sample(POOL, n)
    if rr.random() < 0.15:
        ids.append("Z" * 70)  # too long for an id
    out.append("MD mods")
    out.append("MD mods/targettest")
    out.append("MD mods/_incoming")
    out.append("MD mods/.hidden")
    out.append(f"MK mods/stray.txt {H('a file, not a mod')}")
    mods_info = []
    for mid in ids:
        kind = rr.choice(KINDS)
        req = [x for x in rr.sample(POOL, rr.randint(0, 3)) if rr.random() < 0.6 and x != mid]
        con = [x for x in rr.sample(POOL, rr.randint(0, 2)) if rr.random() < 0.5 and x != mid]
        mods_info.append((mid, kind, req, con))
    for mid, kind, req, con in mods_info:
        legacy = rr.random() < 0.3
        base = f"mods/{mid}" + ("" if legacy else "/files")
        out.append(f"MD mods/{mid}")
        mj = mod_json(rr, mid, kind if rr.random() < 0.9 else None, req, con)
        if mj is not None:
            out.append(f"MK mods/{mid}/mod.json {H(mj)}")
        files = rr.sample(DISC, rr.randint(0, 5))
        if kind == "stage" and rr.random() < 0.7:
            files.append(rr.choice(["GrPs.dat", "GrPs1.usd", "GrIz.dat", "GrNLa.dat", "GrOt.dat", "GrI1.dat"]))
        for f in set(files):
            ext = f.rsplit(".", 1)[-1]
            if ext in ("txt", "json", "lua"):
                data = text_file(rr.random() < 0.5)
            else:
                data = bytes(rr.getrandbits(8) for _ in range(rr.randint(0, 40)))
            out.append(f"MK {base}/{f} {H(data)}")
        if legacy and rr.random() < 0.5:
            out.append(f"MK mods/{mid}/scripts/run.lua {H('print(1)')}")
        if not legacy and rr.random() < 0.3:
            out.append(f"MK mods/{mid}/scripts/run.lua {H('print(2)')}")
        if not legacy and rr.random() < 0.15:
            out.append(f"MD mods/{mid}/empty_dir")
    c = rr.random()
    if c < 0.55:
        lines = []
        for mid in ids:
            if rr.random() < 0.6:
                lines.append(rr.choice([mid, mid.lower(), "  " + mid + "  ", mid + "  # note", mid + "\t"]))
        if rr.random() < 0.4:
            lines.append("ghost-mod")
        if rr.random() < 0.4:
            lines.insert(0, "# header comment")
        if rr.random() < 0.3:
            lines.append("")
        sep = rr.choice(["\n", "\r\n"])
        data = sep.join(lines).encode()
        if rr.random() < 0.15:
            data = b"\xef\xbb\xbf" + data
        out.append(f"MK mods/enabled.txt {H(data)}")
    if rr.random() < 0.1:
        out.append(f"ENV MELEE_MODS {H('0')}")
    out.append("BOOT")
    out.append("FP")
    for _ in range(rr.randint(0, 5)):
        out.append(f"SET {rr.randint(-1, len(ids))} {rr.randint(0, 1)}")
    if rr.random() < 0.6:
        out.append("SAVE")
        out.append("CAT mods/enabled.txt")
    out.append("RESTART")
    for peer in rr.sample(["", "ace-base#1234", "ace-base#1234,ace-wolf#00ff", "a#1,b#2,c#3...", "x#zz", "#12", "ace-wolf#0x1f,Alpha#-1", "q" * 70 + "#1,ok#ff",
                           ",,a#1,,", "long#" + "f" * 20], rr.randint(0, 3)):
        out.append(f"DIFF {H(peer)} {rr.choice([8, 40, 200, 1000])}")
    out.append("DIFF - 100")
    for dp in rr.sample(DISC + ["nope.bin", "/PlWf.dat", "x/y/z.bin"], 4):
        out.append(f"WIN {dp}")
    for js in [b'{"engine": {"pobj_palette": 2}}', b'{"engine": {"pobj_palette": 1}}', b'{"engine": {"zzz": 3, "pobj_palette": 9}}', b"{", b'{"a": [1,2'.decode()]:
        out.append(f"ENGINE {H(js)}")
    return "\n".join(out)


print("\n".join(build(f"sc{i:02d}") for i in range(count)))
