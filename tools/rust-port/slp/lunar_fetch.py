#!/usr/bin/env python3
"""Fetch a per-character .slp corpus from the Lunar Melee public API (https://lunarmelee.com/developers).

Polite by design, following the API's fair-use guide: searches spaced well under the 30/minute limit, then ONE
bulk-download job per batch (never a loop of single downloads; the API allows 5 job creations an hour and 3 running
jobs), job status polled every 20 s, back-off on 429/503 (Retry-After) and 5xx. Data from Lunar Melee (lunarmelee.com).

usage:
  lunar_fetch.py search OUT_DIR [--per N] [--source tournament] [--chars all|24,4,...]
      page-based searches per character (the character in port filter p1 then p2, newest first) up to --per games,
      deduplicated across characters (a game belongs to the first character that picked it); resumable
      -> OUT_DIR/picks.json (replay id -> character slug; no player names are stored).
  lunar_fetch.py fetch OUT_DIR BATCH_DIR --chars slug,slug,... [--slpz PATH]
      ONE bulk job for the picked ids of those characters -> BATCH_DIR/<slug>/<replay id>.slp and
      BATCH_DIR/lunar-manifest.json (file name -> replay id only). .slpz files are decompressed with `slpz -d`
      (--slpz PATH, or `slpz` on PATH), else left as is.
  lunar_fetch.py OUT_DIR [--per N] ...    (one-shot: search, then one job for everything)
OUT_DIR / BATCH_DIR must be git-ignored (replays carry player tags; never commit them).
Env: LUNAR_CLIENT_ID (a UUID you keep; generated into OUT_DIR/.client-id if unset).
"""
import json, os, sys, time, uuid, zipfile, shutil, subprocess, urllib.request, urllib.error

API = "https://api.lunarmelee.com"
UA = "melee-slpfuzz/0.2 (+https://github.com/GurekamDhillon/melee)"
# Slippi external character ids (GET /api/reference/characters)
ALL_CHARS = {0: "falcon", 1: "dk", 2: "fox", 3: "gnw", 4: "kirby", 5: "bowser", 6: "link", 7: "luigi", 8: "mario",
             9: "marth", 10: "mewtwo", 11: "ness", 12: "peach", 13: "pikachu", 14: "ics", 15: "puff", 16: "samus",
             17: "yoshi", 18: "zelda", 19: "sheik", 20: "falco", 21: "younglink", 22: "doc", 23: "roy", 24: "pichu",
             25: "ganon"}
PILOT_CHARS = [24, 4, 5, 10, 11, 18, 3, 23, 21, 6, 17, 1]


def req(method, path, body=None, cid=None):
    for attempt in range(10):
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
            if e.code >= 500:
                wait = min(300, 15 * 2 ** attempt)
                print(f"  {e.code}, retry in {wait}s", flush=True)
                time.sleep(wait)
                continue
            raise SystemExit(f"{method} {path}: HTTP {e.code} {e.read()[:300]!r}")
        except (urllib.error.URLError, TimeoutError) as e:
            wait = min(300, 15 * 2 ** attempt)
            print(f"  network error {e}, retry in {wait}s", flush=True)
            time.sleep(wait)
    raise SystemExit(f"{method} {path}: gave up after retries")


def client_id(out):
    cidf = os.path.join(out, ".client-id")
    cid = os.environ.get("LUNAR_CLIENT_ID") or (open(cidf).read().strip() if os.path.exists(cidf) else None)
    if not cid:
        cid = str(uuid.uuid4()); open(cidf, "w").write(cid)
    return cid


def search(out, per, source, chars):
    os.makedirs(out, exist_ok=True)
    pf = os.path.join(out, "picks.json")
    picks = json.load(open(pf)) if os.path.exists(pf) else {}
    have = {}
    for s in picks.values():
        have[s] = have.get(s, 0) + 1
    for ch, slug in chars.items():
        got = have.get(slug, 0)
        if got >= per:
            print(f"{slug}: {got} already picked", flush=True); continue
        for key in ("p1CharacterId", "p2CharacterId"):
            page = 1
            while got < per:
                lim = min(1000, max(50, (per - got) * 2))
                res = req("GET", f"/api/replays?{key}={ch}&source={source}&limit={lim}&page={page}")
                rows = res.get("replays", [])
                for r in rows:
                    i = r.get("_id")
                    if i and i not in picks and got < per:
                        picks[i] = slug; got += 1
                time.sleep(4)  # ~15 searches/minute, half the documented limit
                pg = res.get("pagination") or {}
                if not rows or page >= pg.get("pages", page):
                    break
                page += 1
            if got >= per:
                break
        print(f"{slug}: {got} picked", flush=True)
        json.dump(picks, open(pf, "w"), indent=1)
    return picks


