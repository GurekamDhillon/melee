"""gd.set_scale source contract: offline-only (netplay/rollback refused before any write), forks the rewind
timeline, clamps 0.5-2.0, and runs retail's size-change sequence. The in-game numbers are ports/goku-xv2/lua/scale_check.lua."""
import pathlib, re
root = pathlib.Path(__file__).resolve().parents[2]
gs = (root / 'pc/platform/gw_script.c').read_text(encoding='utf-8', errors='replace')
sg = (root / 'pc/gameworld/script_game.c').read_text(encoding='utf-8', errors='replace')
fn = gs[gs.index('static int l_set_scale'):]
fn = fn[:fn.index('\n}')]
assert fn.index('gs_require_offline(L, "set_scale")') < fn.index('gs_rw_branch()') < fn.index('gw_ScriptGame_LabSetScale'), 'order: refuse, fork, write'
assert '{"set_scale", l_set_scale}' in gs
body = sg[sg.index('int ScriptGame_LabSetScale'):]
body = body[:body.index('\n}')]
for call in ('ftLib_SetScale', 'ftCo_800D105C', 'ftCamera_80076064', 'ft_80081C88', 'x34_scale.x'):
    assert call in body, call
assert '0.5f' in body and '2.0f' in body
doc = (root / 'docs/geno.md').read_text(encoding='utf-8')
assert 'gd.set_scale(port, s)' in doc
print('set_scale source test PASS')
