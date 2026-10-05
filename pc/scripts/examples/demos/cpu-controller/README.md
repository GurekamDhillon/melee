CPU virtual controller

Offline LAB demo of `gd.cpu_mode(port, "script")`, `gd.cpu_macro`, `gd.cpu_script` and `gd.cpu_goto`.
Launch Battlefield with a CPU as port 2 (any fighter; the timings are read from its attributes):

    MELEE_SCENE="mode=lab;stage=bf;p1=fox/hu;p2=fox/cpu0/idle"

Port 2 puts itself in script mode and, in order: wavedashes right and left, short-hops a
nair and L-cancels it, then walks to the marked spot on the top platform (`gd.cpu_goto`).
The on-screen line says which step it is on and what the controller reports. The console command `cpu_demo` runs it again.
Everything the CPU does is written to its virtual controller (`fp->cpu`), inside the simulation, so
savestates and the LAB's rewind carry it. Verified by state readback in the game on 2026-10-04;
nothing here was judged by eye.
