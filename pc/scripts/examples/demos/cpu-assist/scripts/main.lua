-- @name: CPU technique assist demo
-- @gameplay: true
-- One feature: gd.cpu_assist. Port 2 is a level 9 retail-AI CPU that stays in FIGHT mode; the engine adds technique
-- on top of the AI's own virtual pad (L-cancel, tech, perfect shield, wavedash, fast fall), each with the probability
-- set below. Port 1 is a script-mode CPU attacker (cpu_goto, cpu_macro, and a knockdown every fifth cycle).
-- Scene: MELEE_SCENE="mode=lab;stage=fd;p1=fox/cpu0;p2=fox/cpu9"   Console: assist 0..1 sets every probability.
local ATK, CPU = 1, 2
local TECH = {"lcancel", "perfect_shield", "tech", "wavedash", "fast_fall"}
local EVENT = {lcancel = "lcancel", tech = "tech", wavedash = "wavedash", fast_fall = "fast_fall", perfect_shield = "perfect_shield"}
local aerials = {"nair", "fair", "bair", "uair", "dair"}
local p, seen, started, cycle, wait, phase, frames = 1.0, {}, false, 0, 0, 0, 0

local function config(v)
  p = v
  return gd.cpu_assist(CPU, {lcancel = v, perfect_shield = v, tech = v, tech_dir = "random", wavedash = v, fast_fall = v, seed = 7})
end
gd.command("assist", function(a) config(tonumber(a) or 1) end, "assist <0..1>: every technique's probability")

function on_skill(i)
  if i.port == CPU and EVENT[i.kind] then seen[i.kind] = (seen[i.kind] or 0) + 1 end
end

local function attacker_step()
  local a, b = gd.player(ATK), gd.player(CPU)
  if not a or not b then return end
  cycle = cycle + 1
  if cycle % 5 == 0 then
    -- a knockdown: lift the CPU and hit it down, so it has a landing to tech
    if pcall(gd.teleport, CPU, b.x, b.y + 45) then gd.hit(CPU, {damage = 8, angle = 270, kbg = 90, bkb = 70, from = ATK}) end
  elseif phase == 0 then
    pcall(gd.cpu_goto, ATK, b.x, b.y, 16, 200); phase = 1
  else
    pcall(gd.cpu_macro, ATK, "lcancel_aerial", {aerial = aerials[cycle % 5 + 1], dir = "forward"}); phase = 0
  end
end

function on_frame()
  if not gd.match().active or not gd.player(CPU) then return end
  if not started then
    started = true
    assert(gd.cpu_mode(ATK, "script"), "port 1 must be a CPU")
    gd.log("cpu_assist demo: configured = " .. tostring(config(p)))
  end
  frames = frames + 1
  if frames % 3000 == 0 then
    local s, o = gd.cpu_assist(CPU), {}
    for _, name in ipairs(TECH) do o[#o+1] = string.format("%s %d/%d events %d", name, s.counters[name].opportunities, s.counters[name].performed, seen[name] or 0) end
    gd.log("cpu_assist demo: frame " .. frames .. ": " .. table.concat(o, " | "))
  end
  if wait > 0 then wait = wait - 1
  elseif select(2, pcall(gd.cpu_script_done, ATK)) == true then attacker_step(); wait = 6 end
end

function on_draw()
  local s = gd.cpu_assist(CPU)
  if not s.enabled then gd.text(20, 20, "gd.cpu_assist is not configured (CPU on port 2 in fight mode?)"); return end
  gd.text(20, 20, string.format("gd.cpu_assist on P2, every probability %.2f   (engine counters: opportunities / performed | skill events seen)", p))
  for n, name in ipairs(TECH) do
    local c = s.counters[name]
    gd.text(20, 20 + n * 14, string.format("%-15s %4d / %-4d | %d", name, c.opportunities, c.performed, seen[name] or 0))
  end
end
