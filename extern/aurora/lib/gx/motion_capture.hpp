#pragma once
#include <aurora/motion.hpp>
#include "pipeline.hpp"
#include "shader_info.hpp"
#include "../gfx/resource_cache.hpp"
#include "../gfx/pipeline_cache.hpp"
#include <map>
#include <atomic>
#include <mutex>
#include <unordered_map>
#include <unordered_set>
#include <optional>
#include <chrono>
#include <nmmintrin.h>
namespace aurora::gfx {
bool motion_arena_view(unsigned,Range,const uint8_t*&);
uint64_t recording_serial() noexcept;
bool motion_arena_room(size_t,size_t,size_t,size_t);
void motion_pin_binding(BindGroupRef,const wgpu::BindGroup&);
}
namespace aurora::gx::motion {
inline bool timing_on=false;
// Per-category wall time (ns) accumulated on the GX thread and drained once per frame by the game's profiler.
inline std::array<std::atomic<uint64_t>,TimingCount> timings{};
struct ScopedNs {
    unsigned i;bool on;std::chrono::steady_clock::time_point t0;
    explicit ScopedNs(unsigned n):i(n),on(timing_on){if(on)t0=std::chrono::steady_clock::now();}
    ~ScopedNs(){if(on)timings[i]+=uint64_t(std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now()-t0).count());}
};
// Immutable byte blobs, content-addressed: a pose's static data (vertices, index lists, arrays, uniforms) is identical from
// frame to frame, so identical bytes are stored once and shared by every draw and pose that holds them. Blobs are
// identified by a monotonically increasing id that is never reused (safe as a per-frame cache key).
// Where this blob was last pushed into a recording arena (verts, indices, storage): valid only while `serial` equals the
// current recording serial, so a blob is pushed once per frame however many copies draw it. GX thread only.
struct PushSlot { uint64_t serial=0;gfx::Range range; };
struct Blob { std::vector<uint8_t> data;uint64_t id=0,h1=0,h2=0;bool pooled=false;mutable PushSlot slot[3];mutable uint64_t call=0;mutable gfx::Range call_range; };
using BlobRef=std::shared_ptr<const Blob>;
struct BlobKey { uint64_t h1,h2;size_t n;bool operator==(const BlobKey& o)const{return h1==o.h1&&h2==o.h2&&n==o.n;} };
struct BlobKeyHash { size_t operator()(const BlobKey& k)const{return size_t(k.h1^(k.h2*0x9E3779B97F4A7C15ull));} };
// Leaked on purpose: a pose released during static destruction must still find its pool.
inline std::mutex& blob_mutex(){static auto* m=new std::mutex;return *m;}
inline std::unordered_map<BlobKey,std::weak_ptr<const Blob>,BlobKeyHash>& blob_pool(){
    static auto* p=new std::unordered_map<BlobKey,std::weak_ptr<const Blob>,BlobKeyHash>;return *p;}
