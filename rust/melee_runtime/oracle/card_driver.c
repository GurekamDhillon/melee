/* State oracle for all card.c definitions. The generator includes the game C
 * with only four pointer-width repairs, and generates the per-definition switch.
 * This fixture owns a synthetic card image; no disc-derived input is used. */
#define OSDisableInterrupts card_oracle_OSDisableInterrupts
#define OSRestoreInterrupts card_oracle_OSRestoreInterrupts
#define CARDCreateAsync card_oracle_CARDCreateAsync
#define CARDFastOpen card_oracle_CARDFastOpen
#define CARDOpen card_oracle_CARDOpen
#define CARDClose card_oracle_CARDClose
#define CARDGetXferredBytes card_oracle_CARDGetXferredBytes
#define CARDReadAsync card_oracle_CARDReadAsync
#define CARDRead card_oracle_CARDRead
#define CARDGetStatus card_oracle_CARDGetStatus
#define CARDSetStatusAsync card_oracle_CARDSetStatusAsync
#define CARDWriteAsync card_oracle_CARDWriteAsync
#define CARDWrite card_oracle_CARDWrite
#include "card_native.c"
#include "src/sysdolphin/baselib/crypt.c"

extern int printf(const char*, ...);
extern int fflush(void*);
static CardState card_oracle_state;
static CardCmd card_oracle_cmd;
static CardCmdBuf card_oracle_cmd_buf[2];
static CARDStat card_oracle_stat;
static CardIconInfo card_oracle_icon;
static u8 card_oracle_sector[32768];
static u8 card_oracle_data[65536];
static u8 card_oracle_banner[8192];
static u8 card_oracle_icons[32768];
static u8 card_oracle_comment[64];
static u8 card_oracle_disk[262144];
static s32 card_oracle_primary[64];
static const char card_oracle_filename[16] = "native-card";
static u32 card_oracle_hash, card_oracle_trace, card_oracle_calls;
static int card_oracle_busy, card_oracle_result, card_oracle_k, card_oracle_op;
static int card_oracle_out1, card_oracle_out2;
static CARDCallback card_oracle_pending;
static int card_oracle_pending_chan;
static u32 card_oracle_hit[110];

