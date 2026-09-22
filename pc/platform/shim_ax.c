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
 *     ratio with linear interpolation, applies ve volume + the nine AXPBMIX targets, and writes
 *     back pb.state, pb.ve.currentVolume and pb.addr.currentAddressHi/Lo (big-endian) because
 *     the synth reads them (synth.c:824, 1213, 1289).
 *   - The two aux buses are real: voices' vAuxA and vAuxB sends accumulate into 160-sample s32
 *     buffers laid out exactly as the DSP's (L, R, S contiguous, see AXAux.c), the registered
 *     aux callback processes each in place, and the result is summed back into the main mix.
 *     Melee configures AUX A = AXFX reverb (std) and AUX B = AXFX delay in lbAudioAx_8002838C.
 *   - Output is a lock-free single-producer/single-consumer ring buffer drained by an SDL3 audio
 *     stream callback. The callback only copies out of the ring; it never touches game memory.
 *     Generation is paced by the ring's fill level, i.e. by the audio device clock, which is the
 *     role the DSP's 5 ms interrupt plays on hardware.
 *
 * DSP addressing: a game "DSP address" is an ARAM byte offset * 2 (see synth.c:1495
 * `HSD_Synth_804D7784 *= 2`), i.e. the DSP addresses ARAM in nibbles. A DSP address d maps to
 * host `gw_aram + d/2`; the low bit of d selects the high/low nibble of that byte.
 *
 * Environment:
 *   MELEE_AUDIO_LATENCY_MS  target ring cushion in ms (default 60, clamped 10..200)
 *   MELEE_AUDIO_DUMP=<path> also write the final mix to a 32 kHz stereo WAV file
 *   MELEE_AUDIO_NOFX=1      bypass the aux effect processors (the aux sends still sum in dry)
 */
#define _CRT_SECURE_NO_WARNINGS

#include "gw.h"
#include "shim_vi.h"
#include "shim_ax.h"

#include <SDL3/SDL_audio.h>
#include <SDL3/SDL_init.h>

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h> /* GetModuleFileNameA: audio.cfg beside the exe */

/* ---- voice pool ---------------------------------------------------------------------------
 * AXAcquireVoice cannot return NULL: the synth's stream path (HSD_Synth_8038B5AC) reads
 * voice->index immediately after acquiring, so NULL faults at NULL+0x18 the moment the game starts
 * the menu music. This is the same 64-voice AXVPB-layout pool as before, unchanged in its
 * allocation/steal semantics. */
#define GW_AX_NUM_VOICES 64
#define GW_AX_FRAME_SAMPLES 160 /* 5 ms at 32 kHz */
#define GW_AX_RATE 32000

/* Mirror of AXVPB (extern/dolphin/include/dolphin/ax.h): index at 0x18, pb at 0x138, 0x1F8 total.
 * `index` is read by game code through a byte-swapped load, so it is written big-endian; the full
 * size is reserved so the game's reads at voice+0x1B2 and voice+0x16E stay inside the
 * allocation. */
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

/* Game-visible offsets into the AXVPB (big-endian game memory). pb sits at 0x138. */
#define GW_AX_VPB_PB_STATE 0x146 /* AXVPB.pb.state             (u16) */
#define GW_AX_VPB_ITD_FLAG 0x16E /* AXVPB.pb.itd.flag          (u16) */
#define GW_AX_VPB_VE_VOL 0x19C   /* AXVPB.pb.ve.currentVolume  (u16) */
#define GW_AX_VPB_CURADDR 0x1B2  /* AXVPB.pb.addr.currentAddressHi/Lo (u32) */

/* ---- mix targets ----------------------------------------------------------------------------
 * The nine AXPBMIX destinations, in the order the mixer accumulates them. Melee only ever drives
 * the six non-surround ones (synth.c HSD_SynthSFXUpdateMix), but carrying all nine costs one
 * predictable branch each and keeps the aux buffers laid out exactly like the DSP's. */
enum {
    GW_MIX_L,
    GW_MIX_R,
    GW_MIX_S,
    GW_MIX_AL,
    GW_MIX_AR,
    GW_MIX_AS,
    GW_MIX_BL,
    GW_MIX_BR,
    GW_MIX_BS,
    GW_MIX_COUNT
};

/* Byte offset of each target's volume inside AXPBMIX; its s16 delta sits at +2. */
static const uint8_t gw_ax_mix_off[GW_MIX_COUNT] = {
    0x00, /* vL     */
    0x04, /* vR     */
    0x1C, /* vS     */
    0x08, /* vAuxAL */
    0x0C, /* vAuxAR */
    0x20, /* vAuxAS */
    0x10, /* vAuxBL */
    0x14, /* vAuxBR */
    0x18, /* vAuxBS */
};

/* ---- per-voice mixer state (host side, never game-visible) -------------------------------- */
#define GW_AX_ITD_MAX 32 /* lbl_8040806C in synth.c tops out at a 0x1F-sample shift */

typedef struct {
    bool active;     /* AXSetVoiceState(1) seen and the voice has not finished */
    bool configured; /* AXSetVoiceAddr + AXSetVoiceAdpcm seen */

    /* ADPCM decoder */
    int16_t coef[8][2];
    uint8_t pred_scale;      /* current frame header: predictor<<4 | scale */
    int16_t yn1, yn2;        /* sample history */
    uint8_t init_pred_scale; /* init state from AXPBADPCM (restored on seek) */
    int16_t init_yn1, init_yn2;
    uint32_t cur_addr;  /* DSP (nibble) address of the next data nibble */
    uint32_t end_addr;  /* last valid nibble, inclusive (DSP address) */
    uint32_t loop_addr; /* restart address (DSP address) */
    uint16_t loop_flag; /* 0 = stop at end, 1 = loop */
    uint16_t format;    /* AXPBADDR.format: 0 = ADPCM, 0xA = PCM16, 0x19 = PCM8 */
    uint8_t loop_pred_scale;
    int16_t loop_yn1, loop_yn2;

    /* resampling: `frac` is the 16.16 phase between s_prev and s_next */
    uint32_t ratio;
    uint32_t frac;
    int16_t s_prev, s_next;
    bool primed; /* s_prev/s_next loaded for the current position */

    /* volume / pan */
    int32_t volume;       /* ve.currentVolume, 0..32767 */
    int16_t volume_delta; /* ve.currentDelta, applied per sample */
    int32_t mix[GW_MIX_COUNT];
    int16_t mix_delta[GW_MIX_COUNT];
    bool mix_ramping; /* any non-zero delta; Melee never sets one, so the hot loop skips it */

    /* inter-aural time difference: a small independent delay on the main L and R sends */
    bool itd_on;
    int16_t itd_hist[GW_AX_ITD_MAX];
    int itd_pos;
    int itd_l, itd_r;   /* current shift, in samples */
    int itd_tl, itd_tr; /* target shift */

    uint16_t state; /* mirrored to pb.state */

    int32_t last_out[3]; /* last main L/R/S contribution, for the de-pop ramp */
} gw_ax_vstate;

static gw_ax_vstate gw_ax_state[GW_AX_NUM_VOICES];

static void (*gw_ax_cb)(void);

/* ---- AI state (written by shim_misc.c) ----------------------------------------------------
 * NOTE: these are deliberately *not* applied as a master gain. On hardware AISetStreamVol governs
 * the AI's own DVD-streaming channel; Melee calls it from HSD_SynthStreamSetVolume with the music
 * volume (HSD_Synth_804D6030). This port plays the stream through ordinary AX voices, and
 * 804D6030 is already folded into every node's ve.currentVolume (synth.c:986 and :1345), so
 * applying it again here would both double-attenuate the music and drag every sound effect down
 * with the music slider. Tracked for logging only. */
