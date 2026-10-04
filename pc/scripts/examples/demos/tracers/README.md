# Tracers

After the integrator rebuilds Aurora and the game, start an offline LAB match:
`mode=lab;stage=fd;p1=fox/hu;p2=marth/cpu0`. Load `demos/tracers` through the demo catalogue
or `load <repo>/melee/pc/scripts/examples/demos/tracers` in the console.

S: shader and colour; W: width up to 24; L: length up to 60;
A: hand/element hitboxes; attack P1. The integrator's exaggerated defaults remain
width six, length 31, smoothing eight, intensity one and distinct shader colours.
The overlay reports live parameters. Preparation runs before intensity
is enabled; unloading `demo_tracers` releases owned emitters. This source has not
been run or visually accepted. Changing a fighter material/costume may require
another warm pass; unsupported captures are skipped whole.
