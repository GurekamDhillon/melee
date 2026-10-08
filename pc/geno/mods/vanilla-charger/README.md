# Vanilla Charger

The Geno slice 5 fixture: a native Mario-reference define whose neutral special (ground and air) is a Lua callback with typed per-fighter state
(`lua/charger.lua`, state `charge` and `charged` declared in `geno.json`). Hold B to charge, release to strike; the dash speed and the hit's damage grow with the charge.
Text only (`geno.json`, one Lua module, two small move scripts); no disc data. Offline only. Wears Mario's model and clips; feel is unreviewed.
`"geno": 10` (the format number the Lua keys need; geno.json files below 10 refuse a `lua` block).
Lesson stub: `docs/learn/geno-fighters/13-fighter-lua.md`. Reference: `docs/geno.md` section 23.
