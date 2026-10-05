// Included after command processor helpers. Retained draws own every referenced
// byte and material binding; no game data is read during a historical replay.
namespace aurora::gx::motion {
void begin(uint32_t source,int add,bool warm_capture,bool flat){
    program=source;additive=add!=0;warming=warm_capture;textureless=flat;last_error=Failure::Count;
    reset_capture_state();
    capture=std::make_shared<Snapshot>();
}
Pose end(){
    ScopedNs timer(TimingEnd);
    if(capture&&!warming&&!capture->failed)flush_pending(*capture);
    reset_capture_state();
    auto result=std::move(capture);
    if(!result || warming || result->failed)return {};
    if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return {};}
    // An empty opaque/translucent pass is normal; the whole-frame assembler
    // reports EmptyPose only if all three passes were empty.
    if(result->draws.empty())return {};
    if(result->bytes>MaxResidentBytes-resident){reject_count(Failure::ResidentBytes);return {};}
    {
        std::unordered_set<const Blob*> seen;
        for(const auto& d:result->draws){
            if(d.vertices&&seen.insert(d.vertices.get()).second)result->uniq_verts.push_back(d.vertices.get());
            if(d.indices&&seen.insert(d.indices.get()).second)result->uniq_indices.push_back(d.indices.get());
            if(d.uniform&&seen.insert(d.uniform.get()).second)result->uniq_uniforms.push_back(d.uniform.get());
        }
        for(const auto& b:result->blocks){
            if(b.palette)result->palette_bytes+=b.data->data.size()+256;
            else if(seen.insert(b.data.get()).second)result->uniq_static.push_back(b.data.get());
        }
    }
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
uint64_t take_timing(unsigned i){return i<TimingCount?timings[i].exchange(0):0;}
const char* timing_name(unsigned i){
    static const char* names[]={"capture","capture_info","capture_copy","capture_blocks","capture_merge","pose_end","replay","replay_prepare","replay_push","palette_blocks","block_bytes","blocks","block_pool_hits"};
    return i<TimingCount?names[i]:"invalid";
}
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
// Blobs remember where they were pushed in this recording (Blob::slot), so a static blob (vertices, index list, array) is
// pushed once per frame however many copies or fighters draw it, with no hashing at replay time.
enum { SlotVerts=0,SlotIndices=1,SlotStorage=2 };
static bool pushed_this_frame(const Blob& b,int slot){return b.slot[slot].serial==gfx::recording_serial();}
static uint64_t replay_call=0; // one (delta, params) pair per call: a modified uniform depends only on its source blob
static bool replay_pipelines_ready(const Pose* poses,size_t count){
    std::vector<gfx::PipelineRef> seen;
    for(size_t n=0;n<count;++n){
        if(!poses[n]){reject_count(Failure::Geometry);return false;}
        for(const auto& d:poses[n]->draws){
            if(!seen.empty()&&seen.back()==d.draw.pipeline)continue;
            if(std::find(seen.begin(),seen.end(),d.draw.pipeline)!=seen.end())continue;
            wgpu::RenderPipeline ready;
            if(!gfx::get_pipeline(d.draw.pipeline,ready)||!ready){reject_count(Failure::PipelinePending);return false;}
            seen.push_back(d.draw.pipeline);
        }
    }
    return true;
}
// Arena bytes this group will newly push (conservative across the group's poses; blobs already pushed this frame cost 0).
static void replay_needs(const Pose* poses,size_t count,size_t& v,size_t& i,size_t& u,size_t& s){
    for(size_t n=0;n<count;++n){
        const auto& p=*poses[n];
        for(auto b:p.uniq_verts)if(!pushed_this_frame(*b,SlotVerts))v+=b->data.size()+4;
        for(auto b:p.uniq_indices)if(!pushed_this_frame(*b,SlotIndices))i+=b->data.size()+4;
        for(auto b:p.uniq_uniforms)u+=b->data.size()+256;
        for(auto b:p.uniq_static)if(!pushed_this_frame(*b,SlotStorage))s+=b->data.size()+256;
        s+=p.palette_bytes;
    }
}
static gfx::Range replay_shared(const Blob& blob,int slot){
    auto& at=blob.slot[slot];
    if(at.serial==gfx::recording_serial())return at.range;
    auto range=slot==SlotIndices?gfx::push_indices(blob.data.data(),blob.data.size(),4):
               slot==SlotVerts?gfx::push_verts(blob.data.data(),blob.data.size(),4):
               gfx::push_storage(blob.data.data(),blob.data.size());
    if(!gfx::frame_overflowed()){at.serial=gfx::recording_serial();at.range=range;}
    return range;
}
static bool replay_pose(const Pose& pose,const std::array<float,12>& delta,const float* params){
    static std::vector<uint8_t> scratch;
    static std::vector<gfx::Range> blocks;
    blocks.clear();blocks.reserve(pose->blocks.size());
    for(const auto& b:pose->blocks){
        if(b.palette){
            scratch.assign(b.data->data.begin(),b.data->data.end());
            for(size_t i=0;i+96<=scratch.size();i+=96){transform(scratch.data()+i,delta,false);transform(scratch.data()+i+48,delta,true);}
            blocks.push_back(gfx::push_storage(scratch.data(),scratch.size()));
        }else blocks.push_back(replay_shared(*b.data,SlotStorage));
        if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return false;}
    }
    auto projection=g_gxState.proj;
    if constexpr(UseReversedZ)projection.m2=projection.m2*Vec4{-1.f,-1.f,-1.f,-1.f};
    else projection.m2=projection.m2+projection.m3;
    for(const auto& saved:pose->draws){
        auto draw=saved.draw;
        const Blob& ub=*saved.uniform;
        if(ub.call!=replay_call){
            scratch.assign(ub.data.begin(),ub.data.end());
            memcpy(scratch.data()+16,params,80);
            const size_t proj=96+(saved.line?16:0);
            memcpy(scratch.data()+proj,&projection,64);
            for(unsigned i=0;i<MaxPnMtx;++i){
                transform(scratch.data()+proj+64+i*48,delta,false);
                transform(scratch.data()+proj+64+MaxPnMtx*48+MaxTexMtx*48+i*48,delta,true);
            }
            ub.call_range=gfx::push_uniform(scratch.data(),scratch.size());
            if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return false;}
            ub.call=replay_call;
        }
        draw.uniformRange=ub.call_range;
        draw.vertRange=replay_shared(*saved.vertices,SlotVerts);
        draw.immediateData.vtxStart=draw.vertRange.offset;
        draw.idxRange=saved.indices?replay_shared(*saved.indices,SlotIndices):gfx::Range{};
        for(uint8_t k=0;k<saved.storage.n;++k){
            const auto index=saved.storage.index[k];
            const auto& block=pose->blocks[index];const auto fresh=blocks[index];
            // Indexed arrays are BYTE offsets; palette/fog bases are WORDS.
            for(size_t a=0;a<draw.immediateData.arrayStart.size();++a){
                const auto original=saved.draw.immediateData.arrayStart[a];
                if(!block.palette)relocate_array(original,block.old,fresh,draw.immediateData.arrayStart[a]);
            }
            if(block.palette)draw.immediateData._pad=fresh.offset/4;
        }
        draw.dstAlpha=UINT32_MAX;
        if(saved.pins&&saved.pins->pinned_serial!=gfx::recording_serial()){
            gfx::motion_pin_binding(draw.bindGroups.textureBindGroup,saved.pins->textures);
            saved.pins->pinned_serial=gfx::recording_serial();
        }
        if(gfx::frame_overflowed()){reject_count(Failure::FrameArena);return false;}
        gfx::push_draw_command(draw);
    }
    return true;
}
// Atomic: every pose of the group is drawn or none is. The preflight refuses (never truncates) when any arena would
// run short, so exhausting an arena degrades to "no copy this frame", counted in the failure stats.
static bool replay_poses(const Pose* poses,size_t count,const std::array<float,12>& delta,const float* params){
    ScopedNs timer(TimingReplay);
    if(!params)return false;
    {
        ScopedNs prep(TimingReplayPrep);
        if(!replay_pipelines_ready(poses,count))return false;
        for(size_t n=0;n<count;++n)if(poses[n]->bytes>MaxPoseBytes)return false;
        size_t v=0,i=0,u=0,s=0;replay_needs(poses,count,v,i,u,s);
        if(!gfx::motion_arena_room(v,i,u,s)){reject_count(Failure::ReplayArena);return false;}
    }
    ScopedNs push(TimingReplayPush);
    ++replay_call;
    for(size_t n=0;n<count;++n)if(!replay_pose(poses[n],delta,params))return false;
    return true;
}
bool replay(const Pose& pose,const std::array<float,12>& delta,const float* params){
    if(!pose)return false;
    return replay_poses(&pose,1,delta,params);
}
bool replay_group(const std::vector<Pose>& poses,const std::array<float,12>& delta,const float* params){
    if(poses.empty())return true;
    return replay_poses(poses.data(),poses.size(),delta,params);
}
void pool_stats(uint64_t out[4]){out[0]=blob_live_bytes.load();out[1]=blob_hits.load();out[2]=blob_misses.load();out[3]=resident;}
void set_timing(bool on){timing_on=on;}
void test_fill_uniform_arena(size_t leave){
    static std::vector<uint8_t> junk(1u<<20,0xA5);
    size_t lo=0,hi=size_t(64)<<20;
    while(lo<hi){size_t mid=lo+(hi-lo+1)/2;if(gfx::motion_arena_room(0,0,mid,0))lo=mid;else hi=mid-1;}
    size_t usable=lo*2; // motion_arena_room allows half of what is left
    while(usable>leave+junk.size()){gfx::push_uniform(junk.data(),junk.size());usable-=junk.size();}
    if(usable>leave)gfx::push_uniform(junk.data(),usable-leave);
    gfx::push_uniform(junk.data(),4096);
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
    uint8_t one=1;b.uniform=make_blob(&one,1,false);if(mergeable(a,b))return false;b.uniform=a.uniform;
    b.draw.immediateData.arrayStart[0]=4;if(mergeable(a,b))return false;b.draw.immediateData.arrayStart[0]=0;
    b.has_index=true;if(mergeable(a,b))return false;b.has_index=false;
    b.storage.n=1;if(mergeable(a,b))return false;b.storage.n=0;
    // Content-addressed blobs: equal bytes share one blob, different bytes never do, and ids are never reused.
    {
        uint8_t x[40],y[40];for(int n=0;n<40;++n){x[n]=uint8_t(n*7+1);y[n]=x[n];}y[39]^=1;
        auto p=make_blob(x,40,true),q=make_blob(x,40,true),r=make_blob(y,40,true);
        if(p!=q||p==r||p->id==r->id)return false;
        const uint64_t id=p->id;p.reset();q.reset();
        auto again=make_blob(x,40,true);if(again->id==id)return false;
    }
    c.shaderConfig.attrs[GX_VA_PNMTXIDX].attrType=GX_DIRECT;
    if(!palette_variant(c))return false;c.shaderConfig.lineMode=1;if(palette_variant(c))return false;
    // First failure wins, without changing global diagnostics or a live capture.
    Snapshot sample;unsigned counted=0;
    auto emit=[&](Failure){++counted;};
    for(size_t n=0;n<FailureCount;++n){sample.failed=false;fail_once(sample,Failure(n),emit);fail_once(sample,Failure(n),emit);}
    return counted==FailureCount;
}
}
