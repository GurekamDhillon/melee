// Included in resource_cache.cpp's aurora::gfx namespace.
void motion_pin_binding(BindGroupRef id,const wgpu::BindGroup& binding) {
    if(!id || !binding)return;
    std::lock_guard lock{g_bindGroupCacheMutex};
    g_cachedBindGroups.insert_or_assign(id,CachedBindGroup{binding,current_frame()});
}
