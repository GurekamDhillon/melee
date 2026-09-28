#!/usr/bin/env python3
"""Serial ACE crash sweep. --plan never launches the game.

Copy this directory to tools/sweep/sweep-all in the workspace, or use --harness
with a copied runner. No dependency on the old sweep modules or their defaults.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time

# Legal retail VS stages, excluding unfinished Akaneia (21) and Icetop (26).
# stage.c::stage_id_map / gr/forward.h; all added m-ex externals begin at 288
# (the same boundary used by workspace tools/sweep/crash_sweep.py).
RETAIL = {2: 12, 3: 16, 4: 2, 5: 4, 6: 8, 7: 14, 8: 10, 9: 20,
          10: 18, 11: 3, 12: 5, 13: 6, 14: 7, 15: 9, 16: 11, 17: 13,
          18: 21, 19: 24, 20: 25, 22: 15, 23: 17, 24: 19, 25: 22,
          27: 27, 28: 28, 29: 29, 30: 30, 31: 36, 32: 37}


def vs(tag, external, kind=1, internal=None, group='stage'):
    return dict(tag=tag, group=group, mode=2, fighter_kind=kind, stage=internal,
                scene=f'mode=vs;p1=fk:{kind};p2=fox/cpu0;stage=ext:{external};time=0;items=off')


def subset():
    return [vs(f'subset-ext{ext}-fk{kind}', ext, kind, RETAIL[ext], 'subset')
            for ext in (2, 31, 32) for kind in (1, 22)]


def catalog(log):
    fk = re.search(r'(\d+) m-ex fighter kinds from MxDt\.dat \(kinds (\d+)\.\.(\d+)\)', log)
    st = re.search(r'stage tables ready: (\d+) internal, (\d+) external', log)
    if not fk or not st:
        raise ValueError('ACE discovery missing fighter/stage counts; refusing a partial full sweep')
    count, first, last = map(int, fk.groups())
    internal, external = map(int, st.groups())
    if count != last - first + 1 or not (0 < count <= 94 and 33 <= first <= last < 127
                                       and 288 < external <= 512 and 0 < internal <= 256):
        raise ValueError('unsupported or inconsistent ACE catalog; review port table limits')
    return dict(first_kind=first, last_kind=last, external_count=external,
                internal_count=internal)


def plan(cat):
    cases = [vs(f'vs-ext{e}', e, internal=k) for e, k in RETAIL.items()]
    cases += [vs(f'vs-ext{e}', e) for e in range(288, cat['external_count'])]
    # gw_SceneLaunch_OnePStep: launchable steps, not Adventure substages.
    for mode, mode_id, steps in (('classic', 3, 11), ('adventure', 4, 12)):
        for step in range(steps):
            cases.append(dict(tag=f'{mode}-{step:02}', group=mode, mode=mode_id,
                              fighter_kind=1, stage=None, step=step,
                              scene=f'mode={mode};p1=fk:1;step={step};difficulty=2'))
    cases += [vs(f'fighter-fk{k}', 31, k, 36, 'fighter')
              for k in range(cat['first_kind'], cat['last_kind'] + 1)]
    return cases


def judge(log, crash, code, timed_out, sections):
    combined = log + '\n' + crash
    fatal = re.search(r'[^\n]*(?:FATAL|PANIC|ALLOC_FAIL)[^\n]*', combined)
    # Only the fault line is an instruction address. Never report a return
    # address from the later frame-pointer walk as the crash RVA.
    fault = re.search(r'FATAL[^\n]*melee-pc\.map rva (0x[0-9a-fA-F]+)', combined)
    rva = f'0x{int(fault[1], 16):08x}' if fault else None
    checks = re.findall(r'TEST sweep-all section (\d+): (PASS|FAIL) ([^\r\n]*)', log)
    failed = next((detail for _, state, detail in checks if state == 'FAIL'), None)
    if fatal or crash:
        result, detail = 'CRASH', fatal[0].strip() if fatal else 'crash artifact written'
    elif timed_out:
        result, detail = 'TIMEOUT', 'process exceeded wall-clock deadline'
    elif code != 0:
        result, detail = 'EXIT', f'exit code {code}'
    elif failed is not None:
        result, detail = 'CHECK', failed
    elif [int(n) for n, state, _ in checks if state == 'PASS'] != list(range(1, sections + 1)):
        result, detail = 'INCOMPLETE', f'expected {sections} unique ordered PASS sections'
    elif re.search(r'scene:.*(?:errors=[1-9]|clamped|not on this install)|\bon_(?:tick|frame):.*error', log):
        result, detail = 'CONFIG', 'scene parser or Lua error; inspect log'
    else:
        result, detail = 'PASS', f'{sections} checks passed'
    return dict(result=result, detail=detail, rva=rva,
                tail='\n'.join(log.splitlines()[-40:]),
                crash_tail='\n'.join(crash.splitlines()[-40:]))


def is_known(row, issues):
    def signature(item):
        # The exception instruction's absolute address moves under ASLR. Keep
        # its module RVA and exception type/code; discard only that address.
        detail = re.sub(r'\bat (?:0x)?[0-9a-fA-F]+\s+', 'at <address> ', item.get('detail', ''))
        return item.get('tag'), item.get('result'), item.get('rva'), detail
    return any(signature(row) == signature(issue)
               for issue in issues)


def read(path):
    return path.read_text(encoding='utf-8', errors='replace') if path.is_file() else ''


def run_case(case, args, root):
    sb = root / 'runs' / case['tag']
    sb.mkdir(parents=True)  # fresh root: stale logs can never satisfy a check
    shutil.copy2(args.exe, sb / 'melee-pc.exe')
    for name in ('melee-pc.map', 'SDL3.dll', 'webgpu_dawn.dll',
                 'initial_pipeline_cache.db', 'initial_pipeline_cache.core'):
        src = args.exe.parent / name
        if not src.is_file():
            src = args.runtime / name
        if src.is_file():
            shutil.copy2(src, sb / name)
    env = {k: v for k, v in os.environ.items() if not k.startswith('MELEE_')}
    env.update(MELEE_SCENE=case['scene'], MELEE_SCRIPTS='0', MELEE_INPUT='none',
               MELEE_PAD_SCRIPT=str(args.harness / 'sweep_all.lua'),
               MELEE_MODS_DIR=str(root / 'nomods'), MELEE_VOLUME='0',
               MELEE_PAD_IGNORE_ADAPTER='1', SDL_JOYSTICK_HIDAPI_GAMECUBE='0',
               MELEE_TURBO='1', MELEE_TURBO_RENDER='8', MELEE_TURBO_DRAWS='1',
               MELEE_RUN_LABEL='sweep-all / ' + case['tag'],
               MELEE_LAB_SWEEP_SCENE=case['scene'], MELEE_LAB_SWEEP_MODE=str(case['mode']),
               MELEE_LAB_SWEEP_KIND=str(case['fighter_kind']),
               MELEE_LAB_SWEEP_FRAMES=str(args.frames))
    if case['stage'] is not None:
        env['MELEE_LAB_SWEEP_STAGE'] = str(case['stage'])
    start = time.monotonic()
    timed_out = False
    with (sb / 'stdio.log').open('w', encoding='utf-8') as stream:
        proc = subprocess.Popen([str(sb / 'melee-pc.exe'), '--iso', str(args.iso)],
                                cwd=sb, env=env, stdout=stream, stderr=stream)
        try:
            proc.wait(timeout=args.timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            proc.kill()
            proc.wait()
        finally:
            # Also reap only our own process on Ctrl-C or an unexpected host error.
            if proc.poll() is None:
                proc.kill()
                proc.wait()
    log = read(sb / 'melee-pc.log')
    artifacts = sorted((sb / 'crashlogs').glob('*'))
    crash = '\n'.join(f'{p.name}\n{read(p)}' for p in artifacts if p.is_file())
    row = dict(case, **judge(log, crash, proc.returncode, timed_out, 3),
               seconds=round(time.monotonic() - start, 2), exit_code=proc.returncode,
               sandbox=str(sb))
    if row['result'] == 'PASS' and 'step' in case:
        pattern = rf'1P direct launch: mode {case["mode"]} ckind \d+ color \d+ step {case["step"]} difficulty 2'
        if not re.search(pattern, log):
            row.update(result='CONFIG', detail='missing requested 1P direct-launch milestone')
    return row, log


def report(root, rows, issues, metadata):
    new = [r for r in rows if r['result'] != 'PASS' and not is_known(r, issues)]
    passed = sum(r['result'] == 'PASS' for r in rows)
    known = len(rows) - passed - len(new)
    (root / 'results.json').write_text(json.dumps(dict(metadata=metadata, results=rows), indent=2), encoding='utf-8')
    lines = ['# sweep-all', '', f'PASS: {passed}/{len(rows)}; known failures: {known}; new failures: {len(new)}.',
             '', 'All case details and logs are retained in results.json and runs/.', '']
    for row in new:
        lines += [f'## {row["tag"]}: {row["result"]}', '', row['detail'], '',
                  f'Crash RVA: {row["rva"] or "unavailable (no native fault address recorded)"}',
                  f'Scene: `{row["scene"]}`', '', 'Log tail:', '```text', row['tail'], '```', '']
        if row.get('crash_tail'):
            lines += ['Crash artifact tail:', '```text', row['crash_tail'], '```', '']
    (root / 'summary.md').write_text('\n'.join(lines), encoding='utf-8')
    return bool(new)


def execute(case, args, root):
    try:
        return run_case(case, args, root)
    except OSError as exc:
        return dict(case, result='HARNESS', detail=str(exc), rva=None, tail=''), ''


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--exe', type=Path)
    ap.add_argument('--iso', type=Path, help='ACE disc path (required for game runs)')
    ap.add_argument('--runtime', type=Path, help='DLL/cache source, normally workspace _build')
    ap.add_argument('--out', type=Path, default=Path('sweep-all-results'))
    ap.add_argument('--harness', type=Path, default=Path(__file__).resolve().parent)
    ap.add_argument('--known-issues', type=Path)
    ap.add_argument('--subset', action='store_true', help='3 stages x 2 fighters, no discovery needed')
    ap.add_argument('--plan', action='store_true', help='print cases; never launches game')
    ap.add_argument('--catalog-log', type=Path, help='saved ACE boot log for full --plan only')
    ap.add_argument('--frames', type=int, default=180)
    ap.add_argument('--timeout', type=float, default=75)
    args = ap.parse_args()
    if not 1 <= args.frames <= 1800 or not 1 <= args.timeout <= 3600:
        ap.error('frames must be 1..1800; timeout must be 1..3600 seconds')
    args.harness = args.harness.resolve()
    if args.plan:
        if not args.subset and not args.catalog_log:
            ap.error('full --plan requires --catalog-log; no invented ACE counts')
        cases = subset() if args.subset else plan(catalog(read(args.catalog_log)))
        print(json.dumps(cases, indent=2))
        return 0
    if args.catalog_log:
        ap.error('--catalog-log is only for --plan; runs must discover the current disc')
    if not args.exe or not args.iso or not args.exe.is_file() or not args.iso.is_file():
        ap.error('--exe and --iso must name existing files')
    if os.name != 'nt':
        ap.error('game execution requires Windows')
    args.exe, args.iso = args.exe.resolve(), args.iso.resolve()
    args.runtime = (args.runtime or args.exe.parent).resolve()
    if not (args.harness / 'sweep_all.lua').is_file():
        ap.error('--harness must contain sweep_all.lua')
    issues_path = args.known_issues or args.harness / 'known-issues.json'
    issues = json.loads(issues_path.read_text(encoding='utf-8'))['issues']
    args.out.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix=time.strftime('%Y%m%d-%H%M%S-'), dir=args.out.resolve()))
    (root / 'nomods').mkdir()
    metadata = dict(exe=str(args.exe), exe_sha256=hashlib.sha256(args.exe.read_bytes()).hexdigest(),
                    iso=str(args.iso), frames=args.frames, timeout=args.timeout,
                    known_issues=str(issues_path.resolve()), subset=args.subset)
    rows = []
    if args.subset:
        cases = subset()
    else:
        probe, log = execute(vs('probe', 31, internal=36), args, root)
        rows.append(probe)
        if probe['result'] != 'PASS':
            report(root, rows, issues, metadata)
            print(f'Discovery failed: {root / "summary.md"}')
            return 2
        try:
            cat = catalog(log)
        except ValueError as exc:
            probe.update(result='DISCOVERY', detail=str(exc))
            report(root, rows, issues, metadata)
            print(f'Discovery failed: {root / "summary.md"}')
            return 2
        metadata['catalog'] = cat
        cases = plan(cat)
    (root / 'plan.json').write_text(json.dumps(cases, indent=2), encoding='utf-8')
    for case in cases:
        row, _ = execute(case, args, root)
        rows.append(row)
        report(root, rows, issues, metadata)  # durable progress after every case
        print(f'{len(rows)}/{len(cases) + (not args.subset)} {case["tag"]}: {row["result"]}', flush=True)
    new = report(root, rows, issues, metadata)
    print(root / 'summary.md')
    return 1 if new else 0


if __name__ == '__main__':
    sys.exit(main())
