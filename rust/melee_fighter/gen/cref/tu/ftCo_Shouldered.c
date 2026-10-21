/* TU: src/melee/ft/kinds/ftCommon/ftCo_Shouldered.c */
#include TU_SRC
#include "tu_common.h"
#include "ftco_s209_fill.h"

static float K(int i) { return (float) ((int) (SRV[i & 15] % 41) - 20) / 4.0f; }
#define VS(name, id) void name(Fighter_GObj* g) { T(id); T(PI(g)); }
void Fighter_ChangeMotionState(Fighter_GObj* g, FtMotionId m, MotionFlags fl, float a, float b, float c, Fighter_GObj* o)
{ T(1); T(PI(g)); T(m); T(fl); T(B(a)); T(B(b)); T(B(c)); T(PI(o)); }
VS(ftCommon_8007E2FC, 2)
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(3); T(PI(f)); T((u32) (s32) v); }
VS(ftCo_800DB464, 4)
void ftCommon_InitGrab(Fighter* f, bool a, float v) { T(5); T(PI(f)); T((u32) a); T(B(v)); }
bool ftCommon_GrabMash(Fighter* f, float v) { T(6); T(PI(f)); T(B(v)); return SR(6); }
void ftCo_800DC920(Fighter_GObj* a, Fighter_GObj* b) { T(7); T(PI(a)); T(PI(b)); }
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(8); T(PI(f)); T(p); return (Fighter_Part) (SRV[3] % 4); }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* out) { T(9); T(PI(j)); T(PI(p0)); out->x = K(1); out->y = K(2); out->z = K(5); }
float ftColl_80079C70(Fighter* f, Fighter* v, HitCapsule* h, int n) { T(10); T(PI(f)); T(PI(v)); T(PI(h)); T((u32) n); return K(6); }
void ftColl_80078710(Fighter_GObj* a, Fighter_GObj* b, UNK_T c) { T(11); T(PI(a)); T(PI(b)); T(PI(c)); }
float ScriptGame_FighterDealtDamage(Fighter* f, float d) { T(12); T(PI(f)); T(B(d)); return d * 2.0f; }
float ScriptGame_HitRuleContact(HitCapsule* h, Fighter* f, float v, int kb) { T(13); T(PI(h)); T(PI(f)); T(B(v)); T((u32) kb); return v + K(7); }
void ScriptGame_HitRuleReport(Fighter* a, Fighter* v, HitCapsule* h, float d) { T(14); T(PI(a)); T(PI(v)); T(PI(h)); T(B(d)); }
void ScriptGame_HitRulePercentQueue(HitCapsule* h, Fighter* v, float d) { T(15); T(PI(h)); T(PI(v)); T(B(d)); }
void Fighter_UnkTakeDamage_8006CC30(Fighter* f, float d) { T(16); T(PI(f)); T(B(d)); }
void ftCo_Damage_CalcKnockback(Fighter* f) { T(17); T(PI(f)); }
void ftCo_8008E908(Fighter_GObj* g, float v) { T(18); T(PI(g)); T(B(v)); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(19); T(PI(g)); T(B(r)); }

#define NOPS 7
#define NCASE 1000

int main(void)
{
    int op, k;
    tu_init();
    for (op = 0; op < NOPS; op++) {
        for (k = 0; k < NCASE; k++) {
            u32 ret = 0;
            FtMotionId msid;
            tu_case(op, k);
            s209_fill();
            msid = (rnd() & 1) ? ftCo_MS_ShoulderedWait : (FtMotionId) (rnd() % 400);
            switch (op) {
            case 0: ftCo_8009C5A4(&GO, msid); break;
            case 1: ftCo_8009C640(&GO, msid); break;
            case 2: ftCo_8009C744(&GO); break;
            case 3: ftCo_Shouldered_Anim(&GO); break;
            case 4: ftCo_Shouldered_IASA(&GO); break;
            case 5: ftCo_Shouldered_Phys(&GO); break;
            case 6: ftCo_Shouldered_Coll(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
