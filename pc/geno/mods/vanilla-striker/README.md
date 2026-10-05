# Vanilla Striker

Second Geno native-definition fixture (slice 2, `geno: 7`): a rushdown fighter with its own move set,
still wearing Mario's model and clips (looks and feel are unreviewed). Needs only the vanilla disc.

- Attributes: fast dash, light weight, low gravity, three jumps, short landing lags, own jab windows.
- 26 attack rows own: 3 jabs, dash attack, 3 forward tilts, up/down tilt, 3 forward smashes (up smash is a
  REHIT multi-hit), down smash, 5 aerials (down air is a LINK drag-down), grab, dash grab, pummel, 4 throws.
- Eight specials bound to six Geno states: neutral = charge (hold B, release dash-strike, uses special
  attribute words 1-2), side = lunge, up = steerable rise, down = counter then strike.
- `fx/` holds a small effect (StrikerSparks, a copy of the repo's demo effect) bound to jab 1.

Check it: `python -m tools.geno.check <this folder>`; ownership: `python -m tools.geno.report <this folder> --frames`.
The move scripts are generated text (`moves/*.genoasm` -> `.words`); edit and re-assemble with `tools.geno.asm`.
