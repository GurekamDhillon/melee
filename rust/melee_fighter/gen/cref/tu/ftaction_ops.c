/* TU: src/melee/ft/ftaction.c */
/* C oracle for src/tu/ftaction.rs (the fighter subaction command handlers and runners): the whole TU included
 * verbatim; every callee out of it is a recording stub (gen/mtu/stubs.py); the fighter is a random world filled
 * field by field (gen/mtu/fill.py). A handler's command words are filled through the union view that handler reads
 * (field by field, so the C build's LSB-first bitfields and the Rust types' GC-order ones hold the same logical
 * values); the runners get a stream of opcode-only words (zero payload reads the same through every layout).
 * Each line: op case ret state-hash trace. */
#include TU_SRC
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/gobjproc.h>
#include "mtu_common.h"
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCount(int a) { (void) a; }
int ProfActive(void) { return 0; }

#define NG 3
#define NJ 8
#define NW 512
static HSD_GObj GO[NG];
static HSD_JObj JO[NJ];
static Fighter FT[NG];
static ftData FDATA[NG];
static FighterBone PARTS[NG][256];
static CmdUnion W[NW];
static CommandInfo CI;
static HSD_GObjProc PROC;
static u8 PADB[72][2048];

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % NG]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % NJ]; }
static Fighter* pick_Fighter(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FT[(r >> 8) % NG]; }
static Item* pick_Item(void) { return 0; }
static HSD_DObj* pick_HSD_DObj(void) { return 0; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : (void*) cbev1); }

#include "ftaction_stubs.h"
#include "ftaction_state.h"

/* the command interpreter of lbcommand.c, reduced: a drawn timer / advance / result, so the runner loops end */
int Command_Execute(CommandInfo* cmd, u32 code)
{
    u32 r = fv();
    T(1900); T(PI(cmd)); T(code); T((u32) (cmd->u - W)); T(B(cmd->timer)); T(r);
    if (r & 4) cmd->timer = (float) (r % 7) + 1.0f;
    if (r & 2) ++cmd->u;
    if (code < 10) { if (!(r & 6)) ++cmd->u; return 1; }   /* opcodes 0-9 are always the interpreter's */
    return (int) (r & 1);
}
void Geno_FtCmd(Fighter_GObj* g, CommandInfo* cmd, int mode)
{
    T(1901); T(PI(g)); T(PI(cmd)); T((u32) mode);
    ++cmd->u;
}

static void world(void)
{
    int i, b;
    zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT); zero(FDATA, sizeof FDATA);
    for (i = 0; i < NG; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) {
        GO[i].user_data = &FT[i];
        GO[i].hsd_obj = &JO[i];
        GO[i].classifier = HSD_GOBJ_CLASS_FIGHTER;
        FT[i].gobj = &GO[i];
        FT[i].ft_data = &FDATA[i];
        FT[i].grab_cb = cbev1; FT[i].grabbed_cb = cbev3;
        FT[i].kind = (FighterKind) ((u32) FT[i].kind % 27u); FT[i].player_id %= 4; FT[i].x618_player_id %= 4;
        for (b = 0; b < 256; b++) { zero(&PARTS[i][b], sizeof PARTS[i][b]); PARTS[i][b].joint = &JO[(u32) (i * 5 + b * 3) % NJ]; }
        FT[i].parts = PARTS[i];
        { int pk = i * 24; FDATA[i].x0 = (void*) PADB[pk++]; FDATA[i].ext_attr = (void*) PADB[pk++]; FDATA[i].x8 = (void*) PADB[pk++]; FDATA[i].xC = (void*) PADB[pk++]; FDATA[i].x10 = (void*) PADB[pk++]; FDATA[i].x14 = (void*) PADB[pk++]; FDATA[i].x18 = (void*) PADB[pk++]; FDATA[i].x1C = (void*) PADB[pk++]; FDATA[i].x20 = (void*) PADB[pk++]; FDATA[i].x24 = (void*) PADB[pk++]; FDATA[i].x28 = (void*) PADB[pk++]; FDATA[i].x2C = (void*) PADB[pk++]; FDATA[i].x30 = (void*) PADB[pk++]; FDATA[i].x34 = (void*) PADB[pk++]; FDATA[i].x38 = (void*) PADB[pk++]; FDATA[i].x3C = (void*) PADB[pk++]; FDATA[i].x40 = (void*) PADB[pk++]; FDATA[i].x44 = (void*) PADB[pk++]; FDATA[i].x48_items = (void*) PADB[pk++]; FDATA[i].x4C_sfx = (void*) PADB[pk++]; FDATA[i].x50 = (void*) PADB[pk++]; FDATA[i].x54 = (void*) PADB[pk++]; FDATA[i].x58 = (void*) PADB[pk++]; FDATA[i].x5C = (void*) PADB[pk++]; }
    }
    for (i = 0; i < NJ; i++) {
        JO[i].parent = i ? &JO[(i - 1) / 2] : 0;
        JO[i].child = 2 * i + 1 < NJ ? &JO[2 * i + 1] : 0;
        JO[i].next = (i & 1) && i + 1 < NJ ? &JO[i + 1] : 0;
    }
    PROC.s_link = (u8) (fv() % 20);
    HSD_GObj_CurrentInvokedProc = (fv() & 1) ? &PROC : 0;
    zero(W, sizeof W);
    zero(&CI, sizeof CI);
    CI.u = W;
    CI.timer = rw();
    CI.frame_count = rg();
}
static void hworld(void)
{
    int i;
    HM((u32) (CI.u - W)); HM(B(CI.timer)); HM(B(CI.frame_count));
    for (i = 0; i < NG; i++) {
        CmdUnion* uu = FT[i].x3E4_fighterCmdScript.u;
        HM((uu >= W && uu < W + NW) ? (u32) (uu - W) : 0xFFFFu);
        if (uu >= W && uu < W + NW) FT[i].x3E4_fighterCmdScript.u = 0;   /* an address: not hashed as bytes */
    }
    for (i = 0; i < NG; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) hash_HSD_GObj(&GO[i]);
}

