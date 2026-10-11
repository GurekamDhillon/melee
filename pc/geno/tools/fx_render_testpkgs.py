"""Synthetic one-feature test packages for the Geno FX renderer features (no XV2 data): RxSoftTest (R3, RxSoftOff / RxSoftOn), RxDissolveTest
(R4 + R5), RxBeamTest (R2 span ribbon + R1 glare).

    python fx_render_testpkgs.py OUT_FX_DIR TEMPLATE_EMITTER_JSON

OUT_FX_DIR is a mod's `fx/` directory (e.g. .../mods/goku-xv2/fx); TEMPLATE_EMITTER_JSON is any emitter in the IR's shape
(ports/goku-xv2/tools/fx_emitter_template.json in the workspace). Play them with gd.fx_world("RxSoftTest", x, y, z, 1, 1)."""
import copy, json, math, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import xv2_fx_render_adapter as A


def circle_tex(path):
    def px(x, y):
        d = math.hypot((x - 63.5) / 64.0, (y - 63.5) / 64.0)
        a = max(0.0, 1.0 - d) ** 0.7
        return (255, 255, 255, int(255 * min(1.0, a * 1.4)))
    A.png(path, 128, 128, px)


def pkg(name, textures, emitters):
    return {'geno_fx': 1, 'name': name, 'space': {'forward': [0, 0, 1], 'up': [0, 1, 0]},
            'textures': [A.tex_entry(t) for t in textures], 'meshes': [], 'emitters': emitters}


def emission(start=0, duration=1, one_time=True, interval=0):
    return {'start': start, 'duration': duration, 'one_time': one_time, 'rate': 1.0, 'rate_random': 0.0, 'interval': interval,
            'interval_random': 0.0, 'position_random': 0.0, 'by_distance': False,
            'fade': {'on_stop': False, 'alpha_frames': 0, 'fade_in_frames': 0, 'alpha_fade_in': False, 'scale_fade_in': False}}


def main():
    out = Path(sys.argv[1])
    t = json.load(open(sys.argv[2], encoding='utf-8'))
    def write(name, textures, emitters):
        d = out / name
        (d / 'tex').mkdir(parents=True, exist_ok=True)
        A.make_textures(d / 'tex')
        circle_tex(d / 'tex' / 'rx_circle.png')
        (d / f'{name}.gfx.json').write_text(json.dumps(pkg(name, textures, emitters)), encoding='utf-8')
    flat = [[1, 1, 1, 0.0], [1, 1, 1, 1.0]]
    solid = [[1.0, 0, 0, 0.0], [1.0, 0, 0, 1.0]]
    # R3: two identical big soft discs sitting in the floor; the left is hard (soft off), the right has soft_particle 3
    def disc(name, x, soft):
        e = A.build_emitter(t, name, 40, 22.0, flat, solid, [1.0, 0.6, 0.2], 1.4, [x, 2.0, 0.0], [0, 0, 0],
                            emission(), [A.sampler(0, 'rx_circle')])
        e['material']['blend'] = 'alpha'
        e['material']['bloom'] = {'threshold': 9.0, 'intensity': 0.0}
        if soft:
            e['material']['soft_particle'] = {'distance': 3.0, 'volume': 1}
        return e
    write('RxSoftOff', ['rx_circle'], [disc('hard', 0.0, False)])   # same disc, soft_particle off ...
    write('RxSoftOn', ['rx_circle'], [disc('soft', 0.0, True)])     # ... and on (distance 3): compare the two shots
    # R4 + R5: a dissolving ring (noise mask, glowing edge) beside a plain ring and a dissolving sprite
    dis = {'type': 'dissolve', 'mask': 1, 'edge': 0.25, 'edge_color': [1.0, 0.9, 0.5]}
    grow = [[0.2, 0.2, 0.2, 0.0], [1.0, 1.0, 1.0, 1.0]]
    keys = [[0.0, 0, 0, 0.0], [1.0, 0, 0, 0.1], [1.0, 0, 0, 0.9], [0.0, 0, 0, 1.0]]
    ring_em = emission(0, 0, False, 70)
    def ring(name, x, shader, z=0.0):
        e = A.build_emitter(t, name, 60, 20.0, grow, keys, [0.4, 0.8, 1.0], 2.5, [x, 12.0, z], [0, 0, 0], copy.deepcopy(ring_em),
                            [A.sampler(0, 'rx_ring'), A.sampler(1, 'rx_noise', (0.012, -0.018))], shader=shader,
                            param=[[0.05, 0, 0, 0.0], [0.95, 0, 0, 1.0]] if shader else None,
                            extra={'ring': {'inner': 0.6, 'segments': 64}})
        return e
    spr = A.build_emitter(t, 'dissolve_sprite', 60, 14.0, [[1, 1, 1, 0.0], [1, 1, 1, 1.0]], keys, [1.0, 0.7, 0.3], 2.0,
                          [0.0, 12.0, 0.0], [0, 0, 0], copy.deepcopy(ring_em),
                          [A.sampler(0, 'rx_circle'), A.sampler(1, 'rx_noise', (0.01, 0.015))], shader=dis,
                          param=[[0.05, 0, 0, 0.0], [0.95, 0, 0, 1.0]])
    write('RxDissolveTest', ['rx_circle', 'rx_ring', 'rx_noise'], [ring('ring_dissolve', -16.0, dis), ring('ring_plain', 16.0, None), spr])
    # R2 + R1: one span-ribbon beam with eased ends, with and without a tinted wide glare
    def beam(name, y, bloom, width):
        e = A.build_emitter(t, name, 200, width, flat, [[1.0, 0, 0, 0.0], [1.0, 0, 0, 1.0]], [0.3, 0.7, 1.0], 3.0, [-20.0, y, 0.0],
                            [0, math.pi / 2, 0], emission(), [A.sampler(0, 'rx_streak', (-0.04, 0.0))],
                            param=[[1.0, 0, 0, 0.0], [1.0, 0, 0, 1.0]],
                            extra={'ribbon': {'taper': 1.0, 'uv_per_unit': 0.05, 'span': {'length': 40.0, 'segments': 28},
                                              'ends': [0.12, 0.12]}})
        e['material']['bloom'] = bloom
        return e
    write('RxBeamTest', ['rx_streak'], [beam('beam_plain', 22.0, {'threshold': 9.0, 'intensity': 0.0}, 5.0),
                                        beam('beam_glare', 6.0, {'threshold': 0.4, 'intensity': 1.6, 'tint': [0.3, 0.6, 1.0], 'radius': 3.0}, 5.0)])
    print('wrote RxSoftOff RxSoftOn RxDissolveTest RxBeamTest to', out)


if __name__ == '__main__':
    main()
