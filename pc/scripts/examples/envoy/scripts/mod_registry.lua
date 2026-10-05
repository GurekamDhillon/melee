-- One registry of effect descriptors. Schema (field lists), budget (family coverage) and the
-- checkpoint compiler (which journal operation commits an effect) all read it, so a field, a bound or an
-- executor cannot be declared in one place and be missing from another.
return function(D)
 local R={}
 local function set(words) local out={};for w in words:gmatch('%S+') do out[w]=true end;return out end
 -- Rule effects (the `effects` list of a modifier record). `journal` names the native sim_commit
 -- operation that carries the effect after a frame; 'lua_state' effects live in the exported
 -- checkpoint blob and are restored by the snapshot, never replayed through a native setter.
 R.effects={
  status={fields=set('op status subject duration amount max refresh when'),journal='lua_state'},
  stacks={fields=set('op status subject duration amount max refresh when'),journal='lua_state'},
  remove_status={fields=set('op status subject when'),journal='lua_state'},
  value={fields=set('op key value when'),journal='fighter_mod'},
  heal={fields=set('op amount subject when'),journal='damage'},
  damage={fields=set('op amount subject when'),journal='damage'},
  emit={fields=set('op event subject tag when'),journal='lua_state'},
  echo={fields=set('op copies slots delay match damage knockback element once_per_move status'),journal='echoes'},
  clank_damage={fields=set('op subject'),journal='lua_state'},
  convert={fields=set('op match change'),journal='hit_rules'},
  ['versus-status']={fields=set('op status match change'),journal='hit_rules'},
  -- Technique effects (native timed effects with their own expiry, from a trigger; or passive caps/crit from equip).
  armor={fields=set('op type value frames direction subject when'),journal='fighter_armor'},
  intangible={fields=set('op frames subject when'),journal='fighter_effect'},
  air_jumps={fields=set('op count'),journal='fighter_caps'},
  restrict={fields=set('op forbid'),journal='fighter_caps'},
  -- No sim_commit operation exists for an interrupt window: it is a direct gd.fighter_interrupt on the event frame (not journalled).
  interrupt={fields=set('op frames exits guard restore_jumps subject when'),journal='direct'},
  crit={fields=set('op chance multiplier multiplier_max launch tag min_percent status'),journal='crit'},
  crit_next={fields=set('op count multiplier subject when'),journal='crit'},
  -- Apply a status to the nearest OTHER opponent of the victim (Conductor's Shock chain). Lua-side target choice from the frame's positions.
  chain_status={fields=set('op status duration amount max refresh when'),journal='lua_state'},
 }
 -- Journal operations the native checkpoint admits (gw_script_sim_state.inc). `key` is the entity field
 -- ('port' 1..6 for the first four, 'entity' 1..12 for the capability writers). Bounds mirror the C
 -- parser and the gd.* setter each one calls, so a rule is refused here before the native refusal.
 R.operations={
  fighter_mod={key='port'},damage={key='port',fields={value={0,999}}},hit_rules={key='port'},echoes={key='port'},
  fighter_caps={key='entity',fields={values={table=true}}},
  fighter_effect={key='entity',fields={effect={enum={'intangible','invincible','metal','size'}},value={0,4},frames={0,3600,integer=true}}},
  fighter_armour={key='entity',fields={damage={0,1000},knockback={0,1000}}},
  fighter_armor={key='entity',fields={type={enum={'knockback','damage_threshold','knockback_threshold','super','hit_count','damage_pool'}},value={0,1000},frames={0,36000,integer=true},state={-1,65535,integer=true},from={-1,100000},to={-1,100000},direction={enum={'any','front','back'}},clear={boolean=true}}},
  shock={key='entity',fields={frames={1,3600,integer=true},charges={1,8,integer=true},hitstun={1,4},bonus={0,120,integer=true},stack={boolean=true},clear={boolean=true}}},
  crit={key='entity',fields={slot={enum={'default','jab','dash_attack','tilt','smash','aerial','grab','throw','special','projectile'}},begin={boolean=true},release={boolean=true},chance={0,1},multiplier={1,16},multiplier_max={1,16},launch={1,4},min_percent={0,999},force={0,255,integer=true}}},
  timed_status={key='entity',fields={channel={1,4,integer=true},value={-100000,100000},frames={0,3600,integer=true}}},
 }
 function R.field_set(op) return assert(R.effects[op],'unsupported effect (no connecting-hit mutation)').fields end
 function R.journal(op) return assert(R.effects[op],'unsupported effect').journal end
 -- Validates one journal operation table the way the native parser will. Returns the table.
 function R.operation(o)
  assert(type(o)=='table','operation table required');local d=assert(R.operations[o.op],'unsupported journal operation')
  local limit=d.key=='entity' and 12 or 6;local at=o[d.key]
  assert(type(at)=='number' and at%1==0 and at>=1 and at<=limit,d.key..' 1..'..limit..' required')
  for k,v in pairs(o) do if k~='op' and k~=d.key and not (d.key=='port' and (k=='values' or k=='rules' or k=='status_bits' or k=='sub' or k=='value' or k=='copies' or k=='slots')) then
   local f=d.fields and d.fields[k];assert(f,'unknown field '..tostring(k)..' for '..o.op)
   if f.enum then local ok;for _,n in ipairs(f.enum) do ok=ok or n==v end;assert(ok,'invalid '..k)
   elseif f.boolean then assert(type(v)=='boolean','boolean required for '..k)
   elseif f.table then assert(type(v)=='table','table required for '..k)
   else assert(type(v)=='number' and v==v and v>=f[1] and v<=f[2] and (not f.integer or v%1==0),k..' out of bounds') end
  end end
  return o
 end
 return R
end
