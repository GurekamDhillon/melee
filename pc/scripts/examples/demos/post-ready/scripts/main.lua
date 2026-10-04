-- One feature: readiness means this exact pass was successfully recorded/resolved.
local post,note=nil,'Waiting for active match'
gd.command('demo_state',function(token) local ready=post and gd.post_ready(post);gd.log('tour state '..tostring(token)..' post='..tostring(post~=nil)..' ready='..tostring(ready==true)) end)
local function clear() if post then gd.post_remove(post);post=nil end end
function on_tick()
 if not gd.match().active then clear();return end
 if not post then local h,why=gd.post_add('shaders/identity.wgsl',{stage='world',order=50})
  if not h then note=tostring(why);return end;post=h
 end
 local ready,why=gd.post_ready(post)
 note=why or (ready and 'This exact post pipeline is prepared (identity pass).' or 'Waiting for this post to render.')
end
function on_draw()
 local a=gd.safe_area();gd.fill(a.x+12,a.y+12,math.min(590,a.w-24),52,0x16202AE0)
 gd.text(a.x+22,a.y+22,'Post pipeline readiness',0xEBD175FF,1.1)
 gd.text(a.x+22,a.y+43,note,0xE8EEF4FF,1)
end
function on_match_end() clear() end
function on_scene() clear() end
function on_unload() clear() end
