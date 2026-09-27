-- A held gd.input covers exactly N LOGIC frames, paused and stepped included (gw_script_pad.c,
-- gw_Script_PadFrameConsumed). Run as MELEE_PAD_SCRIPT in the LAB (P1 human Fox), realtime and turbo:
-- paused, let 20 ticks pass (the pad alarm keeps sampling, no logic frame runs), hold side+B for 3
-- frames, let 20 more paused ticks pass, step 3 frames: Fox must be in a Side B state (SpecialS*).
-- A hold eaten by the paused ticks gives neutral B (SpecialN*). Driven from on_tick, which runs every
-- tick paused or not (gd.run tasks advance only on logic frames). Logs "INPUTHOLD PASS" or
-- "INPUTHOLD FAIL <why>", then quits.
local phase, ticks, action0, frames_after = "wait_match", 0, nil, 0

local function finish(ok, why)
  gd.log((ok and "INPUTHOLD PASS " or "INPUTHOLD FAIL ") .. why)
  gd.quit()
  phase = "done"
end

function on_tick()
  ticks = ticks + 1
  if phase == "wait_match" then
    local m = gd.match()
    if m and m.active and m.frame > 90 and gd.player(1) then
      gd.pause()
      phase, ticks = "paused1", 0
    elseif ticks > 20000 then
      finish(false, "no match")
    end
  elseif phase == "paused1" and ticks >= 20 then
    local p = gd.player(1)
    action0 = p.action
    gd.input(1, {buttons = "B", x = 127 * (p.facing or 1), y = 0}, 3)
    phase, ticks = "paused2", 0
  elseif phase == "paused2" and ticks >= 20 then
    gd.step(3)
    phase, ticks = "stepped", 0
  elseif phase == "stepped" and ticks >= 20 then
    local q = gd.player(1)
    local name = gd.motion_name(q.action, 1)
    finish(name ~= nil and name:find("^SpecialS") ~= nil, "(" .. tostring(name) .. " after 3 stepped frames)")
  end
end
