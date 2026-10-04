local T=dofile('melee/pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_budget','mod_pool','mod_engine','drive_loot','drive_bag','foe_roll'})do D[n]=T.module(n,D)end
local R=D.foe_roll.new(D.mod_pool);local P=D.mod_progression
print('depth loop tier slots player_min player_max foe_min foe_max ratio_min ratio_max seed_player seed_foe strength_player strength_foe hits_player hits_foe')
for loop=0,3 do for depth=0,12 do
 local c=P.context(depth,loop);local low,high,fl,fh,rl,rh=math.huge,0,math.huge,0,math.huge,0;local best,quality
 for sample=0,3 do
  local ps=401+depth+13*loop+sample*104729;local fs=500+depth+13*loop+sample*104729
  local player=R:sample(ps,c);local foe=R:roll(player.strength,fs,depth,2,c)
  low=math.min(low,player.strength);high=math.max(high,player.strength);fl=math.min(fl,foe.strength);fh=math.max(fh,foe.strength)
  local ratio=foe.strength/foe.target;rl=math.min(rl,ratio);rh=math.max(rh,ratio)
  local a,b=R:exchange(player.build,foe.build);local q=a and b and math.abs(math.log(a/7))+math.abs(math.log(b/7)) or math.huge
  if not quality or q<quality then best={ps,fs,player.strength,foe.strength,a,b};quality=q end
 end
 print(('%d %d %d %d %.3f %.3f %.3f %.3f %.4f %.4f %d %d %.3f %.3f %s %s'):format(depth,loop,P.tier(c),P.slots(c),low,high,fl,fh,rl,rh,best[1],best[2],best[3],best[4],tostring(best[5]),tostring(best[6])))
end end
local c=P.context(12,3);local player=R:sample(43,c)
for _,role in ipairs({'normal','boss','finalboss'})do
 local foe=R:roll(player.strength,143,12,2,c,role);local a,b=R:exchange(player.build,foe.build)
 print(('role %s player %.3f target %.3f foe %.3f ratio %.4f hits %s/%s'):format(role,player.strength,foe.target,foe.strength,foe.strength/foe.target,tostring(a),tostring(b)))
end