uint8_t gw_ai_stream_vol_left = 255;
uint8_t gw_ai_stream_vol_right = 255;
uint32_t gw_ai_dsp_sample_rate = 0;

/* ---- AXFX: the aux effect processors --------------------------------------------------------
 * The SDK's own reverb_std.c / delay.c are present under extern/dolphin but cannot be built
 * through the gwtool pipeline: their inner loops are MWERKS `asm` blocks, which clang's PowerPC
 * front end will not accept. They are reimplemented natively here, faithfully to that source
 * (reverb_std.c HandleReverb and delay.c AXFXDelayCallback), reading their parameters out of the
 * game-memory AXFX_* struct the caller supplies. The aux heaps the game hands AXDriver therefore
 * go unused; these allocate their own. Melee only ever configures reverb-std on AUX A and delay
 * on AUX B (lbaudio_ax.c:2126, :2134), so reverb-hi and chorus stay unimplemented. */

typedef struct {
    float *buf;
    int len; /* elements */
    int in_point, out_point;
    float last_output;
} gw_fx_dl;

typedef struct {
    gw_fx_dl ap[6], c[6];
    float all_pass_coeff;
    float comb_coef[6];
    float lp_lastout[3];
    float level;
    float damping;
    int pre_delay_time;
    float *pre_delay_line[3];
    int pre_delay_pos[3];
} gw_fx_revstd;

typedef struct {
    int32_t *buf[3];
    uint32_t size[3]; /* in 160-sample blocks */
    uint32_t pos[3];
    uint32_t feedback[3];
    uint32_t output[3];
} gw_fx_delay;

/* Game struct field offsets (big-endian). */
#define GW_FX_REVSTD_TEMPDISABLE 0x13C
#define GW_FX_REVSTD_COLORATION 0x140
#define GW_FX_REVSTD_MIX 0x144
#define GW_FX_REVSTD_TIME 0x148
#define GW_FX_REVSTD_DAMPING 0x14C
#define GW_FX_REVSTD_PREDELAY 0x150
#define GW_FX_DELAY_DELAY 0x3C
#define GW_FX_DELAY_FEEDBACK 0x48
#define GW_FX_DELAY_OUTPUT 0x54

#define GW_FX_KIND_REVSTD 1
#define GW_FX_KIND_DELAY 2

/* One instance per aux channel, keyed by the game pointer AXDriverSetupAux passes. */
#define GW_FX_SLOTS 4
typedef struct {
    void *owner; /* the game-memory AXFX_* struct, or NULL for a free slot */
    int kind;
    union {
        gw_fx_revstd rev;
        gw_fx_delay dly;
    } u;
} gw_fx_slot;

static gw_fx_slot gw_fx_slots[GW_FX_SLOTS];
static bool gw_ax_nofx;

static gw_fx_slot *gw_fx_find(void *owner) {
    int i;
    if (owner == NULL) {
        return NULL;
    }
    for (i = 0; i < GW_FX_SLOTS; i++) {
        if (gw_fx_slots[i].owner == owner) {
            return &gw_fx_slots[i];
        }
    }
    return NULL;
}

static gw_fx_slot *gw_fx_claim(void *owner, int kind) {
    gw_fx_slot *s = gw_fx_find(owner);
    int i;
    if (s == NULL) {
        for (i = 0; i < GW_FX_SLOTS; i++) {
            if (gw_fx_slots[i].owner == NULL) {
                s = &gw_fx_slots[i];
                break;
            }
        }
    }
    if (s == NULL) {
        return NULL;
    }
    memset(s, 0, sizeof *s);
    s->owner = owner;
    s->kind = kind;
    return s;
}

/* -- reverb (std) -- */

static bool gw_fx_dl_create(gw_fx_dl *dl, int max_length, int lag) {
    dl->buf = (float *) calloc((size_t) max_length, sizeof(float));
    if (dl->buf == NULL) {
        return false;
    }
    dl->len = max_length;
    dl->last_output = 0.0f;
    dl->in_point = 0;
    /* DLcreate zeroes both cursors, then ReverbSTDCreate re-runs DLsetdelay(lag) against
     * in_point == 0, which lands out_point at (len - lag). */
    dl->out_point = max_length - lag;
    while (dl->out_point < 0) {
        dl->out_point += max_length;
    }
    return true;
}

static void gw_fx_revstd_free(gw_fx_revstd *rv) {
    int i;
    for (i = 0; i < 6; i++) {
        free(rv->ap[i].buf);
        rv->ap[i].buf = NULL;
        free(rv->c[i].buf);
        rv->c[i].buf = NULL;
    }
    for (i = 0; i < 3; i++) {
        free(rv->pre_delay_line[i]);
        rv->pre_delay_line[i] = NULL;
    }
    rv->pre_delay_time = 0;
}

/* reverb_std.c ReverbSTDCreate. */
static bool gw_fx_revstd_create(gw_fx_revstd *rv, float coloration, float time, float mix,
                                float damping, float predelay) {
    static const int lens[4] = { 0x6FD, 0x7CF, 0x1B1, 0x95 };
    int i, k;

    if (coloration < 0.0f || coloration > 1.0f || time < 0.01f || time > 10.0f || mix < 0.0f ||
        mix > 1.0f || damping < 0.0f || damping > 1.0f || predelay < 0.0f || predelay > 0.1f) {
        return false;
    }
    memset(rv, 0, sizeof *rv);
    for (k = 0; k < 3; k++) {
        for (i = 0; i < 2; i++) {
            if (!gw_fx_dl_create(&rv->c[i + k * 2], lens[i] + 2, lens[i])) {
                goto fail;
            }
            rv->comb_coef[i + k * 2] = powf(10.0f, (float) (lens[i] * -3) / (32000.0f * time));
            if (!gw_fx_dl_create(&rv->ap[i + k * 2], lens[i + 2] + 2, lens[i + 2])) {
                goto fail;
            }
        }
        rv->lp_lastout[k] = 0.0f;
    }
    rv->all_pass_coeff = coloration;
    rv->level = mix;
    rv->damping = damping < 0.05f ? 0.05f : damping;
    rv->damping = 1.0f - (0.05f + 0.8f * rv->damping);
    if (predelay != 0.0f) {
        rv->pre_delay_time = (int) (32000.0f * predelay);
        if (rv->pre_delay_time < 2) {
            rv->pre_delay_time = 0; /* below two elements the original's cursor cannot advance */
        }
    }
    if (rv->pre_delay_time != 0) {
        for (i = 0; i < 3; i++) {
            rv->pre_delay_line[i] = (float *) calloc((size_t) rv->pre_delay_time, sizeof(float));
            if (rv->pre_delay_line[i] == NULL) {
                goto fail;
            }
            rv->pre_delay_pos[i] = 0;
        }
    }
    return true;

fail:
    gw_fx_revstd_free(rv);
    return false;
}

/* reverb_std.c HandleReverb: per channel, two feedback combs feed two Schroeder all-passes with a
 * one-pole damping filter between them; the bus's three 160-sample blocks are processed in
 * sequence, each against its own comb/all-pass pair. */
