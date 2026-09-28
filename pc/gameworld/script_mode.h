#ifndef SCRIPT_MODE_H
#define SCRIPT_MODE_H

/* Opaque, per-source Lua state, not a Lua VM snapshot. Scalar words only cross
 * the retarget boundary. word 0 = length + 1 (0 = free), 1..2 = source hash. */
#define SCRIPT_MODE_SLOTS 16
#define SCRIPT_MODE_BYTES 8192
#define SCRIPT_MODE_WORDS (3 + SCRIPT_MODE_BYTES / 4)

#endif
