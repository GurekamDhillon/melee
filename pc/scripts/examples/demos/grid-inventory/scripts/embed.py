#!/usr/bin/env python3
"""Embed grid.lua and layouts.lua into main.lua (the engine loads one entry file per mod and has no require).

    python scripts/embed.py          # rewrite the generated blocks in main.lua
    python scripts/embed.py --check  # exit 1 if main.lua is stale

grid.lua and layouts.lua stay the source of truth: both load on their own under plain lua for the tests.
"""
import re
import sys
from pathlib import Path

DIR = Path(__file__).resolve().parent
PARTS = [('GRID', 'grid.lua', 'grid'), ('LAYOUTS', 'layouts.lua', 'layouts')]


def block(tag, file, var):
    src = (DIR / file).read_text(encoding='utf-8')
    return (f'-- BEGIN GENERATED {tag} (scripts/{file}; regenerate with scripts/embed.py)\n'
            f'local {var} = (function()\n{src}end)()\n-- END GENERATED {tag}\n')


def main(argv):
    path = DIR / 'main.lua'
    text = path.read_text(encoding='utf-8')
    new = text
    for tag, file, var in PARTS:
        pattern = re.compile(rf'-- BEGIN GENERATED {tag}[^\n]*\n.*?-- END GENERATED {tag}\n', re.S)
        if len(pattern.findall(new)) != 1:
            sys.exit(f'main.lua must have exactly one generated {tag} block')
        new = pattern.sub(lambda _: block(tag, file, var), new, count=1)
    if '--check' in argv:
        if new != text:
            sys.exit('main.lua embeds stale sources; run scripts/embed.py')
        return
    if new != text:
        path.write_bytes(new.encode('utf-8'))


if __name__ == '__main__':
    main(sys.argv[1:])