static void gw_fx_revstd_run(gw_fx_revstd *rv, int32_t *bus) {
    const float apc = rv->all_pass_coeff;
    const float damp = rv->damping;
    const float wet = rv->level * 0.6f;
    const float dry = 0.6f - wet;
    int k;

    for (k = 0; k < 3; k++) {
        gw_fx_dl *c0 = &rv->c[k * 2], *c1 = &rv->c[k * 2 + 1];
        gw_fx_dl *a0 = &rv->ap[k * 2], *a1 = &rv->ap[k * 2 + 1];
        const float cc0 = rv->comb_coef[k * 2], cc1 = rv->comb_coef[k * 2 + 1];
        float *pdl = rv->pre_delay_line[k];
        const int pdt = rv->pre_delay_time;
        int pdp = rv->pre_delay_pos[k];
        float lp = rv->lp_lastout[k];
        int32_t *sp = bus + k * GW_AX_FRAME_SAMPLES;
        int n;

        for (n = 0; n < GW_AX_FRAME_SAMPLES; n++) {
            const float in = (float) sp[n];
            float x, y, t, o0, o1;

            if (pdt != 0) {
                /* The original walks a pointer and wraps one element early, so the usable
                 * pre-delay is (pre_delay_time - 1); keep that so the delay matches hardware. */
                x = pdl[pdp];
                pdl[pdp] = in;
                if (++pdp >= pdt - 1) {
                    pdp = 0;
                }
            } else {
                x = in;
            }

            c0->buf[c0->in_point] = cc0 * c0->last_output + x;
            if (++c0->in_point >= c0->len) {
                c0->in_point = 0;
            }
            c1->buf[c1->in_point] = cc1 * c1->last_output + x;
            if (++c1->in_point >= c1->len) {
                c1->in_point = 0;
            }
            o0 = c0->buf[c0->out_point];
            if (++c0->out_point >= c0->len) {
                c0->out_point = 0;
            }
            o1 = c1->buf[c1->out_point];
            if (++c1->out_point >= c1->len) {
                c1->out_point = 0;
            }
            c0->last_output = o0;
            c1->last_output = o1;
            y = o0 + o1;

            t = apc * a0->last_output + y;
            a0->buf[a0->in_point] = t;
            if (++a0->in_point >= a0->len) {
                a0->in_point = 0;
            }
            y = a0->last_output - apc * t;
            a0->last_output = a0->buf[a0->out_point];
            if (++a0->out_point >= a0->len) {
                a0->out_point = 0;
            }

            y = y * 0.3f;
            y = damp * lp + y;
            lp = y;

            t = apc * a1->last_output + y;
            a1->buf[a1->in_point] = t;
            if (++a1->in_point >= a1->len) {
                a1->in_point = 0;
            }
            y = a1->last_output - apc * t;
            a1->last_output = a1->buf[a1->out_point];
            if (++a1->out_point >= a1->len) {
                a1->out_point = 0;
            }

            sp[n] = (int32_t) (wet * y + dry * in);
        }
        rv->lp_lastout[k] = lp;
        rv->pre_delay_pos[k] = pdp;
    }
}

/* -- delay -- */

static void gw_fx_delay_free(gw_fx_delay *d) {
    int i;
    for (i = 0; i < 3; i++) {
        free(d->buf[i]);
        d->buf[i] = NULL;
    }
}

/* delay.c AXFXDelaySettings. */
static bool gw_fx_delay_create(gw_fx_delay *d, const uint32_t ms[3], const uint32_t fb[3],
                               const uint32_t out[3]) {
    int i;
    memset(d, 0, sizeof *d);
    for (i = 0; i < 3; i++) {
        uint32_t m = ms[i] < 5u ? 5u : ms[i];
        d->size[i] = (((m - 5u) << 5) + 0x9Fu) / 160u;
        if (d->size[i] == 0u) {
            d->size[i] = 1u;
        }
        d->pos[i] = 0;
        d->feedback[i] = (fb[i] << 7) / 100u;
        d->output[i] = (out[i] << 7) / 100u;
        d->buf[i] =
            (int32_t *) calloc((size_t) d->size[i] * GW_AX_FRAME_SAMPLES, sizeof(int32_t));
        if (d->buf[i] == NULL) {
            gw_fx_delay_free(d);
            return false;
        }
    }
    return true;
}

/* delay.c AXFXDelayCallback. */
static void gw_fx_delay_run(gw_fx_delay *d, int32_t *bus) {
    int k;
    for (k = 0; k < 3; k++) {
        int32_t *line = d->buf[k] + d->pos[k] * GW_AX_FRAME_SAMPLES;
        int32_t *sp = bus + k * GW_AX_FRAME_SAMPLES;
        const int32_t fb = (int32_t) d->feedback[k];
        const int32_t og = (int32_t) d->output[k];
        int n;
        for (n = 0; n < GW_AX_FRAME_SAMPLES; n++) {
            int32_t v = line[n];
            line[n] = sp[n] + (int32_t) (((int64_t) v * fb) >> 7);
            sp[n] = (int32_t) (((int64_t) v * og) >> 7);
        }
        d->pos[k] = (d->pos[k] + 1u) % d->size[k];
    }
}

/* ---- aux buses ------------------------------------------------------------------------------
 * Laid out exactly like the DSP's (AXAux.c): L at [0], R at [160], S at [320]. */
#define GW_AX_BUS_SAMPLES (GW_AX_FRAME_SAMPLES * 3)

typedef struct {
    void *cb; /* the registered AXFX*Callback, or NULL when the bus is off */
    void *ctx;
    int32_t bus[GW_AX_BUS_SAMPLES];
} gw_ax_aux;

static gw_ax_aux gw_ax_aux_a;
static gw_ax_aux gw_ax_aux_b;

/* Run one aux bus's effect in place. The registered callback can only ever be one of this file's
 * own gw_AXFX*Callback entry points (axdriver.c has no other function to register), so rather
 * than call back through the pointer the bus is dispatched straight to the native processor that
 * owns `ctx`. An unrecognised context means an effect this shim does not implement: the bus is
 * silenced rather than passed through, because leaving it untouched would fold the raw send back
 * into the mix as a second dry copy. */
static void gw_ax_run_aux(gw_ax_aux *aux) {
    gw_fx_slot *s;
    if (aux->cb == NULL) {
        return;
    }
    if (gw_ax_nofx) {
        return;
    }
    s = gw_fx_find(aux->ctx);
    if (s == NULL) {
        memset(aux->bus, 0, sizeof aux->bus);
        return;
    }
    switch (s->kind) {
    case GW_FX_KIND_REVSTD:
        gw_fx_revstd_run(&s->u.rev, aux->bus);
        break;
    case GW_FX_KIND_DELAY:
        gw_fx_delay_run(&s->u.dly, aux->bus);
        break;
    default:
        memset(aux->bus, 0, sizeof aux->bus);
        break;
    }
}

/* ---- output: SDL3 audio + ring buffer ------------------------------------------------------ */
#define GW_AX_FRAME_BYTES (GW_AX_FRAME_SAMPLES * 4) /* stereo s16 */
#define GW_AX_RING_BYTES 65536                      /* 512 ms */
#define GW_AX_MAX_BURST 16                          /* 80 ms of catch-up per video frame */

static uint8_t gw_ax_ring[GW_AX_RING_BYTES];
static volatile uint32_t gw_ax_ring_w; /* bytes produced (monotonic) */
static volatile uint32_t gw_ax_ring_r; /* bytes consumed (monotonic) */

static SDL_AudioStream *gw_ax_stream;
static bool gw_ax_dev_ok;
static bool gw_ax_dev_attempted;
static uint32_t gw_ax_target_frames = 12; /* 60 ms cushion; see MELEE_AUDIO_LATENCY_MS */
static uint32_t gw_ax_underruns;
static uint32_t gw_ax_overruns;

#if defined(_MSC_VER) && !defined(__clang__)
#include <intrin.h>
#define GW_AX_BARRIER() _ReadWriteBarrier()
#else
#define GW_AX_BARRIER() __atomic_thread_fence(__ATOMIC_SEQ_CST)
#endif

