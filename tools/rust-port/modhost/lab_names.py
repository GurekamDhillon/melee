#!/usr/bin/env python3
"""Regenerate rust/mods/lab/src/names.rs: the common action-state names (the decomp's ftCommon_MotionState
enumerator names, src/melee/ft/kinds/ftCommon/forward.h; identifiers only, no disc data)."""
import os, re
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
src = open(os.path.join(repo, "src/melee/ft/kinds/ftCommon/forward.h"), encoding="utf-8").read()
i = src.index("ftCo_MS_DeadDown"); j = src.index("ftCo_MS_Count", i)
names = re.findall(r"ftCo_MS_(\w+)", src[i:j])
offs = [0]
for n in names:
    offs.append(offs[-1] + len(n))
path = os.path.join(repo, "rust/mods/lab/src/names.rs")
old = open(path, encoding="utf-8").read()
head, tail = old.split("pub const COUNT", 1)[0], old.split("/// The name of common action", 1)[1]
body = 'pub const COUNT: usize = %d;\nconst BLOB: &str = "%s";\nconst OFF: [u16; %d] = [%s];\n\n/// The name of common action' % (
    len(names), "".join(names), len(offs), ", ".join(map(str, offs)))
open(path, "w", encoding="utf-8").write(head + body + tail)
print(f"{len(names)} names")
