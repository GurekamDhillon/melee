#!/usr/bin/env python3
"""Fetch a small per-character .slp corpus from the Lunar Melee public API (https://lunarmelee.com/developers).

Polite by design, following the API's fair-use guide: one search per character (spaced well under the 30/minute
replay-search limit), then ONE bulk-download job for all the chosen replay ids (no loop of single downloads), polled
every 20 s, back-off on 429/503. Data from Lunar Melee (lunarmelee.com).

usage: lunar_fetch.py OUT_DIR [--per N] [--source tournament] [--chars 24,4,5,...]
  OUT_DIR must be git-ignored (replays carry player tags; never commit them). Writes OUT_DIR/<char>/*.slp(z),
  OUT_DIR/lunar-manifest.json and OUT_DIR/picks.json (replay id -> character slug; no player names).
  .slpz files are decompressed with `slpz -d` if it is on PATH (cargo install slpz), else left as is.
Env: LUNAR_CLIENT_ID (a UUID you keep; generated into OUT_DIR/.client-id if unset).
"""
import json, os, sys, time, uuid, zipfile, shutil, subprocess, urllib.request, urllib.error

API = "https://api.lunarmelee.com"
UA = "melee-slpfuzz/0.1 (+https://github.com/GurekamDhillon/melee)"
# Slippi external character ids (GET /api/reference/characters)
DEFAULT_CHARS = {24: "pichu", 4: "kirby", 5: "bowser", 10: "mewtwo", 11: "ness", 18: "zelda", 3: "gnw",
                 23: "roy", 21: "younglink", 6: "link", 17: "yoshi", 1: "dk"}


def req(method, path, body=None, cid=None):
    for attempt in range(8):
        r = urllib.request.Request(API + path, method=method, data=None if body is None else json.dumps(body).encode())
        r.add_header("User-Agent", UA)
        r.add_header("Accept-Encoding", "identity")
        if body is not None:
            r.add_header("Content-Type", "application/json")
        if cid:
            r.add_header("X-Client-Id", cid)
        try:
            with urllib.request.urlopen(r, timeout=60) as f:
                return json.load(f)
        except urllib.error.HTTPError as e:
            if e.code in (429, 503):
                wait = int(e.headers.get("Retry-After") or e.headers.get("RateLimit-Reset") or 60)
                print(f"  {e.code}, waiting {wait}s", flush=True)
                time.sleep(wait + 1)
                continue
            raise SystemExit(f"{method} {path}: HTTP {e.code} {e.read()[:300]!r}")
    raise SystemExit(f"{method} {path}: gave up after retries")


def main():
    a = sys.argv[1:]
    if not a:
        print(__doc__); return 2
    out = a[0]
    opt = lambda k, d: a[a.index(k) + 1] if k in a else d
    per = int(opt("--per", "30")); source = opt("--source", "tournament")
    chars = DEFAULT_CHARS
    if "--chars" in a:
        chars = {int(c): f"c{c}" for c in opt("--chars", "").split(",")}
    os.makedirs(out, exist_ok=True)
    cidf = os.path.join(out, ".client-id")
    cid = os.environ.get("LUNAR_CLIENT_ID") or (open(cidf).read().strip() if os.path.exists(cidf) else None)
    if not cid:
        cid = str(uuid.uuid4()); open(cidf, "w").write(cid)

    picks = {}
    for ch, slug in chars.items():
        q = f"/api/replays?p1CharacterId={ch}&source={source}&limit={per}"
        res = req("GET", q)
        ids = [r["_id"] for r in res.get("replays", []) if r.get("_id") not in picks]
        for i in ids:
            picks[i] = slug
        print(f"{slug}: {len(ids)} picked", flush=True)
        time.sleep(4)  # ~15 searches/minute, half the documented limit
    json.dump(picks, open(os.path.join(out, "picks.json"), "w"), indent=1)

    body = {"replayIds": list(picks)}
    est = req("POST", "/api/replays/estimate", body)
    print(f"estimate: {est.get('replayCount')} games, ~{est.get('estimatedZipSize', 0) // 1_000_000} MB", flush=True)
    job = req("POST", "/api/jobs", body, cid)
    jid = job["jobId"]
    print(f"job {jid} {job.get('status')} reused={job.get('reused')}", flush=True)
    while True:
        st = req("GET", f"/api/jobs/{jid}", cid=cid)
        if st.get("downloadReady"):
            break
        if st.get("status") in ("failed", "cancelled", "expired"):
            raise SystemExit(f"job {st.get('status')}: {st}")
        print(f"  {st.get('status')} {st.get('progress', '')}", flush=True)
        time.sleep(20)
    url = req("GET", f"/api/jobs/{jid}/download", cid=cid)["url"]
    zp = os.path.join(out, "bundle.zip")
    r = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(r, timeout=600) as f, open(zp, "wb") as w:
        shutil.copyfileobj(f, w)
    with zipfile.ZipFile(zp) as z:
        man = json.loads(z.read("lunar-manifest.json")) if "lunar-manifest.json" in z.namelist() else None
        if man is not None:
            open(os.path.join(out, "lunar-manifest.json"), "w").write(json.dumps(man))
        byname = {}
        entries = man.get("replays", []) if isinstance(man, dict) else (man or [])
        for e in entries if isinstance(entries, list) else []:
            if isinstance(e, dict) and "replayId" in e:
                byname[os.path.basename(e.get("file") or e.get("path") or e.get("name") or "")] = e["replayId"]
        for n in z.namelist():
            if not n.endswith((".slp", ".slpz")):
                continue
            base = os.path.basename(n)
            slug = picks.get(byname.get(base, ""), "unsorted")
            d = os.path.join(out, slug); os.makedirs(d, exist_ok=True)
            # name by replay id when known (the original names carry nothing useful and may hold tags)
            rid = byname.get(base)
            dst = os.path.join(d, (rid + os.path.splitext(base)[1]) if rid else base)
            with z.open(n) as s, open(dst, "wb") as w:
                shutil.copyfileobj(s, w)
    os.remove(zp)
    if shutil.which("slpz"):
        subprocess.run(["slpz", "-d", "-r", "--rm", out], check=False)
    print("done", flush=True)


if __name__ == "__main__":
    sys.exit(main())
