/* TU: src/melee/ft/ftcoll.c */
/* C oracle for src/tu/ftcoll.rs (mirrored by tests/tu_ftcoll.rs). Ops: 0 ftColl_80079AB0,
 * 1 ftColl_80079EA8, 2 ftColl_80079C70, 3 ftColl_80076528 (combo push), 4 ftColl_800765F0,
 * 5 ftColl_80076640, 6 ftColl_8007AC68, 7 ftColl_8007A06C over a 3-entry log (kinds 1 and 3),
 * 8 ftColl_8007699C (clank), 9 ftColl_80076CBC (shield hit), 10 ftColl_80076ED8 (hurtbox hit).
 * Recording stubs for the callees; the TARGET_PC echo layer is the identity (4 capsules =
 * fp->x914). Each line: op case hash trace. */
#include TU_SRC
#include "tu_common.h"

static float RV[8];
static u32 RVI;
static float rv(void) { return RV[(RVI++) & 7]; }

/* ---- TARGET_PC echo identity ---- */
int ScriptGame_EchoCapsuleCount(Fighter* f) { return 4; }
HitCapsule* ScriptGame_EchoCapsule(Fighter* f, int i) { return &f->x914[i]; }
int ScriptGame_EchoIsCapsule(HitCapsule* h) { return 0; }
int ScriptGame_EchoBlocked(HitCapsule* h, Fighter* f) { return 0; }
int ScriptGame_EchoSameGroup(HitCapsule* a, HitCapsule* b) { return a->x4 == b->x4; }
int ScriptGame_EchoTargetAllowed(Fighter* s, Fighter* t) { return 0; }
void ScriptGame_EchoConnected(HitCapsule* h, Fighter* f) { T(40); }
void ScriptGame_EchoPrepare(void) { }
int ScriptGame_EchoHitIndex(Fighter* f, HitCapsule* h) { T(41); return (int) (h - f->x914); }
int ScriptGame_EchoReportContext(Fighter* a, Fighter* v, HitCapsule* h) { T(42); return 1; }
int ScriptGame_EchoCredit(Fighter* a, Fighter* v, HitCapsule* h, float d) { T(43); T(B(d)); return SR(4); }
int ScriptGame_EchoSourceX(HitCapsule* h, int fb) { return fb; }
void Script_GameEvent(int a, int b, int c, int d, int e) { T(44); T(a); T(c); T(d); T(e); }
void ScriptGame_SkillPerfectShield(Fighter* f, Fighter* a, int k, float d) { T(45); }
void ScriptGame_IntrWinHit(Fighter* f, int k) { T(46); T(k); }
void Geno_ComboContact(Fighter* f, int k) { T(47); T(k); }
float ScriptGame_HitRuleContact(HitCapsule* h, Fighter* f, float v, int k) { T(48); T(k); T(B(v)); return v; }
float ScriptGame_FighterDealtDamage(Fighter* f, float d) { T(49); return d; }
void ScriptGame_HitRuleWon(HitCapsule* h, Fighter* f) { T(50); }
void Geno_HitStunBonus(Fighter* a, int i, Fighter* v) { T(51); T(i); }
int Geno_HitFlags(Fighter* a, int i, Fighter* v) { T(52); return SR(5) ? 4 : 0; }
void ScriptGame_HitRulePercentQueue(HitCapsule* h, Fighter* f, float d) { T(53); T(B(d)); }
void ScriptGame_CritHit(Fighter* a, HitCapsule* h, Fighter* v, float d, int k) { T(54); }
void ScriptGame_ReportHitContext(Fighter* a, Fighter* v, int el, int k) { T(55); T(el); }
void ScriptGame_SkillHit(Fighter* a, Fighter* v, float d) { T(56); }
void ScriptGame_HitRuleContext(HitCapsule* h, Fighter* f) { T(57); }
int Script_ClankWanted(void) { T(58); return SR(6); }
void Script_Clank(int a, int b, int c, int d, int e, int f, int g, int h, int i, int j, int k, int l)
{ T(59); T(a); T(b); T(e); T(f); T(g); T(h); T(i); T(j); }
void ScriptMods_Clank(int a, int b, float x, float y) { T(60); T(B(x)); T(B(y)); }
int Geno_Autolink(Fighter* a, HitCapsule* h, float* dir, float* angle, float* kb) { T(61); return 0; }
int Mex_Enabled(const char* n) { return 0; }
float ScriptGame_ModValue(int slot, int field, float base) { return base; }

