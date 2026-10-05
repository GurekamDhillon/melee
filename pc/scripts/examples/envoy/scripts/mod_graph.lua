-- The derived interaction graph: who feeds whom, computed ONCE per pool from the records (trigger, conditions, effects), never typed.
-- It is the single source for: the strength formula's "what does the rest of the build feed this record" (mod_budget), connecting offers
-- (run_host), the grid's link display and the archetype banners (synergy_fx), and the chain visuals.
-- Vocabulary of a key:  status:X:self | status:X:target | status:X (a status_applied trigger) | status:any | elem:E | event:crit | event:armor | critchance
--   gives(record) = the keys its effects produce; reads(record) = the keys its trigger, conditions or effects need.
-- Edge kinds: feed (A gives what B reads), enable (A gives a crit chance a crit multiplier needs).
-- Archetypes are named sets of ROLES over those records; their members are checked against the pool when the graph is built.
return function(D)
 local G={}
 -- ---- profiles ------------------------------------------------------------------------------------------------------------
 -- sources: alternatives (the main trigger and each `also` trigger), each a list of keys; effect_reads[i]: keys effect i needs.
 local cache=setmetatable({},{__mode='k'})
 local function source_reads(trigger,conditions)
  local r={}
  if trigger=='status_applied' then for _,c in ipairs(conditions or {}) do if c.status then r[#r+1]='status:'..c.status end end end
  if trigger=='crit' then r[#r+1]='event:crit' end
  if trigger=='armor' then r[#r+1]='event:armor' end
  for _,c in ipairs(conditions or {}) do
   if c.tag then r[#r+1]='ev:'..trigger..':'..c.tag end   -- a tag an `emit` effect of another record can supply
   if c.tag=='fire' or c.tag=='ice' or c.tag=='electric' then r[#r+1]='elem:'..c.tag end
   if c.self_status then r[#r+1]=c.self_status=='any' and 'status:any' or 'status:'..c.self_status..':self' end
   if c.target_status then r[#r+1]=c.target_status=='any' and 'status:any' or 'status:'..c.target_status..':target' end
  end
  return r
 end
 function G.profile(m)
  local p=cache[m];if p and p.cref==m.conditions and p.eref==m.effects and p.tref==m.trigger and p.aref==m.also then return p end
  p={id=m.id,gives={},sources={},effect_reads={},trigger=m.trigger,cref=m.conditions,eref=m.effects,tref=m.trigger,aref=m.also}
  p.sources[1]=source_reads(m.trigger,m.conditions)
  for _,a in ipairs(m.also or {}) do p.sources[#p.sources+1]=source_reads(a.trigger,a.conditions) end
  local function give(k) p.gives[k]=true end
  for i,e in ipairs(m.effects) do
   local er={}
   if e.op=='versus-status' and e.status then er[#er+1]='status:'..e.status..':target' end
   if e.op=='echo' and e.status then er[#er+1]='status:'..e.status..':self' end
   if e.op=='crit' and not e.chance then er[#er+1]='critchance' end
   if e.op=='crit' and e.status then er[#er+1]='status:'..e.status..':self' end
   p.effect_reads[i]=er
   if e.op=='status' or e.op=='stacks' or e.op=='chain_status' then
    local sj=e.op=='chain_status' and 'target' or (e.subject or 'self')
    give('status:'..e.status..':'..sj);give('status:'..e.status);if sj=='target' then give('status:any') end
   elseif e.op=='convert' and e.change.element then give('elem:'..e.change.element)
   elseif (e.op=='crit' and e.chance) or e.op=='crit_next' then give('event:crit');give('critchance')
   elseif e.op=='armor' then give('event:armor')
   elseif e.op=='emit' then give('event:'..e.event);give('ev:'..e.event..':'..e.tag) end
  end
  p.reads={};local seen={}
  for _,s in ipairs(p.sources) do for _,k in ipairs(s) do if not seen[k] then seen[k]=true;p.reads[#p.reads+1]=k end end end
  for _,er in ipairs(p.effect_reads) do for _,k in ipairs(er) do if not seen[k] then seen[k]=true;p.reads[#p.reads+1]=k end end end
  cache[m]=p;return p
 end
 -- How much of what effect `i` of the record needs does `provided` (a set of keys held by the OTHER records) give: 0..1, or nil when it needs nothing.
 -- A triggered record needs its trigger side (the best of its alternative triggers) AND the effect's own reads.
 function G.supply(m,i,provided,triggered)
  local p=G.profile(m);local need,have=0,0
  if triggered then
   local best
   for _,s in ipairs(p.sources) do
    if #s==0 then best=1;break end
    local ok=0;for _,k in ipairs(s) do if provided[k] then ok=ok+1 end end
    local f=ok/#s;if not best or f>best then best=f end
   end
   if #p.sources[1]>0 or #(m.also or {})>0 then need=need+1;have=have+(best or 0) end
  end
  local er=p.effect_reads[i]
  if er and #er>0 then local ok=0;for _,k in ipairs(er) do if provided[k] then ok=ok+1 end end;need=need+1;have=have+ok/#er end
  if need==0 then return nil end
  return have/need
 end
 -- ---- the graph -------------------------------------------------------------------------------------------------------------
 local graphs=setmetatable({},{__mode='k'})
 function G.graph(pool)
  local g=graphs[pool];if g and g.n==#pool then return g end
  g={n=#pool,out={},into={},partners={},list={},prof={}}
  for _,m in ipairs(pool) do g.prof[m.id]=G.profile(m);g.out[m.id]={};g.into[m.id]={};g.partners[m.id]={} end
  for _,a in ipairs(pool) do local pa=g.prof[a.id]
   for _,b in ipairs(pool) do if a~=b then local pb=g.prof[b.id]
    for _,k in ipairs(pb.reads) do
     if pa.gives[k] then
      if not g.out[a.id][b.id] then
       local e={from=a.id,to=b.id,kind=k=='critchance' and 'enable' or 'feed',why=k}
       g.out[a.id][b.id]=e;g.into[b.id][a.id]=e;g.list[#g.list+1]=e;g.partners[a.id][b.id]=true;g.partners[b.id][a.id]=true
      end
      break
     end
    end
   end end
  end
  graphs[pool]=g;return g
 end
 function G.connected(pool,a,b) local g=G.graph(pool);return (g.out[a] and g.out[a][b]) or (g.out[b] and g.out[b][a]) or nil end
 -- Does any of `ids` connect to any of the held set `held` (id -> truthy)? Returns true, offered id, held id.
 function G.connects(pool,ids,held)
  local g=G.graph(pool)
  for _,id in ipairs(ids) do local ps=g.partners[id]
   if ps then for h in pairs(held) do if h~=id and ps[h] then return true,id,h end end end
  end
  return false
 end
 -- ---- archetypes ------------------------------------------------------------------------------------------------------------
 -- roles: {name, alts}; alts = alternatives, each a list of ids that must ALL be held. colour 0xRRGGBB; motif names the emblem and the surface treatment.
 G.archetypes={
  {id='burn',name='Burn stacking',colour=0xFF4A2A,motif='flame',blurb='Your hits set Burn and your payoff drives turn it into damage.',
   roles={{name='a Burn applier',alts={{'plague_bearer'},{'kindling','burning'},{'kindling','pyromancer'},{'kindling','ember_crown'}}},{name='a payoff',alts={{'pyre'},{'cinder'},{'everburn'}}}}},
  {id='chill',name='Chill stacking',colour=0x7FD6FF,motif='frost',blurb='Your hits Chill and your payoff drives turn it into damage.',
   roles={{name='a Chill applier',alts={{'deep_freeze'},{'icebound','frosted'},{'icebound','frozen_oath'},{'icebound','winter_heart'}}},{name='a payoff',alts={{'brittle'},{'shatter'}}}}},
  {id='shock',name='Shock chain',colour=0x21E6E6,motif='arc',blurb='Electric hits Shock one opponent and chain it to the next.',
   roles={{name='an electric source',alts={{'charged'},{'storm_shell'}}},{name='Conductor',alts={{'conductor'}}}}},
  {id='momentum',name='Momentum speed',colour=0xFFB11F,motif='chevron',blurb='Hits build Momentum and a landing spends it.',
   roles={{name='a Momentum source',alts={{'updraft'},{'hit_and_run'},{'banked_momentum'},{'hang_time'},{'reprisal'},{'critical_mass'},{'fury'}}},{name='a landing payoff',alts={{'crosswind'},{'bastion'}}}}},
  {id='haste',name='Haste web',colour=0x5CF04C,motif='web',blurb='Haste feeds your Haste payoffs, and they feed Haste again.',
   roles={{name='a Haste source',alts={{'ledge'},{'clean_landing'},{'combo_surge'},{'skyfarer'},{'perpetual_motion'},{'bloodlust'},{'hit_and_run'},{'critical_flow'},{'crosswind'}}},{name='a payoff',alts={{'rush'},{'echo_weaver'},{'trailing'}}}}},
  {id='guard',name='Guard and heal',colour=0x4C7BFF,motif='plate',blurb='Guarded turns into healing.',
   roles={{name='a Guarded source',alts={{'reprisal'},{'bastion'},{'shelter'},{'tech_guard'},{'parry_master'},{'hang_time'},{'iron_resolve'},{'last_stand'},{'desperado'}}},{name='Renewal',alts={{'renewal'}}}}},
  {id='technique',name='Technique crit',colour=0xFF3FA4,motif='sight',blurb='A technique forces a crit and the crit pays out.',
   roles={{name='a technique trigger',alts={{'wave_edge'},{'combo_conduit'}}},{name='a crit payoff',alts={{'critical_flow'},{'critical_mass'}}}}},
  {id='crit',name='Passive crit',colour=0xA67CFF,motif='burst',blurb='Crit chance meets a stronger crit.',
   roles={{name='a crit chance',alts={{'keen'},{'ruthless'},{'finishing'},{'gambler'},{'executioner'}}},{name='a crit payoff',alts={{'brutal'},{'critical_flow'},{'critical_mass'}}}}},
  {id='armour',name='Armour retaliation',colour=0xC8D2DC,motif='spike',blurb='Armour that absorbs a hit makes your next hit a crit.',
   roles={{name='an armour source',alts={{'shield_stance'},{'wavedasher'},{'powershield_oath'},{'juggernaut'}}},{name='Retaliation',alts={{'retaliation'}}}}},
 }
 G.by_id={};for i,a in ipairs(G.archetypes) do G.by_id[a.id]=a;a.index=i end
 local piece_cache=setmetatable({},{__mode='k'})
 -- id -> list of archetypes the record is a piece of (only ids that exist in the pool count)
 function G.pieces(pool)
  local c=piece_cache[pool];if c and c.n==#pool then return c end
  local exists={};for _,m in ipairs(pool) do exists[m.id]=true end
  c={n=#pool,of={},missing={}}
  for _,a in ipairs(G.archetypes) do for _,r in ipairs(a.roles) do for _,alt in ipairs(r.alts) do for _,id in ipairs(alt) do
   if exists[id] then c.of[id]=c.of[id] or {};local l=c.of[id];local dup;for _,x in ipairs(l) do if x==a then dup=true end end;if not dup then l[#l+1]=a end
   else c.missing[#c.missing+1]=a.id..':'..id end
  end end end end
  piece_cache[pool]=c;return c
 end
 -- Roles filled by the held set: {filled,total,complete,missing=name of the first open role,used=set of ids that filled a role}.
 function G.progress(a,held)
  local filled,missing=0,nil;local used={}
  for _,r in ipairs(a.roles) do
   local ok=false
   for _,alt in ipairs(r.alts) do local all=true;for _,id in ipairs(alt) do if not held[id] then all=false;break end end
    if all then ok=true;for _,id in ipairs(alt) do used[id]=true end;break end
   end
   if ok then filled=filled+1 elseif not missing then missing=r.name end
  end
  return {filled=filled,total=#a.roles,complete=filled==#a.roles,missing=missing,used=used}
 end
 -- Archetypes with at least one role filled, completed ones first. `held` is id -> truthy.
 function G.analyze(held)
  local out={}
  for i,a in ipairs(G.archetypes) do local p=G.progress(a,held);if p.filled>0 then p.archetype=a;out[#out+1]=p end end
  table.sort(out,function(x,y)
   if x.complete~=y.complete then return x.complete end
   if x.filled~=y.filled then return x.filled>y.filled end
   return x.archetype.index<y.archetype.index
  end)
  return out
 end
 function G.completed(held) local out={};for _,p in ipairs(G.analyze(held)) do if p.complete then out[#out+1]=p.archetype end end;return out end
 -- What a set of ids (an offered drive's affixes, a keystone) would do to the held set: 'complete' (finishes an archetype), 'advance' (fills one more role of an
 -- archetype you already have a part of), or nil; and the archetype.
 function G.effect_of(ids,held)
  local after={};for k,v in pairs(held) do after[k]=v end;for _,id in ipairs(ids) do after[id]=true end
  local adv
  for _,a in ipairs(G.archetypes) do
   local b,f=G.progress(a,held),G.progress(a,after)
   if f.complete and not b.complete then return 'complete',a end
   if f.filled>b.filled and b.filled>0 and not adv then adv=a end
  end
  if adv then return 'advance',adv end
  return nil
 end
 -- The archetype a chain link (record `src` fed record `dst`) belongs to: one that has both as pieces, else one that has dst, else src.
 function G.chain_archetype(pool,src,dst)
  local of=G.pieces(pool).of
  for _,a in ipairs(of[dst] or {}) do for _,b in ipairs(of[src] or {}) do if a==b then return a end end end
  return (of[dst] or {})[1] or (of[src] or {})[1]
 end
 -- ---- connecting offers ---------------------------------------------------------------------------------------------------
 -- When none of the offered drive records connects to anything held, replace one (index from `pick`) with a roll of the same rarity pulled toward partners of
 -- a held piece. `seed_of(try,k)` gives the seed of each attempt, so the same stage always shows the same offers. Returns the index replaced, or nil.
 function G.connect_offers(pool,loot,offers,held,pick,ctx,rarity_of,seed_of)
  if #offers==0 or not next(held) then return nil end
  local function ids(r) local l={};for _,a in ipairs(r.affixes) do l[#l+1]=a.id end;return l end
  for _,r in ipairs(offers) do if G.connects(pool,ids(r),held) then return nil end end
  local graph=G.graph(pool);local bias={}
  for h in pairs(held) do for p in pairs(graph.partners[h] or {}) do if not held[p] then bias[p]=12 end end end
  if not next(bias) then return nil end
  local k=1+(pick%#offers)
  for try=1,24 do
   local ok,r=pcall(loot.roll,loot,seed_of(try,k),ctx,rarity_of(k),nil,bias)
   if ok and r and G.connects(pool,ids(r),held) then offers[k]=r;return k,r end
  end
  return nil
 end
 -- ---- words -------------------------------------------------------------------------------------------------------------------
 local status_name={burn='Burn',shock='Shock',chill='Chill',curse='Curse',haste='Haste',guarded='Guarded',momentum='Momentum'}
 local function sets_phrase(key)
  local st,subj=key:match('^status:(%a+):(%a+)$')
  if st then if subj=='target' then return 'Sets '..(status_name[st] or st)..' on the target' end;return 'Gives you '..(status_name[st] or st) end
  st=key:match('^status:(%a+)$');if st and st~='any' then return 'Gives '..(status_name[st] or st) end
  if key=='status:any' then return 'Puts a status on the target' end
  local e=key:match('^elem:(%a+)$');if e then return 'Makes your hits '..e:sub(1,1):upper()..e:sub(2) end
  if key=='event:crit' then return 'Sets up a crit' end
  if key=='critchance' then return 'Gives you a crit chance' end
  if key=='event:armor' then return 'Gives armour to absorb a hit' end
  return 'Feeds'
 end
 -- One short line for a link: "Sets Burn on the target, which your Cinder drive turns into +10% damage."
 function G.link_text(pool,a_id,b_id,tier)
  local g=G.graph(pool);local e=(g.out[a_id] or {})[b_id] or (g.out[b_id] or {})[a_id];if not e then return nil end
  local by={};for _,m in ipairs(pool) do by[m.id]=m end
  local to=by[e.to];if not (by[e.from] and to) then return nil end
  tier=tier or 1
  local uses
  for _,eff in ipairs(to.effects) do
   if eff.op=='versus-status' and eff.status then
    local S=D.mod_schema;local c=eff.change
    if c.percent_damage then uses='turns into +'..math.floor((S.ratio(c.percent_damage,to,tier,'damage_dealt')-1)*100+.5)..'% damage' break end
    if c.launch then uses='turns into +'..math.floor((S.ratio(c.launch,to,tier,'launch_dealt')-1)*100+.5)..'% launch' break end
   end
  end
  local noun=to.kind=='keystone' and 'keystone' or 'drive'
  if not uses then if e.kind=='enable' then uses='then makes use of' else uses=to.trigger=='equip' and 'builds on' or 'fires from' end end
  return sets_phrase(e.why)..', which your '..to.label..' '..noun..' '..uses..'.',e
 end
 return G
end
