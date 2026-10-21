/* Verbatim copies of the sqrtf / sqrtf_accurate bodies in src/MSL/math_ppc.h (extern inline there). */
double __frsqrte(double);
float w_sqrtf(float x)
{
    volatile float y;
    if (x > 0.0f) {
        double guess = __frsqrte((double) x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        y = (float) (x * guess);
        return y;
    }
    return x;
}
float w_sqrtf_accurate(float x)
{
    volatile float y;
    if (x > 0.0f) {
        double guess = __frsqrte((double) x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        guess = 0.5 * guess * (3.0 - guess * guess * x);
        y = (float) (x * guess);
        return y;
    }
    return x;
}
void Replay_RandTrace(unsigned c, unsigned s, int st) { (void)c; (void)s; (void)st; }
