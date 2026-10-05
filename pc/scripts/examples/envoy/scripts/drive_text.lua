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
 local function secs(frames) local s=frames/60;return s==math.floor(s) and (s==1 and '1 second' or ('%d seconds'):format(s)) or ('%.1f seconds'):format(s) end
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
    -- A tier's own number can run past what the engine allows (Damage resistant at tier 9 is x-1.12 raw): the budget caps each family
    -- (damage taken floors at -85%), so the text states the capped number and says so, instead of an impossible "-112%".
    local cap=D.mod_budget.caps[D.mod_budget.families[e.key] or '']
    local capped=false
    if cap and n<cap[1] then n=cap[1];capped=true elseif cap and n>cap[2] then n=cap[2];capped=true end
    if capped then out[#out+1]=('%s %s%s (the most a build can reach)'):format(label,sign(n),pct(math.abs(n)))
    elseif e.key=='status_duration' then out[#out+1]=('%s %s longer'):format('Status effects you cause last',pct(math.abs(n)))
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
 -- ---- technique and crit modifiers: "<when>: <what>" built from the record, plus one line on what to look for ----------------------
 local status_name={haste='Haste',guarded='Guarded',momentum='Momentum',chill='Chill',curse='Curse',burn='Burning',shock='Shock'}
 local armor_words={super='super armour',hit_count='armour that absorbs the next hit',damage_threshold='armour against weak hits',damage_pool='armour that soaks damage',knockback_threshold='armour against weak launches'}
 local function effect_words(m,tier,e)
  local r=function(v) return v~=nil and D.mod_schema.resolve(v,m,tier) or nil end
  if e.op=='status' or e.op=='stacks' then return (status_name[e.status] or e.status)..' for '..secs(r(e.duration)) end
  if e.op=='chain_status' then return 'chain '..(status_name[e.status] or e.status)..' to the nearest other opponent' end
  if e.op=='armor' then
   if e.frames then local f=r(e.frames);return armor_words[e.type]..' for '..(f<60 and (round(f)..' frames') or secs(f)) end
   return 'you do not flinch from hits under '..round(r(e.value))..' damage'
  end
  if e.op=='intangible' then return 'intangible for '..round(r(e.frames))..' frames' end
  if e.op=='interrupt' then return 'cancel your move for '..round(r(e.frames))..' frames' end
  if e.op=='crit_next' then return ('your next hit crits for x%s'):format(('%.2f'):format(r(e.multiplier)):gsub('0+$',''):gsub('%.$','')) end
  if e.op=='heal' then return 'heal '..round(r(e.amount))..'%' end
  if e.op=='damage' then return 'lose '..round(r(e.amount))..' damage' end
  if e.op=='crit' then
   local who=e.tag and ('Your '..e.tag..' hits') or 'Your hits'
   local out=e.chance and (who..' crit '..pct(r(e.chance))..' of the time') or (who..': crits deal more')
   local mult=r(e.multiplier);if mult then out=out..(e.chance and (' (x'..(('%.2f'):format(mult):gsub('0+$',''):gsub('%.$',''))..')') or (' (the multiplier gains '..(('%.2f'):format(mult-1):gsub('0+$',''):gsub('%.$',''))..')')) end
   if e.min_percent then out=out..', but only on a target above '..round(r(e.min_percent))..'% damage' end
   return out
  end
  if e.op=='air_jumps' then return 'you have '..round(r(e.count))..' air jumps' end
  if e.op=='restrict' then return 'you cannot '..table.concat(e.forbid,' or ') end
  return nil
 end
 local technique_look={lcancel='a blue afterimage',shield='a teal afterimage',wave='a gold afterimage',combo='a red afterimage',move='a white afterimage',miss='a violet afterimage'}
 local function technique_lines(m,tier)
  local words=D.mod_skill.words(m.trigger,m.conditions)
  local parts={};for _,e in ipairs(m.effects) do local w=effect_words(m,tier,e);if w then parts[#parts+1]=w end end
  local first
  if m.trigger=='equip' then first=(parts[1] or ''):gsub('^%l',string.upper)..'.'
  else
   local verb=m.trigger=='crit' and 'Land a crit' or m.trigger=='armor' and 'When your armour absorbs a hit' or ((words or m.trigger):gsub('^%l',string.upper))
   if m.trigger=='armor' then first=verb..': '..table.concat(parts,', ')..'.' else first=verb..': '..table.concat(parts,', ')..'.' end
  end
  local out={first}
  local cause=D.mod_skill.cause_of(m.trigger);local earned=D.mod_skill.is_skill(m.trigger)
  for _,e in ipairs(m.effects) do if earned and (e.op=='status') then out[2]='You earn it by technique: '..(technique_look[cause] or 'an afterimage')..' shows while it lasts.';break end end
  if not out[2] and m.trigger=='crit' then out[2]='A crit flashes an impact frame and a tracer on the hit.' elseif not out[2] and m.trigger=='equip' and m.tags[1]=='critical' then out[2]='Crits are rare: the engine has none until a modifier grants a chance.' end
  return out
 end
 -- Lines (strings) for one modifier at one tier. A rule is described by its sentence when it has one, else from its effects.
 function T.mod_lines(m,tier)
  local out={}
  -- Keystones (keystones.lua owns their words: the effect, then the drawback, no ids and no tier codes).
  local k=m.kind=='keystone' and D.keystones and D.keystones.lines(m,tier)
  if k then return {k[1],k[2]} end
  if m.min_depth then return technique_lines(m,tier) end
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
  local labels={};local keep=1   -- the first modifier's label and a count (+N): a name never outgrows its cell, card or panel header; the detail panel lists every modifier
  for i,a in ipairs(r.affixes) do if i<=keep then labels[#labels+1]=loot.rules[a.id].label end end
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
