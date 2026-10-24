# Shard 921–928 implementation plan

Goal: finish the eight live baselib TUs in `melee_runtime`, retaining existing native ports and proving their behavior against native C.

Spec: the shard assignment, `RUST_PORT.md`, inventory section 4, and the group 1 notes.

Constraints: work only on `port/shard-921-928`; no disc data, pushes, merges, or other worktrees. Use the shared type generator and shipping float contraction. Every cargo invocation disables debug info and incremental compilation. Run the crate tests and workspace build, not workspace tests.

- [ ] DObj: retain descriptor renderer, add native generated-record lifecycle, animation, class and PC draw paths. Compare all fields, returns, class methods, and call order against C.
- [ ] FObj/Fog: expose existing logic and finish lifecycle/class entry points; extend their C oracles.
- [ ] Generator: expose all existing ports through the TU module and close oracle coverage gaps.
- [ ] GObj/GX link/init/object: expose existing arena ports, finish callbacks and PC hooks, extend seeded C state oracle.
- [ ] Register each TU exactly once, review the integrated diff, run `cargo build --workspace` and `cargo test -p melee_runtime`, and commit in small steps.

Review focus: null and unequal-length lists; virtual class dispatch and release order; PC render overrides and state invalidation; bit-exact frame/float forwarding; pointer identity and native callback ABI.

Execution: independent file groups use the parallel-agents workflow; root handles DObj, registration, integration, and commits. The user has already authorized implementation and commits.
