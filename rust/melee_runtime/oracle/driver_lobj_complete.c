#include <stdio.h>
#include <stdlib.h>
static FILE* f;
void out(unsigned x)
{
    fprintf(f, "%08x ", x);
}
void* alloc(unsigned n)
{
    return calloc(1, n);
}
void complete_case(unsigned);
void __assert(const char* f, int n, const char* m)
{
    fprintf(stderr, "%s:%d %s", f, n, m);
    exit(3);
}
void HSD_Panic(const char* f, int n, const char* m)
{
    __assert(f, n, m);
}
void OSReport(const char* m, ...)
{
    (void) m;
}
int main(int argc, char** argv)
{
    unsigned i;
    f = fopen(argv[1], "w");
    for (i = 0; i < 32; i++) {
        complete_case(i);
        fputc('\n', f);
    }
    fclose(f);
    return 0;
}
