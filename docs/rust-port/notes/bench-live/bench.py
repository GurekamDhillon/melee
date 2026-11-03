"""Live-play benchmark runner: one process, sampled for memory/CPU, GPU util via nvidia-smi."""
import argparse, json, os, subprocess, time, psutil, sys
from pathlib import Path
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out', required=True)
    ap.add_argument('--env', action='append', default=[])
    ap.add_argument('--cwd')
    ap.add_argument('cmd', nargs=argparse.REMAINDER)
    a = ap.parse_args()
    if a.cmd and a.cmd[0] == '--': a.cmd = a.cmd[1:]
    out = Path(a.out); out.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    for kv in a.env:
        k, v = kv.split('=', 1); env[k] = v
    gpu = subprocess.Popen(['nvidia-smi', '--query-gpu=timestamp,utilization.gpu,memory.used', '--format=csv,noheader,nounits', '-lms', '100'],
                           stdout=open(out/'gpu.csv', 'w'), stderr=subprocess.DEVNULL, creationflags=0x08000000)
    t0 = time.time()
    with open(out/'stdout.log', 'w') as log:
        p = psutil.Popen(a.cmd, cwd=a.cwd, env=env, stdout=log, stderr=subprocess.STDOUT, creationflags=0x08000000)
        samples = []
        while p.poll() is None:
            try:
                m = p.memory_info(); c = p.cpu_times()
                samples.append(dict(t=time.time()-t0, wset=m.wset, peak_wset=m.peak_wset, private=m.private,
                                    user=c.user, system=c.system, threads=p.num_threads()))
            except psutil.Error:
                break
            time.sleep(0.1)
    p.wait()
    wall = time.time() - t0
    gpu.kill(); gpu.wait()
    res = dict(cmd=a.cmd, t0_unix_ms=t0*1000, wall_s=wall, exit=p.returncode, samples=samples)
    (out/'proc.json').write_text(json.dumps(res))
    print(json.dumps(dict(wall_s=round(wall,2), exit=p.returncode, nsamples=len(samples))))
main()
