# Vanilla Charger

The Geno slice 5 fixture: a native Mario-reference define whose neutral special (ground and air) is a Lua callback with typed per-fighter state
(`lua/charger.lua`, state `charge` and `charged` declared in `geno.json`). Hold B to charge, release to strike; the dash speed and the hit's damage grow with the charge.
Text only (`geno.json`, one Lua module, two small move scripts); no disc data. Offline only. Wears Mario's model and clips; feel is unreviewed.
`"geno": 9` (the Lua keys need a format number; whether to mint 10 is the owner's call, see `docs/superpowers/plans/2026-10-07-geno-slice5-fighter-lua.md`).
Lesson stub: `docs/learn/geno-fighters/13-fighter-lua.md`. Reference: `docs/geno.md` section 23.
