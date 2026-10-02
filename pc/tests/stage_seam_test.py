#!/usr/bin/env python3
"""Compile actual seam, floor-follow, dynamic-island and lifetime source with a
synthetic collision map. Allocator/scene objects are fixtures, not live physics.
No shared native build, GPU, installation or game process is used.
"""
from pathlib import Path
import argparse
import re
import subprocess
import tempfile

GAME = Path(__file__).resolve().parents[2]


def function(source, name):
    match = re.search(r'^(?:static\s+(?:inline\s+)?)?(?:int|bool|void|mp_UnkStruct0\*)\s*' +
                      re.escape(name) + r'\s*\(', source, re.M)
    if not match:
        raise AssertionError('missing actual source function ' + name)
    start = source.index('{', match.start())
    # Ignore braces inside comments and string/character literals.
    tokens = re.compile(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[{}]', re.S)
    depth = 0
    for token in tokens.finditer(source, start):
        if token.group() == '{':
            depth += 1
        elif token.group() == '}':
            depth -= 1
            if depth == 0:
                return source[match.start():token.end()] + '\n'
    raise AssertionError('unterminated function ' + name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sanitize', action='store_true')
    args = parser.parse_args()
    groups = [
        ('readers', 'src/melee/mp/mplib.c', ['mpLineGetNext', 'mpLineGetPrev', 'mpLib_8004DD90_Floor', 'mpLib_80054ED8']),
        ('islands', 'src/melee/mp/mpisland.c', ['mpIsland_8005B004', 'mpIsland_8005AB54']),
        ('refresh', 'src/melee/mp/mplib.c', ['mpLib_8005667C', 'mpLib_80057424', 'mpJointListAdd', 'mpJointListUnlink', 'mpLib_80057BC0']),
        ('lifetime', 'pc/gameworld/script_game.c', ['ScriptGame_StageAddLine', 'ScriptGame_StageRemove', 'script_stage_line_set', 'ScriptGame_StageMove', 'ScriptGame_StageJoint']),
        ('stitcher', 'src/melee/mp/mplib.c', ['mpLib_GetJointVtxRange', 'mpLib_800581DC']),
    ]
    with tempfile.TemporaryDirectory(prefix='stage-seam-source-') as temporary:
        build = Path(temporary)
        for group, path, names in groups:
            source = (GAME / path).read_text()
            (build / (group + '.inc')).write_text(''.join(function(source, name) for name in names))
        command = ['cc', '-std=c11', '-Wall', '-Wextra', '-Werror',
                   '-Wno-unused-parameter', '-Wno-unused-variable', '-DTARGET_PC',
                   '-I', str(build), '-I', str(GAME / 'pc/gameworld')]
        if args.sanitize:
            command += ['-fsanitize=address,undefined', '-fno-omit-frame-pointer']
        command += [str(GAME / 'pc/tests/stage_seam_test.c'), '-lm', '-o', str(build / 'test')]
        subprocess.run(command, check=True)
        subprocess.run([str(build / 'test')], check=True)


if __name__ == '__main__':
    main()
