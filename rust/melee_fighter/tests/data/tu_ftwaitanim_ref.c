#include <stdio.h>
typedef int enum_t;
typedef struct { union { struct { int x, y; } i; struct { int x, y; } p; } u; } WaitStruct;
static int g_roll;
#define HSD_Randi(n) g_roll
#define HSD_ASSERTREPORT(a,b,c,d) return -999
#define static
#define inline
static inline enum_t getAnimID(WaitStruct* arg1)
{
    WaitStruct* wait_data = arg1;
    int max = HSD_Randi(100) + 1;
    int count = 0;
    while (wait_data->u.i.x != -1) {
        count += wait_data->u.i.y;
        if (max <= count) {
            return (enum_t) wait_data->u.p.x;
        }
        wait_data += 1;
    }
    HSD_ASSERTREPORT(86, 0, "wait anim data illegal!!\n", max);
}
#undef static
static unsigned s = 12345u;
static unsigned xs(void){ s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
int main(void){ WaitStruct t[8];
 for (int c = 0; c < 500; c++) { int n = xs() % 7; for (int k = 0; k < n; k++) { t[k].u.i.x = xs() % 40; t[k].u.i.y = (int)(xs() % 60) - 5; }
  t[n].u.i.x = -1; t[n].u.i.y = 0; g_roll = xs() % 100;
  printf("%d", n); for (int k = 0; k < n; k++) printf(" %d %d", t[k].u.i.x, t[k].u.i.y);
  printf(" | %d %d\n", g_roll, getAnimID(t)); }
 return 0; }
