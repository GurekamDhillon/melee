// Included in recording.cpp's aurora::gfx namespace. Copy while this frame owns
// the arena: no Range, pointer or FramePacket is retained across frames.
bool motion_arena_view(unsigned kind,Range range,const uint8_t*& out) {
    auto& f=current_frame_packet();
    const ByteBuffer* buffers[]={&f.verts,&f.indices,&f.uniforms,&f.storage};
    out=nullptr;
    if(kind>=4)return false;
    const auto& b=*buffers[kind];
    if(range.offset>b.size() || range.size>b.size()-range.offset)return false;
    if(range.size)out=b.data()+range.offset;
    return true;
}
bool motion_arena_room(size_t v,size_t i,size_t u,size_t s) {
    const auto& f=current_frame_packet();
    // Preserve half of remaining space for the live fighter, effects and HUD.
    return v<=(VertexBufferSize-f.verts.size())/2 &&
           i<=(IndexBufferSize-f.indices.size())/2 &&
           u<=(UniformBufferSize-gx::MaxUniformSize-std::min<size_t>(f.uniforms.size(),UniformBufferSize-gx::MaxUniformSize))/2 &&
           s<=(StorageBufferSize-f.storage.size())/2;
}
