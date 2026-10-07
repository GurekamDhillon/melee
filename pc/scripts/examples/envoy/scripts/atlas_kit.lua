-- Shared helpers of Envoy's Atlas screens (step 3; the bag's own helpers moved here unchanged, and the reward, swap, setup, pause, results,
-- online pick and the match HUD use them): the switch, the seat-suffixed screen id, the trail's parents, the log-once rule, the one-rule
-- paging, and the "one seat's screen at a time" rule.
--
-- THE SWITCH. `K.setting.on` is the one flag every Envoy Atlas screen and the HUD read (`K.enabled(g)`); the console's `uxatlas on|off`
-- sets it, and `envoy ui legacy on|off` forces every screen and the HUD back to the legacy code whatever the flag says (for the owner's
-- look). It is OFF by default: turning it on for everyone is the owner's call after the look (plan step 3, Task 18 part A).
return function(D)
 local K={setting={on=false},legacy={on=false},logged={}}
 K.PARENTS={'SOLO','ENVOY'}   -- the trail's parents; deleted when the registry entry (mod.json menus) supplies them
 K.WHAT_FIELD=159             -- gw_ui_screen.h AT_TEXT - 1: the explainer's `what` holds this many characters

 function K.set(on) K.setting.on=on and true or false end
 function K.set_legacy(on) K.legacy.on=on and true or false end

 -- gd.ui is there, the Atlas fonts are loaded, the switch is on and the legacy override is not
 function K.enabled(g)
  if K.legacy.on or not K.setting.on or type(g.ui)~='table' or type(g.ui.available)~='function' then return false end
  return (g.ui.available()) and true or false
 end

 function K.seat_port(S) return (S.host and S.host.seat and S.host.seat.port) or 1 end

 -- "envoy.bag" for port 1 and "envoy.bag.pN" for port N: two seats' screens are two screens, and closing one never closes the other
 function K.id(base,S)
  local port=K.seat_port(S)
  if port==1 then return base end
  return base..'.p'..tostring(port)
 end

 function K.parents() return {K.PARENTS[1],K.PARENTS[2]} end

 -- one log line per distinct text for the life of the script (a content bug must be visible, not repeated every frame)
 function K.log_once(S,text,prefix)
  if K.logged[text] then return end
  K.logged[text]=true
  if S.host and S.host.log then S.host:log((prefix or 'atlas')..': '..text) end
 end

 -- the rule lines of a cell: the legacy detail lines after the drive's name line, up to the first blank line
 function K.rules(lines)
  local rules={}
  for i=2,#lines do local l=lines[i];if l=='' then break end;rules[#rules+1]=l end
  return rules
 end

 -- a rule that does not fit the field is broken at its last word that fits and ends in "..."; never mid-word; logged once as a content bug
 function K.fit_rule(S,name,rule,prefix)
  if #rule<=K.WHAT_FIELD then return rule end
  K.log_once(S,('content bug: a rule of "%s" is %d characters, the explainer holds %d: %s'):format(tostring(name),#rule,K.WHAT_FIELD,rule:sub(1,40)..'...'),prefix)
  local cut=rule:sub(1,K.WHAT_FIELD-3)
  local at=cut:match('^.*()%s')
  if at and at>1 then cut=cut:sub(1,at-1) end
  return cut..'...'
 end

 -- step the shown rule of the focused cell: `state` holds rule_cid and rule_k; n rules, dir +1 or -1, wrapping. Returns the new rule index.
 function K.page(state,cid,n,dir)
  local k=(state.rule_cid==cid) and state.rule_k or 1
  state.rule_cid=cid;state.rule_k=(k-1+dir)%n+1
  return state.rule_k
 end
 -- the rule index to show now for a cell (1 when the focus moved: moving the focus starts at the first rule)
 function K.rule_index(state,cid,n)
  local k=(state.rule_cid==cid) and state.rule_k or 1
  if k>n then k=1 end
  return k
 end

 -- the corner a seat's own toasts go to: seat 1 top left, seat 2 top right (the match HUD's co-op strips)
 function K.corner(S) return K.seat_port(S)==2 and 'top_right' or 'top_left' end

 -- ONE SEAT'S SCREEN AT A TIME. Another seat's Envoy screen on top of the stack means this seat's must wait: returns false and the line a
 -- toast in this seat's corner says. The top screen's id carries its seat ("envoy.bag", "envoy.reward.p2"); seat 1 has no suffix.
 function K.may_open(S)
  local ui=S.g and S.g.ui
  if type(ui)~='table' or type(ui.state)~='function' then return true end
  local ok,st=pcall(ui.state)
  local top=ok and type(st)=='table' and st.top or nil
  if type(top)~='string' or top:sub(1,6)~='envoy.' then return true end
  local mine=K.seat_port(S)
  local suffix=top:match('%.p(%d)$')
  local other=suffix and tonumber(suffix) or 1
  if other==mine then return true end
  return false,('Player %d has a screen open'):format(other)
 end
 function K.say_busy(S,text)
  local ui=S.g and S.g.ui
  if type(ui)=='table' and type(ui.toast)=='function' then pcall(ui.toast,{zone=K.corner(S),title='BAG',text=text,seconds=3}) end
 end

 -- every screen id Envoy registers: forgotten when the run ends, so the engine's 16 slots are free again
 K.SCREENS={'bag','reward','keystone','swap','setup','pause','results','netpick'}
 function K.forget_all(g)
  local ui=g and g.ui
  if type(ui)~='table' or type(ui.forget)~='function' then return 0 end
  local n=0
  for _,base in ipairs(K.SCREENS) do
   for _,suffix in ipairs({'','.p2','.p3','.p4'}) do
    local ok,r=pcall(ui.forget,'envoy.'..base..suffix);if ok and r then n=n+1 end
   end
  end
  return n
 end
 return K
end
