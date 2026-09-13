/* AX shims: a real audio backend over the existing 64-voice pool.
 *
 * The game's synth (synth.c) is fully implemented and drives everything through the 35 AX entry
 * points below plus the four AI entries in shim_misc.c. On hardware the AX library runs on the
 * DSP and raises a 5 ms frame interrupt that calls the registered callback; here that role is
 * played by gw_ax_frame_tick, which shim_vi.c invokes from gw_frame_tick on the game thread.
 *
 * Layout of the work:
 *   - gw_AXRegisterCallback stores the game's HSD_SynthCallback; gw_ax_frame_tick calls it once
 *     per 5 ms sub-frame, then mixes that sub-frame's 160 stereo samples from the voice pool.
 *   - Every AXSetVoice* call copies its game-memory (big-endian) parameters into a host-side
 *     per-voice state struct using gw_r16/gw_r32, so the mixer never touches game memory while
 *     mixing and no locking is needed.
 *   - The mixer decodes Nintendo DSP-ADPCM (the .ssm/.hps format), resamples by the AXPBSRC
 *     ratio, applies ve volume + AXPBMIX pan, and writes back pb.state and
 *     pb.addr.currentAddressHi/Lo (big-endian) because the synth reads both (synth.c:1262,
 *     787-804, 1186-1187).
 *   - Output is a lock-free single-producer/single-consumer ring buffer drained by an SDL3 audio
 *     stream callback. The callback only copies out of the ring; it never touches game memory.
 *   - FX (reverb/chorus/delay) stays stubbed returning 1 for this milestone (axdriver.c tests
 *     `== 1` before registering the effect handle). ITD is a cheap no-op.
 *
 * DSP addressing: a game "DSP address" is an ARAM byte offset * 2 (see synth.c:1495
 * `HSD_Synth_804D7784 *= 2`), i.e. the DSP addresses ARAM in nibbles. A DSP address d maps to
 * host `gw_aram + d/2`; the low bit of d selects the high/low nibble of that byte.
 */
#include "gw.h"
#include "shim_vi.h"
#include "shim_ax.h"

#include <SDL3/SDL_audio.h>
#include <SDL3/SDL_init.h>

#include <stdlib.h>
#include <string.h>

/* ---- voice pool ---------------------------------------------------------------------------
 * AXAcquireVoice cannot return NULL: the synth's stream path (HSD_Synth_8038B5AC) reads
 * voice->index immediately after acquiring, so NULL faults at NULL+0x18 the moment the game starts
 * the menu music. This is the same 64-voice AXVPB-layout pool as before, unchanged in its
 * allocation/steal semantics; only the mixer state beside it is new. */
#define GW_AX_NUM_VOICES 64

/* Mirror of AXVPB (extern/dolphin/include/dolphin/ax.h): index at 0x18, pb at 0x138, 0x1F8 total.
 * `index` is read by game code through a byte-swapped load, so it is written big-endian; the full
 * size is reserved so the stream-advance read at voice+0x1B2 stays inside the allocation. */
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

/* Game-visible offsets into the AXVPB (big-endian game memory). */
#define GW_AX_VPB_PB_STATE 0x146 /* AXVPB.pb.state (u16) */
#define GW_AX_VPB_CURADDR 0x1B2  /* AXVPB.pb.addr.currentAddressHi/Lo (u32) */

