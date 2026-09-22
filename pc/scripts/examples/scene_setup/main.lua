-- scene_setup: a scripted scenario. "fdtrain [percent]" in the console (or over the console
-- socket) jumps straight into Training - Fox (P1) against a Falco CPU on Final Destination -
-- waits for the match to be live, then puts both fighters at the given percent.
--
-- gd.scene_launch takes a table (below) or the MELEE_SCENE text form
-- ("mode=training;p1=fox;p2=falco/cpu;stage=fd"); _research/scene-launch.md has every field.

local function setup(percent)
  gd.scene_launch{ mode = "training", p1 = "fox", p2 = "falco/cpu", stage = "fd" }
  gd.run(function()
    -- wait for the old scene to end and the match to start (at most 20 s)
    if not gd.wait_until(function() return gd.match().active end, 1200) then
      gd.log("fdtrain: the match never started")
      return
    end
    gd.wait(30) -- let the entry animations settle
    for _, p in ipairs(gd.players()) do gd.set_percent(p.port, percent) end
    gd.log(string.format("fdtrain: ready at %d%%", percent))
  end)
end

gd.command("fdtrain", function(arg) setup(tonumber(arg) or 60) end,
  "Training: Fox vs Falco CPU on FD at <percent> (default 60)")
