# SPDX-License-Identifier: MIT
"""Gecko codes that boot the vanilla NTSC 1.02 disc straight into one VS match of CPUs (no input, no menus), for the
covtest seed-match recorder (record.py, notes/coverage-testing.md). The decomp names the addresses
(config/GALE01/symbols.txt); the GameCube struct offsets are melee_types' generated layout (rust/melee_types/src/gen.rs).

1. 041BFA20 38600002   bootOnLeave: `lbz r3,4(r31)` (the next mode) -> `li r3,2`: the boot scene goes to GM_VS
                       (the known "boot to CSS" code; the decomp's gmboot.c notes it).
2. C21A55EC ...        gmVsMelee_Mode_OnLoad's first instruction: what the port's SceneLaunch_SeedVs does
                       (melee_modes::tu::gmscenelaunch) written out as stores: the VsModeData players and rules, the
                       saved GameRules/GamePrefs that gm_80167BC8 rebuilds the match rules from, the stage ids,
                       the preload cache entries, then lbDvd_SetupVsPreloadCache() and gm_SetGameModeStateId(2), so the
                       mode's state machine starts in the match (state 2) instead of the CSS.
Slippi's own General Codes and Slippi Recording (from the Dolphin build's Sys/GameSettings/Netplay ini, the set Slippi
online plays with) are enabled next to these, so the console writes a normal .slp with the CPUs' inputs.
"""

GMM_PTR = 0x804D3EE0          # gmMainLib_804D3EE0 (struct gmm_x0 *)
VS_START = 0x588 + 0x8 + 0x8  # gmm_x0.modes.table[GmVsMode_Melee].start (VsModeData +8)
RULES = VS_START              # StartMeleeRules, 0x60
PLAYERS = VS_START + 0x60     # PlayerInitData[6], 0x24 each
GAME_RULES = 0x1850           # gmm_x0.x1850 (GameRules)
GAME_PREFS = 0x1868 + 0x448   # gmm_x0.thing.save_data.x1CB0 (GamePrefs)
CSS_STAGE_ID = 0x80473814 + 6  # gm_80473814.stage_id (s16)
PRELOAD_GAME_CACHE = 0x80432078 + 4 + 8  # preloadCache.scene.game_cache
STATE_MACHINE = 0x80479D30    # state_machine.routing: curr_state_id +3, prev_state_id +4
SETUP_VS_PRELOAD = 0x800174BC  # lbDvd_SetupVsPreloadCache
ONLOAD = 0x801A55EC           # gmVsMelee_Mode_OnLoad, first instruction `mflr r0` (7C0802A6)
BOOT_NEXT_MODE = 0x801BFA20   # bootOnLeave `lbz r3,4(r31)` (887F0004)
CHKIND_NONE = 0x21
PKIND_CPU, PKIND_NA = 1, 3


def _d(op, d, a, imm):
    return (op << 26) | (d << 21) | (a << 16) | (imm & 0xFFFF)


def addis(d, a, v): return _d(15, d, a, v)
def addi(d, a, v): return _d(14, d, a, v)
def ori(a, s, v): return _d(24, s, a, v)
def lwz(d, off, a): return _d(32, d, a, off)
def stw(s, off, a): return _d(36, s, a, off)
def stwu(s, off, a): return _d(37, s, a, off)
def stb(s, off, a): return _d(38, s, a, off)
def sth(s, off, a): return _d(44, s, a, off)


MFLR_R0, MTLR_R0, MTCTR_R12, BCTRL, NOP = 0x7C0802A6, 0x7C0803A6, 0x7D8903A6, 0x4E800421, 0x60000000


def load32(r, v):
    v &= 0xFFFFFFFF
    if v < 0x8000 or v >= 0xFFFF8000:
        return [addi(r, 0, v & 0xFFFF)]
    out = [addis(r, 0, v >> 16)]
    if v & 0xFFFF:
        out.append(ori(r, r, v & 0xFFFF))
    return out


def ha(v):
    return ((v >> 16) + (1 if v & 0x8000 else 0)) & 0xFFFF


class Asm:
    def __init__(self):
        self.w = []

    def poke(self, width, base_reg, off, value):
        self.w += load32(4, value)
        self.w.append({1: stb, 2: sth, 4: stw}[width](4, off, base_reg))


