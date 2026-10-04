-- First mission: two waves, a checkpoint and a goal on a flat kit strip. Load it on Final Destination:
--   copy this file to scripts-data/map_editor_main/first_mission.lua, then in the console
--   map play first_mission.lua
-- Positions are world units. The strip is eleven unscaled bf_floor_4m (26 units each); P1 starts on the
-- left, wave 1 (goomba, koopa) is between the start and the checkpoint, wave 2 (redead) is further on,
-- and the goal opens after wave 2 is defeated. It was run to `mission: complete` in the engine with scripted pad input.
return {version=2,units=6.5,
mission={
  start={x=-120,y=40},
  enemies={
    {kind="goomba",x=-50,y=36,wave=1},
    {kind="koopa",x=-20,y=36,wave=1},
    {kind="redead",x=60,y=36,wave=2},
  },
  goal={x=125,y=45,w=24,h=60},
  checkpoints={{x=20,y=45,w=16,h=60}},
  objective={type="defeat_then_goal",time=120,lives=3},
},
parts={
  {part="bf_floor_4m",x=-130,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=-104,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=-78,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=-52,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=-26,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=0,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=26,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=52,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=78,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=104,y=30,z=0,rot=0,collision=true,floor_flags=3},
  {part="bf_floor_4m",x=130,y=30,z=0,rot=0,collision=true,floor_flags=3},
}}
