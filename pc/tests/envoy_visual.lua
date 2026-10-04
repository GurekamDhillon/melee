local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local V=T.module('visual')
local function rig() local x={adds=0,removes=0};local g={post_add=function(path,o) assert(path=='shaders/drive-pulse.wgsl');x.adds=x.adds+1;x.options=o;return x.adds end,post_remove=function() x.removes=x.removes+1 end,post_set=function() return true end};return g,x end
T.test('pulse expires at 12 logic ticks',function() local g,x=rig();local v=V.new(g);assert(v:pickup('S','red'));for i=1,11 do v:tick() end;assert(x.removes==0);v:tick();assert(x.removes==1);v:tick();assert(x.removes==1) end)
T.test('replacement and idempotent clear',function() local g,x=rig();local v=V.new(g);v:pickup('C','blue');v:pickup('A','green');assert(x.adds==2 and x.removes==1);v:clear();v:clear();assert(x.removes==2) end)
T.test('silent failures and disabled',function() local g,x=rig();g.post_add=function() error('bad') end;assert(not V.new(g):pickup('A','red'));assert(not V.new(g,{visuals={pulse=false}}):pickup('S','red'));assert(x.adds==0) end)
T.test('duration clamps 12 to 18',function() local g,x=rig();local v=V.new(g,{visuals={pulse_frames=99}});v:pickup('E','white');for i=1,18 do v:tick() end;assert(x.removes==1) end)
T.test('one sample world pass and capped intensity',function() local g,x=rig();V.new(g):pickup('S','red');assert(x.options.stage=='world' and x.options.params.strength<=.08) end)
T.done()
