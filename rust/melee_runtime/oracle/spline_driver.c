/* Direct coverage for every spline.c definition, including all static helpers.
 * Hex words are scalar returns, xyz outputs, normalized coefficient pointers,
 * followed by every input field to verify that the routines leave it intact. */
#include "../../../src/sysdolphin/baselib/spline.c"
extern void emit(unsigned);
static u32 rng;
static u32 next(void) { rng^=rng<<13; rng^=rng>>17; rng^=rng<<5; return rng; }
static float draw(void) { return ((int)(next()%200001)-100000)/8192.0f; }
static void put(float x) { emit(*(u32*)&x); }
static void pv(Vec3* v) { put(v->x);put(v->y);put(v->z); }
void run_spline_case(int seed) {
    Vec3 cv[16],v; float lengths[5]={0,.25f,.5f,.75f,1}, poly[4][5];
    HSD_Spline s; int i,j; float x[6],u,dx;
    rng=seed;
    for(i=0;i<16;i++){cv[i].x=draw();cv[i].y=draw();cv[i].z=draw();}
    for(i=0;i<4;i++)for(j=0;j<5;j++)poly[i][j]=(float)(next()%10000)/8192.0f;
    for(i=0;i<6;i++)x[i]=draw();
    u=(float)(seed%1025)/1024.0f; if(seed%23==0)u=-.25f;if(seed%29==0)u=1.25f;
    s.type=seed%5;s.numcv=5;s.tension=x[0];s.cv=cv;s.totalLength=4;s.segLength=lengths;s.segPoly=poly;
    emit(seed);put(splGetHelmite(x[0],x[1],x[2],x[3],x[4],x[5]));
    splGetCardinalPoint(&v,cv,x[0],u);pv(&v);
    splGetBSplinePoint(&v,cv,u);pv(&v);
    splGetBezierPoint(&v,cv,u);pv(&v);
    v.x=v.y=v.z=99;splGetSplinePoint(&v,&s,u);pv(&v);
    put(splArcLengthPolynomial(poly[0],u));
    emit(spl_GetCoeffs(&s,seed%4)-&poly[0][0]);
    dx=spl_GetArcLengthDx(x[1],x[2]);put(dx);
    put(spl_IterateSimpsonsMiddle(poly[seed%4],dx,x[3]));
    put(splArcLengthGetParameter(&s,u));
    v.x=v.y=v.z=99;splArcLengthPoint(&v,&s,u);pv(&v);
    emit(s.type);emit(s.numcv);put(s.tension);put(s.totalLength);
    for(i=0;i<16;i++)pv(cv+i);
    for(i=0;i<5;i++)put(lengths[i]);
    for(i=0;i<4;i++)for(j=0;j<5;j++)put(poly[i][j]);
}
