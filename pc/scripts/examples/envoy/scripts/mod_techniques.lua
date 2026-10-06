-- Technique and crit modifiers for ordinary drives: a small set of prefixes and suffixes that reward playing Melee well.
-- Early drives stay one plain effect, so every record here carries `min_depth` (effective depth) and never rolls before it;
-- the keystones carry the loud versions. Each record is one technique (or crit) trigger and one plain effect.
--
-- Opponents roll these too (same pool, same weights). A retail CPU does not perform most techniques, so an opponent's roll of
-- a record whose trigger mod_skill marks 'dead' is inert for now: it costs the opponent budget it cannot use. Foe rolls
-- therefore weight dead-trigger records down (foe_roll.lua) instead of scripting the opponent.
--   live  : opponents fire it anyway (hits, knockdowns, crits)
--   maybe : retail AI does it at some levels (unmeasured)
--   dead  : retail AI never does it
return function(D)
 local X={}
 local function tiers3(a,b,c) return {a,b,c} end
 -- id, label, affix, trigger, conditions, effects, three tier tables, words (the text the schema validates), look, hue, tags, min_depth, note
 local function rec(id,label,affix,trigger,conditions,effects,tiers,text,look,hue,tags,min_depth,note,weight)
  return {id=id,label=label,kind='normal',tags=tags,trigger=trigger,conditions=conditions,effects=effects,tiers=tiers,stacking={max=1},text=text,
   visual={look=look,hue=hue,strength=.35,priority=10},affix=affix,group=id,weight=weight or 40,min_depth=min_depth,notes=note}
 end
 local function status(name,duration,amount,max) return {op='status',status=name,subject='self',duration=duration,amount=amount or 1,max=max or 1,refresh='refresh'} end
 function X.records()
  local out={}
  local function add(r) out[#out+1]=r end
  -- ---- technique triggers (suffixes: they happen on an event) ------------------------------------------------------
  add(rec('clean_landing','Clean Landing','suffix','lcancel_hit',{},{status('haste','$duration')},
   tiers3({duration=90},{duration=120},{duration=150}),'L-cancel after a hit: Haste for {duration} frames.','haste',.6,{'hasted','aerial','technique'},5,
   'Technique: hit-confirmed L-cancel (verified). Haste is earned: blue afterimages for its length.'))
  add(rec('wave_edge','Wave','suffix','wavedash',{},{{op='crit_next',count=1,multiplier='$mult'}},
   tiers3({mult=1.4},{mult=1.6},{mult=1.8}),'Wavedash: your next hit crits (x{mult}).','momentum',.1,{'critical','technique'},6,
   'Technique: wavedash (verified). The forced crit lasts until a hit lands.'))
  add(rec('shield_stance','Stance','suffix','perfect_shield',{},{{op='armor',type='hit_count',value=1,frames='$frames'}},
   tiers3({frames=90},{frames=120},{frames=150}),'Perfect shield: armour absorbs the next hit for up to {frames} frames.','guarded',.5,{'guarded','technique'},7,
   'Technique: perfect shield (verified). Armour only, no status, so no afterimage.'))
  add(rec('tech_guard','Tech','suffix','tech',{},{status('guarded','$duration')},
   tiers3({duration=60},{duration=90},{duration=120}),'Tech: Guarded for {duration} frames.','guarded',.5,{'guarded','technique'},5,
   'Technique: tech (verified; tech directions are partly measured). Guarded is earned: teal afterimages.'))
  add(rec('combo_surge','Combo','suffix','combo',{{combo_at_least=3}},{status('haste','$duration')},
   tiers3({duration=45},{duration=60},{duration=75}),'3rd hit of a combo: Haste for {duration} frames.','haste',.98,{'hasted','technique'},6,
   'Technique: combo count (verified). Opponents fire it too: it needs only hits.'))
  add(rec('combo_finish','Finish','suffix','combo_end',{{combo_at_least=3}},{{op='heal',amount='$heal',subject='self'}},
   tiers3({heal=3},{heal=4},{heal=5}),'Finish a combo of 3+ hits: heal {heal} damage points.','guarded',.42,{'healing','technique'},6,
   'Technique: combo end (verified).'))
  add(rec('retaliation','Retaliation','suffix','armor',{{armor_result='absorbed'}},{{op='crit_next',count=1,multiplier='$mult'}},
   tiers3({mult=1.4},{mult=1.6},{mult=1.8}),'When armour absorbs a hit: your next hit crits (x{mult}).','burn',.04,{'critical','guarded'},8,
   'Armour source (Stance, a Wavedasher or Juggernaut keystone). Armour events are verified.',30))
  -- ---- crit families (prefixes: passive) -----------------------------------------------------------------------------
  add(rec('keen','Keen','prefix','equip',{},{{op='crit',chance='$chance'}},
   tiers3({chance=.05},{chance=.0625},{chance=.075}),'Your hits crit {chance%} of the time.','burn',.08,{'critical','damage'},4,
   'Crit chance. The engine default is no crits; every crit starts from a rule like this one.',60))
  add(rec('brutal','Brutal','prefix','equip',{},{{op='crit',chance=.03},{op='crit',multiplier='$mult'}},
   tiers3({mult=1.25},{mult=1.3125},{mult=1.375}),'Your hits crit 3% of the time; crits deal x{mult} extra.','burn',.0,{'critical','damage'},6,
   'Crit multiplier (a gain on top of the base) with a small chance of its own, so it is never a dead roll; other crit chances add to it.'))
  add(rec('ruthless','Ruthless','prefix','equip',{},{{op='crit',tag='aerial',chance='$chance'}},
   tiers3({chance=.15},{chance=.1875},{chance=.225}),'Your aerial hits crit {chance%} of the time.','momentum',.1,{'critical','aerial'},6,
   'Per-move-tag crit (aerial slot).'))
  add(rec('finishing','Finishing','prefix','equip',{},{{op='crit',chance='$chance',min_percent=100}},
   tiers3({chance=.25},{chance=.3125},{chance=.375}),'Hits on a target above 100% crit {chance%} of the time.','shock',.0,{'critical','damage'},7,
   'Conditional crit: the percent floor applies to all of this fighter\'s crits (the engine has one floor per fighter).'))
  add(rec('critical_flow','Critical Flow','suffix','crit',{},{status('haste','$duration')},
   tiers3({duration=45},{duration=60},{duration=75}),'Land a crit: Haste for {duration} frames.','haste',.55,{'hasted','critical'},6,
   'Crit trigger. Not a technique: no earned afterimage. Needs a crit chance from elsewhere.'))
  return out
 end
 return X
end
