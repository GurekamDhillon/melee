/* SPDX-License-Identifier: GPL-2.0-or-later
 * Boot gating: an unset mode cannot touch a fixture/account or open a socket;
 * an invalid/conflicting configuration fails before network initialization.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../platform/gw_slippi_mode_config.h"

static int failures;
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "FAIL line %d: %s\n", __LINE__, #x); failures++; } } while (0)
static void clear_config(void) {
    const char *names[] = {"MELEE_SLIPPI_MODE", "MELEE_SLIPPI_REPLAY_ROLE",
        "MELEE_SLIPPI_DELAY", "MELEE_SLIPPI_LOCAL_PORT", "MELEE_SLIPPI_REMOTE_PORT",
        "MELEE_SLIPPI_USER_JSON", "MELEE_SLIPPI_CODE", "MELEE_SLIPPI_EVIDENCE",
        "MELEE_NETPLAY", "MELEE_RB_FAKE", "MELEE_RB_INPUT", "MELEE_SLP_RESYNC"};
    unsigned i;
    for (i=0;i<sizeof names/sizeof names[0];i++) _putenv_s(names[i], "");
}
int main(void) {
    SmConfig config;
    const char *why = NULL;
    clear_config();
    CHECK(sm_config(&config, &why) == 0);
    _putenv_s("MELEE_SLIPPI_MODE", "typo");
    CHECK(sm_config(&config, &why) < 0 && why != NULL);
    _putenv_s("MELEE_SLIPPI_MODE", "loopback");
    CHECK(sm_config(&config, &why) < 0);
    _putenv_s("MELEE_SLIPPI_REPLAY_ROLE", "1");
    _putenv_s("MELEE_SLIPPI_LOCAL_PORT", "45101");
    _putenv_s("MELEE_SLIPPI_REMOTE_PORT", "45102");
    _putenv_s("MELEE_SLIPPI_EVIDENCE", "evidence.json");
    CHECK(sm_config(&config, &why) == 1 && config.role == 1 && config.delay == 2);
    _putenv_s("MELEE_RB_FAKE", "4");
    CHECK(sm_config(&config, &why) < 0);
    _putenv_s("MELEE_RB_FAKE", "");
    _putenv_s("MELEE_SLIPPI_DELAY", "2junk");
    CHECK(sm_config(&config, &why) < 0);
    _putenv_s("MELEE_SLIPPI_DELAY", "");
    _putenv_s("MELEE_SLIPPI_LOCAL_PORT", "65536");
    CHECK(sm_config(&config, &why) < 0);
    _putenv_s("MELEE_SLIPPI_LOCAL_PORT", "45101");
    _putenv_s("MELEE_SLIPPI_MODE", "direct");
    CHECK(sm_config(&config, &why) < 0);
    _putenv_s("MELEE_SLIPPI_USER_JSON", "-");
    _putenv_s("MELEE_SLIPPI_CODE", "TEST#123");
    CHECK(sm_config(&config, &why) == 1 && config.direct);
    clear_config();
    printf("slippi-mode: %s\n", failures ? "FAIL" : "PASS");
    return failures != 0;
}
