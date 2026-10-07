/* gw_ui_retailtext.c - the retail SIS stream to UTF-8 (see the header). The walk follows the stream interpreter (HSD_SisLib_803A84BC,
 * src/sysdolphin/baselib/hsd_3A76.c): an item is an opcode byte below 0x20 with its operand bytes, or a 16-bit glyph code (high byte 0x20 or
 * 0x21 in the default atlas, so always 0x20 or more) read most significant byte first. The glyph code is the retail encoder's own table
 * index (HSD_SisLib_803A67EC): lut_glyph[2k] is the glyph of the SJIS pair lut_sjis[2k]. */
#include "gw_ui_retailtext.h"
#include <string.h>

/* operand bytes after each opcode, read from the interpreter's cases: 5 delay u16; 6 two u16; 7 line origin two s16; 8/9 jump pointer;
 * 10 scale two s16; 12 colour rgb; 14 size two u16; every other opcode below 27 takes none. 27 to 31 are not seen: -1 stops the walk. */
static const signed char OPERANDS[32] = {
    0, 0, 0, 0, 0, 2, 4, 4, 4, 4, 4, 0, 3, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, -1, -1
};

int at_sjis_to_utf8(unsigned hi, unsigned lo, char out[5])
{
    int c = 0;
    if (hi == 0x82 && lo >= 0x4F && lo <= 0x58) c = '0' + (int) (lo - 0x4F);
    else if (hi == 0x82 && lo >= 0x60 && lo <= 0x79) c = 'A' + (int) (lo - 0x60);
    else if (hi == 0x82 && lo >= 0x81 && lo <= 0x9A) c = 'a' + (int) (lo - 0x81);
    else if (hi == 0x81) {
        /* the full-width forms of ASCII punctuation: the first 33 glyphs of the font atlas and its bracket run (lbl_8040C8C0) */
        switch (lo) {
        case 0x40: c = ' '; break;  case 0x43: c = ','; break;  case 0x44: c = '.'; break;  case 0x46: c = ':'; break;
        case 0x47: c = ';'; break;  case 0x48: c = '?'; break;  case 0x49: c = '!'; break;  case 0x51: c = '_'; break;
        case 0x5E: c = '/'; break;  case 0x66: c = '\''; break; case 0x68: c = '"'; break;  case 0x69: c = '('; break;
        case 0x6A: c = ')'; break;  case 0x6D: c = '['; break;  case 0x6E: c = ']'; break;  case 0x6F: c = '{'; break;
        case 0x70: c = '}'; break;  case 0x7B: c = '+'; break;  case 0x7C: c = '-'; break;  case 0x81: c = '='; break;
        case 0x83: c = '<'; break;  case 0x84: c = '>'; break;  case 0x90: c = '$'; break;  case 0x93: c = '%'; break;
        case 0x94: c = '#'; break;  case 0x95: c = '&'; break;  case 0x96: c = '*'; break;  case 0x97: c = '@'; break;
        default: break;
        }
    }
    if (c == 0) { out[0] = '\0'; return 0; }
    out[0] = (char) c; out[1] = '\0';
    return 1;
}

int at_sis_decode(const unsigned char *lg, const unsigned char *ls, int npairs, const unsigned char *s, int len, char *out, int cap, AtSisStats *st)
{
    AtSisStats z; int i = 0, n = 0, steps = 0;
    memset(&z, 0, sizeof z);
    if (cap <= 0 || out == NULL) { if (st) *st = z; return 0; }
    out[0] = '\0';
    if (s == NULL || lg == NULL || ls == NULL || len <= 0) { if (st) *st = z; return 0; }
    while (steps++ < 1024) {
        unsigned op;
        if (i >= len) { z.bad++; break; }                                   /* ran out before a terminator */
        op = s[i];
        if (op >= 0x20) {
            unsigned g;
            if (i + 2 > len) { z.bad++; break; }                            /* a cut glyph */
            g = ((unsigned) s[i] << 8) | s[i + 1];
            char u[5]; int k, ok = 0;
            i += 2;
            for (k = 0; k < npairs; k++)
                if ((((unsigned) lg[2 * k] << 8) | lg[2 * k + 1]) == g) { ok = at_sjis_to_utf8(ls[2 * k], ls[2 * k + 1], u); break; }
            if (!ok) { u[0] = '?'; u[1] = '\0'; z.unknown++; }
            z.chars++;
            if (n + 1 < cap) { out[n++] = u[0]; out[n] = '\0'; }
            continue;
        }
        if (op == 0) break;
        if (op == 8 || op == 9) { z.jumps++; break; }
        if (OPERANDS[op] < 0) { z.bad++; break; }
        if (i + 1 + OPERANDS[op] > len) { z.bad++; break; }                 /* its operand bytes are cut */
        if (op == 3 && n + 1 < cap) { out[n++] = '\n'; out[n] = '\0'; z.chars++; }
        i += 1 + OPERANDS[op];
        z.controls++;
    }
    if (st) *st = z;
    return n;
}
