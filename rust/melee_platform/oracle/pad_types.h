// shared by oracle.cpp and oracle_decode.c
#pragma once
#include <stdint.h>
typedef uint8_t u8; typedef int8_t s8; typedef uint16_t u16; typedef uint32_t u32;
#define PAD_ERR_NONE 0
#define PAD_CHANMAX 4
#define PAD_BUTTON_LEFT 0x0001
#define PAD_BUTTON_RIGHT 0x0002
#define PAD_BUTTON_DOWN 0x0004
#define PAD_BUTTON_UP 0x0008
#define PAD_TRIGGER_Z 0x0010
#define PAD_TRIGGER_R 0x0020
#define PAD_TRIGGER_L 0x0040
#define PAD_BUTTON_A 0x0100
#define PAD_BUTTON_B 0x0200
#define PAD_BUTTON_X 0x0400
#define PAD_BUTTON_Y 0x0800
#define PAD_BUTTON_START 0x1000
typedef struct PADStatus { u16 button; s8 stickX, stickY, substickX, substickY; u8 triggerLeft, triggerRight, analogA, analogB; s8 err; u32 extButton; } PADStatus;
