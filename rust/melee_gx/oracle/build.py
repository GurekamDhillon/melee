"""Build and run the melee_gx Aurora oracle (oracle.cpp), writing tests/data/*.

The oracle links the shipping Aurora build (MSVC x86, the CMake tree recorded in AURORA_BUILD, default
_build/agents/rustgold/ax86m, built from this worktree's extern/aurora): compile flags are those CMake
used for lib/gx/shader.cpp, the link line is examples/simple.exe's. Nothing here is disc-derived.

  python rust/melee_gx/oracle/build.py            build + tex + shader + state + render
  python rust/melee_gx/oracle/build.py build      only build
  python rust/melee_gx/oracle/build.py dump       also dump every WGSL source to rust/target/gx-oracle/wgsl/

The render mode opens a 640x480 window (Aurora needs one) for about a second.
"""
import os
import re
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CRATE = os.path.dirname(HERE)
ROOT = os.path.abspath(os.path.join(CRATE, "..", ".."))
WS = os.path.abspath(os.path.join(ROOT, "..", ".."))
AURORA_BUILD = os.environ.get("AURORA_BUILD", os.path.join(WS, "_build", "agents", "rustgold", "ax86m"))
OUT = os.path.join(ROOT, "rust", "target", "gx-oracle")
VCVARS = os.environ.get(
    "GW_VCVARS32", r"C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars32.bat"
)
DATA = os.path.join(CRATE, "tests", "data")


def ninja_block(text, header_re):
    m = re.search(header_re, text, re.M)
    if not m:
        sys.exit(f"build.ninja: no match for {header_re}")
    block = text[m.end():].split("\nbuild ", 1)[0]
    vars_ = {}
    for line in block.splitlines():
        line = line.strip()
        if " = " in line:
            k, v = line.split(" = ", 1)
            vars_[k] = v
    return vars_


def build():
    ninja = open(os.path.join(AURORA_BUILD, "build.ninja"), encoding="utf-8").read()
    cc = ninja_block(ninja, r"^build CMakeFiles\\aurora_gx\.dir\\lib\\gx\\shader\.cpp\.obj:.*$")
    ln = ninja_block(ninja, r"^build examples\\simple\.exe:.*$")
    os.makedirs(OUT, exist_ok=True)
    libs = []
    for tok in ln["LINK_LIBRARIES"].split():
        if not tok.startswith(("-", "/")) and os.path.exists(os.path.join(AURORA_BUILD, tok)):
            libs.append('"' + os.path.join(AURORA_BUILD, tok) + '"')
        else:
            libs.append(tok)
    aurora_lib = os.path.join(ROOT, "extern", "aurora", "lib")
    bat = os.path.join(OUT, "build.bat")
    with open(bat, "w") as f:
        f.write("@echo off\r\n")
        f.write(f'call "{VCVARS}" >nul || exit /b 1\r\n')
        f.write(f'cd /d "{OUT}"\r\n')
        f.write(
            f'cl /nologo /TP {cc["DEFINES"]} {cc["INCLUDES"]} -I"{aurora_lib}" -I"{HERE}" {cc["FLAGS"]} '
            f'/Fooracle.obj /Fdoracle.pdb /FS -c "{os.path.join(HERE, "oracle.cpp")}" || exit /b 1\r\n'
        )
        f.write(
            f'link /nologo oracle.obj {" ".join(libs)} /out:gx_oracle.exe /machine:X86 /subsystem:console '
            f"/debug /INCREMENTAL:NO || exit /b 1\r\n"
        )
    subprocess.run(["cmd", "/c", bat], check=True)
    for dll in ("webgpu_dawn.dll", "SDL3.dll"):
        for cand in (
            os.path.join(AURORA_BUILD, "_deps", "dawn-build", dll),
            os.path.join(AURORA_BUILD, "_deps", "sdl3_prebuilt-src", "bin", dll),
            os.path.join(AURORA_BUILD, "..", dll),
        ):
            if os.path.exists(cand):
                shutil.copy2(cand, os.path.join(OUT, dll))
                break


def run(mode, *args, out=None):
    exe = os.path.join(OUT, "gx_oracle.exe")
    r = subprocess.run([exe, mode, *args], cwd=OUT, capture_output=True, text=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr[-4000:])
        sys.exit(f"oracle {mode} failed ({r.returncode})")
    if out:
        lines = [l for l in r.stdout.splitlines() if l[:1] in "#TPCWOSU"]
        with open(out, "w", newline="\n") as f:
            f.write("\n".join(lines) + "\n")
        print(f"wrote {os.path.relpath(out, ROOT)} ({len(lines)} lines)")


def main():
    build()
    if len(sys.argv) > 1 and sys.argv[1] == "build":
        return
    os.makedirs(DATA, exist_ok=True)
    run("tex", out=os.path.join(DATA, "tex_oracle.txt"))
    dump = []
    if len(sys.argv) > 1 and sys.argv[1] == "dump":
        d = os.path.join(OUT, "wgsl")
        os.makedirs(d, exist_ok=True)
        dump = ["400", d]
    run("shader", *(dump or ["400"]), out=os.path.join(DATA, "shader_oracle.txt"))
    run("state", "300", out=os.path.join(DATA, "state_oracle.txt"))
    png = os.path.join(DATA, "render_aurora.png")
    run("render", png)
    print(f"wrote {os.path.relpath(png, ROOT)}")


if __name__ == "__main__":
    main()
