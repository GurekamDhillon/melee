import json, sys, csv, statistics as st
import numpy as np
def pct(a,p): return float(np.percentile(a,p))
def proc(d):
    j=json.load(open(d+'/proc.json')); s=j['samples']
    last=s[-1]
    return dict(wall=j['wall_s'], peak_ws=max(x['peak_wset'] for x in s)/2**20, end_ws=last['wset']/2**20,
                peak_priv=max(x['private'] for x in s)/2**20, end_priv=last['private']/2**20,
                cpu_s=last['user']+last['system'], t0=j['t0_unix_ms'], samples=s)
def gpu(d, t_from, t_to):
    import datetime
    v=[]
    for l in open(d+'/gpu.csv'):
        try:
            ts,u,m=[x.strip() for x in l.split(',')]
            t=datetime.datetime.strptime(ts,'%Y/%m/%d %H:%M:%S.%f').timestamp()*1000
            if t_from<=t<=t_to: v.append(float(u))
        except Exception: pass
    return sum(v)/len(v) if v else float('nan')
def cpu_window(s, t0, a, b):
    # process CPU seconds between wall offsets a..b (ms since launch)
    pts=[(x['t']*1000, x['user']+x['system']) for x in s]
    def at(t): return min(pts, key=lambda p: abs(p[0]-t))
    (ta,ca),(tb,cb)=at(a),at(b); return ca, cb, ta, tb
def rust(d):
    rows=[l.rstrip().split(',') for l in open(d+'/ft.csv')]
    S=float(rows[0][2]); T=[r for r in rows if r[0]=='T']; P=[r for r in rows if r[0]=='P']
    log=open(d+'/stdout.log',errors='replace').read()
    import re; m_=re.search(r'tick (\d+): loading -> match',log); mt=int(m_.group(1)) if m_ else 3  # newer logs print "scene vs" instead; match starts at tick 3
    T=T[mt:]  # match ticks (tick index order)
    tm0=float(T[0][2]); tm1=float(T[-1][2])+float(T[-1][3])
    pr=[float(r[2]) for r in P if tm0<=float(r[2])<=tm1]
    iv=np.diff(pr); sim=np.array([float(r[3]) for r in T])
    p=proc(d); launch_to_t0=S-p['t0']
    ca,cb,ta,tb=cpu_window(p['samples'],0,launch_to_t0+tm0,launch_to_t0+tm1)
    return dict(ticks=len(T), frames=len(pr), iv=iv, sim=sim, startup=(launch_to_t0+tm0)/1000,
                cpu_per_frame=(cb-ca)*1000/max(1,len(pr)*(tb-ta)/(tm1-tm0)), cpu_per_tick=(cb-ca)*1000/(len(T)*(tb-ta)/(tm1-tm0)),
                cpu_util=(cb-ca)/((tb-ta)/1000), gpu=gpu(d,S+tm0,S+tm1), **{k:v for k,v in p.items() if k!='samples'})
def unl(d):
    sim=[r for r in csv.DictReader(open(d+'/sim.csv')) if int(r['match_frame'])>0]
    m0=int(sim[0]['retrace'])
    allf=list(csv.DictReader(open(d+'/frames.csv'))); ms=int(allf[-1]['simulation'])-len(sim)+1
    fr=[r for r in allf if int(r['simulation'])>=ms]
    iv=np.array([float(r['interval_ms']) for r in fr][1:])
    p=proc(d)
    # wall of match: sum of intervals; match start offset ~ total wall - match wall
    mw=iv.sum()
    ca,cb,ta,tb=cpu_window(p['samples'],0,p['wall']*1000-mw-300,p['wall']*1000-300)
    startup=p['wall']-mw/1000
    return dict(ticks=len(sim), frames=len(iv)+1, iv=iv, sim=np.array([float(r['sim_ms']) for r in sim]),
                draws=np.median([int(r['draws']) for r in fr]), submit=np.median([float(r['submit_ms']) for r in fr]), simcpu=np.array([float(r['cpu_ms']) for r in sim]), startup=startup, match_retrace=m0,
                cpu_per_frame=(cb-ca)*1000/(len(iv)*(tb-ta)/mw), cpu_per_tick=(cb-ca)*1000/(len(sim)*(tb-ta)/mw),
                cpu_util=(cb-ca)/((tb-ta)/1000), gpu=gpu(d,p['t0']+p['wall']*1000-mw,p['t0']+p['wall']*1000-300), **{k:v for k,v in p.items() if k!='samples'})
def summ(r):
    iv=r['iv']; fps=1000/iv.mean(); low=1000/np.mean(np.sort(iv)[-max(1,len(iv)//100):])
    o=dict(fps=fps, low1=low, p50=pct(iv,50), p95=pct(iv,95), p99=pct(iv,99), sim_p50=pct(r['sim'],50), sim_mean=r['sim'].mean(), sim_p99=pct(r['sim'],99),
           cpu_frame=r['cpu_per_frame'], cpu_tick=r['cpu_per_tick'], cpu_util=r['cpu_util'], gpu=r['gpu'], peak_ws=r['peak_ws'], end_ws=r['end_ws'],
           peak_priv=r['peak_priv'], end_priv=r['end_priv'], startup=r['startup'], ticks=r['ticks'], frames=r['frames'])
    if 'draws' in r: o['draws_med']=r['draws']; o['submit_med']=r['submit']
    if 'simcpu' in r: o['simcpu_p50']=pct(r['simcpu'],50); o['simcpu_mean']=r['simcpu'].mean()
    return o
kind=sys.argv[1]; dirs=sys.argv[2:]
rs=[summ(rust(d) if kind=='rust' else unl(d)) for d in dirs]
for r in rs: print({k:round(v,2) for k,v in r.items()})
print('MEDIAN', json.dumps({k:round(st.median([r[k] for r in rs]),2) for k in rs[0]}))
