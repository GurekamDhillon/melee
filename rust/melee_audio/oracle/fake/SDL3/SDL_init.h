#pragma once
#include <stdbool.h>
#define SDL_INIT_AUDIO 0
static inline bool SDL_InitSubSystem(int f) { (void)f; return false; }
