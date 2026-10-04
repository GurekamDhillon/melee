-- Pure rules: all progression and fighter/passive numbers are tuned together here.
return function(D)
  local G=D.genetics
  local C={stats=G.stats,colours={red='power',green='speed',blue='guard',yellow='jump'}}
  C.tuning={
    -- Threshold L = 20L + 5L(L-1): C drives reach L1 in one pickup,
    -- L5 in ten, L10 in 33, L25 in 175. Effects approach their caps
    -- quickly (L1 20%, L5 56%, L10 71%, L25 86% of the stat bonus).
    level_base=20,level_growth=10,level_cap=99,point_cap=50490,
    drive_points=20,drive_lifetime=900,grade_gain={E=.5,D=.75,C=1,B=1.25,A=1.5,S=2},
    effect_half=4,damage_cap=.10,speed_cap=.20,guard_cap=.15,shield_cap=.20,
    pickup_base=17.5,drop_base=.80,
    stat_names={power='Power',speed='Speed',guard='Guard',jump='Jump'},
    -- Desired height caps await native ground/air jump attribute modifiers.
    jump_height_cap=.15,air_jump_height_cap=.15,jump_air_cap=.08,
    passive_power=.02,passive_speed=.03,passive_guard=.03,passive_jump=.02,
    passive_balanced=.01,passive_balanced_air=.02,
    passive_names={power='Heavy Hands',speed='Tailwind',guard='Soft Landing',jump='Skybound',balanced='Sure Footed'},
    white_chance=.02,primary_chance=.80,lifespan=6,carry_fraction=.10,
    depth_drive_points={20,25,30,35},
    levelup_frames=48,hub_station_radius=22,
    retail={default_mode='classic',difficulty=2,stocks=3,reward_every=1,
      reward_points={120,180,240},final_multiplier=3,white_chance=.04,
      physical_drops=false,physical_drop_chance=1,ngplus=true,loop_scale=.15,budget_width=.15,
      spread_width=4,stat_floor=0,stat_ceiling=99,team_exponent=.5,starter_budget=4,
      effect_half=2,damage_cap=.24,speed_cap=.30,guard_cap=.24,shield_cap=.35,
      knockback_cap=.20,jump_height_cap=.30,air_jump_height_cap=.25,jump_air_cap=.15,
      hold_ticks=1800,tag_ticks=240,reward_animation_frames=48,guard_flash_frames=12,change_ai=false},
    juice={glow=true,pool=true,sparkles=true,highlight=true,pop_trail=true,
      collect_burst=true,sound=true,hud_flash=true,blink=true},
    visuals={recolour=true,pulse=true,shiny=true,pulse_frames=12},
  }
  C.types={young=true,egg=true,power=true,speed=true,guard=true,jump=true,balanced=true}
  local function integer(n,cap) return type(n)=='number' and n==n and n>=0 and n<=cap and n%1==0 end
  function C.threshold(level)
    local t=C.tuning;return t.level_base*level+t.level_growth*level*(level-1)/2
  end
  function C.level(points)
    local n=0;while n<C.tuning.level_cap and points>=C.threshold(n+1) do n=n+1 end;return n
  end
  function C.progress(c,key)
    local s=key and c.stats[key] or c;local p=s.points-(s.carry or 0)
    if s.level>=C.tuning.level_cap then return 1,0,0 end
    local current=p-C.threshold(s.level);local needed=C.threshold(s.level+1)-C.threshold(s.level)
    return current/needed,current,needed
  end
  function C.new(dna,rng)
    dna=dna or G.new();local e=G.express(dna,rng);local own={}
    for _,k in ipairs(G.traits) do own[k]={dna[k][1],dna[k][2]} end
    local c={id=1,dna=own,stats={},age=0,type='young',white_drives=0,lives=0,life_unknown=false,
      colour=e.colour,two_tone=e.two_tone,shiny=e.shiny}
    for _,k in ipairs(C.stats) do c.stats[k]={grade=e[k],base_grade=e[k],points=0,carry=0,life_gain=0,level=0} end
    return c
  end
  function C.validate(c)
    assert(type(c)=='table','companion required');G.validate(c.dna)
    assert(integer(c.id,1000000) and c.id>=1,'invalid companion id')
    assert(integer(c.age,C.tuning.lifespan) and C.types[c.type],'invalid age/type')
    assert(integer(c.white_drives,1000000) and integer(c.lives,1000000) and type(c.life_unknown)=='boolean','invalid life counters')
    assert(G.colours[c.colour] and type(c.two_tone)=='boolean' and type(c.shiny)=='boolean','invalid cosmetics')
    assert(type(c.stats)=='table','stats required')
    for k in pairs(c.stats) do assert(k=='power' or k=='speed' or k=='guard' or k=='jump','unknown stat') end
    for _,k in ipairs(C.stats) do
      local s=c.stats[k];assert(type(s)=='table' and G.rank[s.grade] and G.rank[s.base_grade],'invalid grade')
      assert(c.dna[k][1]==s.base_grade or c.dna[k][2]==s.base_grade,'inherited grade absent from DNA')
      assert(G.rank[s.grade]>=G.rank[s.base_grade],'temporary grade below inherited grade')
      assert(integer(s.points,C.tuning.point_cap) and integer(s.carry,C.tuning.point_cap) and s.carry<=s.points,'invalid points/carry')
      assert(integer(s.life_gain,C.tuning.point_cap) and s.life_gain<=s.points-s.carry,'invalid life gain')
      assert(s.level==C.level(s.points-s.carry),'level disagrees with growth points')
    end
    return c
  end
  function C.feed(c,colour,points,rng)
    C.validate(c);assert(integer(points,1000000),'points must be an integer 0..1000000')
    if colour=='white' then
      assert(c.white_drives+points<=1000000,'white drive count full')
      local candidates={};local lowest=7
      for _,k in ipairs(C.stats) do local rank=G.rank[c.stats[k].grade]
        if rank<6 then if rank<lowest then candidates={};lowest=rank end;if rank==lowest then candidates[#candidates+1]=k end end
      end
      -- Stable stat order by default; injected randomness can select tied lows.
      local k=#candidates>0 and candidates[rng and (math.floor(G.random(rng)*#candidates)+1) or 1] or nil
      c.white_drives=c.white_drives+points
      if k and points>0 then c.stats[k].grade=G.grades[math.min(6,G.rank[c.stats[k].grade]+points)] end
      return points,k
    end
    local k=assert(C.colours[colour],'unknown drive colour');local s=c.stats[k];local before=s.points
    s.points=math.min(C.tuning.point_cap,s.points+math.floor(points*C.tuning.grade_gain[s.grade]))
    s.life_gain=s.life_gain+s.points-before;s.level=C.level(s.points-s.carry);return s.points-before
  end
  function C.passive(c)
    local t=C.tuning;local p={
      power={name=t.passive_names.power,description='A small damage boost.',effects={damage_dealt=t.passive_power}},
      speed={name=t.passive_names.speed,description='Extra movement speed.',effects={speed=t.passive_speed}},
      guard={name=t.passive_names.guard,description='Less damage taken.',effects={damage_taken=-t.passive_guard}},
      jump={name=t.passive_names.jump,description='Faster air movement.',effects={jump_air_bonus=t.passive_jump}},
      balanced={name=t.passive_names.balanced,description='Faster ground and air movement.',effects={speed=t.passive_balanced,air_speed_bonus=t.passive_balanced_air-t.passive_balanced}},
    };return p[c.type]
  end
  function C.effects(c)
    C.validate(c);local t=C.tuning
    local function bonus(k,cap) local l=c.stats[k].level;return cap*l/(l+t.effect_half) end
    local e={damage_dealt=1+bonus('power',t.damage_cap),speed=1+bonus('speed',t.speed_cap),
      damage_taken=1-bonus('guard',t.guard_cap),shield_bonus=bonus('guard',t.shield_cap),
      shield_max=1+bonus('guard',t.shield_cap),pickup_radius=t.pickup_base,
      drop_chance=t.drop_base,jump_air_bonus=bonus('jump',t.jump_air_cap),air_speed_bonus=0}
    local p=C.passive(c);if p then for k,v in pairs(p.effects) do e[k]=e[k]+v end end
    e.damage_dealt=math.min(e.damage_dealt,1+t.damage_cap);e.speed=math.min(e.speed,1+t.speed_cap)
    e.damage_taken=math.max(e.damage_taken,1-t.guard_cap)
    e.jump_air_bonus=math.min(e.jump_air_bonus,t.jump_air_cap)
    e.air_speed=math.min(1+t.speed_cap+t.jump_air_cap,e.speed+e.jump_air_bonus+e.air_speed_bonus)
    return e
  end
  function C.retail_effects(c)
    C.validate(c);local t=C.tuning.retail
    local function bonus(k,cap) local l=c.stats[k].level;return cap*l/(l+t.effect_half) end
    local e={damage_dealt=1+bonus('power',t.damage_cap),speed=1+bonus('speed',t.speed_cap),
      damage_taken=1-bonus('guard',t.guard_cap),shield_max=1+bonus('guard',t.shield_cap),
      knockback_taken=1-bonus('guard',t.knockback_cap),
      air_speed=1+bonus('speed',t.speed_cap)+bonus('jump',t.jump_air_cap),
      jump_height=1+bonus('jump',t.jump_height_cap),air_jump_height=1+bonus('jump',t.air_jump_height_cap)}
    local p=C.passive(c)
    if p then
      local v=p.effects;e.damage_dealt=e.damage_dealt+(v.damage_dealt or 0);e.damage_taken=e.damage_taken+(v.damage_taken or 0)
      e.speed=e.speed+(v.speed or 0);e.air_speed=e.air_speed+(v.speed or 0)+(v.air_speed_bonus or 0)+(v.jump_air_bonus or 0)
    end
    e.damage_dealt=math.min(1+t.damage_cap,e.damage_dealt);e.damage_taken=math.max(1-t.guard_cap,e.damage_taken)
    e.speed=math.min(1+t.speed_cap,e.speed);e.air_speed=math.min(1+t.speed_cap+t.jump_air_cap,e.air_speed)
    return e
  end
  function C.reward_effect(c,colour,points)
    if colour=='white' then
      for _,k in ipairs(C.stats) do if G.rank[c.stats[k].grade]<6 then return 'Raise a lowest grade; future drives grow faster' end end
      return 'All grades capped; no further grade gain'
    end
    local before=C.retail_effects(c);local next={};for k,v in pairs(c) do next[k]=v end;next.stats={}
    for k,v in pairs(c.stats) do next.stats[k]={};for n,x in pairs(v) do next.stats[k][n]=x end end
    C.feed(next,colour,points);local after=C.retail_effects(next)
    local function delta(key,words,negative)
      local n=(after[key]-before[key])*100*(negative and -1 or 1)
      if n<.05 then return words..' at cap' end
      return ('+%.1f%% %s'):format(n,words)
    end
    if colour=='red' then return delta('damage_dealt','damage')
    elseif colour=='green' then return delta('speed','run speed')
    elseif colour=='blue' then return delta('damage_taken','damage resistance',true)..' / '..delta('knockback_taken','launch resistance',true)
    else return delta('jump_height','jump height')..' / '..delta('air_jump_height','air jump height') end
  end
  function C.start_life_run(c)
    C.validate(c);assert(c.age<C.tuning.lifespan,'companion life ended')
    if c.type=='egg' then c.type='young' end;c.age=c.age+1;return c.age
  end
  C.start_run=C.start_life_run
  function C.evolve(c)
    C.validate(c);if c.type~='young' and c.type~='egg' then return nil end
    local top=-1;local winner;local tied=false
    for _,k in ipairs(C.stats) do local n=c.stats[k].life_gain
      if n>top then top=n;winner=k;tied=false elseif n==top then tied=true end
    end
    c.type=tied and 'balanced' or winner
    local grade
    if not tied then
      local s=c.stats[winner];grade=G.grades[math.min(6,G.rank[s.base_grade]+1)]
      -- Upgrade allele(s) expressing the inherited grade; preserve the hidden
      -- allele in a mixed pair. Temporary white boosts never become DNA.
      local inherited=s.base_grade
      for i=1,2 do if c.dna[winner][i]==inherited then c.dna[winner][i]=grade end end
      if G.rank[s.grade]<G.rank[grade] then s.grade=grade end;s.base_grade=grade
    end
    return {type=c.type,stat=not tied and winner or nil,grade=not tied and c.stats[winner].grade or nil,inherited_grade=grade,passive=C.passive(c)}
  end
  function C.reincarnate(c)
    C.validate(c);if c.age<C.tuning.lifespan then return nil end
    c.type='egg';c.age=0;c.lives=c.lives+1;c.white_drives=0;c.life_unknown=false
    for _,k in ipairs(C.stats) do local s=c.stats[k]
      s.points=math.floor(s.points*C.tuning.carry_fraction);s.carry=s.points;s.level=0;s.life_gain=0;s.grade=s.base_grade
    end
    C.validate(c);return {type='egg',life=c.lives}
  end
  function C.settle(c,reason)
    assert(reason=='win' or reason=='fail' or reason=='quit','invalid settlement')
    return {evolution=reason=='win' and C.evolve(c) or nil,reincarnation=C.reincarnate(c)}
  end
  return C
end
