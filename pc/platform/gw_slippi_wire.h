/* SPDX-License-Identifier: GPL-2.0-or-later
 * Slippi peer packet codec. Fields and widths follow pinned Dolphin
 * SlippiNetplay.cpp 41a7a3a110ed52999486ae1901c8fbb9a63d4f13.
 * All integers are SFML packet/network byte order. Pads are opaque 8 bytes.
 */
#ifndef GW_SLIPPI_WIRE_H
#define GW_SLIPPI_WIRE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

#define GW_SLIPPI_MAX_PADS 128
#define GW_SLIPPI_PAD_BYTES 8
#define GW_SLIPPI_PAD_HEADER 14
#define GW_SLIPPI_MAX_PACKET (GW_SLIPPI_PAD_HEADER + GW_SLIPPI_MAX_PADS * GW_SLIPPI_PAD_BYTES)

enum { GW_SLIPPI_WIRE_PAD=0x80, GW_SLIPPI_WIRE_ACK=0x81,
       GW_SLIPPI_WIRE_SELECTIONS=0x82, GW_SLIPPI_WIRE_CONN_SELECTED=0x83,
       GW_SLIPPI_WIRE_PREP=0x85 };

typedef struct GwSlippiWirePad {
  int32_t frame;
  uint8_t player_idx;
  int32_t checksum_frame;
  uint32_t checksum;
  uint8_t count;
  uint8_t pads[GW_SLIPPI_MAX_PADS][GW_SLIPPI_PAD_BYTES]; /* newest first */
} GwSlippiWirePad;

typedef struct GwSlippiWireSelections {
  uint8_t character_id, character_color, character_selected, player_idx;
  uint16_t stage_id;
  uint8_t stage_selected;
  uint32_t rng_offset;
  uint8_t team_id, alt_stage_mode;
} GwSlippiWireSelections;

typedef struct GwSlippiWirePrep {
  uint8_t step_idx, character_id, character_color, stage_selections[2];
} GwSlippiWirePrep;

/* Return byte count, or 0 if arguments/output capacity are invalid. */
size_t gw_slippi_wire_encode_pad(uint8_t *out, size_t cap, const GwSlippiWirePad *pad);
/* expected_player 0..3; min/max are the current match's admissible online frames.
 * Rejects stale epoch by the caller's advancing minimum frame. */
int gw_slippi_wire_decode_pad(const uint8_t *data, size_t len, uint8_t expected_player,
                              int32_t min_frame, int32_t max_frame, GwSlippiWirePad *out);
size_t gw_slippi_wire_encode_ack(uint8_t *out, size_t cap, int32_t frame, uint8_t player_idx);
int gw_slippi_wire_decode_ack(const uint8_t *data, size_t len, uint8_t expected_player,
                              int32_t min_frame, int32_t max_frame, int32_t *frame);
size_t gw_slippi_wire_encode_selections(uint8_t *out, size_t cap, const GwSlippiWireSelections *s);
int gw_slippi_wire_decode_selections(const uint8_t *data, size_t len, GwSlippiWireSelections *out);
size_t gw_slippi_wire_encode_prep(uint8_t *out, size_t cap, const GwSlippiWirePrep *s);
int gw_slippi_wire_decode_prep(const uint8_t *data, size_t len, GwSlippiWirePrep *out);
size_t gw_slippi_wire_encode_conn_selected(uint8_t *out, size_t cap);
int gw_slippi_wire_decode_conn_selected(const uint8_t *data, size_t len);
/* SFML sf::Packet string form: network-order u32 byte count then exact bytes. */
size_t gw_slippi_wire_encode_string(uint8_t *out, size_t cap, const char *str, size_t len);
int gw_slippi_wire_decode_string(const uint8_t *data, size_t len, char *out, size_t cap,
                                 size_t *consumed);

#ifdef __cplusplus
}
#endif
#endif
