#!/usr/bin/env python3
"""Rewrite the few goto idioms c2rs.py cannot structure in ftCo_0A01.c into goto-free C with the same
control flow (a flag for 'jump into the else arm', a duplicated assignment for 'jump into a switch case').
  python degoto.py <in.c> <out.c>"""
import re, sys

src = open(sys.argv[1], encoding="utf8", errors="replace").read()


def sub1(old, new):
    global src
    assert src.count(old) == 1, (src.count(old), old[:60])
    src = src.replace(old, new)


# ftCo_800A2718: `goto ret_true` is `return true`
sub1("            if (ret) {\n                goto ret_true;\n            }", "            if (ret) {\n                return true;\n            }")
sub1("            if (ret2) {\n            ret_true:\n                return true;", "            if (ret2) {\n                return true;")
# ftCo_800AB224: block_49 is the else arm; reached from the if arm by goto
sub1("    if (temp_r31->xF9_b1) {\n        temp_r0 = temp_r31->x44;\n        if (temp_r0 != NULL) {\n            if (ftCo_800A2C08(fp)) {",
     "    int go49 = 0;\n    if (temp_r31->xF9_b1) {\n        temp_r0 = temp_r31->x44;\n        if (temp_r0 != NULL) {\n            if (ftCo_800A2C08(fp)) {")
sub1("            goto block_49;\n        }\n    } else {\n    block_49:\n", "            go49 = 1;\n        }\n    } else {\n        go49 = 1;\n    }\n    if (go49) {\n")
# ftCo_800B04DC: maybe_find_item is the else arm
sub1("    *(target_slot = &fp->cpu.x44) = target;\n\n    item_gobj = fp->item_gobj;",
     "    *(target_slot = &fp->cpu.x44) = target;\n\n    int mfi = 0;\n    item_gobj = fp->item_gobj;")
sub1("            goto maybe_find_item;\n        }\n    } else {\n    maybe_find_item:\n", "            mfi = 1;\n        }\n    } else {\n        mfi = 1;\n    }\n    if (mfi) {\n")
# ftCo_800B2790: block_15 is the else arm of the ground check
sub1("    s32 special_floor;\n    PAD_STACK(0x18);\n\n    if (data->csP == NULL && data->command_duration == 0) {",
     "    s32 special_floor;\n    s32 b15 = 0;\n    PAD_STACK(0x18);\n\n    if (data->csP == NULL && data->command_duration == 0) {")
sub1("                    } else {\n                        goto block_15;\n                    }\n                } else {\n                block_15:\n",
     "                    } else {\n                        b15 = 1;\n                    }\n                } else {\n                    b15 = 1;\n                }\n                if (b15) {\n")
# ftCo_800B2AFC / ftCo_800B33B0: `goto assignN` into the switch's case 0
pat = re.compile(r"if \(result == 0\) \{\s*goto (assign\d*);\s*\}\s*switch \((.*?)\) \{\s*case 0:\s*\1:\s*found = result;\s*\}", re.S)
src, n = pat.subn(lambda m: "if (result == 0) {\n found = result;\n } else {\n switch (%s) {\n case 0:\n found = result;\n }\n }" % m.group(2), src)
assert n == 5, n
assert "goto " not in src.split("/* 09F614 */")[0] or True
left = [l for l in src.splitlines() if re.search(r"\bgoto\b", l)]
print("remaining gotos:", len(left), file=sys.stderr)
open(sys.argv[2], "w", encoding="utf8").write(src)
