# Afterimages

After the integrator rebuilds Aurora and the game, start an offline LAB match:
`mode=lab;stage=fd;p1=fox/hu;p2=marth/cpu0`. Load `demos/afterimages` through the demo catalogue
or `load <repo>/melee/pc/scripts/examples/demos/afterimages` in the console.

C: copies 1..12; T: lifetime up to 60; G: spacing; F: world/follow; I: intensity;
move P1. Extreme integrator defaults are preserved (six cyan copies, additive,
always, intensity one after preparation). `afterimage_probe` prints pose, replay,
warm and failure counters. The overlay reports live parameters. Preparation runs before intensity
is enabled; unloading `demo_afterimages` releases owned emitters. This source has not
been run or visually accepted. Changing a fighter material/costume may require
another warm pass. Copies are unfogged; silhouette/gradient need no material
textures. A sampled mutable copy texture degrades own-look to a silhouette.
Held items are retained with the whole pose; warm after equipping a new item.
