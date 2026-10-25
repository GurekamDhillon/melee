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
ISOS2 = ['disc.bin', 'disc.bin', 'C:\Games\Melee\melee.iso', 'c:/games/melee/melee.iso', 'zzz', 'x']
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
    elif c < 0.945: out.append("NOISO")
    elif c < 0.95: out.append("ENVQ " + r.choice(envs))
    elif c < 0.972:
        k = r.random()
        if k < 0.2: out.append("SUMM " + H(r.choice(["", "name=GD delay=2", "x" * 1500, "a=1 b=2 @USER@"])))
        elif k < 0.4: out.append("VERSION " + (H(r.choice(["1.2.3\n", "v9 @USER@\r\nsecond", "", "x" * 600 + "\n", "\n"])) if r.random() < 0.8 else "~"))
        elif k < 0.6:
            fn = "".join(f" 0001:{a:08x}       _fn{a:x}      {0x10000000 + a:08x} f   x.obj\n" for a in sorted(r.sample(range(0x1000, 0x90000, 0x10), r.randint(0, 40))))
            fn += " 0002:00000010 _data 10100000 f y.obj\n garbage line\n 0001:zz\n"
            stamp = r.choice(["@STAMP@", "@STAMP@", "00000000"])
            hdr = f" melee-pc\n\n Timestamp is {stamp} (Mon)\n\n" if r.random() < 0.9 else " no timestamp here\n"
            out.append("MAP " + (H(hdr + fn) if r.random() < 0.9 else "~"))
        elif k < 0.7: out.append("DISCFILE " + (H("A" * 0x20 + ("GALE01 title " * 6)[:0x40] + "tail") if r.random() < 0.7 else r.choice(["~", H("short")])))
        else:
            for _ in range(r.randint(1, 5)):
                a = r.choice([0, 0x10001000, 0x10001234, 0x1000ffff, 0x10020000, 0x1004fff0, 0x10200000, r.randint(0x10001000, 0x10090000)])
                out.append("LOG " + H(f"gw:     melee-pc.map rva 0x{a:08X}" + r.choice(["", " @USER@", " X"])))
            out.append("LOG " + H(r.choice(["gw: fault at @PROFILE@/game/x.c", "gw: c:/users/testy/save/@USER@.dat", "crash for @USER@ in @PROFILE@", "gw: disc image disc.bin, disk id GALE01"])))
    elif c < 0.985:
        fn = "".join(f" 0001:{a:08x}       _fn{a:x}      {0x10000000 + a:08x} f   x.obj\n" for a in sorted(r.sample(range(0x1000, 0x90000, 0x10), r.randint(1, 60))))
        out.append("MAP " + H(" melee-pc\n\n Timestamp is @STAMP@ (Mon)\n\n" + fn))
        out.append("LOG " + H("gw: disc image disc.bin, disk id GALE01"))
        for _ in range(r.randint(1, 60)):
            a = r.choice([0x10001000, 0x10001234, 0x1000ffff, 0x10020000, 0x10200000, r.randint(0x10001000, 0x10090000), r.randint(0x10001000, 0x10090000)])
            out.append("LOG " + H(f"gw:   frame rva 0x{a:08X} and rva 0x{r.randint(0x10001000, 0x10090000):08x}"))
        out.append("CRASH")
        for _ in range(r.randint(0, 80)):
            a = r.randint(0x10001000, 0x10090000)
            out.append("LOG " + H(f"gw:     melee-pc.map rva 0x{a:08X}"))
        out.append("REPORT " + H("panic") + " " + H("sym test") + " 2026/10/24/12/0/0")
    else:
        if r.random() < 0.7: out.append("CRASH")
        out.append("REPORT " + H(r.choice(["panic", "unhandled exception", "crash", "CRT invalid parameter"])) + " " + (H(r.choice(["why", "assert @USER@", "x" * 300, "C:/Users/Testy/a"])) if r.random() < 0.8 else "~") + f" {r.randint(2020, 2030)}/{r.randint(1, 12)}/{r.randint(1, 28)}/{r.randint(0, 23)}/{r.randint(0, 59)}/{r.randint(0, 59)}")
print("\n".join(out))
