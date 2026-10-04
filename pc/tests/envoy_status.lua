local T=dofile('melee/pc/tests/envoy_testlib.lua');local D=T.rules()
D.mod_codec=T.module('mod_codec',D);D.mod_schema=T.module('mod_schema',D);D.mod_budget=T.module('mod_budget',D)
T.test('one status declaration feeds the old tables unchanged',function()
 local M=D.mod_status
 local bits={burn=1,chill=4,curse=8,haste=16,guarded=32,momentum=64}
 for k,v in pairs(bits) do assert(D.mod_schema.status_bits[k]==v and M.bits[k]==v) end
 for k in pairs(M.bits) do assert(bits[k]) end
 local tags={burn='burning',chill='chilled',curse='cursed',haste='hasted',guarded='guarded',momentum='momentum'}
 for k,v in pairs(tags) do assert(M.tags[k]==v and D.mod_schema.tags[v]) end
 assert(table.concat(M.order,',')=='burn,shock,chill,curse,haste,guarded,momentum','engine and display order')
 assert(not D.mod_schema.statuses.shock and D.mod_schema.statuses.burn)
end)
T.test('status budgets and fighter values are the old numbers',function()
 local M=D.mod_status
 assert(M.budget('burn',3,2).sustain==6 and M.budget('chill').speed==-.2 and M.budget('haste').speed==.2)
 assert(M.budget('curse',.025).launch_taken==.025 and M.budget('momentum',0,5).momentum==5 and M.budget('momentum').momentum==5)
 local g=M.budget('guarded');assert(g.damage_taken==-.25 and g.launch_taken==-.15)
 local v=M.values('guarded',{});assert(v.damage_taken==-.25 and v.knockback_taken==-.15)
 assert(M.values('curse',{amount=.04}).knockback_taken==.04 and next(M.values('burn',{}))==nil)
 T.refuses(function() M.budget('shock') end)
end)
T.test('percent resistance is not called armour',function()
 for _,m in ipairs(D.mod_pool or {}) do assert(not m.label:lower():find('armour') and not m.label:lower():find('armor')) end
 local f=assert(io.open(T.root..'mod_status.lua'));local text=f:read('a');f:close();assert(text:find('not reaction armour',1,true))
end)
T.done()
