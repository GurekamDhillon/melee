local slots = {}
local arm,queued=false,false
local function queue()
  if #slots ~= 3 then return end
  local ok,why=gd.stage_queue{
    {slot=slots[2],after=10,indicator=1,transition="wipe"},
    {slot=slots[3],after=10,indicator=1,transition="flash"},
    {slot=slots[1],after=10,indicator=1,transition="morph"},
    loop=true,shuffle=false,seed=42,
  }
  if not ok then gd.log(tostring(why)) end
end
function on_match_start()
  slots={}
  arm,queued=false,false
  for _,name in ipairs{"final_destination","battlefield","yoshis_story"} do
    local slot,why=gd.stage_slot_load(name)
    if not slot then gd.log("stage preload: "..tostring(why));return end
    slots[#slots+1]=slot
  end
  local ok,why=gd.stage_switch(slots[1],{transition="wipe"})
  if not ok then gd.log(tostring(why)) end
end
function on_stage_switch(e)
  if e.phase=="after" and e.slot==slots[1] and not queued then arm=true end
end
function on_frame()
  if arm then arm=false;queued=true;queue() end
end
gd.command("stage",function(arg)
  if arg=="next" then local ok,why=gd.stage_queue_next();if not ok then gd.log(tostring(why)) end
  elseif arg=="queue" then queue()
  elseif arg=="slots" then
    for _,s in ipairs(gd.stage_slots()) do gd.log(string.format("slot %d: %d bytes, %d lines",s.slot,s.bytes,s.lines)) end
  else gd.log("stage next | stage queue | stage slots") end
end,"Static stage switch demo controls")
