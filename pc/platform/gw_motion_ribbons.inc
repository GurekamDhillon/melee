// Included in gw_motion.cpp, inside its private namespace.
struct RibbonVertex {float position[4],color[4],uv[4];};
struct RibbonUniform {float view[12],proj[16],edge[4],params[4],clock[4];};
struct RibbonDraw {aurora::gfx::Range vertices,uniform;uint32_t count;int blend,depth;};
size_t ribbon_frame_vertices;int ribbon_frame_tag=-1;
wgpu::ShaderModule ribbon_module;
wgpu::BindGroupLayout ribbon_bgl;wgpu::PipelineLayout ribbon_layout;
aurora::gfx::DrawTypeId ribbon_type;
struct RibbonPipe {uint64_t layout;int blend,depth;wgpu::RenderPipeline pipe;};
std::vector<RibbonPipe> ribbon_pipes;
std::mutex ribbon_gpu_mutex;
const char* ribbon_wgsl=R"WGSL(
struct Vertex {position:vec4f,color:vec4f,uv:vec4f};
struct U {view:array<vec4f,3>,proj:array<vec4f,4>,edge:vec4f,param:vec4f,clock:vec4f};
@group(0) @binding(0) var<uniform> u:U;
@group(0) @binding(1) var<storage,read> vertices:array<Vertex>;
struct Out {@builtin(position) position:vec4f,@location(0) color:vec4f,@location(1) uv:vec2f};
@vertex fn vs(@builtin(vertex_index) i:u32)->Out {
 let v=vertices[i]; let eye=vec4f(dot(u.view[0],v.position),dot(u.view[1],v.position),dot(u.view[2],v.position),1.0);
 var o:Out; o.position=vec4f(dot(u.proj[0],eye),dot(u.proj[1],eye),dot(u.proj[2],eye),dot(u.proj[3],eye));
 o.color=v.color;o.uv=v.uv.xy;return o;
}
@fragment fn fs(i:Out)->@location(0) vec4f {
 let cross=abs(i.uv.y*2.0-1.0);let core=1.0-smoothstep(clamp(0.1+u.param.z*0.05,0.0,0.8),1.0,cross);
 let style=u.clock.y; let phase=i.uv.x*10.0*max(0.1,u.param.y)-u.clock.x*(2.0+u.param.w);
 var alpha=core;var rgb=mix(i.color.rgb,u.edge.rgb,cross*u.edge.a);
 if(style>0.5 && style<1.5){alpha=pow(core,2.0);rgb=mix(rgb,vec3f(1.0),core*0.35);}
 if(style>1.5 && style<2.5){let heat=0.7+0.3*sin(phase+sin(i.uv.y*8.0));rgb=mix(rgb,vec3f(1.0,0.6,0.05),core*0.45);alpha*=heat;}
 if(style>2.5 && style<3.5){let arc=1.0-smoothstep(0.0,0.25,abs(i.uv.y-0.5-0.16*sin(phase*3.0)));rgb=mix(rgb,vec3f(0.85,0.95,1.0),arc*0.65);alpha*=0.35+0.65*arc;}
 if(style>3.5 && style<4.5){let facets=0.75+0.25*abs(sin(phase*1.5+i.uv.y*12.0));rgb=mix(rgb,vec3f(0.7,0.95,1.0),0.4);alpha*=facets;}
 if(style>4.5){rgb=mix(rgb,vec3f(0.15,0.02,0.25),0.4);alpha*=0.8;}
 return vec4f(rgb,clamp(i.color.a*alpha*u.param.x,0.0,0.75));
}
)WGSL";
wgpu::RenderPipeline ribbon_pipeline(const aurora::gfx::RenderTargetLayout& target,int blend,int depth,bool create){
    std::lock_guard lock(ribbon_gpu_mutex);
    for(const auto& p:ribbon_pipes)if(p.layout==target.key&&p.blend==blend&&p.depth==depth)return p.pipe;
    if(!create||ribbon_pipes.size()>=128)return {};
    auto device=aurora::gfx::device();if(!device)return {};
    if(!ribbon_module){
        wgpu::ShaderSourceWGSL wgsl{};wgsl.code=ribbon_wgsl;
        wgpu::ShaderModuleDescriptor md{};md.nextInChain=&wgsl;md.label="motion/ribbons";
        ribbon_module=device.CreateShaderModule(&md);
        wgpu::BindGroupLayoutEntry entries[2]{};
        entries[0].binding=0;entries[0].visibility=wgpu::ShaderStage::Vertex|wgpu::ShaderStage::Fragment;
        entries[0].buffer.type=wgpu::BufferBindingType::Uniform;
        entries[1].binding=1;entries[1].visibility=wgpu::ShaderStage::Vertex;entries[1].buffer.type=wgpu::BufferBindingType::ReadOnlyStorage;
        wgpu::BindGroupLayoutDescriptor bg{};bg.entryCount=2;bg.entries=entries;ribbon_bgl=device.CreateBindGroupLayout(&bg);
        wgpu::PipelineLayoutDescriptor layout{};layout.bindGroupLayoutCount=1;layout.bindGroupLayouts=&ribbon_bgl;
        ribbon_layout=device.CreatePipelineLayout(&layout);
    }
    wgpu::BlendState bs{};
    bs.color.srcFactor=wgpu::BlendFactor::SrcAlpha;bs.color.dstFactor=blend?wgpu::BlendFactor::One:wgpu::BlendFactor::OneMinusSrcAlpha;
    bs.alpha.srcFactor=wgpu::BlendFactor::Zero;bs.alpha.dstFactor=wgpu::BlendFactor::One;
    wgpu::ColorTargetState colors[aurora::gfx::MaxColorAttachments]{};
    for(unsigned i=0;i<target.colorAttachmentCount;++i){colors[i].format=target.colorAttachments[i].format;colors[i].writeMask=wgpu::ColorWriteMask::None;}
    colors[0].blend=&bs;colors[0].writeMask=wgpu::ColorWriteMask::Red|wgpu::ColorWriteMask::Green|wgpu::ColorWriteMask::Blue;
    wgpu::FragmentState fs{};fs.module=ribbon_module;fs.entryPoint="fs";fs.targetCount=target.colorAttachmentCount;fs.targets=colors;
    wgpu::RenderPipelineDescriptor p{};p.label="motion/ribbon-pipeline";p.layout=ribbon_layout;
    p.vertex.module=ribbon_module;p.vertex.entryPoint="vs";p.fragment=&fs;p.primitive.topology=wgpu::PrimitiveTopology::TriangleList;
    p.multisample.count=target.sampleCount;
    wgpu::DepthStencilState ds{};
    if(target.depthStencilFormat!=wgpu::TextureFormat::Undefined){ds.format=target.depthStencilFormat;
        ds.depthWriteEnabled=wgpu::OptionalBool::False;
        ds.depthCompare=!depth?wgpu::CompareFunction::Always:aurora::gfx::uses_reversed_z()?wgpu::CompareFunction::GreaterEqual:wgpu::CompareFunction::LessEqual;p.depthStencil=&ds;}
    auto result=device.CreateRenderPipeline(&p);if(result)ribbon_pipes.push_back({target.key,blend,depth,result});return result;
}
void ribbon_worker(const aurora::gfx::DrawContext& ctx,const wgpu::RenderPassEncoder& pass,const void* payload,size_t size,void*){
    if(size!=sizeof(RibbonDraw))return;RibbonDraw d;memcpy(&d,payload,size);
    auto pipeline=ribbon_pipeline(ctx.layout,d.blend,d.depth,false);if(!pipeline)return;
    wgpu::BindGroupEntry entries[2]{};
    entries[0].binding=0;entries[0].buffer=ctx.uniformBuffer;entries[0].offset=d.uniform.offset;entries[0].size=d.uniform.size;
    entries[1].binding=1;entries[1].buffer=ctx.storageBuffer;entries[1].offset=d.vertices.offset;entries[1].size=d.vertices.size;
    wgpu::BindGroupDescriptor bg{};bg.layout=ribbon_bgl;bg.entryCount=2;bg.entries=entries;
    auto bind=ctx.device.CreateBindGroup(&bg);if(!bind)return;pass.SetPipeline(pipeline);pass.SetBindGroup(0,bind);pass.Draw(d.count);
}
void ribbon_warm_callback(const void*,uint32_t){
    auto layout=aurora::gfx::scene_render_target_layout();
    for(int blend=0;blend<2;++blend)for(int depth=0;depth<2;++depth)ribbon_pipeline(layout,blend,depth,true);
    if(!ribbon_type){aurora::gfx::DrawTypeDescriptor d{};d.label="motion/ribbons";d.draw=ribbon_worker;ribbon_type=aurora::gfx::register_draw_type(d);}
}
void build_ribbon(const History<Point,GW_MOTION_HISTORY>& h,int frame,const GwMotionOptions& o,float intensity,const Matrix& view,int element,std::vector<RibbonVertex>& out){
    struct Edge {Point left,right;float age;};std::vector<Edge> edges;
    float tint[4];memcpy(tint,o.tint,16);int style=o.shader;
    if(o.anchor==GW_ANCHOR_HITS){
        // Retail element encodings, not rules inferred from action names.
        if(element==1){tint[0]=1;tint[1]=0.3f;tint[2]=0.05f;style=2;}
        else if(element==2){tint[0]=0.4f;tint[1]=0.75f;tint[2]=1;style=3;}
        else if(element==5){tint[0]=0.65f;tint[1]=0.85f;tint[2]=1;style=4;}
        else if(element==13){tint[0]=0.45f;tint[1]=0.1f;tint[2]=0.6f;style=5;}
    }
    (void)style;
    Point forward{view[8],view[9],view[10]};
    auto point=[&](int f)->const Point*{return h.at(f);};
    for(int age=o.length-2;age>=0;--age){
        int f=frame-age-1;auto b=point(f),c=point(f+1);if(!b||!c)continue;
        auto a=point(f-1),d=point(f+2);if(!a)a=b;if(!d)d=c;
        for(int n=0;n<=o.smoothing;++n){
            float t=float(n)/o.smoothing;Point p=catmull(*a,*b,*c,*d,t),q=catmull(*a,*b,*c,*d,std::min(t+0.01f,1.f));
            Point tangent=t>=1?p-catmull(*a,*b,*c,*d,0.99f):q-p;
            Point side{tangent.y*forward.z-tangent.z*forward.y,tangent.z*forward.x-tangent.x*forward.z,tangent.x*forward.y-tangent.y*forward.x};
            float len=std::sqrt(distance2(side,{}));if(len<1e-6f)continue;
            float normalized_age=(age+1-t)/o.length;
            side=side*(o.width*0.5f*width_profile(normalized_age,o.taper,o.swell)/len);
            edges.push_back({p-side,p+side,normalized_age});
        }
    }
    const float seconds=float(frame)/60.f; // logic-frame clock: deterministic, never wall time
    auto vertex=[&](Point p,float age,float across){
        RibbonVertex v{};v.position[0]=p.x;v.position[1]=p.y;v.position[2]=p.z;v.position[3]=1;
        if(o.gradient_count>=2)gradient_sample(o.gradient,o.gradient_count,age,v.color);
        else for(int k=0;k<4;++k)v.color[k]=tint[k]*(1-age)+o.tail[k]*age;
        if(o.hue_drift!=0||o.hue_span!=0)hue_rotate(v.color,o.hue_drift*seconds+o.hue_span*age);
        v.color[3]*=pulse_factor(o.pulse[0],o.pulse[1],seconds,age);
        v.color[3]*=intensity*o.intensity;v.uv[0]=1-age;v.uv[1]=across;return v;
    };
    for(size_t i=1;i<edges.size();++i){
        auto a=edges[i-1],b=edges[i];
        if(distance2(a.left,b.left)>2500)continue;
        out.push_back(vertex(a.left,a.age,0));out.push_back(vertex(a.right,a.age,1));out.push_back(vertex(b.left,b.age,0));
        out.push_back(vertex(b.left,b.age,0));out.push_back(vertex(a.right,a.age,1));out.push_back(vertex(b.right,b.age,1));
    }
}
void record_ribbon(const std::vector<RibbonVertex>& vertices,const GwMotionOptions& o,const Matrix& view,int frame){
    if(vertices.empty() || o.intensity<=0)return;
    if(!aurora::gx::motion::arena_room(0,0,sizeof(RibbonUniform)+256,vertices.size()*sizeof(RibbonVertex)+256)){++skipped;return;}
    auto target=aurora::gfx::scene_render_target_layout();
    if(!ribbon_type || !ribbon_pipeline(target,o.blend,o.depth,false)){++skipped;return;}
    RibbonUniform u{};memcpy(u.view,view.data(),48);aurora::gx::motion::projection(u.proj);
    memcpy(u.edge,o.edge,16);memcpy(u.params,o.params,16);u.params[0]=std::clamp(o.params[0],0.f,1.f);
    u.clock[0]=frame/60.f;u.clock[1]=float(o.shader);
    RibbonDraw d{};d.count=uint32_t(vertices.size());d.blend=o.blend;d.depth=o.depth;
    d.vertices=aurora::gfx::push_storage(reinterpret_cast<const uint8_t*>(vertices.data()),vertices.size()*sizeof(RibbonVertex));
    d.uniform=aurora::gfx::push_uniform(reinterpret_cast<const uint8_t*>(&u),sizeof u);
    if(d.vertices.size && d.uniform.size)aurora::gfx::push_custom_draw(ribbon_type,&d,sizeof d);
}
