-- Live skeleton-region evidence, never persistent disc-derived draw ordinals.
return function()
 local R={}
 local colours={normal={222,232,245},red={245,142,138},green={149,231,169},blue={147,181,245},yellow={245,227,143},white={245,245,245},black={145,150,165},purple={206,161,240},orange={245,181,138},pink={245,172,213},cyan={151,234,239}}
 local types={power='red',speed='green',guard='blue',jump='yellow',balanced='purple',young='cyan',egg='white'}
 local function copy(a,shiny)
  local b={};for i=1,3 do b[i]=math.floor(a[i]+(shiny and (255-a[i])*.25 or 0)+.5) end;return b
 end
 function R.palette(c)
  c=c or {};local a=copy(colours[c.colour] or colours.normal,c.shiny)
  local b=c.two_tone and copy(colours[types[c.type] or 'white'],c.shiny) or copy(a)
  return {primary=a,secondary=b,shiny=c.shiny==true,two_tone=c.two_tone==true}
 end
 local accent={left_arm=true,right_arm=true,left_hand=true,right_hand=true,left_leg=true,right_leg=true,left_foot=true,right_foot=true}
 local body={head=true,torso=true};for k in pairs(accent) do body[k]=true end
 local function region(d)
  local best,weight=nil,0
  for k,v in pairs(d.regions or {}) do if body[k] and type(v)=='number' and (v>weight or v==weight and (not best or k<best)) then best,weight=k,v end end
  if weight>=.5 then return best end
 end
 local function safe(fn,...)
  if type(fn)~='function' then return nil end
  local ok,result=pcall(fn,...);if ok then return result end
 end
 R.REMEASURE=120 -- frames between live-pose re-measurements while the fighter's identity is unchanged
 function R.new(g,tuning)
  local settings=(tuning or {}).visuals or {};local v={active=false,diagnostic='inactive'}
  local key,shader_owned
  -- `refresh` re-measures every draw object's region weights in the live pose (gd.parts(slot,true): ~0.8 ms in
  -- the engine for a fighter); without it the engine returns its last measured snapshot (~0.1 ms). The key holds
  -- each draw's CLASSIFIED region, the only thing the tint depends on, never the raw weights: those move with
  -- every animation frame, and keying on them re-tinted the whole model every frame.
  local function player_ids(p) return table.concat({tostring(p.kind),tostring(p.costume),tostring(p.stocks),tostring(p.falls),p.action==12 and 'rebirth' or 'alive'},'|') end
  local function inspect(refresh)
   local p=safe(g.player,1);if not p then return end
   local parts=safe(g.parts,1,refresh);if not parts then return end
   local ids={tostring(p.kind),tostring(p.costume),tostring(p.stocks),tostring(p.falls),tostring(parts.geometry_signature)}
   -- Rebirth entry catches respawn even when stocks are unlimited.
   ids[#ids+1]=p.action==12 and 'rebirth' or 'alive'
   for _,d in ipairs(parts) do
    ids[#ids+1]=table.concat({tostring(d.index),tostring(d.path),tostring(d.joint),tostring(d.source),region(d) or ''},':')
   end
   return parts,table.concat(ids,'|'),player_ids(p)
  end
  function v:preview(c) local p=R.palette(c);return {p.primary,p.secondary} end
  function v:clear()
   if self.active and type(g.parts_clear)=='function' then
    local ok,result,err=pcall(g.parts_clear);if ok and result~=false and err==nil then self.active=false end
   end
   if shader_owned and safe(g.fighter_shader,1,nil)==true then shader_owned=nil end
   key=nil
   if self.active or shader_owned then self.diagnostic='cleanup pending';return false,self.diagnostic end
   self.diagnostic='inactive';return true,self.diagnostic
  end
  function v:apply(c,parts,k,pid)
   local cleared,why=self:clear();if not cleared then return false,why end
   if settings.recolour==false then self.diagnostic='disabled';return false,self.diagnostic end
   if not parts then parts,k,pid=inspect(true) end
   self.pid=pid;self.measured=g.frame and safe(g.frame) or nil
   if not parts or not parts.geometry_signature then self.diagnostic='no loaded model snapshot';return false,self.diagnostic end
   local selections={};local mapped=0
   for _,d in ipairs(parts) do
    -- path/joint/signature guards are refreshed atomically, not read from old indices.
    if d.source==0 and d.index~=nil and d.path~=nil and d.joint~=nil then
     local r=region(d);if r then mapped=mapped+1 end
     selections[#selections+1]={draw=d,region=r}
    end
   end
   local p=R.palette(c);local accepted,refused=0,0
   self.genes=table.concat(p.primary,',')..'/'..table.concat(p.secondary,',')..'/'..tostring(p.shiny)
   self.active=true;key=k
   for _,s in ipairs(selections) do
    local colour=mapped>0 and accent[s.region] and p.secondary or p.primary
    local packed=g.rgb and safe(g.rgb,table.unpack(colour)) or ((colour[1]<<24)|(colour[2]<<16)|(colour[3]<<8)|255)
    if safe(g.dobj_tint,1,s.draw.index,packed)==true then accepted=accepted+1 else refused=refused+1 end
   end
   -- Aurora's gd_surface receives the completed TEV prev, including ScriptParts_Tint.
   if accepted>0 and c.shiny and settings.shiny~=false then
    shader_owned=safe(g.fighter_shader,1,'shaders/gene-sheen.wgsl',{params={.75,.85,1,.045,2}})==true
   end
   self.diagnostic=(mapped==0 and 'whole-model tint fallback: no body-region map' or 'live body-region tint')..'; '..accepted..' draws, '..refused..' refused'
   return accepted>0 and refused==0,self.diagnostic
  end
  function v:tick(c)
   if settings.recolour==false then self:clear();return false,'disabled' end
   -- Re-measure the pose only when the fighter's identity changed or REMEASURE frames have passed (always, where
   -- the host gives no frame clock); in between the engine's cached snapshot answers the identity check cheaply.
   local now=g.frame and safe(g.frame) or nil
   local refresh=true
   if now and self.measured and self.active and now>=self.measured and now-self.measured<R.REMEASURE then
    local p=safe(g.player,1);refresh=not p or player_ids(p)~=self.pid
    if not refresh and settings.recolour~=false then
     -- Nothing identity-level changed and the pose is not due: the tint stands; only the expressed colour can differ.
     local q=R.palette(c);if table.concat(q.primary,',')..'/'..table.concat(q.secondary,',')..'/'..tostring(q.shiny)==self.genes then return true,self.diagnostic end
    end
   end
   local parts,k,pid=inspect(refresh)
   if not parts then self:clear();return false,'no loaded model' end
   if refresh and now then self.measured=now end
   local p=R.palette(c);local genes=table.concat(p.primary,',')..'/'..table.concat(p.secondary,',')..'/'..tostring(p.shiny)
   if not self.active or k~=key or genes~=self.genes then
    local ok,why=self:apply(c,refresh and parts or nil,refresh and k or nil,refresh and pid or nil);self.genes=genes;return ok,why
   end
   return true,self.diagnostic
  end
  return v
 end
 return R
end
