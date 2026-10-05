#!/usr/bin/env python3
"""Embed grid.lua (a copy of demos/grid-inventory/scripts/grid.lua) into main.lua: the engine loads one entry
file per mod and has no require.   python scripts/embed.py [--check]"""
import re
import sys
from pathlib import Path

DIR = Path(__file__).resolve().parent
path = DIR / 'main.lua'
text = path.read_text(encoding='utf-8')
src = (DIR / 'grid.lua').read_text(encoding='utf-8')
blk = ('-- BEGIN GENERATED GRID (scripts/grid.lua; regenerate with scripts/embed.py)\n'
       f'local grid = (function()\n{src}end)()\n-- END GENERATED GRID\n')
pat = re.compile(r'-- BEGIN GENERATED GRID[^\n]*\n.*?-- END GENERATED GRID\n', re.S)
if len(pat.findall(text)) != 1:
    sys.exit('main.lua must have exactly one generated GRID block')
new = pat.sub(lambda _: blk, text, count=1)
if '--check' in sys.argv:
    if new != text:
        sys.exit('main.lua embeds a stale grid.lua; run scripts/embed.py')
elif new != text:
    path.write_bytes(new.encode('utf-8'))
