-- The Envoy pause as a gd.ui description (Atlas step 3; spec 13.3): Resume, Bag (a rules-on run only), Controls, Quit run. "Controls" and
-- "Quit run" open a gd.ui.dialog (the quit asks once: A quits, B keeps playing).
--
-- LIFECYCLE IS UNCHANGED. Envoy's own START route (menu.lua: START in play shows the pause) and the app's sync_pause keep owning
-- gd.pause / gd.resume; this screen only replaces the legacy pause draw (menu.lua's on_show hook, atlas_kit.menu_show). Resume runs the
-- app's own resume path (menu:show('playing') then sync_pause); Quit runs its existing abandon effect.
--
-- THE RETAIL PAUSE TAKEOVER. The run also names this screen with gd.ui.pause_screen, so when the owner turns the takeover on
-- (MELEE_ATLAS_PAUSE=1) a retail pause in an Envoy run shows this list too; with it off (the default) a retail pause looks as today.
-- Resume then also asks the engine for the unpause (gd.ui.unpause: one-shot, offline only). Online Envoy does not pause: open() returns false.
return function(D)
 local K=D.atlas_kit
 local P={}
 local BASE='envoy.pause'
 local DIRS={up=true,down=true,left=true,right=true}
 local CONTROLS='Z + START  Bag\nA  Collect a drive\nHold Z + Down  Leave the stage'

 local function rules_on(app) return app.retail and app.retail.rules and app.retail.host~=nil end

 function P.rows(app)
  local rows={{id='resume',label='Resume',what='Back to the fight.'}}
  if rules_on(app) then rows[#rows+1]={id='bag',label='Bag',what='Your equipped drives, your bag and your keystones.'} end
  rows[#rows+1]={id='controls',label='Controls',what='The buttons Envoy adds to the game.'}
  rows[#rows+1]={id='quit',label='Quit run',what='Ends the run. Your build is lost.'}
  return rows
 end

 function P.describe(app)
  local rows=P.rows(app);local items,what={},{}
  for _,r in ipairs(rows) do items[#items+1]={id=r.id,label=r.label};what[r.id]=r end
  local trail=K.parents();trail.title='PAUSED'
  return {
   id=BASE,kind='pause',trail=trail,chapter=1,persist=true,
   primary={kind='list',items=items},
   explainer={width='normal',provide=function(cid) local r=what[cid] or rows[1];return {kicker='PAUSED',title=r.label,what=r.what} end},
   keys={{'A','Select'},{'B','Resume'}},
   input='feed',port=1,
   on={accept=function(cid) return P.accept(app,cid) end,back=function() return P.resume(app) end,start=function() return P.resume(app) end}}
 end

 function P.register(app)
  local ok,err=pcall(function() app.g.ui.screen(P.describe(app)) end)
  if not ok then app.g.log('envoy atlas pause: the pause screen was refused, the legacy pause stays: '..tostring(err));return false end
  return true
 end

 -- the screen is registered when a run starts so the retail pause takeover has it to push; gd.ui.pause_screen names it
 function P.ensure(app)
  if not K.enabled(app.g) or not app.g.ui.pause_screen then return false end
  local mt=app.g.match and app.g.match();if mt and mt.netplay then return false end
  if not P.register(app) then return false end
  pcall(app.g.ui.pause_screen,BASE)
  return true
 end

 function P.resume(app)
  local ui=app.g.ui
  if ui.retail then local ok,r=pcall(ui.retail);if ok and type(r)=='table' and r.paused and ui.unpause then pcall(ui.unpause) end end   -- the retail pause (takeover): resume it too
  app.menu:show('playing');app:menu_effect({type='resume'})
  P.close(app)
  return nil
 end

 function P.accept(app,cid)
  local ui=app.g.ui
  if cid=='resume' then return P.resume(app)
  elseif cid=='bag' then
   P.resume(app)
   local host=app.retail and app.retail.host
   if host and host.screen and not host.screen.active then pcall(host.screen.open,host.screen,'bag') end
  elseif cid=='controls' then
   pcall(ui.dialog,{title='CONTROLS',text=CONTROLS,actions={{'A','OK'}}})
  elseif cid=='quit' then
   pcall(ui.dialog,{title='Quit the run?',text='Your build is lost.',actions={{'A','Quit'},{'B','Keep playing'}},
    on=function(btn) if btn=='A' then P.close(app);app:menu_effect({type='abandon'}) end end})
  end
  return nil
 end

 function P.open(app)
  local m=app.menu
  if not K.enabled(app.g) then return false end
  local mt=app.g.match and app.g.match()
  if mt and mt.netplay then return false end   -- Envoy does not pause online
  if m.atlas and m.atlas.screen=='pause' then return true end
  if not P.register(app) then return false end
  -- the retail pause takeover already pushed this screen and the engine reads the pausing port's pad itself: do not push it twice or feed it twice
  local own=false
  if app.g.ui.retail then local ok,r=pcall(app.g.ui.retail);own=ok and type(r)=='table' and r.paused==true and r.takeover==true end
  m.atlas={screen='pause',
   input=function(action)
    if own then return nil end   -- the engine drives it from the pad
    if DIRS[action] or action=='accept' then pcall(app.g.ui.feed,BASE,action)
    elseif action=='back' or action=='start' then P.resume(app) end
    return nil
   end,
   close=function() pcall(app.g.ui.close,BASE) end}
  if not own then app.g.ui.open(BASE) end
  return true
 end

 function P.close(app)
  local at=app.menu.atlas
  if at and at.screen=='pause' then app.menu.atlas=nil;at.close() end
 end

 return P
end