/* ---- game callees ---- */
float Player_GetDefenseRatio(int slot) { T(70); return rv(); }
float Player_GetAttackRatio(int slot) { T(71); return rv(); }
float gm_8016B248(void) { T(72); return rv(); }
bool lbColl_80008688(HitCapsule* h, int t, void* v) { T(73); T(t); return SR(7); }
bool lbColl_80008820(HitCapsule* h, int t, void* v) { T(74); T(t); return SR(8); }
void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { T(75); T(id); return NULL; }
void ftCo_80094138(Fighter* f) { T(76); }
bool ftCo_800BFFD0(Fighter* f, enum_t a, bool b) { T(77); T(a); return 0; }
void ft_PlaySFX(Fighter* f, enum_t id, u8 v, u8 p) { T(78); T(id); }
void pl_8003E150(int a, int b) { T(79); }
void plStale_UpdateStaleMovesFromFighter(HSD_GObj* a, HSD_GObj* b) { T(80); }
void pl_8003EB30(float d, int a, int b, int c, int e, int f) { T(81); T(B(d)); }
void lbColl_80008D30(HitCapsule* h, lbColl_80008D30_arg1* a)
{ T(82); h->x24 = a->x0 * 3; h->x28 = (u32) a->x0 & 3; h->x2C = (u32) a->x0 % 7; h->unk_count = (u32) a->x0 % 5; }
void pl_80038144(HSD_GObj* a, HSD_GObj* v, s32 i, ft_800898B4_t* p, u16 inst, s32 gr, s32 prev) { T(83); T(i); T(gr); }
HSD_GObj* itGetOwner(Item_GObj* g) { T(84); return NULL; }
s32 itGetAttackId(Item_GObj* g) { return 0; }
bool ftLib_80086960(HSD_GObj* g) { return 0; }

static HitCapsule* hit_rand(HitCapsule* h)
{
    zero(h, sizeof *h);
    h->state = rnd() % 3;
    h->x4 = rnd() & 1;
    h->unk_count = rnd() % 20;
    h->damage = (rnd() & 1) ? rw() : (float) (rnd() % 30) + rg();
    h->x24 = rnd() % 200;
    h->x28 = (rnd() & 1) ? 0 : rnd() % 150;
    h->x2C = rnd() % 100;
    h->kb_angle = (rnd() % 4 == 0) ? 0x16A : rnd() % 400;
    h->element = rnd() % 17;
    h->x34 = (int) (rnd() % 20) - 5;
    h->sfx_severity = rnd() % 3;
    h->x40_b1 = rnd() & 1;
    h->hurt_coll_pos.x = rw(); h->hurt_coll_pos.y = rw(); h->hurt_coll_pos.z = rw();
    h->coll_distance = rw();
    return h;
}

static void cd_rand(void)
{
    CD.xF4 = rw(); CD.xF8 = rw(); CD.x108 = (rnd() & 1) ? 100000.0f : rw();
    CD.x110 = rw(); CD.x114 = rw(); CD.x118 = rw(); CD.x11C = rw(); CD.x120 = rw();
    CD.x6D4 = (int) (rnd() % 300); CD.x6D8[0] = (int) (rnd() % 300);
    CD.x128 = rw(); CD.x714 = rw(); CD.x3CC = (int) (rnd() % 10);
    CD.x3D0 = rw(); CD.x3D4 = rw(); CD.x4C8 = (int) (rnd() % 4); CD.x4D0 = rw(); CD.x4D4 = rw();
    CD.x7A8 = rw(); CD.unk_kb_angle_min = rnd() % 400; CD.unk_kb_angle_max = rnd() % 400;
    CD.x1A4 = rw();
}

static void fp_rand(Fighter* f, HSD_GObj* g)
{
    f->gobj = g;
    f->player_id = rnd() % 4;
    f->x2225_b7 = rnd() & 1; f->x2224_b2 = rnd() & 1;
    f->dmg.x1830_percent = rw(); f->dmg.x1838_percentTemp = rw();
    f->dmg.x182c_behavior = rw(); f->dmg.x1834 = rw();
    f->dmg.x183C_applied = (int) (rnd() % 20);
    f->dmg.int_value = (int) (rnd() % 20);
    f->dmg.x1924 = (int) (rnd() % 20);
    f->co_attrs.weight = rw();
    f->facing_dir = rg();
    f->cur_pos.x = rw(); f->cur_pos.y = rw();
    f->coll_data.floor.normal.x = rw(); f->coll_data.floor.normal.y = rw();
    f->x2090 = rnd() % 4; f->x2092 = rnd() % 3;
    f->ground_or_air = rnd() & 1;
    f->motion_id = (rnd() & 1) ? ftCo_MS_DamageIce : 0;
    f->x221B_b5 = rnd() & 1; f->x221C_b4 = rnd() & 1; f->x221C_b2 = rnd() & 1;
    f->lightshield_amount = rw(); f->shield_unk0 = (rnd() & 1) ? 0.0f : rw();
    f->x19A4 = (int) (rnd() % 20);
    f->x1988 = (rnd() % 4 == 0); f->x198C = 0;
    f->kind = rnd() % 30;
}

static void hf(float f) { HM(B(f)); }

