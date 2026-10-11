#!/usr/bin/env python3
"""np_modsync.py - two-client loopback proofs of "match the host's mods" (gw_modsync.h). Loopback ONLY: the local matchmaking server and (scenario a) the
Nucleus fixture server run on 127.0.0.1, both clients bind 127.0.0.1 (MELEE_NETPLAY_BIND), windows offscreen. Never the public server or the real Nucleus.

  GW_MELEE=<game worktree> GW_BUILD_ROOT=<its build root> python np_modsync.py <a|b|c|d> <tag>

  a  the guest lacks a Nucleus stage skin the host has -> sync (download by post id) -> the match starts with matching identity
  b  the guest has an extra identity-affecting mod -> disabled for the session, restored when the room is left
  c  the host has a mod that is NOT from Nucleus -> the sync says what is missing, no download, the match does not start
  d  the game dies during the sync (MELEE_MODSYNC_CRASH_AT=applied) -> the next boot restores the player's own enabled set

Evidence: <build root>/modsync/<tag>/ (trace.txt, host.log, guest.log (all boots, in order), result.json). Exit 0 = every check passed."""
import json, os, re, shutil, socket, subprocess, sys, time, urllib.request

HERE = os.path.dirname(os.path.abspath(__file__)).replace("\\", "/")
ROOT = os.environ.get("GW_ROOT_ENV", "E:/Projects/Melee Workspace")
BUILD = os.environ["GW_BUILD_ROOT"].replace("\\", "/")
WT = os.environ["GW_MELEE"].replace("\\", "/")
BASH = "C:/Program Files/Git/bin/bash.exe"


def load_env():
    env = dict(os.environ)
    for line in open(ROOT + "/.env", encoding="utf-8", errors="replace"):
        m = re.match(r'^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*"?([^"\n]*)"?\s*$', line)
        if m:
            env[m.group(1)] = m.group(2)
    env["GW_MELEE"], env["GW_BUILD_ROOT"] = WT, BUILD
    return env


def free_port(kind=socket.SOCK_DGRAM, lo=51700, hi=58000):
    import random
    for _ in range(200):
        p = random.randrange(lo, hi)
        t = socket.socket(socket.AF_INET, kind)
        try:
            t.bind(("127.0.0.1", p))
            return p
        except OSError:
            continue
        finally:
            t.close()
    raise SystemExit("no free port")


def kill_run(name):
    """stop every process of a run sandbox (the game may have restarted itself, so it is no longer the launcher's child)"""
    ps = "Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*%s*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" % name
    subprocess.run(["powershell", "-NoProfile", "-Command", ps], capture_output=True)


