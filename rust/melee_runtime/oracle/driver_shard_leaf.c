#include <stdio.h>
#include <stdint.h>
#include <string.h>

typedef float Mtx[3][4];
double shard_bits_to_double(uint64_t), shard_make_quiet(double), shard_force25(double);
uint64_t shard_double_to_bits(double);
double shard_infinity(int), shard_nan(void);
float shard_fmadds(float, float, float), shard_madd(float, float, float);
float shard_reduce(float, int*), shard_even(float), shard_odd(float);
void shard_rot(Mtx, char, float, float), PSMTXCopy(Mtx, Mtx), PSMTXIdentity(Mtx);

static uint64_t state = 0x6A09E667F3BCC909ull;
static uint64_t next(void) { state ^= state << 13; state ^= state >> 7; state ^= state << 17; return state; }
static uint32_t fb(float x) { uint32_t r; memcpy(&r, &x, 4); return r; }
static uint64_t db(double x) { uint64_t r; memcpy(&r, &x, 8); return r; }
static float bf(uint32_t x) { float r; memcpy(&r, &x, 4); return r; }
static double bd(uint64_t x) { double r; memcpy(&r, &x, 8); return r; }
static float finite(void) { return bf(((uint32_t)next() & 0x807fffffu) | ((100 + next() % 40) << 23)); }
static void matrix(FILE* f, Mtx m) { int i, j; for (i=0;i<3;i++) for(j=0;j<4;j++) fprintf(f," %08x",fb(m[i][j])); }

int main(int argc, char** argv) {
    static const uint64_t special[] = {0,0x8000000000000000ull,0x7ff0000000000000ull,0xfff0000000000000ull,
        0x7ff0000000000001ull,0xfff123456789abcdull,0x7ff8123456789abcull,1,0xffffffffffffffffull};
    FILE* out = fopen(argv[1], "w");
    int i, j, k;
    if (!out) return 1;
    for (i=0;i<2048;i++) {
        uint64_t bits = i < sizeof(special)/sizeof(special[0]) ? special[i] : next();
        double d = bd(bits);
        int negative = i % 3 - 1;
        float a=finite(), c=finite(), b=finite(), x=(int32_t)next() / 214748.0f, y=(int32_t)next() / 1073741824.0f;
        int quadrant;
        float reduced;
        Mtx src, dst, rot;
        unsigned char axis = "xXyYzZq?"[i%8];
        if (i == 0) { a=1.000244140625f; c=a; b=0x1p-60f; x=-0.0f; }
        if (i == 1) { a=1.0000001192092896f; c=0.9999999403953552f; b=-1.0f; x=0.0f; }
        fprintf(out,"bits %016llx %016llx %016llx %016llx %016llx %08x %016llx %016llx\n",
            (unsigned long long)bits,(unsigned long long)db(shard_bits_to_double(bits)),
            (unsigned long long)shard_double_to_bits(d),(unsigned long long)db(shard_make_quiet(d)),
            (unsigned long long)db(shard_force25(d)),(unsigned)negative,
            (unsigned long long)db(shard_infinity(negative)),(unsigned long long)db(shard_nan()));
        reduced=shard_reduce(x,&quadrant);
        fprintf(out,"trig %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x %08x\n",
            fb(a),fb(c),fb(b),fb(x),fb(y),fb(shard_fmadds(a,c,b)),fb(shard_madd(a,c,b)),
            fb(reduced),(unsigned)quadrant,fb(shard_even(y)),fb(shard_odd(y)));
        for(j=0;j<3;j++) for(k=0;k<4;k++) src[j][k]=bf((uint32_t)next());
        fprintf(out,"copy"); matrix(out,src);
        PSMTXCopy(src,dst); PSMTXCopy(dst,dst); matrix(out,dst);
        PSMTXIdentity(dst); matrix(out,dst); fprintf(out,"\n");
        for(j=0;j<3;j++) for(k=0;k<4;k++) rot[j][k]=finite();
        fprintf(out,"rot"); matrix(out,rot); fprintf(out," %02x %08x %08x",axis,fb(b),fb(y));
        shard_rot(rot,axis,b,y); matrix(out,rot); fprintf(out,"\n");
    }
    return fclose(out) != 0;
}
