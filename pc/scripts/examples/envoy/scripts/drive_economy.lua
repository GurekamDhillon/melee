-- How many drives a run hands out, and what happens when one arrives. Tuning values and pure rules; the run host
-- reads E.tuning (it still owns H.tuning today: drop_chance, battle_drops_max, team_drops_max, offer_count,
-- bonus_offer_count, bag_capacity) and calls E.gain_plan when a drive is gained. Nothing here touches a screen.
return function(D)
 local E={}
 -- What the run hands out TODAY (run_host.lua H.tuning): kept as data so the before/after table is computed, not
 -- remembered. A battle/giant/metal stage drops one at its first trigger and a second with `second_chance`; a team stage
 -- one per opponent up to `team_max`; a bonus or boss stage none on the floor. Every stage clear offers `offers`
 -- (bonus/boss/final `bonus_offers`), the player takes one. The bag holds `bag_capacity`.
 E.before={floor_max=2,second_chance=.25,floor_chance=1,team_max=3,reward_every=1,offers=2,bonus_offers=3,bag_capacity=12,merge=false}
 -- THE PROPOSED CURVE. At most ONE floor drop per stage (a battle stage drops it with `floor_chance`), a team stage
 -- one, rewards only at a bonus stage, the boss and every third stage (fewer, but a pick of three at those: the
 -- run host rolls them at least Magic), and a 4-drive bag with merging (drive_merge.lua).
 E.tuning={floor_max=1,second_chance=0,floor_chance=.7,team_max=1,reward_every=3,offers=3,bonus_offers=3,bag_capacity=4,merge=true}
 -- A representative twelve-stage Classic run, used for the quantity model (the real kinds come from retail).
 E.kinds={'battle','battle','team','battle','bonus','battle','giant','battle','metal','battle','bonus','boss'}
 E.team_opponents=2
 -- Expected drives gained at one stage: floor drops, and whether a reward is shown (the player takes one drive).
 function E.stage(policy,kind,stage)
  local floor=0
  if kind=='team' then floor=math.min(policy.team_max,E.team_opponents)
  elseif kind=='battle' or kind=='giant' or kind=='metal' then
   floor=policy.floor_chance*(1+(policy.floor_max>1 and policy.second_chance or 0))
  end
  local reward=(kind=='bonus' or kind=='boss') or (policy.reward_every<=1) or ((stage+1)%policy.reward_every==0)
  return floor,reward and 1 or 0
 end
 -- Cumulative expected drives gained after each stage (starter included), before merging. Returns an array.
 function E.curve(policy,kinds)
  kinds=kinds or E.kinds;local total,out=1,{}
  for i,kind in ipairs(kinds) do local f,r=E.stage(policy,kind,i-1);total=total+f+r;out[i]=total end
  return out
 end
 -- The rarity of the i-th drive offered at a stage reward (the host passes it as the forced rarity): fewer rewards,
 -- each worth choosing between. The first two are Magic, the third Rare once rares roll at this depth. Affix count
 -- is still held by the depth band, so an early reward is a simple drive too (a Magic at depth 0 is one modifier).
 function E.reward_rarity(context,i)
  if i>=3 and D.mod_progression.rarity_allowed(context,'rare') then return 'rare' end
  return 'magic'
 end
 -- What to do with a gained drive, in this order (the run host calls this and acts on the answer):
 --   {action='merge',index=,info=}   it improves a held drive (list index into `held`) and is consumed;
 --   {action='bag'}                  there is room in the bag;
 --   {action='choose'}               the bag is full and nothing merges: ask "Keep it (replace one) or leave it?"
 -- `held` is every drive the player has, equipped first then the bag; `bag_count` how many are in the bag.
 function E.gain_plan(held,bag_count,gained,loot,policy)
  policy=policy or E.tuning
  if policy.merge then
   local at=D.drive_merge.find_target(held,gained,loot)
   if at then local _,info=D.drive_merge.merge(held[at],gained,loot);return {action='merge',index=at,info=info} end
  end
  if bag_count<policy.bag_capacity then return {action='bag'} end
  return {action='choose'}
 end
 return E
end
