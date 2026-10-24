"""Deterministic random operation script for the gw_log.c logger oracle."""
import random
r = random.Random(20261025)
H = lambda s: s.encode("latin1").hex() if s else "-"
prefixes = ["scene: enter 12", "scene: exit 3", "gw: scene: x", "scene: tick", "gw: DIAG pads 1", "gw: DIAG gcraw 4",
  "gw: gc adapter up", "gw: input ok", "gw: pad 2", "pad: a", "gw: PAD z", "netplay: ready", "rb: frame 5", "net: x",
  "gw: netplay up", "gw: DIAG other", "gw: DIAG proj 1", "gw: DIAG posmtx 2", "gw: DIAG copy #1", "gw: DIAG   mtx",
  "gw: DIAG copy #2", "gw: at 0x1234", "gw:   retrace=5", "gxtex: a -> handle 3", "gxtex: plain", "frontend: a.json: 5 bytes, ok",
  "frontend: nope", "interp: installed", "interp: other", "ftfunction: 5", "itfunction: 1", "grfunction:   2", "grtrace: x",
  "mexdata: MEX_Index 4", "mexeffect: bank 1", "mexeffect: fighter kind 2", "item: kind 3", "gw: kind 5 Kirby hat: x",
  "lbHeap: heap 1", "heap: a", "gw: ARAlloc(5)", "gw: dvd: open", "gw: mods:   /a", "synth: bank 2", "lbAudioAx: m-ex sfx 1",
  "snap: render pass 3", "gobj: draw 4", "gw: disc image C:\Games\Melee\melee.iso, disk id GALE01", "gw: disc image x, disk id GALJ01  ",
  "gw: mods: gdmods 5 new paths)", "gw: mods: other 2 new paths) tail", "gw: mods: nospace new paths)", "melee-pc: window closed, shutting down",
  "Failed to open", "error: bad thing", "warning: low", "x adapter y", "hello", "boot ok", "audio ready", "OVERFLOW noted", "too big", "not found", "tick"]
sfx = ["", " 1", " 22", " [a]", " C:\Games\Melee\melee.iso", " c:/games/melee/melee.iso", " <x>", " " + "z" * 300, " " + "q" * 400]
envs = ["MELEE_HEAP_TRACE", "MELEE_DVD_TRACE", "MELEE_GR_TRACE", "MELEE_MEX_TRACE_CALLS", "MELEE_RB_LOG", "MELEE_CSS_TRACE",
        "MELEE_PC_TRACE_OSREPORT", "MELEE_LOG", "MELEE_ISO", "GW_ISO_VANILLA", "GW_ISO"]
logv = ["", "all", "verbose", "quiet", "none", "default", "-scene", "+render", "-render", "render,pad", "scene; net\tmex", "-all", "bogus,gobj", "!heap", "SCENE,Pad,-NET",
        "a" * 300 + ",dvd"]
ISOS = ['C:\Games\Melee\melee.iso', 'D:/x/y.iso', 'secret']
ISOS2 = ['C:\Games\Melee\melee.iso', 'c:/games/melee/melee.iso', 'zzz', 'x']
def line():
    s = r.choice(prefixes) + r.choice(sfx)
    return s
out = []
tick = 1000; frame = 0
def maybe_time():
    global tick, frame
    c = r.random()
    if c < 0.5:
        tick += r.choice([0, 1, 5, 50, 800, 9000, 10001, 20000, 4000000000]); frame += r.randint(0, 3)
        out.append(f"T {tick} {frame}")
out.append(f"T {tick} {frame}")
blocks = 0
while len(out) < 9000:
    blocks += 1
    if blocks % 50 == 0:
        out.append("RESET")  # a fresh process: the C logger configures once
    maybe_time()
    c = r.random()
    if c < 0.55: out.append("LOG " + H(line()))
    elif c < 0.62: out.append("LOG " + H("\n".join(line() for _ in range(r.randint(1, 4))) + r.choice(["", "\n", "\r\n"])))
    elif c < 0.72:
        s = r.choice(["%s :" % r.choice(prefixes), " %5d KB\n", line() + "\n", "\n", "\r\n", "x" * 1100 + "\n", line() + "\r\n" + line() + "\n", "gw: part"])
        out.append("OSR " + H(s))
    elif c < 0.75:
        for _ in range(r.randint(10, 450)): out.append("LOG " + H("netplay: spam %d" % r.randint(0, 2)))
    elif c < 0.77: out.append(f"RAW {H(r.choice(['gw: ', 'crash: ', 'x' * 400]))} " + r.choice(["~", H(line()), H("%s%n")]))
    elif c < 0.79: out.append("CRASH")
    elif c < 0.81: out.append("GOBJ")
    elif c < 0.83: out.append("DESC")
    elif c < 0.87:
        e = r.choice(envs)
        if e in ("MELEE_ISO", "GW_ISO_VANILLA", "GW_ISO"): out.append(f"ENV {e} {H(r.choice(ISOS))}")
        elif e == "MELEE_LOG": out.append(f"ENV {e} {H(r.choice([v for v in logv if v]))}")
        else: out.append(f"ENV {e} {H(r.choice(['1', '0', 'yes', '00']))}")
    elif c < 0.89: out.append(f"UNENV {r.choice(envs)}")
    elif c < 0.91: out.append(f"SLOG {H(r.choice(logv))}")
    elif c < 0.94: out.append(f"ISO {H(r.choice(ISOS2))}")
    elif c < 0.95: out.append("NOISO")
    else: out.append("ENVQ " + r.choice(envs))
print("\n".join(out))
