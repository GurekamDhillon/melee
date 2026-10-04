return {
  start={x=-185,y=52},
  enemies={
    {kind='goomba',x=-85,y=8,wave=1},
    {kind='koopa',x=-25,y=8,wave=1},
    {kind='topi',x=55,y=8,wave=1},
    {kind='redead',x=100,y=8,wave=2},
    {kind='like_like',x=145,y=8,wave=2},
    {kind='octorok',x=195,y=8,wave=2},
    {kind='polar_bear',x=250,y=8,wave=2},
  },
  checkpoints={{x=35,y=20,w=24,h=80},{x=190,y=20,w=24,h=80}},
  goal={x=270,y=20,w=24,h=80},
  objective={type='defeat_then_goal',time=300,lives=3},
  triggers={{x=-185,y=52,w=30,h=60,action='message',text='Safe start pocket. Drop right into the fight, collect drives, reach the exit.'}},
}
