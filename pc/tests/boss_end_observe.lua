-- Optional MELEE_SCRIPT companion: exercises the hook's pending-event/Sleep
-- wait with no hold. Keep absent for the primary no-hook/no-mod lane.
function on_boss_defeated(e)
    gd.log("BOSSEND observed " .. e.kind .. " port=" .. e.port)
end
