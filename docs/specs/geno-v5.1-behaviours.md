# Geno v5.1 behaviour procedures: clean-room behaviour spec

Status: stage 1 (spec writer), revision 1 (implementer, 2026-11-02: see section 9). Written from public, observable sources only; see "Provenance" below.
Procedures covered: `glide`, `tornado`, `drill`, `cape`, `multi_dash`, `anim_motion`.

## 0. Provenance and honesty rules

- Allowed inputs: SmashWiki (ssbwiki.com) pages on gliding and on the Brawl Meta Knight moves, and general game-physics knowledge.
- No disassembly, no repo geno sources, no game data of any version were consulted.
- Every number in the acceptance tables carries a source tag:
  - `[W:page]` = read from that public wiki page during this session (fetched 2026-11-02).
  - `[D]` = derived arithmetically from `[W]` numbers by the formula shown; the derivation is part of the entry.
  - `[E]` = estimated. The public sources give no number; the value is a design target chosen from observation-level reasoning and is NOT authoritative. A reviewer with video access should confirm or adjust `[E]` values before they are treated as targets. Tolerances for `[E]` rows are advisory.
- The wiki pages used are qualitative for glide, drill speed, cape, and multi-dash. Only the Mach Tornado and Drill Rush timing, the Shuttle Loop timing, and the character stat block carry real numbers.
- All formulas below are OUR OWN designs. They were fitted to the targets, not copied from anything.

Pages read: `Meta_Knight_(SSBB)`, `Glide`, `Mach_Tornado`, `Drill_Rush`, `Dimensional_Cape`, `Shuttle_Loop`, and the subpages `Meta_Knight_(SSBB)/Neutral_special`, `/Side_special`, `/Up_special`, `/Down_special` (all on ssbwiki.com). Note: the subpage text for Neutral special mentions "loop hits" and a "spin speed"; we treat that as the Mach Tornado spin (its numbers close exactly, see T-2). The `/Down_special` subpage had no frame data.

## 1. Numeric conventions (apply to every procedure)

1. All simulation values are `f32`. No `f64` in the simulation path (table generation excepted, section 1.3).
2. No fused multiply-add. Every `a*b + c` is two operations: compute `t = a*b` into a named `f32` temporary, then `t + c`. Build flags: Rust must not use `mul_add` or fast-math; C must use `-ffp-contract=off`.
3. Evaluation order is the order written in the pseudocode, left to right, one operation per line where a temporary is named. Do not reassociate, do not hoist constants into fused forms.
4. Constants written as decimal literals are the nearest `f32` to the decimal value.
5. Angles are `f32` degrees. Unit: world units per frame (`u/f`), 60 frames per second.
6. Facing `f` is `+1.0` or `-1.0` (`f32`).
7. `clamp(x, lo, hi)` = `if x < lo {lo} else if x > hi {hi} else {x}` (NaN never occurs; do not rely on it).
8. `approach(x, target, step)`: `if x < target { x2 = x + step; if x2 > target {target} else {x2} } else { x2 = x - step; if x2 < target {target} else {x2} }`.

### 1.1 Deterministic trig

`sin_deg(a)` and `cos_deg(a)` use a 1025-entry quarter-wave table `SINQ[i]`, `i = 0..1024`, the value `sin(i * 90/1024 degrees)`.

Table generation (offline, one time): evaluate in a correctly-rounded reference (MPFR or CORE-MATH) at the exact argument `i*pi/2048`, round to nearest `f32`. The implementer commits the table as 1025 hex-encoded `u32` words and a CRC32 over them in the test suite. The table is data, not computed at runtime.

Evaluation:
```
fn sin_deg(a: f32) -> f32:
    // wrap to [0,360)
    k = floor(a / 360.0)            // f32 divide then floor
    w = a - k * 360.0               // two ops: t = k*360.0; w = a - t
    // fold into quadrant
    if w < 90.0   { q = w;          sgn = 1.0 }
    else if w < 180.0 { q = 180.0 - w;  sgn = 1.0 }
    else if w < 270.0 { q = w - 180.0;  sgn = -1.0 }
    else          { q = 360.0 - w;  sgn = -1.0 }
    p  = q * KQ                     // KQ = nearest f32 of 1024/90
    i  = floor(p) as i32            // clamp to 0..=1023
    fr = p - (i as f32)
    lo = SINQ[i]; hi = SINQ[i+1]
    d  = hi - lo
    t  = d * fr
    v  = lo + t
    return sgn * v
fn cos_deg(a) = sin_deg(a + 90.0)
```
`sin_deg` must return exactly `+0.0`-signed zero behaviour as the formula gives; no special-casing.

### 1.2 Host contract

Each procedure is a pure function of `(State, Input, Params, Host)` that runs once per sim frame and returns the new `State` and a velocity/position request. The host owns collision, hit detection, and animation playback. Host calls used: `ground_contact(pos, vel) -> Option<Floor>`, `ledge_grab_allowed`, `spawn_hit(id)`, `reflect_window(bool)`, `set_intangible(bool)`, `set_helpless(bool)`, `facing()`, `set_facing(f)`.

