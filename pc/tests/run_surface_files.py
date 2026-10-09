"""Native file-loader regression; no game, disc or renderer needed.

Run on either platform: python pc/tests/run_surface_files.py --cc clang
Outputs and fixtures stay below --build-dir (default .omo/surface-files).
"""
import argparse
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--cc", default=os.environ.get("CC", "clang"))
parser.add_argument("--build-dir", type=Path, default=ROOT / ".omo/surface-files")
args = parser.parse_args()
build = args.build_dir.resolve()
build.mkdir(parents=True, exist_ok=True)
lua = ROOT / "pc/third_party/lua-5.4.7/src"
sources = sorted(p for p in lua.glob("*.c") if p.name not in ("lua.c", "luac.c"))
exe = build / ("surface-files.exe" if os.name == "nt" else "surface-files")
command = [args.cc, "-std=gnu11", "-O0", "-g", str(ROOT / "pc/tests/surface_file_test.c"),
           *map(str, sources), "-o", str(exe)]
if os.name != "nt":
    command += ["-lm"]
print(shlex.join(command), flush=True)
subprocess.run(command, cwd=ROOT, check=True)
with tempfile.TemporaryDirectory(prefix="fixture-", dir=build) as folder:
    parent = Path(folder)
    mod = parent / "mod"
    shaders = mod / "shaders"
    shaders.mkdir(parents=True)
    (mod / "scripts").mkdir()
    (mod / "scripts/main.lua").write_text("-- calling script\n")
    (mod / "mod.json").write_text("{}")
    body = b"fn gd_surface(c: vec4f, s: GdSurfaceInput) -> vec4f { return c; }\n"
    (shaders / "probe.wgsl").write_bytes(body)
    (shaders / "limit.wgsl").write_bytes(body + b" " * (65536 - len(body)))
    (shaders / "large.wgsl").write_bytes(body + b" " * (65537 - len(body)))
    (shaders / "empty.wgsl").write_bytes(b"")
    (shaders / "nul.wgsl").write_bytes(body + b"\0")
    (shaders / "directory.wgsl").mkdir()
    sibling = parent / "mod-other"
    sibling.mkdir()
    (sibling / "probe.wgsl").write_bytes(body)
    (parent / "outside.wgsl").write_bytes(body)
    (parent / "orphan").mkdir()
    (parent / "orphan/main.lua").write_text("-- no mod\n")
    if os.name != "nt":
        (shaders / "escape.wgsl").symlink_to(parent / "outside.wgsl")
        (shaders / "escape-dir").symlink_to(sibling, target_is_directory=True)
        (shaders / "prefix.wgsl").symlink_to(sibling / "probe.wgsl")
        (shaders / "inside.wgsl").symlink_to(shaders / "probe.wgsl")
        os.mkfifo(shaders / "pipe.wgsl")
    print(shlex.join([str(exe), str(mod)]), flush=True)
    subprocess.run([str(exe), str(mod)], cwd=ROOT, check=True, timeout=30)
