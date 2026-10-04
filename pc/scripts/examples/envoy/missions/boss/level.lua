-- Same match-start CPU, moved here by Envoy. No fighter is created mid-match.
return {version=2,units=6.5,parts={},
  fighters={[2]='fight'},
  camera={left=-185,right=185,top=140,bottom=-30},
  blast={left=-220,right=220,top=190,bottom=-95},
  spawn={[0]={x=-100,y=12},[1]={x=60,y=12},[4]={x=-100,y=12},[5]={x=60,y=12}},
  lines={
    {x1=-170,y1=0,x2=170,y2=0,kind='floor',ledges=true,draw=true},
    {x1=-60,y1=40,x2=0,y2=40,kind='floor',passthrough=true,ledges=false,draw=true},
    {x1=35,y1=65,x2=95,y2=65,kind='floor',passthrough=true,ledges=false,draw=true},
  },
}
