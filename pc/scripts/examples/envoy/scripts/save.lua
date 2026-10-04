-- Strict non-executable versioned text, one profile, one atomic replacement.
return function(D)
  local C,G=D.companion,D.genetics
  local S={version=4,file='profile.envoy'};S.__index=S
  local function int(n,min,max)
    assert(type(n)=='number' and n==n and n%1==0 and n>=min and n<=max,'invalid profile integer')
  end
  local outcomes={none=true,win=true,fail=true,quit=true}
  function S.new_profile()
    return {version=4,active=1,companions={C.new()},unlocks={},next_run=1,last_settled=0,last_result='none',
      records={runs=0,wins=0,best_frames=0,unknown_runs=0,companions_raised=0,highest_grade=3}}
  end
  function S.record_start(p,seed)
    int(seed,0,2147483646);assert(p.pending_seed==nil,'run already started')
    p.pending_seed=seed;p.records.runs=p.records.runs+1
  end
  function S.refresh_records(p)
    local raised,highest=0,p.records.highest_grade
    for _,c in ipairs(p.companions) do
      raised=raised+c.lives+((c.type~='young' and c.type~='egg') and 1 or 0)
      for _,k in ipairs(C.stats) do highest=math.max(highest,G.rank[c.stats[k].grade]) end
    end
    p.records.companions_raised=math.max(p.records.companions_raised,raised)
    p.records.highest_grade=highest
  end
  -- Called once when preparing settlement, before updating the run ledger.
  -- Retrying the atomic write reuses that prepared profile without counting again.
  function S.record_result(p,reason,frames)
    assert(reason=='win' or reason=='fail' or reason=='quit','invalid record result')
    int(frames,0,999999999)
    if reason=='win' then assert(frames>0,'winning time requires positive logic frames') end
    if p.pending_seed~=nil then p.last_seed=p.pending_seed;p.pending_seed=nil
    else p.records.runs=p.records.runs+1 end
    S.refresh_records(p)
    if reason=='win' then
      assert(frames>0,'winning time requires positive logic frames')
      p.records.wins=p.records.wins+1
      if p.records.best_frames==0 or frames<p.records.best_frames then p.records.best_frames=frames end
    end
  end
  function S.validate(p)
    assert(type(p)=='table' and p.version==4,'unsupported profile version')
    assert(type(p.companions)=='table' and #p.companions>=1 and #p.companions<=4,'1..4 companions required')
    local n=0;for k in pairs(p.companions) do int(k,1,#p.companions);n=n+1 end
    assert(n==#p.companions,'dense companions required')
    local ids={};for _,c in ipairs(p.companions) do C.validate(c);assert(not ids[c.id],'duplicate companion id');ids[c.id]=true end
    if p.pending_seed~=nil then int(p.pending_seed,0,2147483646) end
    if p.last_seed~=nil then int(p.last_seed,0,2147483646) end
    int(p.active,1,#p.companions);int(p.next_run,1,1000000);int(p.last_settled,0,999999)
    assert(p.next_run==p.last_settled+1 and outcomes[p.last_result],'invalid settlement ledger')
    assert((p.last_settled==0)==(p.last_result=='none'),'inconsistent result')
    local r=p.records;assert(type(r)=='table','records required')
    int(r.runs,0,1000000);int(r.companions_raised,0,1000000);int(r.highest_grade,1,6)
    assert(r.runs>=p.last_settled and r.runs<=p.last_settled+1,'inconsistent start ledger')
    assert((p.pending_seed~=nil)==(r.runs==p.last_settled+1),'inconsistent pending run')
    local raised=0
    for _,c in ipairs(p.companions) do
      raised=raised+c.lives+((c.type~='young' and c.type~='egg') and 1 or 0)
      for _,k in ipairs(C.stats) do assert(r.highest_grade>=G.rank[c.stats[k].grade],'highest grade below companion grade') end
    end
    assert(r.companions_raised>=raised,'raised count below completed lives')
    int(r.wins,0,999999);int(r.best_frames,0,999999999);int(r.unknown_runs,0,999999)
    assert(r.wins+r.unknown_runs<=p.last_settled,'inconsistent records ledger')
    assert((r.wins==0)==(r.best_frames==0),'inconsistent winning time')
    assert(type(p.unlocks)=='table','unlock flags required');local count=0
    for k,v in pairs(p.unlocks) do
      assert(type(k)=='string' and #k<=64 and k:match('^[a-z][a-z0-9_]*$') and type(v)=='boolean','invalid unlock flag')
      count=count+1
    end
    assert(count<=64,'too many unlock flags');return p
  end
  local function token(v) if type(v)=='boolean' then return v and '1' or '0' end
    if type(v)=='number' then return string.format('%.0f',v) end;return tostring(v) end
  function S.encode(p)
    S.validate(p);local lines={'ENVOY 4'}
    local function row(...) local v={...};for i,a in ipairs(v) do v[i]=token(a) end;lines[#lines+1]=table.concat(v,' ') end
    row('profile',p.active,p.next_run,p.last_settled,p.last_result)
    row('records',p.records.wins,p.records.best_frames,p.records.unknown_runs,p.records.runs,p.records.companions_raised,p.records.highest_grade)
    row('seeds',p.pending_seed or 'none',p.last_seed or 'none')
    local keys={};for k in pairs(p.unlocks) do keys[#keys+1]=k end;table.sort(keys)
    row('unlocks',#keys);for _,k in ipairs(keys) do row('flag',k,p.unlocks[k]) end
    row('companions',#p.companions)
    for _,c in ipairs(p.companions) do
      row('companion',c.id,c.age,c.type,c.white_drives,c.colour,c.two_tone,c.shiny)
      row('life',c.lives,c.life_unknown)
      for _,k in ipairs(G.traits) do row('dna',k,c.dna[k][1],c.dna[k][2]) end
      for _,k in ipairs(C.stats) do local a=c.stats[k];row('stat',k,a.grade,a.points,a.level,a.base_grade,a.carry,a.life_gain) end
      row('end')
    end
    row('finish');return table.concat(lines,'\n')..'\n'
  end
  function S.decode(text)
    assert(type(text)=='string' and #text<=65536,'profile too large')
    assert(text:sub(-1)=='\n' and not text:find('\r'),'invalid profile framing')
    local lines={};for l in text:gmatch('([^\n]*)\n') do lines[#lines+1]=l end
    local index=0
    local function row(name,count)
      index=index+1;local l=assert(lines[index],'truncated profile');local a={}
      for v in l:gmatch('%S+') do a[#a+1]=v end
      assert(#a==count and a[1]==name and table.concat(a,' ')==l,'bad profile row '..index)
      return a
    end
    local function number(s)
      assert(s and s:match('^%d+$') and #s<=10,'invalid numeric token')
      local n=tonumber(s);assert(tostring(n)==s,'noncanonical number');return n
    end
    local function boolean(s) assert(s=='0' or s=='1','invalid boolean');return s=='1' end
    local header=row('ENVOY',2);assert(header[2]=='1' or header[2]=='2' or header[2]=='3' or header[2]=='4','unsupported profile version')
    local legacy=header[2]~='4';local modern=header[2]=='3' or header[2]=='4'
    local function saved_key(k) return legacy and k=='jump' and 'reach' or k end
    local a=row('profile',5)
    local p={version=4,active=number(a[2]),next_run=number(a[3]),last_settled=number(a[4]),last_result=a[5],unlocks={},companions={}}
    if header[2]~='1' then
      a=row('records',modern and 7 or 4);p.records={wins=number(a[2]),best_frames=number(a[3]),unknown_runs=number(a[4])}
    else
      -- The old ledger knows total settlements, but contains no historical
      -- win count or elapsed times. Preserve that uncertainty explicitly.
      p.records={wins=0,best_frames=0,unknown_runs=p.last_settled}
    end
    if modern then
      p.records.runs=number(a[5]);p.records.companions_raised=number(a[6]);p.records.highest_grade=number(a[7])
      a=row('seeds',3);p.pending_seed=a[2]~='none' and number(a[2]) or nil;p.last_seed=a[3]~='none' and number(a[3]) or nil
    else
      p.records.runs=p.last_settled;p.records.companions_raised=0;p.records.highest_grade=1
    end
    local n=number(row('unlocks',2)[2]);assert(n<=64,'too many flags')
    for _=1,n do a=row('flag',3);assert(p.unlocks[a[2]]==nil,'duplicate flag');p.unlocks[a[2]]=boolean(a[3]) end
    n=number(row('companions',2)[2]);assert(n>=1 and n<=4,'invalid companion count')
    for i=1,n do
      a=row('companion',8)
      local c={id=number(a[2]),age=number(a[3]),type=a[4],white_drives=number(a[5]),colour=a[6],two_tone=boolean(a[7]),shiny=boolean(a[8]),dna={},stats={}}
      if legacy then
        assert(c.type~='jump','unexpected legacy evolution type')
        if c.type=='reach' then c.type='jump' end
      end
      if modern then a=row('life',3);c.lives=number(a[2]);c.life_unknown=boolean(a[3])
      else c.lives=0;c.life_unknown=true end
      for _,k in ipairs(G.traits) do
        a=row('dna',4);assert(a[2]==saved_key(k),'unexpected DNA trait')
        if k=='two_tone' or k=='shiny' then c.dna[k]={boolean(a[3]),boolean(a[4])} else c.dna[k]={a[3],a[4]} end
      end
      for _,k in ipairs(C.stats) do
        a=row('stat',modern and 8 or 5);assert(a[2]==saved_key(k),'unexpected stat')
        local old_points,level=number(a[4]),number(a[5])
        local stat={grade=a[3],points=old_points,level=level}
        if modern then stat.base_grade=a[6];stat.carry=number(a[7]);stat.life_gain=number(a[8])
        else
          assert(G.rank[stat.grade] and (c.dna[k][1]==stat.grade or c.dna[k][2]==stat.grade),'invalid legacy grade')
          assert(old_points<=9900 and level==math.floor(old_points/100),'invalid legacy points')
          -- Preserve level plus fractional progress, without inventing life feeding history.
          local lo=C.threshold(level);local step=C.threshold(level+1)-lo
          stat.points=lo+math.floor((old_points%100)*step/100)
          stat.base_grade=stat.grade;stat.carry=0;stat.life_gain=0
          p.records.highest_grade=math.max(p.records.highest_grade,G.rank[stat.grade])
        end
        c.stats[k]=stat
      end
      row('end',1);p.companions[i]=c
    end
    row('finish',1);assert(index==#lines,'trailing profile data')
    if not modern then S.refresh_records(p) end
    return S.validate(p)
  end
  function S.open(g)
    local store=setmetatable({g=g},S)
    local ok,text,why=pcall(g.data_read,S.file)
    if not ok then store.error='profile read refused: '..tostring(text)
    elseif text==nil then
      local missing=why=='missing' or why=='not_found' or why=='ENOENT'
      if why~=nil and not missing then store.error='profile read refused: '..tostring(why)
      elseif type(g.data_exists)=='function' then
        local checked,exists,reason=pcall(g.data_exists,S.file)
        if checked and exists==false then store.profile=S.new_profile();store.fresh=true
        else store.error='profile unreadable; preserved: '..tostring(checked and (reason or 'existing or unknown file') or exists) end
      else
        -- Legacy engines return only nil for a missing file. Explicit errors
        -- still refuse; prefer data_exists when the engine supplies it.
        store.profile=S.new_profile();store.fresh=true
      end
    else
      local valid,p=pcall(S.decode,text)
      if valid then store.profile=p else store.error='corrupt profile; preserved: '..tostring(p) end
    end
    return store
  end
  function S:commit(p)
    if self.error then return false,self.error end
    local valid,text=pcall(S.encode,p);if not valid then return false,text end
    if type(self.g.data_write_atomic)~='function' then return false,'engine request: data_write_atomic' end
    local ok,wrote,why=pcall(self.g.data_write_atomic,S.file,text)
    if not ok or not wrote then return false,tostring(ok and why or wrote) end
    self.profile=S.decode(text);self.fresh=nil;return true
  end
  return S
end
