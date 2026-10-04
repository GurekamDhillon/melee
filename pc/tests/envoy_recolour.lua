local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local R=T.module('recolour')
local c={colour='red',two_tone=true,shiny=false,type='speed'}
local function rig()
 local calls={tints={},clears=0};local p={kind=2,costume=0,stocks=4,action=14}
 local parts={geometry_signature='one',{index=0,path=1,joint=2,source=0,status=0,regions={torso=1}},{index=1,path=2,joint=3,source=0,status=0,regions={left_arm=1}}}
 local g={player=function() return p end,parts=function() return parts end,rgb=function(r,g,b) return {r,g,b} end,
 parts_clear=function() calls.clears=calls.clears+1 end,dobj_tint=function(_,i,col) calls.tints[#calls.tints+1]={i,col};return true end}
 return g,p,parts,calls
end
T.test('deterministic fresh palettes with type accent',function() local a,b=R.palette(c),R.palette(c);assert(a.primary[1]==b.primary[1]);a.primary[1]=0;assert(b.primary[1]>0);assert(b.primary[1]~=b.secondary[1]) end)
T.test('monotone and shiny brightness',function() local a=R.palette{colour='blue',two_tone=false};assert(a.primary[1]==a.secondary[1]);local b=R.palette{colour='blue',shiny=true};assert(b.primary[1]>a.primary[1]) end)
T.test('live regions tint independently and cleanup idempotent',function() local g,p,parts,x=rig();local v=R.new(g);assert(v:apply(c));assert(#x.tints==2);assert(x.tints[1][2][1]~=x.tints[2][2][1]);v:clear();v:clear();assert(x.clears==1) end)
T.test('missing map reports whole-model fallback',function() local g,p,parts,x=rig();for _,d in ipairs(parts) do d.regions={} end;local ok,why=R.new(g):apply(c);assert(ok and why:find('whole%-model'));assert(x.tints[1][2][1]==x.tints[2][2][1]) end)
T.test('signature and reused path index reapply guarded fresh snapshot',function() local g,p,parts,x=rig();local v=R.new(g);v:apply(c);v:tick(c);assert(#x.tints==2);parts[1].path=99;v:tick(c);assert(x.clears==1 and #x.tints==4);parts.geometry_signature='two';v:tick(c);assert(x.clears==2) end)
T.test('respawn kind costume and absent fighter transitions',function() local g,p,parts,x=rig();local v=R.new(g);v:apply(c);p.stocks=3;v:tick(c);p.costume=1;v:tick(c);p.kind=3;v:tick(c);assert(x.clears==3);g.player=function() return nil end;v:tick(c);assert(x.clears==4) end)
T.test('refused tint is reported and shader errors are silent',function() local g=rig();g.dobj_tint=function() return false end;g.fighter_shader=function() error('unsupported') end;local ok,why=R.new(g):apply(c);assert(not ok and why:find('refused')) end)
T.test('disabled settings and preview',function() local g,p,parts,x=rig();local v=R.new(g,{visuals={recolour=false}});assert(not v:apply(c));assert(#x.tints==0 and #v:preview(c)==2) end)
T.test('shiny composes and clears only owned successful shader',function()
 local g,p,parts,x=rig();local selected,cleared=0,0
 g.fighter_shader=function(port,path,options) assert(port==1);if path then assert(path=='shaders/gene-sheen.wgsl' and #options.params==5);selected=selected+1 else cleared=cleared+1 end;return true end
 local v=R.new(g);assert(v:apply{colour='purple',shiny=true,two_tone=true,type='guard'});v:clear();v:clear();assert(selected==1 and cleared==1 and x.clears==1)
end)
T.test('owned equipment and missing identity are never recoloured',function()
 local g,p,parts,x=rig();parts[1].source=1;parts[2].path=nil
 local ok,why=R.new(g):apply(c);assert(not ok and #x.tints==0)
end)
T.test('rebirth and changed expressed colour trigger reapply',function()
 local g,p,parts,x=rig();local v=R.new(g);v:apply(c);p.action=12;v:tick(c);assert(x.clears==1)
 p.action=14;v:tick(c);v:tick{colour='cyan',type='guard',two_tone=true};assert(x.clears==3)
end)
T.test('shader failure retains tint without claiming ownership',function()
 local g,p,parts,x=rig();local shader_calls=0
 g.fighter_shader=function() shader_calls=shader_calls+1;error('unsupported shader') end
 local v=R.new(g);assert(v:apply{colour='red',shiny=true});v:clear();assert(shader_calls==1 and x.clears==1)
end)
T.test('unlimited stocks falls and changed live region evidence reapply',function()
 local g,p,parts,x=rig();p.stocks=0;p.falls=0;local v=R.new(g);v:apply(c)
 p.falls=1;v:tick(c);assert(x.clears==1 and #x.tints==4)
 parts[1].regions={left_hand=1};v:tick(c);assert(x.clears==2 and #x.tints==6)
end)
T.test('failed tint cleanup retains owner and blocks apply until retry succeeds',function()
 local g,p,parts,x=rig();local v=R.new(g);v:apply(c)
 g.parts_clear=function() error('temporary refusal') end
 local ok,why=v:clear();assert(ok==false and v.active and why:find('cleanup'))
 assert(not v:apply(c));assert(#x.tints==2)
 g.parts_clear=function() return nil,'temporary refusal' end
 assert(not v:clear() and v.active)
 g.parts_clear=function() x.clears=x.clears+1 end
 assert(v:clear());assert(not v.active and x.clears==1)
end)
T.test('failed sheen cleanup retains ownership for next clear',function()
 local g,p,parts,x=rig();local clears=0;local refuse=true
 g.fighter_shader=function(_,path) if path then return true end;clears=clears+1;if refuse then return nil,'temporary' end;return true end
 local v=R.new(g);v:apply{colour='red',shiny=true};assert(not v:clear());refuse=false;assert(v:clear());assert(clears==2 and x.clears==1)
end)
T.test('Jump evolution expresses yellow two-tone accent',function() local p=R.palette{colour='blue',two_tone=true,type='jump'};assert(p.secondary[1]==245 and p.secondary[2]==227 and p.secondary[3]==143) end)
T.done()
