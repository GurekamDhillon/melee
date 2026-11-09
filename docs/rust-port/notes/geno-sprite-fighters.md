# Geno sprite fighters (plan, 2026-11-08)

> 2026-11-09 renumber: "geno": 11 is already the combo/meter format and the wasm fighter block takes 12, so the sprite block is **13** (it builds on the wasm block).

Planning note only: nothing here is built. Goal: a Geno `define` whose look is 2D sprites (PNG) rather than a
skinned 3D model. The fighter still plays by Melee's rules (Geno scope rule, docs/geno.md 1): Melee's knockback,
hitstun, shield, ledge, tech and air-dodge code runs unchanged. Reference designs (structure only, nothing copied):
Rivals of Aether 1 (GameMaker sprite strips `name_stripN.png` + per-attack `set_window_value` / `set_hitbox_value`
scripts), Super Smash Flash 2 (Flash timelines with hit/hurt shapes per frame), MUGEN (`.air` animations with
per-frame `Clsn1` attack / `Clsn2` hurt boxes, `.sff` sprites, `.cns` state controllers, `.snd`).

Builds on: base `"none"` defines (geno.md 22.3-22.9, notes/geno.md "Native defines (ported)"), define audio
(`.gnsnd`, notes/geno.md "Define audio"), costumes as colour sets (geno.md 22.6), the effects IR (geno.md 20),
melee_gx (notes/render.md), the netplay identity of base "none" files (`geno_host::netplay_identity`).

**MUGEN importer: considered and left out of scope** (user decision 2026-11-08). "Smudging" a character into Geno
is a manual authoring job; the format below is designed so that job is easy by hand.

## 1. What a sprite fighter needs inside Melee's 3D engine

### 1.1 Drawing

