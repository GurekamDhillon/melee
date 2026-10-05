# Vanilla Caster

Third Geno native-definition fixture (slice 3, `geno: 8`): a define with its own article, its own effect and named sounds. Needs only the vanilla disc.

- Neutral special (ground and air): Geno state `CastN`, script `CALL geno.article.spawn 0`; it throws `CasterBolt`, an invisible article (no model) whose look is the
  original effect package `fx/CasterGlow` (a renamed copy of the repo's demo squares effect), 5 damage, tag `projectile`, 70 frames of life.
- `sounds` is the named-sound table the article refers to (`spawn_sound`, `end_sound`): names for engine sound ids the game already plays; no audio is shipped.
- The other specials run the donor's code (`python -m tools.geno.report` lists them as `DONOR`); the fixture exists to prove the article machinery for defines, not a move set.

Check it: `python -m tools.geno.check <this folder>`; `python -m tools.geno.report <this folder>` lists the resources. Looks, sounds and feel are unreviewed.
