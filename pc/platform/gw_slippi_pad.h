/* Eight bytes sent in a Slippi PAD packet. The game-side TriggerSendInput.asm copies
 * PAD_REPORT_SIZE (12) bytes from PADStatus to EXI; pinned Dolphin SlippiPad.h defines
 * SLIPPI_PAD_DATA_SIZE as 8, and SlippiNetplay.cpp sends the first eight only.
 * Byte 0-1: physical PADStatus.button, big-endian; 2-5: signed main X/Y, C X/Y;
 * 6-7: unsigned physical L/R analog trigger bytes. No analog A/B or status is sent.
 * Sources: https://github.com/project-slippi/dolphin/blob/41a7a3a110ed52999486ae1901c8fbb9a63d4f13/Source/Core/Core/Slippi/SlippiPad.h
 * https://github.com/project-slippi/dolphin/blob/41a7a3a110ed52999486ae1901c8fbb9a63d4f13/Source/Core/Core/Slippi/SlippiNetplay.cpp
 * https://github.com/project-slippi/slippi-ssbm-asm/blob/fcf47f10dc244152c2ebaa3a9dec142ea42243b7/Online/Core/TriggerSendInput.asm
 * https://github.com/project-slippi/slippi-wiki/blob/71c6a395f841ff67f75ab4c0084fd1d6ee22c2db/SPEC.md
 * Pre-Frame event offsets 0x31, 0x33, 0x37 are buttons, L, R; raw stick bytes
 * are 0x3B, 0x40, 0x41, 0x42. Minimum modern Pre-Frame payload is 0x42.
 */
#ifndef GW_SLIPPI_PAD_H
#define GW_SLIPPI_PAD_H

#include <stdint.h>
#include "gw_rollback.h"

typedef struct GwSlippiPad { uint8_t bytes[8]; } GwSlippiPad;

/* Replay physical trigger floats are HSD_PadStatus.nml_analogL/R, recorded by
 * https://github.com/project-slippi/slippi-ssbm-asm/blob/fcf47f10dc244152c2ebaa3a9dec142ea42243b7/Recording/SendGamePreFrame.asm
 * Melee clamps raw bytes to 140 then divides by
 * scale_analogLR=140 (gmmain.c / controller.c). The inverse chooses canonical
 * raw byte 0..140, nearest with exact half steps up. Original bytes above 140
 * cannot be recovered but have identical post-clamp effect. */
int gw_SlippiPad_FromPhysical(uint16_t buttons, int8_t sx, int8_t sy, int8_t cx, int8_t cy,
                              float l, float r, GwSlippiPad *out);
uint8_t gw_SlippiPad_TriggerByte(float value);
int gw_SlippiPad_ToRb(const GwSlippiPad *pad, GwRbInput *out);
/* The replay frame is an APPLIED input; online PAD frame 1 is replay frame -123.
 * A sample at online frame F is transmitted for applied PAD frame F + delay.
 * Online PAD frames 1..delay are initially neutral, before any sample exists. */
int gw_SlippiPad_OnlineFrame(int slp_frame);
int gw_SlippiPad_SlpFrame(int online_frame);
int gw_SlippiPad_SampleFrame(int slp_frame, int delay);

typedef struct GwSlippiFixtureInfo {
    uint8_t version[4];
    uint8_t online;
    uint8_t human_ports[2]; /* zero-based physical port indices */
    int first_frame, last_frame;
    uint32_t seed;
    uint8_t game_info[0x138];
} GwSlippiFixtureInfo;

int gw_Replay_SlippiFixtureInfo(GwSlippiFixtureInfo *out);
/* Static diagnostic set by the most recent fixture-info call; contains no file path. */
const char *gw_Replay_SlippiFixtureReason(void);
int gw_Replay_SlippiPad(int port, int slp_frame, GwSlippiPad *out);
#endif
