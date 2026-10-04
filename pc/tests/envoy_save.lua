local T=dofile((io.open('pc/tests/envoy_testlib.lua') and '' or 'melee/')..'pc/tests/envoy_testlib.lua')
local D=T.rules();local S=T.module('save',D)
local function fixture(text)
  local s={text=text,writes=0};local g={}
  g.data_read=function() return s.text end
  g.data_write_atomic=function(_,v) s.writes=s.writes+1;if s.fail then return false,'disk full' end;s.text=v;return true end
  return s,S.open(g),g
end
T.test('full schema stable roundtrip',function()
  local p=S.new_profile();p.companions[1].dna.colour={'red','normal'};p.unlocks.white_found=true
  D.companion.feed(p.companions[1],'green',1234)
  local text=S.encode(p);local q=S.decode(text);assert(S.encode(q)==text and q.companions[1].stats.speed.points==1234)
end)
T.test('legacy missing profile starts in memory; first action one atomic replacement',function()
  local s,store=fixture();assert(store.profile and store.fresh and not store.error and s.writes==0)
  assert(store:commit(store.profile));assert(s.writes==1 and not store.fresh)
  s,store=fixture(S.encode(S.new_profile()))
  local p=S.new_profile();assert(store:commit(p));assert(s.writes==1 and S.decode(s.text))
end)
T.test('existence API distinguishes absent from unreadable without writing',function()
  for _,present in ipairs({false,true}) do
    local writes=0;local store=S.open({data_read=function() return nil end,
      data_exists=function() return present end,data_write_atomic=function() writes=writes+1;return true end})
    if present then assert(store.error and not store.profile);assert(not store:commit(S.new_profile()))
    else assert(store.profile and store.fresh) end
    assert(writes==0)
  end
  for _,exists in ipairs({function() return nil,'access denied' end,function() error('sharing violation') end}) do
    local store=S.open({data_read=function() return nil end,data_exists=exists})
    assert(store.error and not store.profile)
  end
end)
T.test('distinct missing read reason starts default; other errors refuse',function()
  local store=S.open({data_read=function() return nil,'missing' end})
  assert(store.profile and store.fresh and not store.error)
  store=S.open({data_read=function() return nil,'missing' end,data_exists=function() return true end})
  assert(store.error and not store.profile)
end)
T.test('read exceptions and nil reasons never create replacement profiles',function()
  for _,read in ipairs({function() error('access denied') end,function() return nil,'sharing violation' end}) do
    local writes=0;local store=S.open({data_read=read,data_write_atomic=function() writes=writes+1;return true end})
    assert(store.error and not store.profile);assert(not store:commit(S.new_profile()));assert(writes==0)
  end
end)
T.test('corrupt future and executable saves locked without overwrite',function()
  for _,text in ipairs({'broken','return os.exit()','ENVOY 99\n',S.encode(S.new_profile())..'junk\n'}) do
    local s,store=fixture(text);assert(not store.profile and store.error)
    assert(not store:commit(S.new_profile()));assert(s.writes==0 and s.text==text)
  end
end)
T.test('failed atomic write retains old bytes and old profile',function()
  local s,store=fixture(S.encode(S.new_profile()));local old=s.text;s.fail=true
  local p=S.decode(old);D.companion.feed(p.companions[1],'red',100)
  assert(not store:commit(p));assert(s.text==old and store.profile.companions[1].stats.power.points==0)
  s.fail=false;assert(store:commit(p));assert(s.writes==2)
end)
T.test('schema rejects nonfinite inconsistent and missing fields',function()
  local p=S.new_profile();p.next_run=0;T.refuses(function() S.encode(p) end)
  p=S.new_profile();p.companions[1].stats.power.level=9;T.refuses(function() S.encode(p) end)
  p=S.new_profile();p.active=2;T.refuses(function() S.encode(p) end)
end)
T.test('every truncation of an encoded record is corrupt and never overwritten',function()
  local text=S.encode(S.new_profile())
  for n=0,#text-1 do
    local s,store=fixture(text:sub(1,n));assert(store.error and not store.profile)
    assert(not store:commit(S.new_profile()));assert(s.writes==0)
  end
end)
T.test('both legacy schemas preserve levels and uncertainty strictly',function()
  local source=S.new_profile();source.companions[1].type='jump';source.companions[1].dna.jump={'C','A'};S.refresh_records(source)
  local base=S.encode(source):gsub('seeds [^\n]+\n',''):gsub('life [^\n]+\n','')
    :gsub('(stat %w+ %w+ %d+ %d+) %w+ %d+ %d+','%1')
    :gsub('stat power C 0 0','stat power C 550 5'):gsub('stat jump C 0 0','stat jump C 550 5')
  for _,version in ipairs({1,2}) do
    local text=base:gsub('ENVOY 4','ENVOY '..version):gsub('dna jump ','dna reach '):gsub('stat jump ','stat reach '):gsub('0 jump 0','0 reach 0')
    if version==1 then text=text:gsub('records [^\n]+\n','')
    else text=text:gsub('records 0 0 0 0 1 3','records 0 0 0') end
    local p=S.decode(text);local c=p.companions[1];assert(c.stats.power.level==5 and c.life_unknown and c.stats.power.life_gain==0)
    local f=D.companion.progress(c,'power');assert(f==.5);assert(c.type=='jump' and c.stats.jump.level==5 and D.companion.progress(c,'jump')==.5)
    assert(c.dna.jump[1]=='C' and c.dna.jump[2]=='A' and c.stats.jump.base_grade=='C' and c.stats.jump.life_gain==0)
    assert(S.encode(S.decode(S.encode(p)))==S.encode(p))
    T.refuses(function() S.decode(text:gsub('stat power C 550 5','stat power C 550 4')) end)
  end
end)
T.test('pending seed age and records survive atomic retry',function()
  local state,store=fixture(S.encode(S.new_profile()));local p=S.decode(state.text)
  D.companion.start_run(p.companions[1]);S.record_start(p,2147483646)
  state.fail=true;assert(not store:commit(p));assert(store.profile.pending_seed==nil)
  state.fail=false;assert(store:commit(p));assert(store.profile.pending_seed==2147483646 and store.profile.records.runs==1 and store.profile.companions[1].age==1)
  local q=S.decode(state.text);assert(q.pending_seed==p.pending_seed);S.validate(q)
  T.refuses(function() S.record_start(q,1) end)
  q.pending_seed=-1;T.refuses(function() S.encode(q) end)
end)
T.test('unreplayable out-of-range seeds are refused',function()
 local p=S.new_profile();T.refuses(function() S.record_start(p,2147483647) end)
end)
T.test('version three maps Reach DNA and evolution without changing any other bytes',function()
  local p=S.new_profile();local c=p.companions[1]
  c.type='jump';c.age=3;c.lives=2;c.white_drives=4;c.life_unknown=true
  c.dna.jump={'D','A'};c.stats.jump={grade='S',base_grade='D',carry=17,points=237,level=5,life_gain=91}
  c.dna.colour={'yellow','blue'};c.colour='yellow';c.two_tone=true;c.dna.two_tone={true,false}
  p.last_settled=2;p.next_run=3;p.last_result='fail';p.records={runs=3,wins=1,best_frames=456,unknown_runs=1,companions_raised=3,highest_grade=6}
  p.pending_seed=987;p.last_seed=123;p.unlocks.yellow_found=true
  local current=S.encode(p);assert(current:match('^ENVOY 4\n'))
  local legacy=current:gsub('ENVOY 4','ENVOY 3'):gsub('dna jump ','dna reach '):gsub('stat jump ','stat reach '):gsub('3 jump 4','3 reach 4')
  local state,store=fixture(legacy);assert(store.profile and state.writes==0 and state.text==legacy)
  assert(S.encode(store.profile)==current,'migration must only change the version and trait/type names')
  assert(store:commit(store.profile));assert(state.writes==1 and state.text==current)
end)
T.test('version-specific trait and evolution tokens are strict',function()
  local p=S.new_profile();p.companions[1].type='jump';S.refresh_records(p)
  local current=S.encode(p)
  T.refuses(function() S.decode(current:gsub('dna jump ','dna reach ')) end)
  T.refuses(function() S.decode(current:gsub('stat jump ','stat reach ')) end)
  T.refuses(function() S.decode(current:gsub('0 jump 0','0 reach 0')) end)
  local old=current:gsub('ENVOY 4','ENVOY 3'):gsub('dna jump ','dna reach '):gsub('stat jump ','stat reach '):gsub('0 jump 0','0 reach 0')
  assert(S.decode(old).companions[1].type=='jump')
  T.refuses(function() S.decode(old:gsub('dna reach ','dna jump ')) end)
  T.refuses(function() S.decode(old:gsub('stat reach ','stat jump ')) end)
  T.refuses(function() S.decode(old:gsub('0 reach 0','0 jump 0')) end)
end)
T.done()
