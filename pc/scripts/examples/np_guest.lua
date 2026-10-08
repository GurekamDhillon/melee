-- np_guest.lua - INPUT SCRIPT for two-window netplay tests, GUEST side: Atlas main menu > ONLINE >
-- Join a Room, joins THE HOST'S ROOM BY CODE (never random matchmaking), then plays the lobby like
-- lobby_autoplay. The code comes from MELEE_LAB_ROOM (gd.lab_env("ROOM"), set by the driver once
-- the host has logged "ROOMCODE <code>") and goes in with gd.netplay_act("code", code); failing
-- that, from whoever drives the test (np_drive.py sends the same call over the console socket) or
-- Y (pastes the clipboard, which a host on the same machine has just copied).
--
--   MELEE_SCENE=mode=menu MELEE_SCRIPT=builtin:np_guest
--
-- @name: Netplay test - guest
-- @version: 1.0.0
-- @gameplay: true

local function step(msg)
  gd.log("np_guest: " .. msg)
  gd.label("np_guest: " .. msg)
end

local function pick(ok, max)
  for _ = 1, max or 12 do
    if ok() then break end
    gd.press(1, "Down", 4)
    gd.wait(8)
  end
  gd.press(1, "A", 4)
  gd.wait(10)
end

local function code_ready()
  local c = gd.netplay().code
  return #c == 4 and not c:find("?", 1, true)
end

gd.run(function()
  gd.wait_until(function() return gd.scene().name == "GS_FRONTEND" end, 3600)
  gd.wait(30)
  step("main menu -> ONLINE")
  -- the Atlas main menu reports its ONLINE row as hovered = 68 (0x44)
  pick(function() return gd.menu().native_hovered == 68 end)
  if not gd.wait_until(function() return gd.menu().screen == "ONLINE PLAY" end, 600) then
    step("never reached ONLINE PLAY") return
  end
  step("ONLINE PLAY -> Join a Room")
  gd.wait(60)
  -- Host a Room is the first row; the readback does not follow the Atlas cursor, so go Down once
  gd.press(1, "Down", 4) gd.wait(40)
  gd.press(1, "A", 4) gd.wait(10)
  gd.wait_until(function() return gd.menu().screen == "JOIN ROOM" end, 600)
  local env = gd.lab_env("ROOM")
  if env and #env == 4 then
    step("room code from MELEE_LAB_ROOM: " .. env)
    gd.netplay_act("code", env)
  end
  step("waiting for the room code")
  local t = 0
  while not code_ready() and t < 36000 do
    if t % 300 == 299 then gd.press(1, "Y", 4) end -- clipboard, every 5 s
    gd.wait(1)
    t = t + 1
  end
  if not code_ready() then step("no room code") return end
  step("joining " .. gd.netplay().code)
  gd.press(1, "A", 4)
  if not gd.wait_until(function() return gd.netplay().phase == "lobby" end, 3600) then
    step("could not join: " .. gd.netplay().status) return
  end
  step("lobby")
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
      step("connection failed: " .. np.status) return
    end
  end
  step("MATCH STARTED")
end)
