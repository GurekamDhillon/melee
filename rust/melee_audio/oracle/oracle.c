/* Oracle for melee_audio: pc/platform/shim_ax.c itself (copied verbatim by build.sh and compiled as
 * the shipping build compiles shims: clang --target=i686-pc-windows-msvc -O2, default
 * -ffp-contract=on), with the inert gw/SDL stand-ins in fake/. Prints test vectors to stdout:
 *   part D: gw_ax_decode_one over random decoder states (inputs + every decoded sample + end state)
 *   part M: a random AX API script (every call printed with its arguments in game byte order) with
 *           gw_ax_run_frame between calls; after each frame the 320 output samples and the
 *           game-visible voice fields, hashed (FNV-1a 64), plus the full frame every 97th. */
#include "shim_ax.c"
#include <stdio.h>

unsigned char *gw_aram;
uint32_t gw_aram_size;

static uint32_t rs = 0x2545F491u;
static uint32_t rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static uint32_t rn(uint32_t n) { return rnd() % n; }

/* ARAM fill shared with the Rust test (tests/oracle.rs aram_fill): xorshift32 bytes, with every
 * ADPCM frame header (byte offset % 8 == 0) kept to predictor 0..7 (shim_ax.c indexes coef[8]
 * with the raw predictor nibble, so 8..15 would read past the table: undefined in C). */
static void aram_fill(uint8_t *a, uint32_t n, uint32_t seed) {
    uint32_t x = seed, i;
    for (i = 0; i < n; i++) {
        x ^= x << 13; x ^= x >> 17; x ^= x << 5;
        a[i] = (uint8_t)(x >> 24);
        if ((i & 7) == 0) a[i] &= 0x7F;
    }
}

static uint64_t fnv(uint64_t h, const void *p, size_t n) {
    const uint8_t *b = (const uint8_t *)p;
    while (n--) { h ^= *b++; h *= 0x100000001B3ull; }
    return h;
}

static void hex(const uint8_t *p, int n) { int i; for (i = 0; i < n; i++) printf("%02x", p[i]); }

static void part_d(void) {
    static uint8_t aram[4096];
    static const uint16_t fmts[8] = { 0, 0, 0, 0, 0, 0xA, 0x19, 3 };
    int c, k, i;
    aram_fill(aram, sizeof aram, 0xA11CE5u);
    gw_aram = aram;
    gw_aram_size = sizeof aram;
    for (c = 0; c < 4000; c++) {
        gw_ax_vstate s;
        memset(&s, 0, sizeof s);
        for (i = 0; i < 8; i++) { s.coef[i][0] = (int16_t)rnd(); s.coef[i][1] = (int16_t)rnd(); }
        s.pred_scale = (uint8_t)(rnd() & 0x7F);
        s.yn1 = (int16_t)rnd(); s.yn2 = (int16_t)rnd();
        s.format = fmts[rn(8)];
        s.cur_addr = rn(sizeof aram * 2 + 64);
        s.end_addr = rn(4) ? s.cur_addr + rn(160) : rnd();
        s.loop_addr = rn(sizeof aram * 2 + 64);
        s.loop_flag = (uint16_t)rn(2);
        s.loop_pred_scale = (uint8_t)(rnd() & 0x7F);
        s.loop_yn1 = (int16_t)rnd(); s.loop_yn2 = (int16_t)rnd();
        s.active = true;
        printf("D");
        for (i = 0; i < 8; i++) printf(" %d %d", s.coef[i][0], s.coef[i][1]);
        printf(" %u %d %d %u %u %u %u %u %u %d %d\n", s.pred_scale, s.yn1, s.yn2, s.format, s.cur_addr,
               s.end_addr, s.loop_addr, s.loop_flag, s.loop_pred_scale, s.loop_yn1, s.loop_yn2);
        printf("O");
        for (k = 0; k < 64; k++) printf(" %d", gw_ax_decode_one(&s));
        printf("\nE %u %d %d %u %d %u\n", s.pred_scale, s.yn1, s.yn2, s.cur_addr, s.active ? 1 : 0, s.state);
    }
}

/* AXFX parameter blocks in "game memory" (big-endian); owner keys 1 = reverb block, 2 = delay
 * block, 3 = an address no effect owns. */
static uint8_t fx_rev[0x160], fx_dly[0x60];
static uint8_t bogus_fx;
static void *owner_ptr(int k) { return k == 1 ? (void *)fx_rev : k == 2 ? (void *)fx_dly : (void *)&bogus_fx; }
static void wf(uint8_t *p, float f) { uint32_t v; memcpy(&v, &f, 4); gw_w32(p, v); }

#define NCLIP 4
static int16_t clip_pcm[NCLIP][2 * 2400];
static int clip_frames[NCLIP], clip_ch[NCLIP];

