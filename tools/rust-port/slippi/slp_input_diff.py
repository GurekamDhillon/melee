#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Post-hoc input diff of one engine-vs-Dolphin run (reorder_pair.sh OUTDIR): which inputs each side USED, per frame and
port, from the two .slp files (OUTDIR/engine.slp: the engine's confirmed frames; OUTDIR/dslp/*.slp: Slippi Dolphin's
replay, a rolled-back frame written again each time, the last copy kept), and what each side SENT for that frame (the
pad packets in the mock's capture OUTDIR/cap.txt; with --jitter-ms each line also has the datagram's hold time).

    python -I slp_input_diff.py OUTDIR

Prints the first frame where the two sides' inputs for a port differ (joystick, c-stick, trigger, buttons, raw x/y),
the first frame where any Pre-Frame field differs (seed, state), the first Post-Frame difference, and for the first input
mismatch: every pad packet that carried that frame from its owner (receive time, hold, the bytes) and whether the
datagrams around it were delivered out of order. Nothing is logged during the run.
"""
import glob
import os
import struct
import sys

PRE_INPUT = (0x19, 0x3C)  # joystick x/y, c-stick x/y, trigger, processed buttons, physical buttons, L, R, raw x
PRE_RAWY = 0x40


def events(path):
    d = open(path, "rb").read()
    i = d.index(b"raw[$U#l") + 8
    n = struct.unpack(">I", d[i:i + 4])[0]
    raw = d[i + 4:i + 4 + n] if n else d[i + 4:]  # 0: a file still open (the run ended before Game End)
    assert raw[0] == 0x35, "no Event Payloads"
    sz = raw[1]
    sizes = {}
    j = 2
    while j < 1 + sz:
        sizes[raw[j]] = struct.unpack(">H", raw[j + 1:j + 3])[0]
        j += 3
    p = 1 + sz
    out = []
    while p < len(raw):
        c = raw[p]
        if c not in sizes or p + 1 + sizes[c] > len(raw):
            break
        out.append((c, raw[p:p + 1 + sizes[c]]))
        p += 1 + sizes[c]
    return out


def frames(ev):
    """{frame: {(kind, port, follower): bytes}}, the last copy of a frame (a resimulation replaces it), plus the number of
    times each frame was written."""
    out, writes, cur = {}, {}, None
    for c, b in ev:
        if c == 0x3A:
            f = struct.unpack(">i", b[1:5])[0]
            writes[f] = writes.get(f, 0) + 1
            out[f] = {}
            cur = f
        elif c in (0x37, 0x38):
            f = struct.unpack(">i", b[1:5])[0]
            if f != cur:  # no Frame Start (an older replay): a repeated pre-frame of port 0 starts the frame again
                if c == 0x37 and (f not in out or (0x37, b[5], b[6]) in out[f]):
                    writes[f] = writes.get(f, 0) + 1
                    out[f] = {}
                cur = f
            out.setdefault(f, {})[(c, b[5], b[6])] = b
    return out, writes


def fmt_pre(b):
    jx, jy, cx, cy, tr = struct.unpack(">fffff", b[0x19:0x2D])
    pb, phys = struct.unpack(">IH", b[0x2D:0x33])
    l, r = struct.unpack(">ff", b[0x33:0x3B])
    rx = struct.unpack(">b", b[0x3B:0x3C])[0]
    ry = struct.unpack(">b", b[PRE_RAWY:PRE_RAWY + 1])[0] if len(b) > PRE_RAWY else None
    seed = struct.unpack(">I", b[7:11])[0]
    st = struct.unpack(">H", b[11:13])[0]
    return (f"stick ({jx:+.4f},{jy:+.4f}) c ({cx:+.4f},{cy:+.4f}) trig {tr:.4f} proc {pb:08X} phys {phys:04X} L {l:.4f} "
            f"R {r:.4f} raw ({rx},{ry}) | seed {seed:08X} state {st:#x}")


def enet_payloads(h):
    """(channel, payload) of the ENet datagram `h` (bytes); only the send commands."""
    out = []
    if len(h) < 2:
        return out
    peer = struct.unpack(">H", h[0:2])[0]
    p = 4 if peer & 0x8000 else 2
    fixed = {1: 8, 2: 48, 3: 44, 4: 8, 5: 4, 10: 12, 11: 16}
    while p + 4 <= len(h):
        cmd, ch = h[p] & 0x0F, h[p + 1]
        if cmd == 6:
            n = struct.unpack(">H", h[p + 4:p + 6])[0]
            out.append((ch, h[p + 6:p + 6 + n])); p += 6 + n
        elif cmd in (7, 9):
            n = struct.unpack(">H", h[p + 6:p + 8])[0]
            out.append((ch, h[p + 8:p + 8 + n])); p += 8 + n
        elif cmd in (8, 12):
            n = struct.unpack(">H", h[p + 8:p + 10])[0]
            p += 24 + n
        elif cmd in fixed:
            p += fixed[cmd]
        else:
            break
    return out


def wire(cap):
    """[(idx, t_ms, dir, hold_ms, wire_frame_newest, player_idx, [(frame, 9 bytes)...])] pad packets, plus every
    datagram's (t, dir, hold) for the delivery order."""
    pads, dgrams = [], []
    for k, ln in enumerate(open(cap)):
        p = ln.split()
        if len(p) < 3:
            continue
        t, d = int(p[0]), p[1]
        hold = float(p[3]) if len(p) > 3 else 0.0
        dgrams.append((t, d, hold))
        try:
            h = bytes.fromhex(p[2])
        except ValueError:
            continue
        for ch, pl in enet_payloads(h):
            if len(pl) >= 14 and pl[0] == 0x80:
                fr, idx = struct.unpack(">iB", pl[1:6])
                body = pl[14:]
                lst = [(fr - i, body[i * 9:i * 9 + 9]) for i in range(len(body) // 9)]
                pads.append((k, t, d, hold, fr, idx, lst))
    return pads, dgrams


def main():
    o = sys.argv[1]
    ep = os.path.join(o, "engine.slp")
    dl = sorted(glob.glob(os.path.join(o, "dslp", "*.slp")))
    if not os.path.exists(ep) or not dl:
        print(f"slp_input_diff: missing .slp (engine {os.path.exists(ep)}, dolphin {len(dl)})")
        return 2
    ee, ed = events(ep), events(dl[-1])
    gs = [b for c, b in ed if c == 0x36] or [b for c, b in ee if c == 0x36]
    stage = struct.unpack(">H", gs[0][0x13:0x15])[0] if gs else -1
    fe, we = frames(ee)
    fd, wd = frames(ed)
    # the two files number frames one apart (found on the first runs): the offset where the positions agree best
    def agree(k):
        return sum(fe[f][key][13:21] == fd[f + k][key][13:21] for f in fe if f + k in fd for key in fe[f]
                   if key[0] == 0x37 and key in fd[f + k])
    koff = max(range(-3, 4), key=agree)
    fe = {f + koff: v for f, v in fe.items()}
    we = {f + koff: v for f, v in we.items()}
    common = sorted(set(fe) & set(fd))
    print(f"engine.slp frames {min(fe)}..{max(fe)} (renumbered +{koff}: {len(fe)}); dolphin .slp frames {min(fd)}..{max(fd)} "
          f"({len(fd)}, {sum(1 for v in wd.values() if v > 1)} written more than once); common {len(common)}")
    mism, first_pre, first_post = [], None, None
    for f in common:
        for key in sorted(set(fe[f]) & set(fd[f])):
            a, b = fe[f][key], fd[f][key]
            n = min(len(a), len(b))
            if key[0] == 0x37:
                port = key[1]
                ia = a[PRE_INPUT[0]:PRE_INPUT[1]] + (a[PRE_RAWY:PRE_RAWY + 1] if n > PRE_RAWY else b"")
                ib = b[PRE_INPUT[0]:PRE_INPUT[1]] + (b[PRE_RAWY:PRE_RAWY + 1] if n > PRE_RAWY else b"")
                if ia != ib:
                    mism.append((f, port))
                # state fields: seed, action state, position, facing, percent
                sa, sb = a[7:0x19] + a[0x3C:0x40], b[7:0x19] + b[0x3C:0x40]
                if sa != sb and first_pre is None:
                    first_pre = (f, key, a, b)
            elif first_post is None:
                # bytes 0x26..0x36 (state bit flags, misc AS/hitstun, airborne/ground ids) and 0x52.. (hurtbox state and
                # later fields) differ in every run, desync or not: the engine's exporter fills them its own way
                dif = [i for i in range(5, n) if a[i] != b[i] and not (0x26 <= i <= 0x36 or i >= 0x52)]
                if dif:
                    first_post = (f, key, dif[:8])
    if first_pre:
        f, key, a, b = first_pre
        print(f"first Pre-Frame difference: frame {f} port {key[1]}{' (follower)' if key[2] else ''}")
        print(f"  engine : {fmt_pre(a)}\n  dolphin: {fmt_pre(b)}")
    if first_post:
        print(f"first Post-Frame difference: frame {first_post[0]} port {first_post[1][1]} bytes {first_post[2]}")
    # the first divergence's class: "input" (a post-GO input mismatch at or before the first state difference),
    # "Whispy" (Dream Land, inputs equal, the first difference a position with the action state equal: the wind's
    # push), else "state"
    first_state = min([x[0] for x in (first_pre, first_post) if x], default=None)
    if first_state is not None:
        inp_before = [m for m in mism if 0 <= m[0] <= first_state]
        same_as = first_pre is not None and first_pre[0] == first_state and first_pre[2][11:13] == first_pre[3][11:13]
        post_pos = first_post is not None and first_post[0] == first_state and all(0x0A <= i < 0x12 for i in first_post[2])
        cls = "input" if inp_before else ("Whispy" if stage == 0x1C and (same_as or post_pos) else "state")
        print(f"CLASS {cls}: first state difference at frame {first_state} (stage {stage:#x})")
    pre_go = [m for m in mism if m[0] < 0]
    if pre_go:
        print(f"inputs differ in {len(pre_go)} (frame, port) before GO (frame 0; first {pre_go[0][0]}, last {pre_go[-1][0]}): "
              "the countdown, where nothing reads them")
    post = [m for m in mism if m[0] >= 0]
    if not post:
        print("inputs: every common frame's inputs from frame 0 on equal on both sides" + ("" if first_pre or first_post else "; no difference at all"))
        return 0 if not (first_pre or first_post) else 1
    print(f"inputs differ in {len(post)} (frame, port) from frame 0 on: " + " ".join(f"{f}/p{p + 1}" for f, p in post[:20]))
    picks = [post[0]]
    if first_pre:
        before = [m for m in post if m[0] <= first_pre[0]]
        if before and before[-1] != post[0]:
            picks.append(before[-1])
    first_in = {}
    for f, p in picks:
        first_in.setdefault(p, f)
    pads, dgrams = wire(os.path.join(o, "cap.txt"))
    # the wire frame of slp frame f: align the sender's own port (wire pad buttons == its slp physical buttons)
    for port, f in sorted(first_in.items(), key=lambda kv: kv[1]):
        a, b = fe[f][(0x37, port, 0)], fd[f][(0x37, port, 0)]
        print(f"FIRST INPUT MISMATCH: frame {f} port {port + 1} (engine wrote frame {we.get(f, 0)}x, dolphin {wd.get(f, 0)}x)")
        print(f"  engine used : {fmt_pre(a)}\n  dolphin used: {fmt_pre(b)}")
        mine = [x for x in pads if x[5] == port]
        if not mine:
            print("  (no pad packets of that port in the capture)")
            continue
        off = best_offset(mine, fd if mine[0][2].startswith("B") else fe, port)
        wf = f + off
        print(f"  wire frame {wf} (slp frame + {off}); owner sends from {mine[0][2]}")
        seen = {}
        for k, t, d, hold, fr, idx, lst in mine:
            for g, by in lst:
                if g == wf:
                    seen.setdefault(by.hex(), []).append((t, hold, k, fr))
        for hx, occ in seen.items():
            print(f"    sent {hx} in {len(occ)} packets; first at {occ[0][0]} ms (newest frame {occ[0][3]}, hold {occ[0][1]:.2f} ms)")
        if len(seen) > 1:
            print("    OWNER SENT DIFFERENT BYTES FOR ONE FRAME")
        if seen:
            k0 = min(o_[2] for occ in seen.values() for o_ in occ)
            d0 = dgrams[k0][1]
            due = lambda i: dgrams[i][0] + dgrams[i][2]
            win = [i for i in range(max(0, k0 - 40), min(len(dgrams), k0 + 40)) if dgrams[i][1] == d0]
            inv = [(i, j) for i in win for j in win if i < j and due(i) > due(j)]
            print(f"    reorder: {len(inv)} out-of-order pairs among the {len(win)} {d0} datagrams around it")
        for g in range(wf - 3, wf + 4):
            got = sorted({by.hex() for k, t, d, hold, fr, idx, lst in mine for gg, by in lst if gg == g})
            sf = g - off
            print(f"    wire {g} (slp {sf}): sent {','.join(got) or '-'}"
                  + (f"  eng {inputs_hex(fe, sf, port)} dol {inputs_hex(fd, sf, port)}" if sf in fe and sf in fd else ""))
    return 1


def inputs_hex(fr, f, port):
    b = fr[f].get((0x37, port, 0))
    if not b:
        return "-"
    phys = struct.unpack(">H", b[0x31:0x33])[0]
    return f"{phys:04X}/{b[0x3B]:02x}"


def best_offset(mine, owner_frames, port):
    """slp frame + off = wire frame, chosen where the wire buttons match the owner's slp physical buttons best."""
    sent = {}
    for k, t, d, hold, fr, idx, lst in mine:
        for g, by in lst:
            sent[g] = struct.unpack(">H", by[0:2])[0]
    best = (-1, 124)
    for off in range(110, 140):
        score = 0
        for f, ev in owner_frames.items():
            b = ev.get((0x37, port, 0))
            if b is not None and f + off in sent:
                score += (struct.unpack(">H", b[0x31:0x33])[0] & 0x1F7F) == (sent[f + off] & 0x1F7F)
        best = max(best, (score, off))
    return best[1]


if __name__ == "__main__":
    sys.exit(main())
