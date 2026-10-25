#!/usr/bin/env python3
"""cmp.py CDIR RSDIR PREFIX [AGE:AGE_RS ...]: C shots (c_NNNN.png, 1280x960) vs Rust shots
(play_<PREFIX>_NNNN.png, 640x480), both at 640x480: mean abs error 0..255 and share of pixels within 16."""
import sys, glob, os
from PIL import Image
import numpy as np
from PIL import ImageFilter
cd, rd, prefix = sys.argv[1:4]
for cf in sorted(glob.glob(os.path.join(cd, "c_*.png"))):
    age = int(os.path.basename(cf)[2:6])
    rf = os.path.join(rd, "play_%s_%04d.png" % (prefix, age))
    if not os.path.exists(rf):
        continue
    c = np.asarray(Image.open(cf).convert("RGB").resize((640, 480), Image.BOX), dtype=float)
    r = np.asarray(Image.open(rf).convert("RGB").resize((640, 480), Image.BOX), dtype=float)
    d = np.abs(c - r)
    cb = np.asarray(Image.open(cf).convert("RGB").resize((640, 480), Image.BOX).filter(ImageFilter.GaussianBlur(2)), dtype=float)
    rb = np.asarray(Image.open(rf).convert("RGB").resize((640, 480), Image.BOX).filter(ImageFilter.GaussianBlur(2)), dtype=float)
    print("blur2 mae %5.2f  " % np.abs(cb - rb).mean(), end="")
    print("age %4d  mae %6.2f  within16 %5.1f%%" % (age, d.mean(), 100 * (d.max(axis=2) <= 16).mean()))
