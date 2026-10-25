#!/usr/bin/env python3
"""Compare C-build screenshots (c_NNNN.png, 1280x960) with the Rust viewer's (native_NNNN.png,
640x480) at the same match frames. The C shot is box-downscaled to the Rust size.
usage: shotcmp.py C_DIR RUST_DIR [--diff-dir DIR] [frame ...]"""
import sys, os
import numpy as np
from PIL import Image

# The C pad script counts frames from 1 in on_frame_pre: its shot n shows the world of sim frame n - 1.
OFFSET = int(os.environ.get('SHOT_OFFSET', '1'))

def load(p, size=None):
    im = Image.open(p).convert("RGB")
    if size and im.size != size:
        im = im.resize(size, Image.BOX)
    return np.asarray(im).astype(np.int16)

def main():
    a = sys.argv[1:]
    diffdir = None
    if "--diff-dir" in a:
        i = a.index("--diff-dir"); diffdir = a[i + 1]; del a[i:i + 2]
    cd, rd, frames = a[0], a[1], a[2:]
    if not frames:
        frames = sorted(int(f[7:11]) + OFFSET for f in os.listdir(rd) if f.startswith("native_"))
    # A frame may be given as LABEL=SIMFRAME: the C screenshot is captured asynchronously, some
    # 13-22 logic frames after the script requests it; the real frame is found from the CAM log.
    for f in frames:
        if "=" in str(f):
            f, sim = str(f).split("="); f = int(f); rp = f"{rd}/native_{int(sim) - 1:04d}.png"
        else:
            f = int(f); rp = f"{rd}/native_{f - OFFSET:04d}.png"
        cp = f"{cd}/c_{f:04d}.png"
        if not (os.path.exists(cp) and os.path.exists(rp)):
            continue
        r = load(rp); c = load(cp, (r.shape[1], r.shape[0]))
        d = np.abs(c - r).max(axis=2)
        exact = (d == 0).mean() * 100
        close = (d <= 16).mean() * 100
        print(f"frame {f:5d}: mean abs {np.abs(c - r).mean():6.2f}/255  identical {exact:5.1f}%  within16 {close:5.1f}%  differing(>16) {100 - close:5.1f}%")
        if diffdir:
            os.makedirs(diffdir, exist_ok=True)
            side = np.concatenate([c, r, np.repeat(np.clip(d * 4, 0, 255)[..., None], 3, 2)], axis=1).astype(np.uint8)
            Image.fromarray(side).save(f"{diffdir}/cmp_{f:04d}.png")

main()
