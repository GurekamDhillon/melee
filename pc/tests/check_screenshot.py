"""Validate the image from screenshot.lua; optionally pin unchanged match bytes.

Usage: python pc/tests/check_screenshot.py path/to/scene.png [--same before.png]
Requires Pillow (already available in the Windows development environment).
"""
import argparse
import hashlib
from pathlib import Path

from PIL import Image, ImageStat


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("image", type=Path)
    parser.add_argument("--same", type=Path)
    parser.add_argument("--size", default="2560x1440", help="expected render dimensions, WIDTHxHEIGHT")
    args = parser.parse_args()
    with Image.open(args.image) as image:
        image.load()
        rgb = image.convert("RGB")
        stats = ImageStat.Stat(rgb)
        expected = tuple(int(n) for n in args.size.split("x"))
        assert image.size == expected, f"unexpected render resolution: {image.size}, expected {expected}"
        assert min(stats.stddev) > 8, f"blank screenshot: RGB standard deviations {stats.stddev}"
        if "A" in image.getbands():
            assert image.getchannel("A").getextrema() == (255, 255), "screenshot must be opaque"
    if args.same:
        assert args.image.read_bytes() == args.same.read_bytes(), "match PNG bytes changed"
    print(f"PASS screenshot {args.image} {image.size} RGB stddev={stats.stddev} "
          f"sha256={hashlib.sha256(args.image.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