def onload_block(cfg):
    """cfg: players [(ckind, cpu_level, color, team)], stage (external StKind), items (bool), minutes, stocks, teams."""
    a = Asm()
    a.w += [MFLR_R0, stw(0, 4, 1), stwu(1, -0x20, 1)]
    a.w += [addis(5, 0, ha(GMM_PTR)), lwz(5, GMM_PTR & 0xFFFF, 5)]  # r5 = gmm
    teams = cfg.get("teams", False)
    n = len(cfg["players"])
    for i in range(6):
        b = PLAYERS + i * 0x24
        if i < n:
            ck, lvl, color, team = cfg["players"][i]
            a.poke(1, 5, b + 0x0, ck)
            a.poke(1, 5, b + 0x1, PKIND_CPU)
            a.poke(1, 5, b + 0x3, color)
            a.poke(1, 5, b + 0x4, i + 1)          # slot
            a.poke(1, 5, b + 0x7, 0)              # sub_color
            a.poke(1, 5, b + 0x9, team if teams else 0)
            a.poke(1, 5, b + 0xE, 4)              # cpu_kind: VS mode's fighting AI
            a.poke(1, 5, b + 0xF, lvl)
        else:
            a.poke(1, 5, b + 0x0, CHKIND_NONE)
            a.poke(1, 5, b + 0x1, PKIND_NA)
    a.poke(1, 5, RULES + 0x8, 1 if teams else 0)   # is_teams
    a.poke(2, 5, RULES + 0xE, cfg["stage"])         # stkind
    # GameRules: stock match, stocks, the time limit of a stock match, no handicap, 1.0x damage, team attack off
    gr = GAME_RULES
    a.poke(1, 5, gr + 0x2, 1)
    a.poke(1, 5, gr + 0x4, cfg["stocks"])
    a.poke(1, 5, gr + 0x3, cfg["minutes"])
    a.poke(1, 5, gr + 0x8, cfg["minutes"])
    a.poke(1, 5, gr + 0x5, 0)
    a.poke(1, 5, gr + 0x6, 10)
    a.poke(1, 5, gr + 0x9, 0)
    # GamePrefs: item frequency (0xFF off, 2 medium) and every item switch on
    gp = GAME_PREFS
    a.poke(1, 5, gp + 0x0, 2 if cfg["items"] else 0xFF)
    a.poke(4, 5, gp + 0x8, 0xFFFFFFFF)
    a.poke(4, 5, gp + 0xC, 0xFFFFFFFF)
    # absolute: the CSS stage id and the preload cache (r6)
    a.w += [addis(6, 0, ha(CSS_STAGE_ID)), addi(6, 6, CSS_STAGE_ID & 0xFFFF)]
    a.poke(2, 6, 0, cfg["stage"])
    pc = PRELOAD_GAME_CACHE
    a.w += [addis(6, 0, ha(pc)), addi(6, 6, pc & 0xFFFF)]
    a.poke(4, 6, 0x4, cfg["stage"])
    for i in range(8):
        a.poke(4, 6, 0x8 + 8 * i, cfg["players"][i][0] if i < n else CHKIND_NONE)
        a.poke(1, 6, 0x8 + 8 * i + 4, cfg["players"][i][2] if i < n else 0)
    # lbDvd_SetupVsPreloadCache(), then gm_SetGameModeStateId(2)
    a.w += load32(12, SETUP_VS_PRELOAD) + [MTCTR_R12, BCTRL]
    a.w += [addis(6, 0, ha(STATE_MACHINE)), addi(6, 6, STATE_MACHINE & 0xFFFF)]
    a.poke(1, 6, 3, 2)
    a.poke(1, 6, 4, 2)
    a.w += [lwz(0, 0x24, 1), addi(1, 1, 0x20), MTLR_R0]
    a.w.append(MFLR_R0)  # the replaced instruction
    if len(a.w) % 2 == 0:
        a.w.append(NOP)
    a.w.append(0)  # the code handler's branch back
    lines = [f"C2{ONLOAD & 0xFFFFFF:06X} {len(a.w) // 2:08X}"]
    for k in range(0, len(a.w), 2):
        lines.append(f"{a.w[k]:08X} {a.w[k + 1]:08X}")
    return lines


def slippi_blocks(netplay_ini):
    """The Netplay ini's `$Required: General Codes` and `$Required: Slippi Recording`, renamed."""
    want = {"Required: General Codes": "CovTest: Slippi General Codes", "Required: Slippi Recording": "CovTest: Slippi Recording"}
    out, cur, sect = {}, None, None
    for line in open(netplay_ini, encoding="utf-8", errors="replace"):
        line = line.rstrip("\r\n")
        if line.startswith("["):
            sect, cur = line, None
            continue
        if sect != "[Gecko]":
            continue
        if line.startswith("$"):
            cur = want.get(line[1:].split(" [")[0].strip())
            if cur:
                out[cur] = []
            continue
        if cur and line and not line.startswith("*"):
            out[cur].append(line)
    missing = set(want.values()) - set(out)
    if missing:
        raise SystemExit(f"gecko_boot: {missing} not in the Netplay game settings")
    return out


SYS_CODES = ["Required: General Codes", "Required: Slippi Playback", "Recommended: Slippi Recording", "Optional: Show Player Names"]


def game_ini(cfg, netplay_ini):
    blocks = slippi_blocks(netplay_ini)
    blocks["CovTest: boot to VS"] = [f"04{BOOT_NEXT_MODE & 0xFFFFFF:06X} 38600002"]
    blocks["CovTest: seed the VS match"] = onload_block(cfg)
    out = ["# covtest (GD's Melee port, local tool): generated per match by tools/rust-port/covtest/record.py", "[Gecko]"]
    for nm, lines in blocks.items():
        out.append(f"${nm}")
        out += lines
    out.append("[Gecko_Enabled]")
    out += [f"${nm}" for nm in blocks]
    out.append("[Gecko_Disabled]")
    out += [f"${nm}" for nm in SYS_CODES]
    return "\n".join(out) + "\n"
