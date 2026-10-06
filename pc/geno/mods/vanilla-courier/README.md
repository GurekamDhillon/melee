# Vanilla Courier (Geno slice 4: own model, skeleton, clips, hurtboxes)

**State of this fixture (2026-10-05): step (a), a static stand-in.** The folder holds text only. The engine assets are
built from the original art in `ports/vanilla-original/` by ONE command and are git-ignored here:

    tools/geno/build_courier.sh --install        # writes the .dat files into files/ (about 25 s with the art already built)

What it needs (a builder, not a player): Python 3 with numpy, scipy and Pillow; the .NET 8 SDK; the HSDLib checkout at
`experiment/tooling/HSDLib` (Ploaj/HSDLib); Blender only to regenerate `ports/vanilla-original/out/` (`--art-rebuild`, headless).
**A player needs none of it.** The outputs are original data (no game bytes are read at any point), small (four 107 KB
models, a 1.5 MB animation bank) and may be zipped into a mod archive; nothing is committed, so the choice to ship is made
when a release is cut.

What loads today: `geno.json` here is a format 9 define with `base: "none"`: the Courier is a fighter on its own model, skeleton, clips,
parts table and hurtboxes, with the Striker's 32 moves retargeted onto its limbs (`retarget.json`: roles, per-hitbox joint / offset / size,
and a start `delay`; regenerate with `python -m tools.geno.courier_moves`). See `melee/docs/geno.md` 22.4-22.5.

Known gaps (slice 4d): the ftData pointer fields still the donor's are `x18 x1C x24 x2C x48 x5C` (the census line in the log names
them); the metal box has no metal model; ECB offsets, IK lengths, ledge snap and the camera box are the donor's numbers; the held victim
and the shield bubble share one joint (`ThrowN`), so the bubble is centred by clearing that joint's translation; item swings and
Kirby's copy were not exercised; Classic, Versus to the results, stage switching and six slots were not run; the grab hitbox is the
donor's raw words at TopN; dair's clip keeps the feet low and still (see `_build/audit-20261003/geno-slice4d/ART-REQUEST.md`).

Credit: Blender and glTF 2.0 (Khronos) made the art; HSDLib (Ploaj) writes the model files; see `CREDITS.md`.
