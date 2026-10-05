-- Plain words: every modifier at every tier reads as a sentence, never an id, a brace or an internal tier label.
local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua');local D={}
for _,n in ipairs({'mod_progression','mod_schema','mod_codec','mod_engine','mod_pool','drive_loot','drive_text'}) do D[n]=T.module(n,D) end
T.test('every modifier has plain lines at every tier',function()
 for _,m in ipairs(D.mod_pool) do for t=1,#m.tiers do
  local lines=D.drive_text.mod_lines(m,t);assert(#lines>=1,m.id)
  for _,l in ipairs(lines) do assert(type(l)=='string' and #l>8 and not l:find('[{}$]') and not l:find('T%d') and not l:find('x%d %[') and not l:find('^'..m.id..':'),m.id..': '..l) end
 end end
end)
T.test('a rolled drive reads as header, base line and modifiers; totals show before/after',function()
 local loot=D.drive_loot.new(D.mod_pool);local r=loot:roll(5,{depth=3,loop=0},'rare')
 local lines=D.drive_text.drive_lines(loot,r);assert(#lines>=3 and lines[1]:find('Base',1,true))
 assert(D.drive_text.header(loot,r):find('Rare / ',1,true))
 local f,s=D.mod_budget.build(D.mod_pool,{},{},{});local a=D.drive_text.totals(f,s)
 assert(D.drive_text.total_line('strength','Build strength',a):find('1.00',1,true))
 local b={strength=2,damage_dealt=1.2,launch_dealt=1,speed=1,damage_taken=1.1}
 assert(D.drive_text.total_line('strength','Build strength',a,b)=='Build strength 1.00 -> 2.00')
 assert(D.drive_text.better('damage_taken',1,1.1)==false and D.drive_text.better('damage_dealt',1,1.2)==true and D.drive_text.better('speed',1,1)==nil)
end)
T.done()
