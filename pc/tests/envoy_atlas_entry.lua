-- envoy_atlas_entry.lua: Envoy's main-menu entry (SOLO > ENVOY), offline.
-- Run from the game repo root (lua pc/tests/envoy_atlas_entry.lua) or from a root that holds melee/ and tools/.
-- The Atlas stand-in plays the engine's part (pc/tests/atlas_ui_stub.lua); the entry itself is the real code in retail_app.lua.
local prefix=io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/'
local T=dofile(prefix..'pc/tests/envoy_testlib.lua')
local Stub=dofile(prefix..'pc/tests/atlas_ui_stub.lua')
local D=T.rules();for _,k in ipairs({'save','drives','drive_models','fighter','campaign','hub','run','classic','hud','menu','menu_draw','menu_input','recolour','visual'}) do D[k]=T.module(k,D) end
D.retail_app=T.module('retail_app',D);D.app=T.module('app',D)

local function slurp(path) local f=assert(io.open(path,'rb'));local s=f:read('a');f:close();return s end

-- the manifest declares the entry the host reads (at_menus_parse reads the same file in atlas-registry)
T.test('mod.json declares SOLO > ENVOY',function()
 local m=slurp(prefix..'pc/scripts/examples/envoy/mod.json')
 assert(m:find('"menus"',1,true),'menus');assert(m:find('"parent": "solo"',1,true),'under solo')
 assert(m:find('"id": "envoy"',1,true),'id');assert(m:find('"label": "ENVOY"',1,true),'label')
 assert(m:find('"action": "script"',1,true),'a script action');assert(m:find('"online": false',1,true),'offline only')
end)

-- the hook is generated into the bundle's entry (tools/port/envoy_bundle.py); the bundle itself is checked in
T.test('the bundle defines on_entry',function()
 assert(slurp(prefix..'pc/scripts/examples/envoy/scripts/main.lua'):find('function on_entry(id)',1,true),'main.lua has no on_entry: rerun tools/port/envoy_bundle.py')
 local f=io.open('tools/port/envoy_bundle.py') or io.open('../tools/port/envoy_bundle.py')
 if f then local s=f:read('a');f:close();assert(s:find('function on_entry(id)',1,true),'the generator does not write on_entry') end
end)

local function fixture(opts)
 opts=opts or {}
 local s={text=D.save.encode(D.save.new_profile()),pad={},commands={},writes=0,mods={},logs={},online=false,
  mode={mode='classic',stage_index=0,loop=0,player_port=1,opponents={{port=2}}}}
 local ui=Stub.new{caller='envoy/main',owner_mod='envoy',available=opts.available}
 local g={command=function(n,f) s.commands[n]=f end,log=function(t) s.logs[#s.logs+1]=t end,data_read=function() return s.text end,
  data_write_atomic=function(_,v) s.writes=s.writes+1;s.text=v;return true end,
  match=function() return {active=false,netplay=s.online} end,player=function() return {} end,
  pad=function() return s.pad end,input_mask=function() end,paused=function() return false end,pause=function() end,resume=function() end,
  fx_world=function() return 1 end,fx_move=function() end,fx_control=function() end,fx_end=function() end,
  mode_1p=function() return s.mode end,start_1p=function(v) s.launch=v;return true end,spawn_1p=function() return true end,loop_1p=function() return true end,
  end_1p=function() return true end,hold_1p=function() return true end,release_1p=function() return true end,ui=ui}
 local mission={frame=function() end,draw=function() end,stop=function(m) m.current=nil end}
 s.a=D.app.new(g,mission);s.ui=ui;s.g=g;return s,s.a,ui
end

T.test('on_entry pushes the entry screen: four rows, B closes, labels fit',function()
 local s,a,ui=fixture()
 local r=a:entry('envoy')
 assert(type(r)=='table' and r.push=='envoy.entry','pushes envoy.entry')
 local d=ui.screens['envoy.entry'];assert(d,'envoy.entry is registered')
 assert(d.primary.kind=='list' and #d.primary.items==4,'four rows')
 assert(d.on and type(d.on.back)=='function','B closes (on.back given)')
 for _,it in ipairs(d.primary.items) do assert(#it.label<=18,'label fits: '..it.label) end
 assert(d.trail and d.trail[1]=='SOLO' and d.trail.title=='ENVOY','the trail reads SOLO > ENVOY')
 -- the engine's part: the pushed screen opens as the mod's
 assert(ui.open('envoy.entry') and ui.stack[#ui.stack]=='envoy.entry')
 local back=d.on.back();assert(type(back)=='table' and back.pop==true,'B pops')
end)

T.test('another id is not Envoy\'s; netplay and a missing Atlas give nothing',function()
 local s,a=fixture()
 assert(a:entry('other')==nil,'another id')
 s.online=true;assert(a:entry('envoy')==nil,'nothing in netplay');s.online=false
 local s2,a2=fixture{available=false};assert(a2:entry('envoy')==nil,'no Atlas roles: the entry is refused, not half-drawn')
 assert(table.concat(s2.logs,'\n'):find('Atlas',1,true),'and says why')
end)

T.test('START CLASSIC and START ADVENTURE start a retail run; ENVOY MENU opens the legacy menu; BACK pops',function()
 local s,a,ui=fixture();a:entry('envoy');local d=ui.screens['envoy.entry']
 local function row(id) for _,it in ipairs(d.primary.items) do if it.id==id then return it end end end
 assert(row('classic') and row('adventure') and row('menu') and row('back'))
 local r=d.on.accept('adventure','tiles')
 assert(a.menu.run_type=='adventure','the mode is set');assert(a.retail.active or a.retail_request or s.launch,'a retail start was asked for')
 assert(type(r)=='table' and r.pop==true,'the screen pops after a start')
 local s2,a2,ui2=fixture();a2:entry('envoy');local d2=ui2.screens['envoy.entry']
 local held
 ui2.hold_menu=function(on) held=on end
 local r2=d2.on.accept('menu','tiles')
 assert(a2.visible==true and a2.menu.screen=='title','the legacy Envoy menu is up');assert(r2 and r2.pop==true,'the entry screen pops first')
 assert(held==true,'the frontend is held while the legacy menu is up')
 a2.visible=false;a2:tick();assert(held==false,'and released when it closes')
 local r3=d2.on.accept('back','tiles');assert(r3 and r3.pop==true)
end)

T.test('the explainer says one rule per row',function()
 local _,a,ui=fixture();a:entry('envoy');local d=ui.screens['envoy.entry']
 assert(type(d.explainer)=='table' and type(d.explainer.provide)=='function')
 for _,it in ipairs(d.primary.items) do
  local e=d.explainer.provide(it.id,'list');assert(type(e)=='table' and e.title and #e.what>0 and #e.what<=159,'explainer for '..it.id)
 end
end)

T.test('the engine plays its part: a script entry runs on_entry and pushes the mod screen',function()
 local _,a,ui=fixture()
 ui.hooks.on_entry=function(id) return a:entry(id) end
 ui.register_entry{id='envoy',parent='solo',label='ENVOY',action='script',online=false}
 assert(ui.engine_activate('envoy') and ui.stack[#ui.stack]=='envoy.entry')
end)
T.done()
