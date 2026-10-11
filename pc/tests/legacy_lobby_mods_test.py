"""Compile and run the legacy lobby's real builders without a game or window.

The frontend is one PPC TU with retail dependencies. Extract its file-scope
builders, as the workspace's script_policy_fixture.py does for native tests;
only netplay, layout IO and the drawing sink are stand-ins in the C fixture.
"""
import argparse
import os
from pathlib import Path
import re
import subprocess


def function(source, name):
    match = re.search(
        rf"^static [^;\n]*\b{re.escape(name)}\([^;]*?\)\n\{{.*?^\}}",
        source, re.MULTILINE | re.DOTALL,
    )
    if match is None:
        raise ValueError(f"frontend function missing: {name}")
    return match.group()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--clang", required=True, type=Path)
    parser.add_argument("--build-root", required=True, type=Path)
    args = parser.parse_args()
    melee = Path(__file__).resolve().parents[2]
    build = args.build_root.resolve()
    if not build.is_relative_to(melee):
        parser.error("build root must stay inside this game checkout")
    build.mkdir(parents=True, exist_ok=True)
    source = (melee / "src/melee/gm/gmfrontend_online.inc").read_text()
    # Read the actual signature capacity, so a truncated fifth row is caught.
    capacity = re.search(r"char sig\[(\d+)\]", source).group(1)
    grid = source[source.index("enum { FL_GRID_TEMPLATE"):
                  source.index("static void fl_grid_head(")]
    names = ("fl_sig_add", "fl_sig_int", "fl_grid_head", "fl_grid_layout",
             "fl_signature", "fl_build_lobby", "fl_mouse_tile")
    (build / "legacy_lobby_capacity.h").write_text(f"#define LEGACY_SIG_CAP {capacity}\n")
    fixture = grid + "\n"
    fixture += "\n\n".join(function(source, name) for name in names) + "\n"
    (build / "legacy_lobby_functions.inc").write_text(fixture)
    exe = build / "legacy-lobby-mods.exe"
    temporary = build / "legacy-lobby-mods.next.exe"
    cmd = [str(args.clang), "--target=i686-pc-windows-msvc", "-std=c11",
           "-Wall", "-Wextra", "-ffunction-sections", "-I", str(build),
           "-I", str(melee / "pc/tests"),
           str(melee / "pc/tests/legacy_lobby_mods_test.c"),
           "-o", str(temporary), "-Xlinker", "/OPT:REF"]
    subprocess.run(cmd, check=True)
    os.replace(temporary, exe)
    return subprocess.run([str(exe)], check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
