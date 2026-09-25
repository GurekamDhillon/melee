/* SPDX-License-Identifier: GPL-2.0-or-later
 * Game-independent, single-threaded Slippi ENet peer. Port indices are wire 0..3.
 * The owner converts opaque eight-byte pads and submits them to rollback.
 */
#ifndef GW_SLIPPI_PEER_H
#define GW_SLIPPI_PEER_H
#include "gw_slippi_wire.h"
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct GwSlippiPeer GwSlippiPeer;
typedef struct GwSlippiPeerConfig {
  const char *remote_host;
  uint16_t local_udp_port, remote_udp_port;
  uint8_t local_port_idx, remote_port_idx;
  const char *match_id; /* diagnostics only; Slippi PAD has no match ID */
  int loopback_mode;
  void *user;
  /* Return nonzero only after the frame is stored; rejected frames remain unacknowledged. */
  int (*on_remote_pad)(void *user, int online_frame, const uint8_t pad[8]);
  void (*on_remote_checksum)(void *user, int checksum_frame, uint32_t checksum);
  void (*on_selections)(void *user, const GwSlippiWireSelections *s);
  void (*on_prep)(void *user, const GwSlippiWirePrep *s);
  void (*on_connection_selected)(void *user);
} GwSlippiPeerConfig;
typedef struct GwSlippiPeerStats {
  uint32_t packets_sent, packets_received, packets_rejected;
  uint32_t pads_sent, pads_received, retransmits, acks_sent, acks_received;
  int last_sent_frame, last_received_frame, last_acked_frame;
  int remote_checksum_frame;
  uint32_t remote_checksum;
  unsigned queued_local;
  int connected;
} GwSlippiPeerStats;
GwSlippiPeer *gw_slippi_peer_start(const GwSlippiPeerConfig *cfg);
void gw_slippi_peer_poll(GwSlippiPeer *peer, int current_online_frame);
int gw_slippi_peer_send_pad(GwSlippiPeer *peer, int online_frame, const uint8_t pad[8],
                            int checksum_frame, uint32_t checksum);
int gw_slippi_peer_send_selections(GwSlippiPeer *peer, const GwSlippiWireSelections *s);
int gw_slippi_peer_send_prep(GwSlippiPeer *peer, const GwSlippiWirePrep *s);
int gw_slippi_peer_send_connection_selected(GwSlippiPeer *peer);
void gw_slippi_peer_stats(const GwSlippiPeer *peer, GwSlippiPeerStats *out);
void gw_slippi_peer_close(GwSlippiPeer *peer);
#ifdef __cplusplus
}
#endif
#endif
