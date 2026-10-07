-- The Envoy run setup as a gd.ui description (Atlas step 3; spec 13.3): the live choices only (Begin, Mode, Fighter, Difficulty, Stocks).
-- The parked screens (hub, companion, records, garden) are not ported. It mirrors the legacy menu state (app.menu.run_type, .fighter,
-- .difficulty, .stocks are the same fields the legacy setup rows set) and starts through the app's own start path (menu_effect {type='start'}
-- -> start_retail / start_coop), so every refusal keeps its notice, shown here as a screen note.
--
-- HOW IT ATTACHES. The legacy menu calls menu:show('setup') at every refusal and entry; menu.lua's on_show hook (atlas_kit.menu_show) opens this
-- screen instead of drawing the legacy one, and routes the legacy pad events (menu_input.lua: it hides D-pad and START from the game and never
-- masks online) to the engine (menu.atlas.input). The screen persists across scenes: the menu state owns it, and leaving the screen closes it.
return function(D)
 local K=D.atlas_kit
 local U={}
 local BASE='envoy.setup'
 local MODES={'classic','adventure','coop'}
 local MODE_NAME={classic='Classic',adventure='Adventure',coop='Co-op'}
 local DIRS={up=true,down=true,left=true,right=true}
 local STOCK_MAX=5;local DIFF_MAX=4

 local function fighter_list(app) local ok,c=pcall(app.context,app);return ok and c and c.fighters or {} end
 local function fighter_label(app,id)
  for _,f in ipairs(fighter_list(app)) do
   local fid=type(f)=='table' and f.id or f
   if fid==id then return type(f)=='table' and (f.label or f.id) or f end
  end
  return tostring(id)
 end
 local function coop_ok(app) return app.coop_ready and app:coop_ready()==true end
 local function modes(app)
  local out={'classic','adventure'}
  if coop_ok(app) then out[#out+1]='coop' end
  return out
 end

 -- the one short rule of each row's current choice
 local function tuning_every() local e=D.drive_economy and D.drive_economy.tuning;return e and e.reward_every or 3 end
 local RULES={
  classic=function() return 'Classic: the retail ladder, with a drive pick every '..({'first','second','third','fourth','fifth'})[tuning_every()]..' stage.' end,
  adventure=function() return 'Adventure: the retail side-scrolling stages, with the same build and rewards.' end,
  coop=function() return 'Co-op: two players share one team and one build on one screen.' end}

 function U.rows(app)
  local m=app.menu
  local mode_name=MODE_NAME[m.run_type] or tostring(m.run_type)
  local rows={
   {id='begin',label='Begin '..mode_name..' as '..fighter_label(app,m.fighter),what='Starts the run with the choices below.'},
   {id='mode',label='Mode',value={kind='choice',text=mode_name},what=(RULES[m.run_type] or RULES.classic)()},
   {id='fighter',label='Fighter',value={kind='choice',text=fighter_label(app,m.fighter)},what='The fighter you play the whole run.'},
   {id='difficulty',label='Difficulty',value={kind='choice',text=tostring(m.difficulty)},what='How strong the opponents are for this run: higher is harder.'},
   {id='stocks',label='Stocks',value={kind='choice',text=tostring(m.stocks)},what='How many stocks you start the run with.'}}
  return rows
 end

 function U.describe(app)
  local rows=U.rows(app);local items,what={},{}
  for _,r in ipairs(rows) do items[#items+1]={id=r.id,label=r.label,value=r.value};what[r.id]=r end
  local trail=K.parents();trail.title='RUN SETUP'
  return {
   id=BASE,trail=trail,chapter=1,persist=true,
   primary={kind='list',items=items},
   explainer={width='normal',provide=function(cid) local r=what[cid] or rows[1];return {kicker='RUN SETUP',title=r.id=='begin' and 'Begin' or r.label,what=r.what} end},
   keys={{'A',function(cid) return cid=='begin' and 'Begin' or 'Change' end},{'B','Back'}},
   input='feed',port=1,
   on={accept=function(cid) return U.accept(app,cid) end,change=function(cid,v) return U.change(app,cid,v) end,back=function() return U.back(app) end}}
 end

 function U.register(app)
  local ok,err=pcall(function() app.g.ui.screen(U.describe(app)) end)
  if not ok then K.log_once({host={log=function(_,t) app.g.log(t) end}},'the setup screen was refused, the legacy setup stays: '..tostring(err),'envoy atlas setup');return false end
  return true
 end

 local function cycle(list,cur,dir)
  local at=1;for i,v in ipairs(list) do if v==cur then at=i end end
  return list[(at-1+dir)%#list+1]
 end
 local function note(app,text,kind) if text and app.g.ui and app.g.ui.note then pcall(app.g.ui.note,{text=tostring(text),kind=kind or 'warn',seconds=4}) end end

 -- a choice row stepped: write the same menu field the legacy row sets, and describe again
 function U.change(app,cid,dir)
  local m=app.menu;local d=(type(dir)=='number' and dir<0) and -1 or 1
  if cid=='mode' then m.run_type=cycle(modes(app),m.run_type,d)
  elseif cid=='fighter' then
   local ids={};for _,f in ipairs(fighter_list(app)) do local fid=type(f)=='table' and f.id or f;if not (type(f)=='table' and f.disabled) then ids[#ids+1]=fid end end
   if #ids>0 then m.fighter=cycle(ids,m.fighter,d) end
  elseif cid=='difficulty' then m.difficulty=(m.difficulty+d)%(DIFF_MAX+1)
  elseif cid=='stocks' then m.stocks=(m.stocks-1+d)%STOCK_MAX+1 end
  U.register(app)
 end

 -- Begin: the app's own start path. A refusal sets app.notice (and menu:show('setup') again): the screen stays and says why.
 function U.accept(app,cid)
  if cid~='begin' then return nil end
  local m=app.menu
  app.notice=nil
  app:menu_effect({type='start',fighter=m.fighter,mode=m.run_type,difficulty=m.difficulty,stocks=m.stocks})
  if app.notice then note(app,app.notice,'warn') end
  return nil
 end

 -- B: leave Envoy's menus (the legacy B on setup went to the parked fighter screen)
 function U.back(app)
  app:menu_effect({type='quit'})
  U.close(app)
  return nil
 end

 function U.open(app)
  local m=app.menu
  if not K.enabled(app.g) then return false end
  local mt=app.g.match and app.g.match()
  if mt and mt.netplay then return false end
  if m.atlas and m.atlas.screen=='setup' then
   U.register(app)
   if app.notice then note(app,app.notice,'warn') end
   return true
  end
  if not U.register(app) then return false end
  m.atlas={screen='setup',
   input=function(action)
    if DIRS[action] or action=='accept' or action=='back' then pcall(app.g.ui.feed,BASE,action) end
    return nil
   end,
   close=function() pcall(app.g.ui.close,BASE) end}
  app.g.ui.open(BASE)
  if app.notice then note(app,app.notice,'warn') end
  return true
 end

 function U.close(app)
  local at=app.menu.atlas
  if at and at.screen=='setup' then app.menu.atlas=nil;at.close() end
 end

 return U
end
