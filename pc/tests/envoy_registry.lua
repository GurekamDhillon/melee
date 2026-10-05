local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D);D.mod_budget=T.module('mod_budget',D)
T.test('schema and budget cover exactly the registered effects',function()
 for op,d in pairs(D.mod_registry.effects) do
  assert(D.mod_budget.effect_ops[op],'budget misses '..op);assert(d.fields.op,'every effect declares its op field')
  assert(({lua_state=1,fighter_mod=1,damage=1,echoes=1,hit_rules=1,fighter_armor=1,fighter_effect=1,fighter_caps=1,crit=1,direct=1})[d.journal] and (d.journal=='lua_state' or d.journal=='direct' or D.mod_registry.operations[d.journal]),'journal route '..op)
 end
 T.refuses(function() D.mod_registry.field_set('shock') end)
end)
T.test('schema rejects fields the registry does not list',function()
 local m={id='probe_rule',label='Probe',text='x',kind='normal',tags={},tiers={{n=1}},trigger='equip',conditions={},effects={{op='value',key='run_speed',value=1.1,typo=1}},stacking={max=1},visual={look='burn',hue=0,strength=0},families={'speed'}}
 T.refuses(function() D.mod_schema.validate(m) end)
 m.effects[1].typo=nil;assert(D.mod_schema.validate(m))
end)
T.test('journal operations are bounded like the native parser',function()
 local R=D.mod_registry
 assert(R.operation({op='fighter_caps',entity=7,values={air_jumps=3}}))
 assert(R.operation({op='fighter_effect',entity=12,effect='metal',value=1,frames=90}))
 assert(R.operation({op='fighter_armor',entity=1,type='super',value=2,frames=120,direction='front'}))
 assert(R.operation({op='timed_status',entity=1,channel=4,value=-5,frames=0}))
 assert(R.operation({op='damage',port=1,value=10}))
 for _,bad in ipairs{{op='fighter_caps',entity=13},{op='fighter_caps',port=1},{op='fighter_effect',entity=1,effect='typo',value=1,frames=1},
  {op='fighter_effect',entity=1,effect='metal',value=1,frames=3601},{op='fighter_armor',entity=1,type='super',value=-1},{op='timed_status',entity=1,channel=5,value=1,frames=1},
  {op='fighter_armour',entity=1,damage=1001},{op='nope',entity=1},{op='fighter_armor',entity=1,type='super',value=2,extra=1}} do
  T.refuses(function() R.operation(bad) end)
 end
end)
T.done()
