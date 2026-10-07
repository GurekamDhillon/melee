-- The Envoy results as a gd.ui description (Atlas step 3; spec 13.3): the end of a run. A list of read-only rows: the outcome, the stages
-- cleared, the time, and the build's final drives and keystones, one row each with its one rule in the explainer. A in the app's own "Return"
-- effect; B leaves Envoy (the legacy B on the results). The parked companion stat screens are not ported.
--
-- THE RETAIL RESULTS RULE IS KEPT. While the game's own results/clear scene is up (app.results_up) START belongs to the game: the app's
-- own guard (retail_app.lua tick) still keeps START from opening Envoy's pause then. This screen opens from the same call that showed the
-- legacy results (menu:results, through menu.lua's on_show hook, atlas_kit.menu_show), when the run is over. A pending save keeps its row:
-- "Save pending: A retries".
-- The interlude (the campaign's between-room screen) is parked and not ported.
return function(D)
 local K=D.atlas_kit
 local V={}
 local BASE='envoy.results'
 local DIRS={up=true,down=true,left=true,right=true}
 local MAX_ROWS=32

 local OUTCOME={win='Run complete',fail='Run over',quit='Run ended',death='Run over'}
 local function outcome_text(r) return OUTCOME[r.outcome] or ('Run ended: '..tostring(r.outcome or '?')) end
 local function seconds_text(frames)
  local s=math.floor((tonumber(frames) or 0)/60)
  return ('%d:%02d'):format(s//60,s%60)
 end

 -- the build's final drives and keystones, read from the rule host's bag (nothing is invented: a run without the rule host has none)
 local function build_rows(app)
  local rows={}
  local host=app.retail and app.retail.host
  if not host or not host.bag then return rows end
  local ok,b=pcall(host.bag,host);if not ok or not b then return rows end
  local loot=host.mods and host.mods.drives and host.mods.drives.loot
  local tx=D.drive_text
  if not loot or not tx then return rows end
  local function add(r,where)
   local lines=tx.drive_lines(loot,r)
   rows[#rows+1]={id=('drive:%d'):format(#rows+1),label=tx.short(loot,r),sub=where,what=lines[1] or '',title=loot:name(r),kicker='FINAL BUILD - '..where:upper(),from=(tx.rarity_label[r.rarity] or '')..' drive'}
  end
  for i=1,b:slots() do if b.equipped[i] then add(b.equipped[i],'Slot '..i) end end
  for _,r in ipairs(b.items) do add(r,'Bag') end
  for _,id in ipairs(host:keystone_ids()) do
   local rule=host:keystone_rule(id)
   local lines=rule and tx.keystone_lines(rule,D.mod_progression.tier(b.context)) or {}
   rows[#rows+1]={id=('key:%d'):format(#rows+1),label=rule and rule.label or id,sub='Keystone',what=lines[1] or '',title=rule and rule.label or id,kicker='FINAL BUILD - KEYSTONE'}
  end
  return rows
 end

 function V.rows(app)
  local r=app.menu.result or {}
  local rows={
   {id='outcome',label=outcome_text(r),sub=r.boss and tostring(r.boss) or nil,what='How the run ended.',title='Outcome',kicker='RESULT'},
   {id='stages',label='Stages cleared',value={kind='text',text=tostring(type(app.retail and app.retail.cleared)=='table' and #app.retail.cleared or 0)},what='The retail stages you cleared in this run.',title='Stages cleared',kicker='RESULT'},
   {id='time',label='Time',value={kind='text',text=seconds_text(r.frames)},what='The time of the last loop of the run.',title='Time',kicker='RESULT'}}
  for _,b in ipairs(build_rows(app)) do if #rows<MAX_ROWS then rows[#rows+1]=b end end
  if app:context().retail_pending then rows[#rows+1]={id='retry',label='Save pending: A retries',what='Your run could not be saved yet. A tries again.',title='Save pending',kicker='RESULT'} end
  return rows
 end

 function V.describe(app)
  local rows=V.rows(app);local items,by={},{}
  for _,r in ipairs(rows) do items[#items+1]={id=r.id,label=r.label,sub=r.sub,value=r.value};by[r.id]=r end
  local trail=K.parents();trail.title='RESULTS'
  local loop=app.retail and app.retail.loop or 0
  local pending=app:context().retail_pending
  return {
   id=BASE,trail=trail,chapter=1,persist=true,
   primary={kind='list',items=items},
   explainer={width='normal',provide=function(cid)
    local r=by[cid] or rows[1]
    return {kicker=r.kicker,title=r.title or r.label,what=r.what,from=r.from and {text=r.from} or nil}
   end},
   keys={{'A',pending and 'Retry save' or 'Done'},{'B','Leave'}},
   counter=loop>0 and ('NG+'..loop) or nil,
   input='feed',port=1,
   on={accept=function(cid) return V.accept(app,cid) end,back=function() return V.leave(app) end,start=function() return V.leave(app) end}}
 end

 function V.register(app)
  local ok,err=pcall(function() app.g.ui.screen(V.describe(app)) end)
  if not ok then app.g.log('envoy atlas results: the results screen was refused, the legacy results stay: '..tostring(err));return false end
  return true
 end

 function V.leave(app)
  local pending=app:context().retail_pending
  if pending then return nil end   -- a save is pending: the screen stays until it is written (as the legacy results)
  V.close(app)
  app:menu_effect({type='quit'})
  return nil
 end

 function V.accept(app,cid)
  if app:context().retail_pending then app:menu_effect({type='retail_retry'});V.register(app);return nil end
  return V.leave(app)
 end

 function V.open(app)
  local m=app.menu
  if not K.enabled(app.g) then return false end
  local mt=app.g.match and app.g.match()
  if mt and mt.netplay then return false end
  if m.atlas and m.atlas.screen=='results' then V.register(app);return true end
  if not V.register(app) then return false end
  m.atlas={screen='results',
   input=function(action)
    if DIRS[action] or action=='accept' then pcall(app.g.ui.feed,BASE,action)
    elseif action=='back' or action=='start' then V.leave(app) end
    return nil
   end,
   close=function() pcall(app.g.ui.close,BASE) end}
  app.g.ui.open(BASE)
  return true
 end

 function V.close(app)
  local at=app.menu.atlas
  if at and at.screen=='results' then app.menu.atlas=nil;at.close() end
 end

 return V
end
