-- TM-lite: savestate hotkeys for practice (the first step towards Training Mode features).
-- It changes gameplay (it rewinds the match), so mod.json says "gameplay": true, and the engine
-- refuses its savestates during a netplay session.
--
--   F5 / F6            save / load slot 1         F7 / F8   save / load slot 2
--   F9                 reset every fighter's percent to 0
--   port 1: hold D-pad Down + press R = save slot 1, D-pad Down + L = load slot 1
--   console: "tm save [n]", "tm load [n]", "tm percent <p>"

local toast, toast_until = nil, 0

local function say(msg)
  toast, toast_until = msg, gd.frame() + 90
  gd.log(msg)
end

local function save(n) if gd.match().active then gd.savestate(n) say("saved " .. n) end end
local function load(n)
  local ok, err = pcall(gd.loadstate, n)
  say(ok and ("loaded " .. n) or tostring(err):gsub("^.-: ", ""))
end

local prev = 0
function on_tick()
  if gd.key_pressed("F5") then save(1) end
  if gd.key_pressed("F6") then load(1) end
  if gd.key_pressed("F7") then save(2) end
  if gd.key_pressed("F8") then load(2) end
  if gd.key_pressed("F9") then
    for _, p in ipairs(gd.players()) do gd.set_percent(p.port, 0) end
    say("percent reset")
  end
  -- controller combo on port 1, edge-triggered on L/R while D-pad Down is held
  local pad = gd.pad(1)
  local now = pad.buttons
  local pressed = now & ~prev
  if pad.DOWN then
    if pressed & gd.buttons.R ~= 0 then save(1) end
    if pressed & gd.buttons.L ~= 0 then load(1) end
  end
  prev = now
end

function on_draw()
  if toast and gd.frame() < toast_until then
    gd.fill(250, 440, 140, 18, 0x000000B0)
    gd.text(258, 443, "TM-lite: " .. toast, 0x7BE495FF, 0.9)
  end
end

gd.command("tm", function(arg)
  local verb, n = arg:match("^(%a+)%s*(%d*)")
  n = tonumber(n)
  if verb == "save" then save(n or 1)
  elseif verb == "load" then load(n or 1)
  elseif verb == "percent" then
    for _, p in ipairs(gd.players()) do gd.set_percent(p.port, n or 0) end
  else gd.log("tm save [n] | tm load [n] | tm percent <p>") end
end, "TM-lite: save/load states, set percent")
