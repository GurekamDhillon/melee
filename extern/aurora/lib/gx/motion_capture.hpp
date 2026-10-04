#pragma once
#include <aurora/motion.hpp>
#include "pipeline.hpp"
#include "shader_info.hpp"
#include "../gfx/resource_cache.hpp"
#include "../gfx/pipeline_cache.hpp"
#include <map>
#include <atomic>
namespace aurora::gfx {
bool motion_copy_arena(unsigned,Range,std::vector<uint8_t>&);
bool motion_arena_room(size_t,size_t,size_t,size_t);
void motion_pin_binding(BindGroupRef,const wgpu::BindGroup&);
}
namespace aurora::gx::motion {
struct Block { gfx::Range old; std::vector<uint8_t> data; bool palette=false; };
struct Draw {
    DrawData draw;
    PipelineConfig config;
    std::vector<uint8_t> vertices,indices,uniform;
    std::vector<size_t> storage;
    wgpu::BindGroup textures; // keep the actual immutable binding alive, even if cache expires
    std::array<gfx::TextureBind,MaxTextures> pins;
};
struct Snapshot { std::vector<Draw> draws;std::vector<Block> blocks;size_t bytes=0;bool failed=false; };
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
inline bool mergeable(const Draw& a,const Draw& b){
    if(!a.indices.empty()||!b.indices.empty()||a.config.shaderConfig.lineMode||b.config.shaderConfig.lineMode||
       a.draw.instanceCount!=1||b.draw.instanceCount!=1||a.draw.vtxCount%3||b.draw.vtxCount%3||
       a.draw.pipeline!=b.draw.pipeline||memcmp(&a.config,&b.config,sizeof a.config)||
       a.uniform!=b.uniform||a.storage!=b.storage||a.draw.bindGroups.textureBindGroup!=b.draw.bindGroups.textureBindGroup)return false;
    auto x=a.draw.immediateData,y=b.draw.immediateData;x.vtxStart=y.vtxStart=0;
    return !memcmp(&x,&y,sizeof x);
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
inline void retain(const DrawData& source,const PipelineConfig& config){
    if(!active()||warming||capture->failed)return;
    Draw d{};d.draw=source;
    auto original_info=build_shader_info(config.shaderConfig);
    bool copied_texture=sampled_copy(original_info,[&](size_t i){
        const auto& pin=g_gxState.textures[i];
        return pin && g_gxState.copyTextures.find(pin.texObj.data)!=g_gxState.copyTextures.end();
    });
    const bool flat=textureless||copied_texture;
    d.config=variant(config,flat);
    if(config.shaderConfig.fogRangeEnabled||config.shaderConfig.fogType!=GX_FOG_NONE)fallback(Fallback::NoFog);
    if(flat)fallback(Fallback::Textureless);
    if(copied_texture&&!textureless)fallback(Fallback::CopySilhouette);
    d.draw.pipeline=prepared(d.config,false);
    if(!d.draw.pipeline){fail(Failure::Pipeline);return;} // no compilation requested in play
    if(!gfx::motion_copy_arena(0,source.vertRange,d.vertices)){fail(Failure::VertexArena);return;}
    if(!gfx::motion_copy_arena(1,source.idxRange,d.indices)){fail(Failure::IndexArena);return;}
    auto info=build_shader_info(d.config.shaderConfig);
    if(!info.uniformSize){fail(Failure::UniformArena);return;}
    if(!gfx::motion_arena_room(0,0,info.uniformSize+256,0)){fail(Failure::UniformArena);return;}
    // Material stripping changes variable uniform fields. Generate from the
    // exact current GX state rather than splicing an incompatible old layout.
    const auto uniform=build_uniform(info);
    if(uniform.size<96+64+MaxPnMtx*96+MaxTexMtx*48 || !gfx::motion_copy_arena(2,uniform,d.uniform)){fail(Failure::UniformArena);return;}
    if(flat)d.draw.bindGroups.textureBindGroup=0;
    else {
        for(size_t i=0;i<MaxTextures;++i)if(info.sampledTextures.test(i)||info.sampledIndTextures.test(i))d.pins[i]=g_gxState.textures[i];
        if(source.bindGroups.textureBindGroup)d.textures=gfx::find_bind_group(source.bindGroups.textureBindGroup);
    }
    auto add=[&](gfx::Range range,bool palette){
        if(!range.size)return false;
        for(size_t i=0;i<capture->blocks.size();++i)if(capture->blocks[i].old==range){
            capture->blocks[i].palette|=palette;d.storage.push_back(i);return true;
        }
        if(sizeof(Block)+range.size>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return false;}
        Block b{range,{},palette};
        if(!gfx::motion_copy_arena(3,range,b.data))return false;
        capture->bytes+=sizeof(Block)+b.data.size();
        d.storage.push_back(capture->blocks.size());capture->blocks.push_back(std::move(b));return true;
    };
    for(int a=GX_VA_POS;a<=GX_VA_TEX7;++a)
        if(info.indexAttr.test(a) && !add(g_gxState.arrays[a].cachedRange,false)){fail(Failure::IndexArray);return;}
    if(config.shaderConfig.pnPalette && (!g_palette.active||!add({source.immediateData._pad*4,g_palette.n*96},true))){fail(Failure::Palette);return;}
    d.draw.immediateData.fogRangeBase=0;
    // Capture disables the CP's live merging to observe every draw. Coalesce
    // compatible retained triangle chunks afterwards, preserving material,
    // matrix/index-array snapshots and order, so copies do not multiply tiny
    // per-triangle GPU draws.
    if(!capture->draws.empty()&&mergeable(capture->draws.back(),d)){
        if(d.vertices.size()>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return;}
        auto& previous=capture->draws.back();
        previous.vertices.insert(previous.vertices.end(),d.vertices.begin(),d.vertices.end());
        if(only)fallback(Fallback::HeldGeometry);
        previous.draw.vtxCount+=d.draw.vtxCount;capture->bytes+=d.vertices.size();return;
    }
    size_t cost=sizeof(Draw)+d.vertices.size()+d.indices.size()+d.uniform.size();
    cost+=d.storage.size()*sizeof(size_t);
    if(capture->draws.size()>=2048){fail(Failure::DrawCount);return;}
    if(cost>MaxPoseBytes-capture->bytes){fail(Failure::PoseBytes);return;}
    capture->bytes+=cost;capture->draws.push_back(std::move(d));if(only)fallback(Fallback::HeldGeometry);
}
}
