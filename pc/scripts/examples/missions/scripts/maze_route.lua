-- Deterministic bounded A* through unused grid cells.
return function()
  local R={}
  function R.find(a,z,blocked,bounds,work)
    local function key(x,y)return x..','..y end
    local heap,serial={},0
    local function less(a,b)return a.score<b.score or(a.score==b.score and a.order<b.order)end
    local function push(n)
      serial=serial+1;n.order=serial;heap[#heap+1]=n;local i=#heap
      while i>1 do local p=i//2;if not less(heap[i],heap[p])then break end;heap[i],heap[p]=heap[p],heap[i];i=p end
    end
    local function pop()
      local n=heap[1];heap[1]=heap[#heap];heap[#heap]=nil;local i=1
      while heap[i]do local c=i*2;if not heap[c]then break end;if heap[c+1]and less(heap[c+1],heap[c])then c=c+1 end
        if not less(heap[c],heap[i])then break end;heap[i],heap[c]=heap[c],heap[i];i=c
      end;return n
    end
    local start=key(a.x,a.y);local best={[start]=0};local previous={};local points={[start]=a}
    push({x=a.x,y=a.y,cost=0,score=math.abs(z.x-a.x)+math.abs(z.y-a.y)})
    local steps=0
    while #heap>0 do
      local n=pop();local id=key(n.x,n.y)
      if n.x==z.x and n.y==z.y then
        local path={};while id do table.insert(path,1,points[id]);id=previous[id]end;return path
      end
      if n.cost==best[id]then for _,d in ipairs({{1,0},{0,1},{-1,0},{0,-1}})do local x,y=n.x+d[1],n.y+d[2];local k=key(x,y)
        if x>=bounds.left and x<=bounds.right and y>=bounds.bottom and y<=bounds.top and not blocked(x,y)and(best[k]==nil or n.cost+1<best[k])then
          best[k]=n.cost+1;previous[k]=id;points[k]={x=x,y=y};push({x=x,y=y,cost=n.cost+1,score=n.cost+1+math.abs(z.x-x)+math.abs(z.y-y)})
        end
      end end
      steps=steps+1;if steps%64==0 and work then work('route')end
      assert(steps<=50000,'connector routing search limit')
    end
    error('connector cannot be routed without overlap')
  end
  return R
end
