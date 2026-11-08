"""Our pad script (tools/rust-port/play/*.txt: `<hold ticks> <button mask hex> <stick x> <stick y>`, port 1) as the
input script of Hero's host (`--script`: `<retrace> <tokens>`, the latest line at or before a retrace applies), so a
core v1 run can take its controllers through the host's own pad path (pad_read) with no MELEE_PAD_SCRIPT and no OS
input. `offset` shifts every line by that many retraces (the host's retraces before our first tick).

    python to_host_script.py <ours.txt> <out.txt> [offset]
"""
import sys

BITS = [(0x0001, "DL"), (0x0002, "DR"), (0x0004, "DD"), (0x0008, "DU"), (0x0010, "Z"), (0x0020, "R"), (0x0040, "L"),
        (0x0100, "A"), (0x0200, "B"), (0x0400, "X"), (0x0800, "Y"), (0x1000, "START")]


def main():
    src, dst = sys.argv[1], sys.argv[2]
    offset = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    at = offset
    out = ["# converted from " + src.replace("\\", "/").split("/")[-1] + " by to_host_script.py"]
    for line in open(src, encoding="utf-8"):
        f = line.split("#")[0].split()
        if not f:
            continue
        hold, mask = int(f[0]), int(f[1], 16)
        sx = int(f[2]) if len(f) > 2 else 0
        sy = int(f[3]) if len(f) > 3 else 0
        toks = [name for bit, name in BITS if mask & bit]
        # both are raw PADStatus stick values
        toks += [f"sx={max(-128, min(127, sx))}", f"sy={max(-128, min(127, sy))}"]
        out.append(f"{at} " + "+".join(toks))
        at += hold
    out.append(f"{at} sx=0 sy=0")
    open(dst, "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
