/* The per-call budget of the script engine: what stops a runaway call, and what must not.
 *
 * The instruction count is the limit. It does not depend on how busy the machine is, so a script that is fine on an idle
 * PC is fine on a loaded one. Wall time is only a backstop for a call that burns time without burning VM instructions
 * (a long native call in a loop); it is set an order of magnitude above a normal call, so CPU load from other programs
 * cannot trip it. (It was 50 ms: under load an Envoy mod failed to load at boot, with the count nowhere near its limit.)
 * Pure functions, so a native test can read them without the engine. */
#ifndef GW_SCRIPT_BUDGET_H
#define GW_SCRIPT_BUDGET_H

#define GW_SCRIPT_BUDGET_INSTR 2000000     /* VM instructions per call (MELEE_SCRIPT_BUDGET) */
#define GW_SCRIPT_BUDGET_MS 500.0          /* wall clock per call, backstop only (MELEE_SCRIPT_MS) */
#define GW_SCRIPT_HOOK_STEP 1000           /* the count hook fires every this many instructions */

/* 1 when the call must stop. `left` is the instruction allowance still unspent (checked after each hook step), `now_ms`
 * and `deadline_ms` the clock and the call's wall deadline, `turbo` 1 when rendering is uncapped and scripted logic runs
 * flat out (wall time means nothing then). */
static int gw_script_budget_tripped(long left, double now_ms, double deadline_ms, int turbo)
{
    if (left < 0) return 1;
    return !turbo && now_ms > deadline_ms;
}

#endif
