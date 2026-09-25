/* gw_slippi_match.c - Direct-only Slippi matchmaking over ENet.
 * SPDX-License-Identifier: GPL-2.0-or-later
 * Protocol basis: Project Slippi Dolphin SlippiMatchmaking.cpp at
 * 41a7a3a110ed52999486ae1901c8fbb9a63d4f13, GPL-2.0-or-later.
 * Tickets only: no ranked/unranked queue, match report, or replay upload.
 */
#include "gw_slippi_match.h"
#include "gw_slippi_enet.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

int gw_slippi_match_internal_error(const char *why);
enum { MM_IDLE, MM_CONNECT, MM_CREATE, MM_WAIT, MM_ASSIGNED, MM_ERROR };
static struct {
  ENetHost *host;
  ENetPeer *server;
  int state, port;
  ULONGLONG started_ms;
  GwSlippiMatchAssignment assignment;
} mm;

static void release_socket(void) {
  if (mm.server && mm.server->data) {
    enet_packet_destroy((ENetPacket *)mm.server->data);
    mm.server->data = NULL;
  }
  if (mm.server) { enet_peer_reset(mm.server); mm.server = NULL; }
  if (mm.host) { enet_host_destroy(mm.host); mm.host = NULL; }
  enet_deinitialize();
}
void gw_slippi_match_close(void) {
  if (mm.host) release_socket();
  memset(&mm, 0, sizeof mm);
}
static int error_state(const char *why) {
  gw_slippi_match_internal_error(why);
  if (mm.host) release_socket();
  mm.state = MM_ERROR;
  return -1;
}
static int read_profile(const char *path, char *buf, size_t cap) {
  FILE *f; size_t n; int extra;
  if (!path || !*path || cap < 2) return -1;
  f = NULL;
  if (strcmp(path, "-") == 0) f = stdin;
  else if (fopen_s(&f, path, "rb") != 0) f = NULL;
  if (!f) return -1;
  n = fread(buf, 1, cap - 1, f); extra = fgetc(f);
  if (f != stdin) fclose(f);
  if (extra != EOF || n == 0) { memset(buf, 0, cap); return -1; }
  buf[n] = 0; return 0;
}
int gw_slippi_match_start(const char *user_json_path, const char *direct_code) {
  char json[4096]; GwSlippiMatchProfile profile; int rc;
  if (read_profile(user_json_path, json, sizeof json))
    return gw_slippi_match_internal_error("Cannot read bounded Slippi user profile");
  rc = gw_slippi_match_parse_profile(json, &profile);
  SecureZeroMemory(json, sizeof json);
  if (rc < 0) return rc;
  rc = gw_slippi_match_start_profile(&profile, direct_code);
  SecureZeroMemory(&profile, sizeof profile);
  return rc;
}
static int lan_address(ENetAddress *server, int port, char *out, size_t cap) {
  ENetSocket sock = enet_socket_create(ENET_SOCKET_TYPE_DATAGRAM);
  ENetAddress local;
  unsigned char *ip;
  int n;
  if (sock == ENET_SOCKET_NULL) return -1;
  if (enet_socket_connect(sock, server) < 0 || enet_socket_get_address(sock, &local) < 0) {
    enet_socket_destroy(sock); return -1;
  }
  ip = (unsigned char *)&local.host;
  n = snprintf(out, cap, "%u.%u.%u.%u:%d", ip[0], ip[1], ip[2], ip[3], port);
  enet_socket_destroy(sock);
  return n > 0 && (size_t)n < cap ? 0 : -1;
}
int gw_slippi_match_start_profile(const GwSlippiMatchProfile *p, const char *direct_code) {
  ENetAddress local, server; char lan[64], request[2048];
  ENetPacket *packet; unsigned first, i;
  if (!p || !direct_code || !*direct_code) return gw_slippi_match_internal_error("Missing Direct code or profile");
  for (i = 0; direct_code[i]; i++) {
    if (i >= 16 || (unsigned char)direct_code[i] < 33 || (unsigned char)direct_code[i] > 126)
      return gw_slippi_match_internal_error("Invalid Direct connect code");
  }
  if (!p->uid[0] || !p->play_key[0] || !p->connect_code[0] || !p->display_name[0] || !p->app_version[0])
    return gw_slippi_match_internal_error("Incomplete Slippi profile");
  gw_slippi_match_close();
  if (enet_initialize() != 0) return error_state("Cannot initialize ENet matchmaking");
  /* Slippi Dolphin binds its ticket and subsequent peer to the same 41000..50999 UDP port. */
  first = 41000u + (unsigned)((GetTickCount() ^ GetCurrentProcessId()) % 10000u);
  local.host = ENET_HOST_ANY;
  for (i = 0; i < 10000; i++) {
    local.port = (enet_uint16)(41000u + ((first - 41000u + i) % 10000u));
    mm.host = enet_host_create(&local, 1, 3, 0, 0);
    if (mm.host) break;
  }
  if (!mm.host) { enet_deinitialize(); return error_state("Cannot bind Slippi matchmaking UDP port"); }
  mm.port = local.port;
  if (enet_address_set_host(&server, "mm.slippi.gg") < 0)
    return error_state("Cannot resolve Slippi matchmaking host");
  server.port = 43113;
  if (lan_address(&server, mm.port, lan, sizeof lan))
    return error_state("Cannot determine LAN address for Direct matchmaking");
  if (gw_slippi_match_make_request(p, direct_code, lan, request, sizeof request)) {
    SecureZeroMemory(request, sizeof request);
    return error_state("Cannot encode Direct matchmaking ticket");
  }
  mm.server = enet_host_connect(mm.host, &server, 3, 0);
  if (!mm.server) { SecureZeroMemory(request, sizeof request); return error_state("Cannot connect to Slippi matchmaking host"); }
  packet = enet_packet_create(request, strlen(request), ENET_PACKET_FLAG_RELIABLE);
  SecureZeroMemory(request, sizeof request);
  if (!packet) return error_state("Cannot allocate Direct ticket packet");
  /* Queued only after ENet connect: the host owns the packet until send succeeds. */
  mm.server->data = packet;
  mm.state = MM_CONNECT;
  mm.started_ms = GetTickCount64();
  return 0;
}
int gw_slippi_match_poll(GwSlippiMatchAssignment *out) {
  ENetEvent ev; int result, parsed;
  if (mm.state == MM_ASSIGNED) { if (out) *out = mm.assignment; return 1; }
  if (mm.state == MM_ERROR) return -1;
  if (!mm.host || !out) return gw_slippi_match_internal_error("Direct matchmaking was not started");
  if (gw_slippi_match_timeout_elapsed(mm.started_ms, GetTickCount64()))
    return error_state("Direct matchmaking timed out");
  for (;;) {
    result = enet_host_service(mm.host, &ev, 0);
    if (result < 0) return error_state("Direct matchmaking network failure");
    if (result == 0) return 0;
    if (ev.type == ENET_EVENT_TYPE_CONNECT && mm.state == MM_CONNECT) {
      ENetPacket *packet = (ENetPacket *)mm.server->data;
      mm.server->data = NULL;
      if (!packet || enet_peer_send(mm.server, 0, packet) < 0) {
        if (packet) enet_packet_destroy(packet);
        return error_state("Cannot send Direct matchmaking ticket");
      }
      enet_host_flush(mm.host); mm.state = MM_CREATE;
    } else if (ev.type == ENET_EVENT_TYPE_DISCONNECT) {
      return error_state("Slippi matchmaking server disconnected");
    } else if (ev.type == ENET_EVENT_TYPE_RECEIVE) {
      char message[16385]; size_t n = ev.packet->dataLength;
      if (ev.channelID != 0 || n == 0 || n > 16384 || memchr(ev.packet->data, 0, n)) {
        enet_packet_destroy(ev.packet); return error_state("Invalid matchmaking packet");
      }
      memcpy(message, ev.packet->data, n); message[n] = 0;
      enet_packet_destroy(ev.packet);
      parsed = gw_slippi_match_parse_response(message, mm.state != MM_CREATE, &mm.assignment);
      SecureZeroMemory(message, sizeof message);
      if (parsed < 0) {
        char reason[128];
        snprintf(reason, sizeof reason, "%s", gw_slippi_match_error());
        return error_state(reason);
      }
      if (mm.state == MM_CREATE) {
        if (parsed != 0) return error_state("Unexpected Direct ticket response");
        mm.state = MM_WAIT;
      } else if (mm.state == MM_WAIT && parsed == 1) {
        mm.assignment.local_udp_port = mm.port;
        release_socket(); mm.state = MM_ASSIGNED;
        *out = mm.assignment; return 1;
      }
    }
  }
}
