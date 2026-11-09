# Compare Hero-host captures (runs/h<seq>/cap/f.ppm) with our renderer's timeline shots (ref/shots/*tl_<tick>.png).
# His presenter frames the EFB differently (aspect, borders), so the crop of his capture that best matches ours is
# found first (independent x/y scale and offset, coordinate descent on 320x240 grey), then diffs are taken at 640x480.
import sys, glob, os
import numpy as np
from PIL import Image
runs, ref = sys.argv[1], sys.argv[2]
W, H = 640, 480

def grey(im, box, size):
    return np.asarray(im.convert('L').transform(size, Image.EXTENT, box, Image.BILINEAR), dtype=np.float32)

def register(h, r):
    rs = np.asarray(r.convert('L').resize((320, 240), Image.BILINEAR), dtype=np.float32)
    w, hh = h.size
    box = [0.0, 0.0, float(w), float(hh)]
    err = lambda b: np.abs(grey(h, tuple(b), (320, 240)) - rs).mean()
    best = err(box)
    step = w * 0.05
    while step > 0.5:
        improved = False
        for i in range(4):
            for s in (step, -step):
                b = list(box); b[i] += s
                e = err(b)
                if e < best:
                    best, box, improved = e, b, True
        if not improved:
            step /= 2
    return box

for hd in sorted(glob.glob(runs + '/h[0-9]*'), key=lambda p: int(os.path.basename(p)[1:])):
    caps = glob.glob(hd + '/cap/*.ppm')
    if not caps:
        continue
    seq = int(os.path.basename(hd)[1:])
    him = Image.open(caps[0]).convert('RGB')
    cands = [(int(os.path.basename(f).split('_')[-1][:5]), f) for f in glob.glob(ref + '/shots/*tl_*.png')]
    cands = [(t, f) for t, f in cands if abs(t - seq) <= 15]
    small = lambda im: np.asarray(im.convert('L').resize((80, 60)), dtype=np.float32)
    # pick the reference tick with the closest content first (coarse, unregistered), then register on it
    hs = small(him)
    t, rf = min(cands, key=lambda c: np.abs(small(Image.open(c[1])) - hs).mean())
    rim = Image.open(rf).convert('RGB')
    box = register(him, rim)
    # re-pick the tick with the registered crop (neighbouring ticks differ by motion only)
    hreg = him.transform((W, H), Image.EXTENT, tuple(box), Image.BILINEAR)
    scored = []
    for tt, ff in cands:
        r = np.asarray(Image.open(ff).convert('RGB').resize((W, H), Image.BILINEAR), dtype=np.float32)
        scored.append((np.abs(np.asarray(hreg, dtype=np.float32) - r).mean(), tt, ff, r))
    m, t, rf, r = min(scored, key=lambda s: s[0])
    h = np.asarray(hreg, dtype=np.float32)
    d = np.abs(h - r)
    big = (d.max(axis=2) > 24).mean() * 100
    # the gate's bar (notes/hero-renderer-plan.md section 5): >= 97% of pixels within 16 on every channel
    near = (d.max(axis=2) <= 16).mean() * 100
    psnr = 10 * np.log10(255 ** 2 / ((h - r) ** 2).mean())
    out = Image.new('RGB', (W * 3, H))
    out.paste(hreg, (0, 0)); out.paste(Image.fromarray(r.astype(np.uint8)), (W, 0))
    out.paste(Image.fromarray(np.clip(d * 4, 0, 255).astype(np.uint8)), (W * 2, 0))
    out.save(f'{runs}/cmp_{seq}.png')
    print(f'seq {seq} -> ref tick {t}: crop {[round(v) for v in box]}, mean abs {m:.2f}/255, PSNR {psnr:.1f} dB, {big:.1f}% px > 24, {near:.1f}% px <= 16')
