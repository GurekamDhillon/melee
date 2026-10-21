/* TU: src/melee/ft/kinds/ftCommon/ftCo_Thrown.c */
/* C oracle for src/tu/ftCo_Thrown.rs: the whole TU included verbatim; every callee out of it
 * is a recording stub. Each line: op case ret state-hash trace. */
#include TU_SRC
#include "tu_common.h"
#include "s217_fill.h"

CMS_STUB
void ftCo_800DB368(Fighter* a, Fighter* b) { T(10); T(PI(a)); T(PI(b)); }
void ftColl_8007B62C(Fighter_GObj* g, enum_t n) { T(11); T(PI(g)); T((u32) n); }
int ftKb_MexKirbyLike(int k) { T(12); T((u32) k); return chk(12); }
void pl_80040614(int a, bool b, float f) { T(13); T((u32) a); T(b); T(B(f)); }
void ftCommon_8007E2F4(Fighter* f, s16 v) { T(14); T(PI(f)); T((u32) (s32) v); }
STUB_V(ftCommon_8007E358, 15)
STUB_V(ftAnim_8006EBA4, 16)
Fighter_Part ftParts_GetBoneIndex(Fighter* f, Fighter_Part p) { T(17); T(PI(f)); T(p); return (SRV[5] + p) % 8; }
void lb_8000B1CC(HSD_JObj* j, Vec3* p0, Vec3* out) { T(18); T(PI(j)); T(PI(p0)); out->x = rw(); out->y = rw(); out->z = rw(); }
void ftAnim_SetAnimRate(Fighter_GObj* g, float r) { T(19); T(PI(g)); T(B(r)); }
STUB_V(ftCommon_8007E3EC, 20)
bool ftCommon_GrabMash(Fighter* f, float v) { T(21); T(PI(f)); T(B(v)); return chk(21); }
STUB_CHK(fn_800DC044, 22)
STUB_V(fn_800DC070, 23)
STUB_V(ftCo_CaptureCut_Enter, 24)
STUB_V(ftCamera_800762F4, 25)
STUB_V(ftCamera_UpdateCameraBox, 26)
void ftColl_8007B8CC(Fighter* f, Fighter_GObj* v) { T(27); T(PI(f)); T(PI(v)); }
void ftCo_8008DCE0(Fighter_GObj* g, int a, float f) { T(28); T(PI(g)); T((u32) a); T(B(f)); }
void ftCo_8008E5A4(Fighter* f) { T(29); T(PI(f)); }
float ftColl_80079C70(Fighter* a, Fighter* b, HitCapsule* h, int n) { T(30); T(PI(a)); T(PI(b)); T(PI(h)); T((u32) n); return rw(); }
float ScriptGame_FighterDealtDamage(Fighter* f, float d) { T(31); T(PI(f)); T(B(d)); return d + 1.0f; }
float ScriptGame_HitRuleContact(HitCapsule* h, Fighter* f, float v, int kb) { T(32); T(PI(h)); T(PI(f)); T(B(v)); T((u32) kb); return v + (float) kb; }
void ScriptGame_HitRuleReport(Fighter* a, Fighter* b, HitCapsule* h, float d) { T(33); T(PI(a)); T(PI(b)); T(PI(h)); T(B(d)); }
void ScriptGame_HitRulePercentQueue(HitCapsule* h, Fighter* f, float d) { T(34); T(PI(h)); T(PI(f)); T(B(d)); }
void Fighter_UnkTakeDamage_8006CC30(Fighter* f, float d) { T(35); T(PI(f)); T(B(d)); }
void ftColl_8007891C(Fighter_GObj* a, Fighter_GObj* b, float d) { T(36); T(PI(a)); T(PI(b)); T(B(d)); }

int main(void)
{
    int op, k;
    tu_init();
    s_init();
    for (op = 0; op < 25; op++) {
        for (k = 0; k < 1000; k++) {
            u32 ret = 0;
            int ms;
            float sp;
            HSD_GObj* vg;
            int up;
            tu_case(op, k);
            s_fill();
            ms = 239 + (int) (rnd() % 6);
            sp = rg();
            vg = (rnd() & 1) ? &GO2 : NULL;
            up = rnd() & 1;
            switch (op) {
            case 0: F.victim_gobj = &GO2; ftCo_800DE3FC(&GO, ms, sp); break;
            case 1: ftCo_800DE508(&GO); break;
            case 2: ftCo_800DE5A4(&GO); break;
            case 3: ftCo_ThrownF_Anim(&GO); break;
            case 4: ftCo_ThrownF_IASA(&GO); break;
            case 5: ftCo_ThrownF_Phys(&GO); break;
            case 6: ftCo_ThrownF_Coll(&GO); break;
            case 7: ftCo_ThrownB_Anim(&GO); break;
            case 8: ftCo_ThrownB_IASA(&GO); break;
            case 9: ftCo_ThrownB_Phys(&GO); break;
            case 10: ftCo_ThrownB_Coll(&GO); break;
            case 11: ftCo_ThrownHi_Anim(&GO); break;
            case 12: ftCo_ThrownHi_IASA(&GO); break;
            case 13: ftCo_ThrownHi_Phys(&GO); break;
            case 14: ftCo_ThrownHi_Coll(&GO); break;
            case 15: F.victim_gobj = &GO2; ftCo_ThrownHi_Cam(&GO); break;
            case 16: ftCo_ThrownLw_Anim(&GO); break;
            case 17: ftCo_ThrownLw_IASA(&GO); break;
            case 18: ftCo_ThrownLw_Phys(&GO); break;
            case 19: ftCo_ThrownLw_Coll(&GO); break;
            case 20: fn_800DE798(&GO); break;
            case 21: ftCo_800DE7C0(&GO, vg, up); break;
            case 22: F.victim_gobj = &GO2; ftCo_800DE854(&GO); break;
            case 23: ftCo_800DE920(&GO, sp); break;
            case 24: ftCo_800DE974(&GO); break;
            }
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(), TR);
        }
    }
    return 0;
}
