-- The one declaration of the technique ("skill") vocabulary: which engine skill events the modifier engine
-- accepts as triggers, whether each was verified in the game, who can perform it, the plain words for it and
-- the colour of the afterimage a status earned by it shows. Pure data; schema, host, text and the earned
-- afterimage module all read it, so a technique is declared once.
--
-- Presentation table (ONE meaning per output):
--   surface treatment  = a status you have (equipped rules and statuses)
--   earned afterimage  = a status you EARNED through technique, only while it lasts, coloured by its cause
--   tracer             = THIS hit carries something extra (a crit)
--   particle burst / post pass = a moment (a crit, a keystone), never continuous
return function(D)
 local M={}
 -- kind -> {verified, cpu, words, cause, noun}
 --  verified: seen firing in the game by the engine lane (docs/scripting.md "Skill events"); false = available but flagged
 --  cpu: whether a CPU opponent performs it. 'live' = hits and knockdowns happen anyway; 'driven' = the retail AI never does it but foe_driver.lua
 --       performs it for an opponent whose build rewards it (a fight -> script -> fight switch, at a skill that grows with depth);
 --       'maybe' = retail AI does it at some levels (unmeasured); 'dead' = retail AI never performs it, so an opponent's roll is inert
 --  cause: afterimage colour family; the status the technique grants shows that colour while it lasts
 M.kinds={
  lcancel={verified=true,cpu='driven',words='L-cancel a landing',cause='lcancel'},
  lcancel_hit={verified=true,cpu='driven',words='L-cancel a landing after the aerial hit',cause='lcancel'},
  lcancel_miss={verified=true,cpu='maybe',words='miss an L-cancel',cause='miss'},
  wavedash={verified=true,cpu='driven',words='wavedash',cause='wave'},
  perfect_shield={verified=true,cpu='driven',words='perfect shield',cause='shield',legacy=true},
  tech={verified=true,cpu='driven',words='tech',cause='shield'},
  tech_miss={verified=true,cpu='live',words='miss a tech',cause='miss'},
  short_hop={verified=true,cpu='maybe',words='short hop',cause='move'},
  fast_fall={verified=true,cpu='maybe',words='fast fall',cause='move'},
  dash_dance={verified=true,cpu='dead',words='dash dance',cause='move'},
  jump_cancel_grab={verified=true,cpu='dead',words='jump-cancel a grab',cause='combo'},
  combo={verified=true,cpu='live',words='land a hit inside a combo',cause='combo'},
  combo_end={verified=true,cpu='live',words='finish a combo',cause='combo'},
  -- Available, flagged: seen in the build but not confirmed in play (engine lane report), or only partly measured.
  waveland={verified=false,cpu='dead',words='waveland',cause='wave'},
  ledge_dash={verified=false,cpu='dead',words='ledge dash',cause='wave'},
  sdi={verified=false,cpu='maybe',words='smash DI during hitlag',cause='shield'},
  shield_drop={verified=false,cpu='maybe',words='drop your shield',cause='shield'},
  auto_cancel={verified=false,cpu='maybe',words='auto-cancel a landing',cause='lcancel'},
  air_dodge={verified=false,cpu='driven',words='air dodge',cause='wave'},
  full_hop={verified=false,cpu='maybe',words='full hop',cause='move'},
  jump_cancel_usmash={verified=false,cpu='dead',words='jump-cancel an up smash',cause='combo'},
  -- Not a technique: a consequence the engine reports (crit decision, armour absorbing a hit).
  crit={verified=true,cpu='live',words='land a crit',cause='crit',consequence=true},
  armor={verified=true,cpu='live',words='have your armour absorb a hit',cause='shield',consequence=true},
 }
 M.order={'lcancel','lcancel_hit','lcancel_miss','wavedash','perfect_shield','tech','tech_miss','short_hop','fast_fall','dash_dance','jump_cancel_grab','combo','combo_end','waveland','ledge_dash','sdi','shield_drop','auto_cancel','air_dodge','full_hop','jump_cancel_usmash','crit','armor'}
 -- Engine hooks that deliver each kind (on_<kind>); perfect_shield keeps the retail-signature hook.
 function M.is_skill(kind) return M.kinds[kind]~=nil and not M.kinds[kind].consequence end
 function M.is_technique(kind) return M.is_skill(kind) end
 function M.verified(kind) return M.kinds[kind] and M.kinds[kind].verified==true end
 function M.cpu(kind) return M.kinds[kind] and M.kinds[kind].cpu or 'live' end
 -- Afterimage colour by cause (tint, tail), 0..1 RGBA. One meaning each; a status's afterimage is its cause's colour.
 M.cause={
  lcancel={name='blue',words='blue (speed from an L-cancel)',index=1,tint={.15,.45,1,.8},tail={.05,.1,.6,0}},
  shield={name='teal',words='teal (a defence earned by shield or tech)',index=2,tint={.1,.9,.8,.8},tail={0,.35,.45,0}},
  wave={name='gold',words='gold (a wave technique: armour and speed)',index=3,tint={1,.78,.25,.8},tail={.55,.3,0,0}},
  combo={name='red',words='red (an offensive state from combos and cancels)',index=4,tint={1,.3,.25,.8},tail={.5,.05,.1,0}},
  move={name='white',words='white (a movement technique)',index=5,tint={.9,.95,1,.75},tail={.4,.45,.6,0}},
  miss={name='violet',words='violet (a state paid for a miss)',index=6,tint={.7,.35,1,.75},tail={.25,.05,.45,0}},
  crit={name='orange',words='orange (a crit)',index=7,tint={1,.55,.15,.85},tail={.5,.15,0,0}},
 }
 function M.cause_of(kind) local k=M.kinds[kind];return k and k.cause or nil end
 -- How long an earned afterimage may last at most (frames): a presentation bound, never longer than the status.
 M.max_frames=600
 -- Plain words of a trigger for a sentence ("When you L-cancel a landing after the aerial hit, ...").
 function M.words(kind,conditions)
  local k=M.kinds[kind];if not k then return nil end
  local out=k.words
  for _,c in ipairs(conditions or {}) do
   if c.combo_at_least then
    if kind=='combo' then out='land a hit that makes a combo of at least '..tostring(c.combo_at_least) elseif kind=='combo_end' then out='finish a combo of at least '..tostring(c.combo_at_least) end
   end
  end
  return out
 end
 return M
end
