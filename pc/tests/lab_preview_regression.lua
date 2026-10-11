-- Windows integration regression. Two identical fighters, idle CPUs, FD, no LAB
-- script (it would take over the preview/camera). Run via MELEE_PAD_SCRIPT in a
-- LAB scene; Goku's REMAKE package reproduces both original failures.
-- Observe the first paused pose BEFORE Fighter_procMap can mask the Entry scale.
local phase, early, row, ticks, failed, since = 'entry', nil, 2, 0, false, 0
local function check(ok, why)
  gd.log('lab-preview-test: ' .. (ok and 'PASS ' or 'FAIL ') .. why)
  failed = failed or not ok
end
local function root_scale(port)
  for _, j in ipairs(gd.joints(port)) do
    if j.valid and j.parent == -1 then return {j.scale_x, j.scale_y, j.scale_z} end
  end
  error('no root joint')
end
function on_frame()
  local m = gd.match()
  if not m.active then return end
  if phase == 'entry' and m.frame == 9 then
    check(gd.player(1).action == 323, 'early preview interrupts EntryStart')
    assert(gd.anim_preview(1, row, 1))
    phase, ticks = 'early_pose', 0
  elseif phase == 'reference_wait' and m.frame >= 90 then
    assert(gd.anim_preview(2, row, 1))
    phase, ticks = 'reference_pose', 0
  elseif phase == 'damage' then
    if gd.player(2).percent ~= 37 then
      check(false, 'close camera changed parked P2 percent at frame ' .. m.frame .. ' to ' .. gd.player(2).percent)
      phase = 'done'
      gd.quit()
    elseif m.frame >= 1500 then
      check(true, 'parked P2 keeps 37 percent through 1400+ close-camera frames')
      gd.camera_attach(60)
      phase, since = 'camera_return', m.frame
    end
  elseif phase == 'camera_return' and m.frame >= since + 120 then
      check(gd.player(2).percent == 37, 'blending back to the game camera preserves percent')
      gd.camera_set { eye = {x=-20, y=11, z=62}, interest = {x=-20, y=9, z=0}, fov=30 }
      assert(gd.hit(2, {from=1, damage=3, angle=0, kbg=0, bkb=0}))
      check(gd.player(2).percent > 37, 'real damage still applies with a scripted camera')
      gd.set_percent(2, 37)
      assert(gd.anim_preview(2, row, 1))
      local b = gd.stage_bounds()
      gd.teleport(2, (b.camera.right + b.blast.right) / 2, 0)
      phase, since = 'resume_offstage', m.frame
  elseif phase == 'offstage' and m.frame >= since + 120 then
      check(gd.player(2).percent > 37, 'stage-boundary offscreen damage still applies with a scripted camera')
      gd.camera_attach(0)
      gd.set_percent(2, 37)
      phase, since = 'retail_offstage', m.frame
  elseif phase == 'retail_offstage' and m.frame >= since + 120 then
      check(gd.player(2).percent > 37, 'ordinary-camera magnifier damage still applies')
      check(not failed, 'all preview regressions')
      phase = 'done'
      gd.quit()
  end
end
function on_tick()
  if phase ~= 'early_pose' and phase ~= 'reference_pose' then return end
  ticks = ticks + 1
  if ticks < 2 then return end -- pending anim_preview is applied after on_tick
  if phase == 'early_pose' then
    early = root_scale(1)
    gd.log(string.format('lab-preview-test: early paused TopN scale=%.6f,%.6f,%.6f', table.unpack(early)))
    assert(gd.anim_preview(1, nil))
    phase = 'resume_reference'
  else
    local normal = root_scale(2)
    for axis = 1, 3 do
      check(math.abs(early[axis] - normal[axis]) < 0.00001,
        string.format('Entry preview scale axis %d %.6f matches completed Entry %.6f', axis, early[axis], normal[axis]))
    end
    assert(gd.anim_preview(2, nil))
    gd.teleport(1, -20, 0)
    gd.teleport(2, 25, 0)
    gd.set_percent(2, 37)
    gd.camera_set { eye = {x=-20, y=11, z=62}, interest = {x=-20, y=9, z=0}, fov=30 }
    phase = 'resume_damage'
  end
end
-- The requests above intentionally keep one tick paused to apply their changes.
local tick = on_tick
function on_tick()
  if phase == 'resume_reference' then phase = 'reference_wait' gd.resume()
  elseif phase == 'resume_damage' then phase = 'damage' gd.resume()
  elseif phase == 'resume_offstage' then phase = 'offstage' gd.resume()
  else tick() end
end
