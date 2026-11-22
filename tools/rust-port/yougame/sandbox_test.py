#!/usr/bin/env python3
"""The YouGame sandbox's rules, checked on a real run (docs/rust-port/notes/yougame.md, section 9).

A hidden YouGame-style launch, as the app makes it: the bridge on (YOUGAME_STDIO=1, the fake account of
bridge_test.py), YOUGAME_DATA a temp folder that is also the working directory, the args from yougame-native.json
with {build}/{data}/{disc} filled in, and a build folder that is a copy of the exe with write access DENIED to this
user (icacls), like the app's read-only build folder. The run plays the retail pad script into a short match and quits.
Then it asserts:
  1. nothing was written outside YOUGAME_DATA: the build folder, its parent and the exe's real folder are unchanged
     (file lists, sizes and times), and no melee* entry appeared in %TEMP%;
  2. no loopback socket was opened: the process's TCP/UDP endpoints (netstat -ano, polled every 200 ms) never had a
     127.0.0.0/8 or ::1 local or remote address;
  3. the run reached a match (the log), so the check covered the play path, and the data folder got its files.
Usage: python tools/rust-port/yougame/sandbox_test.py [--exe PATH] [--frames N]
Agent rules: hidden window, volume 3, pad script (no OS input), MELEE_PAD_IGNORE_ADAPTER=1, one instance, hard timeout.
"""
import argparse, os, re, shutil, subprocess, sys, tempfile, threading, time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bridge_test as bt  # noqa: E402

LOOP = re.compile(r"^(127\.|\[::1\]|::1)")


def snapshot(root):
    out = {}
    if not root.exists():
        return out
    for dp, dns, fns in os.walk(root):
        for n in dns + fns:
            p = Path(dp) / n
            try:
                st = p.stat()
                out[str(p.relative_to(root))] = (p.is_dir(), 0 if p.is_dir() else st.st_size, 0 if p.is_dir() else st.st_mtime_ns)
            except OSError:
                pass
    return out


def top_level(root):
    try:
        return {n: (root / n).stat().st_mtime_ns for n in os.listdir(root)}
    except OSError:
        return {}


def endpoints(pid):
    """(proto, local, remote) for every socket the process owns now."""
    r = subprocess.run(["netstat", "-ano"], capture_output=True, text=True, errors="replace")
    out = []
    for line in r.stdout.splitlines():
        f = line.split()
        if len(f) >= 4 and f[0] in ("TCP", "UDP") and f[-1] == str(pid):
            out.append((f[0], f[1], f[2] if f[0] == "TCP" or len(f) == 4 else f[2]))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--exe", default=str(bt.REPO / "build/cargo/release/melee.exe"))
    ap.add_argument("--frames", type=int, default=2400)
    ap.add_argument("--timeout", type=float, default=240.0)
    a = ap.parse_args()
    iso = bt.iso_path()
    if not iso or not Path(iso).exists():
        print("sandbox_test: GW_ISO_VANILLA not set or missing", file=sys.stderr)
        return 2
    spec, why = bt.check_spec(bt.SPEC.read_text(encoding="utf-8"))
    if spec is None:
        print("sandbox_test: yougame-native.json: " + why)
        return 1
    fails = []

    def check(ok, what):
        print(("PASS " if ok else "FAIL ") + what.replace(iso, "<iso>"))
        if not ok:
            fails.append(what)

    exe_real = Path(a.exe).resolve()
    tmp = Path(tempfile.mkdtemp(prefix="yg-sandbox-"))
    build = tmp / "build"
    data = tmp / "data"
    build.mkdir()
    data.mkdir()
    shutil.copy2(exe_real, build / exe_real.name)
    for extra in ("version.txt", "melee.snapmask"):
        if (exe_real.parent / extra).exists():
            shutil.copy2(exe_real.parent / extra, build / extra)
    user = os.environ.get("USERNAME", "")
    # the folder: no new files or folders, no deletes (an inherited deny on the exe itself stops it from running);
    # its files: the read-only attribute
    subprocess.run(["icacls", str(build), "/deny", f"{user}:(WD,AD,DC)"], capture_output=True, check=True)
    for f in build.iterdir():
        subprocess.run(["attrib", "+R", str(f)], capture_output=True)
    try:
        probe_ok = True
        try:
            (build / "probe.txt").write_text("x")
        except OSError:
            probe_ok = False
        check(not probe_ok, "the build folder is read-only to this user (a probe write fails)")

        before = {"build": snapshot(build), "exe_dir": top_level(exe_real.parent)}
        tmp_root = Path(os.environ.get("TEMP", tempfile.gettempdir()))
        temp_before = {n for n in os.listdir(tmp_root) if n.lower().startswith("melee")}

        args = bt.fill(spec["args"], str(build), str(data), iso) + [
            "--skip-intro", "--no-audio", "--time-secs", "10", "--max-frames", str(a.frames)]
        env = {"MELEE_INPUT": "n", "MELEE_PAD_IGNORE_ADAPTER": "1", "MELEE_WINDOW_HIDDEN": "1", "MELEE_VOLUME": "3",
               "MELEE_NO_DIALOG": "1"}
        p, lines, err = bt.spawn(str(build / exe_real.name), args, data, "ygaudit sandbox_test", extra_env=env)
        seen, stop = [], threading.Event()

        def watch():
            while not stop.is_set():
                for e in endpoints(p.pid):
                    if e not in seen:
                        seen.append(e)
                stop.wait(0.2)

        threading.Thread(target=watch, daemon=True).start()
        code, dt = bt.wait_exit(p, a.timeout)
        stop.set()
        time.sleep(0.5)
        check(code == 0, "the run ended by itself: exit %s after %.1f s" % (code, dt))
        check(any(re.search(r"scene vs \(", l) for l in err),
              "the run reached a match (log)")

        after = {"build": snapshot(build), "exe_dir": top_level(exe_real.parent)}
        added = sorted(set(after["build"]) - set(before["build"]))
        changed = sorted(k for k in before["build"] if after["build"].get(k) != before["build"][k])
        check(not added and not changed, "build folder unchanged (added %s, changed %s)" % (added, changed))
        ex_added = sorted(set(after["exe_dir"]) - set(before["exe_dir"]))
        check(not ex_added, "the real exe's folder got nothing new (%s)" % ex_added)
        temp_new = sorted({n for n in os.listdir(tmp_root) if n.lower().startswith("melee")} - temp_before)
        check(not temp_new, "no melee* entry appeared in %%TEMP%% (%s)" % temp_new)
        wrote = sorted(str(Path(k)) for k, v in snapshot(data).items() if not v[0])
        check(len(wrote) > 0, "YOUGAME_DATA got the run's files: %s" % wrote[:20])

        loop = [e for e in seen if LOOP.match(e[1]) or LOOP.match(e[2])]
        print("sockets seen: %s" % seen)
        check(not loop, "no loopback socket (%d sockets seen, loopback %s)" % (len(seen), loop))
        if fails:
            print("\n".join(err[-40:]))
    finally:
        subprocess.run(["icacls", str(build), "/remove:d", user], capture_output=True)
        for f in build.iterdir():
            subprocess.run(["attrib", "-R", str(f)], capture_output=True)
        shutil.rmtree(tmp, ignore_errors=True)
    print("SANDBOX " + ("FAIL (%d)" % len(fails) if fails else "PASS"))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main())