inline std::atomic<uint64_t> blob_ids{0},blob_live_bytes{0},blob_hits{0},blob_misses{0};
// 128-bit content hash: four interleaved CRC32C lanes (hardware crc32, ~4 bytes per cycle; the port is 32-bit, where a 64-bit
// multiply hash runs several times slower), mixed with the length. Not cryptographic; this only dedupes presentation data.
// The crc32 intrinsics need SSE4.2; clang and gcc refuse to inline them (always_inline) into a function without the
// feature, which a bare i686 target (the Linux build) does not enable. Every x86 CPU since 2008 has it.
#if defined(__clang__) || defined(__GNUC__)
__attribute__((target("sse4.2")))
#endif
inline void hash128(const uint8_t* p,size_t n,uint64_t& a,uint64_t& b){
    uint32_t c0=0x9E3779B9u^uint32_t(n),c1=0x85EBCA6Bu,c2=0xC2B2AE35u,c3=0x27D4EB2Fu^uint32_t(uint64_t(n)>>16);size_t i=0;
    for(;i+16<=n;i+=16){
        uint32_t w0,w1,w2,w3;memcpy(&w0,p+i,4);memcpy(&w1,p+i+4,4);memcpy(&w2,p+i+8,4);memcpy(&w3,p+i+12,4);
        c0=_mm_crc32_u32(c0,w0);c1=_mm_crc32_u32(c1,w1);c2=_mm_crc32_u32(c2,w2);c3=_mm_crc32_u32(c3,w3);
    }
    for(;i<n;++i)c0=_mm_crc32_u8(c0,p[i]);
    c1=_mm_crc32_u32(c1,c0);c2=_mm_crc32_u32(c2,c1);c3=_mm_crc32_u32(c3,c2);c0=_mm_crc32_u32(c0,c3);
    a=(uint64_t(c0)<<32)|c1;b=(uint64_t(c2)<<32)|c3;
}
inline BlobRef make_blob(const uint8_t* p,size_t n,bool pool){
    BlobKey key{0,0,n};
    if(pool){
        hash128(p,n,key.h1,key.h2);
        std::lock_guard lock{blob_mutex()};
        auto it=blob_pool().find(key);
        if(it!=blob_pool().end())if(auto live=it->second.lock()){++blob_hits;return live;}
    }
    auto* b=new Blob;b->data.assign(p,p+n);b->id=++blob_ids;b->pooled=pool;b->h1=key.h1;b->h2=key.h2;
    blob_live_bytes+=n;++blob_misses;
    BlobRef ref(b,[](const Blob* x){
        blob_live_bytes-=x->data.size();
        if(x->pooled){
            std::lock_guard lock{blob_mutex()};
            auto it=blob_pool().find(BlobKey{x->h1,x->h2,x->data.size()});
            if(it!=blob_pool().end()&&it->second.expired())blob_pool().erase(it);
        }
        delete x;
    });
    if(pool){std::lock_guard lock{blob_mutex()};blob_pool()[key]=ref;}
    return ref;
}
struct Block { gfx::Range old; BlobRef data; bool palette=false; };
struct Pins {
    mutable uint64_t pinned_serial=0;
    wgpu::BindGroup textures; // keep the actual immutable binding alive, even if cache expires
    std::array<gfx::TextureBind,MaxTextures> pins;
};
// Indices into the pose's block table (index arrays, plus the skinning palette).
struct StorageList { uint8_t n=0;uint32_t index[20]{};
    bool operator==(const StorageList& o)const{return n==o.n&&!memcmp(index,o.index,n*sizeof(uint16_t));} };
struct Draw {
    DrawData draw;
    bool line=false,has_index=false;
    BlobRef vertices,indices,uniform;
    StorageList storage;
    std::shared_ptr<const Pins> pins;
};
// Distinct blobs this pose pushes when replayed (for the arena preflight), filled once when the pose is finished.
struct Snapshot { std::vector<Draw> draws;std::vector<Block> blocks;size_t bytes=0;bool failed=false;
    std::vector<const Blob*> uniq_verts,uniq_indices,uniq_uniforms,uniq_static;size_t palette_bytes=0; };
