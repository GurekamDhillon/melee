/* AX shims: the audio DSP interface, stubbed.
 *
 * The port has no audio backend yet. Everything here is inert except the four FX init calls, which
 * axdriver.c checks for `== 1` before storing the effect handle, and the voice allocator below,
 * which must hand out a valid voice because the synth dereferences it unconditionally (see
 * _research/melee-boot.md §5). Nothing on the boot path waits on an AX callback: the two blocking
 * waits that matter (HSD_SynthSFXWaitForLoadCompletion and AXDriver_8038DA70) are completed by the
 * DVD/ARQ callback chain, which works. */
#include "gw.h"

/* ---- minimal voice pool --------------------------------------------------------------------
 * AXAcquireVoice cannot just return NULL. The synth's stream path (HSD_Synth_8038B5AC in synth.c)
 * reads voice->index immediately after acquiring, so a NULL result faults at NULL+0x18 the moment
 * the game starts a stream -- which is exactly what the title->main-menu transition does when it
 * starts the menu music. Model a 64-voice pool (AX_MAX_VOICES) that hands out distinct voices and
 * recycles them on AXFreeVoice. The game reads `index` through a byte-swapped load (it is a scalar
 * in game-visible memory), so it is written big-endian via gw_w32; callback/priority/userContext
 * are only touched by this shim and stay native. */

#define GW_AX_NUM_VOICES 64

/* Mirror of AXVPB (extern/dolphin/include/dolphin/ax.h): index at 0x18, pb at 0x138, 0x1F8 total.
 * Only `index` is read by game code; the full size is reserved so the stream-advance read at
 * voice+0x1B2 (synth.c HSD_Synth_8038ADD0) stays inside the allocation. */
typedef struct {
    void *next, *prev;        /* 0x000 */
    void *next1;              /* 0x008 */
    int priority;             /* 0x00C */
    void (*callback)(void *); /* 0x010 */
    uint32_t userContext;     /* 0x014 */
    uint32_t index;           /* 0x018 */
    uint8_t _rest[0x1F8 - 0x1C];
} gw_ax_voice;

static gw_ax_voice gw_ax_voices[GW_AX_NUM_VOICES];
static bool gw_ax_in_use[GW_AX_NUM_VOICES];

void gw_AXInit(void) {}

void gw_AXRegisterCallback(void *callback) { (void)callback; }

void gw_AXRegisterAuxACallback(void *callback, void *context) {
  (void)callback;
  (void)context;
}

void gw_AXRegisterAuxBCallback(void *callback, void *context) {
  (void)callback;
  (void)context;
}

void *gw_AXAcquireVoice(uint32_t priority, void *callback, uint32_t user_context) {
  gw_ax_voice *v = NULL;
  int i;

  for (i = 0; i < GW_AX_NUM_VOICES; ++i) {
    if (!gw_ax_in_use[i]) {
      v = &gw_ax_voices[i];
      break;
    }
  }
  if (v == NULL) {
    int best = -1;
    for (i = 0; i < GW_AX_NUM_VOICES; ++i) {
      if (gw_ax_in_use[i] && (best < 0 || gw_ax_voices[i].priority < gw_ax_voices[best].priority)) {
        best = i;
      }
    }
    if (best < 0) {
      return NULL;
    }
    v = &gw_ax_voices[best];
    if (v->callback != NULL) {
      v->callback(v); /* drop callback: the synth removes the node for this voice */
    }
  }

  i = (int)(v - gw_ax_voices);
  v->priority = (int)priority;
  v->callback = (void (*)(void *))callback;
  v->userContext = user_context;
  gw_w32(&v->index, (uint32_t)i);
  gw_ax_in_use[i] = true;
  return v;
}

void gw_AXFreeVoice(void *voice) {
  gw_ax_voice *v = (gw_ax_voice *)voice;
  if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
    return;
  }
  gw_ax_in_use[v - gw_ax_voices] = false;
}

void gw_AXSetVoicePriority(void *voice, uint32_t priority) {
  (void)voice;
  (void)priority;
}

void gw_AXSetVoiceState(void *voice, uint16_t state) {
  (void)voice;
  (void)state;
}

void gw_AXSetVoiceMix(void *voice, void *mix) {
  (void)voice;
  (void)mix;
}

void gw_AXSetVoiceItdOn(void *voice) { (void)voice; }

void gw_AXSetVoiceItdTarget(void *voice, uint16_t left_shift, uint16_t right_shift) {
  (void)voice;
  (void)left_shift;
  (void)right_shift;
}

void gw_AXSetVoiceVe(void *voice, void *ve) {
  (void)voice;
  (void)ve;
}

void gw_AXSetVoiceVeDelta(void *voice, int16_t delta) {
  (void)voice;
  (void)delta;
}

void gw_AXSetVoiceAddr(void *voice, void *addr) {
  (void)voice;
  (void)addr;
}

void gw_AXSetVoiceLoop(void *voice, uint16_t loop) {
  (void)voice;
  (void)loop;
}

void gw_AXSetVoiceLoopAddr(void *voice, uint32_t addr) {
  (void)voice;
  (void)addr;
}

void gw_AXSetVoiceEndAddr(void *voice, uint32_t addr) {
  (void)voice;
  (void)addr;
}

void gw_AXSetVoiceCurrentAddr(void *voice, uint32_t addr) {
  (void)voice;
  (void)addr;
}

void gw_AXSetVoiceAdpcm(void *voice, void *adpcm) {
  (void)voice;
  (void)adpcm;
}

void gw_AXSetVoiceSrc(void *voice, void *src) {
  (void)voice;
  (void)src;
}

void gw_AXSetVoiceSrcRatio(void *voice, float ratio) {
  (void)voice;
  (void)ratio;
}

void gw_AXSetVoiceAdpcmLoop(void *voice, void *adpcm_loop) {
  (void)voice;
  (void)adpcm_loop;
}

void gw_AXFXSetHooks(void *alloc_hook, void *free_hook) {
  (void)alloc_hook;
  (void)free_hook;
}

int gw_AXFXReverbHiInit(void *reverb) {
  (void)reverb;
  return 1;
}

int gw_AXFXReverbHiShutdown(void *reverb) {
  (void)reverb;
  return 1;
}

void gw_AXFXReverbHiCallback(void *buffer_update, void *reverb) {
  (void)buffer_update;
  (void)reverb;
}

int gw_AXFXReverbStdInit(void *reverb) {
  (void)reverb;
  return 1;
}

int gw_AXFXReverbStdShutdown(void *reverb) {
  (void)reverb;
  return 1;
}

void gw_AXFXReverbStdCallback(void *buffer_update, void *reverb) {
  (void)buffer_update;
  (void)reverb;
}

int gw_AXFXChorusInit(void *chorus) {
  (void)chorus;
  return 1;
}

int gw_AXFXChorusShutdown(void *chorus) {
  (void)chorus;
  return 1;
}

void gw_AXFXChorusCallback(void *buffer_update, void *chorus) {
  (void)buffer_update;
  (void)chorus;
}

int gw_AXFXDelayInit(void *delay) {
  (void)delay;
  return 1;
}

int gw_AXFXDelayShutdown(void *delay) {
  (void)delay;
  return 1;
}

void gw_AXFXDelayCallback(void *buffer_update, void *delay) {
  (void)buffer_update;
  (void)delay;
}
