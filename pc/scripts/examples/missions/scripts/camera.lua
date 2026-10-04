-- Standard native camera with Lua-owned origin interpolation and fixed world KO edges.
return function(D)
  local C={}
  local EPS=0.01 -- game-unit tolerance exceeds native projection rounding jitter
  local function near(a,b) return a~=nil and b~=nil and math.abs(a-b)<=EPS end
  local function clamp(v,lo,hi)
    if lo>hi then return (lo+hi)/2 end
    return math.max(lo,math.min(hi,v))
  end
  local function leash(origin,target,size) return clamp(origin,target-size,target+size) end
  function C.settings(c)
    local settings={mode='follow'}
    for k,v in pairs(c.doc.level.camera_settings or {}) do settings[k]=v end
    local chunk=c.stream.current
    for k,v in pairs(chunk and chunk.level.camera_settings or {}) do settings[k]=v end
    return settings,chunk
  end
  function C.ends(c,config)
    local ends=config.ends or c.doc.level.camera
    if not ends and #c.doc.chunks>0 then
      ends={left=math.huge,right=-math.huge,bottom=math.huge,top=-math.huge}
      for _,v in ipairs(c.doc.chunks) do
        ends.left=math.min(ends.left,v.rect.left);ends.right=math.max(ends.right,v.rect.right)
        ends.bottom=math.min(ends.bottom,v.rect.bottom);ends.top=math.max(ends.top,v.rect.top)
      end
    end
    return ends
  end
  function C.aspect(g,pose)
    local area=g.safe_area and g.safe_area() or {w=640,h=480}
    local aspect=area.w/area.h
    if g.project and pose then
      local e,t=pose.eye,pose.interest
      local fx,fy,fz=t.x-e.x,t.y-e.y,t.z-e.z
      local n=math.sqrt(fx*fx+fy*fy+fz*fz);local r=math.sqrt(fx*fx+fz*fz)
      if n>0 and r>0 then
        local rx,rz=-fz/r,fx/r
        local ux,uy,uz=-rz*fy/n,(rz*fx-rx*fz)/n,rx*fy/n
        local x,y=g.project(t.x,t.y,t.z)
        local a,b=g.project(t.x+rx,t.y,t.z+rz)
        local u,v=g.project(t.x+ux,t.y+uy,t.z+uz)
        if x and a and u then
          local horizontal=math.sqrt(((a-x)/area.w)^2+((u-x)/area.w)^2)
          local vertical=math.sqrt(((b-y)/area.h)^2+((v-y)/area.h)^2)
          if horizontal>0 and vertical>0 then aspect=vertical/horizontal end
        end
      end
    end
    assert(aspect>0 and aspect<100,'invalid live camera aspect')
    return aspect,area
  end
  function C.footprint(g,pose,area)
    if not g.project or not pose then return end
    local cx,cy=pose.interest.x,pose.interest.y
    local u,v,_,depth=g.project(cx,cy,0)
    local ux,vx,_,dx=g.project(cx+1,cy,0)
    local uy,vy,_,dy=g.project(cx,cy+1,0)
    if not depth or not dx or not dy or depth<=0 then return end
    -- Screen*depth and depth are affine on z=0: invert their exact homography.
    local a0,b0=u*depth,v*depth
    local a1,a2=ux*dx-a0,uy*dy-a0;local b1,b2=vx*dx-b0,vy*dy-b0
    local d1,d2=dx-depth,dy-depth
    local out={left=math.huge,right=-math.huge,bottom=math.huge,top=-math.huge}
    for _,screen in ipairs({{0,0},{area.w,0},{area.w,area.h},{0,area.h}}) do
      local x,y=screen[1],screen[2]
      local A,B,Cc,Dd=a1-x*d1,a2-x*d2,b1-y*d1,b2-y*d2
      local U,V=x*depth-a0,y*depth-b0;local det=A*Dd-B*Cc
      if math.abs(det)<0.00000001 then return end
      local wx,wy=cx+(U*Dd-B*V)/det,cy+(A*V-U*Cc)/det
      out.left=math.min(out.left,wx);out.right=math.max(out.right,wx)
      out.bottom=math.min(out.bottom,wy);out.top=math.max(out.top,wy)
    end
    return out
  end
  function C.release(g,force)
    if C.cut_owned and (force or not g.frame or C.cut_frame~=g.frame()) then
      g.camera_attach(0);C.cut_owned=nil;C.cut_frame=nil
    end
  end
  function C.cut(g,pose)
    assert(g.camera_set and g.camera_attach,'mission camera cut requires camera_set/camera_attach')
    g.camera_set(pose);C.cut_owned=true;C.cut_frame=g.frame and g.frame()
  end
  function C.geometry(config,chunk,ends)
    local mode=config.mode;local aspect=config.aspect or 16/9
    local w,h=250,180
    if mode=='shaft' then w,h=150,260
    elseif mode=='chunk' then w,h=138,112 end -- 130x104 room plus 4 per side
    if config.window then w,h=config.window.w,config.window.h end
    if mode=='chunk' and chunk then
      local b=chunk.rect;local margin=config.margin or 4
      w=b.right-b.left+2*margin;h=b.top-b.bottom+2*margin
    end
    local fov=config.fov or config.tilt or (mode=='shaft' and 20 or 25)
    local half=math.tan(math.rad(fov)/2)
    -- Follow fits its desired window; rooms/shafts fit height, accepting neighbours.
    local distance=h/(2*half)
    if mode=='follow' then distance=math.min(distance,w/(2*aspect*half)) end
    distance=config.min_dist or distance
    local depth=config.max_depth or distance
    assert(distance>=1 and depth>=distance and depth<=49000,'camera footprint cannot fit level at legal distance')
    local params={min_dist=distance,max_depth=depth,fov=fov,fixed_zoom=config.fixed_zoom or 1,
      yaw_gain=config.yaw_gain or 0,pitch_gain=config.pitch_gain or 0,pan=config.pan or 0,
      track_smooth=config.track_smooth or 1.8,track_ratio=config.track_ratio or 1}
    return params,2*depth*half*aspect,2*depth*half
  end
  function C.move(g,c,state,x,y)
    if state.compensated and near(x,state.x) and near(y,state.y) then return end
    local before=g.stage_bounds();local origin=before.origin or {x=state.x,y=state.y}
    local blast=c.doc.level.blast or state.blast or before.blast
    local ok,why=pcall(function()
      -- No native tween: its intermediate origins cannot share our compensation.
      assert(g.stage_set_origin(x,y,{frames=0}))
      assert(g.stage_set_blast_bounds(blast.left-x,blast.right-x,blast.top-y,blast.bottom-y))
    end)
    if not ok then
      pcall(g.stage_set_origin,origin.x,origin.y,{frames=0})
      local b=before.blast
      if b then pcall(g.stage_set_blast_bounds,b.left-origin.x,b.right-origin.x,b.top-origin.y,b.bottom-origin.y) end
      error(why,0)
    end
    state.x=x;state.y=y;state.blast=blast;state.compensated=true
  end
  function C.tick(g,c,p)
    assert(g.camera_params and g.stage_set_origin,'mission camera requires engine batch-2 camera APIs')
    C.release(g) -- one native update has consumed the previous committed cut
    local pose=g.camera_get and g.camera_get()
    local aspect,area
    if C.cut_owned then aspect=C.cached_aspect;area=g.safe_area()
    else aspect,area=C.aspect(g,pose);C.cached_aspect=aspect end
    local footprint=not C.cut_owned and C.footprint(g,pose,area)
    local config,chunk=C.settings(c);local mode=config.mode;local membership=c.membership
    if mode=='chunk' and membership and membership.outside then mode='follow';config.mode='chunk';config.look_ahead=0;config.velocity_lead=0 end
    config.aspect=aspect
    local ends=C.ends(c,config);local params,w,h=C.geometry(config,chunk,ends)
    local state=c.camera
    if not state then
      local origin=g.stage_bounds().origin or {x=0,y=0}
      state={x=origin.x,y=origin.y};c.camera=state
    end
    if state.aspect and math.abs(aspect-state.aspect)<=0.0001 then
      config.aspect=state.aspect;params,w,h=C.geometry(config,chunk,ends)
    else state.aspect=aspect end
    local changed=state.mode~=mode or not near(state.w,w) or not near(state.h,h)
    local params_changed=not state.params
    for k,v in pairs(params) do if not state.params or not near(state.params[k],v) then params_changed=true end end
    changed=changed or params_changed
    if changed then
      if params_changed then g.camera_params(params) end
      -- A tiny target rectangle centres the retail frustum, independent of its aspect.
      if not state.started then assert(g.stage_set_camera_bounds(-0.5,0.5,0.5,-0.5)) end
      state.mode=mode;state.w=w;state.h=h;state.params=params
    end
    local action=p and p.action;local falls=p and p.falls
    local respawning=not p or (action and action<=13)
    if not p or (action and action<12) then
      local at=chunk and chunk.spawn or c.run.respawn or c.doc.mission.start
      p={x=at.x,y=at.y}
    end
    local vx,vy=p.vx or 0,p.vy or 0
    local direction=vx~=0 and (vx>0 and 1 or -1) or (p.facing or 0)
    local lead=clamp(direction*(config.look_ahead or 18)+vx*(config.velocity_lead or 6),
      -(config.max_lead or 40),config.max_lead or 40)
    local vertical=clamp(vy*(config.vertical_lead or (mode=='shaft' and 6 or 3)),
      -(config.vertical_max or (mode=='shaft' and 48 or 24)),config.vertical_max or (mode=='shaft' and 48 or 24))
    if respawning then lead,vertical=0,0 end
    local event=not state.started or (respawning and not state.respawning) or
      (falls~=nil and state.falls~=nil and falls~=state.falls and not state.respawning) or
      (state.px and (math.abs(p.x-state.px)>w or math.abs(p.y-state.py)>h))
    if membership and (membership.outside or state.outside)then event=false end
    local returning=membership and state.outside and not membership.outside
    state.outside=membership and membership.outside
    local snap=event or respawning
    state.respawning=respawning;state.falls=falls or state.falls;state.px=p.x;state.py=p.y
    local x,y
    if mode=='chunk' and chunk then
      local b=chunk.rect;local tx,ty=(b.left+b.right)/2,(b.bottom+b.top)/2
      local tuning=membership and membership.tuning or D.zones.defaults
      if state.room~=chunk then
        state.room=chunk;state.tx=tx;state.ty=ty;state.ease={x=state.x,y=state.y,age=0,frames=tuning.ease_frames,curve=tuning.curve}
      end
      local ease=state.ease
      if snap and not (membership and membership.outside)then x,y=tx,ty;state.ease=nil
      elseif ease then
        ease.age=math.min(ease.frames,ease.age+1);local t=ease.age/ease.frames
        if ease.curve=='smoothstep' then t=t*t*(3-2*t)end
        x=ease.x+(tx-ease.x)*t;y=ease.y+(ty-ease.y)*t
        if ease.age==ease.frames then state.ease=nil end
      else x,y=tx,ty end
      if membership and membership.transition then
        -- Preserve the current framing unless visibility requires a small shift.
        local door=membership.transition;local loX,hiX,loY,hiY=tx,tx,ty,ty
        for _,id in ipairs(door.rooms)do local room=c.doc.zone_data.chunks[id];local r=room.rect
          loX=math.min(loX,(r.left+r.right)/2);hiX=math.max(hiX,(r.left+r.right)/2)
          loY=math.min(loY,(r.bottom+r.top)/2);hiY=math.max(hiY,(r.bottom+r.top)/2)
        end
        local safe=math.min(tuning.doorway_margin,w/4,h/4)
        x=clamp(ease and x or state.x,p.x-w/2+safe,p.x+w/2-safe);y=clamp(ease and y or state.y,p.y-h/2+safe,p.y+h/2-safe)
        x=clamp(x,loX,hiX);y=clamp(y,loY,hiY)
      end
    else
      local lx=config.leash or 10;local ly=config.vertical_leash or (mode=='shaft' and 4 or lx)
      x=snap and p.x+lead or leash(state.x,p.x+lead,lx)
      y=snap and p.y+vertical or leash(state.y,p.y+vertical,ly)
      if mode=='shaft' then x=config.x or (chunk and (chunk.rect.left+chunk.rect.right)/2) or 0 end
    end
    local safe=math.min(config.safe_margin or 12,w/4,h/4)
    if mode~='shaft' then x=clamp(x,p.x-w/2+safe,p.x+w/2-safe) end
    y=clamp(y,p.y-h/2+safe,p.y+h/2-safe)
    if ends then
      x=clamp(x,ends.left+w/2,ends.right-w/2)
      y=clamp(y,ends.bottom+h/2,ends.top-h/2)
    end
    if membership and membership.outside then
      -- Bounded follow even after a debug teleport: no out-of-zone camera cut.
      x=clamp(x,state.x-12,state.x+12);y=clamp(y,state.y-12,state.y+12)
    end
    local moved=not near(x,state.x) or not near(y,state.y)
    C.move(g,c,state,x,y);state.started=true
    if pose and event and (math.abs(pose.interest.x-x)>w or math.abs(pose.interest.y-y)>h) then
      C.cut(g,{eye={x=x,y=y,z=params.min_dist},interest={x=x,y=y,z=0},fov=params.fov,roll=0})
    elseif footprint and ends and not returning and not (membership and membership.outside) and (event or moved or changed) then
      local fw,fh=footprint.right-footprint.left,footprint.top-footprint.bottom
      local cx,cy=(footprint.left+footprint.right)/2,(footprint.bottom+footprint.top)/2
      local dx=clamp(cx,ends.left+fw/2,ends.right-fw/2)-cx
      local dy=clamp(cy,ends.bottom+fh/2,ends.top-fh/2)-cy
      if math.abs(dx)>EPS or math.abs(dy)>EPS then
        local e,t=pose.eye,pose.interest
        C.cut(g,{eye={x=e.x+dx,y=e.y+dy,z=e.z},interest={x=t.x+dx,y=t.y+dy,z=t.z},fov=pose.fov,roll=pose.roll or 0})
      end
    end
  end
  function C.stop(g) if C.cut_owned then pcall(C.release,g,true) end;if g.camera_params then pcall(g.camera_params,nil) end end
  return C
end