static void gw_ax_ring_push(const void *data, uint32_t n) {
    uint32_t w = gw_ax_ring_w;
    uint32_t avail = GW_AX_RING_BYTES - (w - gw_ax_ring_r);
    if (avail < n) {
        /* The producer ran away from the consumer: drop rather than corrupt the cursor
         * invariant. The fill-level pacing below makes this essentially unreachable. */
        ++gw_ax_overruns;
        n = avail;
        if (n == 0) {
            return;
        }
    }
    {
        const uint8_t *src = (const uint8_t *) data;
        uint32_t wpos = w & (GW_AX_RING_BYTES - 1);
        uint32_t first = GW_AX_RING_BYTES - wpos;
        if (first > n) {
            first = n;
        }
        memcpy(gw_ax_ring + wpos, src, first);
        memcpy(gw_ax_ring, src + first, n - first);
    }
    GW_AX_BARRIER(); /* the copies must land before the cursor that publishes them */
    gw_ax_ring_w = w + n;
}

/* SDL audio callback. Runs on SDL's thread; only copies out of the ring (and pads with silence
 * on underrun). Never touches game memory or takes game locks. */
static void SDLCALL gw_ax_sdl_callback(void *userdata, SDL_AudioStream *stream,
                                       int additional_amount, int total_amount) {
    uint8_t buf[8192];
    uint32_t avail, take, done, remain;
    (void) userdata;
    (void) total_amount;
    if (additional_amount <= 0) {
        return;
    }
    avail = gw_ax_ring_w - gw_ax_ring_r;
    GW_AX_BARRIER(); /* read the cursor before the bytes it covers */
    take = (uint32_t) additional_amount;
    if (take > avail) {
        take = avail;
    }
    done = 0;
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
            GW_AX_BARRIER();
            gw_ax_ring_r = r + chunk;
        }
        SDL_PutAudioStreamData(stream, buf, (int) chunk);
        done += chunk;
    }
    remain = (uint32_t) additional_amount - done;
    if (remain != 0) {
        ++gw_ax_underruns;
    }
    while (remain > 0) {
        uint32_t chunk = remain > sizeof buf ? (uint32_t) sizeof buf : remain;
        memset(buf, 0, chunk);
        SDL_PutAudioStreamData(stream, buf, (int) chunk);
        remain -= chunk;
    }
}

/* ---- master volume ---------------------------------------------------------------------------
 * MELEE_VOLUME=0..100 (percent; 0 = silent), else audio.cfg next to the exe ("volume = N"), else
 * 100. Applied as the output stream's gain, so it scales exactly what reaches the speakers (a
 * MELEE_AUDIO_DUMP capture stays at full scale). gw_Audio_SetVolume changes it live and saves
 * audio.cfg, for the settings menu. */
static int gw_ax_volume = -1;

static const char *gw_ax_cfg_path(void) {
    static char buf[MAX_PATH];
    char *slash;
    if (buf[0] == '\0') {
        DWORD n = GetModuleFileNameA(NULL, buf, (DWORD) sizeof buf);
        if (n == 0 || n >= sizeof buf || (slash = strrchr(buf, '\\')) == NULL) {
            strcpy(buf, "audio.cfg");
        } else {
            strcpy(slash + 1, "audio.cfg");
        }
    }
    return buf;
}

static int gw_ax_clamp_volume(int v) { return v < 0 ? 0 : (v > 100 ? 100 : v); }

int gw_Audio_Volume(void) {
    if (gw_ax_volume < 0) {
        const char *env = getenv("MELEE_VOLUME");
        FILE *f;
        gw_ax_volume = 100;
        f = fopen(gw_ax_cfg_path(), "r");
        if (f != NULL) {
            char line[64];
            int v;
            while (fgets(line, sizeof line, f) != NULL) {
                if (sscanf(line, " volume = %d", &v) == 1) {
                    gw_ax_volume = gw_ax_clamp_volume(v);
                }
            }
            fclose(f);
        }
        if (env != NULL && env[0] != '\0') {
            gw_ax_volume = gw_ax_clamp_volume(atoi(env));
        }
    }
    return gw_ax_volume;
}

static void gw_ax_apply_volume(void) {
    if (gw_ax_stream != NULL) {
        SDL_SetAudioStreamGain(gw_ax_stream, (float) gw_Audio_Volume() / 100.0f);
    }
}

void gw_Audio_SetVolume(int percent) {
    FILE *f;
    gw_ax_volume = gw_ax_clamp_volume(percent);
    gw_ax_apply_volume();
    f = fopen(gw_ax_cfg_path(), "w");
    if (f != NULL) {
        fprintf(f, "# melee-pc audio settings (the game rewrites this file)\nvolume = %d\n", gw_ax_volume);
        fclose(f);
    }
    gw_log("gw: AX: master volume %d%%", gw_ax_volume);
}

/* ---- optional WAV capture (MELEE_AUDIO_DUMP) ------------------------------------------------ */
static FILE *gw_ax_wav;
static uint32_t gw_ax_wav_bytes;

static void gw_ax_wav_header(FILE *f, uint32_t data_bytes) {
    uint8_t h[44];
    uint32_t riff = 36 + data_bytes;
    uint32_t rate = GW_AX_RATE;
    uint32_t byte_rate = GW_AX_RATE * 4;
    memcpy(h, "RIFF", 4);
    memcpy(h + 4, &riff, 4);
    memcpy(h + 8, "WAVEfmt ", 8);
    h[16] = 16;
    h[17] = h[18] = h[19] = 0;
    h[20] = 1; /* PCM */
    h[21] = 0;
    h[22] = 2; /* stereo */
    h[23] = 0;
    memcpy(h + 24, &rate, 4);
    memcpy(h + 28, &byte_rate, 4);
    h[32] = 4; /* block align */
    h[33] = 0;
    h[34] = 16; /* bits per sample */
    h[35] = 0;
    memcpy(h + 36, "data", 4);
    memcpy(h + 40, &data_bytes, 4);
    fwrite(h, 1, sizeof h, f);
}

static void gw_ax_wav_write(const void *data, uint32_t n) {
    if (gw_ax_wav == NULL) {
        return;
    }
    fwrite(data, 1, n, gw_ax_wav);
    gw_ax_wav_bytes += n;
    if ((gw_ax_wav_bytes % (GW_AX_FRAME_BYTES * 200u)) == 0u) {
        /* Refresh the sizes about once a second so the file stays playable after a crash. */
        long pos = ftell(gw_ax_wav);
        fseek(gw_ax_wav, 0, SEEK_SET);
        gw_ax_wav_header(gw_ax_wav, gw_ax_wav_bytes);
        fseek(gw_ax_wav, pos, SEEK_SET);
        fflush(gw_ax_wav);
    }
}

static void gw_ax_open_device(void) {
    SDL_AudioSpec spec;
    const char *env;

    if (gw_ax_dev_attempted) {
        return;
    }
    gw_ax_dev_attempted = true;

    env = getenv("MELEE_AUDIO_LATENCY_MS");
    if (env != NULL) {
        int ms = atoi(env);
        if (ms < 10) {
            ms = 10;
        } else if (ms > 200) {
            ms = 200;
        }
        gw_ax_target_frames = (uint32_t) (ms / 5);
        if (gw_ax_target_frames < 2) {
            gw_ax_target_frames = 2;
        }
    }
    gw_ax_nofx = getenv("MELEE_AUDIO_NOFX") != NULL;

    env = getenv("MELEE_AUDIO_DUMP");
    if (env != NULL && *env != '\0') {
        gw_ax_wav = fopen(env, "wb");
        if (gw_ax_wav != NULL) {
            gw_ax_wav_header(gw_ax_wav, 0);
            gw_log("gw: AX: capturing mix to %s", env);
        } else {
            gw_log("gw: AX: cannot open %s for capture", env);
        }
    }

    if (!SDL_InitSubSystem(SDL_INIT_AUDIO)) {
        gw_log("gw: AX: SDL audio init failed: %s", SDL_GetError());
        return;
    }
    spec.format = SDL_AUDIO_S16;
    spec.channels = 2;
    spec.freq = GW_AX_RATE;
    gw_ax_stream =
        SDL_OpenAudioDeviceStream(SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK, &spec, gw_ax_sdl_callback,
                                  NULL);
    if (gw_ax_stream == NULL) {
        gw_log("gw: AX: SDL_OpenAudioDeviceStream failed: %s", SDL_GetError());
        return;
    }
    gw_ax_apply_volume();
    gw_log("gw: AX: master volume %d%%", gw_Audio_Volume());
    SDL_ResumeAudioStreamDevice(gw_ax_stream);
    gw_ax_dev_ok = true;
    gw_log("gw: AX: audio device open (32 kHz stereo s16, %u ms target latency)",
           gw_ax_target_frames * 5u);
}

