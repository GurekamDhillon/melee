# Offline simulation checkpoint demo

Load `demo_sim_checkpoint` in a fresh offline LAB match. P1 percent follows a
0..99 clock; run speed cycles from 1x to almost 1.5x. This is intentionally a
checkpoint demonstration, not a combat modifier.

Enable `gd.history(600)`, wait several seconds, then `gd.step_back(90)` and compare
the restored clock, percent and `gd.fighter_mod(1).run_speed`. Try
`gd.rewind_test(60)` with the demo active. Native helper fixtures pass; the full
LAB snapshot/hash check and screen acceptance require the integrator build.
Unloading clears the owned overlay/checkpoint and invalidates rewind history.
