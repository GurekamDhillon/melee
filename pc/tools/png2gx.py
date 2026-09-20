#!/usr/bin/env python3
"""PNG -> GX texture data (.gxtex), for the port's custom menus.

    python pc/tools/png2gx.py in.png out.gxtex --format auto
    python pc/tools/png2gx.py in.png out.gxtex --format ci8
    python pc/tools/png2gx.py in.png out.gxtex --verify roundtrip.png

Nothing in the port could turn art into a texture before this.  A `.gxtex` is a small
self-describing container holding exactly what `HSD_ImageDesc` + `HSD_Tlut` need: the
GX-tiled image bytes, and for a colour-indexed format the TLUT.  `gw_runtime.c`'s
`gw_GxTex*` entry points read one at runtime and hand the bytes to game code, which
copies them into the game heap and points an `HSD_ImageDesc` at them.

FORMATS
  rgb5a3   GX_TF_RGB5A3 (5)  16 bpp, 4x4 tiles.  Per texel the top bit picks the encoding:
                             1 -> opaque RGB555, 0 -> RGB444 + 3-bit alpha.
  ci8      GX_TF_C8     (9)   8 bpp, 8x4 tiles, with a 256-entry GX_TL_RGB5A3 TLUT.
                             The palette is built from the RGB5A3-QUANTISED texels, so a
                             CI8 encode is byte-identical on screen to the RGB5A3 one
                             whenever the image has <= 256 distinct RGB5A3 values.
  rgba8    GX_TF_RGBA8  (6)  32 bpp, 4x4 tiles, AR plane then GB plane.  Lossless.
  auto                       ci8 if it fits in 256 RGB5A3 colours, else rgb5a3 if the
                             image is within RGB5A3's error budget, else rgba8.

`auto` CANNOT MAKE AN ART JUDGEMENT and will happily pick CI8 for a button whose label
the 5-bit channels shift off-white.  When an art pack ships a manifest, use it:

    python pc/tools/png2gx.py --manifest <menu>/out/manifest.json --outdir <dir>

which converts every element at its `recommended_format`, chosen by eye by whoever made
the art.  `auto` is for one-off images with no manifest behind them.

TWO THINGS THAT SILENTLY RUIN CORRECT ART, both deliberately not done here:

  * NO sRGB -> LINEAR TRANSFORM.  The PNGs are sRGB-tagged because that is what they are,
    and GX samples texels raw.  A converter that colour-manages on the way in washes the
    art out, and it looks like an art problem rather than a pipeline one.  This reads the
    8-bit channel values and quantises them; it never consults an ICC profile.

  * ALPHA STAYS STRAIGHT.  RGB5A3 has 8 translucent alpha steps plus "opaque"; anything
    premultiplied fringes at those steps.  The source PNGs are straight and nothing here
    multiplies by alpha.

The image bytes are big-endian, because that is what GX texture data is on a GameCube and
what Aurora's decoder expects.  The 64-byte header is big-endian too, so the whole file is
one byte order; `gw_runtime.c` byte-swaps the seven header words it reads and then does a
plain byte copy of the payload, which is what keeps the texels intact across gwtool's
byte-swapping world boundary.
"""
import argparse
import os
import struct
import sys

try:
    from PIL import Image
except ImportError:
    sys.exit("png2gx: needs pillow (pip install pillow)")

MAGIC = b"GXTX"
VERSION = 1
HEADER_SIZE = 64
PAYLOAD_ALIGN = 32  # GX wants texture data and TLUTs 32-byte aligned

GX_TF_RGB5A3 = 0x5
GX_TF_RGBA8 = 0x6
GX_TF_C8 = 0x9
GX_TL_RGB5A3 = 2

NO_TLUT = 0xFFFFFFFF


# ---------------------------------------------------------------- RGB5A3 ----

