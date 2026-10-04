-- Source-only orchestration check. No GPU, native registry, or rewind claim.
local root=assert(arg[1],'demo folder required')
local calls,keys,options,commands={}, {}, {}, {}
local ready=false
local gd={
  command=function(name,fn) commands[name]=fn end,
  match=function() return {active=true} end,
  player=function() return {} end,
  afterimage_add=function(port,o) assert(port==1 and o.intensity==0);options=o;calls.add=true;return 1 end,
  tracer_add=function(o) assert(o.port==1 and o.intensity==0);options=o;calls.add=true;return 1 end,
  warm=function(o) assert(calls.add);assert(o.tracers==true or o.fighters[1]==1);calls.warm=true;return 2 end,
  warm_done=function(h) assert(h==2);return ready end,
  warm_release=function(h) assert(h==2);calls.release=true end,
  key_pressed=function(k) return keys[k] or false end,
  safe_area=function() return {x=0,y=0,w=900,h=600} end,
  fill=function() end,text=function() end,log=function() end,
}
local function set(h,o) assert(h==1 and ready and calls.release);for k,v in pairs(o) do options[k]=v end;return true end
gd.afterimage_set=set;gd.tracer_set=set
local env=setmetatable({gd=gd},{__index=_G})
assert(loadfile(root..'/scripts/main.lua','t',env))()
env.on_frame();assert(calls.warm)
env.on_tick();assert(options.intensity==0,'render enabled before warm readiness')
ready=true;env.on_tick();assert(options.intensity==1,'exaggerated integrator default was lost')
for _,key in ipairs({'C','F','I','S','W','L','A','T','G'}) do keys[key]=true;env.on_tick();keys[key]=false end
env.on_draw();assert(commands.demo_state);commands.demo_state('check')
env.on_unload();env.on_scene();assert(env.on_frame)
print('motion demo warm ordering and live controls PASS: '..root)