static void card_oracle_h(u32 v) { card_oracle_hash = card_oracle_hash * 0x01000193u ^ v; }
static void card_oracle_t(u32 v) { card_oracle_trace = card_oracle_trace * 31u + v; }
static u32 card_oracle_ptr(const void* p)
{
    unsigned long long a = (unsigned long long) p;
#define REGION(v, id) if (a >= (unsigned long long) &(v) && a < (unsigned long long) &(v) + sizeof(v)) return (id << 24) | (u32) (a - (unsigned long long) &(v));
    if (!p) return 0;
    REGION(card_oracle_state, 1) REGION(card_oracle_cmd, 2)
    REGION(card_oracle_cmd_buf, 3) REGION(card_oracle_stat, 4)
    REGION(card_oracle_icon, 5) REGION(card_oracle_sector, 6)
    REGION(card_oracle_data, 7) REGION(card_oracle_banner, 8)
    REGION(card_oracle_icons, 9) REGION(card_oracle_comment, 10)
    REGION(card_oracle_disk, 11) REGION(card_oracle_primary, 12)
    REGION(card_oracle_filename, 13)
#undef REGION
    return 0xFFFFFFFFu;
}
static int card_oracle_status(void)
{
    card_oracle_calls++;
    if (card_oracle_busy > 0) { card_oracle_busy--; return -1; }
    return card_oracle_result;
}
static void card_oracle_callback(int f, int r) { card_oracle_t(900); card_oracle_t(f); card_oracle_t(r); }
BOOL card_oracle_OSDisableInterrupts(void) { card_oracle_t(1); return 1; }
BOOL card_oracle_OSRestoreInterrupts(BOOL level) { card_oracle_t(2); card_oracle_t(level); return level; }
s32 card_oracle_CARDOpen(s32 chan, const char* filename, CARDFileInfo* info)
{
    int r;
    card_oracle_t(3); card_oracle_t(chan); card_oracle_t(card_oracle_ptr(filename)); card_oracle_t(card_oracle_ptr(info));
    r = card_oracle_status(); if (!r) { info->chan = chan; info->fileNo = 7; }
    return r;
}
s32 card_oracle_CARDFastOpen(s32 chan, s32 no, CARDFileInfo* info)
{
    int r;
    card_oracle_t(4); card_oracle_t(chan); card_oracle_t(no); card_oracle_t(card_oracle_ptr(info));
    r = card_oracle_status(); if (!r) { info->chan = chan; info->fileNo = no; }
    return r;
}
s32 card_oracle_CARDClose(CARDFileInfo* info)
{ card_oracle_t(5); card_oracle_t(card_oracle_ptr(info)); return card_oracle_status(); }
long card_oracle_CARDGetXferredBytes(long chan)
{ card_oracle_t(6); card_oracle_t(chan); return 768 + card_oracle_k; }
long card_oracle_CARDRead(CARDFileInfo* info, void* buf, long len, long off)
{
    int r;
    card_oracle_t(7); card_oracle_t(card_oracle_ptr(info)); card_oracle_t(card_oracle_ptr(buf)); card_oracle_t(len); card_oracle_t(off);
    r = card_oracle_status();
    if (!r && off >= 0 && len >= 0 && off + len <= sizeof(card_oracle_disk)) memcpy(buf, card_oracle_disk + off, len);
    return r;
}
s32 card_oracle_CARDWrite(CARDFileInfo* info, void* buf, s32 len, s32 off)
{
    int r;
    card_oracle_t(8); card_oracle_t(card_oracle_ptr(info)); card_oracle_t(card_oracle_ptr(buf)); card_oracle_t(len); card_oracle_t(off);
    r = card_oracle_status();
    if (!r && off >= 0 && len >= 0 && off + len <= sizeof(card_oracle_disk)) memcpy(card_oracle_disk + off, buf, len);
    return r;
}
s32 card_oracle_CARDReadAsync(CARDFileInfo* info, void* buf, s32 len, s32 off, CARDCallback cb)
{
    int r;
    card_oracle_t(9); card_oracle_t(cb != 0);
    r = card_oracle_CARDRead(info, buf, len, off);
    if (!r) { card_oracle_pending = cb; card_oracle_pending_chan = info->chan; }
    return r;
}
s32 card_oracle_CARDWriteAsync(CARDFileInfo* info, void* buf, s32 len, s32 off, CARDCallback cb)
{
    int r;
    card_oracle_t(10); card_oracle_t(cb != 0);
    r = card_oracle_CARDWrite(info, buf, len, off);
    if (!r) { card_oracle_pending = cb; card_oracle_pending_chan = info->chan; }
    return r;
}
s32 card_oracle_CARDCreateAsync(s32 chan, const char* filename, u32 size, CARDFileInfo* info, CARDCallback cb)
{
    int r;
    card_oracle_t(11); card_oracle_t(chan); card_oracle_t(card_oracle_ptr(filename)); card_oracle_t(size); card_oracle_t(card_oracle_ptr(info)); card_oracle_t(cb != 0);
    r = card_oracle_status();
    if (!r) { info->chan = chan; info->fileNo = 7; info->length = size; card_oracle_pending = cb; card_oracle_pending_chan = chan; }
    return r;
}
s32 card_oracle_CARDGetStatus(s32 chan, s32 no, CARDStat* stat)
{
    int r;
    card_oracle_t(12); card_oracle_t(chan); card_oracle_t(no); card_oracle_t(card_oracle_ptr(stat));
    r = card_oracle_status(); if (!r) *stat = card_oracle_stat;
    return r;
}
s32 card_oracle_CARDSetStatusAsync(s32 chan, s32 no, CARDStat* stat, CARDCallback cb)
{
    int r;
    card_oracle_t(13); card_oracle_t(chan); card_oracle_t(no); card_oracle_t(card_oracle_ptr(stat)); card_oracle_t(cb != 0);
    r = card_oracle_status();
    if (!r) { card_oracle_stat = *stat; card_oracle_pending = cb; card_oracle_pending_chan = chan; }
    return r;
}
static void card_oracle_bytes(const u8* p, int n) { int i; for (i = 0; i < n; i++) card_oracle_h(p[i]); }
/* A spare block's table index 9 aliases file_data[0] on GC. Verify its
 * serialized 22 address bits against the live pointer, then canonicalize
 * that pointer observation for hashing (and regenerate its encryption).
 * No arbitrary bytes or logical state are ignored. */
