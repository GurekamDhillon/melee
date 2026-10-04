local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local H=T.module('hub',D)
local function fixture()
 local s={p={x=0,y=10},pad={},models={},assets={},id=0,requests=0}
 local g={player=function() return s.p end,pad=function() return s.pad end,
 log=function(t) s.log=t end,match=function() return {active=true} end,
 model_load=function(path) assert(path=='models/bf_floor_4m.gxmesh');s.assets[1]=true;return 1 end,
 model_spawn=function(_,opts) s.id=s.id+1;s.models[s.id]=opts;return s.id end,
 model_set=function(h,o) s.models[h]=o;return true end,
 model_label=function(h,label) s.label=label;return true end,
 model_despawn=function(h) s.models[h]=nil;return true end,
 model_release=function(h) s.assets[h]=nil;return true end,
 project=function(x,y) return x,y,true end,text=function() end}
 local m={command=function(self,arg) s.requests=s.requests+1;assert(arg=='play hub');self.pending={};return true end,
 frame=function() end,stop=function(self) self.current=nil end}
 return s,H.new(g,m),m
end
T.test('garden request once queued then visible tinted placeholder',function()
 local s,h,m=fixture();local c=D.companion.new();c.colour='red'
 assert(h:enter());h:frame(c);h:frame(c);assert(s.requests==1 and not next(s.models))
 m.pending=nil;m.current={doc={name='hub'}};h:frame(c)
 assert(next(s.models) and s.models[1].tint~=0xFFFFFFFF and s.label:find(c.type,1,true))
 h:clear();assert(not next(s.models) and not next(s.assets))
end)
T.test('stations require proximity and rising A; nest remains closed',function()
 local s,h,m=fixture();h.active=true;m.current={doc={name='hub'}}
 s.p={x=-180,y=10};assert(not h:activate(false));local e=h:activate(true);assert(e.screen=='setup')
 assert(not h:activate(true));h:activate(false);s.p.x=-90;assert(h:activate(true).screen=='fighter')
 h:activate(false);s.p.x=90;assert(h:activate(true).screen=='records')
 h:activate(false);s.p.x=180;local e=h:activate(true);assert(e.closed and not e.screen)
 h:activate(false);s.p.x=400;assert(not h:activate(true))
end)
T.test('refused placeholder API is diagnosed and resources retained for cleanup retry',function()
 local s,h,m=fixture();h.active=true;m.current={doc={name='hub'}};h.g.model_spawn=function() return nil,'capacity' end
 h:frame(D.companion.new());assert(s.log:find('placeholder',1,true) and not next(s.models));h:clear();assert(not next(s.assets))
end)
T.done()