| topic | recommendation |
|---|---|
| quad orientation | **Flat quad in the stage plane (Z = the fighter's Z, normally 0), facing +Z**, not a full camera billboard. Melee's camera only pans/zooms with mild pitch/yaw; a stage-plane quad keeps hitbox overlay, platforms and other fighters lined up exactly. Option: Y-axis billboard (yaw only) per define, `"billboard": "y"`, for pause-camera cases. Pause camera orbiting shows the card edge-on; accept, or turn the quad to the camera only while paused (presentation only) |
| anchor | each frame has a pivot (feet origin) in pixels; the quad is placed at `fp->cur_pos` + pivot offset. Pixels -> world: `"px_per_unit"` per define (default chosen so a 64 px tall sprite ~ Mario's height; Mario ECB top ~ 10-12 units) |
| scaling | `"scale"` multiplies px_per_unit; Melee's own `fp->x34_scale` (mushroom, Bowser-size, Kirby-size items) multiplies it again |
| facing flip | mirror U (swap u0/u1) when `fp->facing_dir < 0`; boxes mirror X in the sim. Optional per-frame `"no_flip"` for text/logos |
| filtering | nearest-neighbour sampler by default (pixel art), `"filter": "linear"` option; integer-ish zoom is not enforceable under Melee's camera, so offer an optional pixel-snap shader (snap UV to texel centres) |
| draw order / depth | alpha-tested (cutout, alpha < 0.5 discarded) write depth so the sprite sorts against stage geometry; semi-transparent pixels go to a second blended pass. Small Z bias toward camera per port to stop two sprites z-fighting |
| palettes / costumes | sprites authored as indexed PNG (<= 256 colours); palette swap in the shader from a palette texture row per costume (MUGEN `.act` / Rivals colour slots concept). RGBA PNGs fall back to the existing define "costumes as colour sets" tint, or one full sheet per costume. Team colours = palette rows |
| lighting | default unlit (sprites carry their own shading) with the stage's ambient tint multiplied in lightly (`"light": 0..1`, default 0.25) so a dark stage darkens them. No normal maps in v1 (option later) |
| shadow | Melee's fighter shadow is a projected model; for a sprite use a blob/ellipse decal at the ground point under the ECB bottom, scaled by sprite width (presentation only) |
| hit flash / colour overlays | Melee's `ColorOverlay` (hit red, invincible flash, shield-break, charge glow, Fox's reflector tint) is a colour/alpha state the sim already computes; the sprite shader takes the same overlay colour + blend as uniforms so all existing effects apply |
| hitlag shake | Melee shakes the model during hitlag; apply the same offset to the quad |
| invisibility / Hidden | honour the model-visibility flags (ledge refresh, star KO, entry platform) |
| star KO / screen KO | star KO shrinks the model far behind; quad scales with it. Screen KO uses the same model pose: use the current frame |
| Melee's camera | camera reads ECB/position, not the model, so nothing changes. Character select / results / stock icons use the define art paths that base "none" already has (`DefineArtOpen`), with a sprite frame as a fallback portrait |
| effects | Melee effects attach to joints; a sprite fighter has a tiny virtual skeleton (below) so effects, items held and grab points have joints |

**Virtual skeleton.** Melee code reads bones everywhere (held item at hand joint, grab at `XRotN`, throw release,
hitbox positions, effects, ledge grab). A sprite fighter declares a fixed small joint set (TopN, TransN, hip, head,
L/R hand, L/R foot, ItemHold) and **per-frame joint points in pixels** (optional; missing = last frame's or the
pivot). The define builds a real joint tree (`GenoDefine_BuildOwn` already builds own joint tables for base "none")
whose joint transforms are set from the current frame each tick. Those positions are sim data (hitboxes and grabs
read them), so they live in the define data, not in the renderer.

### 1.2 Physics / ECB

ECB in Melee comes from bone positions (`ftCo` ECB update over chosen joints) with min sizes from attributes. Options:

| option | note |
|---|---|
| A (recommended) | **ECB from the virtual skeleton**: the ECB-joint list points at the virtual joints, so per-frame joint points drive the ECB exactly the way bones do. Same code path, no new physics |
| B | fixed ECB from attributes (diamond width/height) with optional per-state override (`"ecb": [bottom, top, half_width]`). Simple sample-fighter mode |
| not | derive ECB from sprite alpha bounds: unstable frame to frame, and art edits would change gameplay silently |

All movement attributes (walk/dash/run, jump squat, gravity, fall speeds, air mobility, weight, shield size, ledge
snap box) are the existing 67 define attributes, unchanged.

### 1.3 Hurtboxes and hitboxes per frame

Melee semantics, not MUGEN's:

| Melee concept | sprite format |
|---|---|
| hurtbox | Melee hurtboxes are capsules on bones (`HurtCapsule`, 3 per-capsule states: normal, invincible, intangible). Sprite frames give **axis-aligned boxes in pixels** (MUGEN Clsn2 concept); the loader turns each box into a capsule (two endpoints along the long axis, radius = half the short side) attached to the TransN joint with a per-frame offset. Max per frame = Melee's hurt slot count (15). Body-state (intangible/invincible) is Melee's state code (dodges, ledge) and still applies |
| hitbox | Melee hitboxes are spheres (`HitCapsule`, up to 4 active slots, interpolated between frames). Sprite boxes are **circles** (x, y, r) in pixels, authored with Melee fields directly: `dmg`, `angle` (incl. 361 Sakurai), `kbg`, `wbk`, `bkb`, `element`, `shield_dmg`, `sfx`, `hitlag mult` (via element/flags), `slot` (0-3), `id` group for rehit rules, `ground/air` flags, `stun` (Geno v5.5 `HBSTUN`), `flags` (`HBFLAGS`). A rectangle box in authoring is accepted and converted to the circle covering it plus a warning (a rectangle cannot be matched exactly) |
| hitbox life | a box exists from its `on` frame to its `off` frame, exactly like Melee subaction `Hitbox` / `ClearHitboxes`. Interpolation between ticks (Melee's swept capsule) comes free because positions are set each tick |
| SDI, hitlag, shield push, staling, crouch cancel, DI | all Melee's own code: nothing to author |
| grab boxes | Melee grab = a hitbox on a grab slot; format has `"grab": true` boxes in the grab subaction |
| reflect / absorb / shield-style | existing Geno/Melee fields (`counter` windows v5, reflect flags) where available; not in the first slices |

Implementation route: **compile sprite frames to Geno script words** (Melee subaction commands: `Hitbox`,
`ClearHitboxes`, `SetHurtState`, `PlaySFX`, `Wait`, the Geno escape for frame changes). Then the existing ftAction
interpreter, rollback snapshot and NHV1 hash already cover them. The sprite frame index is one more Geno variable
(`geno.sprite_frame`, a new engine value) that the script sets each frame and the renderer reads.

### 1.4 States and moves

A sprite fighter is a base `"none"` define. Melee's action-state machine is kept; the fighter supplies **animations
and scripts per action**, the state code is common.

| group | sprite char must provide | inherits (common code) |
|---|---|---|
| idle, walk (slow/mid/fast), dash, run, turn, run-brake, squat, squat-wait, jump squat, jump F/B, double jump, fall, land, landing lag, helpless fall | animation (looping or not) | all physics and transitions |
| shield (on/hold/off), shield stun, roll F/B, spot dodge, air dodge | animation; may reuse one frame for the shield (Melee draws the bubble itself) | intangibility frames, shield bubble, shield break |
| ledge: catch, wait, getup, attack, roll, jump (fast/slow) | animation + ledge-hand point (virtual joint) | ledge snap, invincibility, timings |
| tech in place / roll, missed tech, jab reset, getup attack | animation | all |
| damage (hi/mid/lo, fly, tumble), dead/KO | 2-4 hurt animations reused across Melee's ~20 damage states (table maps unmapped states to the nearest) | knockback, hitstun |
| grab, pummel, throws F/B/U/D | anim + grab/throw scripts; throw release frame and knockback data are Melee throw hitbox fields | grab-break, mash |
| **being thrown** | Melee plays the thrower's throw anim on the victim's skeleton, which a sprite cannot follow. Use `thrown_as` (geno.md 22.10): victim plays its own short "thrown" sprite set (thrown_hi/lo/spin) positioned by the thrower's grab joint; a 3D fighter throwing a sprite one works the same way | throw physics |
| attacks: jab 1-3/rapid, tilts, smashes (charge), dash attack, aerials + landing lag, specials | animation + per-frame boxes + events | IASA / interrupt rules from script `Iasa` |
| specials | Geno states (`geno.ground`/`geno.air`, behaviours, articles for projectiles) as for any define | — |
| entry, taunt, victory poses, sleep/bury/freeze/etc. | optional; fall back to idle frame with a log | — |

The loader reports every common action with no animation (`python -m tools.geno.report` style) and falls back to
idle so a half-made character is playable.

### 1.5 Sound

Define `sounds` + `.gnsnd` clips already ported (notes/geno.md "Define audio"). Sprite format events reference them by
name (`"sfx": "swing1"`), plus voice clips per Melee voice slot (hurt, KO, jump) using `DefineVoicePlayOwner`.
Authoring accepts WAV/OGG; a build step converts to `.gnsnd`.

## 2. Format: Geno define for sprite fighters

Proposed `"geno": 13` (next free number; additive). One `define` with `"base": "none"` and a `"sprite"` block;
everything else is the existing define keys (attributes, sounds, specials, states, articles, costumes).

```json
{"geno": 13, "fighters": [{
  "define": {"key": "sample-sprite", "name": "Sample Sprite", "base": "none", "common": "melee.common.v1"},
  "attributes": {"walk_max_vel": 1.4, "weight": 90},
  "sprite": {
    "px_per_unit": 6.0, "filter": "nearest", "light": 0.25,
    "palettes": "art/palettes.png",
    "sheets": {"main": "art/main.png"},
    "ecb": {"mode": "joints"},
    "anims": {
      "Wait":   {"strip": "art/idle_strip6.png", "fps": 10, "loop": true, "pivot": [32, 63]},
      "Attack11": {
        "strip": "art/jab_strip5.png", "pivot": [32, 63],
        "frames": [
          {"len": 2, "hurt": [[20,10,44,63]]},
          {"len": 3, "hurt": [[20,10,48,63]],
           "hit": [{"slot":0,"x":52,"y":30,"r":7,"dmg":3,"angle":361,"kbg":50,"bkb":10,"element":"normal","sfx":"punch_s"}],
           "joints": {"RHand": [54, 30]}},
          {"len": 2, "clear_hits": true, "iasa": true}
        ]}
    }
  }
}]}
```

| key | meaning |
|---|---|
| anim key | a Melee action name (`Wait`, `Attack11`, `AttackAirF`, `CliffWait`, ...; the common subaction names) or a Geno state name |
| `strip` | Rivals-style horizontal strip; frame count from `_stripN` in the name or `"count"`; or `"sheet"` + `"rects"` for an atlas |
| `frames[].len` | ticks this image is shown (MUGEN `.air` duration concept); omitted frames = uniform from `fps` |
| boxes | pixels in the frame, origin top-left, `[x0,y0,x1,y1]` for hurt; hit circles `x,y,r`. A box list carries forward until changed (MUGEN `Clsn2Default` concept) so most frames have no box data |
| events | `sfx`, `voice`, `fx` (Geno effect package), `velocity`/`add_velocity`, `iasa`, `clear_hits`, `intangible`, `spawn` (article), `landing_lag` |
| `joints` | virtual-joint points per frame, carried forward |
| `hit_defaults` per anim | shared Melee fields so each hit line stays short |

**Build step** (`tools/geno/sprite_build.py`, offline, not in the engine): packs strips into an atlas (`.gxtex`
pages, the format base "none" art already uses), quantises/validates palettes, converts boxes to capsules/spheres in
world units, emits the compiled script words and a `sprite.bin` frame table. The engine loads the compiled form; the
JSON is the source. Same split as the Blender pipeline (geno.md 22.9).

**Authoring paths**

1. Hand-authored JSON + PNG strips (the primary path; a jab is ~15 lines). This is **our own format**, not a
   Rivals 1 clone. Where it aims to beat Rivals 1 / SSF2 / MUGEN for authoring:
   - one declarative file per move (frames, boxes, events together) instead of Rivals' split between a sprite strip,
     `set_window_value` index tables and `set_hitbox_value` calls with numeric constants; MUGEN splits the same move
     over `.air`, `.cns` and `.sff`;
   - boxes carry forward between frames and per-anim `hit_defaults`, so most frames are one line;
   - named Melee fields (`dmg`, `angle`, `bkb`, `kbg`) validated with errors naming file/move/frame;
   - one PNG per move or an atlas, any size; the build tool packs; no fixed strip naming required;
   - live LAB hot reload and box display, frame stepping, hash-checked determinism and online play out of the box;
   - capability beyond them: Melee's whole engine (DI/SDI, ledge, tech, shield, items, CPUs, rollback netplay),
     Geno states/behaviours/articles/effects for specials, palettes + costumes, virtual joints for items and grabs.
   Optional later, only if cheap: a Rivals-1 layout adapter (strip + window/hitbox tables -> this JSON). Not planned.
2. A folder layout for big characters: `moves/<name>.json` + PNGs; the build tool merges them into the define.
3. The LAB as the editor (later milestone): frame stepping already exists (14.x); add a box-drag overlay that writes
   the JSON and hot-reloads (LAB-only hot reload, layout changes refused).

Hand "smudging" a MUGEN/SSF2-style character: export the sprites yourself, name strips per Melee action, copy the
box rectangles (Clsn2 -> `hurt`, Clsn1 -> `hit`), then give each hit Melee numbers (MUGEN's damage/hitpause/velocity
do not translate; pick dmg/angle/bkb/kbg like a Melee move). MUGEN state controllers and physics are not used at all.

## 3. Engine work in Rust

| piece | where | notes |
|---|---|---|
| parse `sprite` block, compiled frame table | `melee_script::geno::define` | strict like other define keys; frame table is immutable after boot |
| compile to script words / virtual skeleton | `melee_script::geno` + `ftdata::GenoDefine_BuildOwn` | joint table from the fixed virtual set; per-frame joint writes in the define's anim callback (a new `anim` callback id) |
| sim state | `GenoState`: current anim id, frame index, frame timer | **all integer**, inside the Geno state block, so snapshot + `GenoDefine_StateDigest` cover it; the renderer only reads it. Box positions are floats derived from immutable data + fighter pos: same ops on every machine (no contraction-sensitive new math: use the existing vector helpers) |
| renderer | `melee_gx` (wgpu, Vulkan/DX12) | a sprite draw path outside GX emulation: one textured quad pipeline (palette lookup, overlay colour, alpha test + blended pass, pixel snap), drawn in the fighter draw slot with the GX camera matrices. Textures uploaded once per match from the atlas; nothing goes through the GX FIFO, so the Aurora parity oracle is untouched. Needs Opus |
| shadow, hit flash, hitlag shake | presentation, read from `Fighter` | no sim changes |
| mod-host API | `melee_modhost` | read-only `gd_fighter_sprite_anim/frame` accessors for wasm mods (LAB overlay, HUD). No write API in v1 |
| LAB | hitbox display (14.14) already draws Melee capsules, so sprite boxes show for free; box editor later |
| netplay | `geno_host::netplay_identity` | the compiled `sprite.bin`, script words and atlas bytes join the define's handshake part like other base "none" resource files (missing file drops the define). Atlas bytes hashed too: a costume or pixel change shows as a different fighter, which is stricter but simple. Option: hash art separately so peers may differ in art only (open question) |
| replays | define online id in replay metadata (already the open item 24.5) |
| performance | 4-8 quads per frame plus a few atlas pages (<= 4096^2) is negligible next to 3D fighters; sim cost = one script walk per frame like a normal fighter. Limit atlas size (e.g. 64 MiB per define) and frame count at load |

Tests: geno-tests for parse/compile; a coverage golden with the sample sprite fighter (CPU vs Fox) recorded twice
for determinism; rollback synctest scenario; screenshot check of the sprite draw (user verifies visuals).

## 4. Legal / asset policy

- We ship the format, the build tool, the engine support and **our own sample art** only (original, made for this
  project; no GD art per design rules, no ripped sprites).
- Third-party characters a player builds from others' sprites are the player's own local content. We never bundle,
  host, mirror or upload them, and no tool of ours downloads them.
- Docs do not link to rip sites or name specific copyrighted characters as targets.
- YouGame: anything we upload is ours only; the upload is checked by YouGame's pre-upload scan (`yougame:check`) and
  must contain no third-party character art or audio. A build that loads user mods must not ship any mods. Exact scan
  rules to be confirmed against YouGame's published check list before any upload (not read for this note).

## 5. Milestones

| # | slice | size | model |
|---|---|---|---|
| S1 | format + build tool: strips -> atlas, boxes, compiled words; schema + validator | M | Sonnet |
| S2 | engine: sprite block parse, frame state in GenoState, virtual skeleton, ECB mode B (fixed) | M | Opus (define/ftdata seams) |
| S3 | melee_gx sprite quad pipeline: stage-plane quad, flip, nearest filter, alpha test, overlay colour | M | Opus |
| **S4 = first playable** | one hand-made sample fighter: idle, walk, dash, run, jump, fall, land, shield, **jab**; boxes visible in LAB; CPU match deterministic twice | S | Sonnet (art + JSON), Opus review |
| S5 | full common-action coverage with fallbacks, ledge, tech, damage, being thrown via `thrown_as`, grabs/throws | L | Sonnet, Opus for thrown_as |
| S6 | palettes/costumes, shadow, hitlag shake, light tint, pixel snap | M | Opus |
| S7 | netplay identity + rollback synctest + replay metadata | S | Sonnet |
| S8 | multi-file folder layout, sounds/voices from WAV | S | Sonnet |
| S9 | ECB from virtual joints (mode A), specials via Geno states/articles | M | Opus |
| S10 | LAB box editor + hot reload | L | Opus (UI) |

Risks: virtual-skeleton coverage (retail code reading a joint the set lacks: log + map to TransN); being-thrown
looks crude; hurt rectangles -> capsules is approximate (show it in the LAB); pause camera reveals flat cards;
ripped-art temptation in community content (policy above); depth sorting of semi-transparent pixels.

## 6. Open questions

1. Stage-plane quad (recommended) or full camera-facing billboard as the default?
2. Pixel-art only (nearest + palettes) or also hi-res painted sprites (linear, RGBA) in v1?
3. Should peers be allowed to differ in art only (separate art hash), or must art match byte for byte online?
4. Who makes the sample fighter's art: you, an agent-generated placeholder, or a commissioned artist? Style/size (e.g. 64 px tall)?
5. ECB: start with fixed attribute ECB (simple) or go straight to per-frame joints?
6. Is a crude "thrown" sprite set acceptable for being thrown, or do you want per-thrower throw poses?
7. Should the LAB box editor be in scope for the first release, or JSON-only at first?
8. Format number: take `"geno": 13`, or fold into an existing number?
9. Any YouGame listing planned that would include the sample fighter (affects art licence choice)?
10. Should sprite fighters appear on the normal CSS beside 3D fighters immediately, or behind a mods toggle first?
