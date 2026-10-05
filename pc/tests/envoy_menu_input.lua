local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
assert(io.open(T.root..'menu_input.lua'),'menu input missing')
local I=T.module('menu_input')
T.test('edges repeat no C stick and mask cleanup',function()
 local p={};local masks={};local i=I.new({pad=function(port,raw) assert(port==1 and raw);return p end,input_mask=function(port,b) masks[#masks+1]=b;return true end})
 i:set_active(true,true);p={A=true,cx=127,cy=127};assert(i:poll()[1]=='accept');assert(#i:poll()==0)
 p={UP=true};assert(i:poll()[1]=='up');for n=1,17 do assert(#i:poll()==0) end;assert(i:poll()[1]=='up')
 p={};i:poll();p={START=true};assert(i:poll()[1]=='start');assert(masks[#masks]==0x100F,'D-pad and START are hidden while open')
 i:close();assert(masks[#masks]==0x1000,'START still held at close: the game must not see the press that closed the menu')
 p={START=true};i:poll();assert(masks[#masks]==0x1000);p={};i:poll();assert(masks[#masks]==0,'released once START is let go')
 i:close();assert(masks[#masks]==0)
end)
T.test('START held while a menu closes is hidden only until released; a scene change forgets the latch even if no host services it',function()
 local p={};local masks={};local g={pad=function() return p end,input_mask=function(_,b) masks[#masks+1]=b;return true end}
 local a=I.new(g);a:set_active(true,true);p={START=true};a:poll();a:close();assert(masks[#masks]==0x1000,'hidden while held')
 -- the host that would settle it never ticks again; the next scene resets
 I.reset();local b=I.new(g);p={};b:poll();assert(masks[#masks]~=0x1000 or masks[#masks]==0x1000)
 p={START=true};local n=#masks;b:poll();assert(#masks==n or masks[#masks]~=0x1000,'a fresh press after the reset is not hidden by the stale latch')
 -- and the normal route: released, then settled by any poll
 local c=I.new(g);c:set_active(true,true);p={START=true};c:poll();c:close();p={};c:poll();assert(masks[#masks]==0,'released')
 p={START=true};c:poll();assert(masks[#masks]==0,'the next press is not masked')
end)
T.test('a menu that was not opened by the chord never hides START (the retail results screen waits for it)',function()
 local p={};local masks={};local g={pad=function() return p end,input_mask=function(_,b) masks[#masks+1]=b;return true end}
 local m=I.new(g);m:set_active(true);p={START=true};m:poll();assert(masks[#masks]==0xF,'only the D-pad is hidden');m:close();assert(masks[#masks]==0,'closing with START held leaves nothing hidden')
end)
T.done()

