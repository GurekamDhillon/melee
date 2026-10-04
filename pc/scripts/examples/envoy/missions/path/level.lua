-- Original collision-only hand-built route. No disc assets or external models.
return {version=2,units=6.5,parts={},starting_percent=0,
  camera={left=-240,right=330,top=160,bottom=-30},
  blast={left=-290,right=385,top=230,bottom=-120},
  spawn={[0]={x=-185,y=52},[1]={x=-185,y=52},[4]={x=-185,y=52},[5]={x=-185,y=52}},
  lines={
    -- Ground walkers have no gap or unguarded edge to wander off.
    {x1=-220,y1=0,x2=310,y2=0,kind='floor',ledges=false,draw=true},
    {x1=-220,y1=0,x2=-220,y2=120,kind='left_wall',draw=true},
    {x1=310,y1=120,x2=310,y2=0,kind='right_wall',draw=true},
    -- Elevated start pocket; exit right above the short ground barrier.
    {x1=-215,y1=40,x2=-145,y2=40,kind='floor',ledges=false,draw=true},
    {x1=-130,y1=0,x2=-130,y2=30,kind='left_wall',draw=true},
    {x1=-130,y1=30,x2=-130,y2=0,kind='right_wall',draw=true},
    {x1=-60,y1=40,x2=0,y2=40,kind='floor',passthrough=true,ledges=false,draw=true},
    {x1=55,y1=60,x2=115,y2=60,kind='floor',passthrough=true,ledges=false,draw=true},
    {x1=155,y1=40,x2=215,y2=40,kind='floor',passthrough=true,ledges=false,draw=true},
  },
  markers={{name='upper_route',x=75,y=88,cleared_waves={1},frames=300}},
}
