local captured=nil
local line="Hit P2 to inspect a captured collision"
function on_hit(a,v,e)
 captured=e
 line=("%s > %d: %s %s; grounded=%s/%s; damage=%s/%s"):format(tostring(a),v,e.move_tag or "uncaptured",e.element_tag or "uncaptured",tostring(e.attacker_grounded),tostring(e.victim_grounded),tostring(e.attacker_damage),tostring(e.victim_damage)); gd.log(line)
end
function on_draw() gd.text(20,20,line) end

gd.command("demo_state",function(token)
 local e=captured
 local valid=e and e.context_valid and type(e.attacker_damage)=="number" and type(e.victim_damage)=="number" and type(e.attacker_grounded)=="boolean" and type(e.victim_grounded)=="boolean"
 gd.log("tour state "..token.." captured="..tostring(not not valid).." move="..tostring(e and e.move_tag).." element="..tostring(e and e.element_tag).." attacker_damage="..tostring(e and e.attacker_damage).." victim_damage="..tostring(e and e.victim_damage))
end,"demo_state <token>: inspect actual queued collision context")
