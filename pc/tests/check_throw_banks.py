"""Require complete throw-bank evidence; printed summaries alone are not a verdict."""
import argparse
from pathlib import Path
import re


def check(log, mode, frames, banks):
    if re.search(r"\bTBR[^\n]*\b(?:FAIL|ERROR)\b", log):
        raise ValueError("throw-bank case failed")
    if "TBR teleport failed" in log:
        raise ValueError("rewind scene setup failed")
    if mode == "rewind":
        for case in "ABCD":
            if not re.search(rf"\bTBR {case}\b[^\n]*\bPASS\b", log):
                raise ValueError(f"missing passing rewind case {case}")
        if not re.search(r"\bTBR DONE 4/4\b", log):
            raise ValueError("rewind did not finish with 4/4 cases")
    else:
        # Summaries are periodic; a final-frame mismatch may have no later summary.
        if re.search(r"\b(?:CURATED|SYNCTEST) MISMATCH\b", log):
            raise ValueError("SyncTest mismatch")
        done = re.findall(r"\bTBC DONE frames=(\d+)", log)
        if not done or int(done[-1]) < frames:
            raise ValueError("CPU run did not finish the requested frames")
        summaries = re.findall(r"curated hash: (\d+) compared, (\d+) mismatching", log)
        if not summaries or int(summaries[-1][0]) == 0:
            raise ValueError("no curated-hash comparisons")
        if any(int(mismatches) for _, mismatches in summaries):
            raise ValueError("curated-hash mismatch")
        markers = ["read into a throw-bank arena"]
        if banks < 3:  # All three thrower families fit without eviction in 3+ arenas.
            markers.append("evicted kind")
        for marker in markers:
            if marker not in log:
                raise ValueError(f"missing throw-bank activity: {marker}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("rewind", "cpu"))
    parser.add_argument("log", type=Path)
    parser.add_argument("--frames", type=int, default=6000)
    parser.add_argument("--banks", type=int, default=1)
    args = parser.parse_args()
    try:
        check(args.log.read_text(encoding="utf-8", errors="replace"), args.mode, args.frames, args.banks)
    except (OSError, ValueError) as error:
        parser.exit(1, f"throw banks: FAIL: {error}\n")
    print(f"throw banks: {args.mode} PASS")


if __name__ == "__main__":
    main()
