/* Historical inputs to the CURATED checksum, included by gw_rollback.c.
 * Native-only metadata: never snapshotted or hashed. Capture uses the very mix
 * calls that compute the wire checksum, so the field list cannot drift. A late
 * peer report must use these records, never reread the current game state.
 * Offsets name 32-bit words in a region's hash stream, not struct byte offsets.
 * Item slots are list ordinals for diagnostics; RB_ItemHash still sums items.
 */
#define RB_HASH_WORDS 4096

typedef struct {
    int region, slot, offset;
    uint32_t value;
    const char *symbol; /* immutable game image string, not part of any hash */
} RbHashWord;
typedef struct {
    int valid, epoch, frame, count, overflow;
    uint32_t hash, seed_arrived, seed_effective;
    int seed_valid;
    RbHashWord words[RB_HASH_WORDS];
} RbHashDetail;
static RbHashDetail rb_hash_history[RB_RING], *rb_hash_capture;
static int rb_hash_region, rb_hash_slot, rb_hash_offset;
static FILE *rb_hash_file;
static int rb_hash_file_tried;

static void rb_hash_detail_reset(void) {
    int i;
    rb_hash_capture = NULL;
    for (i = 0; i < RB_RING; ++i) rb_hash_history[i].valid = 0;
    if (!rb_hash_file_tried) {
        const char *path = getenv("MELEE_RB_HASH_DETAIL");
        rb_hash_file_tried = 1;
        if (path != NULL && *path != '\0') {
            rb_hash_file = fopen(path, "w");
            if (rb_hash_file != NULL) {
                fprintf(rb_hash_file, "kind,epoch,frame,region,slot,offset,symbol,value\n");
                gw_log("rbhash: confirmed region/word log enabled");
            } else {
                gw_log("rbhash: cannot open MELEE_RB_HASH_DETAIL");
            }
        }
    }
}

static void rb_hash_detail_begin(int frame) {
    RbHashDetail *d = &rb_hash_history[(unsigned) frame % RB_RING];
    d->valid = 0;
    d->epoch = rb.epoch;
    d->frame = frame;
    d->count = d->overflow = d->seed_valid = 0;
    rb_hash_capture = d;
    rb_hash_region = rb_hash_slot = rb_hash_offset = 0;
}

void gw_RbHashSeed(uint32_t arrived, uint32_t effective) {
    if (rb_hash_capture == NULL) return;
    rb_hash_capture->seed_arrived = arrived;
    rb_hash_capture->seed_effective = effective;
    rb_hash_capture->seed_valid = 1;
}

void gw_RbHashRegion(int region, int slot) {
    rb_hash_region = region;
    rb_hash_slot = slot;
    rb_hash_offset = 0;
}

void gw_RbHashWord(uint32_t value, const char *symbol) {
    RbHashDetail *d = rb_hash_capture;
    RbHashWord *w;
    if (d == NULL) return;
    if (d->count == RB_HASH_WORDS) { d->overflow = 1; return; }
    w = &d->words[d->count++];
    w->region = rb_hash_region;
    w->slot = rb_hash_slot;
    w->offset = rb_hash_offset++;
    w->value = value;
    w->symbol = symbol;
}

static void rb_hash_detail_end(uint32_t hash) {
    if (rb_hash_capture != NULL) {
        rb_hash_capture->hash = hash;
        rb_hash_capture->valid = 1;
    }
    rb_hash_capture = NULL;
}

static const RbHashDetail *rb_hash_detail_find(int frame) {
    const RbHashDetail *d = &rb_hash_history[(unsigned) frame % RB_RING];
    return d->valid && d->epoch == rb.epoch && d->frame == frame ? d : NULL;
}

static const char *rb_hash_region_name(int region) {
    return region == 1 ? "fighter" : region == 2 ? "item" : "match";
}

static void rb_hash_detail_emit(const RbHashDetail *d, FILE *file) {
    int i, j;
    if (file != NULL) {
        fprintf(file, "hash,%d,%d,match,0,-1,RB_GameHashForFrame,%08X\n", d->epoch, d->frame, d->hash);
        if (d->overflow) fprintf(file, "overflow,%d,%d,match,0,-1,truncated,1\n", d->epoch, d->frame);
    } else {
        gw_log("rbhash: epoch %d frame %d hash %08X words %d%s (offsets are hash-stream words)",
               d->epoch, d->frame, d->hash, d->count, d->overflow ? " TRUNCATED" : "");
    }
    if (d->seed_valid) {
        if (file != NULL) {
            fprintf(file, "seed,%d,%d,match,0,-1,arrived_rng,%08X\n", d->epoch, d->frame, d->seed_arrived);
            fprintf(file, "seed,%d,%d,match,0,-1,effective_rng,%08X\n", d->epoch, d->frame, d->seed_effective);
        } else gw_log("rbhash: frame %d RNG arrived %08X effective %08X", d->frame, d->seed_arrived, d->seed_effective);
    }
    for (i = 0; i < d->count; i = j) {
        const RbHashWord *first = &d->words[i];
        const char *region = rb_hash_region_name(first->region);
        uint32_t h = 0x811C9DC5u;
        for (j = i; j < d->count && d->words[j].region == first->region && d->words[j].slot == first->slot; ++j) {
            h = (h ^ d->words[j].value) * 0x01000193u;
            h ^= h >> 15;
        }
        if (file != NULL) fprintf(file, "region,%d,%d,%s,%d,-1,region_hash,%08X\n", d->epoch, d->frame, region, first->slot, h);
        else gw_log("rbhash: frame %d region %s[%d] hash %08X", d->frame, region, first->slot, h);
        for (; i < j; ++i) {
            const RbHashWord *w = &d->words[i];
            if (file != NULL) fprintf(file, "word,%d,%d,%s,%d,%d,\"%s\",%08X\n", d->epoch, d->frame, region, w->slot, w->offset, w->symbol, w->value);
            else gw_log("rbhash: frame %d %s[%d] word +%d %s = %08X", d->frame, region, w->slot, w->offset, w->symbol, w->value);
        }
    }
}

void gw_rb_hash_desync(int frame, uint32_t local, uint32_t remote) {
    const RbHashDetail *d = rb_hash_detail_find(frame);
    gw_log("rbhash: first peer mismatch epoch %d frame %d local %08X peer %08X", rb.epoch, frame, local, remote);
    if (d == NULL || d->hash != local) {
        gw_log("rbhash: frame %d history unavailable or replaced; use MELEE_RB_HASH_DETAIL on both peers", frame);
        return;
    }
    rb_hash_detail_emit(d, NULL);
}
