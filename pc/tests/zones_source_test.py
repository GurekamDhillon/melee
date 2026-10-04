"""Packet integration checks without compiling or launching the game."""
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parents[3]
game = root / 'melee'
native = (game / 'pc/platform/gw_script.c').read_text(encoding='utf-8')
adapter = (game / 'pc/platform/gw_script_zones.inc').read_text(encoding='utf-8')
snap = (game / 'pc/platform/gw_snap.c').read_text(encoding='utf-8')
frame = native.split('void gw_Script_FramePost(void)', 1)[1]
assert frame.index('gw_ScriptGame_ZonesFrame(0)') < frame.index('gw_Snap_Resimulating()')
assert frame.index('gs_zones_dispatch()') < frame.index('gs_hook_all("on_frame"')
assert 'strcmp(obj, "pc_gameworld_script_game.c.obj") == 0' in snap
assert '#include "script_zones.inc"' in (game / 'pc/gameworld/script_game.c').read_text()
assert 'if(!strcmp(phase,"after"))gs_zones_release(0)' in (game / 'pc/platform/gw_script_stage_slots.inc').read_text()
names = {'zone_add','zone_set','zone_remove','zones','zones_at','zone_members','point_zones'}
registered = set(re.findall(r'\{"([a-zA-Z_0-9]+)",\s*l_', native))
assert names <= registered
demo = game / 'pc/scripts/examples/demos/zones'
manifest = json.loads((demo / 'mod.json').read_text())
assert manifest['gameplay'] and not manifest['rollback_safe']
calls = set(re.findall(r'gd\.([a-zA-Z_0-9]+)\s*[(\{]', (demo / 'scripts/main.lua').read_text()))
assert calls <= registered, calls - registered
assert all(hook in adapter for hook in ('on_zone_enter','on_zone_exit','on_zone_none','on_zone_some'))
rewind = (game / 'pc/tests/zones_rewind.lua').read_text()
assert 'r.pass and r.diff==0' in rewind and 'gd.zone_add' in rewind
assert '## Zones' in (root / 'docs/scripting.md').read_text(encoding='utf-8')
print('zones source integration: PASS (registration, frame order, snapshot eligibility, stage lifecycle, demo API, rewind assertion)')
