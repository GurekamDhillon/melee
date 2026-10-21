// gc_adapter_decode.h compiled as C (as in the shipping build), behind one entry point.
#include "pad_types.h"
#include <string.h>
static void gw_w16(u16 *p, u16 v) { *p = v; }
static void gw_log(const char *f, ...) { (void)f; }
#define GW_GC_TRIG_DEAD 200
static unsigned char gw_gc_origin[4][4];
static int gw_gc_have_origin[4];
static unsigned char gw_gc_trig_rest[4][2];
static unsigned char gw_gc_btn_stuck[4][2];
#include "gc_adapter_decode.h"
int oracle_decode(const unsigned char *snap, void *status, int recal) { return gw_gc_decode(snap, status, recal); }