def fetch(out, batch, slugs, slpz=None):
    cid = client_id(out)
    picks = json.load(open(os.path.join(out, "picks.json")))
    sel = {i: s for i, s in picks.items() if s in slugs}
    if not sel:
        raise SystemExit("no picked ids for " + ",".join(slugs))
    os.makedirs(batch, exist_ok=True)
    body = {"replayIds": list(sel)}
    est = req("POST", "/api/replays/estimate", body)
    print(f"estimate: {est.get('replayCount')} games, ~{(est.get('estimatedZipSize') or 0) // 1_000_000} MB", flush=True)
    job = req("POST", "/api/jobs", body, cid)
    jid = job["jobId"]
    open(os.path.join(batch, "job-id"), "w").write(jid)
    print(f"job {jid} {job.get('status')} reused={job.get('reused')} lane={job.get('lane')}", flush=True)
    while True:
        st = req("GET", f"/api/jobs/{jid}", cid=cid)
        if st.get("downloadReady"):
            break
        if st.get("status") in ("failed", "cancelled", "expired"):
            raise SystemExit(f"job {st.get('status')}: {st.get('error')}")
        p = st.get("progress") or {}
        print(f"  {st.get('status')} {p.get('filesProcessed', '')}/{p.get('filesTotal', '')} q={st.get('queuePosition')}", flush=True)
        time.sleep(20)
    zp = os.path.join(batch, "bundle.zip")
    for attempt in range(5):
        try:
            url = req("GET", f"/api/jobs/{jid}/download", cid=cid)["url"]
            r = urllib.request.Request(url, headers={"User-Agent": UA})
            with urllib.request.urlopen(r, timeout=900) as f, open(zp, "wb") as w:
                shutil.copyfileobj(f, w, 1 << 20)
            zipfile.ZipFile(zp).close()
            break
        except (urllib.error.URLError, TimeoutError, ConnectionError, zipfile.BadZipFile) as e:
            print(f"  download error {e}, retry", flush=True); time.sleep(30 * (attempt + 1))
    else:
        raise SystemExit("download failed")
    n = 0
    with zipfile.ZipFile(zp) as z:
        man = json.loads(z.read("lunar-manifest.json")) if "lunar-manifest.json" in z.namelist() else None
        byname = {}
        entries = (man.get("replays") or man.get("files") or []) if isinstance(man, dict) else (man or [])
        for e in entries if isinstance(entries, list) else []:
            if isinstance(e, dict) and "replayId" in e:
                nm = e.get("file") or e.get("path") or e.get("name") or e.get("fileName") or ""
                byname[os.path.basename(nm)] = e["replayId"]
        if isinstance(man, dict) and not byname:
            for k, v in man.items():  # a {file name: {replayId, fileHash}} map
                if isinstance(v, dict) and "replayId" in v:
                    byname[os.path.basename(k)] = v["replayId"]
        # keep only names and ids (the original manifest may carry player names)
        json.dump(byname, open(os.path.join(batch, "lunar-manifest.json"), "w"))
        for name in z.namelist():
            if not name.endswith((".slp", ".slpz")):
                continue
            base = os.path.basename(name)
            rid = byname.get(base)
            slug = sel.get(rid or "", "unsorted")
            d = os.path.join(batch, slug); os.makedirs(d, exist_ok=True)
            dst = os.path.join(d, (rid + os.path.splitext(base)[1]) if rid else base)
            with z.open(name) as s, open(dst, "wb") as w:
                shutil.copyfileobj(s, w)
            n += 1
    os.remove(zp)
    tool = slpz or shutil.which("slpz")
    if tool:
        subprocess.run([tool, "-d", "-r", "--rm", batch], check=False)
    print(f"done: {n} files", flush=True)
    return 0


def main():
    a = sys.argv[1:]
    if not a:
        print(__doc__); return 2
    opt = lambda k, d: a[a.index(k) + 1] if k in a else d
    per = int(opt("--per", "30")); source = opt("--source", "tournament")
    cs = opt("--chars", "")
    if a[0] == "fetch":
        return fetch(a[1], a[2], cs.split(","), opt("--slpz", None))
    if cs == "all":
        chars = ALL_CHARS
    elif cs:
        chars = {int(c): ALL_CHARS.get(int(c), f"c{c}") for c in cs.split(",")}
    else:
        chars = {c: ALL_CHARS[c] for c in PILOT_CHARS}
    if a[0] == "search":
        search(a[1], per, source, chars); return 0
    out = a[0]
    search(out, per, source, chars)
    return fetch(out, out, list(chars.values()), opt("--slpz", None))


if __name__ == "__main__":
    sys.exit(main())
