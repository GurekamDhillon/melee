-- Read and validate the entire folder before any gameplay writes.
return function(D)
  local V,L=D.validator,{}
  -- Runtime cannot write the mounted mod. Generated data uses an overlay;
  -- models still resolve to real mounted kit files. Ordinary folders are unchanged.
  L.generated={}
  function L.view(g)
    return setmetatable({
      mod_read=function(path)
        for _,doc in pairs(L.generated) do if doc.files[path] then return doc.files[path] end end
        return g.mod_read(path)
      end,
      mod_stamp=function(path)
        for _,doc in pairs(L.generated) do if doc.files[path] then return doc.stamp end end
        return g.mod_stamp(path)
      end,
    },{__index=g})
  end
  function L.data(g,path,optional)
    local text,why=g.mod_read(path)
    if text==nil then
      assert(optional and g.mod_stamp(path)==nil,'cannot read '..path..': '..tostring(why))
      return nil
    end
    local f,err=load(text,'@'..path,'t',{})
    assert(f,err)
    local ok,value=pcall(f);assert(ok,value)
    V.plain(value,path)
    return value
  end
  function L.catalogue(g,folder,shared)
    for name,doc in pairs(L.generated) do
      if folder:sub(1,#name)==name then return doc.catalogue end
    end
    local entries,why=g.mod_list(folder..'models/')
    assert(entries or (shared and next(shared)),'cannot list models: '..tostring(why))
    local out={}
    for _,e in ipairs(entries or {}) do
      if not e.dir and type(e.name)=='string' then
        local name=e.name:match('^(.-)%.gxmesh$')
        if name then V.name(name);out[name]=folder..'models/'..e.name end
      end
    end
    -- Root meshes resolve their sidecars/colour/glow in the shared models folder.
    -- Prefer them over duplicated chunk copies; retain unique chunk-local meshes.
    for name,path in pairs(shared or {}) do out[name]=path end
    return out
  end
  function L.stamps(g,folder)
    g=L.view(g)
    return {g.mod_stamp(folder..'level.lua'),g.mod_stamp(folder..'mission.lua')}
  end
  function L.same(a,b) return a[1]==b[1] and a[2]==b[2] end
  function L.folder(g,name,work)
    g=L.view(g)
    V.name(name)
    local folder='missions/'..name..'/'
    local stamps=L.stamps(g,folder)
    local prepared=L.generated[folder]
    if prepared and prepared.doc and L.same(prepared.doc.stamps,stamps)then return prepared.doc end
    local data=L.data(g,folder..'level.lua')
    if data.maze and D.maze_check then
      local proof=D.maze_check.folder(g,name,work)
      assert(proof.ok,'maze refused: '..table.concat(proof.errors,'; '))
    end
    local catalogue=L.catalogue(g,folder)
    local level=V.layout(data,catalogue,work)
    local raw=L.data(g,folder..'mission.lua',true)
    local mission=raw and D.mission.validate(raw) or level.mission
    assert(mission,'folder needs mission.lua or an embedded mission')
    assert(not D.mission.check_playable(mission),D.mission.check_playable(mission))
    local waves={};for _,e in ipairs(mission.enemies) do waves[e.wave]=true end
    for _,p in ipairs(level.markers) do
      assert(p.checkpoint==nil or mission.checkpoints[p.checkpoint],'marker checkpoint does not exist')
      for _,w in ipairs(p.cleared_waves) do assert(waves[w],'marker cleared wave does not exist') end
    end
    local chunks,ids={},{}
    V.list(data.chunks or {},'chunks',4096)
    for i,c in ipairs(data.chunks or {}) do
      V.plain(c,'chunk');V.name(c.id);assert(not ids[c.id],'duplicate chunk id');ids[c.id]=true
      local rect=V.rect(c.rect,'chunk rectangle');local spawn=V.point(c.spawn,'chunk spawn')
      if chunks[1] then
        local first=chunks[1].rect;local w,h=first.right-first.left,first.top-first.bottom
        local function integral(n) return math.abs(n-math.floor(n+0.5))<0.000001 end
        assert(math.abs(rect.right-rect.left-w)<0.000001 and math.abs(rect.top-rect.bottom-h)<0.000001,
               'chunks must have equal rectangle sizes for a 3x3 grid')
        assert(integral((rect.left-first.left)/w) and integral((rect.bottom-first.bottom)/h),'chunks must align to the grid')
      end
      assert(spawn.x>=rect.left and spawn.x<rect.right and spawn.y>=rect.bottom and spawn.y<rect.top,'spawn outside chunk')
      for _,old in ipairs(chunks) do
        local b=old.rect
        assert(not (rect.left<b.right and rect.right>b.left and rect.bottom<b.top and rect.top>b.bottom),'overlapping chunks')
      end
      local path=folder..'chunks/'..c.id..'/'
      local child=V.layout(L.data(g,path..'level.lua'),L.catalogue(g,path,catalogue),work)
      assert(not child.mission,'chunk mission belongs in the root mission.lua')
      local camera={}
      for k,v in pairs(level.camera_settings or {}) do camera[k]=v end
      for k,v in pairs(child.camera_settings or {}) do camera[k]=v end
      V.camera(camera) -- catch conflicting inherited parameters even in unloaded chunks

      chunks[i]={id=c.id,serial=i,rect=rect,spawn=spawn,level=child}
      if work then work('validate child')end
    end
    assert(L.same(stamps,L.stamps(g,folder)),'folder changed while reading; retry export')
    local doc={name=name,folder=folder,level=level,mission=mission,catalogue=catalogue,chunks=chunks,stamps=stamps,maze=data.maze}
    if D.zones then D.zones.prepare(doc,work)end
    return doc
  end
  return L
end