/* ---- ADPCM decode --------------------------------------------------------------------------
 * Nintendo DSP-ADPCM: 8-byte frames, each a predictor/scale header byte + 7 bytes (14 nibbles)
 * of 4-bit deltas. The DSP addresses ARAM in nibbles, so a frame spans 16 nibbles of which the
 * first two are the header: the header is read exactly when the nibble address is 16-aligned,
 * which is how the hardware finds it and what keeps the decoder in sync after an arbitrary seek
 * (AXSetVoiceCurrentAddr, used by the stream every ring block). Each nibble n decodes as
 *   sample = (n * 2^(scale+11) + coef0*yn1 + coef1*yn2 + 0x400) >> 11, clamped to s16.
 */
static inline int16_t gw_ax_clamp_s16(int64_t v) {
    if (v > 32767) {
        return 32767;
    }
    if (v < -32768) {
        return (int16_t) -32768;
    }
    return (int16_t) v;
}

/* The end-address test.
 *
 * It has to be an (almost) exact match, not `cur_addr >= end_addr`. The .hps stream plays a
 * three-block ring in ARAM and HSD_Synth_8038ADD0 (synth.c:1289) only moves pb.addr.endAddress
 * onto a block *after* it has seen the voice's current address arrive there, while the loop
 * address already points at the next block. So for one AX frame after every block transition the
 * end address is a whole block *behind* the current address, and a `>=` test fires on every
 * sample: the voice re-loops each sample for up to 5 ms, which is the periodic tick that was
 * audible in the menu and stage music (blocks are ~3.6 s, and two transitions in three go
 * forwards). An exact match cannot fire on a stale end address, which is how the hardware
 * decoder behaves and why the game's scheme works at all.
 *
 * A small window rather than plain equality so that an end address landing on a frame-header
 * nibble - which the cursor skips over - still terminates instead of running away; 16 nibbles is
 * one ADPCM frame. The subtraction is unsigned, so an end address behind the cursor yields a huge
 * difference and does not match. */
static inline bool gw_ax_at_end(const gw_ax_vstate *s, uint32_t next_addr) {
    return (uint32_t) (next_addr - s->end_addr) < 16u;
}

/* Advance past the end of the sample: restart at the loop point, or stop the voice. */
static void gw_ax_hit_end(gw_ax_vstate *s) {
    if (s->loop_flag) {
        s->cur_addr = s->loop_addr;
        s->pred_scale = s->loop_pred_scale;
        s->yn1 = s->loop_yn1;
        s->yn2 = s->loop_yn2;
    } else {
        s->active = false;
        s->state = 0;
    }
}

static int16_t gw_ax_decode_one(gw_ax_vstate *s) {
    uint8_t *aram = gw_aram;
    uint32_t byte_off;

    if (s->format == 0) {
        /* ADPCM */
        if ((s->cur_addr & 0xF) == 0) {
            byte_off = s->cur_addr >> 1;
            if (byte_off >= gw_aram_size) {
                s->active = false;
                s->state = 0;
                return 0;
            }
            s->pred_scale = aram[byte_off];
            s->cur_addr += 2;
        }
        byte_off = s->cur_addr >> 1;
        if (byte_off >= gw_aram_size) {
            s->active = false;
            s->state = 0;
            return 0;
        }
        {
            uint8_t b = aram[byte_off];
            int nibble = (s->cur_addr & 1) ? (b & 0xF) : (b >> 4);
            int predictor = s->pred_scale >> 4;
            int scale = s->pred_scale & 0xF;
            int16_t coef0 = s->coef[predictor][0];
            int16_t coef1 = s->coef[predictor][1];
            int sn = nibble >= 8 ? nibble - 16 : nibble;
            int64_t scaled = (int64_t) sn * ((int64_t) 1 << (scale + 11));
            int64_t pred = (int64_t) coef0 * s->yn1 + (int64_t) coef1 * s->yn2;
            int16_t out = gw_ax_clamp_s16((scaled + pred + 0x400) >> 11);
            s->yn2 = s->yn1;
            s->yn1 = out;
            if (gw_ax_at_end(s, s->cur_addr)) {
                gw_ax_hit_end(s);
            } else {
                s->cur_addr += 1;
            }
            return out;
        }
    } else if (s->format == 0xA) {
        /* 16-bit PCM: one sample per 2 bytes, i.e. 4 nibbles. */
        byte_off = s->cur_addr >> 1;
        if (byte_off + 1 >= gw_aram_size) {
            s->active = false;
            s->state = 0;
            return 0;
        }
        {
            int16_t out = (int16_t) gw_r16(aram + byte_off);
            if (gw_ax_at_end(s, s->cur_addr + 3)) {
                gw_ax_hit_end(s);
            } else {
                s->cur_addr += 4;
            }
            return out;
        }
    } else if (s->format == 0x19) {
        /* 8-bit PCM: one sample per byte, i.e. 2 nibbles. */
        byte_off = s->cur_addr >> 1;
        if (byte_off >= gw_aram_size) {
            s->active = false;
            s->state = 0;
            return 0;
        }
        {
            int16_t out = (int16_t) ((int8_t) aram[byte_off] << 8);
            if (gw_ax_at_end(s, s->cur_addr + 1)) {
                gw_ax_hit_end(s);
            } else {
                s->cur_addr += 2;
            }
            return out;
        }
    }
    /* Unknown format: silence the voice rather than walk ARAM blindly. */
    s->active = false;
    s->state = 0;
    return 0;
}

/* ---- the mixer ----------------------------------------------------------------------------- */
static int32_t gw_ax_acc[3][GW_AX_FRAME_SAMPLES]; /* main L, R, S */
static int16_t gw_ax_out[GW_AX_FRAME_SAMPLES * 2];
static int gw_ax_peak;

/* De-pop: when a voice stops mid-waveform its contribution would step to zero in one sample,
 * which is an audible click. AX carries the last value in AXPBDPOP and ramps it out; the same is
 * done here with one accumulator per main channel, ramped to zero across the next sub-frame. */
static int32_t gw_ax_dpop[3];
static int32_t gw_ax_dpop_step[3];

static void gw_ax_depop_voice(gw_ax_vstate *s) {
    int c;
    for (c = 0; c < 3; c++) {
        int32_t v = s->last_out[c];
        s->last_out[c] = 0;
        if (v == 0) {
            continue;
        }
        gw_ax_dpop[c] += v;
        gw_ax_dpop_step[c] = gw_ax_dpop[c] / GW_AX_FRAME_SAMPLES;
        if (gw_ax_dpop_step[c] == 0) {
            gw_ax_dpop_step[c] = gw_ax_dpop[c] > 0 ? 1 : -1;
        }
    }
}

/* Stop a voice from outside the mixer (state change, free, steal). */
static void gw_ax_stop_voice(gw_ax_vstate *s) {
    if (s->active) {
        s->active = false;
        gw_ax_depop_voice(s);
    }
}

