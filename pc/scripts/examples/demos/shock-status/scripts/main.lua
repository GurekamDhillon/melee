-- Shock demo: one feature, the native Shock status (gd.shock, on_shock / on_shock_hit / on_shock_end).
-- Offline LAB, vanilla Battlefield, P1 yours, P2 an idle CPU. Every hit P1 lands re-shocks P2 for the next hit: 1 charge,
-- hitstun x2, 5 seconds. The overlay shows the status and the last hit it changed (base hitstun frames, frames added).
-- Console: shock_set <frames> <charges> <hitstun> [bonus], shock_clear.
local last, hits = "no shocked hit yet", 0
function on_hit(attacker, victim, info)
  if attacker == 1 and victim == 2 then
    hits = hits + 1
    if not gd.shock(2) then gd.shock(2, {frames = 300, charges = 1, hitstun = 2.0}) end
  end
end
function on_shock(s) gd.log(string.format("on_shock entity %d: %d frames, %d charges, x%.2f +%d", s.entity, s.frames, s.charges, s.hitstun_multiplier, s.bonus)) end
function on_shock_hit(s)
  last = string.format("hit: hitstun %d + %d (x%.2f), %d charges left", s.base_hitstun, s.extra_hitstun, s.hitstun_multiplier, s.charges_left)
  gd.log("on_shock_hit " .. last)
end
function on_shock_end(s) gd.log("on_shock_end entity " .. s.entity .. ": " .. s.reason .. " after " .. s.hits .. " hit(s)") end
gd.command("shock_set", function(f, c, h, b)
  gd.shock(2, {frames = tonumber(f) or 300, charges = tonumber(c) or 1, hitstun = tonumber(h) or 2.0, bonus = tonumber(b) or 0})
end, "shock_set <frames> <charges> <hitstun> [bonus]: shock P2")
gd.command("shock_clear", function() gd.shock(2, nil) end, "shock_clear: remove P2's Shock")
function on_draw()
  local a = gd.safe_area()
  local s = gd.shock(2)
  gd.fill(a.x + 12, a.y + 12, math.min(a.w - 24, 900), 70, 0x101827dd)
  gd.text(a.x + 24, a.y + 22, "Shock: " .. last, 0x9fd8ffff, 1.0)
  gd.text(a.x + 24, a.y + 46, s and string.format("P2 shocked: %d frames, %d charges, x%.2f hitstun, +%d flat (P1 hits %d)", s.frames, s.charges, s.hitstun_multiplier, s.bonus, hits) or string.format("P2 not shocked (P1 hits %d)", hits))
end
