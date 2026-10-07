/* The script call budget: the instruction count is the limit, wall time only a far-out backstop. */
#include <stdio.h>
#include "../platform/gw_script_budget.h"

static int fails;
#define CHECK(c) do { if (!(c)) { printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); fails++; } } while (0)

/* Run a call that never ends (every hook step costs GW_SCRIPT_HOOK_STEP instructions and `ms_per_step` of wall time) and
 * report after how many steps the policy stops it; -1 when it never does within a sane number of steps. */
static int steps_until_stop(double ms_per_step, int turbo)
{
    long left = GW_SCRIPT_BUDGET_INSTR;
    double now = 1000.0, deadline = now + GW_SCRIPT_BUDGET_MS;
    int n;
    for (n = 1; n <= 100000; n++) {
        left -= GW_SCRIPT_HOOK_STEP;
        now += ms_per_step;
        if (gw_script_budget_tripped(left, now, deadline, turbo)) return n;
    }
    return -1;
}

int main(void)
{
    /* the instruction count alone stops an endless loop, even when the clock does not move at all */
    CHECK(steps_until_stop(0.0, 0) == GW_SCRIPT_BUDGET_INSTR / GW_SCRIPT_HOOK_STEP + 1);
    CHECK(steps_until_stop(0.0, 1) == GW_SCRIPT_BUDGET_INSTR / GW_SCRIPT_HOOK_STEP + 1);   /* turbo: count still binds */
    /* a loaded machine: a call at the old 50 ms mark, even at 10x the normal time per instruction, is NOT stopped by time */
    CHECK(!gw_script_budget_tripped(1500000L, 1000.0 + 60.0, 1000.0 + GW_SCRIPT_BUDGET_MS, 0));
    CHECK(!gw_script_budget_tripped(1500000L, 1000.0 + 450.0, 1000.0 + GW_SCRIPT_BUDGET_MS, 0));
    CHECK(GW_SCRIPT_BUDGET_MS >= 10.0 * 50.0);                             /* an order of magnitude above the old limit */
    /* wall time still stops a call that spends time without spending instructions, far beyond a normal call */
    CHECK(gw_script_budget_tripped(1990000L, 1000.0 + GW_SCRIPT_BUDGET_MS + 1.0, 1000.0 + GW_SCRIPT_BUDGET_MS, 0));
    CHECK(!gw_script_budget_tripped(1990000L, 1000.0 + GW_SCRIPT_BUDGET_MS + 1.0, 1000.0 + GW_SCRIPT_BUDGET_MS, 1));   /* not in turbo */
    /* a slow clock (0.5 ms per step: the loop costs a full second) is stopped by whichever limit comes first, never later */
    { int n = steps_until_stop(0.5, 0); CHECK(n > 0 && n <= GW_SCRIPT_BUDGET_INSTR / GW_SCRIPT_HOOK_STEP + 1); }
    /* an ordinary heavy call (300 000 instructions, 20 ms idle, 5x slower under load) passes */
    CHECK(!gw_script_budget_tripped(GW_SCRIPT_BUDGET_INSTR - 300000L, 1000.0 + 100.0, 1000.0 + GW_SCRIPT_BUDGET_MS, 0));
    /* the exact edge: spent exactly the allowance is fine, one step more is not */
    CHECK(!gw_script_budget_tripped(0L, 1000.0, 1500.0, 0) && gw_script_budget_tripped(-GW_SCRIPT_HOOK_STEP, 1000.0, 1500.0, 0));
    printf("script budget: %d failed\n", fails);
    return fails != 0;
}
