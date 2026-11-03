#!/usr/bin/env python3
"""sbs.py CDIR RSDIR PREFIX AGE OUT.png: C | Rust | |diff|x4 side by side at 640x480."""
import sys, os
from PIL import Image, ImageChops
cd, rd, prefix, age, out = sys.argv[1:6]
c = Image.open(os.path.join(cd, "c_%04d.png" % int(age))).convert("RGB").resize((640, 480), Image.BOX)
r = Image.open(os.path.join(rd, "play_%s_%04d.png" % (prefix, int(age)))).convert("RGB").resize((640, 480), Image.BOX)
d = ImageChops.difference(c, r).point(lambda v: min(255, v * 4))
o = Image.new("RGB", (1920, 480)); o.paste(c, (0, 0)); o.paste(r, (640, 0)); o.paste(d, (1280, 0)); o.save(out)
