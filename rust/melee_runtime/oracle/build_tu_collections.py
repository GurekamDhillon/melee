"""Native C oracle for id.c/list.c (synthetic nodes only, no disc data)."""
import os
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = ROOT / 'rust/target/shard-945-952-collections'
OUT.mkdir(parents=True, exist_ok=True)
CLANG = os.environ.get('CLANG', r'C:\Program Files\LLVM\bin\clang.exe')
flags = ['--target=x86_64-pc-windows-msvc', '-std=c99', '-nostdinc', '-fno-builtin',
         '-DLINT', '-DTARGET_PC', '-fno-short-enums', '-fsigned-char', '-mlong-double-64',
         '-fno-strict-aliasing', '-fwrapv', '-fcommon', '-fgnu89-inline',
         '-ftrivial-auto-var-init=zero', '-O2', '-ffp-contract=on', '-Xclang',
         '-disable-llvm-passes', '-S', '-emit-llvm', '-w', '-Isrc', '-Ipc',
         '-Ipc/gameworld', '-isystem', 'src/MSL', '-isystem', 'libs/dolphin/include',
         '-isystem', 'libs/dolphin/src',
         '-D_Static_assert(a,b)=', '-Dsize_t=gc_size_t']
subprocess.run([CLANG, *flags, str(HERE / 'tu_collections.c'), '-o', str(OUT / 'ref.ll')], cwd=ROOT, check=True)
subprocess.run([sys.executable, str(HERE / 'lower.py'), str(OUT / 'ref.ll'), str(OUT / 'ref.lo.ll')], check=True)
subprocess.run([CLANG, '-O2', '-ffp-contract=off', str(OUT / 'ref.lo.ll'), '-o', str(OUT / 'ref.exe')], check=True)
result = subprocess.run([str(OUT / 'ref.exe')], check=True, capture_output=True)
dest = HERE.parent / 'tests/data/tu_collections.txt'
dest.write_bytes(result.stdout)
print(f'{dest}: {len(result.stdout.splitlines())} C state rows')
