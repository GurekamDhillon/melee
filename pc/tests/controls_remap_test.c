#include "../platform/gw_controls_tests.inc"
int main(void) {
    int line = controls_model_check();
    if (line) { fprintf(stderr, "controls remap: FAIL line %d\n", line); return 1; }
    puts("controls remap: PASS");
    return 0;
}
