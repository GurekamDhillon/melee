-- Stock launch aliases from gw_runtime.c; counts from ft*_Init_CostumeStrings.
-- kind is internal FighterKind, distinct from scene CharacterKind.
local R={list={
 {id='captain',name='Captain Falcon',kind=2,costumes=6},
 {id='donkey',name='Donkey Kong',kind=3,costumes=5},
 {id='fox',name='Fox',kind=1,costumes=4},
 {id='gamewatch',name='Mr. Game & Watch',kind=24,costumes=4},
 {id='kirby',name='Kirby',kind=4,costumes=6},
 {id='bowser',name='Bowser',kind=5,costumes=4},
 {id='link',name='Link',kind=6,costumes=5},
 {id='luigi',name='Luigi',kind=17,costumes=4},
 {id='mario',name='Mario',kind=0,costumes=5},
 {id='marth',name='Marth',kind=18,costumes=5},
 {id='mewtwo',name='Mewtwo',kind=16,costumes=4},
 {id='ness',name='Ness',kind=8,costumes=4},
 {id='peach',name='Peach',kind=9,costumes=5},
 {id='pikachu',name='Pikachu',kind=12,costumes=4},
 {id='popo',name='Ice Climbers',kind=10,costumes=4},
 {id='jigglypuff',name='Jigglypuff',kind=15,costumes=5},
 {id='samus',name='Samus',kind=13,costumes=5},
 {id='yoshi',name='Yoshi',kind=14,costumes=6},
 {id='zelda',name='Zelda',kind=19,costumes=5},
 {id='sheik',name='Sheik',kind=7,costumes=5},
 {id='falco',name='Falco',kind=22,costumes=4},
 {id='younglink',name='Young Link',kind=20,costumes=5},
 {id='drmario',name='Dr. Mario',kind=21,costumes=5},
 {id='roy',name='Roy',kind=26,costumes=5},
 {id='pichu',name='Pichu',kind=23,costumes=4},
 {id='ganondorf',name='Ganondorf',kind=25,costumes=5},
}}
local by_id={};for _,entry in ipairs(R.list) do by_id[entry.id]=entry end
function R.validate(choice)
 if type(choice)~='table' or not by_id[choice.id] then return nil,'Unknown stock fighter' end
 local c=choice.costume;local e=by_id[choice.id]
 if type(c)~='number' or c%1~=0 or c<0 or c>=e.costumes then return nil,'Unsupported stock costume' end
 return {id=e.id,costume=c}
end
function R.scene(choice)
 local c,why=R.validate(choice);if not c then return nil,why end
 return c.id..'/c'..c.costume
end
function R.encode(choice) local scene,why=R.scene(choice);return scene and ('ROSTER1 '..scene..'\n') or nil,why end
function R.decode(text)
 if type(text)~='string' or #text>96 then return nil,'Invalid roster preference' end
 local id,c=text:match('^ROSTER1 ([a-z]+)/c(%d+)\n$')
 if not id then return nil,'Invalid roster preference' end
 return R.validate({id=id,costume=tonumber(c)})
end
function R.name(choice)
 local c=R.validate(choice);if not c then return 'Unavailable fighter' end
 return by_id[c.id].name..' / costume '..c.costume
end
function R.new(choice)
 local c=R.validate(choice) or {id='falco',costume=0}
 return {choice=c,focus='fighter:'..c.id}
