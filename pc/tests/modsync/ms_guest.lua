-- ms_guest.lua - TEST DRIVER, guest side of the mod-sync loopback proofs (np_modsync.py). Joins THE HOST'S ROOM BY CODE (MELEE_LAB_ROOM), reads what the room says about the
-- mods (gd.modsync), and when MELEE_LAB_SYNC=1 presses "Match the host's mods" (gd.modsync_act("sync")). The game then restarts itself; this script runs again in the new
-- process, finds gd.modsync().active and just rejoins and plays the lobby. Every observation is a "MS_..." log line the driver greps.
-- @name: Mod sync test - guest
-- @version: 1.0.0
-- @gameplay: true
local function step(msg) gd.log("ms_guest: " .. msg) gd.label("ms_guest: " .. msg) end
local function pick(ok, max)
  for _ = 1, max or 12 do
    if ok() then break end
    gd.press(1, "Down", 4)
    gd.wait(8)
  end
  gd.press(1, "A", 4)
  gd.wait(10)
end
local function rows_text(ms)
  local t = {}
  for _, r in ipairs(ms.rows or {}) do t[#t + 1] = r.id .. ":" .. r.state .. (r.ident and "" or "(fine)") end
  return table.concat(t, ",")
end
gd.run(function()
  gd.wait_until(function() return gd.scene().name == "GS_FRONTEND" end, 3600)
  gd.wait(30)
  pick(function() return gd.menu().native_hovered == 68 end)
  if not gd.wait_until(function() return gd.menu().screen == "ONLINE PLAY" end, 600) then step("never reached ONLINE PLAY") return end
  gd.wait(60)
  gd.press(1, "Down", 4) gd.wait(40)
  gd.press(1, "A", 4) gd.wait(10)
  gd.wait_until(function() return gd.menu().screen == "JOIN ROOM" end, 600)
  local env = gd.lab_env("ROOM")
  if env and #env == 4 then gd.netplay_act("code", env) end
  local t = 0
  while (#gd.netplay().code ~= 4 or gd.netplay().code:find("?", 1, true)) and t < 36000 do gd.wait(1) t = t + 1 end
  gd.press(1, "A", 4)
  if not gd.wait_until(function() return gd.netplay().phase == "lobby" end, 3600) then step("could not join: " .. gd.netplay().status) return end
  step("lobby")
  gd.wait_until(function() return gd.modsync().peer_ready end, 1800)
  local ms = gd.modsync()
  gd.log(string.format("MS_ROOM active=%s blocked=%s peer_ready=%s common_stages=%d can_complete=%s rows=[%s] summary=%s", tostring(ms.active), tostring(ms.blocked), tostring(ms.peer_ready),
    gd.netplay().common_stages or -1, tostring(ms.can_complete), rows_text(ms), tostring(ms.summary)))
  for _, a in ipairs(ms.actions or {}) do gd.log("MS_ACTION " .. a.kind .. " " .. a.id .. " post=" .. a.post .. " " .. a.why) end
  if not ms.active and gd.lab_env("SYNC") == "1" then
    local ok = gd.modsync_act("sync")
    gd.log("MS_SYNC_STARTED ok=" .. tostring(ok) .. " status=" .. gd.modsync().status)
    if not ok then
      -- cannot complete: stay in the menus; the match must not start
      gd.wait(1800)
      gd.log("MS_STILL_BLOCKED phase=" .. gd.netplay().phase .. " match=" .. tostring(gd.match().active))
      return
    end
    while true do gd.wait(60) end -- the game restarts itself
  end
  while not gd.match().active do
    gd.wait(30)
    local np = gd.netplay()
    if np.phase == "lobby" then
      local me = np.players[np.me + 1]
      local lp = np.lobby
      if (lp == "char_blind" and not me.locked) or ((lp == "char_winner" or lp == "char_loser") and np.turn == np.me) then
        gd.netplay_act("char")
      elseif (lp == "strike" or lp == "ban" or lp == "pick") and np.turn == np.me then
        for i, st in ipairs(np.stages) do if st == 0 then gd.netplay_act("stage", i) break end end
      elseif lp == "ready" and not me.ready then
        gd.netplay_act("ready", true)
      end
    elseif np.phase == "failed" then
      step("connection failed: " .. np.status) return
    end
  end
  gd.log("MS_MATCH_STARTED blocked=" .. tostring(gd.modsync().blocked))
  if gd.modsync().active and gd.lab_env("LEAVE") == "1" then gd.modsync_act("end") gd.log("MS_ENDED") end
end)
