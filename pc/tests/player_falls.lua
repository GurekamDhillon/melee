-- @name: player_falls
-- @gameplay: true
-- @rollback_safe: false
-- Native LAB death accounting: falls advance even when time-mode stocks do not.
local ticks,finished=0,false
function on_tick()
 ticks=ticks+1
 if not finished and ticks>4000 then gd.log('TEST player_falls FAIL watchdog');finished=true;gd.quit() end
end
gd.run(function()
 local ok,why=pcall(function()
  assert(gd.wait_until(function()return gd.match().active and gd.match().frame>90 end,3000),'match timeout')
  assert(gd.player(2)==nil,'solo scene contains P2')
  local p=assert(gd.player(1));assert(type(p.falls)=='number','missing falls counter')
  gd.set_stocks(1,99)
  local baseline=p.falls
  local blast=assert(gd.stage_bounds().blast)
  gd.teleport(1,0,blast.bottom-100)
  assert(gd.wait_until(function()local q=gd.player(1);return q and q.falls>baseline end,300),'fall not counted')
  local after=assert(gd.player(1));assert(after.falls==baseline+1,'fall counted multiple times');assert(after.stocks==99,'LAB stocks unexpectedly decremented')
  gd.wait(15);assert(gd.player(1).falls==baseline+1,'repeat frame counted twice')
 end)
 finished=true;gd.log('TEST player_falls '..(ok and 'PASS' or 'FAIL '..tostring(why)));gd.quit()
end)