/* ---- per-voice mixer state (host side, never game-visible) -------------------------------- */
typedef struct {
    bool active;     /* AXSetVoiceState(1) seen and voice not finished */
    bool configured; /* AXSetVoiceAddr + AXSetVoiceAdpcm seen */

    /* ADPCM decoder */
    int16_t coef[8][2];
    uint8_t pred_scale; /* current frame: predictor<<4 | scale */
    int16_t yn1, yn2;   /* sample history */
    uint8_t init_pred_scale; /* init state from AXPBADPCM (restored on seek) */
    int16_t init_yn1, init_yn2;
    uint32_t cur_addr;  /* DSP (nibble) address of next data nibble */
    uint32_t end_addr;  /* exclusive end, DSP address */
    uint32_t loop_addr; /* restart address, DSP address */
    uint16_t loop_flag; /* 0 = stop at end, 1 = loop */
    uint16_t format;    /* AXPBADDR.format: 0 = ADPCM (supported) */
    uint8_t loop_pred_scale;
    int16_t loop_yn1, loop_yn2;
    uint8_t samples_left; /* data nibbles left in the current frame (ADPCM: 14) */

    /* resampling (16.16 source samples per output sample) */
    uint32_t ratio;
    uint32_t frac;
    int16_t last; /* last decoded sample (nearest-neighbour) */

    /* volume / pan */
    int32_t volume;       /* ve current volume 0..32767 */
    int16_t volume_delta; /* ve.currentDelta, applied per sample */
    int32_t mix_l, mix_r; /* AXPBMIX.vL / .vR */
    uint16_t state;       /* mirrored to pb.state */
} gw_ax_vstate;

static gw_ax_vstate gw_ax_state[GW_AX_NUM_VOICES];

static void (*gw_ax_cb)(void);

/* ---- AI state (written by shim_misc.c) ---------------------------------------------------- */
uint8_t gw_ai_stream_vol_left = 255;
uint8_t gw_ai_stream_vol_right = 255;
uint32_t gw_ai_dsp_sample_rate = 0;

/* ---- output: SDL3 audio + ring buffer ------------------------------------------------------ */
#define GW_AX_RATE 32000
#define GW_AX_FRAME_SAMPLES 160 /* 5 ms at 32 kHz */
#define GW_AX_RING_BYTES 65536

static uint8_t gw_ax_ring[GW_AX_RING_BYTES];
static volatile uint32_t gw_ax_ring_w; /* bytes produced (monotonic) */
static volatile uint32_t gw_ax_ring_r; /* bytes consumed (monotonic) */

static SDL_AudioStream *gw_ax_stream;
static bool gw_ax_dev_ok;
static bool gw_ax_dev_attempted;

static void gw_ax_ring_push(const void *data, uint32_t n) {
    uint32_t w = gw_ax_ring_w;
    uint32_t avail = GW_AX_RING_BYTES - (w - gw_ax_ring_r);
    if (avail < n) {
        /* Producer is ahead of the consumer (should not happen at real-time rates): drop rather
         * than corrupt the cursor invariant. */
        n = avail;
        if (n == 0) {
            return;
        }
    }
    {
        const uint8_t *src = (const uint8_t *)data;
        uint32_t wpos = w & (GW_AX_RING_BYTES - 1);
        uint32_t first = GW_AX_RING_BYTES - wpos;
        if (first > n) {
            first = n;
        }
        memcpy(gw_ax_ring + wpos, src, first);
        memcpy(gw_ax_ring, src + first, n - first);
    }
    gw_ax_ring_w = w + n; /* publish */
}

/* SDL audio callback. Runs on SDL's thread; only copies out of the ring (and pads with silence
 * on underrun). Never touches game memory or takes game locks. */
static void SDLCALL gw_ax_sdl_callback(void *userdata, SDL_AudioStream *stream,
                                       int additional_amount, int total_amount) {
    uint8_t buf[8192];
    (void)userdata;
    (void)total_amount;
    if (additional_amount <= 0) {
        return;
    }
    uint32_t avail = gw_ax_ring_w - gw_ax_ring_r;
    uint32_t take = (uint32_t)additional_amount;
    if (take > avail) {
        take = avail;
    }
    uint32_t done = 0;
    while (done < take) {
        uint32_t chunk = take - done;
        if (chunk > sizeof buf) {
            chunk = sizeof buf;
        }
        {
            uint32_t r = gw_ax_ring_r;
            uint32_t rpos = r & (GW_AX_RING_BYTES - 1);
            uint32_t first = GW_AX_RING_BYTES - rpos;
            if (first > chunk) {
                first = chunk;
            }
            memcpy(buf, gw_ax_ring + rpos, first);
            memcpy(buf + first, gw_ax_ring, chunk - first);
            gw_ax_ring_r = r + chunk;
        }
        SDL_PutAudioStreamData(stream, buf, (int)chunk);
        done += chunk;
    }
    uint32_t remain = (uint32_t)additional_amount - done;
    while (remain > 0) {
        uint32_t chunk = remain > sizeof buf ? sizeof buf : remain;
        memset(buf, 0, chunk);
        SDL_PutAudioStreamData(stream, buf, (int)chunk);
        remain -= chunk;
    }
}

