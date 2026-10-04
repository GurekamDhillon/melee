-- Custom material, glass and light: read this file top to bottom; offline LAB, vanilla Final Destination.
-- Host keys live in on_tick; simulation observations live in on_frame.
local status = "Ready"
local requested = {}
local function pressed(key)
  if requested[key] then requested[key]=nil; return true end
  return gd.key_pressed(key)
end
gd.command('demo_key',function(key) requested[key]=true end,'demo_key <uppercase key>: demo-owned tour control')
local function caption(detail)
  local a = gd.safe_area()
  local width=math.min(a.w-24,590)
  local function wrap(value)
    local rows,line={},''; local limit=math.max(12,math.floor((width-24)/7))
    for word in tostring(value):gmatch('%S+') do
      if #line+#word+1>limit and #line>0 then rows[#rows+1]=line; line='' end
      while #word>limit do rows[#rows+1]=word:sub(1,limit); word=word:sub(limit+1) end
      line=line=='' and word or line..' '..word
    end
    if line~='' then rows[#rows+1]=line end
    return rows
  end
  local controls,notes=wrap("L: warm/cool light; optional lit/glass/custom meshes; fallback slabs"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Custom material, glass and light", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local assets,instances,slabs={},{},{}
local warm=false
function on_match_start()
  assets={}; instances={}; slabs={}
  for i,name in ipairs{'lit','glass','custom'} do
    slabs[#slabs+1]=assert(gd.stage_add_platform((i-1)*65-32,30,50,{passthrough=true}))
    local ok,a=pcall(gd.model_load,name)
    if ok and a then
      assets[#assets+1]=a
      instances[#instances+1]=assert(gd.model_spawn(a,{x=(i-1)*65-32,y=30,collision=false}))
    end
  end
  status=#instances==0 and 'Install your own meshes beside the supplied material templates' or 'Opt-in model materials loaded'
end
function on_tick()
  if gd.match().active and pressed('L') then
    warm=not warm
    gd.light_set{dir={0.25,1,0.55},color=warm and {1,0.6,0.3} or {0.3,0.7,1},ambient={0.3,0.3,0.3}}
  end
end
function on_draw() caption(status..'; key light '..(warm and 'warm' or 'cool')) end
function on_unload()
  for _,h in ipairs(instances) do gd.model_despawn(h) end
  for _,a in ipairs(assets) do gd.model_release(a) end
  if gd.match().active then for _,h in ipairs(slabs) do gd.stage_remove(h) end end
  -- Owner-scoped light resets automatically when the script unloads.
end
function on_match_end() assets={}; instances={}; slabs={} end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(tostring(warm))..' | '..status)
end,'demo_state <token>: report this demo state')

-- Catch up when loaded into an existing match; never initialize twice.
local begin, frame, finish = on_match_start, on_frame, on_match_end
local started = false
function on_match_start()
  if not started then started=true; if begin then begin() end end
end
function on_frame()
  if gd.match().active then on_match_start(); if frame then frame() end end
end
function on_match_end()
  if finish then finish() end
  started=false
end
