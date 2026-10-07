-- Plain words for a run's screens: what a drive does, never ids, tiers or internal points. Pure functions of a
-- loot record and the pool; the screens, the HUD and the opponent plate all read them.
return function(D)
 local T={}
 T.rarity_colour={common=0xD7D4CFFF,magic=0x8CABFFFF,rare=0xEBD175FF,unique=0xE8A269FF}
 T.rarity_label={common='Common',magic='Magic',rare='Rare',unique='Unique'}
 T.base_colour={red=0xF07474FF,green=0x77CD9CFF,blue=0x79AAF0FF,yellow=0xEBD175FF,purple=0xC79BFFFF,white=0xFFFFFFFF}
 T.base_line={red='Base: +8% damage you deal',green='Base: +8% run and air speed',blue='Base: 8% less launch taken',yellow='Base: +8% jump height',
  purple='Base: your status effects last 20% longer',white='Base: no bonus'}
 local function round(n) return math.floor(n+.5) end
 local function pct(n) return ('%d%%'):format(round(n*100)) end
 local function secs(frames) local s=frames/60;return s==math.floor(s) and ('%d s'):format(s) or ('%.1f s'):format(s) end
 local value_labels={damage_dealt='Damage you deal',damage_taken='Damage you take',run_speed='Run speed',air_speed='Air speed',jump_height='Jump height',
  air_jump_height='Air jump height',knockback_taken='Launch you take',status_duration='Your status effects last'}
 local elements={fire='fire',ice='ice',electric='electric'}
 local function status_label(name) return D.mod_status.label(name) end
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
    local target=e.status and (status_label(e.status)..' targets') or 'targets'
    if e.change.launch then out[#out+1]=('Your hits launch %s %s farther'):format(target,pct(res(m,tier,'ratio')-1))
    elseif e.change.percent_damage then out[#out+1]=('Your hits deal %s more damage to %s'):format(pct(res(m,tier,'ratio')-1),target) end
   end
  end
 end
 -- Triggered rules: written by hand, numbers pulled from the modifier's own tier table. ONE word per status (mod_status.label): Burning,
-- Chilled, Haste, Guarded, Marked; Momentum is a counter. The status's own numbers (Haste is 20% faster) live in the glossary, not here.
 local function times(n) return n==1 and 'once' or n==2 and 'twice' or (n..' times') end
 local special={
  kindling=function(m,t) return ('Your hits set the target Burning for %s.'):format(secs(res(m,t,'duration'))) end,
  feasting=function(m,t) return ('Knock out a target that has a status: heal %d%%.'):format(round(res(m,t,'heal'))) end,
  updraft=function(m,t) return ('Aerial hits add Momentum (up to 5) for %s.'):format(secs(res(m,t,'duration'))) end,
  crosswind=function(m,t) return ('Land with Momentum: spend 1, gain Haste for %s.'):format(secs(res(m,t,'duration'))) end,
  icebound=function(m,t) return ('Your hits Chill the target for %s.'):format(secs(res(m,t,'duration'))) end,
  brittle=function(m,t) return ('Hit a Chilled target: Mark it for %s (+%s launch).'):format(secs(res(m,t,'duration')),pct(res(m,t,'bonus'))) end,
  reprisal=function(m,t) return ('Perfect shield: Guarded for %s.'):format(secs(res(m,t,'duration'))) end,
  ledge=function(m,t) return ('Grab a ledge: Haste for %s.'):format(secs(res(m,t,'duration'))) end,
  bastion=function(m,t) return ('Land with Momentum: spend 1, gain Guarded for %s.'):format(secs(res(m,t,'duration'))) end,
  renewal=function(m,t) return ('Whenever you become Guarded: heal %d%%.'):format(round(res(m,t,'heal'))) end,
  rush=function(m,t) return ('Hit while you have Haste: add 1 Momentum for %s.'):format(secs(res(m,t,'duration'))) end,
  shelter=function(m,t) return ('Grab a ledge: Guarded for %s.'):format(secs(res(m,t,'duration'))) end,
  malice=function(m,t) return ('Hit a Burning target: Mark it for %s (+%s launch).'):format(secs(res(m,t,'duration')),pct(res(m,t,'bonus'))) end,
  cleansing=function() return 'Every second, Burning, Chilled and Marked on you wear off.' end,
  mirror_shard=function() return 'When your attack clashes with a fighter: they take both attacks\' damage.' end,
  trailing=function() return 'While you have Haste, you leave afterimages.' end,
  echoes=function(m,t) local n=math.max(1,math.min(3,math.floor(res(m,t,'copies'))));return ('Aerial attacks repeat %s, weaker.'):format(times(n)) end,
  echo_heart=function() return 'Every attack repeats a moment later at reduced damage.' end,
  pyromancer=function() return 'All your attacks become fire.' end,
  frozen_oath=function() return 'All your attacks become ice.' end,
  glass_core=function(m,t) local n=D.mod_schema.ratio(1.6,m,t,'damage_dealt')-1;return ('Damage you deal and take both %s%s.'):format('+',pct(n)) end,
 }
 local drawback={pyromancer='Drawback: ice hits against you deal 60% more damage.',frozen_oath='Drawback: fire hits against you deal 60% more damage.',
  echo_heart='Drawback: all your attack damage is 25% lower.'}
 -- ---- technique and crit modifiers: "<when>: <what>" built from the record ----------------------------------------------------
 -- The words of a trigger as the rule line says them (short: the tutorial line that used to follow is gone).
 local function ordinal(n) return n==1 and '1st' or n==2 and '2nd' or n==3 and '3rd' or (n..'th') end
 local function trigger_words(m)
  local k=m.trigger;local n;for _,c in ipairs(m.conditions or {}) do if c.combo_at_least then n=c.combo_at_least end end
  if k=='combo' then return ordinal(n or 3)..' hit of a combo' end
  if k=='combo_end' then return 'Finish a combo of '..(n or 3)..'+ hits' end
  if k=='lcancel_hit' then return 'L-cancel after a hit' end
  if k=='crit' then return 'Land a crit' end
  if k=='armor' then return 'When armour absorbs a hit' end
  local words=D.mod_skill.words(k,m.conditions) or k
  return (words:gsub('^%l',string.upper))
 end
 local armor_words={super='super armour',hit_count='armour that absorbs the next hit',damage_threshold='armour against weak hits',damage_pool='armour that soaks damage',knockback_threshold='armour against weak launches'}
 local function has_tag(m,t) for _,x in ipairs(m.tags or {}) do if x==t then return true end end return false end
 local function mult_text(v) return (('%.2f'):format(v):gsub('0+$',''):gsub('%.$','')) end
 local function effect_words(m,tier,e)
  local r=function(v) return v~=nil and D.mod_schema.resolve(v,m,tier) or nil end
  if e.op=='status' or e.op=='stacks' then return D.mod_status.label(e.status)..' for '..secs(r(e.duration)) end
  if e.op=='chain_status' then return 'chain '..D.mod_status.label(e.status)..' to the nearest other opponent' end
  if e.op=='armor' then
   if e.frames then local f=r(e.frames);return armor_words[e.type]..' for '..(f<60 and (round(f)..' frames') or secs(f)) end
   return 'you do not flinch from hits under '..round(r(e.value))..' damage'
  end
  if e.op=='intangible' then return 'intangible for '..round(r(e.frames))..' frames' end
  if e.op=='interrupt' then return 'cancel your move for '..round(r(e.frames))..' frames' end
  if e.op=='crit_next' then return 'your next hit crits (x'..mult_text(r(e.multiplier))..')' end
  if e.op=='heal' then return 'heal '..round(r(e.amount))..'%' end
  if e.op=='damage' then return 'lose '..round(r(e.amount))..' damage' end
  if e.op=='air_jumps' then return 'you have '..round(r(e.count))..' air jumps' end
  if e.op=='restrict' then return 'you cannot '..table.concat(e.forbid,' or ') end
  return nil
 end
 -- A passive crit rule speaks EVERY crit effect it has (Brutal used to say only its chance and hide the multiplier it exists for):
 -- chance, a multiplier gain, a tag, a percent floor. A crit is x1.5 unless a piece states its own multiplier.
 local function crit_line(m,tier)
  local r=function(v) return v~=nil and D.mod_schema.resolve(v,m,tier) or nil end
  local chance,gain,own,tag,floor=0,0,nil,nil,nil
  for _,e in ipairs(m.effects) do if e.op=='crit' then
   if e.chance then chance=chance+r(e.chance) end
   if e.multiplier then if e.chance then own=r(e.multiplier) else gain=gain+(r(e.multiplier)-1) end end
   tag=tag or e.tag;floor=floor or e.min_percent
  end end
  local out
  if floor then out=('Hits on a target above %d%% crit %s of the time'):format(round(r(floor)),pct(chance))
  else out=('Your %shits crit %s of the time'):format(tag and (tag..' ') or '',pct(chance)) end
  if own then out=out..' (x'..mult_text(own)..')' end
  if gain>0 then out=out..'; crits gain +'..mult_text(gain)..'x' end
  return out..'.'
 end
 local function technique_lines(m,tier)
  local has_crit;for _,e in ipairs(m.effects) do if e.op=='crit' then has_crit=true end end
  if m.trigger=='equip' and has_crit then return {crit_line(m,tier)} end
  local parts={};for _,e in ipairs(m.effects) do local w=effect_words(m,tier,e);if w then parts[#parts+1]=w end end
  if m.trigger=='equip' then return {((parts[1] or ''):gsub('^%l',string.upper))..'.'} end
  return {trigger_words(m)..': '..table.concat(parts,', ')..'.'}
 end
-- Lines (strings) for one modifier at one tier. A rule is described by its sentence when it has one, else from its effects.
 function T.mod_lines(m,tier)
  local out={}
  -- Keystones (keystones.lua owns their words: the effect, then the drawback, no ids and no tier codes).
  local k=m.kind=='keystone' and D.keystones and D.keystones.lines(m,tier)
  if k then return {k[1],k[2]} end
  if special[m.id] then
   out[#out+1]=special[m.id](m,tier)
  elseif has_tag(m,'technique') or has_tag(m,'critical') then return technique_lines(m,tier)
  else generic(m,tier,out) end
  if #out==0 then out[1]=(D.mod_schema.describe(m,tier):gsub('^Tier %d+: ','')) end
  -- A keystone's price is its own line; a unique already states its price in its value lines.
  if m.kind=='keystone' and (drawback[m.id] or m.cost) then out[#out+1]=drawback[m.id] or ('Drawback: '..m.cost) end
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
 -- The glossary: one line per word a player learns (statuses, the Momentum counter, the crit multiplier).
 function T.glossary() return D.mod_status.glossary() end
 -- Keystone: its sentence and its price.
 function T.keystone_lines(m,tier) return T.mod_lines(m,tier or 1) end
 -- Totals a player cares about, from the budget families. `after` may be nil.
 T.total_rows={{'strength','Build strength'},{'damage_dealt','Damage you deal'},{'launch_dealt','Launch you deal'},{'speed','Speed'},{'damage_taken','Damage you take'}}
 function T.totals(families,strength)
  return {strength=strength,damage_dealt=families.damage_dealt.value,launch_dealt=families.launch_dealt.value,speed=families.speed.value,damage_taken=families.damage_taken.value}
 end
 -- Build strength is an INDEX, not a damage figure: how many times stronger the whole build is than an empty one (x1.00), the same "x" the
 -- other rows use. It was "+358%", which read as a damage bonus, and at New Game+ as "+8600%". Three significant figures: x1.19, x4.58, x87.1, x164.
 -- Damage you take at its floor (x0.15: the engine takes nothing lower, and the budget clamps there) says so, so a defence pick that
 -- changes nothing is explained, not mysterious.
 T.damage_taken_floor=.15
 function T.strength_text(v) if v<10 then return ('x%.2f'):format(v) elseif v<100 then return ('x%.1f'):format(v) end;return ('x%.0f'):format(v) end
 local function fmt(key,v)
  if key=='strength' then return T.strength_text(v) end
  if key=='damage_taken' and v<=T.damage_taken_floor+.0005 then return 'x0.15 (limit)' end
  return ('x%.2f'):format(v)
 end
 -- Whether a change is good for the player: more strength/damage/launch/speed is good, more damage taken is bad.
 function T.better(key,a,b) if math.abs(a-b)<.005 then return nil end;if key=='damage_taken' then return b<a end;return b>a end
 function T.total_line(key,label,before,after)
  if after==nil then return label..' '..fmt(key,before[key]) end
  if math.abs(before[key]-after[key])<.005 then return label..' '..fmt(key,before[key])..' (no change)' end
  return label..' '..fmt(key,before[key])..' -> '..fmt(key,after[key])
 end
 -- A technique or crit rule (L-cancel, wavedash, perfect shield, tech, a crit chance) is worth what the player makes of it, and the strength index counts it
 -- at a modest assumed rate. When a drive carries one the detail panel says so, so "Build strength: no change" is not read as "does nothing".
 T.technique_note='Technique: worth more the more you use it. The strength number counts it lightly.'
 function T.has_technique(loot,r)
  for _,a in ipairs(r.affixes or {}) do
   local rule=loot.rules[a.id]
   for _,tag in ipairs(rule and rule.tags or {}) do if tag=='technique' or tag=='critical' then return true end end
  end
  return false
 end
 -- Opponent plate lines: each KEYSTONE the opponent holds (its name and its rule) and a count of its drive rules. The drive rules
 -- themselves are not listed (an opponent shows what defines it, not a page of text).
 function T.rule_count(build)
  local n=0
  for slot=1,D.mod_progression.slots(build.context) do local r=build.equipped[slot];if r then n=n+#r.affixes end end
  return n
 end
 function T.build_lines(loot,build,rules)
  local out={}
  local keys={};for _,id in ipairs(build.keystones or {}) do keys[id]=true end;if build.keystone then keys[build.keystone]=true end
  for _,m in ipairs(rules) do if keys[m.id] then local l=T.mod_lines(m,1);out[#out+1]=m.label..': '..(l[1] or '') end end
  local n=T.rule_count(build)
  if n>0 or #out>0 then out[#out+1]=n==1 and '1 drive rule' or (n..' drive rules') end
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
