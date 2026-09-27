-- The state hash is written by gw_Script_FramePost after this hook.
function on_frame()
  local match = gd.match()
  if match.active and match.frame >= 120 then gd.quit() end
end
