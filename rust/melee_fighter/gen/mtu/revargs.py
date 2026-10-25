#!/usr/bin/env python3
"""Make the port evaluate call arguments in the order the C build does.

  python rust/melee_fighter/gen/mtu/revargs.py <src/tu/file.rs>

The C build (clang, x86_64 windows target, then the PPC-retarget lowering) evaluates call arguments right to left, so
`f(a(), b())` runs b() first; Rust runs a() first. It only shows when two arguments each call something with an effect
(a seam / hook). Such calls (>= 2 arguments containing `ext::x(` / `h::x(`) become
  { let __b = <last>; let __a = <first>; f(__a, __b) }
with only the hook-calling arguments hoisted, last first. Idempotent (a hoisted block is not rewritten again).
"""
import re, sys
from pathlib import Path

HOOK = re.compile(r"(?<![\w])(?:ext|h|crate::hooks)::\w+\(")
CALL = re.compile(r"(?<![\w.])((?:[A-Za-z_]\w*::)*[A-Za-z_]\w*)\(")
SKIP_KW = {"if", "while", "for", "match", "return", "fn", "loop", "unsafe", "let", "in", "as", "else"}


def match_paren(s, i):
    depth = 0
    j = i
    while j < len(s):
        c = s[j]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                return j
        j += 1
    return -1


def split_top(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out


def rewrite(text, counter):
    out, pos = [], 0
    while True:
        m = CALL.search(text, pos)
        if not m:
            out.append(text[pos:])
            break
        name = m.group(1)
        op = m.end() - 1
        cl = match_paren(text, op)
        if cl < 0 or name in SKIP_KW or text[max(0, m.start() - 3):m.start()].strip().endswith("fn"):
            out.append(text[pos:m.end()])
            pos = m.end()
            continue
        inner = text[op + 1:cl]
        args = split_top(inner)
        new_args = [rewrite(a, counter) for a in args]
        hot = [i for i, a in enumerate(new_args) if HOOK.search(a) and not a.lstrip().startswith("{ let __")]
        if len(hot) >= 2:
            lets = []
            call_args = list(new_args)
            for i in reversed(hot):
                counter[0] += 1
                v = f"__a{counter[0]}"
                lets.append(f"let {v} = {new_args[i].strip()};")
                call_args[i] = v
            call = f"{name}({', '.join(a.strip() if i in hot else a for i, a in enumerate(call_args))})"
            expr = "{ " + " ".join(lets) + " " + call + " }"
        else:
            expr = f"{name}({','.join(new_args)})"
        out.append(text[pos:m.start()] + expr)
        pos = cl + 1
    return "".join(out)


def main():
    p = Path(sys.argv[1])
    s = p.read_text(encoding="utf-8")
    crlf = "\r\n" in s
    s = s.replace("\r\n", "\n")
    a = s.index("pub mod ext")
    # the ext module is left alone; rewrite the rest
    depth, j = 0, s.index("{", a)
    k = j
    while True:
        depth += {"{": 1, "}": -1}.get(s[k], 0)
        k += 1
        if depth == 0:
            break
    head, ext, body = s[:a], s[a:k], s[k:]
    before = body
    body = rewrite(body, [0])
    changed = sum(1 for _ in re.finditer(r"\{ let __a\d+ =", body)) - sum(1 for _ in re.finditer(r"\{ let __a\d+ =", before))
    p.write_text((head + ext + body).replace("\n", "\r\n") if crlf else head + ext + body, encoding="utf-8", newline="")
    print(f"revargs: {p.name}: {changed} call(s) reordered")


if __name__ == "__main__":
    main()