class Side:
    def __init__(self, drv, role, mods):
        self.d, self.role, self.mods = drv, role, mods
        self.name = "ms-%s-%s" % (drv.tag, role)
        self.sandbox = "%s/runs/%s" % (BUILD, self.name)
        self.proc, self.acc, self.pos, self.sig = None, "", 0, None
        os.makedirs(self.mods, exist_ok=True)

    def launch(self, script, extra=None, room=None):
        d = self.d
        marker = self.sandbox + "/.active-run"
        if os.path.isdir(marker):
            os.rmdir(marker)
        env = dict(d.env0, MELEE_SCENE="mode=menu", MELEE_SKIP_INTRO="1", MELEE_VOLUME="0", MELEE_PAD_IGNORE_ADAPTER="1", MELEE_NETPLAY_SERVER=d.server,
                   MELEE_NETPLAY_BIND="127.0.0.1", MELEE_NETPLAY_PORT=str(d.udp), MELEE_NETPLAY_DELAY="2", MELEE_RB_LOG="0", MELEE_MODS_DIR=self.mods,
                   MELEE_WINDOW_X="30000", MELEE_WINDOW_Y="30000", MELEE_WINDOW_W="640", MELEE_WINDOW_H="360", MELEE_TURBO="1",
                   MELEE_SCRIPT=HERE + "/" + script,
                   MELEE_PAD_BOT=("0,0,%d" % (100 + d.rnd % 50)) if self.role == "host" else ("1,0,%d" % (200 + d.rnd % 50)),
                   MELEE_NUCLEUS_DIR=self.mods + "/.nucleus")
        if d.fixture:
            env["MELEE_NUCLEUS_API"] = "http://127.0.0.1:%d/api/public/v1" % d.fixture_port
        env.update(extra or {})
        if room:
            env["MELEE_LAB_ROOM"] = room
        os.makedirs(self.sandbox, exist_ok=True)
        self.proc = subprocess.Popen([BASH, ROOT + "/tools/port/run.sh", "--realtime", self.name, "--iso", d.iso], cwd=WT, env=env,
                                     stdout=open("%s/%s.out" % (d.out, self.role), "a"), stderr=subprocess.STDOUT)
        d.log("launched", self.role, self.name, "pid", self.proc.pid)

    def text(self):
        """every boot's log, in order: the file is re-read, and a restart (the file shrank or was replaced) starts a new section"""
        p = self.sandbox + "/melee-pc.log"
        try:
            b = open(p, "rb").read().decode("utf-8", "replace")
        except OSError:
            return self.acc
        if len(b) < self.pos or (self.sig is not None and not b.startswith(self.sig)):
            self.acc += "\n=== NEW BOOT ===\n" + b
            self.sig = b[:200]
        elif self.sig is None:
            self.acc += b
            self.sig = b[:200]
        else:
            self.acc += b[self.pos:]
        self.pos = len(b)
        return self.acc

    def has(self, pat):
        return re.search(pat, self.text())

    def kill(self):
        kill_run(self.name)
        if self.proc is not None and self.proc.poll() is None:
            subprocess.run(["taskkill", "/PID", str(self.proc.pid), "/T", "/F"], capture_output=True)
        m = self.sandbox + "/.active-run"
        if os.path.isdir(m):
            try:
                os.rmdir(m)
            except OSError:
                pass


class Driver:
    def __init__(self, scenario, tag):
        self.sc, self.tag = scenario, tag
        self.out = "%s/modsync/%s" % (BUILD, tag)
        os.makedirs(self.out, exist_ok=False)
        self.trace = open(self.out + "/trace.txt", "w", encoding="utf-8")
        self.t0, self.checks = time.time(), []
        self.env0 = load_env()
        self.iso = self.env0["GW_ISO_VANILLA"]
        self.rnd = int(time.time()) % 1000
        self.fixture = scenario == "a"
        self.procs = []
        sport = free_port(socket.SOCK_STREAM, 52000, 53000)
        self.server = "127.0.0.1:%d" % sport
        # the workspace's matchmaking server still speaks protocol 5; this game is protocol 6 (gw_net.h): run a patched COPY (the copy is test evidence)
        src = open(ROOT + "/tools/netplay/server/gdmelee_server.py", encoding="utf-8").read()
        src = re.sub(r"NETPLAY_PROTOCOL = \d+", "NETPLAY_PROTOCOL = 6", src, count=1)
        open(self.out + "/gdmelee_server_p6.py", "w", encoding="utf-8").write(src)
        self.srv = subprocess.Popen([sys.executable, self.out + "/gdmelee_server_p6.py", "--bind", "127.0.0.1", "--port", str(sport)],
                                    stdout=open(self.out + "/server.log", "w"), stderr=subprocess.STDOUT)
        self.udp = free_port()
        self.fixture_port = free_port(socket.SOCK_STREAM, 53000, 54000)
        self.host = Side(self, "host", self.out + "/host/mods")
        self.guest = Side(self, "guest", self.out + "/guest/mods")

    def log(self, *x):
        line = "[%6.1f] %s" % (time.time() - self.t0, " ".join(str(i) for i in x))
        print(line, flush=True)
        self.trace.write(line + "\n")
        self.trace.flush()

    def check(self, what, ok, detail=""):
        self.checks.append({"check": what, "ok": bool(ok), "detail": str(detail)})
        self.log("CHECK %-4s %s %s" % ("PASS" if ok else "FAIL", what, detail))
        return bool(ok)

    def wait(self, what, fn, timeout=240, every=2.0):
        t = time.time()
        while time.time() - t < timeout:
            v = fn()
            if v:
                self.log("ok   %s (%.0fs)" % (what, time.time() - t))
                return v
            time.sleep(every)
        self.log("TIMEOUT %s after %ds" % (what, timeout))
        return None

    def finish(self):
        for s in (self.host, self.guest):
            open("%s/%s.log" % (self.out, s.role), "w", encoding="utf-8").write(s.text())
            s.kill()
        for p in self.procs:
            subprocess.run(["taskkill", "/PID", str(p.pid), "/F"], capture_output=True)
        subprocess.run(["taskkill", "/PID", str(self.srv.pid), "/F"], capture_output=True)
        ok = all(c["ok"] for c in self.checks) and bool(self.checks)
        json.dump({"scenario": self.sc, "ok": ok, "checks": self.checks}, open(self.out + "/result.json", "w"), indent=1)
        self.log("RESULT", "PASS" if ok else "FAIL")
        return 0 if ok else 1


