#!/bin/bash
# The YouGame desktop build of the Rust engine (Windows x64), for a "Patches" listing: the player's own disc comes
# from the YouGame app ("disc": true, {disc} in yougame-native.json); nothing of the disc is in the zip.
#   bash tools/rust-port/yougame/package.sh [--skip-build] [--out DIR]
# Builds melee.exe (release, cargo -j 4, own target dir YG_TARGET_DIR, static CRT, paths remapped), lays out
#   melee-rust/melee.exe  yougame-native.json  version.txt  LICENSE.txt (GPL-3, rust/LICENSE)
#   THIRD_PARTY_NOTICES.txt (every crate linked in, with its licence texts)  README.txt
# zips it, then audits the zip: no disc data (GameCube/Wii disc headers, Melee's DAT/USD/HPS file names, big
# binary blobs other than melee.exe), no disc-image paths, no personal paths or user names; prints sha256 + size.
# CARGO_TARGET_DIR is honoured (default: the workspace's shared target).
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
build=1; out="$repo/build/yougame"
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-build) build=0 ;;
    --out) out="$2"; shift ;;
    *) echo "package: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
# A target of its own: the release exe is built with the CRT linked in (no vcruntime140.dll to ship) and every
# build-machine path remapped (panic locations and the PDB path would otherwise carry the user's folders).
target="${YG_TARGET_DIR:-$(cd "$repo/../.." 2>/dev/null && pwd)/_build/cargo-ygpack}"
export CARGO_TARGET_DIR="$target"
unset RUSTFLAGS
win() { cygpath -w "$1" 2>/dev/null || echo "$1"; }
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
rustup_home="${RUSTUP_HOME:-$HOME/.rustup}"
flags=("-Ctarget-feature=+crt-static" "-Clink-arg=/PDBALTPATH:melee.pdb")
ws_root="$(cd "$repo/../.." && pwd)"
for pair in "$cargo_home/registry/src=crates" "$cargo_home=cargo" "$rustup_home=rustup" "$repo=melee" "$target=target" "$ws_root=workspace"; do
  from="$(win "${pair%%=*}")"; to="${pair#*=}"
  fwd="$(printf '%s' "$from" | tr '\\' '/')"
  flags+=("--remap-path-prefix=$from=$to" "--remap-path-prefix=$fwd=$to")
done
CARGO_ENCODED_RUSTFLAGS="$(IFS=$''; echo "${flags[*]}")"
export CARGO_ENCODED_RUSTFLAGS
if [ $build = 1 ]; then
  (cd "$repo/rust" && cargo build -j 4 --release -p melee 2>&1 | tail -2)
fi
exe="$target/release/melee.exe"
[ -f "$exe" ] || { echo "package: no $exe" >&2; exit 1; }

stage="$out/melee-rust"
rm -rf "$stage"; mkdir -p "$stage"
cp "$exe" "$stage/melee.exe"
cp "$here/yougame-native.json" "$stage/yougame-native.json"
cp "$repo/rust/LICENSE" "$stage/LICENSE.txt"
rev="$(git -C "$repo" rev-parse --short=10 HEAD)"
dirty=""; git -C "$repo" diff --quiet HEAD -- rust || dirty="-dirty"
printf 'melee-rust %s%s (%s)\n' "$rev" "$dirty" "$(date -u +%Y-%m-%d)" > "$stage/version.txt"
cat > "$stage/README.txt" <<'EOF'
Melee (Rust engine): a native engine for Super Smash Bros. Melee.

This folder holds no part of the game. It plays on your own copy of
Super Smash Bros. Melee (USA, revision 2 / NTSC 1.02), which the YouGame
app hands to the program when you press Play.

Licence: GNU General Public License v3.0 or later (LICENSE.txt).
Third-party components: THIRD_PARTY_NOTICES.txt.
Not affiliated with or endorsed by Nintendo or HAL Laboratory.
EOF

