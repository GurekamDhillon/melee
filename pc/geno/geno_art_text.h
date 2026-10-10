#ifndef GENO_ART_TEXT_H
#define GENO_ART_TEXT_H
/* The first two words, or two letters of a single word. UI only. */
static void genoArt_Initials(const char* name, char out[3])
{
    int i, n = 0, start = 1;
    char second = 0;
    for (i = 0; name[i]; ++i) {
        char c = name[i];
        int alnum = (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') ||
                    (c >= '0' && c <= '9');
        if (c >= 'a' && c <= 'z') c -= 'a' - 'A';
        if (alnum && start && n < 2) out[n++] = c;
        else if (alnum && n == 1 && second == 0) second = c;
        start = !alnum;
    }
    if (n == 1 && second) out[n++] = second;
    if (n == 0) out[n++] = '?';
    out[n] = '\0';
}
#endif
