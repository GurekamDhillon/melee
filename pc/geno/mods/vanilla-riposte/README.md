# Vanilla Riposte

The Geno slice 5 second fixture: a native Mario-reference define whose down special (ground and air) is a counter answered in Lua (`lua/riposte.lua`, slots `power`, `streak` and
`follow` declared in `geno.json`). Different in kind from the Vanilla Charger: it reacts to a hit rather than to the pad, and it runs in phases.

- **Parry** (stance): the engine's counter window (action frames 4 to 24, `geno.json` `counter`) drops the hit and sends the fighter to Riposte. Letting go of B after frame 8 cancels the stance; running out the
  window is a whiff; both break the streak.
- **Riposte** (the answer): damage 1.5 x the countered hit, clamped 9 to 30, plus one per hit of the streak (up to +3); turns around if the hit came from behind. Press A in action frames 6 to 16 to chain into
- **Follow**: a second strike for 0.6 x the riposte's damage. One follow-up per riposte.

Text only (`geno.json`, one Lua module, three small move scripts); no disc data. Offline only. Wears Mario's model and clips; feel is unreviewed.
`"geno": 10` (the format number fighter Lua needs). Demo and headless proof: `pc/scripts/examples/demos/geno-define-riposte/`. Lesson: `docs/learn/geno-fighters/13-fighter-lua.md`. Reference: `docs/geno.md` section 23.7.