static void gw_ax_mix_voice(gw_ax_vstate *s) {
    int i, t;
    bool ended = false;

    /* Fill the interpolation window at the current position. */
    if (!s->primed) {
        s->s_prev = gw_ax_decode_one(s);
        s->s_next = s->active ? gw_ax_decode_one(s) : 0;
        s->frac = 0;
        s->primed = true;
        if (!s->active) {
            ended = true;
        }
    }

    for (i = 0; i < GW_AX_FRAME_SAMPLES && !ended; i++) {
        int32_t sample, v, vl, vr;

        /* Linear interpolation between the two source samples straddling the phase. AX's SRC
         * does the same (AX_SRC_TYPE_LINEAR); holding the last sample instead aliases badly on
         * every pitched voice, which is most of them. */
        sample = (int32_t) s->s_prev +
                 (int32_t) (((int64_t) (s->s_next - s->s_prev) * (int64_t) s->frac) >> 16);

        s->volume += s->volume_delta;
        if (s->volume < 0) {
            s->volume = 0;
        } else if (s->volume > 32767) {
            s->volume = 32767;
        }

        if (s->mix_ramping) {
            for (t = 0; t < GW_MIX_COUNT; t++) {
                int32_t m = s->mix[t] + s->mix_delta[t];
                if (m < 0) {
                    m = 0;
                } else if (m > 0x8000) {
                    m = 0x8000;
                }
                s->mix[t] = m;
            }
        }

        v = (sample * s->volume) >> 15;

        /* ITD delays the main L and R sends independently by up to 31 samples, which is the
         * stereo-placement cue synth.c drives through AXSetVoiceItdTarget. */
        s->itd_hist[s->itd_pos] = (int16_t) v;
        if (s->itd_on) {
            vl = s->itd_hist[(s->itd_pos - s->itd_l) & (GW_AX_ITD_MAX - 1)];
            vr = s->itd_hist[(s->itd_pos - s->itd_r) & (GW_AX_ITD_MAX - 1)];
        } else {
            vl = v;
            vr = v;
        }
        s->itd_pos = (s->itd_pos + 1) & (GW_AX_ITD_MAX - 1);

        s->last_out[0] = (vl * s->mix[GW_MIX_L]) >> 15;
        s->last_out[1] = (vr * s->mix[GW_MIX_R]) >> 15;
        s->last_out[2] = (v * s->mix[GW_MIX_S]) >> 15;
        gw_ax_acc[0][i] += s->last_out[0];
        gw_ax_acc[1][i] += s->last_out[1];
        if (s->mix[GW_MIX_S]) {
            gw_ax_acc[2][i] += s->last_out[2];
        }
        if (s->mix[GW_MIX_AL]) {
            gw_ax_aux_a.bus[i] += (v * s->mix[GW_MIX_AL]) >> 15;
        }
        if (s->mix[GW_MIX_AR]) {
            gw_ax_aux_a.bus[GW_AX_FRAME_SAMPLES + i] += (v * s->mix[GW_MIX_AR]) >> 15;
        }
        if (s->mix[GW_MIX_AS]) {
            gw_ax_aux_a.bus[GW_AX_FRAME_SAMPLES * 2 + i] += (v * s->mix[GW_MIX_AS]) >> 15;
        }
        if (s->mix[GW_MIX_BL]) {
            gw_ax_aux_b.bus[i] += (v * s->mix[GW_MIX_BL]) >> 15;
        }
        if (s->mix[GW_MIX_BR]) {
            gw_ax_aux_b.bus[GW_AX_FRAME_SAMPLES + i] += (v * s->mix[GW_MIX_BR]) >> 15;
        }
        if (s->mix[GW_MIX_BS]) {
            gw_ax_aux_b.bus[GW_AX_FRAME_SAMPLES * 2 + i] += (v * s->mix[GW_MIX_BS]) >> 15;
        }

        /* Advance the source phase for the next output sample. */
        s->frac += s->ratio;
        while (s->frac >= 0x10000u) {
            s->s_prev = s->s_next;
            s->s_next = gw_ax_decode_one(s);
            s->frac -= 0x10000u;
            if (!s->active) {
                ended = true;
                break;
            }
        }
    }

    if (ended) {
        gw_ax_depop_voice(s);
        s->primed = false;
    }

    /* Step the ITD shifts one sample per sub-frame towards their targets; AX interpolates the
     * same way, and a jump would be audible as a click. */
    if (s->itd_l < s->itd_tl) {
        ++s->itd_l;
    } else if (s->itd_l > s->itd_tl) {
        --s->itd_l;
    }
    if (s->itd_r < s->itd_tr) {
        ++s->itd_r;
    } else if (s->itd_r > s->itd_tr) {
        --s->itd_r;
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

    memset(gw_ax_acc, 0, sizeof gw_ax_acc);
    memset(gw_ax_aux_a.bus, 0, sizeof gw_ax_aux_a.bus);
    memset(gw_ax_aux_b.bus, 0, sizeof gw_ax_aux_b.bus);

    for (i = 0; i < GW_AX_NUM_VOICES; i++) {
        if (gw_ax_in_use[i] && gw_ax_state[i].active && gw_ax_state[i].configured) {
            gw_ax_mix_voice(&gw_ax_state[i]);
            if (gw_ax_state[i].active) {
                ++active;
            }
        }
    }

    /* Ramp out any voice that stopped this sub-frame. */
    for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
        int c;
        for (c = 0; c < 3; c++) {
            if (gw_ax_dpop[c] == 0) {
                continue;
            }
            gw_ax_acc[c][i] += gw_ax_dpop[c];
            gw_ax_dpop[c] -= gw_ax_dpop_step[c];
            if ((gw_ax_dpop_step[c] > 0) == (gw_ax_dpop[c] < 0)) {
                gw_ax_dpop[c] = 0; /* crossed zero */
            }
        }
    }

    /* Run the aux effects in place, then fold the buses back into the main mix. The surround bus
     * has no output path here, so it is dropped as the DSP drops it in stereo mode. */
    gw_ax_run_aux(&gw_ax_aux_a);
    gw_ax_run_aux(&gw_ax_aux_b);
    if (gw_ax_aux_a.cb != NULL) {
        for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
            gw_ax_acc[0][i] += gw_ax_aux_a.bus[i];
            gw_ax_acc[1][i] += gw_ax_aux_a.bus[GW_AX_FRAME_SAMPLES + i];
        }
    }
    if (gw_ax_aux_b.cb != NULL) {
        for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
            gw_ax_acc[0][i] += gw_ax_aux_b.bus[i];
            gw_ax_acc[1][i] += gw_ax_aux_b.bus[GW_AX_FRAME_SAMPLES + i];
        }
    }

    /* Track the loudest mixed sample for the heartbeat (proof of non-silent output). */
    for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
        int a = gw_ax_acc[0][i] < 0 ? -gw_ax_acc[0][i] : gw_ax_acc[0][i];
        int b = gw_ax_acc[1][i] < 0 ? -gw_ax_acc[1][i] : gw_ax_acc[1][i];
        if (a > gw_ax_peak) {
            gw_ax_peak = a;
        }
        if (b > gw_ax_peak) {
            gw_ax_peak = b;
        }
    }

    /* Write back the game-visible fields the synth reads (big-endian). */
    for (i = 0; i < GW_AX_NUM_VOICES; i++) {
        if (gw_ax_in_use[i]) {
            uint8_t *v = (uint8_t *) &gw_ax_voices[i];
            gw_w16(v + GW_AX_VPB_PB_STATE, gw_ax_state[i].state);
            gw_w16(v + GW_AX_VPB_VE_VOL, (uint16_t) gw_ax_state[i].volume);
            gw_w32(v + GW_AX_VPB_CURADDR, gw_ax_state[i].cur_addr);
        }
    }

    if (active > 0 && !reported_first) {
        reported_first = true;
        gw_log("gw: AX: first audible frame, %d active voice(s), peak=%d", active, gw_ax_peak);
    }
    ++frames_run;
    if ((frames_run % 2400u) == 0u) { /* every 12 s */
        gw_log("gw: AX: %u frames, %d active voice(s), peak=%d, underruns=%u overruns=%u",
               frames_run, active, gw_ax_peak, gw_ax_underruns, gw_ax_overruns);
        gw_ax_peak = 0;
    }

    /* Clip to s16 and hand the sub-frame to the output ring. */
    for (i = 0; i < GW_AX_FRAME_SAMPLES; i++) {
        int32_t l = gw_ax_acc[0][i];
        int32_t r = gw_ax_acc[1][i];
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
        gw_ax_out[i * 2] = (int16_t) l;
        gw_ax_out[i * 2 + 1] = (int16_t) r;
    }
    gw_ax_wav_write(gw_ax_out, sizeof gw_ax_out);
    if (gw_ax_dev_ok) {
        gw_ax_ring_push(gw_ax_out, sizeof gw_ax_out);
    }
}

