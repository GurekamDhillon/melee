/* Runtime shared by the shard-497-504 C harnesses (rt.c) and called from the generated tu_part.c.
   Only builtin types, so tu_part.c can include it under -nostdinc. The Rust twin is
   tests/s497_rt/mod.rs: the two print byte-identical traces for one scenario file. */
#ifndef S497_RT_H
#define S497_RT_H
void tr_begin(const char* name);
void tr_i(unsigned long long v);     /* integer argument, already masked to its width */
void tr_f(unsigned bits);            /* f32 argument */
void tr_d(unsigned long long bits);  /* f64 argument */
void tr_p(const void* p);            /* pointer argument (normalised) */
void tr_s(const char* p);            /* char* argument: arena offset, or the literal's text */
void tr_m(const void* p, unsigned long long n); /* hash of n bytes behind a pointer argument */
void tr_o(void);                     /* out-parameter argument (traced as O) */
void tr_end(void);
unsigned long long ret_i(const char* name, unsigned bits);  /* next scripted/random integer */
unsigned ret_f(const char* name);                           /* next scripted/random f32 bits */
unsigned long long ret_d(const char* name);
void* ret_p(const char* name);                              /* next scripted pointer (default NULL) */
void rt_fill_out(void* p, unsigned long long n);            /* fill an out parameter with random bytes */
unsigned long long rt_rng(void);
void* rt_arena(void);
unsigned long long rt_ncalls(void);
/* dump callbacks used by the generated dump_<Type> functions */
void d_i(const char* path, unsigned long long v);
void d_f(const char* path, unsigned bits);
void d_d(const char* path, unsigned long long bits);
void d_p(const char* path, unsigned long long v);
void d_blob(const char* path, const void* p, unsigned long long n);
#endif
