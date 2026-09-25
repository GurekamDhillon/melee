/* SPDX-License-Identifier: GPL-2.0-or-later
 * Experimental replay-driver boot configuration, shared with its native test.
 * Reading this configuration does not load a replay, account, or network host.
 */
#ifndef GW_SLIPPI_MODE_CONFIG_H
#define GW_SLIPPI_MODE_CONFIG_H
#include <stdlib.h>
#include <string.h>
#include <errno.h>

typedef struct SmConfig {
    int direct, role, delay, local_udp_port, remote_udp_port;
    const char *profile, *code, *evidence;
} SmConfig;

static const char *sm_env(const char *name) {
    const char *value = getenv(name);
    return value && *value ? value : NULL;
}
static int sm_int(const char *name, int fallback, int low, int high, int *out) {
    const char *value = sm_env(name);
    char *end;
    long result;
    if (!value) { *out=fallback; return 1; }
    errno=0; result=strtol(value,&end,10);
    if (errno || end==value || *end || result<low || result>high) return 0;
    *out=(int)result; return 1;
}
static int sm_config(SmConfig *out, const char **why) {
    const char *mode=sm_env("MELEE_SLIPPI_MODE");
    memset(out,0,sizeof *out); *why=NULL;
    if (!mode) return 0;
    if (strcmp(mode,"direct") && strcmp(mode,"loopback")) {
        *why="mode must be loopback or direct"; return -1;
    }
    out->direct=!strcmp(mode,"direct");
    if (sm_env("MELEE_NETPLAY") || sm_env("MELEE_RB_FAKE") ||
        sm_env("MELEE_RB_INPUT") || sm_env("MELEE_SLP_RESYNC")) {
        *why="conflicting input or resync mode"; return -1;
    }
    if (!sm_int("MELEE_SLIPPI_REPLAY_ROLE",0,1,2,&out->role) ||
        !sm_int("MELEE_SLIPPI_DELAY",2,1,7,&out->delay) ||
        !sm_int("MELEE_SLIPPI_LOCAL_PORT",0,1,65535,&out->local_udp_port) ||
        !sm_int("MELEE_SLIPPI_REMOTE_PORT",0,1,65535,&out->remote_udp_port)) {
        *why="invalid numeric Slippi setting"; return -1;
    }
    if (!out->direct && (!out->role || !out->local_udp_port || !out->remote_udp_port ||
                         out->local_udp_port==out->remote_udp_port)) {
        *why="loopback requires a role and two distinct UDP ports"; return -1;
    }
    out->profile=sm_env("MELEE_SLIPPI_USER_JSON");
    out->code=sm_env("MELEE_SLIPPI_CODE");
    out->evidence=sm_env("MELEE_SLIPPI_EVIDENCE");
    if (out->direct && (!out->profile || !out->code)) {
        *why="Direct requires a profile and opponent connect code"; return -1;
    }
    if (!out->evidence) { *why="missing Slippi evidence output"; return -1; }
    return 1;
}
#endif
