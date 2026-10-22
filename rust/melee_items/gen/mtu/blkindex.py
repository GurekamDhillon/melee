"""Index of the melee_types records (gen.rs) by C source location, for the oracle generator: the Rust names of a
record's anonymous members carry the GameCube offset (`anon_daa`), which a native parse cannot recompute."""
import re


def build_index(gen_rs):
    blk = {}
    pat = re.compile(r"^/// C `[^\n]*` \((src/[^:]+):(\d+)\); GameCube size[^\n]*\n(?:#\[[^\n]*\n)+pub (?:struct|union) (\w+) \{\n(.*?)^\}",
                     re.M | re.S)
    for m in pat.finditer(gen_rs):
        flds, prev = [], ""
        for line in m.group(4).split("\n"):
            fm = re.match(r"\s+pub (\w+):", line)
            if fm:
                flds.append((fm.group(1), prev.strip() == "/// anonymous C member"))
            prev = line
        blk.setdefault((m.group(1), int(m.group(2))), []).append((m.group(3), flds))
    return blk