end
local function add(v,id,x,y,w,h,label,action)
 local c={id=id,x=x,y=y,w=w,h=h,label=label,action=action};v.controls[#v.controls+1]=c;return c
end
local function layout()
 local area=gd and gd.safe_area and gd.safe_area() or nil
 local width=area and tonumber(area.w) or 640
 if not width or width<640 or width~=width then width=640 end
 return {x=area and tonumber(area.x) or 0,y=area and tonumber(area.y) or 0,sx=width/640,w=width}
end
function R.view(s,ctx)
 local v={controls={},choice=R.validate(s.choice) or {id='falco',costume=0}}
 for i,e in ipairs(R.list) do
  local col,row=(i-1)%4,(i-1)//4
  local c=add(v,'fighter:'..e.id,26+col*149,98+row*36,140,30,e.name,{kind='select',id=e.id})
  c.selected=e.id==v.choice.id
 end
 v.entry=by_id[v.choice.id];v.name=R.name(v.choice)
 v.note=v.choice.id=='popo' and 'Genes / tints target primary climber; partner is unbound.' or
  (v.choice.id=='zelda' or v.choice.id=='sheik') and 'Transform changes the model; each form needs its own binding.' or 'Costume order follows the native stock fighter tables.'
 if ctx and ctx.coverage then local _,label=ctx.coverage(v.choice);v.coverage=label end
 add(v,'costume:previous',26,362,143,30,'< COSTUME',{kind='costume',delta=-1})
 add(v,'costume:next',181,362,143,30,'COSTUME >',{kind='costume',delta=1})
 add(v,'accept',26,427,285,32,'USE FIGHTER / '..v.choice.costume,{kind='fighter_selected',fighter=v.choice})
 add(v,'back',324,427,289,32,'BACK TO COLLECTION',{kind='fighter_back'})
 local a=layout();v.canvas={x=a.x,y=a.y,w=a.w,h=480}
 for _,c in ipairs(v.controls) do c.x=a.x+c.x*a.sx;c.y=a.y+c.y;c.w=c.w*a.sx end
 return v
end
function R.update(s,ctx,input)
 if input.back then return {kind='fighter_back'} end
 local v=R.view(s,ctx);local focus=1
 for i,c in ipairs(v.controls) do if c.id==s.focus then focus=i end end
 local current=v.controls[focus];local dx=input.right and 1 or input.left and -1 or 0;local dy=input.down and 1 or input.up and -1 or 0
 if dx~=0 or dy~=0 then
  local x,y=current.x+current.w/2,current.y+current.h/2;local best,score
  for i,c in ipairs(v.controls) do if i~=focus then
   local a,b=c.x+c.w/2-x,c.y+c.h/2-y;local along=dx~=0 and a*dx or b*dy;local cross=dx~=0 and math.abs(b) or math.abs(a)
   local value=along+3*cross
   if along>5 and (not score or value<score) then best,score=i,value end
  end end
  if best then focus=best end
 end
 local activate=input.confirm
 if input.mx and input.my and input.click then for i,c in ipairs(v.controls) do
  if input.mx>=c.x and input.mx<c.x+c.w and input.my>=c.y and input.my<c.y+c.h then focus=i;activate=true end
 end end
 current=v.controls[focus];s.focus=current.id
 if not activate then return nil end
 local a=current.action
 if a.kind=='select' then s.choice={id=a.id,costume=0}
 elseif a.kind=='costume' then s.choice.costume=(s.choice.costume+a.delta)%by_id[s.choice.id].costumes
 else return a end
end
local function text(x,y,value,role,color,w)
 local a=layout()
 gd.kit.text(a.x+x*a.sx,a.y+y,value,role or 'caption',color or 'bone','left',{max_w=(w or 588)*a.sx,shear=0})
end
function R.draw(s,ctx)
 local v=R.view(s,ctx);local a=layout()
 gd.fill(a.x,a.y,a.w,480,0x030712ea);gd.kit.panel(a.x+14*a.sx,a.y+17,612*a.sx,450,{piece=12})
 gd.kit.icon('rogue_assault',a.x+27*a.sx,a.y+30,.65,'gold');text(61,52,'CHOOSE FIGHTER / COSTUME','label','gold',550)
 text(27,78,v.coverage or '26 STOCK SELECTIONS / GENES USE THE SAME RULES','caption','muted')
 gd.fill(a.x+26*a.sx,a.y+87,588*a.sx,1,0xf0b429ff)
 for _,c in ipairs(v.controls) do gd.kit.button(c.x,c.y,c.w,c.label,s.focus==c.id or c.selected,{h=c.h}) end
 text(338,382,v.name,'caption','gold',274)
 text(27,407,v.note,'caption','muted',586)
 text(27,478,'D-PAD: FOCUS   A: SELECT   B: BACK   /   MOUSE: CLICK','caption','muted',586)
end
return R
