-- Every item name an Envoy drive drop can ask the engine for must be one the mod defines (items/<name>/item.json, "name" = the
-- folder): an unknown name makes gd.item_spawn answer "unknown item name" and the drop is lost ("drop failed for Blue Drive: ...").
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();D.drive_drop=T.module('drive_drop',D)
local items=T.root:gsub('scripts/$','items/')
local function defined(name)
  local f=io.open(items..name..'/item.json','rb');if not f then return false end
  local text=f:read('a');f:close();return text:match('"name"%s*:%s*"([^"]+)"')==name
end
local function every_name()
  local seen,list={},{}
  for _,item_name in ipairs({false,'drive','drive_coop'}) do for _,press in ipairs({false,true}) do for _,payout in ipairs({false,true}) do
    local drops=D.drive_drop.new({});drops.item_name=item_name or nil;drops.press=press or nil
    local n=drops:item_for(payout and {payout=true} or nil)
    if not seen[n] then seen[n]=true;list[#list+1]=n end
  end end end
  return list
end
T.test('every name a drive drop asks for is a defined item',function()
  local names=every_name();assert(#names==6,'expected the six drive items, got '..#names)
  for _,n in ipairs(names) do assert(defined(n),'no items/'..n..'/item.json defining "'..n..'"') end
end)
T.test('the spawn call passes exactly those names',function()
  local asked={}
  local g={player=function() return {x=0,y=0} end,item_spawn=function(name) asked[#asked+1]=name;return 1 end,log=function() end}
  local drops=D.drive_drop.new(g)
  local ok=pcall(drops.spawn,drops,{colour='blue',affixes={},rarity='common'},0,0)
  assert(asked[1]=='drive' or not ok,'the plain drop asks for "drive", asked '..tostring(asked[1]))
  drops.press=true;drops.next_id=1;drops.records={}
  pcall(drops.spawn,drops,{colour='blue',affixes={},rarity='common'},0,0,{payout=true})
  assert(asked[#asked]=='drive_payout' and defined(asked[#asked]))
end)
T.done()