# ---- fixtures ----------------------------------------------------------------------------------------------------------------------

def write_script_mod(mods, mid, name, source=None):
    d = "%s/%s" % (mods, mid)
    os.makedirs(d + "/scripts", exist_ok=True)
    j = {"id": mid, "name": name, "version": "1.0.0", "kind": "script", "gameplay": True, "rollback_safe": False, "api_version": 2, "entry": "scripts/main.lua"}
    if source:
        j["source"] = source
    json.dump(j, open(d + "/mod.json", "w"), indent=1)
    open(d + "/scripts/main.lua", "w").write("-- @name: %s\n-- @version: 1.0.0\n-- @gameplay: true\ngd.log('%s loaded')\n" % (name, mid))


def read_enabled(mods):
    p = mods + "/enabled.txt"
    if not os.path.exists(p):
        return None
    return sorted(l.strip() for l in open(p, encoding="utf-8") if l.strip() and not l.startswith("#"))


def start_fixture(d):
    gd_iso = d.iso
    stage = d.out + "/GrPs.dat"
    subprocess.run([sys.executable, ROOT + "/tools/gc_extract.py", gd_iso, d.out + "/iso", "--only", "GrPs.dat"], capture_output=True)
    src = None
    for dp, _, fn in os.walk(d.out + "/iso"):
        for f in fn:
            if f.lower() == "grps.dat":
                src = os.path.join(dp, f)
    if not src:
        return None
    data = open(src, "rb").read() + b"\0" * 32  # a different file (so the stage's identity differs), never loaded by the test
    open(stage, "wb").write(data)
    p = subprocess.Popen([sys.executable, ROOT + "/tools/nucleus/fixture_server.py", "--port", str(d.fixture_port), "--stage-dat", "GrPs=" + stage],
                         stdout=open(d.out + "/fixture.log", "w"), stderr=subprocess.STDOUT)
    d.procs.append(p)
    for _ in range(40):
        try:
            u = "http://127.0.0.1:%d/api/public/v1/mods?limit=100&type=stage_skin" % d.fixture_port
            js = json.load(urllib.request.urlopen(u, timeout=3))
            for m in js["data"]:
                if any("GrPs" in f["filename"] for f in m["files"]):
                    return m["id"]
            return None
        except Exception:
            time.sleep(0.5)
    return None


def host_code(d):
    def f():
        m = re.search(r"ROOMCODE (\w{4})", d.host.text())
        return m.group(1) if m else None
    return d.wait("host room code", f, 600)


