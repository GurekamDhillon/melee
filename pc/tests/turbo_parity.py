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
    with path.open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    if len(rows) < 120:
        raise AssertionError(f"{path}: only {len(rows)} game frames (need 120)")
    result = [(int(row["frame"]), row["hash"]) for row in rows[:120]]
    if any(frame != result[0][0] + i for i, (frame, _) in enumerate(result)):
        raise AssertionError(f"{path}: game frames are not consecutive")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--iso", type=Path, required=True)
    parser.add_argument("--bash", default="C:/Program Files/Git/bin/bash.exe" if os.path.exists("C:/Program Files/Git/bin/bash.exe") else "bash",
                        help="Git Bash (on Windows, a bare `bash` can be WSL's)")
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    with tempfile.TemporaryDirectory(prefix="melee-turbo-parity-") as tmp:
        traces = []
        for label, extra in (("realtime", []), ("turbo", [])):
            trace = Path(tmp, f"{label}.csv")
            env = os.environ.copy()
            env.update(
                MELEE_SCENE="mode=training;at=match;p1=fox;p2=marth/cpu1;stage=battlefield",
                MELEE_PAD_SCRIPT=str(here / "turbo_parity_pad.txt"),
                MELEE_SCRIPT=str(here / "turbo_parity_quit.lua"),
                MELEE_TURBO_HASHLOG=str(trace),
                MELEE_TURBO_RENDER="8",
                MELEE_TEST_SEED="12345",
                MELEE_SCRIPTS="0",
                MELEE_CARD="0",
                MELEE_WINDOW_X="0",
                MELEE_WINDOW_Y="0",
                MELEE_WINDOW_W="640",
                MELEE_WINDOW_H="480",
                MELEE_INPUT="none",
                MELEE_VOLUME="3",
            )
            command = [args.bash, str(args.runner)]
            command += extra + [f"turbo-parity-{label}", "--iso", str(args.iso)]
            if label == "turbo":
                env["MELEE_TURBO"] = "1"
            else:
                env["MELEE_TURBO"] = "0"
            subprocess.run(command, env=env, check=True, timeout=90)
            traces.append(read_hashes(trace))
        for a, b in zip(*traces):
            if a != b:
                raise AssertionError(f"frame {a[0]}: realtime {a[1]}, turbo {b[1]}")
        print(f"PASS: {len(traces[0])} consecutive game-frame state hashes match")


if __name__ == "__main__":
    main()
