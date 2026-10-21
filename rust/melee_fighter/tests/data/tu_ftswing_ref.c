#include <stdio.h>
#define HSD_ASSERTREPORT(a,b,c) return -999
int lbl_803C6D70[6][5] = {
    { 120, 121, 122, -1, 123 }, { 124, 125, 126, -1, 127 },
    { 128, 129, 130, -1, 131 }, { 132, 133, 134, -1, 135 },
    { 136, 137, 138, -1, 139 }, { 140, 141, 142, -1, 143 }
};
int fn_800CCEC4(int x)
{
    switch (x) {
    case 12:
        return 0;
    case 11:
        return 1;
    case 13:
        return 2;
    case 24:
        return 3;
    case 22:
        return 4;
    case 23:
        return 5;
    default:
        HSD_ASSERTREPORT(82, 0, "ellegal swing item!!!\n");
    }
}
int main(void){ for(int x=-2;x<40;x++) printf("k %d %d\n",x,fn_800CCEC4(x));
 for(int s=0;s<6;s++) for(int a=0;a<5;a++) if(a!=3) printf("a %d %d %d\n",s,a,lbl_803C6D70[s][a]); return 0;}
