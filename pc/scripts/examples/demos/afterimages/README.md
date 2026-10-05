# Afterimages

After the integrator rebuilds Aurora and the game, start an offline LAB match:
`mode=lab;stage=fd;p1=fox/hu;p2=marth/cpu0`. Load `demos/afterimages` through the demo catalogue
or `load <repo>/melee/pc/scripts/examples/demos/afterimages` in the console.

Afterimages are earned, never always on: land an aerial on a standing opponent and L-cancel the landing, and a speed
status (`gd.fighter_timed_status` channel 1, 120 frames) starts; the emitter is bound to it (`gd.afterimage_bind`), so blue
afterimages run for exactly that long. Miss the L-cancel and none show. D toggles a labelled continuous debug view.
C: copies 1..12; T: lifetime up to 60; G: spacing; F: world/follow; I: intensity;
move P1 (six blue copies, additive). `afterimage_probe` prints pose, replay,
warm and failure counters. The overlay reports live parameters. Preparation runs before intensity
is enabled; unloading `demo_afterimages` releases owned emitters. This source has not
been run or visually accepted. Changing a fighter material/costume may require
another warm pass. Copies are unfogged; silhouette/gradient need no material
textures. A sampled mutable copy texture degrades own-look to a silhouette.
Held items are retained with the whole pose; warm after equipping a new item.
