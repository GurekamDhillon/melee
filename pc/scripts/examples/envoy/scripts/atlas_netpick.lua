-- The online reward pick as a gd.ui description (Atlas step 3; spec 13.3): in the online lobby, between games, this seat picks one of three
-- offered rules or keeps its build. FOUR CARDS (the three offers and "Keep my build"), the countdown from the lobby's own clock (env.left / 60),
-- the trail "ENVOY REWARD  GAME n".
--
-- NON-MODAL AND LOCAL. The screen never calls gd.pause or an input mask (menu_input.lua never masks online): it is drawn and driven locally and
-- writes no gameplay state. A reports the pick to the lobby (gd.netplay_act 'rpick', the card index; B sends 3, keep). It closes when the
-- seat's pick is in (env.picks[seat] >= 0) or the lobby's pick window (env.open) ends. The test hook `envoynet auto <n>` still picks by
-- itself; the legacy box (mod_lab.lua net_draw) stays for `envoy ui legacy` and for any build without Atlas fonts.
return function(D)
 local K=D.atlas_kit
 local N={}
 local ID='envoy.netpick'
 local DIRS={up=true,down=true,left=true,right=true}
 local KEEP_RULE='Keep the build you ended the last game with.'

 local function label_of(id) for _,m in ipairs(D.mod_pool or {}) do if m.id==id then return m.label end end;return id end
 local function rule_of(id)
  for _,m in ipairs(D.mod_pool or {}) do
   if m.id==id then
    local ok,lines=pcall(D.drive_text.mod_lines,m,1)
    if ok and type(lines)=='table' and lines[1] then return lines[1] end
   end
  end
  return ''
 end

 function N.describe(lab,np,env)
  local n=lab.net;local cards={}
  for i,id in ipairs(n.offers or {}) do
   cards[#cards+1]={id='offer:'..i,name=label_of(id),rule=K.fit_rule({host=nil},label_of(id),rule_of(id),'atlas netpick'),tag='PICK'}
  end
  cards[#cards+1]={id='keep',name='Keep my build',rule=KEEP_RULE,letter='K',rgba=0x9AA2B4FF,tag='KEEP'}
  local trail=K.parents();trail.title=('ENVOY REWARD  GAME %d'):format(np.game or 0)
  local g=lab.g
  local function send(index) pcall(g.netplay_act,'rpick',index) end
  return {
   id=ID,trail=trail,chapter=3,
   primary={kind='cards',cards=cards},
   explainer={width='narrow',provide=function(cid)
    for _,c in ipairs(cards) do if c.id==cid then
     return {kicker=cid=='keep' and 'KEEP' or 'OFFER',title=c.name,what=c.rule}
    end end
   end},
   keys={{'A','Take it'},{'B','Keep my build'}},
   countdown=math.ceil((env.left or 0)/60),
   input='feed',port=1,
   on={accept=function(cid)
    if cid=='keep' then send(3) else send((tonumber(cid:match(':(%d+)$')) or 1)-1) end
   end,back=function() send(3) end}}
 end

 function N.register(lab,np,env)
  local ok,err=pcall(function() lab.g.ui.screen(N.describe(lab,np,env)) end)
  if not ok then
   if lab.net and lab.net.atlas_err~=tostring(err) then lab.net.atlas_err=tostring(err);lab:net_log('the pick screen was refused, the legacy box stays: '..tostring(err)) end
   return false
  end
  lab.net.atlas_left=math.ceil((env.left or 0)/60)
  return true
 end

 function N.enabled(lab)
  local g=lab.g
  return K.enabled(g) and type(g.ui.open)=='function'
 end

 -- called from net_sync while this seat's pick is open: registers (or refreshes the countdown) and feeds the legacy pad events to the screen.
 -- Returns true when the Atlas screen handled it (the legacy cursor input is skipped).
 function N.sync(lab,np,env)
  local n=lab.net
  if not N.enabled(lab) then return false end
  if not n.atlas then
   if not N.register(lab,np,env) then return false end
   lab.g.ui.open(ID);n.atlas=true
  elseif math.ceil((env.left or 0)/60)~=n.atlas_left then N.register(lab,np,env) end
  n.input=n.input or D.menu_input.new(lab.g,1);n.input:set_active(true)
  for _,a in ipairs(n.input:poll()) do
   if DIRS[a] or a=='accept' or a=='back' then pcall(lab.g.ui.feed,ID,a) end
  end
  return true
 end

 function N.active(lab) return lab.net~=nil and lab.net.atlas==true end

 -- the pick is in, or the window ended
 function N.close(lab)
  local n=lab.net
  if not n or not n.atlas then return end
  n.atlas=nil;n.atlas_left=nil
  pcall(lab.g.ui.close,ID)
 end

 return N
end
