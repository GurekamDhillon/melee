-- Training card: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("LEFT/RIGHT: move; M: inspect chosen move; D: landing drill; SPACE: resume"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Training card", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local moves,selected,watcher,platform,cursor,trace={ },1,nil,nil,0,{}
local old_debug,was_paused
function on_match_start()
  -- Only intentional player attacks; omit death, entry, capture, damage and respawn states.
  moves={}
  for _,m in ipairs(gd.motion_list(1) or {}) do
    if m.name:match('^Attack') or m.name:match('^Special') then moves[#moves+1]=m end
  end
  selected=1
  for i,m in ipairs(moves) do if m.name=='Attack11' then selected=i; break end end -- the list opens with death states
  watcher=nil; cursor=0; trace={}
  old_debug=gd.debug_draw(1); was_paused=gd.paused()
  platform=assert(gd.stage_add_platform(0,35,55,{passthrough=true}))
  gd.contact_trace{file='training-card.jsonl',state_changes=true}
end
function on_tick()
  if not gd.match().active then return end
  if pressed('RIGHT') and #moves>0 then selected=selected%#moves+1 end
  if pressed('LEFT') and #moves>0 then selected=(selected-2)%#moves+1 end
  if pressed('M') and moves[selected] then
    local ok,why=gd.set_motion(1,moves[selected].id,1,1,0)
    status=ok and 'Motion queued; N in console: step 1; SPACE resumes' or tostring(why)
  end
  if pressed('D') and platform then
    gd.resume()
    watcher=gd.wait_until{port=1,on=platform,airborne=false,timeout=180}
    status='Drill: land on cyan platform within 180 logic frames'
  end
  if pressed('SPACE') then gd.resume() end
end
function on_frame()
  local events; events,cursor=gd.contact_events(cursor)
  for _,e in ipairs(events) do
    if e.port==1 then trace[#trace+1]=('%d %s %s'):format(e.frame,e.event,e.label or '-') end
  end
  while #trace>5 do table.remove(trace,1) end
  if watcher then
    local s=gd.wait_status(watcher)
    if not s then watcher=nil; status='Drill cancelled'
    elseif s.done then status=(s.ok and 'PASS: ' or 'FAIL: ')..s.reason; watcher=nil end
  end
end
function on_draw()
  caption(status)
  local p=gd.player(1); if not p then return end
  local t=gd.timeline(1,moves[selected] and moves[selected].id)
  local hitboxes=gd.hitboxes(1) or {}; local y=140
  gd.text(24,y,'Selected '..(moves[selected] and moves[selected].name or '-')..'; current '..gd.motion_name(p.action,1)); y=y+20
  gd.text(24,y,('Action frame %.1f; anim %.1f; timeline length %s; hitboxes %d'):format(p.action_frame,p.anim_frame,t and tostring(t.length) or '-',#hitboxes)); y=y+20
  for i,h in ipairs(hitboxes) do
    if i>5 then break end
    gd.text(24,y,('HB %s damage %s angle %s radius %s'):format(tostring(h.id),tostring(h.damage),tostring(h.angle),tostring(h.radius))); y=y+18
  end
  if t then for i,e in ipairs(t.events or {}) do
    if i>4 then break end
    gd.text(24,y,('Timeline %s %s'):format(tostring(e.frame),tostring(e.name or e.op or e.type))); y=y+18
  end end
  for _,line in ipairs(trace) do gd.text(24,y,line); y=y+18 end
end
function on_loadstate() watcher=nil; trace={}; cursor=0; status='State loaded: drill cancelled' end
function on_unload()
  gd.contact_trace(false)
  if gd.match().active then
    if platform then gd.stage_remove(platform) end
    if old_debug then gd.debug_draw(1,old_debug) end
    if was_paused then gd.pause() else gd.resume() end
  end
end
function on_match_end() platform=nil; watcher=nil; trace={}; cursor=0 end

-- Owner-local state for the external behavior tour; never exposes another mod's handles.
gd.command('demo_state',function(token)
  gd.log('tour state '..token..' '..tostring((moves[selected] and moves[selected].name or '-'))..' | '..status)
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
