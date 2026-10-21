#pragma once
/* Oracle stand-in: no device ever opens (SDL_InitSubSystem fails), so only the types matter. */
#include <stdbool.h>
#define SDLCALL
typedef struct SDL_AudioStream SDL_AudioStream;
typedef struct { int format, channels, freq; } SDL_AudioSpec;
#define SDL_AUDIO_S16 0
#define SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK 0
typedef void (*SDL_AudioStreamCallback)(void *, SDL_AudioStream *, int, int);
static inline bool SDL_PutAudioStreamData(SDL_AudioStream *s, const void *b, int n) { (void)s; (void)b; (void)n; return true; }
static inline SDL_AudioStream *SDL_OpenAudioDeviceStream(int d, const SDL_AudioSpec *s, SDL_AudioStreamCallback cb, void *u) { (void)d; (void)s; (void)cb; (void)u; return 0; }
static inline bool SDL_SetAudioStreamGain(SDL_AudioStream *s, float g) { (void)s; (void)g; return true; }
static inline bool SDL_ResumeAudioStreamDevice(SDL_AudioStream *s) { (void)s; return true; }
static inline const char *SDL_GetError(void) { return ""; }