def scenario_a(d):
    post = start_fixture(d)
    d.check("fixture serves a GrPs stage skin post", post, post)
    if not post:
        return
    # the HOST has the skin installed by the real installer (a prep run), the guest has nothing
    d.host.launch("ms_prep.lua", {"MELEE_LAB_POST": str(post)})
    ok = d.wait("host: skin installed", lambda: [x for x in os.listdir(d.host.mods) if x.startswith("nucleus-%d-" % post)], 300)
    d.check("host installed the stage skin through the Nucleus installer", ok, ok)
    d.host.kill()
    time.sleep(3)
    d.host.proc, d.host.acc, d.host.pos, d.host.sig = None, "", 0, None
    shutil.rmtree(d.host.sandbox, ignore_errors=True)
    d.host.launch("ms_host.lua")
    code = host_code(d)
    d.check("host room code", code, code)
    d.guest.launch("ms_guest.lua", {"MELEE_LAB_SYNC": "1"}, room=code)
    room = d.wait("guest: room line", lambda: d.guest.has(r"MS_ROOM "), 600)
    line = re.search(r"MS_ROOM [^\n]*", d.guest.text())
    d.log(line.group(0) if line else "no MS_ROOM")
    d.check("before the sync the guest sees the skin as a difference", line and "nucleus-%d-" % post in line.group(0) and "theirs_only" in line.group(0), line.group(0) if line else "")
    d.check("a DOWNLOAD action for the post was planned", d.guest.has(r"MS_ACTION download .*post=%d" % post))
    d.wait("guest: restarted with the synced set", lambda: d.guest.has(r"=== NEW BOOT ===") and d.guest.has(r"a synced mod set is active"), 600)
    after = d.wait("guest: back in the room", lambda: d.guest.has(r"MS_ROOM active=true"), 600)
    line2 = re.search(r"MS_ROOM active=true[^\n]*", d.guest.text())
    d.log(line2.group(0) if line2 else "no second MS_ROOM")
    d.check("after the sync no identity-affecting difference remains", line2 and "theirs_only" not in line2.group(0) and "mine_only" not in line2.group(0) and "differs" not in line2.group(0), line2.group(0) if line2 else "")
    d.check("the stage skin installed from the fixture (Nucleus by post id)", [x for x in os.listdir(d.guest.mods) if x.startswith("nucleus-%d-" % post)])
    d.check("the match started on both sides", d.wait("match", lambda: d.guest.has("MS_MATCH_STARTED") and d.host.has("MS_MATCH_STARTED"), 900))
    orig = read_enabled(d.guest.mods)
    d.log("guest enabled.txt during the session:", orig)


def scenario_b(d):
    write_script_mod(d.guest.mods, "ms-extra", "Extra gameplay mod")
    base = read_enabled(d.guest.mods)
    d.host.launch("ms_host.lua")
    code = host_code(d)
    d.guest.launch("ms_guest.lua", {"MELEE_LAB_SYNC": "1", "MELEE_LAB_LEAVE": "1"}, room=code)
    d.wait("guest: room line", lambda: d.guest.has(r"MS_ROOM "), 600)
    line = re.search(r"MS_ROOM [^\n]*", d.guest.text())
    d.log(line.group(0) if line else "no MS_ROOM")
    d.check("the extra mod is a difference that blocks the match", line and "blocked=true" in line.group(0) and "ms-extra:mine_only" in line.group(0), line.group(0) if line else "")
    d.check("a DISABLE action was planned", d.guest.has(r"MS_ACTION disable ms-extra"))
    d.check("the host did not start a match while blocked", not d.host.has("MS_MATCH_STARTED"))
    d.wait("guest restarted", lambda: d.guest.has(r"a synced mod set is active"), 600)
    en = read_enabled(d.guest.mods)
    d.check("during the session ms-extra is off", en is not None and "ms-extra" not in en, en)
    d.check("the match starts after the sync", d.wait("match", lambda: d.guest.has("MS_MATCH_STARTED") and d.host.has("MS_MATCH_STARTED"), 900))
    d.check("leaving the room puts the player's own set back at once", d.wait("ended", lambda: d.guest.has("MS_ENDED"), 120) and read_enabled(d.guest.mods) is not None and "ms-extra" in read_enabled(d.guest.mods), read_enabled(d.guest.mods))
    # leave the room (the guest window is closed the way a player quits the room: the scripted way is gd.modsync_act("end"))
    d.guest.kill()
    time.sleep(2)
    # a normal quit leaves the journal ACTIVE with a dead pid: the next boot restores
    d.guest.proc, d.guest.acc, d.guest.pos, d.guest.sig = None, "", 0, None
    d.guest.launch("ms_prep.lua", {"MELEE_LAB_POST": "0"})
    d.wait("guest: restored at boot", lambda: d.guest.has(r"your own mod set was restored"), 300)
    en = read_enabled(d.guest.mods)
    d.check("the player's own enabled set is back (ms-extra on)", en is not None and "ms-extra" in en, en)
    d.check("the journal is gone", not os.path.exists(d.guest.mods + "/.modsync/state.txt"))


