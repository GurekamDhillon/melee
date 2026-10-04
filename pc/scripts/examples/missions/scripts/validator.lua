-- Layout normalization: same v1/v2 part, bounds and spawn rules as the editor.
return function(D)
  local V = {}
  function V.number(n)
    return type(n)=='number' and n==n and math.abs(n)<=100000
  end
  function V.plain(t, what)
    assert(type(t)=='table' and getmetatable(t)==nil, what..' must be a plain table')
  end
  function V.list(t, what, max)
    V.plain(t,what)
    local n=0
    for k in pairs(t) do
      assert(type(k)=='number' and k%1==0 and k>=1,what..' must be a dense list')
      n=n+1
    end
    assert(n==#t and n<=max,what..' sparse or oversized')
  end
  function V.name(s)
    assert(type(s)=='string' and #s<=64 and s:match('^[%w_-]+$'),'use a plain folder or part name')
    return s
  end
  function V.point(p,what)
    V.plain(p,what)
    assert(V.number(p.x) and V.number(p.y),what..' needs finite x and y')
    return {x=p.x,y=p.y}
  end
  function V.rect(b,what)
    V.plain(b,what)
    for _,k in ipairs({'left','right','top','bottom'}) do assert(V.number(b[k]),'invalid '..what..' '..k) end
    assert(b.left<b.right and b.bottom<b.top,what..' requires left < right and bottom < top')
    return {left=b.left,right=b.right,top=b.top,bottom=b.bottom}
  end
  function V.camera(data)
    V.plain(data,'camera settings')
    local q={};local allowed={mode=true,window=true,ends=true,x=true,margin=true,frames=true,
      min_dist=true,max_depth=true,fov=true,tilt=true,fixed_zoom=true,track_smooth=true,track_ratio=true,
      yaw_gain=true,pitch_gain=true,pan=true,aspect=true,leash=true,vertical_leash=true,look_ahead=true,
      velocity_lead=true,max_lead=true,vertical_lead=true,vertical_max=true,door_margin=true,safe_margin=true,commit_distance=true,vertical_commit_distance=true,
      ease_frames=true,curve=true,doorway_margin=true,doors=true}
    for k,v in pairs(data) do assert(allowed[k],'unknown camera setting '..tostring(k));q[k]=v end
    assert(q.mode==nil or q.mode=='follow' or q.mode=='chunk' or q.mode=='shaft','invalid camera mode')
    if q.window~=nil then
      V.plain(q.window,'camera window')
      assert(V.number(q.window.w) and q.window.w>0 and q.window.w<=49000 and
        V.number(q.window.h) and q.window.h>0 and q.window.h<=49000,'invalid camera window')
      q.window={w=q.window.w,h=q.window.h}
    end
    if q.ends~=nil then q.ends=V.rect(q.ends,'camera ends') end
    assert(q.curve==nil or q.curve=='linear' or q.curve=='smoothstep','invalid camera curve')
    if q.doors then
      V.plain(q.doors,'camera doors');local doors={}
      for name,v in pairs(q.doors)do V.name(name);doors[name]=V.door_camera(v)end
      q.doors=doors
    end
    local ranges={min_dist={1,49000},max_depth={1,49000},fov={1,89},tilt={1,89},
      fixed_zoom={0,10},track_smooth={0,10},track_ratio={0,10},yaw_gain={0,10},pitch_gain={0,10},
      pan={-180,180},x={-49000,49000},margin={0,100},frames={30,60},aspect={0.5,4},leash={0,100},vertical_leash={0,100},look_ahead={0,100},
      velocity_lead={0,30},max_lead={0,200},vertical_lead={0,30},vertical_max={0,200},
      door_margin={0,100},safe_margin={0,100},commit_distance={1,32},vertical_commit_distance={1,40},ease_frames={1,120},doorway_margin={0,40}}
    for k,b in pairs(ranges) do if q[k]~=nil then
      assert(V.number(q[k]) and q[k]>=b[1] and q[k]<=b[2],'invalid camera '..k)
    end end
    assert(not q.frames or q.frames%1==0,'camera frames must be integer')
    assert(not q.ease_frames or q.ease_frames%1==0,'camera ease_frames must be integer')
    assert(not (q.min_dist and q.max_depth) or q.min_dist<=q.max_depth,'camera min_dist exceeds max_depth')
    assert(not (q.fov and q.tilt) or q.fov==q.tilt,'camera fov and tilt differ')
    return q
  end
  function V.door_camera(data)
    V.plain(data,'door camera')
    local allowed={commit_distance=true,vertical_commit_distance=true,ease_frames=true,curve=true,doorway_margin=true}
    for k in pairs(data)do assert(allowed[k],'invalid door camera field '..tostring(k))end
    return V.camera(data)
  end
  function V.layout(data, catalogue, work)
    V.plain(data,'layout')
    assert(data.version==1 or data.version==2,'layout version must be 1 or 2')
    assert(data.units==6.5,'kit scale differs; units must be 6.5')
    local out={parts={},spawn={},lines={},markers={},fighters={}}
    if data.autostart~=nil then assert(type(data.autostart)=='boolean','autostart must be boolean');out.autostart=data.autostart end
    if data.warm_items~=nil then
      V.list(data.warm_items,'warm items',32);out.warm_items={}
      for i,k in ipairs(data.warm_items)do assert(type(k)=='string'and #k<=127 or type(k)=='number'and k>=0 and k<2147483648 and k%1==0,'invalid warm item');out.warm_items[i]=k end
    end
    if data.starting_percent~=nil then
      assert(V.number(data.starting_percent) and data.starting_percent%1==0 and
        data.starting_percent>=0 and data.starting_percent<=999,'invalid starting percent')
      out.starting_percent=data.starting_percent
    end
    if data.fighters~=nil then
      V.plain(data.fighters,'fighters')
      for port,mode in pairs(data.fighters) do
        assert(type(port)=='number' and port%1==0 and port>=2 and port<=6,'fighter policy ports are 2-6')
        assert(mode=='stand' or mode=='fight','fighter policy must be stand or fight');out.fighters[port]=mode
      end
    end
    if data.mission~=nil then
      assert(data.version==2,'mission requires layout version 2')
      out.mission=D.mission.validate(data.mission)
    end
    if data.version==2 then
      if data.camera~=nil then
        V.plain(data.camera,'camera')
        local cfg={}
        for k,v in pairs(data.camera) do
          if k~='left' and k~='right' and k~='top' and k~='bottom' then cfg[k]=v end
        end
        if data.camera.left~=nil or data.camera.right~=nil or data.camera.top~=nil or data.camera.bottom~=nil then
          out.camera=V.rect(data.camera,'camera')
        end  -- exporter writes rectangle and settings in one table
        if next(cfg)~=nil then out.camera_settings=V.camera(cfg) end
      end
      if data.blast~=nil then out.blast=V.rect(data.blast,'blast') end
      if data.spawn~=nil then
        V.plain(data.spawn,'spawn')
        for slot,p in pairs(data.spawn) do
          assert(type(slot)=='number' and slot%1==0 and ((slot>=0 and slot<=7) or (slot>=127 and slot<=146)),
                 'spawn slots are 0-7 or 127-146')
          out.spawn[slot]=V.point(p,'spawn')
        end
      end
    end
    out.zones={};V.list(data.zones or {},'zones',4096)
    local zone_names={}
    for _,z in ipairs(data.zones or {})do
      V.plain(z,'zone');V.name(z.name);assert(not zone_names[z.name],'duplicate zone '..z.name);zone_names[z.name]=true
      assert(z.kind=='room' or z.kind=='transition' or z.kind=='region','invalid zone kind '..z.name)
      local q={name=z.name,kind=z.kind,rect=V.rect(z.rect,'zone '..z.name)}
      if z.kind=='room' then q.room=V.name(z.room)
      elseif z.kind=='region'then q.region=V.name(z.region)
      else
        V.list(z.rooms,'zone rooms',16);assert(#z.rooms>=2,'transition needs at least two rooms');q.rooms={}
        local ids={};for _,id in ipairs(z.rooms)do V.name(id);assert(not ids[id],'duplicate zone room');ids[id]=true;q.rooms[#q.rooms+1]=id end
        if z.camera then q.camera=V.door_camera(z.camera)end
      end
      out.zones[#out.zones+1]=q;if work then work('normalize zone')end
    end
    V.list(data.parts,'parts',128)
    for i,p in ipairs(data.parts) do
      V.plain(p,'part'); V.name(p.part)
      assert(catalogue[p.part],'unknown folder part: '..p.part)
      local label=p.name or p.label or p.part
      assert(type(label)=='string' and #label>=1 and #label<=80 and not label:find('%c'),'invalid part label')
      local q={part=p.part,path=catalogue[p.part],label=label}
      for _,k in ipairs({'x','y','z','rot'}) do assert(V.number(p[k]),'invalid '..k);q[k]=p[k] end
      assert(math.abs(p.rot)<=360,'rotation must be -360..360')
      assert(type(p.collision)=='boolean','collision must be boolean')
      assert(type(p.floor_flags)=='number' and p.floor_flags%1==0 and p.floor_flags>=0 and p.floor_flags<=3,
             'floor_flags must be 0..3')
      q.collision=p.collision;q.floor_flags=p.floor_flags
      for _,k in ipairs({'scale','scale_x','scale_y','scale_z'}) do
        local n=p[k]==nil and 1 or p[k]
        assert(V.number(n) and math.abs(n)>=0.001 and math.abs(n)<=100,'invalid '..k);q[k]=n
      end
      out.parts[i]=q
    end
    V.list(data.lines or {},'lines',768)
    for i,l in ipairs(data.lines or {}) do
      V.plain(l,'line')
      local q={}
      for _,k in ipairs({'x1','y1','x2','y2'}) do assert(V.number(l[k]),'invalid line '..k);q[k]=l[k] end
      assert(l.kind=='floor' or l.kind=='ceiling' or l.kind=='left_wall' or l.kind=='right_wall','invalid line kind')
      assert((l.kind=='floor' and l.x1<l.x2) or (l.kind=='ceiling' and l.x1>l.x2) or
             (l.kind=='left_wall' and l.y1<l.y2) or (l.kind=='right_wall' and l.y1>l.y2),'wrong line direction')
      q.kind=l.kind;q.opts={}
      for _,k in ipairs({'passthrough','ledges','draw'}) do
        assert(l[k]==nil or type(l[k])=='boolean','line '..k..' must be boolean');q.opts[k]=l[k]
      end
      out.lines[i]=q
    end
    V.list(data.markers or {},'markers',512)
    local names={}
    for i,p in ipairs(data.markers or {}) do
      local q=V.point(p,'marker');q.name=V.name(p.name)
      assert(not names[q.name],'duplicate marker name');names[q.name]=true
      V.list(p.cleared_waves or {},'cleared_waves',8)
      q.cleared_waves={}
      for j,w in ipairs(p.cleared_waves or {}) do
        assert(type(w)=='number' and w%1==0 and w>=1 and w<=8,'invalid cleared wave')
        q.cleared_waves[j]=w
      end
      assert(p.checkpoint==nil or (type(p.checkpoint)=='number' and p.checkpoint%1==0 and p.checkpoint>=1),'invalid marker checkpoint')
      assert(p.frames==nil or (V.number(p.frames) and p.frames%1==0 and p.frames>=0),'invalid marker frames')
      q.checkpoint=p.checkpoint;q.frames=p.frames or 0;q.authored=true
      out.markers[i]=q;if work then work('normalize marker')end
    end
    return out
  end
  return V
end
