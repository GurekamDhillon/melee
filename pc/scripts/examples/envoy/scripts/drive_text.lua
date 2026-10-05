-- Plain words for a run's screens: what a drive does, never ids, tiers or internal points. Pure functions of a
-- loot record and the pool; the screens, the HUD and the opponent plate all read them.
return function(D)
 local T={}
 T.rarity_colour={common=0xD7D4CFFF,magic=0x8CABFFFF,rare=0xEBD175FF,unique=0xE8A269FF}
 T.rarity_label={common='Common',magic='Magic',rare='Rare',unique='Unique'}
 T.base_colour={red=0xF07474FF,green=0x77CD9CFF,blue=0x79AAF0FF,yellow=0xEBD175FF,purple=0xC79BFFFF,white=0xFFFFFFFF}
 T.base_line={red='Base: +8% damage you deal',green='Base: +8% run and air speed',blue='Base: 8% less launch taken',yellow='Base: +8% jump height',
  purple='Base: your status effects last 20% longer',white='Base: no bonus, but one extra modifier'}
 local function round(n) return math.floor(n+.5) end
 local function pct(n) return ('%d%%'):format(round(n*100)) end
 local function secs(frames) local s=frames/60;return s==math.floor(s) and ('%d seconds'):format(s) or ('%.1f seconds'):format(s) end
 local value_labels={damage_dealt='Damage you deal',damage_taken='Damage you take',run_speed='Run speed',air_speed='Air speed',jump_height='Jump height',
  air_jump_height='Air jump height',knockback_taken='Launch you take',status_duration='Your status effects last'}
 local elements={fire='fire',ice='ice',electric='electric'}
 local statuses={burn='Burning',chill='Chilled'}
 local function sign(n) return n>=0 and '+' or '-' end
 local function res(m,tier,key) return D.mod_schema.resolve('$'..key,m,tier) end
 -- Effects that read well straight from the data.
 local function generic(m,tier,out)
  for _,e in ipairs(m.effects) do
   if e.op=='value' then
    local n=D.mod_schema.ratio(e.value,m,tier,D.mod_budget.families[e.key])-1
    local label=value_labels[e.key] or e.key
    if e.key=='status_duration' then out[#out+1]=('%s %s longer'):format('Status effects you cause last',pct(math.abs(n)))
    elseif e.key=='knockback_taken' then out[#out+1]=('Launch you take %s%s'):format(sign(n),pct(math.abs(n)))
    else out[#out+1]=('%s %s%s'):format(label,sign(n),pct(math.abs(n))) end
   elseif e.op=='convert' and e.change and e.change.element then
    local who=e.match.move=='smash' and 'Smash attacks' or e.match.move=='aerial' and 'Aerial attacks' or 'All your attacks'
    out[#out+1]=('%s become %s'):format(who,elements[e.change.element] or e.change.element)
   elseif e.op=='versus-status' and not e.match.incoming then
    local target=e.status and ((statuses[e.status] or e.status)..' targets') or 'targets'
    if e.change.launch then out[#out+1]=('Your hits launch %s %s farther'):format(target,pct(res(m,tier,'ratio')-1))
    elseif e.change.percent_damage then out[#out+1]=('Your hits deal %s more damage to %s'):format(pct(res(m,tier,'ratio')-1),target) end
   end
  end
 end
 -- Triggered rules: written by hand, numbers pulled from the modifier's own tier table.
 local special={
  kindling=function(m,t) return ('Fire hits set the target Burning for %s (%d damage every second)'):format(secs(res(m,t,'duration')),round(res(m,t,'damage'))) end,
  feasting=function(m,t) return ('Knock out a target that has a status effect: heal %d%%'):format(round(res(m,t,'heal'))) end,
  updraft=function(m,t) return ('Aerial hits build Momentum (up to 5) for %s'):format(secs(res(m,t,'duration'))) end,
  crosswind=function(m,t) return ('Land with Momentum: spend it for +20%% run and air speed for %s'):format(secs(res(m,t,'duration'))) end,
  icebound=function(m,t) return ('Ice hits Chill the target (20%% slower) for %s'):format(secs(res(m,t,'duration'))) end,
  brittle=function(m,t) return ('Hit a Chilled target: your later hits launch them %s farther for %s'):format(pct(res(m,t,'bonus')),secs(res(m,t,'duration'))) end,
  reprisal=function(m,t) return ('Perfect shield: take 25%% less damage for %s (Guarded), and gain 1 Momentum'):format(secs(res(m,t,'duration'))) end,
  ledge=function(m,t) return ('Grab a ledge: +20%% run and air speed for %s'):format(secs(res(m,t,'duration'))) end,
  bastion=function(m,t) return ('Land with Momentum: take 25%% less damage for %s'):format(secs(res(m,t,'duration'))) end,
  renewal=function(m,t) return ('Whenever you become Guarded (less damage taken): heal %d%%'):format(round(res(m,t,'heal'))) end,
  rush=function(m,t) return ('Hit while moving fast: gain Momentum for %s'):format(secs(res(m,t,'duration'))) end,
  shelter=function(m,t) return ('Grab a ledge: take 25%% less damage for %s'):format(secs(res(m,t,'duration'))) end,
  malice=function(m,t) return ('Hit a Burning target: your later hits launch them %s farther for %s'):format(pct(res(m,t,'bonus')),secs(res(m,t,'duration'))) end,
  cleansing=function() return 'Every second, Burn, Chill and Curse on you wear off' end,
  mirror_shard=function() return 'When your attack clashes with a fighter: they take both attacks\' damage' end,
  trailing=function() return 'While moving fast, you leave afterimages' end,
  echoes=function() return 'Your aerial attacks repeat a moment later at reduced damage (more copies at higher tiers)' end,
  echo_heart=function() return 'Every attack repeats a moment later at reduced damage' end,
  echo_oath=function() return 'Every attack repeats a moment later at reduced damage' end,
  pyromancer=function() return 'All your attacks become fire' end,
  frozen_oath=function() return 'All your attacks become ice' end,
  still_heart=function() return 'Moving fast turns into taking less damage instead' end,
  glass_core=function(m,t) local n=D.mod_schema.ratio(1.6,m,t,'damage_dealt')-1;return ('Damage you deal and take both %s%s'):format('+',pct(n)) end,
 }
 local drawback={pyromancer='Drawback: ice hits against you deal 60% more damage',frozen_oath='Drawback: fire hits against you deal 60% more damage',
  still_heart='Drawback: you cannot get a speed boost',echo_oath='Drawback: all your attack damage is 50% lower',echo_heart='Drawback: all your attack damage is 25% lower'}
 -- Lines (strings) for one modifier at one tier. A rule is described by its sentence when it has one, else from its effects.
 function T.mod_lines(m,tier)
  local out={}
  -- Keystones (keystones.lua owns their words: the effect, then the drawback, no ids and no tier codes).
  local k=m.kind=='keystone' and D.keystones and D.keystones.lines(m,tier)
  if k then return {k[1],k[2]} end
  if special[m.id] then
   out[#out+1]=special[m.id](m,tier)
  else generic(m,tier,out) end
  if #out==0 then out[1]=(D.mod_schema.describe(m,tier):gsub('^Tier %d+: ','')) end
  -- A keystone's price is its own line; a unique already states its price in its value lines.
  if m.kind=='keystone' and (drawback[m.id] or m.cost) then out[#out+1]=drawback[m.id] or ('Drawback: '..m.cost)
  elseif m.id=='mirror_shard' then out[#out+1]='Only works against fighters, not items' end
  return out
 end
 -- One record: the rarity line, the base line, then every modifier in plain words.
 function T.drive_lines(loot,r)
  local out={}
  out[1]=T.base_line[r.colour] or 'Base: none'
  for _,a in ipairs(r.affixes) do for _,line in ipairs(T.mod_lines(loot.rules[a.id],a.tier)) do out[#out+1]=line end end
  return out
 end
 function T.header(loot,r) return (T.rarity_label[r.rarity] or r.rarity)..' / '..loot:name(r) end
 -- Keystone: its sentence and its price.
 function T.keystone_lines(m,tier) return T.mod_lines(m,tier or 1) end
 -- Totals a player cares about, from the budget families. `after` may be nil.
 T.total_rows={{'strength','Build strength'},{'damage_dealt','Damage you deal'},{'launch_dealt','Launch you deal'},{'speed','Speed'},{'damage_taken','Damage you take'}}
 function T.totals(families,strength)
  return {strength=strength,damage_dealt=families.damage_dealt.value,launch_dealt=families.launch_dealt.value,speed=families.speed.value,damage_taken=families.damage_taken.value}
 end
 -- Build power is shown as a percentage over a build with nothing equipped, never the budget's own number.
 local function fmt(key,v) if key=='strength' then return ('%+d%%'):format(round((v-1)*100)) end;return ('x%.2f'):format(v) end
 -- Whether a change is good for the player: more strength/damage/launch/speed is good, more damage taken is bad.
 function T.better(key,a,b) if math.abs(a-b)<.005 then return nil end;if key=='damage_taken' then return b<a end;return b>a end
 function T.total_line(key,label,before,after)
  if after==nil then return label..' '..fmt(key,before[key]) end
  if math.abs(before[key]-after[key])<.005 then return label..' '..fmt(key,before[key])..' (no change)' end
  return label..' '..fmt(key,before[key])..' -> '..fmt(key,after[key])
 end
 -- Opponent plate lines: every modifier a build carries, once each, in plain words.
 function T.build_lines(loot,build,rules)
  local out,seen={},{}
  local function add(line) if not seen[line] then seen[line]=true;out[#out+1]=line end end
  for slot=1,D.mod_progression.slots(build.context) do local r=build.equipped[slot];if r then
   for _,a in ipairs(r.affixes) do for _,line in ipairs(T.mod_lines(loot.rules[a.id],a.tier)) do add(line) end end
  end end
  local keys={};for _,id in ipairs(build.keystones or {}) do keys[id]=true end;if build.keystone then keys[build.keystone]=true end
  -- one line per keystone: its name and its effect (the drawback is on the player's own screens, not on an opponent's plate)
  for _,m in ipairs(rules) do if keys[m.id] then local l=T.mod_lines(m,1);add(m.label..': '..(l[1] or '')) end end
  return out
 end
  -- ---- one line per drive in lists, one call per grid cell ---------------------------------------------------------
 -- A list shows ONE short line per drive: the colour name and its modifiers' labels (at most two, then "+N").
 -- Full detail (drive_lines, header) is for the focused item only. Never an id, a tier code or a budget number.
 function T.short(loot,r)
  local base=r.colour:sub(1,1):upper()..r.colour:sub(2)..' Drive'
  if r.unique then return loot.rules[r.unique].label end
  local labels={};for i,a in ipairs(r.affixes) do if i<=2 then labels[#labels+1]=loot.rules[a.id].label end end
  local more=#r.affixes-#labels
  return base..': '..table.concat(labels,', ')..(more>0 and (' +'..more) or '')
 end
 -- Everything a grid cell needs, from one call, with no text needed to read it: colour = family, rarity = border,
 -- pips = how many modifiers it carries, level = 1..5 (its highest modifier's tier, clamped; drawn as pips or a
 -- corner mark, never printed as a code), flags for NEW and for "can merge". `flags` is optional: {new=,can_merge=,equipped=}.
 function T.cell(loot,r,flags)
  flags=flags or {}
  local top=0;for _,a in ipairs(r.affixes) do local t=a.tier;if type(t)=='table' then t=t.tier or 1 end;if t>top then top=t end end
  return {kind='drive',colour=r.colour,colour_rgba=T.base_colour[r.colour],rarity=r.rarity,rarity_rgba=T.rarity_colour[r.rarity],
   affixes=#r.affixes,level=math.max(1,math.min(5,top)),merged=r.merged or 0,unique=r.unique~=nil,
   name=T.short(loot,r),new=flags.new==true,can_merge=flags.can_merge==true,equipped=flags.equipped==true}
 end
 -- A keystone's cell: its drive-family colour (red damage, green speed, blue defence, yellow air, purple status, white wild).
 function T.keystone_cell(m,flags)
  flags=flags or {};local family=D.keystones and D.keystones.family(m.id) or 'white'
  return {kind='keystone',colour=family,colour_rgba=T.base_colour[family],name=m.label,
   family=D.keystones and D.keystones.family_names[family] or 'Wild',held=flags.held==true,offered=flags.offered==true,new=flags.new==true}
 end
 return T
end
