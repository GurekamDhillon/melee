-- Crits demo: one feature, the engine's seeded crit decision (gd.crit, gd.crit_force, gd.crit_seed, on_crit).
-- Offline LAB, vanilla Battlefield, P1 yours, P2 an idle CPU. P1 crits 25% of the time for x1.5..x2.5 (damage only,
-- with a little extra launch); aerials crit half the time for x2.5. The overlay shows the last crit: base and final
-- damage, strength (how hard it was against P1's strongest possible crit), the contact point and the victim's hitlag.
-- Console: crit_force <n> makes the next n hits crit; crit_seed <n> restarts the generator (same seed, same crits).
local last, count, hits = "no crit yet", 0, 0
local function configure()
  gd.crit(1, {chance = 0.25, multiplier = 1.5, multiplier_max = 2.5, launch = 1.15, tags = {aerial = {chance = 0.5, multiplier = 2.5}}})
  gd.crit_seed(1)
  gd.log("crit config: " .. tostring(gd.crit(1) ~= nil))
end
function on_match_start() configure() end
function on_hit(attacker, victim, info) if attacker == 1 then hits = hits + 1 end end
function on_crit(c)
  count = count + 1
  last = string.format("crit #%d  %s  %.1f -> %.1f (x%.2f, +%.1f)  strength %.2f  hitlag %d  at (%.1f, %.1f)%s",
    count, c.move_tag, c.base_damage, c.final_damage, c.multiplier, c.added_damage, c.strength, c.hitlag_frames, c.x, c.y, c.forced and "  [forced]" or "")
  gd.log("on_crit " .. last)
end
gd.command("crit_force", function(n) gd.crit_force(1, tonumber(n) or 1) end, "crit_force <n>: the next n eligible hits by P1 crit")
gd.command("crit_seed", function(n) gd.crit_seed(tonumber(n) or 1) end, "crit_seed <n>: restart the crit generator")
function on_draw()
  local a = gd.safe_area()
  local c = gd.crit(1)
  gd.fill(a.x + 12, a.y + 12, math.min(a.w - 24, 900), 70, 0x101827dd)
  gd.text(a.x + 24, a.y + 22, "Crits: " .. last, 0xffd369ff, 1.0)
  gd.text(a.x + 24, a.y + 46, c and string.format("P1 hits %d, crits %d, force %d, max multiplier %.2f", c.hits, c.crits, c.force, c.max_multiplier) or "not configured (start a match)")
end
gd.command("demo_state", function(token) gd.log("tour state " .. tostring(token) .. " crits=" .. count .. " hits=" .. hits) end, "demo_state <token>")