static void gw_ax_open_device(void) {
    SDL_AudioSpec spec;
    if (gw_ax_dev_attempted) {
        return;
    }
    gw_ax_dev_attempted = true;
    if (!SDL_InitSubSystem(SDL_INIT_AUDIO)) {
        gw_log("gw: AX: SDL audio init failed: %s", SDL_GetError());
        return;
    }
    spec.format = SDL_AUDIO_S16;
    spec.channels = 2;
    spec.freq = GW_AX_RATE;
    gw_ax_stream =
        SDL_OpenAudioDeviceStream(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, &spec, &gw_ax_sdl_callback, NULL);
    if (gw_ax_stream == NULL) {
        gw_log("gw: AX: SDL_OpenAudioDeviceStream failed: %s", SDL_GetError());
        return;
    }
    SDL_ResumeAudioStreamDevice(gw_ax_stream);
    gw_ax_dev_ok = true;
    gw_log("gw: AX: audio device open (32 kHz stereo s16)");
}

/* ---- ADPCM decode --------------------------------------------------------------------------
 * Nintendo DSP-ADPCM: 8-byte frames, each a predictor/scale header byte + 7 bytes (14 nibbles)
 * of 4-bit deltas. The first frame's header is supplied by AXPBADPCM.pred_scale (the byte at the
 * sample's start is therefore skipped; the current address already points past it). Each nibble n
 * decodes as:  sample = (n * 2^(scale+11) + coef0*yn1 + coef1*yn2 + 0x400) >> 11, clamped to s16.
 */
static inline uint16_t gw_ax_clamp_s16(int64_t v) {
    if (v > 32767) {
        return 32767;
    }
    if (v < -32768) {
        return (uint16_t)-32768;
    }
    return (uint16_t)v;
}

static int16_t gw_ax_decode_one(gw_ax_vstate *s) {
    uint8_t *aram = gw_aram;
    uint32_t byte_off = s->cur_addr >> 1;

    if (s->format == 0) {
        /* ADPCM */
        if (byte_off >= gw_aram_size) {
            s->active = false;
            s->state = 0;
            return 0;
        }
        if (s->samples_left == 0) {
            uint8_t hdr = aram[byte_off];
            s->pred_scale = hdr;
            s->cur_addr += 2;
            s->samples_left = 14;
            byte_off = s->cur_addr >> 1;
            if (byte_off >= gw_aram_size) {
                s->active = false;
                s->state = 0;
                return 0;
            }
        }
        {
            uint8_t b = aram[byte_off];
            int nibble = (s->cur_addr & 1) ? (b & 0xF) : (b >> 4);
            int predictor = s->pred_scale >> 4;
            int scale = s->pred_scale & 0xF;
            int16_t coef0 = s->coef[predictor][0];
            int16_t coef1 = s->coef[predictor][1];
            int sn = nibble >= 8 ? nibble - 16 : nibble;
            int64_t scaled = (int64_t)sn * ((int64_t)1 << (scale + 11));
            int64_t pred = (int64_t)coef0 * s->yn1 + (int64_t)coef1 * s->yn2;
            int64_t sample = (scaled + pred + 0x400) >> 11;
            int16_t out = (int16_t)gw_ax_clamp_s16(sample);
            s->yn2 = s->yn1;
            s->yn1 = out;
            s->cur_addr += 1;
            s->samples_left -= 1;
            if (s->cur_addr >= s->end_addr) {
                if (s->loop_flag) {
                    s->cur_addr = s->loop_addr;
                    s->pred_scale = s->loop_pred_scale;
                    s->yn1 = s->loop_yn1;
                    s->yn2 = s->loop_yn2;
                    s->samples_left = 14;
                } else {
                    s->active = false;
                    s->state = 0;
                }
            }
            return out;
        }
    } else if (s->format == 0xA) {
        /* 16-bit PCM: one sample per 2 bytes (4 nibbles). Not used by Melee; defensive. */
        if (byte_off + 1 >= gw_aram_size) {
            s->active = false;
            s->state = 0;
            return 0;
        }
        {
            int16_t out = (int16_t)gw_r16(aram + byte_off);
            s->cur_addr += 4;
            if (s->cur_addr >= s->end_addr) {
                s->active = false;
                s->state = 0;
            }
            return out;
        }
    } else {
        /* Unknown format (0x19 PCM8, etc.): not produced by Melee; silence it. */
        s->active = false;
        s->state = 0;
        return 0;
    }
}

