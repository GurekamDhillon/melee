/* C reference for src/stale.rs: plstale.c and plattack.c compiled whole, the staling tail of
 * ft_0881.c and the two player.c accessors cut verbatim by run.py, driven by random op
 * sequences. tests/stale_ref.rs repeats the sequences and compares each printed line. */
#include "pl/plstale.c"
#include "pl/plattack.c"
#include <melee/it/types.h>
#include <melee/db/db.h>

StaticPlayer player_slots[6];
float* Fighter_804D6548;
DbLKind DbLevel;
static int g_owner_is_fighter;
bool ftLib_80086960(HSD_GObj* gobj) { (void) gobj; return g_owner_is_fighter; }
/* Player_CheckSlot's report (never reached: slots are 0..5) */
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* fmt, ...) { (void) fmt; }

#include "stale_extract.inc"

int printf(const char*, ...);
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void) { return (float) (int) (rnd() % 2001) / 9000.0f; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void fill(void* p, unsigned n, unsigned char v) { unsigned char* b = p; while (n--) *b++ = v; }

static Fighter fp;
static Item it;
static float dec[9];

int main(void)
{
    int k, step, i;
    HSD_GObj fg, ig;
    fill(&fg, sizeof fg, 0); fill(&ig, sizeof ig, 0);
    fg.user_data = &fp; ig.user_data = &it;
    for (k = 0; k < 300; k++) {
        fill(player_slots, sizeof player_slots, 0);
        fill(&fp, sizeof fp, 0); fill(&it, sizeof it, 0);
        rs = 0xC0FFEEu + (u32) k * 2654435761u;
        staleAttackInstance = (u16) rnd();
        unk_804D6480 = (u16) rnd();
        for (i = 0; i < 9; i++) dec[i] = rf();
        Fighter_804D6548 = dec;
        DbLevel = rnd() % 8 == 0 ? 3 : 0;
        for (step = 0; step < 40; step++) {
            int slot = (int) (rnd() % 6);
            u32 op = rnd() % 11;
            u32 ret = 0;
            StaleMoveTable* t;
            fp.player_id = (u8) slot;
            switch (op) {
            case 0: plStale_InitAttackInstance(); break;
            case 1: plStale_ResetStaleMoveTableForPlayer(slot); break;
            case 2: ret = plStale_IncrementAttackInstance(); break;
            case 3: {
                int same = rnd() % 4 == 0;
                fp.x2068_attackID = (s32) (rnd() % 8);
                fp.x206C_attack_instance = (u16) (rnd() % 5);
                plStale_UpdateStaleMovesFromFighter(&fg, same ? &fg : NULL);
            } break;
            case 4: {
                g_owner_is_fighter = (int) (rnd() & 1);
                it.owner = &fg;
                it.xD88_attackID = (s32) (rnd() % 8);
                it.xD8C_attack_instance = (u16) (rnd() % 5);
                plStale_UpdateStaleMovesFromItem(&ig, NULL);
            } break;
            case 5: ft_800890D0(&fp, rnd() % 8); break;
            case 6: ft_800890BC(&fp); break;
            case 7: ft_800892A0(&fg); break;
            case 8: {
                int id = (int) (rnd() % 8);
                float dmg = rf() * 30.0f;
                ret = B(ft_80089228(&fp, id, 0, dmg));
            } break;
            case 9: ret = plAttack_80037B08(); break;
            case 10: {
                plActionStats* s = Player_GetActionStats(slot);
                fill(s, sizeof *s, 0xFF);
                plAttack_8003759C((u32) slot);
                ret = s->attacks.total + s->hits.by_attack_counts[5] + s->x358_hits.thrown_item_count +
                      s->x504[3] + s->x598[7] + s->x5B8[1] + (s->x5BC_b0 << 3 | s->x5BC_b1 << 2 | s->x5BC_b2 << 1 | s->x5BC_b3);
            } break;
            }
            t = Player_GetStaleMoveTableIndexPtr(slot);
            printf("%d %d %u %u %08X %d", k, step, op, slot, ret, t->current_index);
            for (i = 0; i < 10; i++) printf(" %u:%u", t->StaleMoves[i].move_id, t->StaleMoves[i].attack_instance);
            printf(" %u %u %d %u\n", staleAttackInstance, unk_804D6480, fp.x2068_attackID, fp.x206C_attack_instance);
        }
    }
    return 0;
}
