/* atlas-retailtext: the retail SIS text stream (opcodes plus 16-bit glyph codes) decoded to UTF-8 through the font's own SJIS lookup run in
 * reverse. Every table and string here is INVENTED (no disc text): the lookup pairs use the glyph and SJIS codes the retail encoder itself
 * writes (HSD_SisLib_803A67EC, hsd_3A64.c). */
#include "atlas_check.h"
#include "../platform/gw_ui_retailtext.h"

/* invented glyph codes for "A", "b", "7" and the space; SJIS pairs as the retail encoder writes them */
static const unsigned char GLYPH[8] = { 0x20, 0x41, 0x20, 0x62, 0x20, 0x37, 0x20, 0x00 };
static const unsigned char SJIS[8]  = { 0x82, 0x60, 0x82, 0x82, 0x82, 0x56, 0x81, 0x40 };

int main(void)
{
    char out[64]; AtSisStats st;
    /* opcode 10 (+4 operand bytes) then glyphs then the end opcode, as the encoder lays a line out */
    const unsigned char s1[] = { 10, 0, 0, 0, 0, 0x20, 0x41, 0x20, 0x62, 0x20, 0x00, 0x20, 0x37, 0 };
    CHECK(at_sis_decode(GLYPH, SJIS, 4, s1, out, sizeof out, &st) == 4);
    CHECK_STR(out, "Ab 7");
    CHECK(st.chars == 4 && st.unknown == 0 && st.controls == 1 && st.bad == 0);

    /* an unknown glyph becomes '?' and is counted; the text still comes out */
    { const unsigned char s[] = { 0x20, 0x41, 0x2F, 0xFF, 0x20, 0x62, 0 };
      CHECK(at_sis_decode(GLYPH, SJIS, 4, s, out, sizeof out, &st) == 3);
      CHECK_STR(out, "A?b"); CHECK(st.unknown == 1); }

    /* a glyph that is in the table but outside the Latin subset (kana) is unknown too */
    { const unsigned char kana_glyph[2] = { 0x20, 0x77 }, kana_sjis[2] = { 0x83, 0x41 };
      const unsigned char s[] = { 0x20, 0x77, 0 };
      CHECK(at_sis_decode(kana_glyph, kana_sjis, 1, s, out, sizeof out, &st) == 1);
      CHECK_STR(out, "?"); CHECK(st.unknown == 1); }

    /* newline (opcode 3) becomes a newline; colour (12, +3) and scale (14, +4) are skipped */
    { const unsigned char s[] = { 12, 255, 0, 0, 0x20, 0x41, 3, 14, 1, 0, 1, 0, 0x20, 0x62, 0 };
      CHECK(at_sis_decode(GLYPH, SJIS, 4, s, out, sizeof out, &st) == 3);
      CHECK_STR(out, "A\nb"); CHECK(st.controls == 3); }

    /* a delay (5, +2), a line origin (7, +4) and an alignment (16) are skipped with their operands */
    { const unsigned char s[] = { 5, 0, 9, 7, 0, 0, 0, 0, 16, 0x20, 0x41, 0 };
      CHECK(at_sis_decode(GLYPH, SJIS, 4, s, out, sizeof out, &st) == 1);
      CHECK_STR(out, "A"); CHECK(st.controls == 3 && st.bad == 0); }

    /* a jump (8, 9) is not followed: the walk stops and says so; an unknown opcode stops it and says so */
    { const unsigned char j[] = { 0x20, 0x41, 8, 0, 0, 0, 0, 0 };
      CHECK(at_sis_decode(GLYPH, SJIS, 4, j, out, sizeof out, &st) == 1); CHECK(st.jumps == 1);
      { const unsigned char u[] = { 0x20, 0x41, 29, 0x20, 0x62, 0 };
        CHECK(at_sis_decode(GLYPH, SJIS, 4, u, out, sizeof out, &st) == 1); CHECK(st.bad == 1); } }

    /* output never overruns: a 3-byte buffer holds 2 characters and a terminator */
    { char tiny[3]; const unsigned char s[] = { 0x20, 0x41, 0x20, 0x62, 0x20, 0x37, 0 };
      CHECK(at_sis_decode(GLYPH, SJIS, 4, s, tiny, sizeof tiny, &st) == 2); CHECK_STR(tiny, "Ab"); }

    /* a stream that never ends is cut at the step cap, not walked off the end of memory (1100 colour codes, then a terminator) */
    { static unsigned char endless[1100 * 4 + 1]; int i;
      for (i = 0; i < 1100; i++) { endless[i * 4] = 12; }
      endless[1100 * 4] = 0;
      CHECK(at_sis_decode(GLYPH, SJIS, 4, endless, out, sizeof out, &st) == 0); CHECK(st.controls <= 1024); }

    /* NULL stream and a zero-size buffer are harmless */
    CHECK(at_sis_decode(GLYPH, SJIS, 4, NULL, out, sizeof out, &st) == 0 && out[0] == '\0');
    CHECK(at_sis_decode(GLYPH, SJIS, 4, s1, out, 0, &st) == 0);

    /* the SJIS subset the retail encoder itself emits */
    { char u[5];
      CHECK(at_sjis_to_utf8(0x82, 0x4F, u) == 1 && u[0] == '0');   /* digits: 0x82, c + 0x1F */
      CHECK(at_sjis_to_utf8(0x82, 0x58, u) == 1 && u[0] == '9');
      CHECK(at_sjis_to_utf8(0x82, 0x60, u) == 1 && u[0] == 'A');   /* capitals */
      CHECK(at_sjis_to_utf8(0x82, 0x79, u) == 1 && u[0] == 'Z');
      CHECK(at_sjis_to_utf8(0x82, 0x81, u) == 1 && u[0] == 'a');   /* lower case: c + 0x20 */
      CHECK(at_sjis_to_utf8(0x82, 0x9A, u) == 1 && u[0] == 'z');
      CHECK(at_sjis_to_utf8(0x81, 0x40, u) == 1 && u[0] == ' ');
      CHECK(at_sjis_to_utf8(0x81, 0x46, u) == 1 && u[0] == ':');
      CHECK(at_sjis_to_utf8(0x81, 0x7C, u) == 1 && u[0] == '-');
      CHECK(at_sjis_to_utf8(0x83, 0x41, u) == 0);                  /* kana: not in the Latin subset */
      CHECK(u[0] == '\0'); }

    /* the rest of the Latin punctuation the font atlas holds (its first 33 glyphs and the bracket run), read from the table's own order */
    { static const struct { unsigned hi, lo; char c; } P[] = {
          { 0x81, 0x49, '!' }, { 0x81, 0x68, '"' }, { 0x81, 0x94, '#' }, { 0x81, 0x90, '$' }, { 0x81, 0x93, '%' }, { 0x81, 0x95, '&' },
          { 0x81, 0x66, '\'' }, { 0x81, 0x69, '(' }, { 0x81, 0x6A, ')' }, { 0x81, 0x96, '*' }, { 0x81, 0x7B, '+' }, { 0x81, 0x43, ',' },
          { 0x81, 0x44, '.' }, { 0x81, 0x5E, '/' }, { 0x81, 0x47, ';' }, { 0x81, 0x83, '<' }, { 0x81, 0x81, '=' }, { 0x81, 0x84, '>' },
          { 0x81, 0x48, '?' }, { 0x81, 0x97, '@' }, { 0x81, 0x6D, '[' }, { 0x81, 0x6E, ']' }, { 0x81, 0x51, '_' }, { 0x81, 0x6F, '{' },
          { 0x81, 0x70, '}' } };
      char u[5]; unsigned k;
      for (k = 0; k < sizeof P / sizeof P[0]; k++) { CHECK(at_sjis_to_utf8(P[k].hi, P[k].lo, u) == 1); CHECK(u[0] == P[k].c); CHECK(u[1] == '\0'); } }

    /* the glyph count of a stream's characters excludes controls; stats may be NULL */
    CHECK(at_sis_decode(GLYPH, SJIS, 4, s1, out, sizeof out, NULL) == 4);
    ATLAS_DONE("atlas retailtext");
}
