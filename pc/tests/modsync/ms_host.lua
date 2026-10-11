-- ms_host.lua - TEST DRIVER, host side of the mod-sync loopback proofs: np_host (ONLINE > Host a Room, play the lobby) plus MS_ lines about the room's mods.
-- @name: Mod sync test - host
-- @version: 1.0.0
-- @gameplay: true
local function step(msg) gd.log("ms_host: " .. msg) gd.label("ms_host: " .. msg) end
local function pick(ok, max)
  for _ = 1, max or 12 do
    if ok() then break end
    gd.press(1, "Down", 4)
    gd.wait(8)
  end
  gd.press(1, "A", 4)
  gd.wait(10)
end
gd.run(function()
  gd.wait_until(function() return gd.scene().name == "GS_FRONTEND" end, 3600)
  gd.wait(30)
  pick(function() return gd.menu().native_hovered == 68 end)
  if not gd.wait_until(function() return gd.menu().screen == "ONLINE PLAY" end, 600) then step("never reached ONLINE PLAY") return end
  pick(function() return gd.menu().item == "Host a Room" end)
  if not gd.wait_until(function() local c = gd.netplay().code return c ~= "" and not c:find("?", 1, true) end, 1800) then step("no room code") return end
  gd.log("ROOMCODE " .. gd.netplay().code)
  local was = false
  while not gd.match().active do
    gd.wait(30)
    local np = gd.netplay()
    if np.phase == "lobby" then
      local ms = gd.modsync()
      if not was then was = true gd.log("MS_HOST_LOBBY blocked=" .. tostring(ms.blocked) .. " peer_ready=" .. tostring(ms.peer_ready)) end
      local me = np.players[np.me + 1]
      local lp = np.lobby
      if (lp == "char_blind" and not me.locked) or ((lp == "char_winner" or lp == "char_loser") and np.turn == np.me) then
        gd.netplay_act("char")
      elseif (lp == "strike" or lp == "ban" or lp == "pick") and np.turn == np.me then
        for i, st in ipairs(np.stages) do if st == 0 then gd.netplay_act("stage", i) break end end
      elseif lp == "ready" and not me.ready then
        gd.netplay_act("ready", true)
      end
    else
      was = false
    end
  end
  gd.log("MS_MATCH_STARTED host")
end)
