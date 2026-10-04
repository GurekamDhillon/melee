-- One capability: freeze the completed retail scene for a controller panel.
local held,left,was=false,0,false
function on_1p_stage_clear()
 held=gd.hold_1p(180);left=180
end
function on_tick()
 if not held then return end
 local down=gd.pad(1).A
 left=left-1
 if (down and not was) or left<=0 then gd.release_1p();held=false end
 was=down
end
function on_draw()
 if held then gd.kit.panel(30,40,550,70);gd.kit.text('Stage clear: press A to continue (auto releases)',45,65,14,'bone') end
end
function on_unload() gd.release_1p() end