int main(void)
{
    int op, k, i;
    static HitCapsule H0, H1;
    static DmgLogEntry LOG[3];
    static lbColl_80008D30_arg1 ENV[3];
    static FighterHurtCapsule HURT[3];
    static float OUT[16];
    tu_init();
    for (op = 0; op < 11; op++) {
        for (k = 0; k < 2000; k++) {
            float r = 0;
            tu_case(op, k);
            for (i = 0; i < 8; i++) RV[i] = rw();
            RVI = 0;
            cd_rand();
            fp_rand(&F, &GO);
            fp_rand(&F2, &GO2);
            hit_rand(&H0); hit_rand(&H1);
            for (i = 0; i < 4; i++) { hit_rand(&F.x914[i]); hit_rand(&F2.x914[i]); }
            dmg_log0_idx = rnd() % 2; dmg_log1_idx = 0;
            H = 0x811C9DC5u;
            switch (op) {
            case 0: r = ftColl_80079AB0(&F, &H0, rnd() % 20, rw(), rw(), rw(), rw()); break;
            case 1: r = ftColl_80079EA8(&F, &H0, rnd() % 20); break;
            case 2: r = ftColl_80079C70(&F, &F2, &H0, (int) (rnd() % 20)); break;
            case 3: ftColl_80076528(&GO); break;
            case 4: r = ftColl_800765F0(&F, (rnd() & 1) ? &GO : &GO2, rw()); F.victim_gobj = &GO2; r += ftColl_800765F0(&F, &GO, rw()); break;
            case 5: { float d = rw(); T(ftColl_80076640(&F, &d)); r = d; break; }
            case 6: T(ftColl_8007AC68(rnd() % 400)); break;
            case 7: {
                for (i = 0; i < 3; i++) {
                    zero(&LOG[i], sizeof LOG[i]);
                    LOG[i].x0 = (rnd() & 1) ? 1 : 3;
                    LOG[i].gobj = &GO2;
                    LOG[i].kind = rnd() % 30;
                    HURT[i].capsule.a_pos.x = rw(); HURT[i].capsule.a_pos.y = rw();
                    HURT[i].capsule.b_pos.x = rw(); HURT[i].capsule.b_pos.y = rw();
                    HURT[i].height = rnd() % 3;
                    LOG[i].hurt1 = &HURT[i];
                    if (LOG[i].x0 == 1) {
                        LOG[i].hit0 = &F2.x914[i];
                    } else {
                        ENV[i].x0 = (int) (rnd() % 100);
                        ENV[i].kb_angle = (rnd() & 1) ? 0x16A : rnd() % 360;
                        ENV[i].element = rnd() % 17;
                        LOG[i].unk_anim0 = (DynamicsDesc*) &ENV[i];
                    }
                    LOG[i].pos.x = rw(); LOG[i].pos.y = rw();
                    LOG[i].x20 = rw();
                    LOG[i].size_of_xC = rnd() % 20;
                }
                zero(OUT, sizeof OUT);
                ftColl_8007A06C(&GO, OUT, LOG, 1 + rnd() % 3, (int) (rnd() & 1));
                for (i = 0; i < 9; i++) HM(((u32*) OUT)[i]);
                HM(F.x221C_b0); hf(F.x1960_vibrateMult);
                break;
            }
            case 8: T(ftColl_8007699C(&F, &F.x914[rnd() % 4], &F2, &F2.x914[rnd() % 4])); break;
            case 9: ftColl_80076CBC(&F, &F.x914[rnd() % 4], &F2); break;
            case 10: { HitCapsule* hh = &F.x914[rnd() % 4]; T(ftColl_80076ED8(&F, hh, &F2, &H1)); break; }
            }
            hf(r);
            hf(F.cur_pos.x); hf(F.cur_pos.y); HM(F.x2092);
            HM(F.x221C_b4); hf(F.dmg.x1834); hf(F.dmg.x1838_percentTemp); HM(F.dmg.x183C_applied);
            HM(F.dmg.int_value); hf(F.dmg.x191C); hf(F.dmg.facing_dir); HM(F.dmg.x1914);
            HM(F.dmg.x1924); hf(F.dmg.x1928); hf(F.dmg.x192c); hf(F.x1964);
            HM(F2.dmg.int_value); hf(F2.dmg.x191C); hf(F2.dmg.facing_dir); HM(F2.x19A4);
            hf(F2.specialn_facing_dir); HM(F2.x19B0); HM(F2.x19A0_shieldDamageTaken); hf(F2.x1964);
            HM(F2.dmg.x1840); hf(F2.dmg.x1834); hf(F2.dmg.x1838_percentTemp); HM(F2.dmg.x183C_applied);
            HM(F2.x221C_b4);
            HM(dmg_log0_idx); HM(dmg_log1_idx);
            for (i = 0; i < 44; i++) HM((u8) ftColl_804D6560[i]);
            for (i = 0; i < dmg_log0_idx && i < 4; i++) { HM(dmg_log0[i].x0); hf(dmg_log0[i].x20); HM((u32) dmg_log0[i].size_of_xC); }
            for (i = 0; i < dmg_log1_idx && i < 4; i++) { HM(dmg_log1[i].x0); hf(dmg_log1[i].x20); HM((u32) dmg_log1[i].size_of_xC); }
            printf("%d %d %08X %08X\n", op, k, H, TR);
        }
    }
    return 0;
}