# Third-party notices: every non-workspace package in the dependency graph of melee (normal deps, this target).
(cd "$repo/rust" && cargo metadata --format-version 1 --filter-platform x86_64-pc-windows-msvc) > "$out/metadata.json"
python - "$out/metadata.json" "$stage/THIRD_PARTY_NOTICES.txt" <<'PY'
import json, sys, os, glob
m = json.load(open(sys.argv[1], encoding="utf-8"))
pk = {p["id"]: p for p in m["packages"]}
ws = set(m["workspace_members"])
nodes = {n["id"]: n for n in m["resolve"]["nodes"]}
root = next(i for i in ws if pk[i]["name"] == "melee")
seen, todo = set(), [root]
while todo:
    i = todo.pop()
    if i in seen: continue
    seen.add(i)
    for d in nodes[i]["deps"]:
        if any(k["kind"] in (None, "normal") for k in d["dep_kinds"]):
            todo.append(d["pkg"])
third = sorted((pk[i] for i in seen if i not in ws and not pk[i]["manifest_path"].replace("\\", "/").split("/rust/")[0].endswith("worktrees/ygpack")),
               key=lambda p: (p["name"], p["version"]))
out = ["Third-party components linked into melee.exe", "=" * 46, "",
       "Each component below is used under its own licence, reproduced after the list.", ""]
for p in third:
    out.append(f"{p['name']} {p['version']}  licence: {p.get('license') or 'see file'}  {p.get('repository') or ''}".rstrip())
