# Shard 961–968: quaternion and RNG coverage

`tu/quatlib.rs` re-exports all six existing native quaternion implementations under
the C names. `tu/random.rs` delegates the six existing RNG operations to `HsdRand`
and exposes the private current-seed selector; it also exposes random.h's `randi`.
No C structs were duplicated. None of these entries is only smoke-tested.

| C definition | Native C oracle |
|---|---|
| MatToQuat | 1,500 matrices; quaternion bits and successful C status |
| HSD_QuatLib_8037EB28 | 1,500 matrices; Euler bits and successful C status |
| HSD_QuatLib_8037EC4C | 1,500 quaternion pairs; product bits and successful C status |
| HSD_QuatLib_8037ECE0 | 1,500 axes/angles including zero axes; status, quaternion bits, untouched output on failure |
| EulerToQuat | 1,500 Euler triples; quaternion bits and successful C status |
| HSD_QuatLib_8037EF28 | 1,500 quaternion pairs/parameters; slerp bits and successful C status |
| HSD_RandRenderBegin | 1,000 complete state/trace rows |
| HSD_RandRenderEnd | 1,000 complete state/trace rows |
| HSD_RandCurrentSeed | 1,000 returned pointer identities and complete state/trace rows |
| HSD_Rand | 1,000 return/state/trace rows |
| HSD_Randf | 1,000 return bit patterns/state/trace rows |
| HSD_Randi | 1,000 return/state/trace rows |
| _HSD_RandForgetMemory | 1,000 state/trace rows with all half-open range boundaries |
| randi (header inline, additional) | 1,000 return/state/trace rows including empty ranges |

Regeneration uses `oracle/build_hsd.sh` and `oracle/build_random_tu.sh`: native C,
shipping contraction, zero initialization and `lower.py`'s gwtool lowering.
Quatlib has 38 f32 fused sites; random has none. The existing HSD fixture is
unchanged; its driver now also rejects unexpected status returns from the five
quaternion functions whose native API returns output values with implicit success.

The RNG driver includes actual random.c so it can set and observe all file statics.
It covers default/redirected seeds, nested render streams, depth zero/one/two and
unsigned wraparound, arbitrary signed Randi bounds including INT_MIN/INT_MAX,
empty ranges and forget-memory boundaries. The Rust test compares every state
field, return value, pointer identity, trace count, traced seed and stream after
each call. Caller addresses are host-specific and intentionally excluded.

Validation: `cargo test -p melee_runtime --test oracle_hsd --test oracle_random_tu`
passes with the three required cargo environment variables. Reversing the
current-seed selector's render-depth condition makes oracle row zero fail;
restoring it passes. RNG's existing optional trace callback remains the host
interface; no new unresolved callee is introduced. Native object ownership and
return-value APIs are reused; no shipping game/replay run is claimed.