def rgb5a3_encode(r, g, b, a):
    """One texel as its 16-bit RGB5A3 word.

    The alpha decision is "is the nearest representable 3-bit alpha saturated", i.e.
    a >= 228 becomes fully opaque and takes the 5-bit colour path.  Encoders differ here
    (>= 0xE0 is also common); this one is just "round to the nearest thing the format
    can hold", which is the rule the rest of the file follows too."""
    a3 = (a * 7 + 127) // 255
    if a3 == 7:
        return (0x8000
                | (((r * 31 + 127) // 255) << 10)
                | (((g * 31 + 127) // 255) << 5)
                | ((b * 31 + 127) // 255))
    return ((a3 << 12)
            | (((r * 15 + 127) // 255) << 8)
            | (((g * 15 + 127) // 255) << 4)
            | ((b * 15 + 127) // 255))


def rgb5a3_decode(w):
    """What the hardware hands the TEV stage back, using GX's bit-replication expands."""
    if w & 0x8000:
        r5, g5, b5 = (w >> 10) & 31, (w >> 5) & 31, w & 31
        return ((r5 << 3) | (r5 >> 2), (g5 << 3) | (g5 >> 2),
                (b5 << 3) | (b5 >> 2), 255)
    a3 = (w >> 12) & 7
    r4, g4, b4 = (w >> 8) & 15, (w >> 4) & 15, w & 15
    return ((r4 << 4) | r4, (g4 << 4) | g4, (b4 << 4) | b4,
            (a3 << 5) | (a3 << 2) | (a3 >> 1))


# ------------------------------------------------------------------ tiling ----

def _tiles(width, height, tw, th):
    for ty in range(0, height, th):
        for tx in range(0, width, tw):
            yield tx, ty


def tile_rgb5a3(px, width, height):
    out = bytearray()
    for tx, ty in _tiles(width, height, 4, 4):
        for y in range(ty, ty + 4):
            for x in range(tx, tx + 4):
                r, g, b, a = px[x, y]
                out += struct.pack(">H", rgb5a3_encode(r, g, b, a))
    return bytes(out)


def tile_rgba8(px, width, height):
    out = bytearray()
    for tx, ty in _tiles(width, height, 4, 4):
        ar = bytearray()
        gb = bytearray()
        for y in range(ty, ty + 4):
            for x in range(tx, tx + 4):
                r, g, b, a = px[x, y]
                ar += bytes((a, r))
                gb += bytes((g, b))
        out += ar + gb
    return bytes(out)


def tile_c8(indices, width, height):
    out = bytearray()
    for tx, ty in _tiles(width, height, 8, 4):
        for y in range(ty, ty + 4):
            row = y * width
            out += bytes(indices[row + tx: row + tx + 8])
    return bytes(out)


# ------------------------------------------------------------------ encode ----

def load_rgba(path):
    img = Image.open(path)
    if img.mode != "RGBA":
        img = img.convert("RGBA")
    return img


def check_dimensions(width, height):
    problems = []
    for name, v in (("width", width), ("height", height)):
        if v & (v - 1):
            problems.append("%s %d is not a power of two" % (name, v))
        if v % 4:
            problems.append("%s %d is not a multiple of 4" % (name, v))
        if v > 1024:
            problems.append("%s %d is over GX's 1024 limit" % (name, v))
    return problems


def quantised_words(img):
    """Every texel's RGB5A3 word, in row-major order."""
    px = img.load()
    w, h = img.size
    return [rgb5a3_encode(*px[x, y]) for y in range(h) for x in range(w)]


def encode_ci8(img):
    words = quantised_words(img)
    palette = []
    index_of = {}
    for word in words:
        if word not in index_of:
            index_of[word] = len(palette)
            palette.append(word)
    if len(palette) > 256:
        raise ValueError(
            "%d distinct colours after an RGB5A3 quantise - CI8 holds 256.  Use "
            "--format rgb5a3 (same pixels, twice the memory) or rgba8." % len(palette))
    indices = [index_of[w] for w in words]
    w, h = img.size
    image = tile_c8(indices, w, h)
    # The TLUT is always emitted full length.  GXInitTlutObj takes an entry count and the
    # hardware loads in 16-entry units; a short LUT with a stale tail is the kind of thing
    # that reads back as somebody else's colours.
    tlut = b"".join(struct.pack(">H", palette[i] if i < len(palette) else 0)
                    for i in range(256))
    return GX_TF_C8, image, GX_TL_RGB5A3, 256, tlut, len(palette)


def encode_rgb5a3(img):
    px = img.load()
    w, h = img.size
    return GX_TF_RGB5A3, tile_rgb5a3(px, w, h), None, 0, b"", None


def encode_rgba8(img):
    px = img.load()
    w, h = img.size
    return GX_TF_RGBA8, tile_rgba8(px, w, h), None, 0, b"", None


def choose_format(img):
    try:
        encode_ci8(img)
        return "ci8"
    except ValueError:
        pass
    # RGB5A3's worst case is a 4-bit channel on a translucent texel.  Anything carrying
    # text or a soft edge wants RGBA8; measure rather than guess.
    px = img.load()
    w, h = img.size
    worst = 0
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            dr, dg, db, da = rgb5a3_decode(rgb5a3_encode(r, g, b, a))
            worst = max(worst, abs(r - dr), abs(g - dg), abs(b - db), abs(a - da))
    return "rgb5a3" if worst <= 8 else "rgba8"


ENCODERS = {"rgb5a3": encode_rgb5a3, "ci8": encode_ci8, "rgba8": encode_rgba8}


def write_gxtex(path, width, height, fmt, image, tlut_fmt, tlut_entries, tlut):
    img_off = HEADER_SIZE
    tlut_off = img_off + len(image)
    if tlut:
        tlut_off = (tlut_off + PAYLOAD_ALIGN - 1) // PAYLOAD_ALIGN * PAYLOAD_ALIGN
    header = struct.pack(
        ">4sIIIIIIII", MAGIC, VERSION, fmt, width, height,
        NO_TLUT if tlut_fmt is None else tlut_fmt, tlut_entries,
        len(image), len(tlut))
    header += struct.pack(">II", img_off, tlut_off)
    header += b"\0" * (HEADER_SIZE - len(header))
    with open(path, "wb") as f:
        f.write(header)
        f.write(image)
        if tlut:
            f.write(b"\0" * (tlut_off - (img_off + len(image))))
            f.write(tlut)


# ------------------------------------------------------------------ verify ----

def decode_gxtex(path):
    """Read a .gxtex back to an RGBA image - the offline half of the round trip."""
    with open(path, "rb") as f:
        blob = f.read()
    (magic, version, fmt, width, height, tlut_fmt, tlut_entries,
     img_size, tlut_size) = struct.unpack(">4sIIIIIIII", blob[:36])
    img_off, tlut_off = struct.unpack(">II", blob[36:44])
    if magic != MAGIC or version != VERSION:
        raise ValueError("%s: not a v%d .gxtex" % (path, VERSION))
    image = blob[img_off:img_off + img_size]
    tlut = blob[tlut_off:tlut_off + tlut_size]

    out = Image.new("RGBA", (width, height))
    px = out.load()
    if fmt == GX_TF_RGB5A3:
        i = 0
        for tx, ty in _tiles(width, height, 4, 4):
            for y in range(ty, ty + 4):
                for x in range(tx, tx + 4):
                    px[x, y] = rgb5a3_decode(struct.unpack_from(">H", image, i)[0])
                    i += 2
    elif fmt == GX_TF_RGBA8:
        i = 0
        for tx, ty in _tiles(width, height, 4, 4):
            for n in range(16):
                x, y = tx + n % 4, ty + n // 4
                a, r = image[i + n * 2], image[i + n * 2 + 1]
                g, b = image[i + 32 + n * 2], image[i + 32 + n * 2 + 1]
                px[x, y] = (r, g, b, a)
            i += 64
    elif fmt == GX_TF_C8:
        if tlut_fmt != GX_TL_RGB5A3:
            raise ValueError("only RGB5A3 TLUTs are understood")
        pal = [rgb5a3_decode(struct.unpack_from(">H", tlut, n * 2)[0])
               for n in range(tlut_entries)]
        i = 0
        for tx, ty in _tiles(width, height, 8, 4):
            for y in range(ty, ty + 4):
                for x in range(tx, tx + 8):
                    px[x, y] = pal[image[i]]
                    i += 1
    else:
        raise ValueError("format %d has no decoder here" % fmt)
    return out


def compare(src, back):
    """Max/mean channel delta, and alpha separately - alpha is where the format bites."""
    a = src.load()
    b = back.load()
    w, h = src.size
    worst_rgb = worst_a = 0
    total = n = 0
    for y in range(h):
        for x in range(w):
            sr, sg, sb, sa = a[x, y]
            br, bg, bb, ba = b[x, y]
            d = max(abs(sr - br), abs(sg - bg), abs(sb - bb))
            worst_rgb = max(worst_rgb, d)
            worst_a = max(worst_a, abs(sa - ba))
            total += d
            n += 1
    return worst_rgb, worst_a, total / float(n)


# ---------------------------------------------------------------- manifest ----

# An art pack names a GX format in prose ("CI8 (RGB5A3 TLUT)"), because the manifest is
# written for a human first.  Map the ones we can honour and refuse the rest loudly rather
# than silently substituting a format nobody chose.
MANIFEST_FORMATS = {
    "CI8 (RGB5A3 TLUT)": "ci8",
    "CI8": "ci8",
    "RGB5A3": "rgb5a3",
    "RGBA8": "rgba8",
}


def convert_manifest(manifest_path, outdir, res, verify_dir):
    import json
    with open(manifest_path) as f:
        manifest = json.load(f)
    base = os.path.dirname(os.path.abspath(manifest_path))
    # The manifest's paths are relative to the pack root, one level above out/.
    root = os.path.dirname(base) if os.path.basename(base) == "out" else base
    os.makedirs(outdir, exist_ok=True)
    if verify_dir:
        os.makedirs(verify_dir, exist_ok=True)
    failures = 0
    for element in manifest.get("elements", []):
        name = element["name"]
        src = os.path.join(root, element["file_%s" % res])
        want = element.get("recommended_format", "")
        fmt = MANIFEST_FORMATS.get(want)
        if fmt is None:
            print("%-20s SKIPPED - manifest asks for %r, which png2gx cannot write"
                  % (name, want))
            failures += 1
            continue
        argv = [src, os.path.join(outdir, name + ".gxtex"), "--format", fmt]
        if verify_dir:
            argv += ["--verify", os.path.join(verify_dir, name + ".png")]
        try:
            main(argv)
        except SystemExit as e:
            if e.code:
                print("%-20s FAILED: %s" % (name, e.code))
                failures += 1
    return 1 if failures else 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    if argv is None and "--manifest" in sys.argv[1:]:
        mp = argparse.ArgumentParser()
        mp.add_argument("--manifest", required=True)
        mp.add_argument("--outdir", required=True)
        mp.add_argument("--res", default="2x", choices=["1x", "2x"])
        mp.add_argument("--verify-dir")
        a = mp.parse_args(sys.argv[1:])
        return convert_manifest(a.manifest, a.outdir, a.res, a.verify_dir)
    ap.add_argument("png")
    ap.add_argument("gxtex")
    ap.add_argument("--format", default="auto",
                    choices=["auto", "rgb5a3", "ci8", "rgba8"])
    ap.add_argument("--verify", metavar="PNG",
                    help="decode the written .gxtex back to this PNG and report the "
                         "round-trip error")
    ap.add_argument("--allow-odd-size", action="store_true",
                    help="write a texture that is not POT/mult-4/<=1024 anyway")
    args = ap.parse_args(argv)

    img = load_rgba(args.png)
    width, height = img.size
    problems = check_dimensions(width, height)
    if problems and not args.allow_odd_size:
        sys.exit("png2gx: %s: %s" % (args.png, "; ".join(problems)))
    for p in problems:
        print("  warning: %s" % p)

    fmt_name = choose_format(img) if args.format == "auto" else args.format
    try:
        fmt, image, tlut_fmt, tlut_entries, tlut, used = ENCODERS[fmt_name](img)
    except ValueError as e:
        sys.exit("png2gx: %s: %s" % (args.png, e))

    write_gxtex(args.gxtex, width, height, fmt, image, tlut_fmt, tlut_entries, tlut)
    note = "" if used is None else "  (%d of 256 palette entries used)" % used
    print("%s  %dx%d  %s  image %d B  tlut %d B%s"
          % (os.path.basename(args.gxtex), width, height, fmt_name,
             len(image), len(tlut), note))

    if args.verify:
        back = decode_gxtex(args.gxtex)
        back.save(args.verify)
        worst_rgb, worst_a, mean = compare(img, back)
        print("  round trip: max rgb delta %d, max alpha delta %d, mean rgb delta %.3f"
              % (worst_rgb, worst_a, mean))
        print("  wrote %s" % args.verify)
    return 0


if __name__ == "__main__":
    sys.exit(main())