/* ---- frame driver --------------------------------------------------------------------------
 * Called from gw_frame_tick on the game thread. The number of 5 ms sub-frames to generate is
 * taken from how much audio the output ring still holds, i.e. from the audio device's own clock:
 * that is the clock the DSP's 5 ms interrupt runs on, it self-corrects against every source of
 * drift and jitter, and it keeps a fixed cushion in front of the SDL callback. The previous
 * wall-clock scheme kept no cushion at all, so ordinary frame jitter emptied the ring and the
 * callback padded silence — a continuous crackle.
 *
 * Without a device there is no such clock, so the wall clock drives the synth callback instead
 * (the game's audio state machine has to keep running either way). */
#define GW_AX_TIMER_CLOCK 40500000ull
#define GW_AX_FRAME_TICKS (GW_AX_TIMER_CLOCK / 200u) /* 5 ms */
#define GW_AX_STALL_TICKS (GW_AX_TIMER_CLOCK / 10u)  /* 100 ms: resume fresh after a stall */

static uint64_t gw_ax_last_tick;

void gw_ax_frame_tick(void) {
    uint32_t nframes;
    uint32_t f;

    if (gw_ax_cb == NULL) {
        gw_ax_last_tick = gw_time_ticks();
        return;
    }
    if (!gw_ax_dev_attempted) {
        gw_ax_open_device();
    }

    if (gw_ax_dev_ok) {
        uint32_t buffered = (gw_ax_ring_w - gw_ax_ring_r) / GW_AX_FRAME_BYTES;
        nframes = buffered >= gw_ax_target_frames ? 0u : gw_ax_target_frames - buffered;
    } else {
        uint64_t now = gw_time_ticks();
        uint64_t elapsed;
        if (gw_ax_last_tick == 0) {
            gw_ax_last_tick = now;
            return;
        }
        elapsed = now - gw_ax_last_tick;
        if (elapsed > GW_AX_STALL_TICKS) {
            gw_ax_last_tick = now;
            return;
        }
        nframes = (uint32_t) (elapsed / GW_AX_FRAME_TICKS);
        gw_ax_last_tick += (uint64_t) nframes * GW_AX_FRAME_TICKS;
    }

    if (nframes > GW_AX_MAX_BURST) {
        nframes = GW_AX_MAX_BURST;
    }
    for (f = 0; f < nframes; f++) {
        gw_ax_run_frame();
    }
}

/* ---- entry points -------------------------------------------------------------------------- */

void gw_AXInit(void) {}

void gw_AXRegisterCallback(void *callback) { gw_ax_cb = (void (*)(void)) callback; }

void gw_AXRegisterAuxACallback(void *callback, void *context) {
    gw_ax_aux_a.cb = callback;
    gw_ax_aux_a.ctx = context;
}

void gw_AXRegisterAuxBCallback(void *callback, void *context) {
    gw_ax_aux_b.cb = callback;
    gw_ax_aux_b.ctx = context;
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
            if (gw_ax_in_use[i] &&
                (best < 0 || (int32_t) gw_r32(&gw_ax_voices[i].priority) <
                                 (int32_t) gw_r32(&gw_ax_voices[best].priority))) {
                best = i;
            }
        }
        if (best < 0) {
            return NULL;
        }
        v = &gw_ax_voices[best];
        gw_ax_stop_voice(&gw_ax_state[best]);
        if (gw_rptr(&v->callback) != NULL) {
            ((void (*)(void *)) gw_rptr(&v->callback))(v);
        }
    }

    i = (int) (v - gw_ax_voices);
    gw_w32(&v->priority, priority);
    gw_wptr(&v->callback, (const void *) callback);
    gw_w32(&v->userContext, user_context);
    gw_w32(&v->index, (uint32_t) i);
    gw_ax_in_use[i] = true;
    memset(&gw_ax_state[i], 0, sizeof gw_ax_state[i]);
    gw_w16((uint8_t *) v + GW_AX_VPB_ITD_FLAG, 0);
    return v;
}

void gw_AXFreeVoice(void *voice) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        int i = (int) (v - gw_ax_voices);
        gw_ax_stop_voice(&gw_ax_state[i]);
        gw_ax_in_use[i] = false;
        gw_ax_state[i].state = 0;
    }
}

void gw_AXSetVoicePriority(void *voice, uint32_t priority) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_w32(&v->priority, priority);
}

void gw_AXSetVoiceState(void *voice, uint16_t state) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        s->state = state;
        if (state == 1) {
            /* Start playing. The decoder was already initialised by AXSetVoiceAddr/Adpcm (and,
             * for the stream, AXSetVoiceCurrentAddr), so this just gates mixing on. */
            s->active = s->configured;
        } else {
            gw_ax_stop_voice(s);
        }
        gw_w16((uint8_t *) v + GW_AX_VPB_PB_STATE, state);
    }
}

void gw_AXSetVoiceMix(void *voice, void *mix) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *m = (const uint8_t *) mix;
        bool ramping = false;
        int t;
        for (t = 0; t < GW_MIX_COUNT; t++) {
            s->mix[t] = gw_r16(m + gw_ax_mix_off[t]);
            s->mix_delta[t] = (int16_t) gw_r16(m + gw_ax_mix_off[t] + 2);
            if (s->mix_delta[t] != 0) {
                ramping = true;
            }
        }
        s->mix_ramping = ramping;
    }
}

void gw_AXSetVoiceItdOn(void *voice) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].itd_on = true;
    /* synth.c HSD_SynthSFXUpdateMix tests pb.itd.flag to decide whether to write the shifts
     * directly or go through AXSetVoiceItdTarget; AX sets it here, so mirror that. It then
     * writes pb.itd.shiftL/shiftR itself, which the mixer picks up as the initial value. */
    gw_w16((uint8_t *) v + GW_AX_VPB_ITD_FLAG, 1);
}

void gw_AXSetVoiceItdTarget(void *voice, uint16_t left_shift, uint16_t right_shift) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        s->itd_tl = left_shift & (GW_AX_ITD_MAX - 1);
        s->itd_tr = right_shift & (GW_AX_ITD_MAX - 1);
    }
}

void gw_AXSetVoiceVe(void *voice, void *ve) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *) ve;
        s->volume = gw_r16(p + 0x00);
        s->volume_delta = (int16_t) gw_r16(p + 0x02);
    }
}

void gw_AXSetVoiceVeDelta(void *voice, int16_t delta) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].volume_delta = delta;
}

void gw_AXSetVoiceAddr(void *voice, void *addr) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *) addr;
        s->loop_flag = gw_r16(p + 0x00);
        s->format = gw_r16(p + 0x02);
        s->loop_addr = gw_r32(p + 0x04);
        s->end_addr = gw_r32(p + 0x08);
        s->cur_addr = gw_r32(p + 0x0C);
        s->primed = false;
        s->frac = 0;
    }
}

void gw_AXSetVoiceLoop(void *voice, uint16_t loop) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].loop_flag = loop;
}

