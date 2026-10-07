/* gw_ui_retailtext.h - decode a retail SIS string (opcodes plus 16-bit glyph codes) to UTF-8, using the font's own SJIS lookup run in
 * reverse. Pure C. The game side passes the two retail tables as arguments, so the host never links the game's data, and the decoded text is
 * only ever put into the caller's bounded buffer: it is never written to a file or a log (the game side logs the counts in AtSisStats). */
#ifndef GW_UI_RETAILTEXT_H
#define GW_UI_RETAILTEXT_H
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { int chars, unknown, controls, jumps, bad; } AtSisStats;
/* lut_glyph[2k..2k+1] is the glyph code of lut_sjis[2k..2k+1] (the pair of HSD_SisLib_8040C680 and lbl_8040C8C0, npairs of them), both
 * big-endian byte pairs. Returns the characters written (a newline counts); out is always terminated. Never reads past a terminating 0
 * opcode, a jump or an unknown opcode; at most 1024 stream items are walked. len is the number of readable stream bytes (the rest of the archive's
 * data from the string's start): every operand skip and glyph read is checked against it, and a stream that runs out before its terminator, or whose
 * last opcode or glyph is cut, is bad (st->bad), never read past. A glyph that is not in the table, or is outside the Latin
 * subset, becomes '?' and is counted in st->unknown. s NULL decodes nothing. */
int at_sis_decode(const unsigned char *lut_glyph, const unsigned char *lut_sjis, int npairs, const unsigned char *s, int len, char *out, int cap, AtSisStats *st);
/* the Latin subset the font atlas holds (digits, letters, the ASCII punctuation as full-width SJIS): 1 and out[0] set for ASCII;
 * 0 and out[0] == 0 when the code is outside it (kana, kanji) */
int at_sjis_to_utf8(unsigned hi, unsigned lo, char out[5]);
#ifdef __cplusplus
}
#endif
#endif
