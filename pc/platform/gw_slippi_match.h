/* gw_slippi_match.h - experimental Slippi Direct matchmaking.
 * SPDX-License-Identifier: GPL-2.0-or-later
 * Protocol basis: project-slippi/dolphin SlippiMatchmaking.cpp at
 * 41a7a3a110ed52999486ae1901c8fbb9a63d4f13 (GPL-2.0-or-later).
 * No profile contents or server response body may be logged.
 */
#ifndef GW_SLIPPI_MATCH_H
#define GW_SLIPPI_MATCH_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct GwSlippiMatchAssignment {
  int local_port;                 /* zero-based P1/P2 index */
  int remote_port;
  int is_host;
  int local_udp_port;             /* ticket socket's port; reuse for peer hole punching */
  uint64_t local_uid_hash;        /* stable private UID fingerprint; no raw UID in logs */
  uint64_t remote_uid_hash;
  char peer_public[64];          /* IPv4:UDP-port */
  char peer_lan[64];             /* IPv4:UDP-port, possibly empty */
  char match_id[128];
} GwSlippiMatchAssignment;

typedef struct GwSlippiMatchProfile {
  char uid[96];
  char play_key[128];
  char connect_code[32];
  char display_name[128];
  char app_version[32];          /* profile's actual latestVersion; never guessed */
} GwSlippiMatchProfile;

/* Pure parsers and request builder for offline tests. Errors are fixed sanitized strings. */
int gw_slippi_match_parse_profile(const char *json, GwSlippiMatchProfile *out);
int gw_slippi_match_parse_response(const char *json, int ticket_created,
                                   GwSlippiMatchAssignment *out);
int gw_slippi_match_make_request(const GwSlippiMatchProfile *profile,
                                 const char *direct_code, const char *lan_address,
                                 char *out, size_t cap);
int gw_slippi_match_timeout_elapsed(uint64_t started_ms, uint64_t now_ms);

/* 0 started, <0 error; poll returns 1 assigned, 0 pending, <0 error. */
int gw_slippi_match_start(const char *user_json_path, const char *direct_code);
int gw_slippi_match_start_profile(const GwSlippiMatchProfile *profile, const char *direct_code);
int gw_slippi_match_poll(GwSlippiMatchAssignment *out);
void gw_slippi_match_close(void);
const char *gw_slippi_match_error(void);

#ifdef __cplusplus
}
#endif
#endif
