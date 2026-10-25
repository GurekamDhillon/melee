"""Build and run the memory-card oracle (card_oracle.cpp) against the shipping Aurora CARD library.

The oracle links the shipping Aurora build (MSVC x86, the CMake tree in AURORA_BUILD, default
_build/agents/rustgold/ax86m): compile flags are those CMake used for lib/card/CardGciFolder.cpp, the
link line is examples/simple.exe's (as rust/melee_gx/oracle/build.py does). Nothing here is disc-derived.

  python rust/melee_data/oracle/card_oracle.py            build, generate scenarios, run -> tests/data/card_*.txt
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
OUT = os.path.join(ROOT, "rust", "target", "card-oracle")
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
    cc = ninja_block(ninja, r"^build CMakeFiles\\aurora_card\.dir\\lib\\card\\CardGciFolder\.cpp\.obj:.*$")
    ln = ninja_block(ninja, r"^build examples\\simple\.exe:.*$")
    os.makedirs(OUT, exist_ok=True)
    libs = []
    for tok in ln["LINK_LIBRARIES"].split():
        if not tok.startswith(("-", "/")) and os.path.exists(os.path.join(AURORA_BUILD, tok)):
            libs.append('"' + os.path.join(AURORA_BUILD, tok) + '"')
        else:
            libs.append(tok)
    libs.insert(0, '"' + os.path.join(AURORA_BUILD, "aurora_card.lib") + '"')
    bat = os.path.join(OUT, "build.bat")
    with open(bat, "w") as f:
        f.write("@echo off\r\n")
        f.write(f'call "{VCVARS}" >nul || exit /b 1\r\n')
        f.write(f'cd /d "{OUT}"\r\n')
        f.write(
            f'cl /nologo /TP {cc["DEFINES"]} {cc["INCLUDES"]} {cc["FLAGS"]} '
            f'/Foorac.obj /Fdorac.pdb /FS -c "{os.path.join(HERE, "card_oracle.cpp")}" || exit /b 1\r\n'
        )
        f.write(
            f'link /nologo orac.obj {" ".join(libs)} /out:card_oracle.exe /machine:X86 /subsystem:console '
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


def main():
    build()
    if len(sys.argv) > 1 and sys.argv[1] == "build":
        return
    count = os.environ.get("CARD_SCENARIOS", "80")
    scen = subprocess.run([sys.executable, os.path.join(HERE, "card_scenarios.py"), count], capture_output=True, text=True, check=True).stdout
    scen = scen.replace("\r", "")
    open(os.path.join(DATA, "card_scenarios.txt"), "w", newline="\n").write(scen)
    out_lines = []
    exe = os.path.join(OUT, "card_oracle.exe")
    name, script = None, []

    def flush():
        if name is None:
            return
        rootdir = os.path.join(OUT, "sc", "root")
        shutil.rmtree(os.path.join(OUT, "sc"), ignore_errors=True)
        os.makedirs(rootdir)
        sp = os.path.join(OUT, "sc", "script.txt")
        open(sp, "w", newline="\n").write("\n".join(script) + "\n")
        r = subprocess.run([exe, sp, rootdir.replace("\\", "/")], cwd=OUT, capture_output=True, text=True)
        out_lines.append(f"== {name}")
        body = r.stdout.replace("\r", "")
        out_lines.append(body.rstrip("\n"))
        if r.returncode != 0:
            out_lines.append(f"EXIT {r.returncode}")

    for line in scen.split("\n"):
        if line.startswith("=== "):
            flush()
            name, script = line[4:], []
        elif name is not None:
            script.append(line)
    flush()
    open(os.path.join(DATA, "card_oracle.txt"), "w", newline="\n").write("\n".join(out_lines) + "\n")
    print(f"wrote card_oracle.txt ({len(out_lines)} lines)")


if __name__ == "__main__":
    main()
