-- Native effects and emitter controls: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("E: play square sparks; C: slow/brighten; R: fade out; F: attach package"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Native effects and emitter controls", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local handle,shader
function on_tick()
  if not gd.match().active then return end
  if pressed('E') then
    if handle then gd.fx_end(handle,0) end
    handle=gd.fx_play('DemoSquares',1,0,0,12,0,1,42)
    status=handle>0 and 'Original texture-free square particles' or 'Mount this mod at boot to register the package'
    if handle<=0 then handle=nil end
  end
  if handle and pressed('C') then
    gd.fx_control(handle,-1,{speed=0.3,brightness=2},12)
    shader=assert(gd.shader_load('shaders/effect.wgsl',{kind='effect',params={tint={0.4,0.8,1,1}}}))
    local ok,why=gd.fx_shader('DemoSquares','squares',shader)
    status=ok and 'All emitters slowed; custom effect shader active' or tostring(why)
  end
  if handle and pressed('R') then gd.fx_end(handle,15); handle=nil end
  if pressed('F') then gd.fx_attach('DemoSquares',1,0,0,12,0,1) end
end
gd.command('demo_fx_play',function()
  if gd.match().active then handle=gd.fx_play('DemoSquares',1,0,0,12,0,1,42); if handle<=0 then handle=nil end end
end,'Play the original demo particle package')
function on_draw()
  local particles=0; for _,f in ipairs(gd.fx()) do particles=particles+f.live_particles end
  if handle then local snapshot=gd.fx_instance(handle); if not snapshot then handle=nil end end
  caption(status..'; live particles '..particles)
end
function on_unload()
  if handle and gd.match().active then gd.fx_end(handle,0) end
  if shader then gd.fx_shader('DemoSquares','squares',nil) end
  gd.fx_stop()
end
function on_match_end() handle=nil; shader=nil end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring(status)..' | '..status)
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
