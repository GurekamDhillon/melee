-- Gameplay descriptions contain scalar delays; presentation consumes the same ages.
return function(D)
 local X={moves={any=true,aerial=true,nair=true,fair=true,bair=true,uair=true,dair=true,unknown=true,grab=true,jab=true,tilt=true,smash=true,special=true,dash_attack=true,throw=true}}
 local function integer(n,lo,hi)assert(type(n)=='number' and n%1==0 and n>=lo and n<=hi,'bounded echo integer required');return n end
 function X.description(e)
  local count=integer(e.copies or 3,1,3);local delay=integer(e.delay or 8,1,20);local armed={}
  for _,i in ipairs(e.slots or {})do integer(i,1,count);assert(not armed[i],'duplicate echo copy');armed[i]=true end
  local out={copies={},visual={}}
  for i=1,count do local c={age=i*delay};out.copies[i]=c
   if armed[i]then
    assert(type(e.damage)=='number' and e.damage==e.damage and e.damage>=.1 and e.damage<=4,'echo damage scale out of bounds')
    assert(type(e.knockback or 1)=='number' and (e.knockback or 1)>=.1 and (e.knockback or 1)<=4,'echo knockback scale out of bounds')
    local match=e.match or {move='any'};assert(X.moves[match.move or 'any'],'unknown echo move')
    c.echo={delay=c.age,match=D.mod_codec and D.mod_codec.decode(D.mod_codec.encode(match)) or {move=match.move},damage=e.damage,knockback=e.knockback or 1,once_per_move=e.once_per_move~=false,element=e.element}
   end
  end;return out
 end
 function X.resolve(e,m,tier)
  local S=D.mod_schema;local n=math.min(3,math.max(1,math.floor(S.resolve(e.copies or 3,m,tier))))
  local slots={};for _,i in ipairs(e.slots or {1,2,3})do if i<=n then slots[#slots+1]=i end end
  return X.description{copies=n,slots=slots,delay=S.resolve(e.delay or 8,m,tier),match=e.match,damage=math.min(4,math.max(.1,S.resolve(e.damage or .4,m,tier))),knockback=math.min(4,math.max(.1,S.resolve(e.knockback or 1,m,tier))),once_per_move=e.once_per_move,element=e.element}
 end
 function X.engine(engine,p)
  local out={copies={},rules={},visual={}};local by_age,by_rule={},{}
  for _,m in ipairs(engine.list)do local tier=(engine.equipped[p] or {})[m.id]
   if tier then for _,e in ipairs(m.effects)do if e.op=='echo' and (not e.status or engine:status(p,e.status))then
    for _,instance in ipairs(D.mod_schema.instances(tier))do local d=X.resolve(e,m,instance)
     for _,c in ipairs(d.copies)do
      if not by_age[c.age]then by_age[c.age]={age=c.age};out.copies[#out.copies+1]=by_age[c.age]end
      if c.echo then
       local key=tostring(c.age)..D.mod_codec.encode(c.echo.match)..tostring(c.echo.element)..tostring(c.echo.once_per_move);local old=by_rule[key]
       if old then old.damage=old.damage+c.echo.damage;old.knockback=math.max(old.knockback,c.echo.knockback)
       else by_rule[key]=c.echo;out.rules[#out.rules+1]=c.echo end
       by_age[c.age].echo=by_age[c.age].echo or by_rule[key]
      end
     end
    end
   end end end
  end
  for _,r in ipairs(out.rules)do r.damage=math.min(4,r.damage) end
  table.sort(out.copies,function(a,b)return a.age<b.age end);table.sort(out.rules,function(a,b)if a.delay~=b.delay then return a.delay<b.delay end;return D.mod_codec.encode(a.match)<D.mod_codec.encode(b.match)end)
  assert(#out.copies<=3 and #out.rules<=8,'echo capacity exceeded');return out
 end
 function X.records()
  local function r(id,label,kind,copies,damage,cost,live)
   local effects={{op='echo',copies=copies,delay=8,damage=damage,knockback=1,match={move=kind=='normal' and 'aerial' or 'any'},once_per_move=true}}
   if live then effects[#effects+1]={op='value',key='damage_dealt',value=live}end
   return {id=id,label=label,kind=kind,cost=cost,tags={'aerial','damage','momentum'},trigger='equip',conditions={},effects=effects,tiers={{copies=1},{copies=2},{copies=3}},stacking={max=1},text='Historical '..(kind=='normal' and 'aerial' or 'all move')..' hitboxes repeat at '..tostring(damage*100)..'% damage; higher tiers arm more copies.',visual={look='momentum',hue=.72,strength=.45,priority=20},families=live and {'echo','damage_dealt'} or {'echo'},affix=kind=='normal' and 'suffix' or nil,group=kind=='normal' and id or nil,weight=kind=='normal' and 100 or nil,fixed_colour=kind=='unique' and 'purple' or nil}
  end
  local trail=r('trailing','Trailing','normal',3,.4);trail.affix='prefix';trail.tags={'hasted','momentum'};trail.effects[1].slots={};trail.effects[1].status='haste';trail.text='While Hasted, leave three historical afterimages.'
  return {trail,r('echoes','of Echoes','normal','$copies',.4),r('echo_heart','Echo Heart','unique',3,.4,'All attack damage, including echoes, is 25% lower.',.75),r('echo_oath','Echo Oath','keystone',3,.6,'All attack damage, including echoes, is 50% lower.',.5)}
 end
 return X
end
