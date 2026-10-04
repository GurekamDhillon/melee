-- Authored garden with explicit visible collision; optional kit models live in hub.lua.
return {version=2,units=6.5,starting_percent=0,parts={},
 camera={left=-245,right=245,top=150,bottom=-30},blast={left=-280,right=280,top=220,bottom=-100},
 spawn={[0]={x=-180,y=10},[1]={x=-180,y=10}},
 lines={{x1=-230,y1=0,x2=230,y2=0,kind='floor',draw=true,ledges=false},
 {x1=-230,y1=0,x2=-230,y2=100,kind='left_wall',draw=true},
 {x1=230,y1=100,x2=230,y2=0,kind='right_wall',draw=true}},
 markers={{name='exit',x=-180,y=10},{name='fighter',x=-90,y=10},{name='companion',x=0,y=10},{name='records',x=90,y=10},{name='nest',x=180,y=10}}}
