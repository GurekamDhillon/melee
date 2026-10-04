-- Derived from vocabulary only: no curated modifier pair registry.
return function(D)
 local status_tags={burn='burning',chill='chilled',curse='cursed',haste='hasted',guarded='guarded',momentum='momentum'}
 local function profile(m)
  local tags,produces,consumes,events={},{},{},{}
  for _,tag in ipairs(m.tags) do tags[tag]=true end
  for _,c in ipairs(m.conditions) do for _,key in ipairs({'status','self_status','target_status'}) do if c[key] then consumes[c[key]]=true end end end
  local function event(kind,status,tag) events[#events+1]={kind=kind,status=status,tags=tag and {[tag]=true} or {}} end
  for _,e in ipairs(m.effects) do
   if e.op=='status' or e.op=='stacks' then produces[e.status]=true;event('status_applied',e.status,status_tags[e.status]);if e.op=='stacks' or e.max>1 then event('stacks_changed',e.status,status_tags[e.status]) end
   elseif e.op=='remove_status' then consumes[e.status]=true;event('status_removed',e.status)
   elseif e.op=='emit' then event(e.event,nil,e.tag)
   elseif e.op=='versus-status' and e.status then consumes[e.status]=true
   elseif e.op=='convert' and e.change.element then tags[e.change.element]=true end
  end
  return {tags=tags,produces=produces,consumes=consumes,events=events,trigger=m.trigger,conditions=m.conditions}
 end
 local function can_trigger(source,receiver)
  for _,event in ipairs(source.events) do if event.kind==receiver.trigger then
   local matches=true;for _,c in ipairs(receiver.conditions) do
    if c.status and (c.status=='any' and not event.status or c.status~='any' and c.status~=event.status) then matches=false end
    if c.tag and not event.tags[c.tag] then matches=false end
   end
   if matches then return true end
  end end;return false
 end
 local function reasons(a,b)
  local out={};for tag in pairs(a.tags) do if b.tags[tag] then out[#out+1]='tag:'..tag end end
  for status in pairs(a.produces) do if b.consumes[status] or b.consumes.any then out[#out+1]='status:forward:'..status end end
  for status in pairs(b.produces) do if a.consumes[status] or a.consumes.any then out[#out+1]='status:reverse:'..status end end
  if can_trigger(a,b) then out[#out+1]='event:forward:'..b.trigger end
  if can_trigger(b,a) then out[#out+1]='event:reverse:'..a.trigger end
  table.sort(out);return out
 end
 local M={}
 function M.generate(pool)
  local list={};for _,m in ipairs(pool) do list[#list+1]=m end;table.sort(list,function(a,b)return a.id<b.id end)
  local pairs,degree={},{};for i,a in ipairs(list) do degree[a.id]=degree[a.id] or 0
   for j=i+1,#list do local b=list[j];local why=reasons(profile(a),profile(b))
    if #why>0 then pairs[#pairs+1]={a=a.id,b=b.id,reasons=why};degree[a.id]=degree[a.id]+1;degree[b.id]=(degree[b.id] or 0)+1 end
   end
  end
  return pairs,degree
 end
 return M
end
