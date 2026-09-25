/* Offline Slippi Direct parser tests. No account file or network access. */
#include "../platform/gw_slippi_match.h"
#include <stdio.h>
#include <string.h>

static int failures;
#define CHECK(c) do { if (!(c)) { fprintf(stderr, "line %d: %s\n", __LINE__, #c); ++failures; } } while (0)

int main(void) {
  const char *p1 = "{\"uid\":\"one\",\"playKey\":\"secret-one\",\"connectCode\":\"ONE#1\",\"displayName\":\"One\",\"latestVersion\":\"3.6.4\"}";
  const char *p2 = "{\"uid\":\"two\",\"playKey\":\"secret-two\",\"connectCode\":\"TWO#2\",\"displayName\":\"Two\",\"latestVersion\":\"3.6.4\"}";
  const char *assignment = "{\"type\":\"get-ticket-resp\",\"matchId\":\"test.match\",\"isHost\":true,\"players\":[{\"uid\":\"one\",\"port\":1,\"isLocalPlayer\":true,\"ipAddress\":\"198.51.100.1:42000\"},{\"uid\":\"two\",\"port\":2,\"isLocalPlayer\":false,\"ipAddress\":\"203.0.113.2:43000\",\"ipAddressLan\":\"192.168.1.2:43000\"}]}";
  const char *same_wan = "{\"type\":\"get-ticket-resp\",\"matchId\":\"test.match\",\"isHost\":true,\"players\":[{\"uid\":\"one\",\"port\":1,\"isLocalPlayer\":true,\"ipAddress\":\"203.0.113.2:42000\"},{\"uid\":\"two\",\"port\":2,\"isLocalPlayer\":false,\"ipAddress\":\"203.0.113.2:43000\",\"ipAddressLan\":\"192.168.1.2:43000\"}]}";
  const char *bad_local_ip = "{\"type\":\"get-ticket-resp\",\"matchId\":\"test.match\",\"isHost\":true,\"players\":[{\"uid\":\"one\",\"port\":1,\"isLocalPlayer\":true,\"ipAddress\":\"203.0.113.999:42000\"},{\"uid\":\"two\",\"port\":2,\"isLocalPlayer\":false,\"ipAddress\":\"203.0.113.2:43000\",\"ipAddressLan\":\"192.168.1.2:43000\"}]}";
  GwSlippiMatchProfile a, b;
  GwSlippiMatchAssignment m;
  char request[1024];
  CHECK(gw_slippi_match_parse_profile(p1, &a) == 0);
  CHECK(gw_slippi_match_parse_profile(p2, &b) == 0);
  CHECK(strcmp(a.uid, b.uid) != 0);
  CHECK(gw_slippi_match_parse_profile("{\"uid\":", &a) < 0);
  CHECK(gw_slippi_match_parse_profile("{\"uid\":\"one\",\"connectCode\":\"ONE#1\",\"latestVersion\":\"3.6.4\"}", &a) < 0);
  CHECK(gw_slippi_match_make_request(&b, "ONE#1", "192.168.1.1:41001", request, sizeof request) == 0);
  CHECK(strstr(request, "\"mode\":2") != NULL);
  CHECK(strstr(request, "\"appVersion\":\"3.6.4\"") != NULL);
  CHECK(strstr(request, "\"connectCode\":[79,78,69,35,49]") != NULL);
  CHECK(gw_slippi_match_make_request(&b, "ONE#1\n", "192.168.1.1:41001", request, sizeof request) < 0);
  CHECK(gw_slippi_match_timeout_elapsed(100, 120099) == 0);
  CHECK(gw_slippi_match_timeout_elapsed(100, 120100) == 1);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"create-ticket-resp\"}", 0, &m) == 0);
  CHECK(gw_slippi_match_parse_response(assignment, 1, &m) == 1);
  CHECK(m.local_port == 0 && m.remote_port == 1 && m.is_host);
  CHECK(strcmp(m.peer_public, "203.0.113.2:43000") == 0);
  CHECK(strcmp(m.peer_lan, "192.168.1.2:43000") == 0);
  CHECK(m.same_external_ip == 0);
  CHECK(gw_slippi_match_parse_response(same_wan, 1, &m) == 1);
  CHECK(m.same_external_ip == 1);
  CHECK(gw_slippi_match_parse_response(bad_local_ip, 1, &m) < 0);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"get-ticket-resp\",\"error\":\"Play key expired: secret-one\"}", 1, &m) < 0);
  CHECK(strstr(gw_slippi_match_error(), "secret-one") == NULL);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"get-ticket-resp\",\"error\":\"not authorized: secret-two\"}", 1, &m) < 0);
  CHECK(strstr(gw_slippi_match_error(), "secret-two") == NULL);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"get-ticket-resp\",\"error\":\"appVersion rejected\",\"latestVersion\":\"3.6.5\"}", 1, &m) < 0);
  CHECK(strstr(gw_slippi_match_error(), "version") != NULL);
  CHECK(gw_slippi_match_parse_response("{bad", 1, &m) < 0);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"get-ticket-resp\",\"error\":123}", 1, &m) < 0);
  CHECK(gw_slippi_match_parse_response("{\"type\":\"get-ticket-resp\",\"players\":[]}", 1, &m) < 0);
  printf("slippi_match_test: %d failure(s)\n", failures);
  return failures ? 1 : 0;
}
