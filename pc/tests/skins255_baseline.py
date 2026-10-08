"""Compare the same lane against integration HEAD without touching git metadata.

Temporarily restore tracked source bytes from git show; restore working edits even
on a build failure. Run only when no other process is building or reading tests.
"""
from pathlib import Path
import subprocess

workspace = Path(__file__).resolve().parents[4]
game = workspace / "worktrees/skins255"
logs = workspace / "_build/agents/skins255/skins255-logs"
names = subprocess.check_output(["git", "-C", str(game), "diff", "--name-only"], text=True).splitlines()
saved = {name: (game / name).read_bytes() for name in names}
try:
    for name in names:
        (game / name).write_bytes(subprocess.check_output(["git", "-C", str(game), "show", f"HEAD:{name}"]))
    with (logs / "native-baseline-summary.txt").open("wb") as log:
        subprocess.run(["C:/Program Files/Git/usr/bin/bash.exe", "worktrees/skins255/pc/tests/skins255_suite.sh", "baseline"], cwd=workspace, stdout=log, stderr=subprocess.STDOUT)
    for command, output in [("build", "build-baseline.log"), ("baseline", "headless-baseline.log")]:
        with (logs / output).open("wb") as log:
            result = subprocess.run(["C:/Program Files/Git/usr/bin/bash.exe", "worktrees/skins255/pc/tests/skins255_run.sh", command], cwd=workspace, stdout=log, stderr=subprocess.STDOUT)
        print(f"integration {command}: exit {result.returncode}", flush=True)
        if command == "build" and result.returncode:
            raise SystemExit(result.returncode)
finally:
    for name, contents in saved.items():
        (game / name).write_bytes(contents)
    print("skins255 edits restored; final build required", flush=True)
