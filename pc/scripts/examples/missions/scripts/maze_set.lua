-- Authored data recipes, not generated art. Coordinates are local game units.
return function()
  local exits={
    {side='left',slot=1,traversal='walk'}, {side='right',slot=1,traversal='walk'},
    {side='up',slot=1,traversal='climb'}, {side='down',slot=1,traversal='drop'}}
  local platforms={{x1=29,x2=61,y=20},{x1=69,x2=101,y=40},
    {x1=49,x2=81,y=60},{x1=49,x2=81,y=80},{x1=49,x2=81,y=100}}
  return {version=1,size={w=130,h=104},door={width=24,height=32},
    exits=exits,platforms=platforms,spawn={x=20,y=8},
    -- Potential exits; any unused potential slot is sealed at assembly.
    templates={
      {id='gallery',sides='LRUD',tags={'theme:gallery'}},
      {id='vault',sides='LRUD',tags={'theme:vault'}},
      {id='switchback',sides='LRUD',tags={'theme:switchback'}},
      {id='well',sides='LRUD',tags={'theme:well'}},
      {id='corridor',sides='LR',tags={'theme:kit'}},
      {id='corner_up',sides='RU',tags={'theme:kit'}},
      {id='corner_down',sides='RD',tags={'theme:kit'}},
      {id='t_junction',sides='LRU',tags={'theme:kit'}},
      {id='platform_room',sides='LRUD',tags={'theme:kit'}},
      {id='shaft',sides='UD',tags={'theme:kit'}},
      {id='drop',sides='RD',tags={'drop','theme:kit'}},
      {id='reward_end',sides='LRUD',tags={'secret','reward'}},
      {id='start',sides='LRUD',tags={'start'}},
      {id='goal',sides='LRUD',tags={'goal'}},
      {id='boss_room',sides='LRUD',tags={'boss'}}},
    enemy_kinds={'goomba','koopa'},enemy_slots={{x=32,y=8},{x=88,y=8},{x=104,y=8}},
    -- Render-only slabs: collision is explicit, so kit floor sidecar pass-through
    -- flags cannot accidentally open a sealed roof/floor.
    part='bf_floor_4m',
    -- One hand-authored shell shared by the starter recipes. The generator
    -- translates these lines; only doorway seals are added procedurally.
    shell={
      {'floor',0,0,53,0},{'ceiling',53,0,0,0},
      {'floor',77,0,130,0},{'ceiling',130,0,77,0},
      {'floor',0,104,53,104},{'ceiling',53,104,0,104},
      {'floor',77,104,130,104},{'ceiling',130,104,77,104},
      {'left_wall',0,32,0,104},{'right_wall',0,104,0,32},
      {'left_wall',130,32,130,104},{'right_wall',130,104,130,32}},
    slabs={{x1=0,x2=53,y=0},{x1=77,x2=130,y=0}}}
end