/* ---- the mixer ----------------------------------------------------------------------------- */
static int32_t gw_ax_acc_l[GW_AX_FRAME_SAMPLES];
static int32_t gw_ax_acc_r[GW_AX_FRAME_SAMPLES];
static int16_t gw_ax_out[GW_AX_FRAME_SAMPLES * 2];
static int gw_ax_peak;

static void gw_ax_mix_voice(gw_ax_voice *voice, gw_ax_vstate *s) {
    int i;
    (void)voice;
    for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
        s->frac += s->ratio;
        while (s->frac >= 0x10000u) {
            s->last = gw_ax_decode_one(s);
            s->frac -= 0x10000u;
            if (!s->active) {
                break;
            }
        }
        if (!s->active) {
            break;
        }
        s->volume += s->volume_delta;
        if (s->volume < 0) {
            s->volume = 0;
        } else if (s->volume > 32767) {
            s->volume = 32767;
        }
        {
            int32_t sample = s->last;
            int32_t gl = (int32_t)(((int64_t)s->mix_l * s->volume) >> 15);
            int32_t gr = (int32_t)(((int64_t)s->mix_r * s->volume) >> 15);
            gw_ax_acc_l[i] += (int32_t)(((int64_t)sample * gl) >> 15);
            gw_ax_acc_r[i] += (int32_t)(((int64_t)sample * gr) >> 15);
        }
    }
}

