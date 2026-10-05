-- @name: CPU virtual controller demo
-- @gameplay: true
-- One feature: a CPU performing technique through its own virtual controller.
local PORT = 2
local SPOT = {x = 0, y = 54}  -- the top platform of Battlefield
local steps = {
  {"wavedash right", function() gd.cpu_macro(PORT, "wavedash", {dir = "right"}) end},
  {"wavedash left",  function() gd.cpu_macro(PORT, "wavedash", {dir = "left"}) end},
  {"short hop, nair, L-cancel", function() gd.cpu_macro(PORT, "lcancel_aerial", {aerial = "nair"}) end},
  {"walk to the marked spot", function() gd.cpu_goto(PORT, SPOT.x, SPOT.y) end},
}
local k, state, idle, line = 0, "wait", 0, "waiting for a match with a CPU on port 2"

local function begin()
  k, state, idle = 0, "go", 0
end

function on_match_start() begin() end
gd.command("cpu_demo", function() begin() end, "cpu_demo: run the CPU controller demo again")

function on_frame()
  local p = gd.player(PORT)
  if not gd.match().active or not p or not p.cpu then return end
  if state == "wait" then begin() end
  if state == "go" then
    if k == 0 then
      assert(gd.cpu_mode(PORT, "script"), "port 2 must be a CPU")
      gd.teleport(PORT, -30, 0)
    end
    if p.airborne or p.action ~= 14 then return end
    idle = idle + 1
    if idle < 10 then return end
    k, idle = k + 1, 0
    if k > #steps then state, line = "done", "done: every step ran on the CPU's virtual controller"; return end
    steps[k][2]()
    state = "run"
  elseif state == "run" then
    if gd.cpu_script_done(PORT) then state = "go" end
  end
  if state ~= "done" and steps[k] then
    local s = gd.cpu_script_status(PORT)
    line = string.format("step %d/%d: %s   stick %d,%d  buttons %04X", k, #steps, steps[k][1], s.x, s.y, s.buttons)
  end
end

function on_draw() gd.text(20, 20, line) end
