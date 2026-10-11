"""Adapter: take the XV2-converted Kamehameha packages (tools/xv2_fx.py output, workspace worktree kamexv2) and switch on
the Geno FX renderer features R1-R6 so the demo shows them. It edits COPIES; the converter is untouched. Textures it adds
are SYNTHETIC (generated here, no XV2 data).

    python xv2_fx_render_adapter.py SRC_MODS_DIR DST_MODS_DIR [--tint r,g,b] [--no-events]

SRC/DST are `.../package/mods` dirs (they hold goku-xv2/fx/P_TrailGokuXKame*/ and goku-xv2/fx/fx_bindings.json).
What it does to every P_TrailGokuXKame* package:
  R1  additive emitters with bloom: tint + radius (a blue glare that reaches further), beam-mesh intensity raised
  R3  alpha-blend billboards: soft_particle distance 1.5 (no hard edge where a sprite meets the floor)
  R2  + `rx_stream`: a camera-facing tapered ribbon streaming out of the hand along the beam (Tap/Mid/Full/Super)
  R5  + `rx_ring_head`: shock rings leaving the beam head, `rx_ring_gather`: a contracting ring on the charge
  R4  the rings use the dissolve shader (noise mask, independent scroll, glowing edge)
  R6  fx_bindings.json: a shake + flash call at the release frame of KameTap/Mid/Full/Super (no package)
"""
import argparse, copy, json, math, random, shutil, struct, sys, zlib
from pathlib import Path


def png(path, w, h, px):
    """px: function (x, y) -> (r, g, b, a) 0..255; writes an RGBA PNG with no dependencies."""
    raw = bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w):
            raw += bytes(px(x, y))
    def chunk(t, d):
        c = struct.pack('>I', len(d)) + t + d
        return c + struct.pack('>I', zlib.crc32(t + d) & 0xFFFFFFFF)
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)) +
                           chunk(b'IDAT', zlib.compress(bytes(raw), 9)) + chunk(b'IEND', b''))


def make_textures(tex_dir):
    rnd = random.Random(7)
    # streak: across (v) a bright core falling off, along (u) banded streaks; alpha carries the picture
    def streak(x, y):
        u, v = x / 128.0, y / 128.0
        core = math.exp(-((v - 0.5) * 3.4) ** 2)
        band = 0.72 + 0.28 * math.sin(u * 6.2832 * 3 + 2.0 * math.sin(v * 9.0))
        a = max(0.0, min(1.0, core * band))
        return (255, 255, 255, int(a * 255))
    # ring band: soft peak in the middle of the band (v = inner->outer), faint ticks around (u)
    def ring(x, y):
        u, v = x / 128.0, y / 128.0
        prof = math.exp(-((v - 0.55) * 4.2) ** 2)
        tick = 0.85 + 0.15 * math.sin(u * 6.2832 * 24)
        return (255, 255, 255, int(max(0.0, min(1.0, prof * tick)) * 255))
    # tileable value noise for the dissolve mask (red channel)
    g = [[rnd.random() for _ in range(8)] for _ in range(8)]
    def noise(x, y):
        fx, fy = x / 128.0 * 8, y / 128.0 * 8
        x0, y0 = int(fx) % 8, int(fy) % 8
        x1, y1 = (x0 + 1) % 8, (y0 + 1) % 8
        tx, ty = fx - int(fx), fy - int(fy)
        sx, sy = tx * tx * (3 - 2 * tx), ty * ty * (3 - 2 * ty)
        a = g[y0][x0] * (1 - sx) + g[y0][x1] * sx
        b = g[y1][x0] * (1 - sx) + g[y1][x1] * sx
        n = a * (1 - sy) + b * sy
        return (int(n * 255), int(n * 255), int(n * 255), 255)
    png(tex_dir / 'rx_streak.png', 128, 128, streak)
    png(tex_dir / 'rx_ring.png', 128, 128, ring)
    png(tex_dir / 'rx_noise.png', 128, 128, noise)


def tex_entry(name):
    return {'name': name, 'file': f'tex/{name}.png', 'w': 128, 'h': 128, 'source_format': 'png', 'swizzle': 'rgba'}


