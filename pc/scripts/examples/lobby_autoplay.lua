-- lobby_autoplay.lua - plays this side of the online lobby by itself: locks the current fighter,
-- strikes / bans / picks the first free stage on its turns, readies up. The script form of the old
-- native MELEE_LOBBY_AUTOPLAY switch (which now loads this built-in).
--
--   MELEE_LOBBY_AUTOPLAY=1, or MELEE_SCRIPT=builtin:lobby_autoplay, or in the console:
--   load builtin:lobby_autoplay
--
-- @name: Lobby autoplay
-- @version: 1.0.0

gd.run(function()
  while true do
    gd.wait(30) -- about half a second between moves, like a person
    local np = gd.netplay()
    if np.phase == "lobby" then
      local me = np.players[np.me + 1]
      local lp = np.lobby
      if lp == "char_blind" and not me.locked then
        gd.netplay_act("char")
      elseif (lp == "char_winner" or lp == "char_loser") and np.turn == np.me then
        gd.netplay_act("char")
      elseif (lp == "strike" or lp == "ban" or lp == "pick") and np.turn == np.me then
        for i, st in ipairs(np.stages) do
          if st == 0 then
            gd.netplay_act("stage", i)
            break
          end
        end
      elseif lp == "ready" and not me.ready then
        gd.netplay_act("ready", true)
      end
    end
  end
end)
