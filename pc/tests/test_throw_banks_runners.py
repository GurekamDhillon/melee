"""Exercise runner verdicts without starting a game or using disc data."""
import os
from pathlib import Path
import shutil
import subprocess

import pytest


HERE = Path(__file__).resolve().parent
BASH = shutil.which("bash")
pytestmark = pytest.mark.skipif(not BASH, reason="Git Bash is required")


def run_fixture(tmp_path, script, log, exit_code=0, banks=1):
    root = tmp_path / "workspace"
    harness = root / "tools/geno/moves_check/run_scene.sh"
    harness.parent.mkdir(parents=True)
    harness.write_text(
        '#!/bin/bash\nmkdir -p "$GW_BUILD_ROOT/runs/$1"\n'
        'printf "%s\\n" "$TBR_TEST_LOG" > "$GW_BUILD_ROOT/runs/$1/melee-pc.log"\n'
        'exit "$TBR_TEST_EXIT"\n', encoding="utf-8"
    )
    build = tmp_path / "build"
    (build / "mods").mkdir(parents=True)
    env = dict(os.environ, GW_ROOT=root.as_posix(), GW_BUILD_ROOT=build.as_posix(),
               TBR_TEST_LOG=log, TBR_TEST_EXIT=str(exit_code))
    return subprocess.run([BASH, str(HERE / script), "fixture", "3000", str(banks)]
                          if "cpu" in script else [BASH, str(HERE / script), "fixture"],
                          env=env, capture_output=True, text=True)


REWIND = "\n".join(f"TBR {case} case PASS detail" for case in "ABCD") + "\nTBR DONE 4/4"
CPU = ("read into a throw-bank arena\nevicted kind 126\nTBC DONE frames=3000\n"
       "snap: curated hash: 37200 compared, 0 mismatching, resim iteration (logic only) 1.00 ms")


@pytest.mark.parametrize("script,log", [
    ("run_throw_banks_rewind.sh", REWIND),
    ("run_throw_banks_cpu_synctest.sh", CPU),
])
def test_accepts_complete_proof(tmp_path, script, log):
    result = run_fixture(tmp_path, script, log)
    assert result.returncode == 0, result.stdout + result.stderr


@pytest.mark.parametrize("script,log", [
    ("run_throw_banks_rewind.sh", REWIND.replace("4/4", "3/4")),
    ("run_throw_banks_rewind.sh", REWIND.replace("B case PASS", "B case FAIL")),
    ("run_throw_banks_rewind.sh", REWIND.replace("TBR D case PASS detail\n", "")),
    ("run_throw_banks_rewind.sh", REWIND + "\nTBR ERROR failed restore"),
    ("run_throw_banks_rewind.sh", REWIND + "\nTBR teleport failed port 4"),
    ("run_throw_banks_rewind.sh", REWIND.replace("TBR DONE 4/4", "")),
    ("run_throw_banks_cpu_synctest.sh", CPU.replace("0 mismatching", "1 mismatching")),
    ("run_throw_banks_cpu_synctest.sh", CPU.replace("37200 compared", "0 compared")),
    ("run_throw_banks_cpu_synctest.sh", CPU.replace("frames=3000", "frames=2999")),
    ("run_throw_banks_cpu_synctest.sh", CPU.replace("read into a throw-bank arena", "no loads")),
    ("run_throw_banks_cpu_synctest.sh", CPU.replace("evicted kind 126", "no evictions")),
    ("run_throw_banks_cpu_synctest.sh", CPU.split("snap:")[0]),
    ("run_throw_banks_cpu_synctest.sh", CPU + "\nsnap: curated hash: 20 compared, 3 mismatching"),
    ("run_throw_banks_cpu_synctest.sh", CPU + "\nsnap: CURATED MISMATCH frame 3001"),
])
def test_rejects_incomplete_or_failed_proof(tmp_path, script, log):
    result = run_fixture(tmp_path, script, log)
    assert result.returncode != 0, result.stdout + result.stderr


@pytest.mark.parametrize("script,log", [
    ("run_throw_banks_rewind.sh", REWIND),
    ("run_throw_banks_cpu_synctest.sh", CPU),
])
def test_preserves_game_exit_failure(tmp_path, script, log):
    result = run_fixture(tmp_path, script, log, exit_code=86)
    assert result.returncode != 0, result.stdout + result.stderr


def test_three_arenas_need_not_evict(tmp_path):
    result = run_fixture(tmp_path, "run_throw_banks_cpu_synctest.sh",
                         CPU.replace("evicted kind 126", "all families resident"), banks=3)
    assert result.returncode == 0, result.stdout + result.stderr
