-- A single-feature demonstration: native room/doorway membership.
local rooms = {
  {name='zones-left', label='Left room', kind='room', x0=-120,y0=-20,x1=-12,y1=90,color=0x66baffff},
  {name='zones-door', label='Doorway', kind='transition', x0=-12,y0=-20,x1=12,y1=90,color=0xffd369ff},
  {name='zones-right', label='Right room', kind='room', x0=12,y0=-20,x1=120,y1=90,color=0x88e0aaff},
}
local handles = {}
local function setup()
  handles={}
  for _,r in ipairs(rooms) do
    handles[#handles+1]=gd.zone_add{name=r.name,label=r.label,kind=r.kind,x0=r.x0,y0=r.y0,x1=r.x1,y1=r.y1}
  end
  gd.contact_overlay(true)
end
function on_match_start() if #handles==0 then setup() end end
function on_scene() handles={}; if gd.match().active then setup() end end
function on_match_end() handles={} end
function on_unload() gd.contact_overlay(false) end -- engine retires owned zones
local function log(event,e)
  gd.log(event, 'P'..e.port, 'entity '..e.entity, e.sub==1 and 'sub' or 'primary',e.label or '',e.x,e.y)
end
function on_zone_enter(e) log('zone enter',e) end
function on_zone_exit(e) log('zone exit',e) end
function on_zone_none(e) log('outside every zone',e) end
function on_zone_some(e) log('inside a zone',e) end
local function line(x0,y0,x1,y1,color)
  local ax,ay=gd.project(x0,y0)
  local bx,by=gd.project(x1,y1)
  if ax and bx then gd.line(ax,ay,bx,by,color) end
end
function on_draw()
  if not gd.match().active then return end
  for _,r in ipairs(rooms) do
    line(r.x0,r.y0,r.x1,r.y0,r.color);line(r.x1,r.y0,r.x1,r.y1,r.color)
    line(r.x1,r.y1,r.x0,r.y1,r.color);line(r.x0,r.y1,r.x0,r.y0,r.color)
  end
  local a=gd.safe_area()
  gd.text(a.x+12,a.y+12,'Zones: stop in the doorway, turn back, leave and return',0xffd369ff)
  local row=0
  for port=1,6 do
    if gd.player(port) then
      for sub=0,1 do
        local names={}
        for _,z in ipairs(gd.zones_at(port,sub)) do names[#names+1]=z.label..' ('..z.frames..'f)' end
        if sub==0 or #names>0 then
          row=row+1
          gd.text(a.x+12,a.y+20+row*18,'P'..port..(sub==1 and ' sub: ' or ': ')..(#names>0 and table.concat(names,', ') or 'outside every zone'),0xffffffff)
        end
      end
    end
  end
end
