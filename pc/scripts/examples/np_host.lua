-- np_host.lua - INPUT SCRIPT for two-window netplay tests, HOST side: from the Atlas main menu to
-- a match through the real menus (ONLINE > Host a Room; ONLINE is a main-menu row now, hovered =
-- 68 = 0x44), then plays the lobby like lobby_autoplay. Every step waits for the game's own state
-- (gd.menu, gd.netplay), not for frame counts. It logs "ROOMCODE <code>" for whoever starts the
-- guest (np_guest takes it from MELEE_LAB_ROOM or the console).
--
--   MELEE_SCENE=mode=menu MELEE_SCRIPT=builtin:np_host   (tools/netplay/np_drive.py does this)
--
-- @name: Netplay test - host
-- @version: 1.0.0
-- @gameplay: true

local function step(msg)
  gd.log("np_host: " .. msg)
  gd.label("np_host: " .. msg)
end

-- Move the cursor down until `ok()` holds (at most `max` presses), then press A.
local function pick(ok, max)
  for _ = 1, max or 12 do
    if ok() then break end
    gd.press(1, "Down", 4)
    gd.wait(8)
  end
  gd.press(1, "A", 4)
  gd.wait(10)
end

local function to_online()
  gd.wait_until(function() return gd.scene().name == "GS_FRONTEND" end, 3600)
  gd.wait(30)
  step("main menu -> ONLINE")
  -- the Atlas main menu reports its ONLINE row as hovered = 68 (0x44)
  pick(function() return gd.menu().native_hovered == 68 end)
  return gd.wait_until(function() return gd.menu().screen == "ONLINE PLAY" end, 600)
end

local function play_lobby()
  while not gd.match().active do
    gd.wait(30)
    local np = gd.netplay()
    if np.phase == "lobby" then
      local me = np.players[np.me + 1]
      local lp = np.lobby
      if (lp == "char_blind" and not me.locked) or
         ((lp == "char_winner" or lp == "char_loser") and np.turn == np.me) then
        gd.netplay_act("char")
      elseif (lp == "strike" or lp == "ban" or lp == "pick") and np.turn == np.me then
        for i, st in ipairs(np.stages) do
          if st == 0 then gd.netplay_act("stage", i) break end
        end
      elseif lp == "ready" and not me.ready then
        gd.netplay_act("ready", true)
      end
    elseif np.phase == "failed" then
      step("connection failed: " .. np.status)
      return false
    end
  end
  return true
end

gd.run(function()
  if not to_online() then step("never reached ONLINE PLAY") return end
  step("ONLINE PLAY -> Host a Room")
  pick(function() return gd.menu().item == "Host a Room" end)
  if not gd.wait_until(function() local c = gd.netplay().code return c ~= "" and not c:find("?", 1, true) end, 1800) then
    step("no room code from the server") return
  end
  step("room " .. gd.netplay().code .. " - waiting for the guest")
  gd.log("ROOMCODE " .. gd.netplay().code)
  if not gd.wait_until(function() return gd.netplay().phase == "lobby" end, 36000) then
    step("the guest never arrived") return
  end
  step("lobby")
  if play_lobby() then step("MATCH STARTED") end
end)