### 1.3 Input conventions

- `stick` = `(sx, sy)` in `[-1,1]`, with a radial dead zone: if `sx*sx + sy*sy < DZ2` then `(0,0)`. `DZ2 = 0.0826` (0.2875 squared) `[E]`.
- Buttons are edge-flagged by the host: `x_down`, `x_pressed` (true only the first frame), `x_held`.
- `special_pressed`, `jump_pressed`, `jump_held`, `attack_pressed`.

---

## 2. glide

### 2.1 States
`GL_START` (wing deploy, frames 0 to `start_frames - 1`), `GL_FLY`, `GL_ATTACK` (glide attack), `GL_JUMPOUT` (short exit), `GL_LAND`, `GL_TRIP` (crash), `GL_END` (stall, return to fall).

State record: `theta` (pitch, degrees, positive = nose up), `speed` (scalar, `u/f`, always >= 0), `f`, `t` (frame in state), `glides_used`, `attack_buf` (frames).

### 2.2 Entry
Preconditions: airborne, `glides_used < max_glides`, not already helpless.
Three entry paths (any one starts `GL_START`):
- A. Midair-jump hold: a midair jump was consumed, and `jump_held` has been true for `entry_hold` consecutive frames since that jump, and `vel.y <= entry_vy_max`.
- B. Apex flick: `|vel.y| <= apex_vy` and within the last `flick_window` frames the stick went from `sx*f <= -flick_thr` to `sx*f >= flick_thr`, or the reverse (left-then-right either way).
- C. Chained from an up-special-style procedure (host flag `chain_slow`): enter with `slow = true` (speed cap and gravity gain `GK` multiplied by `chain_speed_mul`, turn rate multiplied by `chain_turn_mul`).

On entry: `speed = clamp(length(vel), 0, s_start_cap)` computed as `speed = sqrt(vx*vx + vy*vy)` (f32 `sqrt`, correctly rounded, which is IEEE exact); `theta = theta_def`; `f` unchanged unless `sx*f < -flick_thr` at entry, then `f = -f`; `glides_used += 1`.

### 2.3 Per-frame update (`GL_FLY`)
```
// 1. pitch command: d > 0 means nose down
t0 = sx * f
t1 = 0.0 - sy
d  = clamp(t0 + t1, -1.0, 1.0)
// 2. target angle
if d > 0.0 { r = d * R_DOWN } else { r = d * R_UP }      // r <= 0 when d<0 (nose-up)
theta_t = theta_def - r                                   // note: d<0 gives r<0 so theta_t > theta_def
// 3. turn toward target
rate = TURN_RATE            // times chain_turn_mul when slow
theta = approach(theta, theta_t, rate)
// 4. speed: gravity component along heading, then drag
s_th  = sin_deg(theta)
gk    = GK                  // times chain_speed_mul when slow (rev 1)
g     = gk * s_th           // positive when nose up
speed = speed - g           // nose down (s_th < 0) increases speed
dr    = speed * DRAG
speed = speed - dr
cap   = S_MAX               // times chain_speed_mul when slow
if speed > cap { speed = cap }
// 5. velocity
c_th = cos_deg(theta)
vx   = speed * c_th
vx   = vx * f
vy   = speed * s_th
```
`GL_START` (frames 0 to `START_FRAMES-1`): same pitch/turn code is NOT run; `theta = theta_def`; `speed = approach(speed, S_START, START_ACCEL)`; velocity from the same step-5 formulas. At `t == START_FRAMES` switch to `GL_FLY`.

### 2.4 Exits (checked after the update, in this priority order)
1. Ground contact (host reports floor below along `vel`):
   - if `attack_buf > 0` or the state is `GL_ATTACK`: land with zero landing lag (`lag_cancel`).
   - else if `theta < -TRIP_ANGLE` (nose down steeper than the trip angle): `GL_TRIP` (knockdown, `TRIP_FRAMES` frames).
   - else `GL_LAND` with `LAND_LAG` frames.
2. `jump_pressed` and the player has a jump left: `GL_JUMPOUT` (`JUMPOUT_FRAMES`), then free fall with the jump applied.
3. `attack_pressed` (or `special_pressed`): `GL_ATTACK` (`ATTACK_FRAMES` total). Start-up hit frames `ATTACK_HIT0..ATTACK_HIT1`. Velocity continues from the glide (speed multiplied by `ATTACK_SPEED_KEEP` once on entry), then decays with `drag * 4`. After the attack, `GL_END` with `ATTACK_ENDLAG` frames of fall.
4. Stall: `speed < S_STALL` -> `GL_END` (falls, not helpless if jumps remain, helpless if none remain).
5. Landing-cancel buffer: if an attack/special press happens while `height_above_floor <= vy_mag * ATTACK_BUF_FRAMES` (projected contact within `ATTACK_BUF_FRAMES`), set `attack_buf = ATTACK_BUF_FRAMES`; it decrements each frame.

