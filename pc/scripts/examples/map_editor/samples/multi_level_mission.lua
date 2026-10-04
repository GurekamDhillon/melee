-- Multi-level mission: climb two ramps to a goal at the top. Load it on Final Destination:
--   copy this file to scripts-data/map_editor_main/multi_level_mission.lua, then in the console
--   map play multi_level_mission.lua
-- Every part is authored at the editor's 2x scale (52 units wide). Heights: the start floor is y=26, the
-- middle floor y=52 and the top floor y=78; each ramp rises 26 over 52 units (about 27 degrees).
-- Flow: wave 1 (a goomba) guards the start; a trigger zone where the first ramp ends spawns wave 2
-- (a koopa and a goomba) on the middle floor; the checkpoint sits on the middle floor; crossing x=130
-- starts wave 3 (a redead) on the top floor; the goal opens once every enemy is gone. Two message
-- triggers talk to the player. It was completed once in the engine with scripted pad input (walk right,
-- attack when an enemy is within reach: 11.8 s, no deaths); nothing about how it looks was checked.
-- Change it with the mission tool (key 6) in the editor.
return {version=2,units=6.5,
mission={
  start={x=-130,y=40},
  enemies={
    {kind="goomba",x=-90,y=36,wave=1},
    {kind="koopa",x=90,y=62,wave=2},
    {kind="goomba",x=65,y=62,wave=2},
    {kind="redead",x=190,y=88,wave=3},
  },
  waves={{wave=3,x=130,dir=1}},
  triggers={
    {x=20,y=70,w=24,h=60,action="wave",wave=2},
    {x=-130,y=60,w=40,h=70,action="message",text="Climb to the top"},
    {x=170,y=100,w=30,h=70,action="message",text="Nearly there"},
  },
  checkpoints={{x=70,y=80,w=20,h=60}},
  goal={x=190,y=105,w=24,h=60},
  objective={type="defeat_then_goal",time=240,lives=3},
},
parts={
  {part="bf_floor_4m",x=-130,y=26,z=0,rot=0,collision=true,floor_flags=3,scale=2},
  {part="bf_floor_4m",x=-78,y=26,z=0,rot=0,collision=true,floor_flags=3,scale=2},
  {part="bf_ramp_4m_rise2m",x=-26,y=26,z=0,rot=0,collision=true,floor_flags=1,scale=2},
  {part="bf_floor_4m",x=26,y=52,z=0,rot=0,collision=true,floor_flags=3,scale=2},
  {part="bf_floor_4m",x=78,y=52,z=0,rot=0,collision=true,floor_flags=3,scale=2},
  {part="bf_ramp_4m_rise2m",x=130,y=52,z=0,rot=0,collision=true,floor_flags=1,scale=2},
  {part="bf_floor_4m",x=182,y=78,z=0,rot=0,collision=true,floor_flags=3,scale=2},
}}
