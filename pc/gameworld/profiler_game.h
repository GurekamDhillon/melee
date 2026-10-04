/* Scalar-only diagnostics. No timestamps or native pointers enter game state. */
#ifndef PROFILER_GAME_H
#define PROFILER_GAME_H
#include "../platform/gw_profiler.h"
#if defined(TARGET_PC)
extern void ProfBegin(int id, int detail);
extern void ProfEnd(void);
extern void ProfCounter(int id, int value);
extern int ProfEnabled(void);
#define PC_PROF_BEGIN(id, detail) do { if (ProfEnabled()) ProfBegin((id), (detail)); } while (0)
#define PC_PROF_END() ProfEnd()
#else
#define PC_PROF_BEGIN(id, detail) ((void) 0)
#define PC_PROF_END() ((void) 0)
#endif
#endif
