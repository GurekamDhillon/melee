/* Prints rb_hash / rb_gen / rb_arrival over many inputs (see rb_oracle.sh). */
#include <stdio.h>
#include "rb_cut.c"

int main(void) {
    unsigned i;
    int s, f;
    for (i = 0; i < 2000; ++i) {
        unsigned a = i * 2654435761u, b = i ^ 0xDEADBEEFu, c = i * 7u + 3u;
        printf("hash %u %u %u %u\n", a, b, c, rb_hash(a, b, c));
    }
    {
        unsigned seeds[] = {777u, 4242u, 99u, 0u, 0xFFFFFFFFu};
        for (i = 0; i < 5; ++i) {
            rb.gen_seed = seeds[i];
            for (s = 0; s < 8; s += 2) {
                for (f = -123; f < 500; f += 1) {
                    GwRbInput o;
                    rb_gen(s, f, &o);
                    printf("gen %u %d %d %u %d %d %d %d %u %u %u %u %u\n", rb.gen_seed, s, f, o.buttons, o.raw[0], o.raw[1],
                           o.raw[2], o.raw[3], o.pad_l, o.pad_r, o.present, o.is_raw, o.confirmed);
                }
            }
        }
    }
    {
        int cfg[][4] = {{0, 0, 0, 2}, {4, 2, 0, 2}, {6, 3, 15, 2}, {9, 0, 5, 0}, {20, 7, 40, 8}};
        for (i = 0; i < 5; ++i) {
            rb.fake_lat = cfg[i][0];
            rb.fake_jit = cfg[i][1];
            rb.fake_loss = cfg[i][2];
            rb.delay = cfg[i][3];
            rb.seed = 12345u + i;
            for (s = 0; s < 8; ++s) {
                for (f = -123; f < 200; ++f) {
                    printf("arr %d %d %d %d %u %d %d %ld\n", rb.fake_lat, rb.fake_jit, rb.fake_loss, rb.delay, rb.seed, s, f,
                           rb_arrival(s, f));
                }
            }
        }
    }
    return 0;
}
