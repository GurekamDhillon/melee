-- The named tuning values of the Envoy mod, in one table, with two presets.
--   current  : how the mod behaved before the synergy work (the default for every value that is the owner's taste).
--   proposed : the values the synergy analysis (_research/envoy-synergy-analysis-2026-10-05.md) suggests. `envoy tuning proposed` switches them on
--              for a run, `envoy tuning current` back; `envoy tuning` lists every value with both columns.
-- Values here are DATA read by the rule layer (budget, schema, loot), never state that only Lua knows: a preset is applied by the console on
-- every peer alike (netplay would carry it in the lobby rules). Presentation values (T.fx) are per peer.
return function(D)
 local T={rev=0}
 -- name -> {current, proposed, what it does}
 T.table={
  strength={'v2','v2','Build strength formula: v2 counts a record for what it does given what the build feeds it; v1 is the old additive number.'},
  payoff_scale={1,2.5,'Multiplier on the payoff drives of a chain (Pyre, Cinder, Shatter launch/damage bonus; Brittle and Malice curse bonus). Analysis: +5% of total damage is not felt; aim +20..30% on the status uptime.'},
  plague_scale={1,.5,'Multiplier on the burn damage Plague Bearer applies to the target. Analysis: dealt per minute x2.6 with it; Smasher\'s Creed measures +31%.'},
  min_depth_cap={0,4,'Technique and crit drives never roll before this effective depth (0 = their own min_depth, 5 to 8). Analysis: technique crit completes in 4% of 12-stage runs.'},
  connect_offers={1,1,'1: when none of the offered drives (or keystones) connects to anything held, one offer is swapped for a partner of a held piece (seeded).'},
 }
 T.names={'strength','payoff_scale','plague_scale','min_depth_cap','connect_offers'}
 T.values={};T.preset_name='current'
 local function reset() for _,n in ipairs(T.names) do T.values[n]=T.table[n][1] end end
 reset()
 -- strength v2 and connecting offers are the data-supported changes, so they are on in both columns; the three taste values differ.
 function T.get(name) return T.values[name] end
 function T.set(name,value)
  assert(T.table[name],'unknown tuning value '..tostring(name))
  if name=='strength' then assert(value=='v1' or value=='v2','strength is v1 or v2') else value=tonumber(value);assert(value and value==value and value>=0 and value<=64,'a number 0..64 is required') end
  if T.values[name]~=value then T.values[name]=value;T.rev=T.rev+1 end
  T.preset_name='custom'
 end
 function T.preset(name)
  assert(name=='current' or name=='proposed','preset is current or proposed')
  local col=name=='current' and 1 or 2
  for _,n in ipairs(T.names) do local v=T.table[n][col];if T.values[n]~=v then T.values[n]=v;T.rev=T.rev+1 end end
  T.preset_name=name
 end
 -- The payoff drives of the chains and the amount a status effect of a record is scaled by.
 T.payoff_versus={pyre=true,cinder=true,shatter=true}
 T.payoff_curse={brittle=true,malice=true}
 -- Factor on a versus-status ratio delta (S.ratio): only the payoff drives.
 function T.ratio_scale(m) if m and T.payoff_versus[m.id] then return T.values.payoff_scale end;return 1 end
 -- Factor on a status effect's amount (engine apply and budget agree through this one function).
 function T.amount_scale(m,e)
  if not m then return 1 end
  if T.payoff_curse[m.id] and e.status=='curse' then return T.values.payoff_scale end
  if m.id=='plague_bearer' and e.status=='burn' and e.subject=='target' then return T.values.plague_scale end
  return 1
 end
 -- The minimum depth a record rolls at now; validation accepts the lower of the two presets, so a drive rolled under `proposed` stays valid after `current`.
 function T.min_depth(m)
  local d=m.min_depth;if not d then return nil end
  local cap=T.values.min_depth_cap;if cap and cap>0 then d=math.min(d,cap) end
  return d
 end
 function T.min_depth_floor(m)
  local d=m.min_depth;if not d then return nil end
  local cap=T.table.min_depth_cap[2];if cap and cap>0 then d=math.min(d,cap) end
  return d
 end
 function T.lines()
  local out={}
  for _,n in ipairs(T.names) do local r=T.table[n];out[#out+1]=('%-14s now %-6s | current %-5s | proposed %-5s | %s'):format(n,tostring(T.values[n]),tostring(r[1]),tostring(r[2]),r[3]) end
  return out
 end
 -- Presentation layers (per peer, optional one by one). `intensity` scales every synergy visual 0..1.
 -- THE DEVELOPER OVERLAY (off by default, mod side): one predicate gates every developer-only figure and message: the strength and depth
 -- text on the build strip, the strip's flash text, the opponent-strength figure, the mod's error and teaching toasts (always logged, shown only
 -- with it on), the Modifier LAB text box and the Drive LAB card. `envoy devui on|off` sets it; the ENVOY_DEVUI environment flag ('1' or 'on')
 -- is the default where the mod can read it (nothing here asks the engine for anything).
 local env_on=false
 do local ok,v=pcall(function() return os and os.getenv and os.getenv('ENVOY_DEVUI') end);env_on=ok and (v=='1' or v=='on') end
 T.dev_flag=nil
 function T.dev_ui() if T.dev_flag~=nil then return T.dev_flag end;return env_on end
 function T.set_dev_ui(v) T.dev_flag=v and true or false;T.rev=T.rev+0;return T.dev_flag end
 T.fx={intensity=1,grid_links=true,grid_banner=true,offer_marks=true,chain=true,hud=true,surface=true,announce=true,nameplate=true}
 T.fx_names={'grid_links','grid_banner','offer_marks','chain','hud','surface','announce','nameplate'}
 return T
end
