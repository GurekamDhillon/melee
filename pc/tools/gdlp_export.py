#!/usr/bin/env python3
"""Export a retail-only, whole-match .gdlp (format 2) as a plain Slippi .slp.

    python pc/tools/gdlp_export.py match.gdlp [out.slp]      write the .slp (default: beside the input)
    python pc/tools/gdlp_export.py --info match.gdlp          print what the recording is and whether it exports

The same rules as the game's `lab rec export-slp` (gw_script_rec.inc, gr_export_ok), so a file the game
refuses is refused here with the same reason: a Geno / non-retail fighter, a mid-match clip that starts from
a saved state, a trimmed excerpt, a version-1 file (no Slippi events), a match recorded with mods.

A .gdlp is a Slippi-framed event stream (docs/geno.md 14.20): the export keeps the standard events (Game
Start 0x36, Pre-Frame 0x37, Post-Frame 0x38, Game End 0x39, Frame Start 0x3A, Frame Bookend 0x3C) and drops
the GD events 0x90..0x98. Exit code 0 = written, 1 = refused, 2 = unreadable.
"""
from __future__ import annotations

import struct
import sys
from pathlib import Path

HEAD = b"{U\x03raw[$U#l"
KEEP = (0x36, 0x37, 0x38, 0x39, 0x3A, 0x3C)
FL_RETAIL, FL_STATE, FL_WHOLE, FL_SLIPPI = 1, 2, 4, 8
META = b"U\x08metadata{U\x08playedOnSU\x08melee-pc}}"


class Refused(Exception):
    pass


def read_events(data: bytes):
    """Yield (cmd, payload) for every whole event after the size table; a torn tail ends the walk."""
    if len(data) < 17 or data[:11] != HEAD or data[15] != 0x35:
        raise ValueError("not a .gdlp (or .slp) file")
    tab_len = data[16]
    sizes = {}
    for i in range((tab_len - 1) // 3):
        o = 17 + 3 * i
        sizes[data[o]] = (data[o + 1] << 8) | data[o + 2]
    pos = 16 + tab_len
    while pos < len(data):
        cmd = data[pos]
        if cmd == 0x55 or cmd not in sizes:  # 'U': the metadata block
            return
        size = sizes[cmd]
        if pos + 1 + size > len(data):
            return
        yield cmd, data[pos + 1:pos + 1 + size], pos
        pos += 1 + size
        if cmd == 0x39:
            return


def describe(data: bytes) -> dict:
    info = {"ver": 0, "flags": 0, "mods": "", "fighters": [], "post": 0, "pre": 0, "start": False,
            "seg": False, "complete": False, "frames": 0, "emb": 0}
    first = last = None
    for cmd, p, _ in read_events(data):
        if cmd == 0x90:
            info["ver"] = (p[0] << 8) | p[1]
            info["flags"] = p[2]
        elif cmd == 0x91 and p[1]:
            chr_ = struct.unpack(">h", p[2:4])[0]
            info["fighters"].append((p[0], chr_, p[6:30].split(b"\0")[0].decode("ascii", "replace")))
        elif cmd == 0x97 and p[0] == 1:
            info["mods"] = p[1:256].split(b"\0")[0].decode("utf-8", "replace")
        elif cmd == 0x96:
            if struct.unpack(">II", p[:8])[1] != 0:
                info["seg"] = True
        elif cmd == 0x94:
            info["frames"] += 1
        elif cmd == 0x98:
            info["emb"] = struct.unpack(">I", p[8:12])[0]
        elif cmd == 0x36:
            info["start"] = True
        elif cmd == 0x37:
            info["pre"] += 1
        elif cmd == 0x38:
            info["post"] += 1
            f = struct.unpack(">i", p[:4])[0]
            first = f if first is None else min(first, f)
            last = f if last is None else max(last, f)
        elif cmd == 0x39:
            info["complete"] = True
    info["first"], info["last"] = first, last
    if info["ver"] == 0:
        raise ValueError("no GD header (an .slp is not a LAB recording)")
    if info["ver"] > 2:
        raise ValueError(f"recorded by a newer game (format {info['ver']}, this reads 2)")
    return info


def refusal(info: dict) -> str | None:
    f = info["flags"]
    if f & FL_STATE:
        return "this clip starts from a saved state (a mid-match LAB take), not a whole match"
    if not (f & FL_WHOLE) or not (f & FL_SLIPPI):
        return ("a format-1 recording has no Slippi events (only whole matches recorded by format 2 export)"
                if info["ver"] < 2 else "not a whole-match recording (no Slippi events)")
    if not (f & FL_RETAIL):
        bad = next(((s, n) for s, c, n in info["fighters"] if c > 25), (0, "?"))
        return (f"a Geno / non-retail fighter is in this match (P{bad[0] + 1} {bad[1]}): "
                "a .slp can only hold the retail roster")
    if info["mods"] and info["mods"] != "none":
        return f"recorded with mods ({info['mods']}): not a retail match"
    if info["seg"]:
        return "a trimmed excerpt cannot be exported as a match"
    if not info["start"] or info["post"] == 0:
        return "the recording has no Game Start / frames"
    return None


def export(data: bytes) -> tuple[bytes, int]:
    info = describe(data)
    why = refusal(info)
    if why:
        raise Refused(why)
    body = bytearray()
    sizes = {}
    ended = False
    posts = 0
    tab_len = data[16]
    for i in range((tab_len - 1) // 3):
        o = 17 + 3 * i
        sizes[data[o]] = (data[o + 1] << 8) | data[o + 2]
    for cmd, p, _ in read_events(data):
        if cmd in KEEP:
            body += bytes([cmd]) + p
            posts += cmd == 0x38
            ended = ended or cmd == 0x39
    if not ended:
        body += bytes([0x39, 7, 0, 0, 0, 0, 0])  # Game End: method 7 (no contest)
    table = bytearray([0x35, 1 + 3 * len(KEEP)])
    for c in KEEP:
        table += bytes([c]) + struct.pack(">H", sizes[c])
    raw = bytes(table) + bytes(body)
    return HEAD + struct.pack(">I", len(raw)) + raw + META, posts


def main(argv: list[str]) -> int:
    args = [a for a in argv if not a.startswith("--")]
    if not args:
        print(__doc__)
        return 2
    src = Path(args[0])
    try:
        data = src.read_bytes()
        if "--info" in argv:
            info = describe(data)
            why = refusal(info)
            print(f"format {info['ver']}, flags {info['flags']}, frames {info['first']}..{info['last']} "
                  f"({info['post']} post-frame events), fighters {info['fighters']}, mods '{info['mods']}'")
            print("exports" if why is None else f"refused: {why}")
            return 0 if why is None else 1
        out, posts = export(data)
    except Refused as e:
        print(f"refused: {e}", file=sys.stderr)
        return 1
    except (OSError, ValueError, struct.error) as e:
        print(f"unreadable: {e}", file=sys.stderr)
        return 2
    dst = Path(args[1]) if len(args) > 1 else src.with_suffix(".slp")
    if dst.exists():
        print(f"refused: {dst} exists (nothing is overwritten)", file=sys.stderr)
        return 1
    dst.write_bytes(out)
    print(f"wrote {dst} ({posts} post-frame events)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
