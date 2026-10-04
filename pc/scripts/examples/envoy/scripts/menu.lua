-- Controller navigation only; pure state, no engine calls.
return function(D)
 local M={};local S={};S.__index=S
 local function row(label,target,disabled) return {label=label,target=target,disabled=disabled} end
 function M.new() return setmetatable({screen='title',focus={},fighter='mario',run_type='classic',difficulty=2,stocks=3,scroll=0},S) end
 function S:show(screen) self.screen=screen;if screen=='confirm' or screen=='quit' then self.focus[screen]=1 else self.focus[screen]=self.focus[screen] or 1 end;self.scroll=0 end
 function S:interlude(data) self.interlude_data=data or {};self:show('interlude') end
 function S:results(result) self.result=result or {};self:show('results') end
 function S:entries(c)
  c=c or {};local n=self.screen
  if n=='title' then return {row('Enter Envoy','profile'),row('Close Envoy','close')}
  elseif n=='profile' then return {row('Continue profile','hub',c.error~=nil),row('Close Envoy','close')}
  elseif n=='hub' then
   if c.retail_menu then
    local rows={row('Begin run','setup')}
    if c.garden_available then rows[#rows+1]=row('Walk in garden','hub') end
    for _,v in ipairs({row('Fighter','fighter'),row('Companion','companion'),row('Records','records'),row('Quit','quit')}) do rows[#rows+1]=v end
    return rows
   end
   return {row('Walk in garden','hub'),row('Fighter','fighter'),row('Companion','companion'),row('Records','records'),row('Nest / Breeding - unavailable',nil,true),row('Later campaigns - unavailable',nil,true),row('Quit','quit')}
  elseif n=='fighter' then
   local rows={};for _,f in ipairs(c.fighters or {'fox','marth','kirby'}) do
    local id=type(f)=='table' and f.id or f
    rows[#rows+1]={label=type(f)=='table' and (f.label or id) or id,fighter=id,disabled=type(f)=='table' and f.disabled or false}
   end
   return rows
  elseif n=='setup' then return {row('Begin '..self.run_type..' as '..self.fighter,'playing'),
   row('Mode: '..self.run_type,'mode'),row('Difficulty: '..self.difficulty,'difficulty'),row('Stocks: '..self.stocks,'stocks'),
   row('Change fighter','fighter'),row('Back to hub','hub')}
  elseif n=='reward' then
   local r=c.reward or {};if r.preview then return {row('Continue','reward_done')} end
   local rows={};for i,v in ipairs(r.options or {}) do rows[i]={label=v.colour..' drive / '..(v.effect or 'Growth bonus'),reward=i} end;return rows
  elseif n=='pause' then return {row('Resume','playing'),row('Companion','companion'),row('Abandon Run','confirm')}
  elseif n=='confirm' then return {row('Keep playing','pause'),row('Confirm abandon','hub')}
  elseif n=='quit' then return {row('Stay in Envoy','hub'),row('Confirm quit','title')}
  elseif n=='interlude' then return {row('Continue','playing')}
  elseif n=='results' then
   if c.retail_pending then return {row('Save pending: A retries','retail_retry')} end
   return {row('Return to menu','hub')}
  elseif n=='playing' then return {} end
  return {row('Back',n=='results' and 'hub' or self.return_to or 'hub')}
 end
 function S:input(action,c)
  local screen=self.screen
  if screen=='playing' then
   if action=='start' then self:show('pause');return {type='pause'} end
   return
  end
  if action=='back' or (action=='start' and screen=='pause') then
   if screen=='interlude' or screen=='reward' then return end
   if screen=='results' then if c and c.retail_pending then return end;self:show('hub');return {type='hub'} end
   if screen=='pause' then self:show('playing');return {type='resume'} end
   if screen=='title' then return {type='quit'} end
   local parents={title='title',profile='title',hub='title',fighter='hub',setup='fighter',confirm='pause',quit='hub',results='hub'}
   self:show(parents[screen] or self.return_to or 'hub');return
  end
  local entries=self:entries(c);local count=#entries;if count==0 then return end
  local focus=math.min(self.focus[screen] or 1,count)
  if action=='up' or action=='down' then
   local delta=action=='up' and -1 or 1
   for _=1,count do focus=(focus-1+delta)%count+1;if not entries[focus].disabled then break end end
   self.focus[screen]=focus
   if screen=='companion' then self.scroll=math.max(0,self.scroll+delta)
   elseif screen=='records' then self.scroll=math.max(0,math.min(math.max(0,#((c or {}).records or {})-10),self.scroll+delta)) end
   return
  end
  if action~='accept' then return end
  local entry=entries[focus];if entry.disabled then return end
  if screen=='reward' then return {type=entry.reward and 'reward' or 'reward_done',index=entry.reward} end
  if entry.target=='retail_retry' then return {type='retail_retry'} end
  if screen=='setup' then
   if entry.target=='mode' then self.run_type=self.run_type=='classic' and 'adventure' or 'classic';return end
   if entry.target=='difficulty' then self.difficulty=(self.difficulty+1)%5;return end
   if entry.target=='stocks' then self.stocks=self.stocks%5+1;return end
  end
  if entry.target=='close' then return {type='quit'} end
  if screen=='fighter' then self.fighter=entry.fighter;self:show('setup');return {type='fighter',fighter=self.fighter} end
  if entry.target=='companion' or entry.target=='records' then self.return_to=screen end
  self:show(entry.target or self.return_to or 'hub')
  if screen=='interlude' and entry.target=='playing' then return {type='continue'}
  elseif entry.target=='hub' and (screen=='results' or screen=='hub' or screen=='profile' or screen=='setup') then return {type='hub'}
  elseif screen=='setup' and entry.target=='playing' then return {type='start',fighter=self.fighter,mode=self.run_type,difficulty=self.difficulty,stocks=self.stocks}
  elseif screen=='pause' and entry.target=='playing' then return {type='resume'}
  elseif screen=='confirm' and entry.target=='hub' then return {type='abandon'}
  elseif screen=='quit' and entry.target=='title' then return {type='quit'} end
 end
 return M
end


