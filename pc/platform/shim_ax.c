/* AX shims: the audio DSP interface, stubbed.
 *
 * The port has no audio backend yet. Everything here is inert except the four FX init calls, which
 * axdriver.c checks for `== 1` before storing the effect handle, and AXAcquireVoice, which reports
 * "no voice" by returning NULL -- callers treat that as a normal failure and retry (see
 * _research/melee-boot.md §5). Nothing on the boot path waits on an AX callback: the two blocking
 * waits that matter (HSD_SynthSFXWaitForLoadCompletion and AXDriver_8038DA70) are completed by the
 * DVD/ARQ callback chain, which works. */
#include "gw.h"

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
  (void)priority;
  (void)callback;
  (void)user_context;
  return NULL;
}

void gw_AXFreeVoice(void *voice) { (void)voice; }

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
