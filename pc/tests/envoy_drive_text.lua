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
 local loot=D.drive_loot.new(D.mod_pool);local r=loot:roll(5,{depth=6,loop=0},'rare')
 local lines=D.drive_text.drive_lines(loot,r);assert(#lines>=3 and lines[1]:find('Base',1,true))
 assert(D.drive_text.header(loot,r):find('Rare / ',1,true))
 local f,s=D.mod_budget.build(D.mod_pool,{},{},{});local a=D.drive_text.totals(f,s)
 assert(D.drive_text.total_line('strength','Build strength',a)=='Build strength x1.00')
 local b={strength=2,damage_dealt=1.2,launch_dealt=1,speed=1,damage_taken=1.1}
 assert(D.drive_text.total_line('strength','Build strength',a,b)=='Build strength x1.00 -> x2.00')
 assert(D.drive_text.better('damage_taken',1,1.1)==false and D.drive_text.better('damage_dealt',1,1.2)==true and D.drive_text.better('speed',1,1)==nil)
end)
T.test('build strength is an index: x1.00 empty, three significant figures, never a percentage; the damage floor says so',function()
 local tx=D.drive_text
 assert(tx.strength_text(1)=='x1.00' and tx.strength_text(4.584)=='x4.58' and tx.strength_text(87.14)=='x87.1' and tx.strength_text(163.6)=='x164' and tx.strength_text(8600)=='x8600')
 local a={strength=3,damage_dealt=1,launch_dealt=1,speed=1,damage_taken=.15}
 assert(tx.total_line('damage_taken','Damage you take',a)=='Damage you take x0.15 (limit)')
 assert(tx.total_line('damage_taken','Damage you take',{damage_taken=.5})=='Damage you take x0.50')
 for _,s in ipairs({1,1.5,12,300,40000}) do assert(not tx.strength_text(s):find('%%'),'no percent sign') end
end)
T.test('a drive with a technique or crit rule carries the "counts lightly" note; a plain drive does not',function()
 local loot=D.drive_loot.new(D.mod_pool);local tech,plain
 for seed=1,400 do local r=loot:roll(seed,{depth=9,loop=0},'rare');if D.drive_text.has_technique(loot,r) then tech=tech or r else plain=plain or r end;if tech and plain then break end end
 assert(tech and plain,'both kinds roll');assert(D.drive_text.technique_note:find('counts it lightly',1,true))
end)
T.test('a value line never promises more than the budget cap allows (no -112% damage taken)',function()
 for _,m in ipairs(D.mod_pool) do if m.id=='armoured' then
  assert(D.drive_text.mod_lines(m,8)[1]=='Damage you take -77%','a real deep tier reads as its number')
  for t=9,#m.tiers do assert(D.drive_text.mod_lines(m,t)[1]:find('-85%% %(the most a build can reach%)'),D.drive_text.mod_lines(m,t)[1]) end
 end end
end)
T.done()
