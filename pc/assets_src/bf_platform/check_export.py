"""Offline contract check; does not launch Melee. Run after export.py."""
from pathlib import Path
import math
import struct

ROOT = Path(__file__).resolve().parents[2] / 'scripts/examples/bf_platform/models'
blob = (ROOT / 'bf_platform.gxmesh').read_bytes()
magic, version, nv, ni, width, depth, thickness, vo, io = struct.unpack_from('>4sIIIfffII', blob)
assert magic == b'GXMS' and version == 2, 'Expected GXMS v2 with corner normals'
assert io == vo + nv * 32 and len(blob) == io + ni * 2
normals = set()
for i in range(nv):
    x, y, z, u, v, nx, ny, nz = struct.unpack_from('>8f', blob, vo + i * 32)
    assert all(math.isfinite(f) for f in (x, y, z, u, v, nx, ny, nz))
    assert 0 <= u <= 1 and 0 <= v <= 1
    assert abs(nx*nx + ny*ny + nz*nz - 1) < 1e-4
    normals.add(tuple(round(f, 3) for f in (nx, ny, nz)))
assert len(normals) > 8, 'Top, bevels, rim and underside need distinct normals'
assert max(struct.unpack_from(f'>{ni}H', blob, io)) < nv
for suffix in ('.gxtex', '.glow.gxtex'):
    tex = (ROOT / ('bf_platform' + suffix)).read_bytes()
    magic, version, fmt, w, h, _, _, size, _, off, _ = struct.unpack_from('>4s10I', tex)
    assert (magic, version, fmt) == (b'GXTX', 1, 6)
    expected, levels = 0, 0
    while w >= 64 and h >= 64:
        expected += w*h*4
        levels += 1
        w //= 2
        h //= 2
    assert size == expected and len(tex) == off + size and levels > 1, 'Missing mip chain'
print(f'PASS: {nv} vertices, {ni//3} triangles, {len(normals)} normals; base/glow mip chains')