def sampler(slot, tex, scroll_add=(0.0, 0.0), wrap='Repeat'):
    return {'slot': slot, 'texture': tex, 'wrap': [wrap, wrap], 'filter': 'linear', 'sphere_map': False, 'uv_channel': 0,
            'pattern': {'mode': 'none', 'count': 0.0, 'frequency': 1.0, 'count_random': 0.0, 'table': [0], 'loop_random_start': False},
            'uv': {'scroll': [0.0, 0.0], 'scroll_add': list(scroll_add), 'scroll_random': [0.0, 0.0], 'scale': [1.0, 1.0],
                   'scale_add': [0.0, 0.0], 'scale_random': [0.0, 0.0], 'divide': [1, 1], 'rotate': 0.0, 'rotate_add': 0.0,
                   'rotate_random': 0.0, 'enable': {'scroll': 1, 'scale': 1, 'rotate': 1}}}


def pick_template(emitters):
    for e in emitters:
        if e['kind'] == 'particle' and e.get('material', {}).get('blend') == 'add':
            return e
    return next((e for e in emitters if e['kind'] == 'particle'), emitters[0])


def clone(t, name, **kw):
    e = copy.deepcopy(t)
    e['name'] = name
    for k, v in kw.items():
        e[k] = v
    return e


def keys_alpha(v):
    return {'kind': 'keys', 'value': v[0][0], 'keys': v}


def build_emitter(t, name, life, scale_base, scale_keys, alpha_keys, rgb, hdr, translate, rotate, emission, samplers,
                  shader=None, extra=None, vel=None, param=None):
    e = clone(t, name)
    e['kind'] = 'particle'
    e['mesh'] = None
    e['follow'] = 'srt'
    e['transform'] = {'translate': translate, 'translate_random': [0.0, 0.0, 0.0], 'rotate': rotate,
                      'rotate_random': [0.0, 0.0, 0.0], 'scale': [1.0, 1.0, 1.0]}
    e['emission'] = emission
    p = e['particle']
    p['life'] = life
    p['life_random_pct'] = 0
    p['shape'] = 'billboard'
    p['velocity'] = {'all_direction': 0.0, 'direction': vel[0] if vel else [0, 1, 0], 'direction_scale': vel[1] if vel else 0.0,
                     'diffusion_angle': 0.0, 'xz_diffusion': 0.0, 'diffusion': [0.0, 0.0, 0.0], 'random_pct': 0.0,
                     'inherit': 0.0, 'momentum_random': 0.0}
    p['rotation'] = {'axes': [False, False, 1], 'init': [0, 0, 0], 'init_random': [0, 0, 0], 'add': [0, 0, 0],
                     'add_random': [0, 0, 0], 'regist': 1.0, 'reverse_random': [False, False, False]}
    p['scale'] = {'base': [scale_base] * 3, 'random_pct': [0.0, 0.0, 0.0], 'keys': scale_keys, 'loop': False, 'loop_rate': 1,
                  'add_velocity': 0.0, 'limit_distance': [200.0, 200.0]}
    p['param_keys'] = param or [[0.0, 0.0, 0.0, 0.0]]
    c = e['color']
    c['scale'] = hdr
    c['color0'] = {'kind': 'constant', 'value': rgb}
    c['color1'] = {'kind': 'constant', 'value': rgb}
    c['alpha0'] = keys_alpha(alpha_keys)
    c['alpha1'] = {'kind': 'constant', 'value': 1.0}
    e['samplers'] = samplers
    m = e['material'] = copy.deepcopy(t.get('material', {}))
    m['blend'] = 'add'
    m['depth_test'] = 1
    m['shader'] = shader or {'type': 'sprite', 'color': 'modulate'}
    m['bloom'] = {'threshold': 0.5, 'intensity': 1.2}
    m.pop('alpha_test', None)
    if extra:
        e.update(extra)
    return e


SKIP = set()