Duration: unbounded by timer. A glide ends only by exits 1-4.
Helpless: if a glide begins after the last midair jump, `set_helpless(true)` when the glide ends by stall or by attack end.

### 2.5 Default parameters (Meta Knight-like)
| Param | Value | Note |
|---|---|---|
| `max_glides` | 2 | `[W:Glide]` "can glide twice" |
| `entry_hold` | 6 | `[E]` frames |
| `entry_vy_max` | 0.6 | `[E]` |
| `apex_vy` | 0.25 | `[E]` |
| `flick_window` | 12 | `[E]` frames |
| `flick_thr` | 0.55 | `[E]` |
| `START_FRAMES` | 12 | `[E]` (wiki: "fastest startup" of the three, no number) |
| `S_START` | 1.50 | `[E]` (rev 1, was 1.20: with `DRAG = 0.002` a 1.20 start sits 7.5% under G-2's target at 600 frames) |
| `START_ACCEL` | 0.12 | `[E]` |
| `s_start_cap` | 2.0 | `[E]` |
| `theta_def` | -5.0 | `[E]` slight downward default |
| `R_DOWN` | 60.0 | `[E]` |
| `R_UP` | 85.0 | `[E]`; wiki: MK "can go nearly straight up" |
| `TURN_RATE` | 3.0 | `[E]` deg/frame |
| `GK` | 0.0366 | designed so steady speed at `theta_def` is `GK*sin(5 deg)/DRAG` (see G-2) |
| `DRAG` | 0.002 | designed |
| `S_MAX` | 2.40 | `[E]` |
| `S_STALL` | 0.35 | `[E]` |
| `TRIP_ANGLE` | 30.0 | `[E]` |
| `TRIP_FRAMES` | 30 | `[E]` |
| `LAND_LAG` | 14 | `[E]` |
| `JUMPOUT_FRAMES` | 8 | `[E]` ("jumping out is faster than attacking out" `[W:Glide]`) |
| `ATTACK_FRAMES` | 28 | `[E]` |
| `ATTACK_HIT0/1` | 8 / 14 | `[E]` |
| `ATTACK_ENDLAG` | 14 | `[E]` |
| `ATTACK_SPEED_KEEP` | 0.6 | `[E]` |
| `ATTACK_BUF_FRAMES` | 6 | `[E]` |
| `chain_speed_mul` | 0.75 | `[E]` ("slower ... better control" `[W:Shuttle_Loop]`) |
| `chain_turn_mul` | 1.4 | `[E]` |
| attack damage | 12 | `[W:Glide]` (host data) |

### 2.6 Acceptance table (glide)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| G-1 | Number of glides per airtime | 2 | `[W:Meta_Knight_(SSBB)]` | exact |
| G-2 | Steady speed holding neutral stick, long run | `GK*sin_deg(5)/DRAG` = 1.595 u/f, reached monotonically from `S_START`; sits just under MK run speed 1.847 | `[D]` from params; run speed `[W:MK(SSBB)]`; absolute glide speed `[E]` | within 3% of 1.595 after 600 frames |
| G-3 | Stick down/forward gains speed; stick up/back loses speed | sign of `d(speed)/dt` matches | `[W:Glide]` | exact sign, every frame after settle |
| G-4 | Stick fully up from steady state ends glide by stall | stall frame count N (measure; fixture value to be recorded from the reference implementation of this spec) | `[E]` | exact vs fixture |
| G-5 | Max nose-up angle reached | `theta_def + R_UP` = 80 deg | `[D]`; "nearly straight up" `[W:Glide]` | exact steady value after settle; transit time = `ceil(85/3)` = 29 frames |
| G-6 | Max nose-down angle | -65 deg | `[D]` | exact |
| G-7 | Turn time default to full-up | 29 frames | `[D]` | exact |
| G-8 | Glide attack damage | 12 | `[W:Glide]` | exact |
| G-9 | Attack pressed within buffer before floor contact: no landing lag | lag 0 | `[W:Glide]`, `[W:Shuttle_Loop]` (window length `[E]`: 6) | exact |
| G-10 | Hitting floor while steeper than 30 deg down: trip | `GL_TRIP` | `[W:Glide]` ("glide tripping"; angle `[E]`) | exact |
| G-11 | Gliding after the last jump leaves helpless afterwards | helpless on end | `[W:Glide]` | exact |
| G-12 | Chained (slow) glide steady speed | G-2 value times 0.75 | `[W:Shuttle_Loop]` direction; factor `[E]` | within 3% |
| G-13 | Entry paths A and B and C each start `GL_START` | yes | `[W:Glide]` | exact |
| G-14 | Start phase length | 12 frames | `[E]` | exact (parameter) |

---

## 3. tornado (spinning rise with mash-to-extend)

### 3.1 States and timeline
Timeline (global frame `n` from move start, grounded or aerial):
- `0..11`: start-up (`TN_START`), no hits. Hit loop begins at frame 12. `[W:MK_SSBB/Neutral_special]`: loop hits frames 12-58.
- `TN_LOOP`: loop frame `k = n - 12`, `k = 0, 1, ...`
- `TN_END`: ending animation, 29 frames. `[W:Neutral_special]`
- Final hit: in the ending animation, frames 1-3 grounded, 1-2 aerial. `[W:Neutral_special]`
- After `TN_END` airborne: helpless. `[W:Mach_Tornado]`
- Ledge grab disabled throughout `TN_LOOP` and the first `LEDGE_LOCK` frames of `TN_END`; edge contact bounces horizontally instead. `[W:Mach_Tornado]` (lock length `[E]`: 6).

### 3.2 Spin model (matches the published numbers)
State: `spin` (deg/frame, f32), `rot` (deg, accumulates to hit trigger), `press_latch` (bool), `vy`, `vx`.
Constants: `SPIN0 = 80.0`, `DECAY_A = 1.5` (loop frames `k < 70`), `DECAY_B = 2.0` (`k >= 70`), `MASH_ADD = 16.0`, `SPIN_MAX = 80.0`, `SPIN_END = 10.0`, `READ_PERIOD = 10`, `READ_LIMIT = 70`.
At `k = 0`: `spin = SPIN0`, `rot = 0.0`.

Per loop frame:
```
if special_pressed { press_latch = true }
// loop-hit generation (before spin update)
if spin <= SPIN_END { goto TN_END }        // exit test happens at the top
if k == 0 { spawn_hit(LOOP_HIT) }          // rev 1: the loop window opens with a hit (T-1, global frame 12)
rot = rot + spin
if rot >= 360.0 { rot = rot - 360.0; spawn_hit(LOOP_HIT) }
// mash read, only on read frames and only inside the read window
if k < READ_LIMIT && (k % READ_PERIOD) == 0 && press_latch {
    spin = spin + MASH_ADD; if spin > SPIN_MAX { spin = SPIN_MAX }
    rise_impulse = true
    press_latch = false
}
// decay
if k < READ_LIMIT { spin = spin - DECAY_A } else { spin = spin - DECAY_B }
// presses outside read frames stay latched until the next read frame (k<70), then are dropped
```
Check with no mashing: `spin(k) = 80 - 1.5k`, `spin <= 10` first at `k = 47` (9.5), so the loop runs `k = 0..46`, 47 frames, i.e. global frames 12..58. This reproduces `[W]` exactly (T-2).
Hit spacing: `360/spin` frames: at 80 deg/f it is 4.5 (the first two gaps are 4 then 5; the decay lengthens later gaps); at 30 deg/f it is 12 between accumulator hits. Unmashed loop hits: the loop-start hit plus `floor(2138.5/360) = 5` accumulator hits = 6 (T-16). Matches "as low as four frames and as high as 12+" `[W]`.
Final hit: spawned by the host on ending-animation frame 1 (frames 1-3 grounded, 1-2 aerial).

### 3.3 Movement (our design)
Aerial:
```
// gravity (reduced while spinning)
vy = vy - GRAV_TN                    // GRAV_TN = 0.0239  (0.25 * 0.0956)
// rise on accepted mash
if rise_impulse { vy = vy + RISE_IMPULSE; if vy > RISE_MAX { vy = RISE_MAX }; rise_impulse = false }
// terminal fall
if vy < -FALL_TN { vy = -FALL_TN }   // FALL_TN = 0.50
// horizontal control
tx = sx * VX_MAX
vx = approach(vx, tx, VX_ACCEL)
```
Start-up bonus: if `jump_held` on frame 0, at frame 12 `vy = vy + JUMP_BONUS`. At frame 0, `vy` is set to `vy * START_VY_KEEP`, `vx` to `vx * START_VX_KEEP`.
Grounded: `vx = approach(vx, sx*VX_MAX_GROUND, VX_ACCEL)`, no gravity effect, no rise (mash only extends duration).
Defaults: `RISE_IMPULSE = 0.9 [E]`, `RISE_MAX = 1.6 [E]`, `VX_MAX = 1.1 [E]`, `VX_MAX_GROUND = 1.3 [E]`, `VX_ACCEL = 0.06 [E]`, `JUMP_BONUS = 1.0 [E]`, `START_VY_KEEP = 0.2 [E]`, `START_VX_KEEP = 0.5 [E]`, `LEDGE_LOCK = 6 [E]`.

### 3.4 Mash extension, literal rule
With a press landing on every read frame (k = 0, 10, ..., 60): read frames at `k=0` has `spin` already at 80 (cap) so no change; spin at `k=10` is 65 + 16 -> 80 (capped); the last read is `k=60`. From `k=60` the spin is 80 at that read, decays 1.5 for 10 frames to 65 at `k=70`, then 2.0/frame: `65 - 2*j <= 10` at `j = 28`, exit test at `k = 98`. Total loop frames 98, i.e. 51 more than the 47 unmashed. `[D]`

The wiki states "can be increased up to an extra 70 frames". The literal rule above gives 51, so the two are not consistent under our reading of the rule. Resolution: keep literal constants (they are the sourced ones), record the 51 as the model value, and treat "up to 70" as an upper bound that is satisfied (T-5). If a reviewer measures that real mashing yields 70, change `DECAY_B` to `1.1702...` (solved in T-5 notes) and re-record; the code takes both as parameters.

### 3.5 Acceptance table (tornado)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| T-1 | Loop hit window start | global frame 12 | `[W:MK_SSBB/Neutral_special]` | exact |
| T-2 | Unmashed loop end | frame 58 (loop frames 0..46, 47 frames) | `[W]`, `[D]` (80-1.5k <= 10 at k=47) | exact |
| T-3 | Ending animation length | 29 frames | `[W:Neutral_special]` | exact |
| T-4 | Final hit frames | 1-3 grounded, 1-2 aerial | `[W:Neutral_special]` | exact |
| T-5 | Mash extension, maximum | <= 70 extra frames; literal-rule model yields 51 | `[W]` upper bound; 51 `[D]` | exact 51 for the model; <= 70 |
| T-6 | Hit spacing at start | alternating 4 and 5 frames | `[W]` "as low as four", `[D]` 360/80=4.5 | exact |
| T-7 | Hit spacing near end | >= 12 frames when spin <= 30 | `[W]` "12+" | exact |
| T-8 | Mash read cadence | presses read only at k%10==0, k<70 | `[W]` | exact |
| T-9 | Spin cap | never above 80 | `[W]` | exact |
| T-10 | Loop damage | 1% per loop hit, 3% final | `[W:Mach_Tornado, SSBB page]` (host data) | exact |
| T-11 | Helpless after aerial end | yes | `[W:Mach_Tornado]` | exact |
| T-12 | Ledge grab locked during loop | yes | `[W:Mach_Tornado]` | exact |
| T-13 | Jump bonus raises peak height | peak(jump) > peak(no jump) | `[W:Mach_Tornado]`; magnitude `[E]` | sign exact |
| T-14 | Horizontal control by stick | `vx` approaches `sx*1.1` | `[W:Mach_Tornado]`; speed `[E]` | within 3% of 1.1 at steady |
| T-15 | Net rise with full mashing | positive net vertical displacement at k=98 | `[W]` qualitative; magnitude `[E]` (record the model value) | sign exact |
| T-16 | Number of loop hits unmashed | 6 (5.9 by `sum(spin)/360`, accumulator) | `[D]` | exact vs accumulator |

---

## 4. drill (diagonal dive rush)

### 4.1 Timeline
Global frame `n`.
- `0..25`: wind-up (`DR_WIND`). `[W:Side_special]`: multihits begin at 26.
- Movement starts at frame `MOVE0 = 16 [E]`; velocity ramps for `RAMP = 4` frames.
- `26..65`: rushing hits. 10 hits, one every 4 frames: frames 26, 30, ..., 62. `[W]` 1% each, 10 hits; multihit frames 26-65 `[W]`. The 4-frame cadence is `[D]` (40 frames / 10 hits; last hit at 62, hit window ends at 65).
- Final hit: frames 66-67 grounded, 66-70 aerial `[W]` (the page labels both rows grounded; we read the longer one as aerial).
- Animation length 96 (grounded). `[W]`
- Afterwards airborne: helpless (`[W:Drill_Rush]`).
- Recoil: on frame 66, if the move is aerial, `vy = RECOIL_VY` (Brawl always recoils, SSB4 only on hit, `[W:Drill_Rush]`).

### 4.2 Heading model
State: `psi` (heading, degrees; 0 = horizontal facing forward, negative = down), `speed`.
At frame 0: `psi = psi0` where `psi0 = -clamp(-sy, -1.0, 1.0)`-based: 
```
a   = sy * PSI_STICK          // PSI_STICK = 45.0
psi = DIVE_DEFAULT + a        // DIVE_DEFAULT = -20.0
psi = clamp(psi, PSI_MIN, PSI_MAX)
```
Defaults: `PSI_MIN = -60.0`, `PSI_MAX = 30.0`. Mid-move steering ("you can steer with the Control Stick" `[W:Drill_Rush]`):
```
tgt = clamp(DIVE_DEFAULT + sy * PSI_STICK, PSI_MIN, PSI_MAX)
psi = approach(psi, tgt, STEER_RATE)       // STEER_RATE = 2.0 deg/frame  [E]
```
Speed:
```
if n < MOVE0 { speed = 0 }
else { speed = approach(speed, DR_SPEED, DR_SPEED / RAMP) }     // DR_SPEED = 2.0 u/f [E]
vx = speed * cos_deg(psi); vx = vx * f
vy = speed * sin_deg(psi)
```
No gravity during frames `MOVE0..65`. On frame 66, velocity is replaced with the recoil/end values: `vx = vx * END_VX_KEEP` (0.2), `vy = RECOIL_VY` (aerial, 1.0) and gravity returns. Grounded drill follows the floor: `vy` forced to 0 and heading clamped to `psi >= 0` when the floor is flat (host supplies slope; `psi = max(psi, slope_angle)`).

### 4.3 Acceptance table (drill)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| D-1 | First hit frame | 26 | `[W:MK_SSBB/Side_special]` | exact |
| D-2 | Rushing hit count | 10 | `[W]` | exact |
| D-3 | Hit cadence | every 4 frames, frames 26..62 | `[D]` | exact |
| D-4 | Rushing hit window end | frame 65 | `[W]` | exact |
| D-5 | Final hit | 66-67 grounded; 66-70 aerial | `[W]` | exact |
| D-6 | Total animation length | 96 | `[W]` | exact |
| D-7 | Rushing damage | 1% x 10, final 3% (grounded hit1) | `[W]` | exact |
| D-8 | Helpless after aerial end | yes | `[W:Drill_Rush]` | exact |
| D-9 | Travel distance | speed 2.0 u/f over frames 20..65 (46 frames) => ~ 90 u after ramp (computed) | speed/distance `[E]`; wiki says "decent distance" | the model value, within 3% of the fixture |
| D-10 | Stick-down steers heading to -60 deg (clamped) | heading reaches -60 | `[W]` steer; limits `[E]` | exact after `ceil(|dpsi|/2)` frames |
| D-11 | Recoil on frame 66 (aerial) | `vy = 1.0` | `[W:Drill_Rush]` (exists in Brawl); value `[E]` | exact (param) |
| D-12 | Dive heading, no stick | -20 deg | `[E]` | exact |

---

## 5. cape (reflect, turnaround, momentum stall)

This is a design built from the task brief (reflect/turnaround with momentum stall). Public numbers for any specific game's version of this move were not read in this session (the Dimensional Cape page has none, and in Brawl that move is a teleport that does not reflect `[W:Dimensional_Cape]`). All timing below is therefore `[E]`.

### 5.1 Timeline
- `0..CAPE_START-1` start-up (`CAPE_START = 6`).
- `CAPE_START..CAPE_END` active (`CAPE_END = 25`): reflect window open, `reflect_window(true)`. 20 frames.
- Turnaround: on frame `CAPE_START`, if `sx * f < -TURN_THR` (stick held backward), `set_facing(-f)` once. The facing flip is decided exactly once.
- `CAPE_END + 1 .. CAPE_TOTAL-1`: endlag, `CAPE_TOTAL = 40`.
- Air use: allowed, not helpless afterward.
- Reflect: host reflects projectiles whose hitbox overlaps the cape zone; the returned projectile has damage multiplied `REFLECT_DMG_MUL = 1.5` and speed multiplied `REFLECT_SPEED_MUL = 1.2`.

### 5.2 Momentum stall
Applied each frame during active (airborne only):
```
vx = vx * STALL_X            // 0.80
if vy > STALL_VY_HI { vy = vy * STALL_Y_UP }     // vy>0: scale 0.85
else { vy = vy - STALL_GRAV }                    // STALL_GRAV = 0.015 (slow fall)
if vy < -STALL_FALL { vy = -STALL_FALL }         // STALL_FALL = 0.45
```
With `STALL_VY_HI = 0.0`. On the first endlag frame the stall ends and normal gravity returns; velocity is NOT reset. Grounded: `vx = vx * 0.5` (friction-like), no vertical change.
Defaults: `TURN_THR = 0.5`, `STALL_X = 0.80`, `STALL_Y_UP = 0.85`.

### 5.3 Acceptance table (cape)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| C-1 | Start-up | 6 frames | `[E]` | exact |
| C-2 | Reflect window | frames 6..25 (20 frames) | `[E]` | exact |
| C-3 | Total length | 40 frames | `[E]` | exact |
| C-4 | Turnaround flip | exactly one flip, only when stick backward at frame 6 | `[E]`, design | exact |
| C-5 | Horizontal speed after k active frames | `vx0 * 0.8^k` (f32 sequential product) | `[D]` from the formula | bit-exact |
| C-6 | Terminal fall during stall | 0.45 u/f | `[E]` | within 3% |
| C-7 | Reflected projectile | damage x1.5, speed x1.2 | `[E]` | exact (params) |
| C-8 | Not helpless | yes | `[E]` | exact |
| C-9 | Brawl Dimensional Cape reflects nothing | informative only, not implemented here | `[W:Dimensional_Cape]` | n/a |

---

## 6. multi_dash (directed multi-segment dash)

Sonic-Blade-like: a chain of straight dashes, each aimed with the stick, with a re-press window between segments. No public frame data was read; everything is `[E]` design.

### 6.1 Structure
State: `seg` (segment index, 0-based), `phase` in {`AIM`, `DASH`, `PAUSE`, `END`}, `t`, `dir` (unit vector), `speed`.
Parameters: `MAX_SEG = 4`, `AIM_FRAMES = 6` (first segment) / `4` (later), `DASH_FRAMES = 10`, `PAUSE_FRAMES = 3`, `REPRESS_FRAMES = 8` (window opens at the last `REPRESS_FRAMES` of the dash and runs into the pause), `END_LAG = 20`, `DASH_SPEED = 3.2 [E]`, `END_KEEP = 0.15`, `AIM_SLOW = 0.1` (velocity multiplier during aim).

### 6.2 Aim and dash
At the end of `AIM` the direction is fixed (it never changes during a dash):
```
m2 = sx*sx; n2 = sy*sy; mm = m2 + n2          // two temporaries, then add
if mm < DZ2 { ax = f; ay = 0.0 }              // no stick: facing direction
else { len = sqrt(mm); ax = sx / len; ay = sy / len }
// facing: if ax*f < 0 then flip facing
dir = (ax, ay)
```
Vertical component is limited (rev 1; renormalising `(ax, clamp(ay))` undid the clamp when `ax = 0`): if `|ay| > AY_MAX` with `AY_MAX = 0.9397` (sin 70 deg) `[E]`: `ay = clamp(ay, -AY_MAX, AY_MAX); l3 = ay*ay; ll = 1.0 - l3; l = sqrt(ll); ax = l * sgn` where `sgn` is the sign of `ax`, or the (possibly flipped) facing `f` when `ax == 0`.
During `DASH`:
```
speed = approach(speed, DASH_SPEED, DASH_SPEED / 3.0)     // two ramp frames, full speed from frame 3 (rev 1, was / 2.0)
vx = speed * ax;  vy = speed * ay
```
No gravity. Distance per segment at full speed: frames 3..10 at 3.2 plus ramp = `DASH_SPEED*(0.5 + 1.0) ... ` computed exactly by the loop; target value is the fixture produced by running the formula (the sum is order-sensitive in f32).
`PAUSE`: velocity multiplied by `0.3` each frame. If `special_pressed` was seen in the re-press window and `seg + 1 < MAX_SEG`: `seg += 1`, phase `AIM`. Else go to `END`, `END_LAG` frames, velocity `*= END_KEEP` once, then gravity returns; airborne afterwards: helpless if `seg + 1 == MAX_SEG`, else not (design).
Hit: each `DASH` frame spawns `DASH_HIT(seg)` (host).

### 6.3 Acceptance table (multi-dash)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| M-1 | Segments max | 4 | `[E]` | exact |
| M-2 | Segment timeline | aim 6 (later 4), dash 10, pause 3 | `[E]` | exact |
| M-3 | Aim direction fixed during dash | yes | `[E]` | exact |
| M-4 | Steady dash speed | 3.2 u/f | `[E]` | within 3% |
| M-5 | Distance of one straight segment | fixture from the formula (about 29 u; rev 1 reference run 28.8 = 1.0667 + 2.1333 + 8 x 3.2) | `[D]` of `[E]` params | bit-exact vs fixture, within 3% of 29 |
| M-6 | Aim clamp | |vertical| <= sin 70 deg | `[E]` | within 3% |
| M-7 | Re-press window | last 8 dash frames + pause | `[E]` | exact |
| M-8 | No re-press ends the move after the pause + 20 end-lag | yes | `[E]` | exact |
| M-9 | Unit direction | `ax^2+ay^2 = 1` | `[D]` | within 1e-6 |

---

## 7. anim_motion (root motion driven by animation)

### 7.1 Data
A motion track is an array `P[0..N-1]` of `(px, py)` `f32` root offsets in animation-local space (forward-positive x, up-positive y), one sample per animation frame at rate 1.0, plus parameters:
`rate` (playback, `f32`, default 1.0), `scale` (`f32`, default 1.0), `mode` in {`DELTA`, `ABSOLUTE`}, `gravity_on` (bool), `y_mode` in {`TRACK`, `IGNORE`}, `loop` (bool).

### 7.2 Sampling (exact)
```
fn sample(P, t: f32) -> (f32,f32):
    if t <= 0.0 { return P[0] }
    last = (N - 1) as f32
    if t >= last { return P[N-1] }
    i  = floor(t) as i32
    fr = t - (i as f32)
    ax = P[i].x; bx = P[i+1].x
    dx = bx - ax;  tx = dx * fr;  x = ax + tx
    ay = P[i].y; by = P[i+1].y
    dy = by - ay;  ty = dy * fr;  y = ay + ty
    return (x, y)
```

### 7.3 Per-frame step
```
t0 = anim_t                      // before advance
t1 = anim_t + rate               // advance
(x0,y0) = sample(P, t0)
(x1,y1) = sample(P, t1)
dx = x1 - x0;  dy = y1 - y0
dx = dx * scale;  dy = dy * scale
dx = dx * f                       // facing
if y_mode == IGNORE { dy = 0.0 }
// DELTA mode: displacement this frame
pos_request = (pos.x + dx, pos.y + dy)     // host resolves collision
// velocity reported to the host (for knockback/handoff when the animation ends)
vx = dx;  vy = dy
if gravity_on { vy = vy + (vy_carry); vy_carry = vy_carry - GRAV }   // gravity integrates a separate carry term
anim_t = t1
if anim_t >= last: if loop { anim_t = anim_t - last } else { end -> handoff velocity (vx,vy) to the host }
```
`ABSOLUTE` mode: `pos = origin + (x1*scale*f, y1*scale)` where `origin` is `pos` at frame 0 and `x1`, `y1` from `sample`; computed per frame (products first, then add).
Blocked move: if the host reports a wall hit, the remaining `dx` is dropped and `anim_t` is NOT rewound (animation continues). In `ABSOLUTE` mode the track is re-based: `origin.x` is shifted by the blocked amount.

### 7.4 Acceptance table (anim_motion)
| ID | Behaviour | Target | Source | Tolerance |
|---|---|---|---|---|
| A-1 | Frames to play a track of length N at rate 1 | `N - 1` | `[D]` | exact |
| A-2 | Total displacement in DELTA mode | `(P[N-1] - P[0]) * scale * f` | `[D]` | within 3% (expected 1e-6 relative; f32 sum drift) |
| A-3 | Rate 0.5 doubles the frame count | `2*(N-1)` | `[D]` | exact |
| A-4 | Rate 2.0 halves the frame count (ceil) | `ceil((N-1)/2)` | `[D]` | exact |
| A-5 | DELTA total equals ABSOLUTE final offset | equal | `[D]` | within 3% |
| A-6 | Linear sample midpoint | `sample(P, i+0.5) = P[i] + 0.5*(P[i+1]-P[i])` | `[D]` | within 1 ulp; exact since `0.5` is exact |
| A-7 | Facing flip mirrors x only | yes | `[D]` | exact |
| A-8 | Blocked move drops `dx` and does not rewind | yes | `[E]` design | exact |
| A-9 | Looping wraps by subtraction not modulo | `t - last` | `[D]` | exact |

---

## 8. Cross-cutting implementer notes

1. Each procedure exposes `fn step(state: &mut S, input: &I, p: &Params, host: &mut dyn Host)`. State must be `Copy`-able plain data (rollback snapshot).
2. A shared test module reads the acceptance tables as data (ID -> expected). "Fixture" rows (G-4, T-15, D-9, M-5) are generated by the implementer's own reference run of this spec and checked into the test data, with the formula, not by hand-picking.
3. Order of work per sim frame: input snapshot -> procedure step -> host collision -> hit detection. The procedure never reads collision results from the same frame it requests movement.
4. All "frame counts" are counted from the first frame in which the state is active as frame 0, except wiki hit frames, which are 1-based in the wiki and 0-based here only where a row says "global frame n". Where a wiki row says "frames a-b" for a hit window, the implementation uses the inclusive range `a..=b` on the wiki's numbering; the implementer reconciles the base once at the animation layer and documents it.
5. Open questions for a reviewer: confirm the glide `[E]` rows from public video (speed, angle limits, start-up); confirm the tornado mash extension (51 vs "up to 70"); confirm cape timing from a public frame-data table.

## 9. Revision 1 (2026-11-02, implementer)

The first implementation followed sections 2-7 literally, and six rows could not be reached by the spec's own formulas: G-2, G-12, T-6, T-16, M-5 and M-6. Revision 1 changes only `[E]` / designed values and our own formulas. It changes no `[W]` number and no acceptance target. Each change is our own design reasoning; no new source was consulted.

| Row | Problem in stage 1 | Revision 1 |
|---|---|---|
| G-2 | `DRAG = 0.002` has a 500-frame time constant, so a start at `S_START = 1.20` is still 7.5% under 1.595 at frame 600 | `S_START = 1.50` `[E]` (1.564 at frame 600, 1.9% under; still monotone) |
| G-12 | the slow glide only lowered the cap (1.8), which is above the neutral steady speed, so the steady speed did not change | the slow glide also multiplies `GK` by `chain_speed_mul`, so its steady speed is `0.75 x` (measured 1.194) |
| T-16 | the accumulator alone gives `floor(5.94) = 5` loop hits | the loop window opens with a hit on loop frame 0 (global frame 12, consistent with T-1): 6 hits |
| T-6 | with the 1.5 deg/frame decay and no hit at k = 0, the gaps ran 5, 6, 7 | with the loop-start hit, the first two gaps are 4 then 5; the decay makes the third gap 6, so "alternating 4 and 5" means the first two gaps |
| M-5 | the `/ 2.0` ramp reaches full speed on frame 2: 30.4 u (4.8% over 29) | `/ 3.0`: two ramp frames, full speed on frames 3..10 as the text says: 28.8 u |
| M-6 | renormalising after the clamp undid it (stick straight up gave `ay = 1.0`) | clamp the unit vector's `ay` and rebuild `ax` from it: `|ay| <= 0.9397` |

T-7 is measured between accumulator hits. The loop-start hit is not "near the end", and frame 0's spin already counts toward the next hit. Re-recorded fixture: G-4 = 43 frames (it was 39 with `S_START = 1.20`).
