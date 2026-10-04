-- Rules re-derived from the Chao research description; no source/data copied.
return function()
  local G={}
  G.grades={'E','D','C','B','A','S'}
  G.rank={};for i,g in ipairs(G.grades) do G.rank[g]=i end
  G.stats={'power','speed','guard','jump'}
  G.traits={'power','speed','guard','jump','colour','two_tone','shiny'}
  G.colours={normal=true,red=true,green=true,blue=true,yellow=true,white=true,
             black=true,purple=true,orange=true,pink=true,cyan=true}
  function G.random(rng)
    local n=(rng or math.random)()
    assert(type(n)=='number' and n==n and n>=0 and n<1,'random source must return [0,1)')
    return n
  end
  local function choose(pair,rng) return pair[G.random(rng)<.5 and 1 or 2] end
  function G.validate(d)
    assert(type(d)=='table' and getmetatable(d)==nil,'DNA must be plain')
    local known={};for _,k in ipairs(G.traits) do known[k]=true end
    for k in pairs(d) do assert(known[k],'unknown DNA trait') end
    for _,k in ipairs(G.traits) do
      local p=d[k];assert(type(p)=='table' and getmetatable(p)==nil and #p==2,'two alleles required: '..k)
      for i,v in pairs(p) do
        assert(i==1 or i==2,'unexpected allele')
        if G.rank[k] then error('invalid trait') end
        if k=='colour' then assert(G.colours[v],'unknown colour allele')
        elseif k=='two_tone' or k=='shiny' then assert(type(v)=='boolean','boolean allele required')
        else assert(G.rank[v],'grade allele E..S required') end
      end
    end
    return d
  end
  function G.new(grade)
    grade=grade or 'C';assert(G.rank[grade],'grade E..S required')
    local d={colour={'normal','normal'},two_tone={false,false},shiny={false,false}}
    for _,k in ipairs(G.stats) do d[k]={grade,grade} end
    return d
  end
  function G.blend(a,b,rng)
    G.validate(a);G.validate(b);local child={}
    for _,k in ipairs(G.traits) do child[k]={choose(a[k],rng),choose(b[k],rng)} end
    return child
  end
  function G.express(d,rng)
    G.validate(d);local e={}
    for _,k in ipairs(G.stats) do
      local a,b=d[k][1],d[k][2]
      local hi,lo=a,b;if G.rank[a]<G.rank[b] then hi,lo=b,a end
      e[k]=G.random(rng)<.7 and hi or lo
    end
    local a,b=d.colour[1],d.colour[2]
    if a=='normal' then e.colour=b elseif b=='normal' then e.colour=a else e.colour=choose(d.colour,rng) end
    -- Envoy's two-tone trait is the inverse description of monotone; both
    -- express by choosing an allele, rather than inventing dominance.
    e.two_tone=choose(d.two_tone,rng);e.shiny=d.shiny[1] or d.shiny[2]
    return e
  end
  return G
end
