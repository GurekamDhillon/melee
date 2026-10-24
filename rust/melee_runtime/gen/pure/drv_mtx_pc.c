/* C oracle for melee_runtime::tu::mtx_pc (PSVECAdd, PSVECCrossProduct, PSMTXIdentity, PSMTXConcat,
 * MTXLightPerspective): pc/gameworld/mtx_pc.c included verbatim. One line per case: op case hash. */
#include "pc/gameworld/mtx_pc.c"
#include "common.h"
extern int printf(const char*, ...);

static void rm(Mtx m) { int i, j; for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) m[i][j] = rndf(); }
static void rv(Vec* v) { v->x = rndf(); v->y = rndf(); v->z = rndf(); }

int main(void)
{
    int k;
    for (k = 0; k < 400; k++) {
        Vec a, b, c; Mtx m1, m2, m3; u64_ h;
        rv(&a); rv(&b); rv(&c);
        PSVECAdd(&a, &b, &c);
        h = fnv(&c, sizeof c, FNV0);
        printf("add %d %016llx\n", k, h);
        rv(&a); rv(&b); rv(&c);
        PSVECCrossProduct(&a, &b, &c);
        h = fnv(&c, sizeof c, FNV0);
        printf("cross %d %016llx\n", k, h);
        /* aliased destination */
        rv(&a); rv(&b);
        PSVECCrossProduct(&a, &b, &a);
        h = fnv(&a, sizeof a, FNV0);
        printf("crossa %d %016llx\n", k, h);
        rm(m1);
        PSMTXIdentity(m1);
        h = fnv(m1, sizeof(Mtx), FNV0);
        printf("ident %d %016llx\n", k, h);
        rm(m1); rm(m2); rm(m3);
        PSMTXConcat(m1, m2, m3);
        h = fnv(m3, sizeof(Mtx), FNV0);
        printf("concat %d %016llx\n", k, h);
        rm(m1); rm(m2);
        PSMTXConcat(m1, m2, m1);
        h = fnv(m1, sizeof(Mtx), FNV0);
        printf("concat_a %d %016llx\n", k, h);
        {
            float fov = rndf() * 0.1f, asp = rndf(), ss = rndf(), st = rndf(), ts = rndf(), tt = rndf();
            rm(m1);
            MTXLightPerspective(m1, fov, asp, ss, st, ts, tt);
            h = fnv(m1, sizeof(Mtx), FNV0);
            printf("lightp %d %016llx\n", k, h);
        }
        {
            Vec v1, v2, v3; Quaternion q; f32 s1;
            rv(&v1); rv(&v2); rv(&v3);
            PSVECNormalize(&v1, &v3);
            h = fnv(&v3, sizeof v3, FNV0); printf("vnorm %d %016llx\n", k, h);
            rv(&v1);
            PSVECNormalize(&v1, &v1);
            h = fnv(&v1, sizeof v1, FNV0); printf("vnorma %d %016llx\n", k, h);
            rv(&v1);
            s1 = PSVECMag(&v1);
            h = fnv(&s1, sizeof s1, FNV0); printf("vmag %d %016llx\n", k, h);
            rv(&v1); rv(&v2);
            s1 = PSVECDotProduct(&v1, &v2);
            h = fnv(&s1, sizeof s1, FNV0); printf("vdot %d %016llx\n", k, h);
            rm(m1); rv(&v1); rv(&v2); rv(&v3);
            C_MTXLookAt(m1, &v1, &v2, &v3);
            h = fnv(m1, sizeof(Mtx), FNV0); printf("lookat %d %016llx\n", k, h);
            { f32 tx = rndf(), ty = rndf(), tz = rndf(); rm(m1);
            PSMTXTrans(m1, tx, ty, tz); }
            h = fnv(m1, sizeof(Mtx), FNV0); printf("trans %d %016llx\n", k, h);
            rm(m1); rm(m2);
            PSMTXTranspose(m1, m2);
            h = fnv(m2, sizeof(Mtx), FNV0); printf("transp %d %016llx\n", k, h);
            rm(m1);
            PSMTXTranspose(m1, m1);
            h = fnv(m1, sizeof(Mtx), FNV0); printf("transpa %d %016llx\n", k, h);
            rm(m1); q.x = rndf(); q.y = rndf(); q.z = rndf(); q.w = rndf();
            PSMTXQuat(m1, &q);
            h = fnv(m1, sizeof(Mtx), FNV0); printf("quat %d %016llx\n", k, h);
            {
                static const char ax[] = { 'x', 'X', 'y', 'Y', 'z', 'Z', 'q' };
                char a = ax[rnd() % 7]; f32 sn = rndf(), cs = rndf();
                rm(m1);
                mtx_rot_trig(m1, a, sn, cs);
                h = fnv(m1, sizeof(Mtx), FNV0); printf("rottrig %d %016llx\n", k, h);
            }
            {
                f32 t = rndf(), b = rndf(), l = rndf(), r = rndf(), ss = rndf(), st = rndf(), ts = rndf(), tt = rndf();
                rm(m1);
                MTXLightOrtho(m1, t, b, l, r, ss, st, ts, tt);
                h = fnv(m1, sizeof(Mtx), FNV0); printf("lighto %d %016llx\n", k, h);
            }
        }
    }
    return 0;
}
