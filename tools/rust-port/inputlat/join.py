"""join.py A.csv B.csv: the input-latency probe files (melee/src/inputlat.rs) of two peers of one match.

Per direction (A -> B, B -> A), over the frames both files have:
  sample->sent   our pad read to our input on the wire (sender's own time)
  sample->used   our pad read to the peer's session taking that input (one-way effective input delay, wall clock;
                 includes the network lag)
  lead           frames the input arrived ahead of the peer's current frame (f - cur; below 0: the peer had already
                 predicted it and rolls back)
and per side, sample->sim: the tick's pad read to its first session frame."""
import statistics
import sys


def load(path):
    rows = {"S": {}, "R": {}, "T": []}
    for line in open(path).read().split("\n")[1:]:
        if not line:
            continue
        k, f, a, b = line.split(",")
        f, a, b = int(f), int(a), int(b)
        if k == "T":
            rows["T"].append(b - a)
        elif f not in rows[k]:
            rows[k][f] = (a, b)
    return rows


def stat(xs, scale=1.0):
    if not xs:
        return "n=0"
    xs = sorted(x / scale for x in xs)
    p = lambda q: xs[min(len(xs) - 1, int(q * len(xs)))]
    return f"n={len(xs)} mean {statistics.fmean(xs):.2f} median {p(0.5):.2f} p10 {p(0.1):.2f} p90 {p(0.9):.2f}"


def direction(name, s, r):
    frames = sorted(set(s["S"]) & set(r["R"]))
    frames = [f for f in frames if f > 60]  # after the start hold
    sent = [s["S"][f][1] - s["S"][f][0] for f in frames]
    used = [r["R"][f][0] - s["S"][f][0] for f in frames]
    lead = [f - r["R"][f][1] for f in frames]
    print(f"{name}: sample->sent ms {stat(sent, 1000)}")
    print(f"{name}: sample->used ms {stat(used, 1000)}")
    print(f"{name}: lead frames     {stat(lead)}  (late, lead < 0: {sum(1 for x in lead if x < 0)})")


a, b = load(sys.argv[1]), load(sys.argv[2])
direction("A->B", a, b)
direction("B->A", b, a)
print(f"A: sample->sim ms {stat(a['T'][60:], 1000)}")
print(f"B: sample->sim ms {stat(b['T'][60:], 1000)}")