def scenario_c(d):
    write_script_mod(d.host.mods, "ms-hostonly", "Host only mod")  # no source: not from Nucleus
    d.host.launch("ms_host.lua")
    code = host_code(d)
    d.guest.launch("ms_guest.lua", {"MELEE_LAB_SYNC": "1"}, room=code)
    d.wait("guest: sync refused", lambda: d.guest.has(r"MS_SYNC_STARTED ok=false"), 900)
    line = re.search(r"MS_SYNC_STARTED[^\n]*", d.guest.text())
    d.log(line.group(0) if line else "none")
    d.check("the sync says the mod is missing, in words", line and "you need this mod" in line.group(0).lower() or d.guest.has(r"you need this mod: Host only mod"), line.group(0) if line else "")
    d.check("a MISSING action names the mod", d.guest.has(r"MS_ACTION missing ms-hostonly"))
    d.check("no download was queued", not d.guest.has(r"modsync: \d+ post"))
    d.wait("guest still blocked", lambda: d.guest.has(r"MS_STILL_BLOCKED"), 300)
    d.check("no match started", not d.guest.has("MS_MATCH_STARTED") and not d.host.has("MS_MATCH_STARTED"))
    d.check("the guest's enabled set is untouched", read_enabled(d.guest.mods) is None or "ms-hostonly" not in read_enabled(d.guest.mods))


def scenario_d(d):
    write_script_mod(d.guest.mods, "ms-extra", "Extra gameplay mod")
    open(d.guest.mods + "/enabled.txt", "w").write("# own set\nms-extra\n")
    own = read_enabled(d.guest.mods)
    d.host.launch("ms_host.lua")
    code = host_code(d)
    d.guest.launch("ms_guest.lua", {"MELEE_LAB_SYNC": "1", "MELEE_MODSYNC_CRASH_AT": "applied"}, room=code)
    d.wait("guest died after APPLIED", lambda: d.guest.has(r"TEST - dying after APPLIED"), 600)
    time.sleep(5)
    d.guest.kill()
    mid = read_enabled(d.guest.mods)
    d.check("the sync had changed enabled.txt before the crash", mid != own, mid)
    d.check("the journal was left behind", os.path.exists(d.guest.mods + "/.modsync/state.txt"))
    d.guest.proc, d.guest.acc, d.guest.pos, d.guest.sig = None, "", 0, None
    d.guest.launch("ms_prep.lua", {"MELEE_LAB_POST": "0"})
    d.wait("restored at the next boot", lambda: d.guest.has(r"your own mod set was restored"), 300)
    d.check("enabled.txt is the player's own set again", read_enabled(d.guest.mods) == own, read_enabled(d.guest.mods))
    d.check("the journal is gone", not os.path.exists(d.guest.mods + "/.modsync/state.txt"))


def main():
    sc, tag = sys.argv[1], sys.argv[2]
    d = Driver(sc, tag)
    try:
        {"a": scenario_a, "b": scenario_b, "c": scenario_c, "d": scenario_d}[sc](d)
    except Exception as e:  # keep the evidence
        import traceback
        d.log("EXCEPTION", traceback.format_exc())
        d.check("driver ran to the end", False, repr(e))
    return d.finish()


if __name__ == "__main__":
    sys.exit(main())
