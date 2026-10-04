-- Unified item list: read this file top to bottom; offline LAB, vanilla Final Destination.
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
  local controls,notes=wrap("I: spawn common capsule (kind 0); inspect layer, name and owner"),wrap(detail or status)
  local lines=math.min(#controls,3)+math.min(#notes,3)
  gd.fill(a.x+12,a.y+12,width,48+lines*16,0x101827dd)
  gd.text(a.x+24, a.y+22, "Unified item list", 0xffd369ff, 1.2)
  local y=a.y+46
  for i,line in ipairs(controls) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
  y=y+8
  for i,line in ipairs(notes) do if i>3 then break end; gd.text(a.x+24,y,line); y=y+16 end
end

local owned={}
function on_tick()
  local p=gd.player(1)
  if p and pressed('I') then
    local h,why=gd.item_spawn(0,p.x+20,p.y+35,{})
    if h then owned[h]=true; status='Owned common-item handle '..h else status='Refused: '..tostring(why) end
  end
end
function on_draw()
  caption('Lists vanilla, m-ex, standalone and fighter articles; no m-ex rows on vanilla.')
  for i,item in ipairs(gd.items()) do
    if i>10 then break end
    gd.text(24,125+i*18,('%s %s kind=%d owner=%d'):format(item.layer or '-',item.name or '-',item.kind,item.owner_port))
  end
end
function on_item_expire(e) owned[e.item]=nil end
function on_item_collect(e) owned[e.item]=nil end
function on_unload() if gd.match().active then for h in pairs(owned) do gd.item_despawn(h) end end; owned={} end
function on_match_end() owned={} end

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
