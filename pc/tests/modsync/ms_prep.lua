-- ms_prep.lua - TEST DRIVER: installs one Nucleus post (MELEE_LAB_POST) from the fixture server into the mods folder, then idles. np_modsync.py kills it once the folder exists.
-- @name: Mod sync test - prep
-- @version: 1.0.0
-- @gameplay: false
gd.run(function()
  gd.wait_until(function() return gd.scene().name == "GS_FRONTEND" end, 3600)
  gd.wait(60)
  local post = tonumber(gd.lab_env("POST")) or 0
  if post > 0 then gd.log("MS_PREP install " .. tostring(gd.modsync_act("install", post))) else gd.log("MS_PREP boot only") end
  while true do gd.wait(60) end
end)