def adapt_package(pdir, tint, radius):
    name = pdir.name
    j = pdir / f'{name}.gfx.json'
    d = json.loads(j.read_text(encoding='utf-8'))
    ems = d['emitters']
    # R1 glare, R3 soft particles
    for e in ems:
        m = e.setdefault('material', {})
        b = m.get('bloom') or {}
        if 'glare' in SKIP:
            pass
        elif m.get('blend') == 'add' and b.get('intensity', 0) > 0:
            b['tint'] = tint
            b['radius'] = radius
            if e['kind'] == 'mesh':
                b['threshold'] = 0.35
                b['intensity'] = max(b['intensity'], 0.9)
            m['bloom'] = b
        elif 'soft' not in SKIP and m.get('blend') != 'add' and e['kind'] == 'particle' and e['particle'].get('shape') == 'billboard':
            m['soft_particle'] = {'distance': 1.5, 'volume': 1}
    t = pick_template(ems)
    n_orig = len(ems)
    if 'tex' not in str(d.get('textures')):
        pass
    tex_dir = pdir / 'tex'
    make_textures(tex_dir)
    have = {x['name'] for x in d['textures']}
    for n in ('rx_streak', 'rx_ring', 'rx_noise'):
        if n not in have:
            d['textures'].append(tex_entry(n))
    cyan = [0.45, 0.8, 1.0]
    beam = next((e for e in ems if e['kind'] == 'mesh' and e['name'].startswith(('bas_', 'ribbon_'))), None)
    if 'new' in SKIP:
        beam = None
    if beam is not None and name.endswith(('Tap', 'Mid', 'Full', 'Super')):
        blife = max(beam['particle']['life'], 20)
        head_z = beam['transform']['translate'][2]
        rot = beam['transform']['rotate']
        em = lambda start, dur, interval: {'start': start, 'duration': dur, 'one_time': False, 'rate': 1.0, 'rate_random': 0.0,
                                         'interval': interval, 'interval_random': 0.0, 'position_random': 0.0, 'by_distance': False,
                                         'fade': {'on_stop': True, 'alpha_frames': 0, 'fade_in_frames': 0, 'alpha_fade_in': False,
                                                  'scale_fade_in': False}}
        # R2 ribbon stream out of the hand: particles move outward along the beam, the strip joins them
        ems.append(build_emitter(
            t, 'rx_stream', 16, 3.0, [[1, 1, 1, 0.0], [1.15, 1.15, 1.15, 0.3], [0.25, 0.25, 0.25, 1.0]],
            [[0.0, 0, 0, 0.0], [1.0, 0, 0, 0.12], [1.0, 0, 0, 0.6], [0.0, 0, 0, 1.0]], cyan, 3.0,
            [0.0, 0.0, 1.0], rot, em(2, blife, 0), [sampler(0, 'rx_streak', (-0.06, 0.0))],
            extra={'ribbon': {'taper': 0.2, 'uv_per_unit': 0.05}}, vel=([0, 0, 1], head_z / 14.0)))
        # R5 + R4 shock rings leaving the beam head (dissolve with a noise mask)
        dis = {'type': 'dissolve', 'mask': 1, 'edge': 0.25, 'edge_color': [0.7, 0.95, 1.0]}
        ems.append(build_emitter(
            t, 'rx_ring_head', 18, 14.0, [[0.25, 0.25, 0.25, 0.0], [1.0, 1.0, 1.0, 0.45], [1.7, 1.7, 1.7, 1.0]],
            [[0.0, 0, 0, 0.0], [1.0, 0, 0, 0.15], [0.8, 0, 0, 0.6], [0.0, 0, 0, 1.0]], cyan, 3.2,
            [0.0, 0.0, head_z], rot, em(4, blife, 10),
            [sampler(0, 'rx_ring', (0.0, 0.0)), sampler(1, 'rx_noise', (0.015, -0.02))], shader=dis,
            param=[[0.05, 0, 0, 0.0], [0.9, 0, 0, 1.0]], extra={'ring': {'inner': 0.62, 'segments': 56}}))
    if name.endswith(('Charge', 'SuperCharge')) and 'new' not in SKIP:
        em2 = {'start': 0, 'duration': 0, 'one_time': False, 'rate': 1.0, 'rate_random': 0.0, 'interval': 14,
               'interval_random': 0.0, 'position_random': 0.0, 'by_distance': False,
               'fade': {'on_stop': True, 'alpha_frames': 0, 'fade_in_frames': 0, 'alpha_fade_in': False, 'scale_fade_in': False}}
        e = build_emitter(
            t, 'rx_ring_gather', 14, 9.0, [[1.7, 1.7, 1.7, 0.0], [0.7, 0.7, 0.7, 0.7], [0.2, 0.2, 0.2, 1.0]],
            [[0.0, 0, 0, 0.0], [0.9, 0, 0, 0.25], [1.0, 0, 0, 0.8], [0.0, 0, 0, 1.0]], cyan, 3.0, [0.0, 0.0, 3.0], [0.0, 0.0, 0.0],
            em2, [sampler(0, 'rx_ring')], extra={'ring': {'inner': 0.7, 'segments': 48}})
        e['transform']['translate'] = list(t['transform']['translate'])
        ems.append(e)
    if name.endswith('Muzzle') and 'new' not in SKIP:
        ems.append(build_emitter(
            t, 'rx_ring_muzzle', 14, 12.0, [[0.2, 0.2, 0.2, 0.0], [1.0, 1.0, 1.0, 0.4], [1.9, 1.9, 1.9, 1.0]],
            [[0.0, 0, 0, 0.0], [1.0, 0, 0, 0.1], [0.8, 0, 0, 0.5], [0.0, 0, 0, 1.0]], [0.7, 0.9, 1.0], 3.4,
            list(t['transform']['translate']), list(t['transform']['rotate']),
            {'start': 0, 'duration': 1, 'one_time': True, 'rate': 1.0, 'rate_random': 0.0, 'interval': 0, 'interval_random': 0.0,
             'position_random': 0.0, 'by_distance': False,
             'fade': {'on_stop': True, 'alpha_frames': 0, 'fade_in_frames': 0, 'alpha_fade_in': False, 'scale_fade_in': False}},
            [sampler(0, 'rx_ring')], extra={'ring': {'inner': 0.7, 'segments': 48}}))
    # The runtime loads at most FX_MAX_EMITTERS (32) emitters per package and drops the rest, so a package the converter
    # already filled (Mid/Full/Tap/Super have 50+) would lose anything appended: the demo's emitters go FIRST and the
    # tail is trimmed to the cap (a demo shortcut; a real fix raises the cap or splits the package).
    added = ems[n_orig:]
    d['emitters'] = (added + ems[:n_orig])[:32]
    j.write_text(json.dumps(d), encoding='utf-8')
    return name


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('src'); ap.add_argument('dst')
    ap.add_argument('--tint', default='0.55,0.8,1.0'); ap.add_argument('--radius', type=float, default=2.5)
    ap.add_argument('--no-events', action='store_true'); ap.add_argument('--skip', default='', help='comma list: glare,soft,new (ablation)')
    a = ap.parse_args()
    SKIP.update(x for x in a.skip.split(',') if x)
    src, dst = Path(a.src), Path(a.dst)
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    tint = [float(x) for x in a.tint.split(',')]
    fx = dst / 'goku-xv2/fx'
    done = [adapt_package(p, tint, a.radius) for p in sorted(fx.glob('P_TrailGokuXKame*')) if p.is_dir()]
    if not a.no_events:
        bj = fx / 'fx_bindings.json'
        b = json.loads(bj.read_text(encoding='utf-8'))
        for s in b['states']:
            if s['state'] in ('KameTap', 'KameMid', 'KameFull', 'KameSuper'):
                f = next(c['frame'] for c in s['calls'] if 'Muzzle' in c['package'])
                big = s['state'] in ('KameFull', 'KameSuper')
                s['calls'].append({'frame': f, 'shake': {'amp': 0.012 if big else 0.008, 'frames': 16 if big else 10, 'seed': 5},
                                   'flash': {'color': [0.85, 0.95, 1.0], 'alpha': 0.5 if big else 0.3, 'frames': 5}})
        bj.write_text(json.dumps(b, indent=1), encoding='utf-8')
    print('adapted', len(done), 'packages:', ', '.join(done))


if __name__ == '__main__':
    main()
