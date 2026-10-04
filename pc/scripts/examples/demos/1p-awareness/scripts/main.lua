-- One capability: read and log retail 1P lifecycle snapshots.
local status='Enter an offline Classic or Adventure match'
local function event(name,e)
 status=('%s: %s match %d NG+%d (%s), %d opponents'):format(name,e.mode,e.stage_index,e.loop,e.stage_kind,#e.opponents)
 gd.log(status)
end
function on_1p_stage_start(e) event('start',e) end
function on_1p_stage_clear(e) event('clear',e) end
function on_1p_boss_defeated(e) event(e.boss_kind,e) end
function on_1p_game_over(e) event('game over',e) end
function on_1p_complete(e) event('complete',e) end
function on_draw() gd.kit.text(status,24,28,14,'bone') end
