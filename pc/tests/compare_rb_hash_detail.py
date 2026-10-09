"""Compare confirmed MELEE_RB_HASH_DETAIL logs, including pre-match frames.
Offsets are hash-stream word indices, not memory byte offsets.
Different item-list order can change diagnostics without changing the summed
wire hash; only unequal wire hashes count as mismatches.
"""
import argparse
import csv
import sys


def load(path):
    frames = {}
    with open(path, newline="") as stream:
        for row in csv.DictReader(stream):
            key = int(row["epoch"]), int(row["frame"])
            frame = frames.setdefault(key, {})
            kind = row["kind"]
            if kind in ("missing", "overflow"):
                raise ValueError(f"{path}: {kind} at epoch/frame {key}")
            field = kind, row["region"], int(row["slot"]), int(row["offset"]), row["symbol"]
            if field in frame:
                raise ValueError(f"{path}: duplicate record at {key}: {field}")
            frame[field] = row["value"]
    for key, frame in frames.items():
        if sum(field[0] == "hash" for field in frame) != 1:
            raise ValueError(f"{path}: missing frame hash at {key}")
    return frames


def compare(a, b, min_frames=1, require_prematch=False):
    shared = sorted(a.keys() & b.keys())
    if len(shared) < min_frames:
        raise ValueError(f"only {len(shared)} shared frames; need {min_frames}")
    if require_prematch and not any(frame < 0 for _, frame in shared):
        raise ValueError("no shared pre-match frames")
    # Unequal endpoints are normal shutdown skew; gaps inside overlap are not.
    for epoch in {key[0] for key in shared}:
        af = {f for e, f in a if e == epoch}
        bf = {f for e, f in b if e == epoch}
        lo, hi = max(min(af), min(bf)), min(max(af), max(bf))
        if any(f not in af or f not in bf for f in range(lo, hi + 1)):
            raise ValueError(f"missing frame inside epoch {epoch} overlap {lo}..{hi}")
    bad = []
    for key in shared:
        ha = next(v for field, v in a[key].items() if field[0] == "hash")
        hb = next(v for field, v in b[key].items() if field[0] == "hash")
        if ha != hb:
            bad.append(key)
    print(f"shared {len(shared)} frames, pre-match {sum(f < 0 for _, f in shared)}, mismatches {len(bad)}")
    if bad:
        key = bad[0]
        print(f"first mismatch: epoch {key[0]} frame {key[1]}")
        for field in sorted(a[key].keys() | b[key].keys()):
            va, vb = a[key].get(field), b[key].get(field)
            if va != vb:
                kind, region, slot, offset, symbol = field
                print(f"  {kind} {region}[{slot}] word +{offset} {symbol}: {va} vs {vb}")
    return not bad


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("a")
    parser.add_argument("b")
    parser.add_argument("--min-frames", type=int, default=1)
    parser.add_argument("--require-prematch", action="store_true")
    args = parser.parse_args()
    try:
        return 0 if compare(load(args.a), load(args.b), args.min_frames, args.require_prematch) else 1
    except (OSError, ValueError, KeyError, StopIteration) as error:
        print(f"incomplete evidence: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
