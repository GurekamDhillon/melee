-- Stage tour: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("N: next stage; cycles FD/BF/YS every 8s; wipe/flash/morph"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Stage tour", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local slots,armed,queued,current,pending_look={},false,false,1,nil
local looks={{1,0.8,0.6},{0.5,0.85,1},{0.6,1,0.7}}
local names={'Final Destination','Battlefield','Yoshis Story'}
local function look(i)
  local c=looks[i]
  local ok,why=gd.stage_shader('shaders/tour.wgsl',{params=c})
  -- Temporary engine-hang workaround: use only the surface look. No post churn.
  status=ok and (names[i]..' static topology; no destination hazards') or tostring(why)
end
function on_match_start()
  slots={}; armed=false; queued=false; current=1; pending_look=nil
  for _,name in ipairs{'fd','bf','ys'} do
    local slot,why=gd.stage_slot_load(name)
    if not slot then status='Preload refused: '..tostring(why); gd.log(status); return end
    slots[#slots+1]=slot
  end
  local ok,why=gd.stage_switch(slots[1],{transition='wipe',place='keep'})
  if not ok then status='Initial switch refused: '..tostring(why); gd.log(status) end
end
function on_stage_switch(e)
  if e.phase~='after' then return end
  for i,s in ipairs(slots) do if e.slot==s then current=i; pending_look=i end end
  if not queued then armed=true end
end
function on_frame()
  -- Apply after event dispatch, never while the engine owns its live transition cover.
  if pending_look then local i=pending_look; pending_look=nil; look(i) end
  -- The callback cannot recursively request another switch; arm the next frame.
  if armed then
    armed=false
    local ok,why=gd.stage_queue{
      {slot=slots[2],after=8,transition='wipe',indicator=1},
      {slot=slots[3],after=8,transition='flash',indicator=1},
      {slot=slots[1],after=8,transition='morph',indicator=1},loop=true,seed=42}
    queued=ok and true or false
    if not ok then status='Queue refused: '..tostring(why); gd.log(status) end
  end
end
function on_tick()
  if queued and pressed('N') then
    local ok,why=gd.stage_queue_next(); if not ok then status=tostring(why) end
  end
end
function on_draw() caption(status..'; stop/unload restores original host') end
function on_unload()
  if gd.match().active then gd.stage_queue_clear(); gd.stage_shader(nil) end
  -- Active slots cannot be freed by hand; owner unload reclaims all slots/restores host.
end
function on_match_end() slots={}; queued=false; armed=false; pending_look=nil end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring('slot='..current..' queued='..tostring(queued))..' | '..status)
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