static void gw_ax_run_frame(void) {
    int i;
    int active = 0;
    static uint32_t frames_run;
    static bool reported_first;

    if (gw_ax_cb != NULL) {
        gw_ax_cb();
    }

    memset(gw_ax_acc_l, 0, sizeof gw_ax_acc_l);
    memset(gw_ax_acc_r, 0, sizeof gw_ax_acc_r);

    for (i = 0; i < GW_AX_NUM_VOICES; i++) {
        if (gw_ax_in_use[i] && gw_ax_state[i].active && gw_ax_state[i].configured) {
            gw_ax_mix_voice(&gw_ax_voices[i], &gw_ax_state[i]);
            if (gw_ax_state[i].active) {
                ++active;
            }
        }
    }

    /* Track the loudest mixed sample for the heartbeat (proof of non-silent output). */
    for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
        int a = gw_ax_acc_l[i] < 0 ? -gw_ax_acc_l[i] : gw_ax_acc_l[i];
        int b = gw_ax_acc_r[i] < 0 ? -gw_ax_acc_r[i] : gw_ax_acc_r[i];
        if (a > gw_ax_peak) {
            gw_ax_peak = a;
        }
        if (b > gw_ax_peak) {
            gw_ax_peak = b;
        }
    }

    if (active > 0 && !reported_first) {
        reported_first = true;
        gw_log("gw: AX: first audible frame, %d active voice(s), peak=%d", active, gw_ax_peak);
    }
    ++frames_run;
    if ((frames_run % 400u) == 0u) {
        gw_log("gw: AX: %u frames, %d active voice(s), peak=%d", frames_run, active, gw_ax_peak);
        gw_ax_peak = 0;
    }

    /* Write back the game-visible fields the synth reads (big-endian). */
    for (i = 0; i < GW_AX_NUM_VOICES; i++) {
        if (gw_ax_in_use[i]) {
            gw_w16((uint8_t *)&gw_ax_voices[i] + GW_AX_VPB_PB_STATE, gw_ax_state[i].state);
            gw_w32((uint8_t *)&gw_ax_voices[i] + GW_AX_VPB_CURADDR, gw_ax_state[i].cur_addr);
        }
    }

    if (!gw_ax_dev_ok) {
        return;
    }

    /* Apply the AI master volume and clip. */
    {
        float ml = gw_ai_stream_vol_left * (1.0f / 255.0f);
        float mr = gw_ai_stream_vol_right * (1.0f / 255.0f);
        for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
            int l = (int)(gw_ax_acc_l[i] * ml);
            int r = (int)(gw_ax_acc_r[i] * mr);
            if (l > 32767) {
                l = 32767;
            } else if (l < -32768) {
                l = -32768;
            }
            if (r > 32767) {
                r = 32767;
            } else if (r < -32768) {
                r = -32768;
            }
            gw_ax_out[i * 2] = (int16_t)l;
            gw_ax_out[i * 2 + 1] = (int16_t)r;
        }
    }
    gw_ax_ring_push(gw_ax_out, sizeof gw_ax_out);
}

/* ---- frame driver --------------------------------------------------------------------------
 * Called from gw_frame_tick on the game thread. Generates one 5 ms sub-frame per 5 ms of elapsed
 * virtual time, which keeps the average rate pinned to 32 kHz while a slow or fast game frame
 * only changes the number of sub-frames, never the sample rate. */
#define GW_AX_TIMER_CLOCK 40500000ull
#define GW_AX_FRAME_TICKS (GW_AX_TIMER_CLOCK / 200u)  /* 5 ms */
#define GW_AX_STALL_TICKS (GW_AX_TIMER_CLOCK / 10u)   /* 100 ms: resume fresh after a stall */

static uint64_t gw_ax_last_tick;

void gw_ax_frame_tick(void) {
    uint64_t now;
    uint64_t elapsed;
    uint32_t nframes;
    uint32_t f;

    if (gw_ax_cb == NULL) {
        gw_ax_last_tick = gw_time_ticks();
        return;
    }
    if (!gw_ax_dev_attempted) {
        gw_ax_open_device();
    }
    now = gw_time_ticks();
    if (gw_ax_last_tick == 0) {
        gw_ax_last_tick = now;
        return;
    }
    elapsed = now - gw_ax_last_tick;
    if (elapsed > GW_AX_STALL_TICKS) {
        gw_ax_last_tick = now;
        return;
    }
    nframes = (uint32_t)(elapsed / GW_AX_FRAME_TICKS);
    for (f = 0; f < nframes; f++) {
        gw_ax_run_frame();
    }
    gw_ax_last_tick += (uint64_t)nframes * GW_AX_FRAME_TICKS;
}

/* ---- entry points -------------------------------------------------------------------------- */

void gw_AXInit(void) {}

void gw_AXRegisterCallback(void *callback) { gw_ax_cb = (void (*)(void))callback; }

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
    memset(&gw_ax_state[i], 0, sizeof gw_ax_state[i]);
    return v;
}

void gw_AXFreeVoice(void *voice) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        int i = (int)(v - gw_ax_voices);
        gw_ax_in_use[i] = false;
        gw_ax_state[i].active = false;
        gw_ax_state[i].state = 0;
    }
}