/* the opcode-only stream the runners walk: ids 10..59, payload zero */
static void stream(void)
{
    int i;
    /* one command per 8 words (the longest command is 7): a handler that reads more or fewer words than its table says
       still only sees zero payload, which every layout reads alike */
    for (i = 0; i < NW; i += 8) W[i].Command_09.id = 10 + rnd() % 50;
}

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT); preg(FDATA, sizeof FDATA); preg(&CI, sizeof CI); preg(&PROC, sizeof PROC);
    freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3);
    for (op = 0; op < 18; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            tu_case(op, k);
            world();
            ASRT_ARMED = 1;
            if (_setjmp(ASRT_JB, 0) != 0) { printf("%d %d ASSERT%c", op, k, 10); fflush(0); continue; }
            {
                Fighter_GObj* g = &GO[rnd() % NG];
                Fighter* fp = &FT[(u32) (g - GO)];
                switch (op) {
                case 0: /* 8007121C: hitbox spawn. The skip test reads byte 16 bit 4 from the start of the stream in the C build (the first byte of word 2,
                       a create_hitbox_2 field: sizeof(CmdUnion) is 8) and GC byte 0xF (padding) in the Rust types, so word 2 stays zero in both */
                    fill_spawn_hitbox_0(&W[0].create_hitbox_0); fill_spawn_hitbox_1(&W[1].create_hitbox_1);
                    fill_spawn_hitbox_3(&W[3].create_hitbox_3);
                    fill_spawn_hitbox_4(&W[4].create_hitbox_4); fill_spawn_hitbox_5(&W[5].create_hitbox_5);
                    if (W[0].create_hitbox_0.id >= 4) fp->x914[3].state = HitCapsule_Disabled; /* a free slot: no out-of-array retail indexing */
                    ftAction_8007121C(g, &CI); break;
                case 1: fill_set_hitbox_x42_b57(&W[0].set_hitbox_x42_b57); W[0].set_hitbox_x42_b57.idx %= 8; if (W[0].set_hitbox_x42_b57.idx >= 4) { fp->x914[0].x43_b7 = 1; fp->x914[0].state = HitCapsule_Enabled; } ftAction_80071708(g, &CI); break;
                case 2: fill_set_throw_flags(&W[0].set_throw_flags); ftAction_800718A4(g, &CI); break;
                case 3: fill_set_jab_combo(&W[0].set_jab_combo); ftAction_80071AE8(g, &CI); break;
                case 4:
                    fill_set_throw_hitbox_0(&W[0].set_throw_hitbox_0); fill_set_throw_hitbox_1(&W[1].set_throw_hitbox_1);
                    fill_set_throw_hitbox_2(&W[2].set_throw_hitbox_2); W[0].set_throw_hitbox_0.idx %= 2;
                    ftAction_80071E04(g, &CI); break;
                case 5: fill_unk27(&W[0].unk27); ftAction_80071F34(g, &CI); break;
                case 6: fill_set_tex_anim(&W[0].set_tex_anim); ftAction_800726F4(g, &CI); break;
                case 7: fill_part_anim(&W[0].part_anim); ftAction_800727C8(g, &CI); break;
                case 8: fill_unk10(&W[0].unk10); ftAction_800728F8(g, &CI); break;
                case 9: fill_unk19(&W[0].unk19); ftAction_80072C6C(g, &CI); break;
                case 10: ftAction_8007296C(g, &CI); break;
                case 11: ftAction_80072E24(g, &CI); break;
                case 12: ftAction_800717D8(g, &CI); break;
                case 13: stream(); fp->x3E4_fighterCmdScript.u = W; fp->x3E4_fighterCmdScript.timer = CI.timer; ftAction_80073240(g); break;
                case 14: stream(); fp->x3E4_fighterCmdScript.u = W; fp->x3E4_fighterCmdScript.timer = CI.timer; ftAction_80073354(g); break;
                case 15: stream(); fp->x3E4_fighterCmdScript.u = W; fp->x3E4_fighterCmdScript.timer = CI.timer; ftAction_8007349C(g); break;
                case 16: stream(); ftAction_80071820(g, &CI); break;
                case 17: stream(); ftAction_8007320C(g, &CI); break;
                }
            }
            hworld();
            ASRT_ARMED = 0;
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
