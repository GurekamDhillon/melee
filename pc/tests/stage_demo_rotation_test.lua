local path=assert(arg[1])
local loaded,queues,initial,commands={},{},{},{}
gd={
  stage_slot_load=function(name) loaded[#loaded+1]=name;return #loaded end,
  stage_switch=function(slot,opt) initial={slot,opt};return true end,
  stage_queue=function(q) queues[#queues+1]=q;return true end,
  command=function(name,fn) commands[name]=fn end,
  log=function(msg) error(msg) end,
}
dofile(path)
on_match_start();assert(#loaded==5 and initial[1]==1 and #queues==0)
assert(loaded[4]=="dream_land" and loaded[5]=="fountain_of_dreams")
on_frame();assert(#queues==0)
on_stage_switch{phase="before",slot=1};on_frame();assert(#queues==0)
on_stage_switch{phase="after",slot=2};on_frame();assert(#queues==0)
on_stage_switch{phase="after",slot=1};assert(#queues==0);on_frame();assert(#queues==1)
local q=queues[1];assert(#q==5 and q.loop and not q.shuffle and q.seed==42)
local seen={};for i,e in ipairs(q) do assert(e.slot==i%5+1 and e.after==10 and e.indicator==1);seen[e.transition]=true end
assert(seen.wipe and seen.flash and seen.morph)
for i=1,10 do on_stage_switch{phase="after",slot=i%5+1};on_frame() end
assert(#queues==1)
loaded={};on_match_start();assert(#queues==1);on_stage_switch{phase="after",slot=1};on_frame();assert(#queues==2)
print("real Lua demo: five destinations, wipe/flash/morph, post-initial one-shot arming and match reset passed")
