-- 200 seeded pseudo-random switches among the five stages. Start: soak <n> <seed>
local names={fd="final_destination",bf="battlefield",ys="yoshis_story",dl="dream_land",fod="fountain_of_dreams"}
local order={"fd","bf","ys","dl","fod"}
local slots={}
local S={run=false}
local function preload(lo)
  slots={}
  gd.log('soak preload begin free='..gd.stage_slots().heap_free)
  for _,k in ipairs(lo or order) do
    local s,why=gd.stage_slot_load(names[k])
    if not s then gd.log("soak preload fail "..k..": "..tostring(why)) else slots[k]=s end
  end
  gd.log("soak preload done free="..gd.stage_slots().heap_free)
end
gd.command('soakpre',function(arg) if arg=='big' then preload({'fod','ys','dl','bf','fd'}) else preload() end end,'soakpre [big]')
local function rng(st) st.seed=(st.seed*1103515245+12345)%2147483648; return st.seed end
local function build(n,seed)
  local st={seed=seed}; local seq={}
  local cur="fd"
  -- first lap in order, then random with forced patterns
  for i=1,n do
    local pick
    if i<=4 then pick=order[i+1]
    elseif i==5 then pick="fd"
    elseif i%50==0 then pick="fd"
    else
      local r=rng(st)%100
      if r<20 and i>6 then pick=seq[i-2] -- back-and-forth (A->B->A)
      elseif r<45 then pick=(rng(st)%2==0) and "fod" or "ys"   -- largest stages
      else pick=order[rng(st)%5+1] end
    end
    if i%50==49 and pick=="fd" then pick="ys" end
    if pick==cur then pick=order[(rng(st)%4)+2]; if pick==cur then pick="fd" end end
    seq[i]=pick; cur=pick
  end
  return seq
end
local function status() return gd.stage_slots() end
local function step()
  if S.i>S.n then
    S.run=false
    gd.log(string.format("soak DONE n=%d ok=%d refused=%d retries=%d preflight=%d base_fd_free=%s last_fd_free=%s",S.n,S.ok,S.refused,S.retries,S.pre or 0,tostring(S.base),tostring(S.lastfd)))
    return
  end
  local k=S.seq[S.i]
  for p=1,2 do local f=gd.player(p); if f and f.y<-5 then gd.teleport(p,p==1 and -20 or 20,8) end end
  local ok,why=gd.stage_switch(slots[k],{transition=S.tr,duration_frames=S.dur,place="keep"})
  if ok then S.pending=k; S.wait=0
  else
    if tostring(why):find("heap") then S.refused=S.refused+1 else S.pre=(S.pre or 0)+1 end
    S.retries=S.retries+1
    gd.log(string.format("soak REFUSED i=%d %s->%s : %s free=%d",S.i,S.cur,k,tostring(why),status().heap_free))
    S.cool=20
    if S.retries>400 then S.run=false; gd.log("soak ABORT too many refusals") end
  end
end
function on_stage_switch(e)
  if e.phase=="after" and S.run and S.pending then
    local k=S.pending; S.pending=nil; S.cur=k; S.ok=S.ok+1; S.cool=S.gap
    local free=status().heap_free
    if k=="fd" then
      if not S.base then S.base=free end
      S.lastfd=free
    end
    if S.i%50==0 or S.i<=5 or S.i==S.n then
      gd.log(string.format("soak i=%d at=%s free=%d base_fd=%s",S.i,k,free,tostring(S.base)))
    end
    S.i=S.i+1
  end
end
function on_frame()
  if not S.run then return end
  if S.pending then S.wait=S.wait+1; if S.wait>600 then S.run=false; gd.log("soak ABORT switch never completed i="..S.i) end return end
  if S.cool>0 then S.cool=S.cool-1; return end
  step()
end
gd.command("soak",function(arg)
  local n,seed,tr,dur,gap=arg:match("(%d+)%s+(%d+)%s*(%a*)%s*(%d*)%s*(%d*)")
  S={run=true,n=tonumber(n),seed=tonumber(seed),tr=(tr~="" and tr or "flash"),dur=tonumber(dur) or 6,gap=tonumber(gap) or 10,
     i=1,ok=0,refused=0,retries=0,cur="fd",cool=0}
  if S.dur==nil or S.dur<=0 then S.dur=6 end
  S.seq=build(S.n,S.seed)
  gd.log("soak start n="..S.n.." seed="..S.seed.." seq="..table.concat(S.seq,",",1,math.min(S.n,40)))
  gd.log("soak initial free="..status().heap_free)
end,"soak <n> <seed> [transition] [dur] [gap]")
gd.command("soakpath",function(arg)
  local seq={} for k in arg:gmatch("[^,]+") do seq[#seq+1]=k end
  S={run=true,n=#seq,seed=0,tr="flash",dur=6,gap=30,i=1,ok=0,refused=0,retries=0,cur="fd",cool=0,seq=seq}
  gd.log("soak start path "..arg)
end,"soakpath fod,fd,...")
gd.command("soakdump",function(arg) gd.log("soakdump BEGIN "..arg.." free="..gd.stage_slots(true).heap_free); gd.log("soakdump END "..arg) end,"heap dump")

gd.command("soaktrack",function(arg) gd.stage_slots(2); gd.log("soak tracking on") end,"enable alloc tracking")
