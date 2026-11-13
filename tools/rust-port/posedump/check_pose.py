"""Cross-check a `melee pose-dump` directory against the external FigaTree decoder (geno_workbench
figatree_decode.py / figatree_import.py), which shares no code with the engine.

  python tools/rust-port/posedump/check_pose.py DUMP/<fighter> WORKBENCH_DIR [--clips N] [--frames a,b,c]

Needs the dump made with --raw (raw/Pl??AJ.dat and raw/Pl??Nr.dat beside index.json). For each checked clip and frame,
both sides' world matrices are taken relative to joint 0 (TopN), which removes the engine's root transform (position,
facing turn, model scale). Two numbers per clip: the raw local SRT against the decoder's channel values (the evaluator
check), and the world matrices of the joints the engine leaves alone (engine SRT == raw SRT for the joint and all its
ancestors except TopN) against the decoder's world matrices. Joints the engine rewrites (cape dynamics, per-joint scale,
root-motion extraction) are counted, not compared.
"""
import json, math, os, struct, sys


def read_pose(path):
    b = open(path, "rb").read()
    magic, ver, nj, nf, per = b[:4], *struct.unpack_from("<4I", b, 4)
    assert magic == b"MPD1" and ver == 1 and per == 31, path
    frames, off = [], 20
    for _ in range(nf):
        k, cur, ext = struct.unpack_from("<IfI", b, off)
        off += 12 + 24
        joints = []
        for _ in range(nj):
            mask = struct.unpack_from("<I", b, off)[0]
            v = struct.unpack_from("<30f", b, off + 4)
            off += 124
            joints.append((mask, v[0:9], v[9:18], v[18:30]))  # mask, raw SRT, engine SRT, world
        frames.append((k, cur, ext, joints))
    return frames


def m4(w):  # 12 floats row-major 3x4 -> 4x4
    return [list(w[0:4]), list(w[4:8]), list(w[8:12]), [0., 0., 0., 1.]]


def inv(m):  # affine inverse
    a = [r[:3] for r in m[:3]]
    det = (a[0][0]*(a[1][1]*a[2][2]-a[1][2]*a[2][1]) - a[0][1]*(a[1][0]*a[2][2]-a[1][2]*a[2][0])
           + a[0][2]*(a[1][0]*a[2][1]-a[1][1]*a[2][0]))
    c = [[(a[(j+1) % 3][(i+1) % 3]*a[(j+2) % 3][(i+2) % 3] - a[(j+1) % 3][(i+2) % 3]*a[(j+2) % 3][(i+1) % 3])/det
          for j in range(3)] for i in range(3)]
    t = [-sum(c[i][k]*m[k][3] for k in range(3)) for i in range(3)]
    return [c[0]+[t[0]], c[1]+[t[1]], c[2]+[t[2]], [0., 0., 0., 1.]]


def mul(a, b):
    return [[sum(a[i][k]*b[k][j] for k in range(4)) for j in range(4)] for i in range(4)]


def main():
    d, wb = sys.argv[1], sys.argv[2]
    nclips = int(sys.argv[sys.argv.index("--clips")+1]) if "--clips" in sys.argv else 8
    fsel = [int(x) for x in sys.argv[sys.argv.index("--frames")+1].split(",")] if "--frames" in sys.argv else [0, 1, 7, 20]
    sys.path.insert(0, wb)
    import figatree_decode as D
    import figatree_import as I
    idx = json.load(open(os.path.join(d, "index.json")))
    raw = os.path.join(d, "raw")
    aj = open(os.path.join(raw, [f for f in os.listdir(raw) if f.endswith("AJ.dat")][0]), "rb").read()
    nr = open(os.path.join(raw, [f for f in os.listdir(raw) if f.endswith("Nr.dat")][0]), "rb").read()
    arch = D._Archive(nr)
    sym = [n for n, _ in arch.publics if n.endswith("_Share_joint")][0]
    rest = D.read_model(nr, sym)
    assert len(rest) == idx["joint_count"], (len(rest), idx["joint_count"])
    clips = [c for c in idx["clips"] if "file" in c]
    step = max(1, len(clips)//nclips)
    worst, worstl, report = 0., 0., []
    for c in clips[::step][:nclips]:
        anim = D.read_bank(aj, {"clips": {c["symbol"]: {"offset": c["aj_offset"], "bytes": c["aj_size"]}}})[0]
        frames = read_pose(os.path.join(d, c["file"]))
        cw, cl, nt = 0., 0., set()
        for f in fsel:
            if f >= len(frames):
                continue
            k, cur, ext, joints = frames[f]
            try:
                py = I.world_matrices(rest, anim, f)
            except D.FigatreeError as e:
                report.append(f"{c['symbol']}: decoder refused: {e}")
                break
            e0, p0 = inv(m4(joints[0][3])), inv([list(r) for r in py[0]])
            par = [j["parent"] for j in idx["joints"]]
            touched = [i > 0 and max(abs(x - y) for x, y in zip(joints[i][1], joints[i][2])) > 1e-4 for i in range(len(joints))]
            for i in range(1, len(joints)):
                if par[i] > 0 and touched[par[i]]:
                    touched[i] = True
            for i, jt in enumerate(joints):
                vals = {"rotation": list(rest[i].rotation), "translation": list(rest[i].translation), "scale": list(rest[i].scale)}
                for t in (anim.joints[i] if i < len(anim.joints) else []):
                    if t.type in I.CHANNELS:
                        pr, ax = I.CHANNELS[t.type]
                        vals[pr][ax] = t.evaluate(f, vals[pr][ax])
                pl = vals["scale"] + vals["rotation"] + vals["translation"]
                if idx["joints"][i]["anim_applied"]:
                    cl = max(cl, max(abs(x - y) for x, y in zip(pl, jt[1])))
                if i == 0:
                    continue
                if touched[i]:
                    nt.add(i)
                    continue
                a = mul(e0, m4(jt[3]))
                b = mul(p0, [list(r) for r in py[i]])
                for r in range(3):
                    for q in range(4):
                        cw = max(cw, abs(a[r][q]-b[r][q]))
        worst = max(worst, cw)
        worstl = max(worstl, cl)
        report.append(f"{c['row']:3d} {c['symbol']}: raw local max err {cl:.3g}; world max |dW| {cw:.3g}; engine-rewritten joints {sorted(nt)}")
    print("\n".join(report))
    print(f"overall: raw local max err {worstl:.3g}, world max |dW| {worst:.3g} (frames {fsel})")


if __name__ == "__main__":
    main()
