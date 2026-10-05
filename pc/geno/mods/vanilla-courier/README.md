# Vanilla Courier (Geno slice 4: own model, skeleton, clips, hurtboxes)

**State of this fixture (2026-10-05): step (a), a static stand-in.** The folder holds text only. The engine assets are
built from the original art in `ports/vanilla-original/` by ONE command and are git-ignored here:

    tools/geno/build_courier.sh --install        # writes the .dat files into files/ (about 25 s with the art already built)

What it needs (a builder, not a player): Python 3 with numpy, scipy and Pillow; the .NET 8 SDK; the HSDLib checkout at
`experiment/tooling/HSDLib` (Ploaj/HSDLib); Blender only to regenerate `ports/vanilla-original/out/` (`--art-rebuild`, headless).
**A player needs none of it.** The outputs are original data (no game bytes are read at any point), small (four 107 KB
models, a 1.5 MB animation bank) and may be zipped into a mod archive; nothing is committed, so the choice to ship is made
when a release is cut.

What loads today: `geno.json` here is a v8 define on the Mario preset whose neutral special spawns the Courier's MODEL
as an article (`articles[0].model`), which proves the converter's output draws on the vanilla disc through the palette POBJ
path, textured, one piece (see `melee/docs/geno.md` 22.3). It is NOT yet a fighter: it has no clips playing on this
skeleton, no hurtboxes, no `base: "none"`. The full fixture replaces this `geno.json` when the engine side (22.3 "What is
left") lands: the Striker's move set with `ANIM_RATE` 1.0 on the Courier's own bank.

Credit: Blender and glTF 2.0 (Khronos) made the art; HSDLib (Ploaj) writes the model files; see `CREDITS.md`.
