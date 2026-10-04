// Included after command processor helpers. Retained draws own every referenced
// byte and material binding; no game data is read during a historical replay.
namespace aurora::gx::motion {
void begin(uint32_t source,int add,bool warm_capture,bool flat){
    program=source;additive=add!=0;warming=warm_capture;textureless=flat;last_error=Failure::Count;
    capture=std::make_shared<Snapshot>();
}
Pose end(){
    auto result=std::move(capture);
    if(!result || warming || result->failed)return {};
    if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return {};}
    // An empty opaque/translucent pass is normal; the whole-frame assembler
    // reports EmptyPose only if all three passes were empty.
    if(result->draws.empty())return {};
    if(result->bytes>MaxResidentBytes-resident){reject_count(Failure::ResidentBytes);return {};}
    resident+=result->bytes;
    auto* value=new Snapshot(std::move(*result));
    return Pose(value,[](const Snapshot* p){resident-=p->bytes;delete p;});
}
size_t bytes(const Pose& p){return p?p->bytes:0;}
size_t draws(const Pose& p){return p?p->draws.size():0;}
void clear_warm(){capture.reset();warmed.clear();}
void capture_only(bool value){only=value;}
bool suppress_live(){return only;}
void rejected(Failure f){reject_count(f);}
Failure last_failure(){return last_error;}
void held_draws(){fallback(Fallback::HeldDraws);}
const char* failure_name(Failure f){
    static const char* names[]={"pipeline_missing","vertex_arena","index_arena","uniform_arena",
        "copy_texture","index_array","palette","fog_lut","pose_bytes","draw_count","frame_arena",
        "resident_bytes","empty_pose","pipeline_pending","replay_arena","geometry"};
    return size_t(f)<FailureCount?names[size_t(f)]:"none";
}
void diagnostics(uint64_t* f,uint64_t* b,uint64_t* w){
    for(size_t i=0;i<FailureCount;++i)f[i]=failures[i].load();
    for(size_t i=0;i<FallbackCount;++i)b[i]=fallbacks[i].load();
    *w=warm_variants.load();
}
bool replaying(){return fifo::replaying();}
bool arena_room(size_t v,size_t i,size_t u,size_t s){return gfx::motion_arena_room(v,i,u,s);}
void projection(float* out){
    auto p=g_gxState.proj;
    if constexpr(UseReversedZ)p.m2=p.m2*Vec4{-1.f,-1.f,-1.f,-1.f};
    else p.m2=p.m2+p.m3;
    memcpy(out,&p,64);
}
static void transform(uint8_t* bytes,const std::array<float,12>& a,bool normal){
    float b[12],r[12]{};memcpy(b,bytes,sizeof b);
    for(int i=0;i<3;++i)for(int j=0;j<4;++j){
        for(int k=0;k<3;++k)r[i*4+j]+=a[i*4+k]*b[k*4+j];
        if(j==3 && !normal)r[i*4+j]+=a[i*4+3];
    }
    // Uniform scale is allowed; normals must not change length when copies grow.
    if(normal){float s=std::sqrt(a[0]*a[0]+a[1]*a[1]+a[2]*a[2]);
        if(s>1e-8f)for(float& x:r)x/=s;}
    memcpy(bytes,r,sizeof r);
}
bool replay(const Pose& pose,const std::array<float,12>& delta,const float* params){
    if(!pose||!params)return false;
    // Preflight against all arenas BEFORE replay; never truncate the live frame.
    // Conservative per-copy caps supplement the recording arena's overflow guard.
    if(pose->bytes>MaxPoseBytes)return false;
    size_t v=0,i=0,u=0,s=0;
    for(const auto& b:pose->blocks)s+=b.data.size()+256;
    for(const auto& saved:pose->draws){
        v+=saved.vertices.size()+4;i+=saved.indices.size()+4;u+=saved.uniform.size()+256;
        auto draw=saved.draw;
        wgpu::RenderPipeline ready;
        if(!gfx::get_pipeline(draw.pipeline,ready) || !ready){reject_count(Failure::PipelinePending);return false;}
    }
    if(!gfx::motion_arena_room(v,i,u,s)){reject_count(Failure::ReplayArena);return false;}
    std::vector<gfx::Range> blocks;blocks.reserve(pose->blocks.size());
    for(const auto& b:pose->blocks){
        auto data=b.data;
        if(b.palette)for(size_t i=0;i+96<=data.size();i+=96){
            transform(data.data()+i,delta,false);transform(data.data()+i+48,delta,true);
        }
        blocks.push_back(gfx::push_storage(data.data(),data.size()));
    }
    for(const auto& saved:pose->draws){
        auto draw=saved.draw;auto uniform=saved.uniform;
        memcpy(uniform.data()+16,params,80);
        const size_t proj=96+(saved.config.shaderConfig.lineMode?16:0);
        auto projection=g_gxState.proj;
        if constexpr(UseReversedZ)projection.m2=projection.m2*Vec4{-1.f,-1.f,-1.f,-1.f};
        else projection.m2=projection.m2+projection.m3;
        memcpy(uniform.data()+proj,&projection,64);
        for(unsigned i=0;i<MaxPnMtx;++i){
            transform(uniform.data()+proj+64+i*48,delta,false);
            transform(uniform.data()+proj+64+MaxPnMtx*48+MaxTexMtx*48+i*48,delta,true);
        }
        draw.vertRange=gfx::push_verts(saved.vertices.data(),saved.vertices.size(),4);
        draw.immediateData.vtxStart=draw.vertRange.offset;
        draw.idxRange=gfx::push_indices(saved.indices.data(),saved.indices.size(),4);
        draw.uniformRange=gfx::push_uniform(uniform.data(),uniform.size());
        for(auto index:saved.storage){
            const auto& block=pose->blocks[index];const auto fresh=blocks[index];
            // Indexed arrays are BYTE offsets; palette/fog bases are WORDS.
            for(size_t a=0;a<draw.immediateData.arrayStart.size();++a){
                const auto original=saved.draw.immediateData.arrayStart[a];
                if(!block.palette)relocate_array(original,block.old,fresh,draw.immediateData.arrayStart[a]);
            }
            if(block.palette)draw.immediateData._pad=fresh.offset/4;
        }
        draw.dstAlpha=UINT32_MAX;
        gfx::motion_pin_binding(draw.bindGroups.textureBindGroup,saved.textures);
        if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return false;}
        gfx::push_draw_command(draw);
    }
    return true;
}
bool replay_group(const std::vector<Pose>& poses,const std::array<float,12>& delta,const float* params){
    size_t v=0,i=0,u=0,s=0;
    for(const auto& p:poses){
        if(!p){reject_count(Failure::Geometry);return false;}
        for(const auto& b:p->blocks)s+=b.data.size()+256;
        for(const auto& d:p->draws){
            wgpu::RenderPipeline ready;
            if(!gfx::get_pipeline(d.draw.pipeline,ready)||!ready){reject_count(Failure::PipelinePending);return false;}
            v+=d.vertices.size()+4;i+=d.indices.size()+4;u+=d.uniform.size()+256;
        }
    }
    if(!gfx::motion_arena_room(v,i,u,s)){reject_count(Failure::ReplayArena);return false;}
    for(const auto& p:poses)if(!replay(p,delta,params))return false;
    return true;
}
bool contract_test(){
    PipelineConfig c{};c.shaderConfig.fogType=GX_FOG_LIN;c.shaderConfig.fogRangeEnabled=true;
    c.shaderConfig.surfaceProgram=77;c.shaderConfig.tevStageCount=1;
    c.shaderConfig.tevStages[0].colorPass.d=GX_CC_TEXC;
    c.shaderConfig.tevStages[0].alphaPass.d=GX_CA_TEXA;
    c.shaderConfig.tevStages[0].texMapId=GX_TEXMAP0;c.shaderConfig.tevStages[0].texCoordId=GX_TEXCOORD0;
    c.shaderConfig.attrs[GX_VA_POS].attrType=GX_INDEX16;c.shaderConfig.pnPalette=true;
    auto flat=variant(c,true),own=variant(c,false);auto info=build_shader_info(flat.shaderConfig);
    if(flat.shaderConfig.fogRangeEnabled||flat.shaderConfig.fogType!=GX_FOG_NONE||info.sampledTextures.any()||info.sampledIndTextures.any())return false;
    if(memcmp(flat.shaderConfig.attrs.data(),c.shaderConfig.attrs.data(),sizeof c.shaderConfig.attrs)||!flat.shaderConfig.pnPalette||!info.indexAttr.test(GX_VA_POS))return false;
    if(own.shaderConfig.fogRangeEnabled||own.shaderConfig.fogType!=GX_FOG_NONE)return false;
    auto changed=c;changed.shaderConfig.surfaceProgram=99;changed.shaderConfig.tevStages[0].texMapId=GX_TEXMAP1;
    auto canonical=variant(changed,true);if(memcmp(&canonical,&flat,sizeof flat))return false;
    ShaderInfo sampled{};sampled.sampledTextures.set(3);
    if(sampled_copy(sampled,[](size_t i){return i==1;}))return false; // stale stage slot
    if(!sampled_copy(sampled,[](size_t i){return i==3;}))return false; // actual EFB sampler
    sampled.sampledTextures.reset();sampled.sampledIndTextures.set(2);
    if(!sampled_copy(sampled,[](size_t i){return i==2;}))return false;
    uint32_t relocated=0;
    if(!relocate_array(44,{20,40},{128,40},relocated)||relocated!=152)return false;
    if(relocate_array(60,{20,40},{128,40},relocated)||relocate_array(19,{20,40},{128,40},relocated))return false;
    // Rebase from ORIGINAL offsets, not a fresh base that could match another
    // captured range. A palette instead uses fresh.offset / sizeof(u32).
    if(!relocate_array(44,{20,40},{256,40},relocated)||relocated!=280)return false;
    Draw a{},b{};a.draw.instanceCount=b.draw.instanceCount=1;a.draw.vtxCount=b.draw.vtxCount=3;
    if(!mergeable(a,b))return false;
    b.uniform.push_back(1);if(mergeable(a,b))return false;b.uniform.clear();
    b.draw.immediateData.arrayStart[0]=4;if(mergeable(a,b))return false;b.draw.immediateData.arrayStart[0]=0;
    b.indices.push_back(0);if(mergeable(a,b))return false;
    c.shaderConfig.attrs[GX_VA_PNMTXIDX].attrType=GX_DIRECT;
    if(!palette_variant(c))return false;c.shaderConfig.lineMode=1;if(palette_variant(c))return false;
    // First failure wins, without changing global diagnostics or a live capture.
    Snapshot sample;unsigned counted=0;
    auto emit=[&](Failure){++counted;};
    for(size_t n=0;n<FailureCount;++n){sample.failed=false;fail_once(sample,Failure(n),emit);fail_once(sample,Failure(n),emit);}
    return counted==FailureCount;
}
}