static void part_m(void) {
    static uint8_t aram[1 << 18];
    static const uint16_t fmts[6] = { 0, 0, 0, 0xA, 0x19, 5 };
    int step, frame = 0, c, i, t;
    aram_fill(aram, sizeof aram, 0xBEEF01u);
    gw_aram = aram;
    gw_aram_size = sizeof aram;
    /* clip PCM shared with the Rust test (clip_pcm there) */
    for (c = 0; c < NCLIP; c++) {
        clip_ch[c] = 1 + (c & 1);
        clip_frames[c] = 200 + c * 700;
        for (i = 0; i < clip_frames[c] * clip_ch[c]; i++) clip_pcm[c][i] = (int16_t)(((i * 37 + c * 1000) % 4000) * 8 - 16000);
    }
    for (step = 0; step < 30000; step++) {
        uint32_t op = rn(100);
        int idx = (int)rn(GW_AX_NUM_VOICES);
        gw_ax_voice *v = &gw_ax_voices[idx];
        if (op < 6) {
            uint32_t prio = rn(32), ctx = rnd();
            gw_ax_voice *got = (gw_ax_voice *)gw_AXAcquireVoice(prio, NULL, ctx);
            printf("acq %u %u %d\n", prio, ctx, got ? (int)(got - gw_ax_voices) : -1);
        } else if (op < 8) {
            printf("free %d\n", idx); gw_AXFreeVoice(v);
        } else if (op < 9) {
            uint32_t p = rn(32); printf("prio %d %u\n", idx, p); gw_AXSetVoicePriority(v, p);
        } else if (op < 16) {
            uint16_t st = (uint16_t)(rn(4) ? 1 : rn(3)); printf("state %d %u\n", idx, st); gw_AXSetVoiceState(v, st);
        } else if (op < 22) {
            uint8_t m[0x24];
            for (t = 0; t < 0x24; t += 2) gw_w16(m + t, (uint16_t)(rn(3) ? rn(0x8001) : 0));
            if (rn(4)) { for (t = 2; t < 0x24; t += 4) gw_w16(m + t, 0); } /* Melee never ramps */
            else { for (t = 2; t < 0x24; t += 4) gw_w16(m + t, (uint16_t)((int)rn(201) - 100)); }
            if (rn(3) == 0) { for (t = 0x08; t < 0x24; t++) m[t] = 0; } /* main-only */
            printf("mix %d ", idx); hex(m, 0x24); printf("\n"); gw_AXSetVoiceMix(v, m);
        } else if (op < 24) {
            printf("itdon %d\n", idx); gw_AXSetVoiceItdOn(v);
        } else if (op < 27) {
            uint16_t l = (uint16_t)rn(40), r = (uint16_t)rn(40);
            printf("itdt %d %u %u\n", idx, l, r); gw_AXSetVoiceItdTarget(v, l, r);
        } else if (op < 33) {
            uint8_t p[4];
            gw_w16(p, (uint16_t)rn(0x8000));
            gw_w16(p + 2, (uint16_t)(rn(3) ? 0 : (int)rn(2001) - 1000));
            printf("ve %d ", idx); hex(p, 4); printf("\n"); gw_AXSetVoiceVe(v, p);
        } else if (op < 35) {
            int16_t d = (int16_t)((int)rn(2001) - 1000); printf("vedelta %d %d\n", idx, d); gw_AXSetVoiceVeDelta(v, d);
        } else if (op < 43) {
            uint8_t p[0x10];
            uint32_t cur = rn(sizeof aram * 2 + 256);
            gw_w16(p, (uint16_t)rn(2)); gw_w16(p + 2, fmts[rn(6)]);
            gw_w32(p + 4, rn(sizeof aram * 2)); gw_w32(p + 8, rn(8) ? cur + rn(40000) : rnd()); gw_w32(p + 12, cur);
            printf("addr %d ", idx); hex(p, 0x10); printf("\n"); gw_AXSetVoiceAddr(v, p);
        } else if (op < 45) {
            uint16_t l = (uint16_t)rn(2); printf("loop %d %u\n", idx, l); gw_AXSetVoiceLoop(v, l);
        } else if (op < 47) {
            uint32_t a = rn(sizeof aram * 2); printf("loopaddr %d %u\n", idx, a); gw_AXSetVoiceLoopAddr(v, a);
        } else if (op < 49) {
            uint32_t a = rn(sizeof aram * 2 + 64); printf("endaddr %d %u\n", idx, a); gw_AXSetVoiceEndAddr(v, a);
        } else if (op < 51) {
            uint32_t a = rn(sizeof aram * 2); printf("curaddr %d %u\n", idx, a); gw_AXSetVoiceCurrentAddr(v, a);
        } else if (op < 58) {
            uint8_t p[0x28];
            for (t = 0; t < 0x20; t += 2) gw_w16(p + t, (uint16_t)((int)rn(8192) - 4096));
            gw_w16(p + 0x20, 0); gw_w16(p + 0x22, (uint16_t)(rnd() & 0x7F));
            gw_w16(p + 0x24, (uint16_t)rnd()); gw_w16(p + 0x26, (uint16_t)rnd());
            printf("adpcm %d ", idx); hex(p, 0x28); printf("\n"); gw_AXSetVoiceAdpcm(v, p);
        } else if (op < 62) {
            uint8_t p[6];
            gw_w16(p, (uint16_t)(rnd() & 0x7F)); gw_w16(p + 2, (uint16_t)rnd()); gw_w16(p + 4, (uint16_t)rnd());
            printf("adpcmloop %d ", idx); hex(p, 6); printf("\n"); gw_AXSetVoiceAdpcmLoop(v, p);
        } else if (op < 66) {
            uint8_t p[4]; uint32_t r = rn(0x40000);
            gw_w16(p, (uint16_t)(r >> 16)); gw_w16(p + 2, (uint16_t)r);
            printf("src %d ", idx); hex(p, 4); printf("\n"); gw_AXSetVoiceSrc(v, p);
        } else if (op < 69) {
            float f = (float)((int)rn(50000) - 2000) / 9999.0f; uint32_t b; memcpy(&b, &f, 4);
            printf("srcratio %d %u\n", idx, b); gw_AXSetVoiceSrcRatio(v, f);
        } else if (op < 70) {
            int ret;
            wf(fx_rev + 0x140, (float)rn(1100) / 1000.0f); /* coloration (sometimes > 1: init fails) */
            wf(fx_rev + 0x144, (float)rn(1000) / 1000.0f); /* mix */
            wf(fx_rev + 0x148, 0.01f + (float)rn(3000) / 1000.0f); /* time */
            wf(fx_rev + 0x14C, (float)rn(1000) / 1000.0f); /* damping */
            wf(fx_rev + 0x150, (float)rn(110) / 1000.0f); /* predelay (sometimes > 0.1) */
            ret = gw_AXFXReverbStdInit(fx_rev);
            printf("revinit 1 "); hex(fx_rev + 0x13C, 0x18); printf(" %d\n", ret);
        } else if (op < 71) {
            int ret;
            for (t = 0; t < 3; t++) {
                gw_w32(fx_dly + 0x3C + t * 4, rn(300));
                gw_w32(fx_dly + 0x48 + t * 4, rn(100));
                gw_w32(fx_dly + 0x54 + t * 4, rn(101));
            }
            ret = gw_AXFXDelayInit(fx_dly);
            printf("dlyinit 2 "); hex(fx_dly + 0x3C, 0x24); printf(" %d\n", ret);
        } else if (op < 72) {
            int k = (int)rn(3) + 1; printf("revshut %d\n", k); gw_AXFXReverbStdShutdown(owner_ptr(k));
        } else if (op < 73) {
            int k = (int)rn(3) + 1; printf("dlyshut %d\n", k); gw_AXFXDelayShutdown(owner_ptr(k));
        } else if (op < 75) {
            int on = (int)rn(4), k = (int)rn(3) + 1;
            printf("auxa %d %d\n", on ? 1 : 0, k); gw_AXRegisterAuxACallback(on ? (void *)1 : NULL, owner_ptr(k));
        } else if (op < 77) {
            int on = (int)rn(4), k = (int)rn(3) + 1;
            printf("auxb %d %d\n", on ? 1 : 0, k); gw_AXRegisterAuxBCallback(on ? (void *)1 : NULL, owner_ptr(k));
        } else if (op < 78) {
            int cl = (int)rn(NCLIP), vol = (int)rn(160) - 10; uint32_t owner = rn(3);
            int r = gw_Audio_PlayOwnedClip(clip_pcm[cl], clip_frames[cl], clip_ch[cl], vol, owner);
            printf("clip %d %d %u %d\n", cl, vol, owner, r);
        } else if (op < 79) {
            int cl = (int)rn(NCLIP); uint32_t owner = rn(3);
            if (rn(2)) printf("stopclip %d %d\n", cl, gw_Audio_StopClips(clip_pcm[cl]));
            else printf("stopowned %d %u %d\n", cl, owner, gw_Audio_StopOwnedClips(clip_pcm[cl], owner));
        } else {
            uint64_t h = 0xCBF29CE484222325ull;
            gw_ax_run_frame();
            h = fnv(h, gw_ax_out, sizeof gw_ax_out);
            for (i = 0; i < GW_AX_NUM_VOICES; i++) {
                const uint8_t *vb = (const uint8_t *)&gw_ax_voices[i];
                h = fnv(h, vb + 0x0C, 4); /* priority */
                h = fnv(h, vb + 0x14, 8); /* userContext, index */
                h = fnv(h, vb + GW_AX_VPB_PB_STATE, 2);
                h = fnv(h, vb + GW_AX_VPB_ITD_FLAG, 2);
                h = fnv(h, vb + GW_AX_VPB_VE_VOL, 2);
                h = fnv(h, vb + GW_AX_VPB_CURADDR, 4);
            }
            printf("frame %d %016llx", frame, (unsigned long long)h);
            if (frame % 97 == 0) { printf(" ="); for (i = 0; i < 320; i++) printf(" %d", gw_ax_out[i]); }
            printf("\n");
            frame++;
        }
    }
    printf("clipstat %u %u\n", gw_Audio_ClipStat(0), gw_Audio_ClipStat(1));
}

int main(void) {
    part_d();
    part_m();
    return 0;
}
