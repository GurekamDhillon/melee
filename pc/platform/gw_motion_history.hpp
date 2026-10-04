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
