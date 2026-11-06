#!/usr/bin/env python3
"""Codec for Slippi Dolphin's netplay messages (the ENet payloads), from source.

Source: project-slippi/dolphin Source/Core/Core/Slippi/SlippiNetplay.cpp
(SendSlippiPad, OnData, writeToPacket, WriteChatMessageToPacket, SendGamePrepStep,
SendSyncedGameState, SendPlayerDropReport, SendSlippiPadSuggestion) and NetPlayProto.h
(MessageID 0x80..0x88). sf::Packet writes integers big-endian, bool as one byte,
std::string as u32 length + bytes.

This only builds and parses payload bytes; ENet framing (3 channels; pads/acks/
suggestions unsequenced on channel 1, ack replies on channel 2, the rest reliable
on channel 0) is not reproduced here. No sockets are opened.

usage: python -I slippi_wire.py --selftest
"""
import struct
import sys

PAD = 0x80
PAD_ACK = 0x81
MATCH_SELECTIONS = 0x82
CONN_SELECTED = 0x83
CHAT_MESSAGE = 0x84
COMPLETE_STEP = 0x85
SYNCED_STATE = 0x86
PAD_SUGGESTION = 0x87
PLAYER_DROP = 0x88

PAD_INPUT_SIZE = 8   # SlippiPad.h SLIPPI_PAD_INPUT_SIZE
PAD_WIRE_SIZE = 9    # inputs + 1 RNG-entropy byte the ASM mixes into the seed
PAD_FULL_SIZE = 12   # what the game gets back over EXI per frame


def pad_packet(frame, player_idx, checksum_frame, checksum, pads_newest_first):
    """SLIPPI_PAD: u8 id, s32 frame (newest), u8 player idx, s32 checksum frame,
    u32 checksum, then one 9-byte pad per unacked frame, newest first. Header = 14 bytes
    (OnData's pad_data_offset)."""
    out = struct.pack(">BiBiI", PAD, frame, player_idx, checksum_frame, checksum)
    for p in pads_newest_first:
        assert len(p) == PAD_WIRE_SIZE
        out += p
    return out


def parse_pad(b):
    mid, frame, pidx, cf, cs = struct.unpack_from(">BiBiI", b, 0)
    assert mid == PAD
    body = b[14:]
    n = len(body) // PAD_WIRE_SIZE
    pads = [(frame - i, body[i * PAD_WIRE_SIZE:(i + 1) * PAD_WIRE_SIZE]) for i in range(n)]
    return {"frame": frame, "player": pidx, "checksum_frame": cf, "checksum": cs, "pads": pads}


def ack_packet(frame, player_idx):
    return struct.pack(">BiB", PAD_ACK, frame, player_idx)


def selections_packet(char, color, char_selected, player_idx, stage, stage_selected,
                      rng_offset, team, alt_stage_mode):
    """writeToPacket: u8 char, u8 color, bool, u8 player, u16 stage, bool, u32 rng_offset,
    u8 team, u8 alt stage mode."""
    return struct.pack(">BBBBBH?IBB", MATCH_SELECTIONS, char, color, char_selected, player_idx,
                       stage, stage_selected, rng_offset, team, alt_stage_mode)


def chat_packet(message_id, player_idx):
    return struct.pack(">BiB", CHAT_MESSAGE, message_id, player_idx)


def step_packet(step, char, color, stage0, stage1):
    return struct.pack(">BBBBBB", COMPLETE_STEP, step, char, color, stage0, stage1)


def synced_state_packet(player_idx, match_id, game_idx, tiebreak_idx, seconds, fighters):
    m = match_id.encode()
    out = struct.pack(">BBI", SYNCED_STATE, player_idx, len(m)) + m
    out += struct.pack(">III", game_idx, tiebreak_idx, seconds)
    for stocks, health in fighters:
        out += struct.pack(">BH", stocks, health)
    return out


def selftest():
    pads = [bytes([i] * PAD_WIRE_SIZE) for i in (3, 2, 1)]
    b = pad_packet(10, 1, 7, 0xDEADBEEF, pads)
    assert len(b) == 14 + 27
    p = parse_pad(b)
    assert p["frame"] == 10 and p["player"] == 1 and p["checksum"] == 0xDEADBEEF
    assert [f for f, _ in p["pads"]] == [10, 9, 8]
    assert len(ack_packet(10, 0)) == 6
    assert len(selections_packet(2, 0, 1, 0, 0x1F, True, 1234, 0, 0)) == 14
    assert len(synced_state_packet(0, "mode.direct-x", 1, 0, 480, [(4, 0)] * 4)) == 1 + 1 + 4 + 13 + 12 + 12
    print("slippi_wire selftest OK")
    return 0


if __name__ == "__main__":
    sys.exit(selftest() if "--selftest" in sys.argv else (print(__doc__) or 2))