void gw_AXSetVoiceLoopAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].loop_addr = addr;
}

void gw_AXSetVoiceEndAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    gw_ax_state[v - gw_ax_voices].end_addr = addr;
}

void gw_AXSetVoiceCurrentAddr(void *voice, uint32_t addr) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        s->cur_addr = addr;
        s->pred_scale = s->init_pred_scale;
        s->yn1 = s->init_yn1;
        s->yn2 = s->init_yn2;
        s->primed = false;
        s->frac = 0;
    }
}

void gw_AXSetVoiceAdpcm(void *voice, void *adpcm) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *) adpcm;
        int i;
        for (i = 0; i < 8; i++) {
            s->coef[i][0] = (int16_t) gw_r16(p + i * 4 + 0);
            s->coef[i][1] = (int16_t) gw_r16(p + i * 4 + 2);
        }
        s->pred_scale = (uint8_t) (gw_r16(p + 0x22) & 0xFF);
        s->yn1 = (int16_t) gw_r16(p + 0x24);
        s->yn2 = (int16_t) gw_r16(p + 0x26);
        s->init_pred_scale = s->pred_scale;
        s->init_yn1 = s->yn1;
        s->init_yn2 = s->yn2;
        s->primed = false;
        s->configured = true;
    }
}

void gw_AXSetVoiceSrc(void *voice, void *src) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        const uint8_t *p = (const uint8_t *) src;
        gw_ax_state[v - gw_ax_voices].ratio = ((uint32_t) gw_r16(p + 0x00) << 16) | gw_r16(p + 0x02);
    }
}

void gw_AXSetVoiceSrcRatio(void *voice, float ratio) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    if (ratio < 0.0f) {
        ratio = 0.0f;
    }
    gw_ax_state[v - gw_ax_voices].ratio = (uint32_t) (ratio * 65536.0f);
}

void gw_AXSetVoiceAdpcmLoop(void *voice, void *adpcm_loop) {
    gw_ax_voice *v = (gw_ax_voice *) voice;
    if (v < &gw_ax_voices[0] || v >= &gw_ax_voices[GW_AX_NUM_VOICES]) {
        return;
    }
    {
        gw_ax_vstate *s = &gw_ax_state[v - gw_ax_voices];
        const uint8_t *p = (const uint8_t *) adpcm_loop;
        s->loop_pred_scale = (uint8_t) (gw_r16(p + 0x00) & 0xFF);
        s->loop_yn1 = (int16_t) gw_r16(p + 0x02);
        s->loop_yn2 = (int16_t) gw_r16(p + 0x04);
    }
}

/* ---- AXFX entry points ----------------------------------------------------------------------
 * axdriver.c tests each Init for `== 1` before registering the matching callback, so an effect
 * that fails to build must return 0 and stay unregistered rather than leave a live aux send
 * feeding a processor that is not there. */

void gw_AXFXSetHooks(void *alloc_hook, void *free_hook) {
    /* The AXFX processors here allocate natively rather than out of the game's aux heap. */
    (void) alloc_hook;
    (void) free_hook;
}

int gw_AXFXReverbHiInit(void *reverb) {
    (void) reverb;
    gw_log("gw: AX: AXFX reverb-hi is not implemented (Melee does not use it); aux left off");
    return 0;
}

int gw_AXFXReverbHiShutdown(void *reverb) {
    (void) reverb;
    return 1;
}

void gw_AXFXReverbHiCallback(void *buffer_update, void *reverb) {
    (void) buffer_update;
    (void) reverb;
}

int gw_AXFXReverbStdInit(void *reverb) {
    const uint8_t *p = (const uint8_t *) reverb;
    gw_fx_slot *s;
    float coloration, mix, time, damping, predelay;

    if (p == NULL) {
        return 0;
    }
    coloration = gw_rf32(p + GW_FX_REVSTD_COLORATION);
    mix = gw_rf32(p + GW_FX_REVSTD_MIX);
    time = gw_rf32(p + GW_FX_REVSTD_TIME);
    damping = gw_rf32(p + GW_FX_REVSTD_DAMPING);
    predelay = gw_rf32(p + GW_FX_REVSTD_PREDELAY);

    s = gw_fx_claim(reverb, GW_FX_KIND_REVSTD);
    if (s == NULL) {
        gw_log("gw: AX: no AXFX slot free for reverb-std");
        return 0;
    }
    if (!gw_fx_revstd_create(&s->u.rev, coloration, time, mix, damping, predelay)) {
        gw_log("gw: AX: AXFX reverb-std init failed (col=%.3f time=%.3f mix=%.3f damp=%.3f "
               "pre=%.4f)",
               (double) coloration, (double) time, (double) mix, (double) damping,
               (double) predelay);
        s->owner = NULL;
        return 0;
    }
    gw_log("gw: AX: AXFX reverb-std ready (col=%.3f time=%.3f mix=%.3f damp=%.3f pre=%.4f)",
           (double) coloration, (double) time, (double) mix, (double) damping, (double) predelay);
    return 1;
}

int gw_AXFXReverbStdShutdown(void *reverb) {
    gw_fx_slot *s = gw_fx_find(reverb);
    if (s != NULL && s->kind == GW_FX_KIND_REVSTD) {
        gw_fx_revstd_free(&s->u.rev);
        s->owner = NULL;
    }
    return 1;
}

void gw_AXFXReverbStdCallback(void *buffer_update, void *reverb) {
    /* Unused: gw_ax_run_frame dispatches the aux buses directly (see gw_ax_run_aux). Kept
     * because axdriver.c takes this symbol's address to register it. */
    (void) buffer_update;
    (void) reverb;
}

int gw_AXFXChorusInit(void *chorus) {
    (void) chorus;
    gw_log("gw: AX: AXFX chorus is not implemented (Melee does not use it); aux left off");
    return 0;
}

int gw_AXFXChorusShutdown(void *chorus) {
    (void) chorus;
    return 1;
}

void gw_AXFXChorusCallback(void *buffer_update, void *chorus) {
    (void) buffer_update;
    (void) chorus;
}

int gw_AXFXDelayInit(void *delay) {
    const uint8_t *p = (const uint8_t *) delay;
    gw_fx_slot *s;
    uint32_t ms[3], fb[3], out[3];
    int i;

    if (p == NULL) {
        return 0;
    }
    for (i = 0; i < 3; i++) {
        ms[i] = gw_r32(p + GW_FX_DELAY_DELAY + i * 4);
        fb[i] = gw_r32(p + GW_FX_DELAY_FEEDBACK + i * 4);
        out[i] = gw_r32(p + GW_FX_DELAY_OUTPUT + i * 4);
    }

    s = gw_fx_claim(delay, GW_FX_KIND_DELAY);
    if (s == NULL) {
        gw_log("gw: AX: no AXFX slot free for delay");
        return 0;
    }
    if (!gw_fx_delay_create(&s->u.dly, ms, fb, out)) {
        gw_log("gw: AX: AXFX delay init failed");
        s->owner = NULL;
        return 0;
    }
    gw_log("gw: AX: AXFX delay ready (%u/%u/%u ms, fb %u/%u/%u%%, out %u/%u/%u%%)", ms[0], ms[1],
           ms[2], fb[0], fb[1], fb[2], out[0], out[1], out[2]);
    return 1;
}

int gw_AXFXDelayShutdown(void *delay) {
    gw_fx_slot *s = gw_fx_find(delay);
    if (s != NULL && s->kind == GW_FX_KIND_DELAY) {
        gw_fx_delay_free(&s->u.dly);
        s->owner = NULL;
    }
    return 1;
}

void gw_AXFXDelayCallback(void *buffer_update, void *delay) {
    /* Unused; see gw_AXFXReverbStdCallback. */
    (void) buffer_update;
    (void) delay;
}
