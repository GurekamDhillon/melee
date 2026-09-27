"""Compare 120 scripted match state hashes at realtime and turbo speeds.

Run after a Windows build, for example:
  py pc/tests/turbo_parity.py --runner C:/gdm/tools/port/run.sh --iso C:/iso/melee.iso
The runner belongs to gd-melee-workspace; see pc/tools/run-turbo.patch.
"""
import argparse
import csv
import os
from pathlib import Path
import subprocess
import tempfile


def read_hashes(path: Path):
    """The PARITY lines turbo_parity_pad.lua logs: (match frame, both fighters' gameplay state)."""
    rows = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        i = line.find("PARITY ")
        if i >= 0:
            frame, rest = line[i + 7:].split(" ", 1)
            rows.append((int(frame), rest.strip()))
    if len(rows) < 120:
        raise AssertionError(f"{path}: only {len(rows)} game frames (need 120)")
    result = rows
    if any(frame != result[0][0] + i for i, (frame, _) in enumerate(result)):
        raise AssertionError(f"{path}: game frames are not consecutive")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--iso", type=Path, required=True)
    parser.add_argument("--bash", default="C:/Program Files/Git/bin/bash.exe" if os.path.exists("C:/Program Files/Git/bin/bash.exe") else "bash",
                        help="Git Bash (on Windows, a bare `bash` can be WSL's)")
    parser.add_argument("--modes", default="realtime,turbo", help="two of realtime/turbo, e.g. realtime,realtime to check the trace itself")
    parser.add_argument("--keep", type=Path, help="copy the two traces here")
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    with tempfile.TemporaryDirectory(prefix="melee-turbo-parity-") as tmp:
        traces = []
        for i, mode in enumerate(args.modes.split(",")):
            label, extra = f"{mode}{i}", []
            trace = Path(tmp, f"{label}.csv")
            env = os.environ.copy()
            env.update(
                MELEE_SCENE="mode=vs;p1=fox;p2=marth/cpu1;stage=battlefield",
                MELEE_PAD_SCRIPT=str(here / "turbo_parity_pad.lua"),
                MELEE_SCRIPT=str(here / "turbo_parity_quit.lua"),
                MELEE_TURBO_RENDER="8",
                MELEE_TEST_SEED="12345",
                MELEE_SCRIPTS="0",
                MELEE_WINDOW_X="0",
                MELEE_WINDOW_Y="0",
                MELEE_WINDOW_W="640",
                MELEE_WINDOW_H="480",
                MELEE_INPUT="none",
                MELEE_VOLUME="3",
            )
            command = [args.bash, str(args.runner)]
            command += extra + [f"turbo-parity-{label}", "--iso", str(args.iso)]
            if mode == "turbo":
                env["MELEE_TURBO"] = "1"
            else:
                env["MELEE_TURBO"] = "0"
            subprocess.run(command, env=env, check=True, timeout=90)
            trace = Path(os.environ["GW_BUILD_ROOT"], "runs", f"turbo-parity-{label}", "melee-pc.log")
            traces.append(read_hashes(trace))
            if args.keep:
                args.keep.mkdir(parents=True, exist_ok=True)
                Path(args.keep, f"{label}.log").write_bytes(trace.read_bytes())
        for a, b in zip(*traces):
            if a != b:
                raise AssertionError(f"frame {a[0]}: realtime {a[1]}, turbo {b[1]}")
        print(f"PASS: {min(len(t) for t in traces)} consecutive game frames of fighter state match")


if __name__ == "__main__":
    main()
