#!/usr/bin/env python3
"""Write rust/melee_modes/src/tu/gmscdata.rs from src/melee/gm/gmscdata.c (the two tables are
transcribed mechanically; the four functions are hand-written below). Run once from the worktree root;
afterwards `gen.py ext gmscdata` refreshes the ext block."""
import re, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import model  # noqa: E402

ROOT = model.ROOT
SRC = (ROOT / "src/melee/gm/gmscdata.c").read_text(encoding="utf-8")

MINE = {k: v for k, v in model.mine().items() if v != 'gmscdata'}


def strip(s):
    s = re.sub(r"/\*.*?\*/", "", s, flags=re.S)
    s = re.sub(r"//[^\n]*", "", s)
    return s


def table(name):
    m = re.search(r"static \w+ " + name + r"\[\] = \{(.*?)\n\};", SRC, re.S)
    body = strip(m.group(1))
    out, depth, cur, on = [], 0, "", True
    # preprocessor: TARGET_PC is defined, nothing else is conditional in these tables
    lines = [l for l in body.split("\n") if not l.strip().startswith("#")]
    body = "\n".join(lines)
    ent = []
    for ch in body:
        if ch == "{":
            depth += 1
            if depth == 1:
                cur = ""
                continue
        if ch == "}":
            depth -= 1
            if depth == 0:
                ent.append([t.strip() for t in cur.replace("\n", " ").split(",") if t.strip()])
                continue
        if depth >= 1:
            cur += ch
    return ent


def fn(name, ty):
    if name == "NULL":
        return "None"
    mod = MINE.get(name)
    ref = f"super::{mod}::{name}" if mod else f"ext::{name}"
    return f"Some({ref} as {ty})"


F0 = 'unsafe extern "C" fn()'
F1 = 'unsafe extern "C" fn(*mut c_void)'
scenes = table("scenes")
modes = table("modes")
S = []
for e in scenes:
    assert len(e) == 5, e
    S.append(f"    GameScene {{ kind: {e[0]} as u8, on_frame: {fn(e[1], F0)}, on_enter: {fn(e[2], F1)}, on_exit: {fn(e[3], F1)}, unused: null_mut() }},")
M = []
for e in modes:
    assert len(e) == 6, e
    pre = {"true": "1", "false": "0"}[e[0]]
    st = "null_mut()" if e[5] == "NULL" else (f"&raw mut super::{MINE[e[5]]}::{e[5]} as *mut GameModeState" if e[5] in MINE else f"&raw mut ext::{e[5]} as *mut GameModeState")
    M.append(f"    GameMode {{ preloaded: {pre}, kind: {e[1]} as u8, on_load: {fn(e[2], F0)}, on_unload: {fn(e[3], F0)}, on_init: {fn(e[4], F0)}, states: {st} }},")

txt = f'''//! src/melee/gm/gmscdata.c: the minor-scene table (`scenes`, GS_* -> on_frame/on_enter/on_exit),
//! the major-scene table (`modes`, GM_* -> on_load/on_init/states) and their four accessors
//! (`gm_GetAllGameScenes`, `gm_GetAllGameModes`, and the TARGET_PC counters `gm_GetNumGameModes` /
//! `gm_GetNumGameScenes`).
//!
//! The tables are transcribed mechanically from the C (`oracle/s497/mk_scdata.py`, TARGET_PC on, so
//! they include GS_FRONTEND, GM_FRONTEND and GM_LAB). Entries naming functions or state tables of
//! this shard's other TUs point at the real ports; the rest point at the `ext` TODO stubs, which are
//! `extern "C"` so they can sit in the table. The state oracle `tests/s497_gmscdata.rs` checks the
//! accessors, the counters and every table entry (kind, flags, which function each slot names).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::ptr::null_mut;

use melee_types::*;

/// The port's own scene/mode kinds past the retail terminators (gmfrontend.h, pc/geno/geno_lab_mode.h).
pub const GM_FRONTEND: GameModeKind = GM_COUNT + 1;
pub const GS_FRONTEND: GameSceneKind = GS_COUNT + 1;
pub const GM_LAB: GameModeKind = GM_COUNT + 2;

// @@ext-begin
// @@ext-end

/// `scenes`: sentinel-terminated by `GS_COUNT`.
pub static mut scenes: [GameScene; {len(S)}] = unsafe {{
    [
{chr(10).join(S)}
    ]
}};

/// `modes`: sentinel-terminated by `GM_COUNT`.
pub static mut modes: [GameMode; {len(M)}] = unsafe {{
    [
{chr(10).join(M)}
    ]
}};

pub unsafe extern "C" fn gm_GetAllGameScenes() -> *mut GameScene {{
    scenes.as_mut_ptr()
}}

pub unsafe extern "C" fn gm_GetAllGameModes() -> *mut GameMode {{
    modes.as_mut_ptr()
}}

/// Major-scene count derived from the sentinel-terminated `modes` table (m-ex swaps this accessor to
/// return an MxDt-supplied table; the port exposes the same data-derived count).
pub unsafe extern "C" fn gm_GetNumGameModes() -> i32 {{
    let base = modes.as_mut_ptr();
    let mut cur = base;
    while (*cur).kind as u32 != GM_COUNT {{
        cur = cur.add(1);
    }}
    cur.offset_from(base) as i32
}}

/// Minor-scene count derived from the sentinel-terminated `scenes` table. The table has a hole at
/// GS_0x6, so the count is the number of entries, not the terminator value.
pub unsafe extern "C" fn gm_GetNumGameScenes() -> i32 {{
    let base = scenes.as_mut_ptr();
    let mut cur = base;
    while (*cur).kind as u32 != GS_COUNT {{
        cur = cur.add(1);
    }}
    cur.offset_from(base) as i32
}}
'''
(ROOT / "rust/melee_modes/src/tu/gmscdata.rs").write_text(txt, encoding="utf-8", newline="\n")
print(len(S), "scenes", len(M), "modes")
