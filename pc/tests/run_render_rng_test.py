"""Compile real production RNG/scope functions for a portable native regression.

Only Replay_RandTrace (diagnostics) is stubbed. The scope is extracted from the
native replay TU to avoid linking its unrelated disc/netplay dependencies.
"""
import argparse
from pathlib import Path
import subprocess


def function(text, name):
    start = text.index("void " + name + "(")
    brace = text.index("{", start)
    depth = 1
    end = brace + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[start:end]


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--clang", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--target", default="i686-pc-windows-msvc")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    args.out.mkdir(parents=True, exist_ok=True)
    inc = args.out / "render_rng_production.inc"
    inc.write_text(
        '#define _random_h_\n#define TARGET_PC 1\n'
        'typedef uint32_t u32; typedef int32_t s32; typedef float f32;\n'
        's32 HSD_Rand(void); f32 HSD_Randf(void); s32 HSD_Randi(s32);\n'
        '#include "' + (root / 'src/sysdolphin/baselib/random.c').as_posix() + '"\n'
        '#define gw_HSD_RandRenderBegin HSD_RandRenderBegin\n'
        '#define gw_HSD_RandRenderEnd HSD_RandRenderEnd\n'
        'static int gw_det_in_render; static uint32_t gw_det_render_serial;\n' +
        function((root / 'pc/platform/gw_replay.c').read_text(), 'gw_Det_SetInRender') + '\n',
        encoding='utf-8')
    exe = args.out / 'render_rng_test.exe'
    subprocess.run([args.clang, '--target=' + args.target, '-Wno-pointer-to-int-cast',
                    '-I' + str(args.out.resolve()),
                    str(root / 'pc/tests/render_rng_test.c'), '-o', str(exe)], check=True)
    return subprocess.run([str(exe.resolve())]).returncode


if __name__ == '__main__':
    raise SystemExit(main())