void gw_AXSetVoicePriority(void *voice, uint32_t priority) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    v->priority = (int)priority;
}

void gw_AXSetVoiceState(void *voice, uint16_t state) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        s->state = state;
        if (state == 1) {
            /* Start playing. The decoder was already initialised by AXSetVoiceAddr/Adpcm (and, for
             * the stream, AXSetVoiceCurrentAddr), so this just gates mixing on. */
            s->active = s->configured;
        } else {
            s->active = false;
        }
        gw_w16((uint8_t *)v + GW_AX_VPB_PB_STATE, state);
    }
}

void gw_AXSetVoiceMix(void *voice, void *mix) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *m = (const uint8_t *)mix;
        s->mix_l = gw_r16(m + 0x00);
        s->mix_r = gw_r16(m + 0x04);
    }
}

void gw_AXSetVoiceItdOn(void *voice) { (void)voice; }

void gw_AXSetVoiceItdTarget(void *voice, uint16_t left_shift, uint16_t right_shift) {
    (void)voice;
    (void)left_shift;
    (void)right_shift;
}

void gw_AXSetVoiceVe(void *voice, void *ve) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *)ve;
        s->volume = gw_r16(p + 0x00);
        s->volume_delta = (int16_t)gw_r16(p + 0x02);
    }
}

void gw_AXSetVoiceVeDelta(void *voice, int16_t delta) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].volume_delta = delta;
}

void gw_AXSetVoiceAddr(void *voice, void *addr) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *)addr;
        s->loop_flag = gw_r16(p + 0x00);
        s->format = gw_r16(p + 0x02);
        s->loop_addr = gw_r32(p + 0x04);
        s->end_addr = gw_r32(p + 0x08);
        s->cur_addr = gw_r32(p + 0x0C);
    }
}

void gw_AXSetVoiceLoop(void *voice, uint16_t loop) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].loop_flag = loop;
}

void gw_AXSetVoiceLoopAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].loop_addr = addr;
}

void gw_AXSetVoiceEndAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].end_addr = addr;
}

void gw_AXSetVoiceCurrentAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        s->cur_addr = addr;
        s->pred_scale = s->init_pred_scale;
        s->yn1 = s->init_yn1;
        s->yn2 = s->init_yn2;
        s->samples_left = 14;
    }
}

void gw_AXSetVoiceAdpcm(void *voice, void *adpcm) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *)adpcm;
        int i;
        for (i = 0; i < 8; i++) {
            s->coef[i][0] = (int16_t)gw_r16(p + i * 4 + 0);
            s->coef[i][1] = (int16_t)gw_r16(p + i * 4 + 2);
        }
        s->pred_scale = (uint8_t)(gw_r16(p + 0x22) & 0xFF);
        s->yn1 = (int16_t)gw_r16(p + 0x24);
        s->yn2 = (int16_t)gw_r16(p + 0x26);
        s->init_pred_scale = s->pred_scale;
        s->init_yn1 = s->yn1;
        s->init_yn2 = s->yn2;
        s->samples_left = 14;
        s->configured = true;
    }
}

void gw_AXSetVoiceSrc(void *voice, void *src) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        const uint8_t *p = (const uint8_t *)src;
        gw_ax_state[v - gw_ax_voices].ratio = ((uint32_t)gw_r16(p + 0x00) << 16) | gw_r16(p + 0x02);
    }
}

void gw_AXSetVoiceSrcRatio(void *voice, float ratio) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].ratio = (uint32_t)(ratio * 65536.0f);
}

void gw_AXSetVoiceAdpcmLoop(void *voice, void *adpcm_loop) {
    gw_ax_voice *v = (gw_ax_voice *)voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *)adpcm_loop;
        s->loop_pred_scale = (uint8_t)(gw_r16(p + 0x00) & 0xFF);
        s->loop_yn1 = (int16_t)gw_r16(p + 0x02);
        s->loop_yn2 = (int16_t)gw_r16(p + 0x04);
    }
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
