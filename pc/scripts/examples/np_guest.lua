-- np_guest.lua - INPUT SCRIPT for two-window netplay tests, GUEST side: main menu > VERSUS >
-- ONLINE > Join a Room, fills in the room code, joins, then plays the lobby like lobby_autoplay.
-- The code comes from whoever drives the test (np_drive.py sends gd.netplay_act("code", ...) over
-- the console socket); failing that, Y pastes it from the clipboard, which a host on the same
-- machine has just copied. The Lua form of _build/pad_np_menu_guest.txt + pad_np_ready_guest.txt.
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
  step("main menu -> VERSUS")
  pick(function() return gd.menu().native_hovered == 1 end)
  gd.wait_until(function() return gd.menu().native_menu == 2 end, 600)
  gd.wait(20)
  step("VERSUS -> ONLINE")
  -- the VERSUS menu reports the port's ONLINE row as hovered = 64 (0x40), between Melee and
  -- Tournament Melee
  pick(function() return gd.menu().native_hovered == 64 end)
  if not gd.wait_until(function() return gd.menu().screen == "ONLINE PLAY" end, 600) then
    step("never reached ONLINE PLAY") return
  end
  step("ONLINE PLAY -> Join a Room")
  pick(function() return gd.menu().item == "Join a Room" end)
  gd.wait_until(function() return gd.menu().screen == "JOIN ROOM" end, 600)
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