out.append("")
for p in third:
    d = os.path.dirname(p["manifest_path"])
    files = sorted(f for f in glob.glob(os.path.join(d, "*")) if os.path.basename(f).upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE", "COPYRIGHT")) and os.path.isfile(f))
    out.append("-" * 78)
    out.append(f"{p['name']} {p['version']} ({p.get('license') or 'licence file'})")
    if not files:
        out.append("(no licence file in the package; licence: " + (p.get("license") or "unknown") + ")")
    for f in files:
        out.append(f"--- {os.path.basename(f)} ---")
        out.append(open(f, encoding="utf-8", errors="replace").read().rstrip())
    out.append("")
open(sys.argv[2], "w", encoding="utf-8", newline="\n").write("\n".join(out) + "\n")
print(f"package: {len(third)} third-party components in THIRD_PARTY_NOTICES.txt")
PY
rm -f "$out/metadata.json"

# DLLs: the exe must need only Windows' own.
python - "$stage/melee.exe" <<'PY'
import struct, sys
b = open(sys.argv[1], "rb").read()
names = set()
import re
for mm in re.finditer(rb"([A-Za-z0-9_\-.]{3,40}\.(?:dll|DLL))\x00", b):
    names.add(mm.group(1).decode().lower())
sysdll = lambda n: n.startswith(("api-ms-win-", "ext-ms-")) or n in {
    "kernel32.dll","user32.dll","gdi32.dll","advapi32.dll","ole32.dll","oleaut32.dll","shell32.dll","ws2_32.dll","bcrypt.dll",
    "ntdll.dll","dwmapi.dll","uxtheme.dll","imm32.dll","winmm.dll","setupapi.dll","hid.dll","winusb.dll","d3d12.dll","dxgi.dll",
    "d3dcompiler_47.dll","opengl32.dll","xinput1_4.dll","xinput9_1_0.dll","comctl32.dll","propsys.dll","mmdevapi.dll","avrt.dll",
    "userenv.dll","secur32.dll","crypt32.dll","ncrypt.dll","shlwapi.dll","cfgmgr32.dll","dbghelp.dll","powrprof.dll","version.dll",
    "vcruntime140.dll","msvcp140.dll","ucrtbase.dll","combase.dll","synchronization.dll","dcomp.dll","d3d11.dll","dxcompiler.dll","dxil.dll",
    "windows.ui.dll","coremessaging.dll","iphlpapi.dll","ntoskrnl.exe","wtsapi32.dll","msimg32.dll","dinput8.dll","xinput1_3.dll","xinputuap.dll","winhttp.dll","rpcrt4.dll"}
other = sorted(n for n in names if not sysdll(n))
print("package: DLL names in the exe:", ", ".join(sorted(names)))
if other:
    print("package: NOTE non-system DLL names referenced (check they are optional or load-on-demand):", ", ".join(other))
PY

zip="$out/melee-rust-windows-x64.zip"
rm -f "$zip"
python - "$stage" "$zip" <<'PY'
import os, sys, zipfile
stage, z = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(z, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as f:
    for d, _, fs in os.walk(stage):
        for n in sorted(fs):
            p = os.path.join(d, n)
            f.write(p, os.path.relpath(p, os.path.dirname(stage)).replace("\\", "/"))
PY

# The audit.
python - "$zip" <<'PY'
import os, re, sys, zipfile, getpass, hashlib
z = sys.argv[1]
bad = []
users = {u for u in {os.environ.get("USERNAME"), os.environ.get("USER"), getpass.getuser()} if u and len(u) > 2}
iso_env = [v for k, v in os.environ.items() if k.startswith("GW_ISO") and v]
dat = re.compile(rb"\b(?:Pl[A-Z][a-z][A-Z][a-z]\.dat|Gr[A-Z][A-Za-z]{1,3}\.dat|Ty[A-Z][a-z]+\.dat|MnSlChr\.usd|MnMaAll\.usd|IfAll\.usd|SdMenu\.usd|ItCo\.usd|LbAd\.dat|EfCoData\.dat|[a-z0-9_]{2,24}\.hps)\b")
with zipfile.ZipFile(z) as f:
    for i in f.infolist():
        b = f.read(i.filename)
        n = i.filename.lower()
        if n.endswith((".iso", ".gcm", ".rvz", ".ciso", ".gcz", ".wbfs", ".wia", ".dat", ".usd", ".hps", ".ssm", ".sem", ".thp", ".dol")):
            bad.append(f"{i.filename}: a disc-type file name")
        for off in range(0, min(len(b), 1 << 26), 0x8000):
            h = b[off:off + 0x20]
            if len(h) == 0x20 and (h[0x1C:0x20] == b"\xC2\x33\x9F\x3D" or h[0x18:0x1C] == b"\x5D\x1C\x9E\xA3"):
                bad.append(f"{i.filename}: a GameCube/Wii disc header at {off:#x}")
                break
        if b"GALE01" in b and not n.endswith(".exe"):
            bad.append(f"{i.filename}: contains GALE01")
        hits = sorted({m.group(0).decode() for m in dat.finditer(b)})
        if hits and not n.endswith(".exe"):
            bad.append(f"{i.filename}: names disc files {hits[:5]}")
        if n.endswith(".exe") and hits:
            print(f"audit: note: melee.exe names {len(hits)} disc file names (the engine looks them up on the player's disc; names, not data)")
        if i.file_size > 1 << 20 and not n.endswith((".exe", ".txt")):
            bad.append(f"{i.filename}: a large binary ({i.file_size} bytes)")
        if i.file_size > 256 << 20:
            bad.append(f"{i.filename}: over 256 MB")
        low = b.lower()
        for u in users:
            for form in (f"\\users\\{u}".lower().encode(), f"/users/{u}".lower().encode(), f"/home/{u}".lower().encode()):
                if form in low:
                    bad.append(f"{i.filename}: a personal path (user folder)")
        for v in iso_env:
            if v.encode() in b or v.replace("\\", "/").encode() in b:
                bad.append(f"{i.filename}: a disc image path")
        if re.search(rb"[A-Za-z]:[\\/][^\x00\n]{0,200}\.(?:iso|gcm)\b", b, re.I):
            bad.append(f"{i.filename}: an absolute .iso/.gcm path")
        for w in (b"gurek", b"gurekam", b"melee workspace"):
            if w in low:
                bad.append(f"{i.filename}: contains '{w.decode()}'")
    print("audit: files:", ", ".join(f"{i.filename} ({i.file_size})" for i in f.infolist()))
if bad:
    print("AUDIT FAIL"); [print("  " + x) for x in sorted(set(bad))]; sys.exit(1)
d = open(z, "rb").read()
print("AUDIT OK")
print(f"zip:    {os.path.basename(z)}")
print(f"size:   {len(d)} bytes ({len(d)/1048576:.1f} MiB)")
print(f"sha256: {hashlib.sha256(d).hexdigest()}")
PY