inline std::shared_ptr<Snapshot> capture;
inline uint32_t program;
inline bool additive,warming,textureless,only;
inline std::array<std::atomic<uint64_t>,FailureCount> failures{};
inline std::array<std::atomic<uint64_t>,FallbackCount> fallbacks{};
inline std::atomic<uint64_t> warm_variants{};
inline Failure last_error=Failure::Count;
inline void reject_count(Failure f){if(f!=Failure::Count){++failures[size_t(f)];last_error=f;}}
template<class Emit> inline void fail_once(Snapshot& s,Failure f,Emit emit){if(!s.failed){s.failed=true;emit(f);}}
inline void fail(Failure f){if(capture)fail_once(*capture,f,reject_count);}
inline void fallback(Fallback f){++fallbacks[size_t(f)];}
inline size_t resident=0;
inline constexpr size_t MaxPoseBytes=4*1024*1024,MaxResidentBytes=384*1024*1024;
struct Warm { PipelineConfig config;uint64_t layout;gfx::PipelineRef ref; };
inline std::vector<Warm> warmed;
inline bool active(){return bool(capture);}
template<class IsCopy> inline bool sampled_copy(const ShaderInfo& info,IsCopy is_copy){
    for(size_t i=0;i<MaxTextures;++i)if((info.sampledTextures.test(i)||info.sampledIndTextures.test(i))&&is_copy(i))return true;
    return false;
}
inline bool palette_variant(const PipelineConfig& c){return c.shaderConfig.lineMode==0&&c.shaderConfig.attrs[GX_VA_PNMTXIDX].attrType==GX_DIRECT;}
inline bool relocate_array(uint32_t original,gfx::Range old,gfx::Range fresh,uint32_t& out){
    if(original<old.offset||original-old.offset>=old.size)return false;
    out=fresh.offset+(original-old.offset);return true;
}
inline PipelineConfig variant(PipelineConfig c,bool flat){
    c.shaderConfig.surfaceProgram=program;
    // Stage fog state can remain enabled even with GX_FOG_NONE. Copies are
    // intentionally unfogged; build their own matching uniform layout below.
    c.shaderConfig.fogType=GX_FOG_NONE;c.shaderConfig.fogRangeEnabled=false;
    if(flat){
        // Preserve vertex decoding, culling and skinning, but remove every
        // material dependency, including texture alpha tests and indirect TEV.
        c.shaderConfig.tevStages={};c.shaderConfig.tevStageCount=1;
        auto& stage=c.shaderConfig.tevStages[0];
        stage.colorPass.d=GX_CC_ONE;stage.alphaPass.d=GX_CA_KONST;
        stage.kaSel=GX_TEV_KASEL_1;
        c.shaderConfig.tevSwapTable={};c.shaderConfig.colorChannels={};
        c.shaderConfig.tcgs={};c.shaderConfig.indStages={};c.shaderConfig.numIndStages=0;
        c.shaderConfig.alphaCompare={};
    }
    c.depthCompare=true;c.depthUpdate=false;c.alphaUpdate=false;c.colorUpdate=true;
    c.depthFunc=GX_LEQUAL;c.blendMode=GX_BM_BLEND;c.blendFacSrc=GX_BL_SRCALPHA;
    c.blendFacDst=additive?GX_BL_ONE:GX_BL_INVSRCALPHA;c.dstAlpha=UINT32_MAX;
    return c;
}
inline gfx::PipelineRef prepared_for(const PipelineConfig& c,const gfx::RenderTargetLayout& layout,bool request){
    for(const auto& w:warmed)if(w.layout==layout.key && !memcmp(&w.config,&c,sizeof c))return w.ref;
    if(!request || warmed.size()>=4096)return 0;
    auto ref=gfx::find_pipeline(c,layout);warmed.push_back({c,layout.key,ref});++warm_variants;return ref;
}
inline gfx::PipelineRef prepared(const PipelineConfig& c,bool request){return prepared_for(c,gfx::get_render_target_layout(),request);}
inline void warm(const PipelineConfig& config){
    if(!active()||!warming)return;
    // Warm both material and texture-free fallback, plus the palette variant
    // that HSD's live envelope path may select outside descriptor warm traversal.
    const gfx::RenderTargetLayout targets[]={gfx::get_render_target_layout(),gfx::scene_render_target_layout()};
    for(const auto& target:targets)for(int flat=textureless?1:0;flat<2;++flat){
        auto c=variant(config,flat!=0);
        auto ref=prepared_for(c,target,true);if(ref)gfx::pipeline_warm_record(ref);
        if(palette_variant(c)){
            c.shaderConfig.pnPalette=!c.shaderConfig.pnPalette;
            ref=prepared_for(c,target,true);if(ref){gfx::pipeline_warm_record(ref);fallback(Fallback::PaletteWarm);}
        }
    }
}
// Draws whose material, uniforms, arrays and textures match are merged into one retained draw: capture turns the CP's
// live merging off to observe every draw, so this restores it for the copies. `a` is the pending draw, `b` the candidate.
inline bool mergeable(const Draw& a,const Draw& b){
    if(a.has_index||b.has_index||a.line||b.line||
       a.draw.instanceCount!=1||b.draw.instanceCount!=1||a.draw.vtxCount%3||b.draw.vtxCount%3||
       a.draw.pipeline!=b.draw.pipeline||a.uniform!=b.uniform||!(a.storage==b.storage)||
       a.draw.bindGroups.textureBindGroup!=b.draw.bindGroups.textureBindGroup)return false;
    auto x=a.draw.immediateData,y=b.draw.immediateData;x.vtxStart=y.vtxStart=0;
    return !memcmp(&x,&y,sizeof x);
}
// What the previous retained draw already worked out; valid while the material (config + flat decision) is unchanged.
struct Memo {
    bool valid=false,flat=false,uniform_valid=false,pins_valid=false;
    PipelineConfig src,variant;ShaderInfo info;gfx::PipelineRef pipeline=0;
    gfx::Range uniform_src;BlobRef uniform;
    gfx::BindGroupRef pins_bind=0;std::shared_ptr<const Pins> pins;
};
struct Pending { bool has=false;Draw draw;std::vector<uint8_t> verts; };
// Block lookup within the pose being captured, by (offset,size): a pose can hold thousands of blocks.
inline std::unordered_map<uint64_t,uint32_t> block_index;
inline Memo memo;
inline Pending pending;
inline void reset_capture_state(){memo.valid=memo.uniform_valid=memo.pins_valid=false;memo.uniform.reset();memo.pins.reset();pending.has=false;pending.draw=Draw{};block_index.clear();}
inline void flush_pending(Snapshot& s){
    if(!pending.has)return;
    pending.has=false;
    pending.draw.vertices=make_blob(pending.verts.data(),pending.verts.size(),true);
    s.draws.push_back(std::move(pending.draw));
    pending.draw=Draw{};
}
// The frame arenas are mapped GPU staging memory: write-combined, so reading them back runs at ~120 MB/s. Retention
// therefore reads the host-side sources the CP already holds (the draw's vertex and index bytes, and the CP's shadow of
// uploaded arrays and palettes); the arena is only a fallback.
inline const uint8_t* (*host_storage)(const gfx::Range&)=nullptr;
inline void retain(const DrawData& source,const PipelineConfig& config,const ShaderInfo& original_info,const uint8_t* vp,const uint8_t* ip){
    if(!active()||warming||capture->failed)return;
    ScopedNs total(TimingCapture);
    std::optional<ScopedNs> phase(std::in_place,TimingInfo);
    Draw d{};d.draw=source;
    bool copied_texture=sampled_copy(original_info,[&](size_t i){
        const auto& pin=g_gxState.textures[i];
        return pin && g_gxState.copyTextures.find(pin.texObj.data)!=g_gxState.copyTextures.end();
    });
    const bool flat=textureless||copied_texture;
    if(!memo.valid||memo.flat!=flat||memcmp(&memo.src,&config,sizeof config)){
        memo.valid=false;memo.src=config;memo.flat=flat;
        memo.variant=variant(config,flat);
        memo.info=build_shader_info(memo.variant.shaderConfig);
        memo.pipeline=prepared(memo.variant,false); // no compilation requested in play
        memo.uniform_valid=memo.pins_valid=false;memo.valid=true;
    }
    if(config.shaderConfig.fogRangeEnabled||config.shaderConfig.fogType!=GX_FOG_NONE)fallback(Fallback::NoFog);
    if(flat)fallback(Fallback::Textureless);
    if(copied_texture&&!textureless)fallback(Fallback::CopySilhouette);
    if(!memo.pipeline){fail(Failure::Pipeline);return;}
    d.draw.pipeline=memo.pipeline;
    const auto& info=memo.info;
    if(!info.uniformSize){fail(Failure::UniformArena);return;}
    phase.emplace(TimingCopy);
    if(source.vertRange.size&&!vp){fail(Failure::VertexArena);return;}
    if(source.idxRange.size&&!ip){fail(Failure::IndexArena);return;}
    // The retained uniform is built from the exact current GX state with the stripped material (it never enters the
    // frame arena). The live draw reuses one range until its state changes, so an unchanged range means unchanged bytes.
    if(!memo.uniform_valid||memo.uniform_src!=source.uniformRange){
        static ByteBuffer scratch;scratch.clear();
        build_uniform_bytes(info,scratch);
        if(scratch.size()<96+64+MaxPnMtx*96+MaxTexMtx*48){fail(Failure::UniformArena);return;}
        memo.uniform=make_blob(scratch.data(),scratch.size(),true);memo.uniform_src=source.uniformRange;memo.uniform_valid=true;
    }
    d.uniform=memo.uniform;
    if(flat)d.draw.bindGroups.textureBindGroup=0;
    else{
        if(!memo.pins_valid||memo.pins_bind!=source.bindGroups.textureBindGroup){
            auto pins=std::make_shared<Pins>();
            for(size_t i=0;i<MaxTextures;++i)if(info.sampledTextures.test(i)||info.sampledIndTextures.test(i))pins->pins[i]=g_gxState.textures[i];
            if(source.bindGroups.textureBindGroup)pins->textures=gfx::find_bind_group(source.bindGroups.textureBindGroup);
            memo.pins=std::move(pins);memo.pins_bind=source.bindGroups.textureBindGroup;memo.pins_valid=true;
        }
        d.pins=memo.pins;
    }
    phase.emplace(TimingBlocks);
    auto add=[&](gfx::Range range,bool palette){
        if(!range.size)return false;
        const uint64_t key=(uint64_t(range.offset)<<32)|range.size;
        auto found=block_index.find(key);
        if(found!=block_index.end()){
            capture->blocks[found->second].palette|=palette;
            if(d.storage.n>=20)return false;d.storage.index[d.storage.n++]=found->second;return true;
        }
        if(sizeof(Block)+range.size>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return false;}
        const uint8_t* bp=host_storage?host_storage(range):nullptr;
        if(!bp&&(!gfx::motion_arena_view(3,range,bp)||!bp))return false;
        if(d.storage.n>=20)return false;
        // Palettes are unique to their frame; static arrays repeat every frame and are shared.
        std::optional<ScopedNs> pal;if(palette)pal.emplace(TimingPalette);
        const auto hits_before=blob_hits.load();
        Block b{range,make_blob(bp,range.size,!palette),palette};
        if(timing_on){timings[CountBlockBytes]+=range.size;++timings[CountBlocks];timings[CountBlockHits]+=blob_hits.load()-hits_before;}
        capture->bytes+=sizeof(Block)+b.data->data.size();
        block_index.emplace(key,uint32_t(capture->blocks.size()));d.storage.index[d.storage.n++]=uint32_t(capture->blocks.size());capture->blocks.push_back(std::move(b));return true;
    };
    for(int a=GX_VA_POS;a<=GX_VA_TEX7;++a)
        if(info.indexAttr.test(a) && !add(g_gxState.arrays[a].cachedRange,false)){fail(Failure::IndexArray);return;}
    if(config.shaderConfig.pnPalette && (!g_palette.active||!add({source.immediateData._pad*4,g_palette.n*96},true))){fail(Failure::Palette);return;}
    phase.emplace(TimingMerge);
    d.draw.immediateData.fogRangeBase=0;
    d.line=config.shaderConfig.lineMode!=0;d.has_index=source.idxRange.size!=0;
    const size_t vn=source.vertRange.size,in=source.idxRange.size;
    // Capture disables the CP's live merging to observe every draw. Coalesce compatible retained triangle chunks
    // afterwards, preserving material, matrix/index-array snapshots and order.
    if(pending.has&&mergeable(pending.draw,d)){
        if(vn>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return;}
        if(vn)pending.verts.insert(pending.verts.end(),vp,vp+vn);
        if(only)fallback(Fallback::HeldGeometry);
        pending.draw.draw.vtxCount+=d.draw.vtxCount;capture->bytes+=vn;return;
    }
    flush_pending(*capture);
    size_t cost=sizeof(Draw)+vn+in+d.uniform->data.size()+size_t(d.storage.n)*sizeof(size_t);
    if(capture->draws.size()>=2048){fail(Failure::DrawCount);return;}
    if(cost>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return;}
    capture->bytes+=cost;
    if(in)d.indices=make_blob(ip,in,true);
    pending.draw=std::move(d);pending.verts.assign(vp,vp+vn);pending.has=true;
    if(only)fallback(Fallback::HeldGeometry);
}
}