static void card_oracle_card_bytes(const u8* p, int n)
{
    u8 block[8192];
    int at, i;
    /* Only the direct fn_803B1338 cases serialize spare blocks in this
     * fixture; queued requests are observed before dispatch. */
    if (card_oracle_op != 96) { card_oracle_bytes(p, n); return; }
    for (at = 0; at < n; at += 8192) {
        int encrypted, changed = 0;
        memcpy(block, p + at, 8192);
        encrypted = HSD_Decrypt(block, 8192) == 0;
        if (!encrypted) memcpy(block, p + at, 8192);
        if (block[16] == 255 && block[17] == 255) {
            for (i = 19; i < 31; i += 4) {
                if (block[i] == 9) {
                    u32 got = ((block[i+1] & 63) << 16) | (block[i+2] << 8) | block[i+3];
                    u32 live = (u32)(unsigned long long)card_oracle_state.file_data[0];
                    u32 token = card_oracle_ptr(card_oracle_state.file_data[0]);
                    card_oracle_h(got == (live & 0x3FFFFF));
                    block[i+1] = (block[i+1] & 192) | ((token >> 16) & 63);
                    block[i+2] = token >> 8; block[i+3] = token;
                    changed = 1;
                }
            }
        }
        if (changed && encrypted) HSD_Encrypt(block, 8192);
        if (changed) card_oracle_bytes(block, 8192);
        else card_oracle_bytes(p + at, 8192);
    }
}
static void card_oracle_hash_cmd(const CardCmd* c)
{
    card_oracle_h(c->type); card_oracle_h(card_oracle_ptr(c->state));
    switch (c->type) {
    case CARD_CMD_WRITE_BLOCK: case CARD_CMD_READ_BLOCK: case CARD_CMD_VERIFY_BLOCK:
    case CARD_CMD_CLEAR_BUF: case CARD_CMD_SCAN_BLOCK: case CARD_CMD_READ_SECTOR: case CARD_CMD_WRITE_SECTOR:
        card_oracle_h(c->write.file_idx); card_oracle_h(c->write.phys); card_oracle_h(c->write.block_id); card_oracle_h(c->write.seq);
        card_oracle_h(card_oracle_ptr(c->write.data)); card_oracle_h(c->write.offset); card_oracle_h(c->write.size); break;
    case CARD_CMD_CREATE_FILE:
        card_oracle_h(card_oracle_ptr(c->create.filename)); card_oracle_h(c->create.size); break;
    case CARD_CMD_WRITE_HEADER: case CARD_CMD_VERIFY_HEADER:
        card_oracle_h(c->header.index); card_oracle_h(card_oracle_ptr(c->header.banner)); card_oracle_h(card_oracle_ptr(c->header.icons)); break;
    case CARD_CMD_READ_HEADER:
        card_oracle_h(c->read_header.index); card_oracle_h(card_oracle_ptr(c->read_header.comment));
        card_oracle_h(card_oracle_ptr(c->read_header.banner)); card_oracle_h(card_oracle_ptr(c->read_header.icons)); break;
    case CARD_CMD_GET_STATUS: card_oracle_h(c->get_status.file_no); break;
    }
}
static void card_oracle_hash_req(const CardRequest* r)
{
    card_oracle_h(r->type); card_oracle_h(card_oracle_ptr(r->state)); card_oracle_h(r->callback != 0);
    switch (r->type) {
    case CARD_REQ_READ_FILE: case CARD_REQ_WRITE_FILE:
        card_oracle_h(r->file.file_idx); card_oracle_h(card_oracle_ptr(r->file.buf)); break;
    case CARD_REQ_CREATE_FILE:
        card_oracle_h(card_oracle_ptr(r->create.filename)); card_oracle_h(card_oracle_ptr(r->create.banner)); card_oracle_h(card_oracle_ptr(r->create.icons)); break;
    case CARD_REQ_SET_STATUS:
        card_oracle_h(r->status.x0); card_oracle_h(card_oracle_ptr(r->status.banner)); card_oracle_h(card_oracle_ptr(r->status.icons)); break;
    case CARD_REQ_OPEN_FILE: card_oracle_h(r->open.file_no); break;
    case CARD_REQ_READ_HEADER:
        card_oracle_h(card_oracle_ptr(r->header.comment)); card_oracle_h(card_oracle_ptr(r->header.banner)); card_oracle_h(card_oracle_ptr(r->header.icons)); break;
    }
}
#include "card_hash.inc"
static void card_oracle_hash_all(void)
{
    int i;
    card_oracle_hash_CardState(&card_oracle_state);
    card_oracle_hash_CARDStat(&card_oracle_stat);
    card_oracle_hash_CardIconInfo(&card_oracle_icon);
    card_oracle_hash_cmd(&card_oracle_cmd);
    for (i = 0; i < 2; i++) { card_oracle_hash_cmd(&card_oracle_cmd_buf[i].cmd); card_oracle_bytes((u8*)card_oracle_cmd_buf[i].pad_24, 4); }
    for (i = 0; i < 128; i++) card_oracle_hash_cmd(&commands[i]);
    for (i = 0; i < 32; i++) card_oracle_hash_req(&requests[i]);
    card_oracle_h(curr_head); card_oracle_h(curr_tail); card_oracle_h(curr_result);
    card_oracle_h(hsd_804D798C); card_oracle_h(hsd_804D7990); card_oracle_h(hsd_804D7994); card_oracle_h(hsd_804D7998); card_oracle_h(hsd_804D799C);
    card_oracle_h(active_requests.type); card_oracle_h(card_oracle_ptr(active_requests.state)); card_oracle_h(active_requests.callback != 0); card_oracle_h(active_requests.callback_arg);
    card_oracle_card_bytes(card_oracle_sector, sizeof card_oracle_sector);
    card_oracle_bytes(card_oracle_data, sizeof card_oracle_data);
    card_oracle_bytes(card_oracle_banner, sizeof card_oracle_banner);
    card_oracle_bytes(card_oracle_icons, sizeof card_oracle_icons);
    card_oracle_bytes(card_oracle_comment, sizeof card_oracle_comment);
    card_oracle_card_bytes(card_oracle_disk, sizeof card_oracle_disk);
    card_oracle_h(card_oracle_out1); card_oracle_h(card_oracle_out2);
    card_oracle_h(card_oracle_calls); card_oracle_h(card_oracle_trace); card_oracle_h(card_oracle_pending != 0);
}
static void card_oracle_setup(int k)
{
    CardState* s = &card_oracle_state;
    int i, j, n, logical = 0;
    memset(s, 0, sizeof *s); memset(commands, 0, sizeof commands); memset(requests, 0, sizeof requests);
    memset(&active_requests, 0, sizeof active_requests); memset(&card_oracle_cmd, 0, sizeof card_oracle_cmd);
    memset(card_oracle_cmd_buf, 0, sizeof card_oracle_cmd_buf); memset(&card_oracle_stat, 0, sizeof card_oracle_stat);
    memset(&card_oracle_icon, 0, sizeof card_oracle_icon); memset(card_oracle_sector, 0, sizeof card_oracle_sector);
    memset(card_oracle_disk, 0, sizeof card_oracle_disk);
    for (i = 0; i < sizeof card_oracle_data; i++) card_oracle_data[i] = (i * 13 + k * 7) & 255;
    for (i = 0; i < sizeof card_oracle_banner; i++) card_oracle_banner[i] = (i * 3 + k) & 255;
    for (i = 0; i < sizeof card_oracle_icons; i++) card_oracle_icons[i] = (i * 17 + k * 9) & 255;
    for (i = 0; i < 64; i++) { card_oracle_comment[i] = (i * 3 + k) & 255; card_oracle_primary[i] = i; }
    s->sector_buf = card_oracle_sector; s->sector_size = 8192; s->chan = k & 1; s->file_no = 7;
    s->file_info.chan = s->chan; s->file_info.fileNo = 7; s->file_info.length = 8192 * 20;
    s->icon_info.banner_format = k % 3;
    for (i = 0; i < k % 9; i++) { s->icon_info.icon_speed[i] = 1 + (i + k) % 3; s->icon_info.icon_format[i] = 1 + i % 2; }
    s->header_size = hsd_803AC340(&s->icon_info);
    memcpy(s->comment, card_oracle_comment, 64); card_oracle_icon = s->icon_info;
    card_oracle_stat.bannerFormat = s->icon_info.banner_format;
    for (i = 0; i < 8; i++) { card_oracle_stat.iconFormat |= s->icon_info.icon_format[i] << (i * 2); card_oracle_stat.iconSpeed |= s->icon_info.icon_speed[i] << (i * 2); }
    card_oracle_stat.length = 8192 * 20; s->stat = card_oracle_stat;
    for (i = 0; i < 3; i++) { s->file_sizes[i] = 128 + ((k + i * 7) % 3) * 8192; s->file_flags[i] = (k + i) % 4; s->file_data[i] = card_oracle_data + i * 20000; }
    s->num_blocks = fn_803AC7DC(s);
    for (i = 0; i < 64; i++) { s->block_ids[i] = -0x7FFF; s->block_seqs[i] = 0; }
    s->block_ids[0] = 0; s->block_seqs[0] = k & 255;
    n = 1;
    for (i = 0; i < 3; i++) {
        int count = fn_803AC634(s, i), start = i == 0 ? 1 : 0;
        for (j = start; j < count; j++) { s->block_ids[n] = logical + j; s->block_seqs[n++] = k & 255; }
        logical += count;
    }
    for (i = 0; i <= s->num_blocks && i < 25; i++) {
        int offset = fn_803ACBE8(s, i), hdr = i == 0 ? (s->header_size + 48) % 8192 : 0;
        u8* b = card_oracle_disk + offset + hdr;
        b[16] = s->block_ids[i] >> 8; b[17] = s->block_ids[i]; b[18] = s->block_seqs[i];
        fn_803AC3F8(s, b + 19, i % 3);
        for (j = 32; j < 8192 - hdr; j++) b[j] = card_oracle_data[(i * 8176 + j - 32) % 65536];
        HSD_Encrypt(b, 8192 - hdr);
    }
    memcpy(card_oracle_sector, card_oracle_disk, 8192);
    curr_head = curr_tail = k % 128; curr_result = 0; hsd_804D798C = 0;
    hsd_804D7990 = hsd_804D7994 = k % 32; hsd_804D7998 = -1; hsd_804D799C = 2;
    card_oracle_pending = 0; card_oracle_pending_chan = 0;
    card_oracle_calls = card_oracle_trace = card_oracle_hash = 0;
    card_oracle_busy = k % 13;
    card_oracle_result = k % 5 == 0 ? -3 : k % 7 == 0 ? -5 : 0;
    card_oracle_out1 = card_oracle_out2 = 0; card_oracle_k = k;
    card_oracle_cmd.state = s; card_oracle_cmd.type = CARD_CMD_CLEAR_BUF;
    card_oracle_cmd.clear.data = card_oracle_data; card_oracle_cmd.clear.size = 16 + k % 128;
    memset(card_oracle_hit, 0, sizeof card_oracle_hit);
}

static u32 card_oracle_run(int op, int k)
{
    CardState* s = &card_oracle_state;
    CardCmd* c = &card_oracle_cmd;
    int r = 0, i, b = k % 3, f = k % 3, len = k % 4 == 0 ? 0 : 16 + k % 128;
    card_oracle_setup(k);
    card_oracle_op = op;
    if (k >= 48) {
        hsd_804D799C = 0;
        commands[curr_head] = *c;
        requests[hsd_804D7990].type = CARD_REQ_READ_FILE;
        requests[hsd_804D7990].state = s;
        requests[hsd_804D7990].file.file_idx = f;
        requests[hsd_804D7990].file.buf = card_oracle_data;
    }
#include "card_cases.inc"
    card_oracle_h(r);
    card_oracle_hash_all();
    return card_oracle_hash;
}
int main(void)
{
    int op, k;
    for (op = 0; op < 110; op++) {
        for (k = 0; k < 96; k++) {
            u32 h = card_oracle_run(op, k);
            printf("%d %d %08X\n", op, k, h);
        }
    }
    return 0;
}
