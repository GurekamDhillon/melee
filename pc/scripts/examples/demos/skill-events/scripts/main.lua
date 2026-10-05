-- Skill events demo: one feature, the engine's technique telemetry (gd.skill_*, on_skill and the on_<kind> hooks).
-- Offline LAB, vanilla Battlefield. P1 is yours; P2 is a CPU you drive through its virtual controller, so every
-- technique can be shown on demand. Console: skill_do <macro> runs it on P2 (wavedash, waveland, short_hop,
-- full_hop, lcancel_aerial, dash_dance, jump_cancel_grab, perfect_shield, shield). The overlay lists the newest
-- events with their fields; P1's own wavedashes, L-cancels, techs and combos appear the same way.
local lines = {}
local SKIP = {kind = true, port = true, entity = true, subfighter = true, frame = true}
local function describe(e)
  local keys = {}
  for k in pairs(e) do if not SKIP[k] then keys[#keys + 1] = k end end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    local v = e[k]
    parts[#parts + 1] = k .. "=" .. (type(v) == "number" and string.format("%.2f", v):gsub("%.?0+$", "") or tostring(v))
  end
  return string.format("f%d P%d %s  %s", e.frame, e.port, e.kind, table.concat(parts, " "))
end
function on_skill(e)
  table.insert(lines, 1, describe(e))
  if #lines > 12 then lines[#lines] = nil end
  gd.log("skill " .. lines[1])
end
-- One kind-specific hook, to show the second delivery form: this fires only for L-cancels.
function on_lcancel(e) gd.log("on_lcancel: P" .. e.port .. " " .. e.aerial .. (e.hit and " (hit)" or "") .. " lag " .. e.lag .. " -> " .. e.lag_cancelled) end
function on_match_start() gd.cpu_mode(2, "script") end
gd.command("skill_do", function(name)
  gd.cpu_mode(2, "script")
  local ok, why = gd.cpu_macro(2, name ~= "" and name or "wavedash")
  gd.log("skill_do " .. tostring(name) .. " -> " .. tostring(ok) .. " " .. tostring(why))
end, "skill_do <macro>: run a technique macro on the P2 CPU")
function on_draw()
  local a = gd.safe_area()
  gd.fill(a.x + 12, a.y + 12, math.min(a.w - 24, 900), 28 + 16 * math.max(#lines, 1), 0x101827dd)
  gd.text(a.x + 24, a.y + 18, "Skill events (newest first). skill_do <macro> drives P2.", 0xffd369ff, 1.1)
  for i, l in ipairs(lines) do gd.text(a.x + 24, a.y + 22 + 16 * i, l) end
end
gd.command("demo_state", function(token)
  local t = gd.skill_thresholds()
  gd.log(string.format("tour state %s kinds=%d events=%d lcancel_window=%d", tostring(token), #gd.skill_kinds(), #lines, t.lcancel_window_frames))
end, "demo_state <token>")
