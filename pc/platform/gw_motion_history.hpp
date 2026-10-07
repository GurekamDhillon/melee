#pragma once
// Presentation history only. No game-memory pointers or simulation state.
#include <array>
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <optional>
namespace gw_motion {
struct Point { float x=0,y=0,z=0; };
inline Point operator+(Point a,Point b){return {a.x+b.x,a.y+b.y,a.z+b.z};}
inline Point operator-(Point a,Point b){return {a.x-b.x,a.y-b.y,a.z-b.z};}
inline Point operator*(Point a,float b){return {a.x*b,a.y*b,a.z*b};}
inline float distance2(Point a,Point b){auto d=a-b;return d.x*d.x+d.y*d.y+d.z*d.z;}
inline Point catmull(Point a,Point b,Point c,Point d,float t){
    return (b*2+(c-a)*t+(a*2-b*5+c*4-d)*(t*t)+(b*3-a-c*3+d)*(t*t*t))*0.5f;
}
inline float fade(int age,int lifetime,float curve){
    return lifetime>0?std::pow(std::clamp(1-float(age)/lifetime,0.f,1.f),curve):0;
}
// Colour variety helpers (pure, presentation only). Rotate an RGB triple's hue about the grey axis by `degrees`.
inline void hue_rotate(float* rgb,float degrees){
    if(degrees==0)return;
    const float r=degrees*0.01745329252f,c=std::cos(r),s=std::sin(r),k=(1-c)/3,q=0.57735026919f*s;
    const float in[3]={rgb[0],rgb[1],rgb[2]};
    rgb[0]=in[0]*(c+k)+in[1]*(k-q)+in[2]*(k+q);
    rgb[1]=in[0]*(k+q)+in[1]*(c+k)+in[2]*(k-q);
    rgb[2]=in[0]*(k-q)+in[1]*(k+q)+in[2]*(c+k);
    for(int i=0;i<3;++i)rgb[i]=std::clamp(rgb[i],0.f,1.f);
}
// Evenly spaced stops, t=0 first stop, t=1 last. count must be >=2.
inline void gradient_sample(const float (*stops)[4],int count,float t,float* out){
    const float x=std::clamp(t,0.f,1.f)*float(count-1);
    const int i=std::min(int(x),count-2);const float f=x-float(i);
    for(int k=0;k<4;++k)out[k]=stops[i][k]*(1-f)+stops[i+1][k]*f;
}
// Alpha multiplier of a travelling pulse: `phase` = rate*seconds - 2*age (a ripple that runs down the trail).
inline float pulse_factor(float rate,float depth,float seconds,float age){
    if(depth<=0)return 1;
    return 1-depth*(0.5f+0.5f*std::sin(6.28318530718f*(rate*seconds-2*age)));
}
// Width multiplier along the trail: tapers to the tail, with an optional mid-trail swell (+) or pinch (-).
inline float width_profile(float age,float taper,float swell){
    const float base=std::pow(std::max(0.f,1-age),taper);
    return base*std::max(0.f,1+swell*std::sin(3.14159265359f*std::clamp(age,0.f,1.f)));
}
template<class T,size_t N> class History {
    struct Entry {int frame;T value;};
    std::array<std::optional<Entry>,N> entries{};
    size_t next=0,count=0;int head=-1;
public:
    void clear(){for(auto& e:entries)e.reset();next=count=0;head=-1;}
    bool push(int frame,T value){
        if(count && frame==head)return false;
        if(count && frame<head)clear();
        entries[next]=Entry{frame,std::move(value)};next=(next+1)%N;
        count=std::min(count+1,N);head=frame;return true;
    }
    // Drop every entry older than `keep` frames. Poses are large; history must hold only what the emitter can
    // still draw, or the resident-memory cap fills and no new pose can ever be captured.
    void trim(int now,int keep){
        size_t n=0;
        for(auto& e:entries){if(e && e->frame<now-keep)e.reset();if(e)++n;}
        count=n;
    }
    const T* age(int now,int age)const{
        if(age<=0)return nullptr;
        for(const auto& e:entries)if(e && e->frame==now-age)return &e->value;
        return nullptr;
    }
    const T* at(int frame)const{
        for(const auto& e:entries)if(e && e->frame==frame)return &e->value;
        return nullptr;
    }
    size_t size()const{return count;}
    int latest()const{return head;}
};
struct Budget {
    size_t bytes,draws;
    bool take(size_t b,size_t d){if(b>bytes||d>draws)return false;bytes-=b;draws-=d;return true;}
};
// Row-major affine matrices: current camera * world offset/scale * inverse(old camera).
using Matrix=std::array<float,12>;
inline Matrix identity(){return {1,0,0,0,0,1,0,0,0,0,1,0};}
inline Matrix multiply(const Matrix& a,const Matrix& b){
    Matrix r{};for(int i=0;i<3;++i)for(int j=0;j<4;++j){
        for(int k=0;k<3;++k)r[i*4+j]+=a[i*4+k]*b[k*4+j];
        if(j==3)r[i*4+j]+=a[i*4+3];
    }return r;
}
inline bool inverse(const Matrix& a,Matrix& r){
    float det=a[0]*(a[5]*a[10]-a[6]*a[9])-a[1]*(a[4]*a[10]-a[6]*a[8])+a[2]*(a[4]*a[9]-a[5]*a[8]);
    if(!std::isfinite(det)||std::abs(det)<1e-8f)return false;
    r={ (a[5]*a[10]-a[6]*a[9])/det,(a[2]*a[9]-a[1]*a[10])/det,(a[1]*a[6]-a[2]*a[5])/det,0,
        (a[6]*a[8]-a[4]*a[10])/det,(a[0]*a[10]-a[2]*a[8])/det,(a[2]*a[4]-a[0]*a[6])/det,0,
        (a[4]*a[9]-a[5]*a[8])/det,(a[1]*a[8]-a[0]*a[9])/det,(a[0]*a[5]-a[1]*a[4])/det,0};
    for(int i=0;i<3;++i)for(int k=0;k<3;++k)r[i*4+3]-=r[i*4+k]*a[k*4+3];
    return true;
}
}
