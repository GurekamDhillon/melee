/* Oracle driver for cobj.c's logic half (see build_cobj.sh, helper_cobj.c). Generated inputs only;
 * writes rows that tests/oracle_cobj.rs replays. */
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
static uint64_t st = 0xD1B54A32D192ED03ull;
static FILE* o;
unsigned g_scale_bits;
static uint64_t nx(void) { st ^= st << 13; st ^= st >> 7; st ^= st << 17; return st; }
unsigned drv_rand(void) { return (unsigned) (nx() >> 33); }
float drv_randf(float r) { return (float) ((double) (nx() >> 11) / 9007199254740992.0 * 2.0 - 1.0) * r; }
unsigned drv_rword(void) { return (unsigned) (nx() >> 32); }
void drv_tag(const char* t) { fprintf(o, "%s ", t); }
static int hashing; static unsigned hsh;
void drv_u32(unsigned v)
{
    int i;
    if (!hashing) { fprintf(o, "%08x ", v); return; }
    for (i = 0; i < 4; i++) { hsh ^= (v >> (8 * i)) & 0xFF; hsh *= 16777619u; }
}
/* FNV-1a over the little-endian bytes of the words written between begin and end. */
void drv_hash_begin(void) { hashing = getenv("COBJ_FULL") ? 0 : 1; hsh = 2166136261u; }
void drv_hash_end(void) { if (hashing) fprintf(o, "%08x ", hsh); hashing = 0; }
void drv_eol(void) { fputc('\n', o); }
void* orc_alloc(unsigned n) { return calloc(1, n); }
void orc_cobj_run(int nscen, int nops);
int main(int argc, char** argv)
{
    o = fopen(argv[1], "w");
    if (!o) return 1;
    orc_cobj_run(500, 60);
    fclose(o);
    return 0;
}
