/* gw_slippi_match_json.c - bounded JSON codec for Slippi Direct matchmaking.
 * SPDX-License-Identifier: GPL-2.0-or-later
 * The message schema follows Project Slippi Dolphin SlippiMatchmaking.cpp,
 * commit 41a7a3a110ed52999486ae1901c8fbb9a63d4f13 (GPL-2.0-or-later).
 * This is an independent implementation, with fixed errors to avoid reflecting secrets.
 */
#include "gw_slippi_match.h"
#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { J_OBJ = 1, J_ARR, J_STR, J_NUM, J_TRUE, J_FALSE, J_NULL };
typedef struct { int start, end, next, type; } JTok;
typedef struct { const char *s; size_t len, pos; JTok t[128]; int count, bad; } JDoc;
static char match_error[128] = "No matchmaking error";
const char *gw_slippi_match_error(void) { return match_error; }
static int fail(const char *why) {
  snprintf(match_error, sizeof match_error, "%s", why);
  return -1;
}
int gw_slippi_match_internal_error(const char *why) { return fail(why); }
static void ws(JDoc *d) { while (d->pos < d->len && isspace((unsigned char)d->s[d->pos])) d->pos++; }
static int token(JDoc *d, int type) {
  int n = d->count++;
  if (n >= (int)(sizeof d->t / sizeof d->t[0])) { d->bad = 1; return -1; }
  d->t[n].type = type; d->t[n].start = (int)d->pos; d->t[n].end = -1; d->t[n].next = -1;
  return n;
}
static int jvalue(JDoc *d, int depth);
static int jstring(JDoc *d) {
  int n = token(d, J_STR);
  if (n < 0) return -1;
  d->pos++; d->t[n].start = (int)d->pos;
  while (d->pos < d->len) {
    unsigned char c = (unsigned char)d->s[d->pos++];
    if (c == '"') { d->t[n].end = (int)d->pos - 1; d->t[n].next = d->count; return n; }
    if (c < 32) break;
    if (c == '\\') {
      if (d->pos >= d->len) break;
      c = (unsigned char)d->s[d->pos++];
      if (c == 'u') {
        int i;
        for (i = 0; i < 4; i++) {
          if (d->pos >= d->len || !isxdigit((unsigned char)d->s[d->pos++])) { d->bad = 1; break; }
        }
        if (d->bad) break;
      } else if (!strchr("\"\\/bfnrt", c)) break;
    }
  }
  d->bad = 1; return -1;
}
static int literal(JDoc *d, const char *value, int type) {
  size_t len = strlen(value); int n;
  if (d->len - d->pos < len || memcmp(d->s + d->pos, value, len)) { d->bad = 1; return -1; }
  n = token(d, type); if (n < 0) return -1;
  d->pos += len; d->t[n].end = (int)d->pos; d->t[n].next = d->count; return n;
}
static int jvalue(JDoc *d, int depth) {
  int n, child;
  if (depth > 16) { d->bad = 1; return -1; }
  ws(d); if (d->pos >= d->len) { d->bad = 1; return -1; }
  if (d->s[d->pos] == '"') return jstring(d);
  if (d->s[d->pos] == 't') return literal(d, "true", J_TRUE);
  if (d->s[d->pos] == 'f') return literal(d, "false", J_FALSE);
  if (d->s[d->pos] == 'n') return literal(d, "null", J_NULL);
  if (d->s[d->pos] == '{' || d->s[d->pos] == '[') {
    int obj = d->s[d->pos] == '{', first = 1;
    n = token(d, obj ? J_OBJ : J_ARR); if (n < 0) return -1;
    d->pos++;
    for (;;) {
      ws(d);
      if (d->pos >= d->len) break;
      if (d->s[d->pos] == (obj ? '}' : ']')) {
        d->pos++; d->t[n].end = (int)d->pos; d->t[n].next = d->count; return n;
      }
      if (!first) {
        if (d->s[d->pos++] != ',') break;
        ws(d);
        if (d->pos >= d->len || d->s[d->pos] == (obj ? '}' : ']')) break;
      }
      if (obj) {
        if (d->s[d->pos] != '"') break;
        child = jstring(d); if (child < 0) break;
        ws(d); if (d->pos >= d->len || d->s[d->pos++] != ':') break;
      }
      child = jvalue(d, depth + 1); if (child < 0) break;
      first = 0;
    }
    d->bad = 1; return -1;
  }
  if (d->s[d->pos] == '-' || isdigit((unsigned char)d->s[d->pos])) {
    size_t start = d->pos;
    if (d->s[d->pos] == '-') d->pos++;
    if (d->pos >= d->len) { d->bad = 1; return -1; }
    if (d->s[d->pos] == '0') d->pos++;
    else {
      if (!isdigit((unsigned char)d->s[d->pos])) { d->bad = 1; return -1; }
      while (d->pos < d->len && isdigit((unsigned char)d->s[d->pos])) d->pos++;
    }
    if (d->pos < d->len && d->s[d->pos] == '.') {
      d->pos++; if (d->pos >= d->len || !isdigit((unsigned char)d->s[d->pos])) { d->bad = 1; return -1; }
      while (d->pos < d->len && isdigit((unsigned char)d->s[d->pos])) d->pos++;
    }
    if (d->pos < d->len && (d->s[d->pos] == 'e' || d->s[d->pos] == 'E')) {
      d->pos++; if (d->pos < d->len && (d->s[d->pos] == '+' || d->s[d->pos] == '-')) d->pos++;
      if (d->pos >= d->len || !isdigit((unsigned char)d->s[d->pos])) { d->bad = 1; return -1; }
      while (d->pos < d->len && isdigit((unsigned char)d->s[d->pos])) d->pos++;
    }
    n = token(d, J_NUM); if (n < 0) return -1;
    d->t[n].start = (int)start; d->t[n].end = (int)d->pos; d->t[n].next = d->count; return n;
  }
  d->bad = 1; return -1;
}
static int parse(JDoc *d, const char *s) {
  if (!s) return -1;
  memset(d, 0, sizeof *d); d->s = s; d->len = strlen(s);
  if (d->len > 16384 || jvalue(d, 0) != 0 || d->bad) return -1;
  ws(d); return d->pos == d->len && d->t[0].type == J_OBJ ? 0 : -1;
}
static int str(JDoc *d, int t, char *out, size_t cap) {
  int i, k = 0;
  if (t < 0 || d->t[t].type != J_STR || cap == 0) return -1;
  for (i = d->t[t].start; i < d->t[t].end; i++) {
    unsigned char c = (unsigned char)d->s[i];
    if (c == '\\') {
      c = (unsigned char)d->s[++i];
      if (c == 'u') return -1; /* identifiers and addresses must be plain ASCII */
      if (c == 'n') c = '\n'; else if (c == 'r') c = '\r'; else if (c == 't') c = '\t';
      else if (c == 'b' || c == 'f') return -1;
    }
    if (!c || k + 1 >= (int)cap) return -1;
    out[k++] = (char)c;
  }
  out[k] = 0; return 0;
}
static int eq(JDoc *d, int t, const char *s) {
  char key[64]; return str(d, t, key, sizeof key) == 0 && strcmp(key, s) == 0;
}
static int field(JDoc *d, int obj, const char *key) {
  int t;
  if (obj < 0 || d->t[obj].type != J_OBJ) return -1;
  for (t = obj + 1; t < d->t[obj].next;) {
    int v = t + 1;
    if (v >= d->count) return -1;
    if (eq(d, t, key)) return v;
    t = d->t[v].next;
  }
  return -1;
}
static int get(JDoc *d, int obj, const char *key, char *out, size_t cap) {
  return str(d, field(d, obj, key), out, cap);
}
static int num(JDoc *d, int t) {
  char b[16]; int n;
  if (t < 0 || d->t[t].type != J_NUM) return -1;
  n = d->t[t].end - d->t[t].start;
  if (n < 1 || n >= (int)sizeof b) return -1;
  memcpy(b, d->s + d->t[t].start, n); b[n] = 0;
  for (int i = 0; i < n; i++) if (!isdigit((unsigned char)b[i])) return -1;
  return atoi(b);
}
int gw_slippi_match_parse_profile(const char *json, GwSlippiMatchProfile *out) {
  JDoc d; GwSlippiMatchProfile p;
  if (!out || parse(&d, json)) return fail("Invalid Slippi profile JSON");
  memset(&p, 0, sizeof p);
  if (get(&d, 0, "uid", p.uid, sizeof p.uid) || !p.uid[0] ||
      get(&d, 0, "playKey", p.play_key, sizeof p.play_key) || !p.play_key[0] ||
      get(&d, 0, "connectCode", p.connect_code, sizeof p.connect_code) || !p.connect_code[0] ||
      get(&d, 0, "displayName", p.display_name, sizeof p.display_name) || !p.display_name[0])
    return fail("Slippi profile is missing an account field or play key");
  if (get(&d, 0, "latestVersion", p.app_version, sizeof p.app_version) || !p.app_version[0])
    return fail("Slippi profile has no app version");
  *out = p; return 0;
}
static int contains_ci(const char *s, const char *word) {
  size_t n = strlen(word); const char *p;
  for (p = s; *p; p++) {
    size_t i = 0;
    while (i < n && p[i] && tolower((unsigned char)p[i]) == tolower((unsigned char)word[i])) i++;
    if (i == n) return 1;
  }
  return 0;
}
static int address(const char *s) {
  int i; unsigned value;
  if (!s) return 0;
  for (i = 0; i < 5; i++) {
    int digits = 0; value = 0;
    while (*s >= '0' && *s <= '9' && digits < 5) {
      value = value * 10 + (unsigned)(*s++ - '0'); digits++;
    }
    if (!digits || (i < 4 && (digits > 3 || value > 255)) ||
        (i == 4 && (value == 0 || value > 65535))) return 0;
    if (i < 3 && *s++ != '.') return 0;
    if (i == 3 && *s++ != ':') return 0;
  }
  return *s == 0;
}
static uint64_t uid_hash(const char *s) {
  uint64_t h = UINT64_C(14695981039346656037);
  while (*s) { h ^= (unsigned char)*s++; h *= UINT64_C(1099511628211); }
  return h;
}
int gw_slippi_match_parse_response(const char *json, int ticket_created,
                                   GwSlippiMatchAssignment *out) {
  JDoc d; char type[32], error[256]; int players, t, local = -1, remote = -1, count = 0;
  GwSlippiMatchAssignment m;
  if (!out || parse(&d, json) || get(&d, 0, "type", type, sizeof type))
    return fail("Invalid matchmaking response");
  if (strcmp(type, ticket_created ? "get-ticket-resp" : "create-ticket-resp"))
    return fail("Unexpected matchmaking response type");
  { int e = field(&d, 0, "error");
    if (e >= 0 && d.t[e].type != J_STR && d.t[e].type != J_NULL)
      return fail("Invalid matchmaking error response");
  }
  if (get(&d, 0, "error", error, sizeof error) == 0 && error[0]) {
    if (contains_ci(error, "version")) return fail("Slippi app version rejected by matchmaking server");
    if (contains_ci(error, "key") || contains_ci(error, "auth") || contains_ci(error, "expired"))
      return fail("Slippi authentication rejected; play key may be expired");
    return fail("Slippi matchmaking server rejected request");
  }
  if (!ticket_created) return 0;
  memset(&m, 0, sizeof m);
  if (get(&d, 0, "matchId", m.match_id, sizeof m.match_id) || !m.match_id[0])
    return fail("Matchmaking assignment has no match ID");
  { int host = field(&d, 0, "isHost");
    if (host < 0 || (d.t[host].type != J_TRUE && d.t[host].type != J_FALSE))
      return fail("Matchmaking assignment has no host role");
    m.is_host = d.t[host].type == J_TRUE;
  }
  players = field(&d, 0, "players");
  if (players < 0 || d.t[players].type != J_ARR) return fail("Matchmaking assignment has no players");
  for (t = players + 1; t < d.t[players].next; t = d.t[t].next) {
    int port = num(&d, field(&d, t, "port"));
    int is_local = field(&d, t, "isLocalPlayer");
    char uid[96];
    if (d.t[t].type != J_OBJ || (port != 1 && port != 2) || is_local < 0 ||
        (d.t[is_local].type != J_TRUE && d.t[is_local].type != J_FALSE) ||
        get(&d, t, "uid", uid, sizeof uid) || !uid[0])
      return fail("Invalid matchmaking player assignment");
    if (d.t[is_local].type == J_TRUE) {
      if (local >= 0) return fail("Invalid matchmaking player assignment");
      local = port - 1; m.local_uid_hash = uid_hash(uid);
    } else {
      if (remote >= 0 || get(&d, t, "ipAddress", m.peer_public, sizeof m.peer_public) ||
          !address(m.peer_public)) return fail("Invalid matchmaking peer address");
      if (get(&d, t, "ipAddressLan", m.peer_lan, sizeof m.peer_lan)) m.peer_lan[0] = 0;
      if (m.peer_lan[0] && !address(m.peer_lan)) return fail("Invalid matchmaking LAN address");
      remote = port - 1; m.remote_uid_hash = uid_hash(uid);
    }
    count++;
  }
  if (count != 2 || local < 0 || remote < 0 || local == remote ||
      m.local_uid_hash == m.remote_uid_hash)
    return fail("Matchmaking requires two distinct player ports");
  m.local_port = local; m.remote_port = remote; *out = m; return 1;
}
int gw_slippi_match_timeout_elapsed(uint64_t started_ms, uint64_t now_ms) {
  return now_ms >= started_ms && now_ms - started_ms >= UINT64_C(120000);
}
static int append(char **out, size_t *cap, const char *s) {
  size_t n = strlen(s);
  if (n >= *cap) return -1;
  memcpy(*out, s, n); *out += n; *cap -= n; **out = 0; return 0;
}
static int quoted(char **out, size_t *cap, const char *s) {
  if (append(out, cap, "\"")) return -1;
  for (; *s; s++) {
    char c[3] = {0,0,0};
    if ((unsigned char)*s < 32) return -1;
    if (*s == '"' || *s == '\\') c[0] = '\\', c[1] = *s;
    else c[0] = *s;
    if (append(out, cap, c)) return -1;
  }
  return append(out, cap, "\"");
}
int gw_slippi_match_make_request(const GwSlippiMatchProfile *p, const char *code,
                                 const char *lan, char *out, size_t cap) {
  char *q = out; size_t i;
  if (!p || !code || !*code || !lan || !address(lan) || !out || cap < 2)
    return fail("Invalid Direct matchmaking request");
  out[0] = 0;
  if (append(&q,&cap,"{\"type\":\"create-ticket\",\"user\":{\"uid\":") || quoted(&q,&cap,p->uid) ||
      append(&q,&cap,",\"playKey\":") || quoted(&q,&cap,p->play_key) ||
      append(&q,&cap,",\"connectCode\":") || quoted(&q,&cap,p->connect_code) ||
      append(&q,&cap,",\"displayName\":") || quoted(&q,&cap,p->display_name) ||
      append(&q,&cap,"},\"search\":{\"mode\":2,\"connectCode\":["))
    return fail("Direct request exceeds buffer");
  for (i = 0; code[i]; i++) {
    char numbuf[8];
    if (i >= 16 || (unsigned char)code[i] < 33 || (unsigned char)code[i] > 126)
      return fail("Invalid Direct connect code");
    snprintf(numbuf, sizeof numbuf, "%s%u", i ? "," : "", (unsigned char)code[i]);
    if (append(&q,&cap,numbuf)) return fail("Direct request exceeds buffer");
  }
  if (append(&q,&cap,"]},\"appVersion\":") || quoted(&q,&cap,p->app_version) ||
      append(&q,&cap,",\"ipAddressLan\":") || quoted(&q,&cap,lan) || append(&q,&cap,"}"))
    return fail("Direct request exceeds buffer");
  return 0;
}
