/* Tiny check macros shared by the atlas-* native tests (no framework, no game). */
#ifndef ATLAS_CHECK_H
#define ATLAS_CHECK_H
#include <math.h>
#include <stdio.h>
#include <string.h>

static int at_t_fails, at_t_count;
#define CHECK(cond) do { at_t_count++; if (!(cond)) { at_t_fails++; printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond); } } while (0)
#define CHECK_NEAR(a, b) do { double _a = (a), _b = (b); at_t_count++; if (fabs(_a - _b) > 0.01) { at_t_fails++; printf("FAIL %s:%d: %s = %g, want %g\n", __FILE__, __LINE__, #a, _a, _b); } } while (0)
#define CHECK_STR(a, b) do { const char *_a = (a), *_b = (b); at_t_count++; if (strcmp(_a, _b) != 0) { at_t_fails++; printf("FAIL %s:%d: \"%s\", want \"%s\"\n", __FILE__, __LINE__, _a, _b); } } while (0)
#define ATLAS_DONE(name) do { printf("%s: %d checks, %d failed\n", name, at_t_count, at_t_fails); return at_t_fails ? 1 : 0; } while (0)
#endif
